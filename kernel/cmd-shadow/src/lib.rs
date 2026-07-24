//! # cmd-shadow — the Shadow World Engine
//!
//! Before the agent touches the real machine, fork it. The agent finishes the
//! whole plan inside the fork; you are shown a *finished outcome* and choose
//! whether it becomes reality. Discarding costs nothing, because nothing real
//! was ever changed.
//!
//! Reversibility, dry-run and undo are all special cases of this: a dry-run is a
//! fork you never promote, and an undo is a promotion you decline.
//!
//! # How a fork works
//!
//! A fork is copy-on-write. It starts empty and holds no copy of anything.
//! Reading falls through to reality; the first time a path is *written*, the
//! real file is copied into the fork and the write lands on the copy. So forking
//! a large folder is instant, and the cost is proportional to what actually
//! changed, not to what exists.
//!
//! Several forks can hang off one root at once. That is the point: the agent can
//! finish the same job three different ways, and you pick.
//!
//! This first implementation is scoped to the filesystem. At VM level the same
//! model spans the whole computer — files, browser sessions, open apps — which
//! is RFC-0005's eventual target.
//!
//! Defined by RFC-0005.

use cmd_types::Id;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};

/// What a fork did to one path, relative to reality.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Change {
    /// A path that does not exist in reality was created in the fork.
    Created(String),
    /// A path that exists in reality has different contents in the fork.
    Modified(String),
    /// A path that exists in reality is marked for deletion.
    Deleted(String),
}

impl Change {
    /// The path this change applies to, relative to the fork root.
    pub fn path(&self) -> &str {
        match self {
            Change::Created(p) | Change::Modified(p) | Change::Deleted(p) => p,
        }
    }
}

/// Everything that can go wrong in the shadow world.
#[derive(Debug)]
pub enum ShadowError {
    /// The path escaped the fork root (`..`, absolute paths, symlink tricks).
    Escapes(String),
    /// An underlying filesystem failure.
    Io(String),
}

impl std::fmt::Display for ShadowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShadowError::Escapes(p) => write!(f, "path escapes the shadow root: {p}"),
            ShadowError::Io(m) => write!(f, "shadow io error: {m}"),
        }
    }
}

impl std::error::Error for ShadowError {}

impl From<std::io::Error> for ShadowError {
    fn from(e: std::io::Error) -> Self {
        ShadowError::Io(e.to_string())
    }
}

type Result<T> = std::result::Result<T, ShadowError>;

/// Reject anything that would let a fork write outside its root: absolute paths,
/// parent traversal, or a root prefix. A shadow that can reach reality is not a
/// shadow.
fn safe_relative(rel: &str) -> Result<PathBuf> {
    // Backslashes are separators too. A path produced on Windows and one written
    // into a plan on Linux have to resolve to the same place.
    let normalised = rel.replace('\\', "/");
    let p = Path::new(&normalised);
    if p.is_absolute() || normalised.contains(':') {
        return Err(ShadowError::Escapes(rel.into()));
    }
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            _ => return Err(ShadowError::Escapes(rel.into())),
        }
    }
    if out.as_os_str().is_empty() {
        return Err(ShadowError::Escapes(rel.into()));
    }
    Ok(out)
}

/// A copy-on-write fork of a directory: somewhere the agent can finish a whole
/// plan without reality noticing.
pub struct ShadowFork {
    id: Id,
    /// The real directory this fork shadows.
    root: PathBuf,
    /// Where the fork's own copies live.
    work: PathBuf,
    /// Paths the fork has marked for deletion (they still exist in reality).
    deleted: Vec<String>,
    /// A human-readable name for the outcome this fork represents.
    label: String,
    promoted: bool,
}

impl ShadowFork {
    /// Fork `root`. Nothing is copied: the fork is empty until something is
    /// written, so this is instant regardless of how large `root` is.
    ///
    /// `work` must be a directory the caller owns (a scratch area); it is created
    /// if missing.
    pub fn new(
        root: impl Into<PathBuf>,
        work: impl Into<PathBuf>,
        label: impl Into<String>,
    ) -> Result<Self> {
        let root = root.into();
        let work = work.into();
        fs::create_dir_all(&work)?;
        Ok(ShadowFork {
            id: Id::new(),
            root,
            work,
            deleted: Vec::new(),
            label: label.into(),
            promoted: false,
        })
    }

    /// This fork's identity.
    pub fn id(&self) -> Id {
        self.id
    }

    /// The outcome this fork represents, for the chooser.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// The real directory being shadowed.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Where the fork keeps its copies.
    pub fn work_dir(&self) -> &Path {
        &self.work
    }

    /// Read a path as the fork sees it: the fork's copy if it has one, otherwise
    /// reality. Returns `None` if it exists in neither, or the fork deleted it.
    pub fn read(&self, rel: &str) -> Result<Option<Vec<u8>>> {
        let r = safe_relative(rel)?;
        let key = key_of(&r);
        if self.deleted.contains(&key) {
            return Ok(None);
        }
        let shadow = self.work.join(&r);
        if shadow.exists() {
            return Ok(Some(fs::read(shadow)?));
        }
        let real = self.root.join(&r);
        if real.exists() {
            return Ok(Some(fs::read(real)?));
        }
        Ok(None)
    }

    /// Write inside the fork. Reality is untouched.
    pub fn write(&mut self, rel: &str, contents: &[u8]) -> Result<()> {
        let r = safe_relative(rel)?;
        let target = self.work.join(&r);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(target, contents)?;
        let key = key_of(&r);
        self.deleted.retain(|d| d != &key); // writing un-deletes
        Ok(())
    }

    /// Mark a path deleted in this fork. The real file stays where it is until
    /// (and unless) the fork is promoted.
    pub fn delete(&mut self, rel: &str) -> Result<()> {
        let r = safe_relative(rel)?;
        let key = key_of(&r);
        let shadow = self.work.join(&r);
        if shadow.exists() {
            fs::remove_file(&shadow)?;
        }
        if self.root.join(&r).exists() && !self.deleted.contains(&key) {
            self.deleted.push(key);
        }
        Ok(())
    }

    /// Copy a real file into the fork so it can be edited in place — the
    /// copy-on-write step, done explicitly.
    pub fn materialize(&mut self, rel: &str) -> Result<()> {
        let r = safe_relative(rel)?;
        let shadow = self.work.join(&r);
        if shadow.exists() {
            return Ok(());
        }
        let real = self.root.join(&r);
        if !real.exists() {
            return Ok(());
        }
        if let Some(parent) = shadow.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(real, shadow)?;
        Ok(())
    }

    /// Everything this fork would do to reality if promoted. This is what the
    /// chooser shows: the *outcome*, not the steps that produced it.
    pub fn changes(&self) -> Result<Vec<Change>> {
        let mut out = Vec::new();
        collect(&self.work, &self.work, &self.root, &mut out)?;
        for d in &self.deleted {
            out.push(Change::Deleted(d.clone()));
        }
        out.sort_by(|a, b| a.path().cmp(b.path()));
        Ok(out)
    }

    /// Whether this fork would change anything at all.
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.changes()?.is_empty())
    }

    /// Make this fork real: apply every change to the root directory. After a
    /// promotion the fork is spent.
    pub fn promote(&mut self) -> Result<Vec<Change>> {
        let changes = self.changes()?;
        for change in &changes {
            let rel = safe_relative(change.path())?;
            let real = self.root.join(&rel);
            match change {
                Change::Created(_) | Change::Modified(_) => {
                    if let Some(parent) = real.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(self.work.join(&rel), &real)?;
                }
                Change::Deleted(_) => {
                    if real.exists() {
                        fs::remove_file(&real)?;
                    }
                }
            }
        }
        self.promoted = true;
        Ok(changes)
    }

    /// Throw the fork away. Reality never knew about it, so this is free — and
    /// it is the reason the agent can be allowed to try things.
    pub fn discard(self) -> Result<()> {
        if self.work.exists() {
            fs::remove_dir_all(&self.work)?;
        }
        Ok(())
    }

    /// Whether this fork has been promoted.
    pub fn was_promoted(&self) -> bool {
        self.promoted
    }
}

/// The canonical form of a relative path: forward slashes, always. A `Change`
/// crosses into the UI and back into this engine, so it must not carry the
/// separator of whichever machine happened to produce it.
fn key_of(rel: &Path) -> String {
    rel.components()
        .filter_map(|c| match c {
            Component::Normal(p) => Some(p.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Walk the fork's own files and classify each against reality.
fn collect(dir: &Path, work_root: &Path, real_root: &Path, out: &mut Vec<Change>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect(&path, work_root, real_root, out)?;
            continue;
        }
        let rel = path
            .strip_prefix(work_root)
            .map_err(|_| ShadowError::Escapes(path.to_string_lossy().into()))?;
        let key = key_of(rel);
        let real = real_root.join(rel);
        if !real.exists() {
            out.push(Change::Created(key));
        } else if fs::read(&path)? != fs::read(&real)? {
            out.push(Change::Modified(key));
        }
        // identical contents produce no change at all
    }
    Ok(())
}

/// Several forks of the same root: the futures the user chooses between.
pub struct ShadowWorld {
    root: PathBuf,
    scratch: PathBuf,
    forks: Vec<ShadowFork>,
}

impl ShadowWorld {
    /// A world over `root`, keeping fork data under `scratch`.
    pub fn new(root: impl Into<PathBuf>, scratch: impl Into<PathBuf>) -> Result<Self> {
        let scratch = scratch.into();
        fs::create_dir_all(&scratch)?;
        Ok(ShadowWorld {
            root: root.into(),
            scratch,
            forks: Vec::new(),
        })
    }

    /// Open another candidate future.
    pub fn fork(&mut self, label: impl Into<String>) -> Result<Id> {
        let label = label.into();
        let dir = self.scratch.join(format!("fork-{}", self.forks.len()));
        let fork = ShadowFork::new(self.root.clone(), dir, label)?;
        let id = fork.id();
        self.forks.push(fork);
        Ok(id)
    }

    /// Borrow a fork to work in it.
    pub fn get_mut(&mut self, id: Id) -> Option<&mut ShadowFork> {
        self.forks.iter_mut().find(|f| f.id() == id)
    }

    /// Borrow a fork to inspect it.
    pub fn get(&self, id: Id) -> Option<&ShadowFork> {
        self.forks.iter().find(|f| f.id() == id)
    }

    /// The outcomes on offer: each fork with what it would change.
    pub fn outcomes(&self) -> Result<Vec<Outcome>> {
        self.forks
            .iter()
            .map(|f| {
                Ok(Outcome {
                    id: f.id(),
                    label: f.label().to_string(),
                    changes: f.changes()?,
                })
            })
            .collect()
    }

    /// Choose one future: promote it, and discard every other fork. This is the
    /// whole product in one call — one outcome becomes real, the rest never
    /// happened.
    pub fn choose(&mut self, id: Id) -> Result<Vec<Change>> {
        let idx = self
            .forks
            .iter()
            .position(|f| f.id() == id)
            .ok_or_else(|| ShadowError::Io("no such fork".into()))?;

        let applied = self.forks[idx].promote()?;

        // Everything not chosen is thrown away.
        let rest: Vec<ShadowFork> = self
            .forks
            .drain(..)
            .enumerate()
            .filter_map(|(i, f)| if i == idx { None } else { Some(f) })
            .collect();
        for f in rest {
            f.discard()?;
        }
        self.forks.clear();
        Ok(applied)
    }

    /// Walk away from all of them. Reality is untouched.
    pub fn discard_all(&mut self) -> Result<()> {
        for f in self.forks.drain(..) {
            f.discard()?;
        }
        Ok(())
    }

    /// How many futures are currently open.
    pub fn len(&self) -> usize {
        self.forks.len()
    }

    /// Whether no futures are open.
    pub fn is_empty(&self) -> bool {
        self.forks.is_empty()
    }
}

/// A finished outcome offered to the user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    pub id: Id,
    pub label: String,
    pub changes: Vec<Change>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// A real directory with one file, plus a scratch area for forks.
    fn world() -> (tempfile::TempDir, tempfile::TempDir) {
        let real = tempdir().unwrap();
        fs::write(real.path().join("notes.txt"), b"original").unwrap();
        let scratch = tempdir().unwrap();
        (real, scratch)
    }

    #[test]
    fn forking_copies_nothing_and_changes_nothing() {
        let (real, scratch) = world();
        let fork = ShadowFork::new(real.path(), scratch.path().join("f"), "try it").unwrap();

        // The fork is empty; reality is untouched.
        assert!(fork.changes().unwrap().is_empty());
        assert_eq!(
            fs::read(real.path().join("notes.txt")).unwrap(),
            b"original"
        );
    }

    #[test]
    fn reads_fall_through_to_reality() {
        let (real, scratch) = world();
        let fork = ShadowFork::new(real.path(), scratch.path().join("f"), "read").unwrap();
        assert_eq!(fork.read("notes.txt").unwrap().unwrap(), b"original");
        assert!(fork.read("missing.txt").unwrap().is_none());
    }

    #[test]
    fn writing_in_the_fork_leaves_reality_alone() {
        let (real, scratch) = world();
        let mut fork = ShadowFork::new(real.path(), scratch.path().join("f"), "edit").unwrap();

        fork.write("notes.txt", b"rewritten").unwrap();

        // The fork sees the new text; reality still has the old.
        assert_eq!(fork.read("notes.txt").unwrap().unwrap(), b"rewritten");
        assert_eq!(
            fs::read(real.path().join("notes.txt")).unwrap(),
            b"original"
        );
        assert_eq!(
            fork.changes().unwrap(),
            vec![Change::Modified("notes.txt".into())]
        );
    }

    #[test]
    fn promoting_makes_the_fork_real() {
        let (real, scratch) = world();
        let mut fork = ShadowFork::new(real.path(), scratch.path().join("f"), "edit").unwrap();
        fork.write("notes.txt", b"rewritten").unwrap();
        fork.write("new.txt", b"fresh").unwrap();

        let applied = fork.promote().unwrap();

        assert_eq!(
            fs::read(real.path().join("notes.txt")).unwrap(),
            b"rewritten"
        );
        assert_eq!(fs::read(real.path().join("new.txt")).unwrap(), b"fresh");
        assert!(applied.contains(&Change::Modified("notes.txt".into())));
        assert!(applied.contains(&Change::Created("new.txt".into())));
        assert!(fork.was_promoted());
    }

    #[test]
    fn discarding_is_free() {
        let (real, scratch) = world();
        let mut fork = ShadowFork::new(real.path(), scratch.path().join("f"), "abandon").unwrap();
        fork.write("notes.txt", b"nonsense").unwrap();
        fork.write("junk.txt", b"junk").unwrap();

        fork.discard().unwrap();

        // Nothing the fork did ever reached reality.
        assert_eq!(
            fs::read(real.path().join("notes.txt")).unwrap(),
            b"original"
        );
        assert!(!real.path().join("junk.txt").exists());
    }

    #[test]
    fn deletion_is_staged_not_performed() {
        let (real, scratch) = world();
        let mut fork = ShadowFork::new(real.path(), scratch.path().join("f"), "clean").unwrap();

        fork.delete("notes.txt").unwrap();

        // Gone as far as the fork is concerned; still there in reality.
        assert!(fork.read("notes.txt").unwrap().is_none());
        assert!(real.path().join("notes.txt").exists());
        assert_eq!(
            fork.changes().unwrap(),
            vec![Change::Deleted("notes.txt".into())]
        );

        fork.promote().unwrap();
        assert!(!real.path().join("notes.txt").exists());
    }

    #[test]
    fn an_identical_rewrite_is_not_a_change() {
        let (real, scratch) = world();
        let mut fork = ShadowFork::new(real.path(), scratch.path().join("f"), "noop").unwrap();
        fork.write("notes.txt", b"original").unwrap();
        // Same bytes as reality — the chooser should not offer this as an outcome.
        assert!(fork.changes().unwrap().is_empty());
    }

    #[test]
    fn change_paths_are_reported_the_same_on_every_platform() {
        // A Change travels to the UI and can come back in. It must not carry the
        // separator of whichever machine produced it — this failed on Windows.
        let (real, scratch) = world();
        let mut fork = ShadowFork::new(real.path(), scratch.path().join("f"), "sep").unwrap();
        fork.write("deep/inside/file.txt", b"hi").unwrap();

        let changes = fork.changes().unwrap();
        assert_eq!(
            changes,
            vec![Change::Created("deep/inside/file.txt".into())]
        );
        assert!(
            !changes[0].path().contains('\\'),
            "no backslashes in a change path"
        );
    }

    #[test]
    fn either_separator_names_the_same_file() {
        let (real, scratch) = world();
        let mut fork = ShadowFork::new(real.path(), scratch.path().join("f"), "sep").unwrap();
        fork.write("a/b/c.txt", b"one").unwrap();
        // The Windows spelling of the same path must find it.
        assert_eq!(fork.read("a\\b\\c.txt").unwrap().unwrap(), b"one");
        assert_eq!(fork.changes().unwrap().len(), 1, "still one file, not two");
    }

    #[test]
    fn a_fork_cannot_reach_outside_its_root() {
        let (real, scratch) = world();
        let mut fork = ShadowFork::new(real.path(), scratch.path().join("f"), "hostile").unwrap();

        assert!(matches!(
            fork.write("../escape.txt", b"x"),
            Err(ShadowError::Escapes(_))
        ));
        assert!(matches!(
            fork.write("/etc/passwd", b"x"),
            Err(ShadowError::Escapes(_))
        ));
        assert!(matches!(
            fork.write("C:\\Windows\\system32", b"x"),
            Err(ShadowError::Escapes(_))
        ));
        assert!(matches!(
            fork.write("..\\escape.txt", b"x"),
            Err(ShadowError::Escapes(_))
        ));
        assert!(matches!(
            fork.read("../../secret"),
            Err(ShadowError::Escapes(_))
        ));
    }

    #[test]
    fn nested_paths_work() {
        let (real, scratch) = world();
        let mut fork = ShadowFork::new(real.path(), scratch.path().join("f"), "nested").unwrap();
        fork.write("deep/inside/file.txt", b"hi").unwrap();
        assert_eq!(
            fork.changes().unwrap(),
            vec![Change::Created("deep/inside/file.txt".into())]
        );
        fork.promote().unwrap();
        assert_eq!(
            fs::read(real.path().join("deep/inside/file.txt")).unwrap(),
            b"hi"
        );
    }

    // The product, in one test: the same job finished three ways, one chosen.
    #[test]
    fn choose_your_future() {
        let (real, scratch) = world();
        let mut world = ShadowWorld::new(real.path(), scratch.path()).unwrap();

        let by_type = world.fork("Organised by type").unwrap();
        let by_date = world.fork("Organised by date").unwrap();
        let archived = world.fork("Archive everything").unwrap();

        world
            .get_mut(by_type)
            .unwrap()
            .write("pdf/notes.txt", b"original")
            .unwrap();
        world
            .get_mut(by_date)
            .unwrap()
            .write("2026-07/notes.txt", b"original")
            .unwrap();
        world
            .get_mut(archived)
            .unwrap()
            .write("archive.zip", b"zipped")
            .unwrap();

        // Three finished outcomes are on offer, and reality has none of them.
        let outcomes = world.outcomes().unwrap();
        assert_eq!(outcomes.len(), 3);
        assert!(outcomes.iter().any(|o| o.label == "Organised by date"));
        assert!(!real.path().join("pdf").exists());

        // Pick one. It becomes real; the others never happened.
        world.choose(by_type).unwrap();

        assert!(real.path().join("pdf/notes.txt").exists());
        assert!(!real.path().join("2026-07").exists());
        assert!(!real.path().join("archive.zip").exists());
        assert!(world.is_empty(), "choosing closes the world");
    }

    #[test]
    fn walking_away_leaves_reality_untouched() {
        let (real, scratch) = world();
        let mut world = ShadowWorld::new(real.path(), scratch.path()).unwrap();
        let a = world.fork("one").unwrap();
        world
            .get_mut(a)
            .unwrap()
            .write("notes.txt", b"changed")
            .unwrap();

        world.discard_all().unwrap();

        assert_eq!(
            fs::read(real.path().join("notes.txt")).unwrap(),
            b"original"
        );
        assert!(world.is_empty());
    }
}
