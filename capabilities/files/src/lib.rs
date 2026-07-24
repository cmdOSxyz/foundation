//! # cap-files — the filesystem capability
//!
//! The first Rust component to replace a prototype capability. It implements the
//! [`cmd_transaction::Resource`] trait so every file operation runs through the
//! reversible engine: simulate → snapshot → execute → verify → commit | rollback.
//!
//! Behavior is defined by the prototype reference
//! `prototype/capabilities/filesystem.ts` and its 8 behavior contracts in
//! `prototype/tests/filesystem.behavior.test.ts`. This crate reproduces them.
//!
//! Actions: `list` (R0 read-only), `rename` / `move` (R1 reversible),
//! `delete` (R1 — safe delete to a `.cmdos-trash` folder, recoverable).
//!
//! Defined by RFC-0009.

use cmd_transaction::{Resource, ResourceError, Snapshot};
use cmd_types::PlanStep;
use std::fs;
use std::path::{Path, PathBuf};

/// What a dry-run reports: the effect, whether it can be undone, and warnings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRun {
    pub summary: String,
    pub reversible: bool,
    pub warnings: Vec<String>,
}

/// Snapshot for the filesystem resource: enough to undo the last executed step.
///
/// For rename/move we remember the two paths; for delete we remember where the
/// file was trashed so we can move it back.
#[derive(Clone, Debug)]
pub enum FsSnapshot {
    /// A rename/move from `original` to `moved_to`; undo moves it back.
    Moved {
        original: PathBuf,
        moved_to: PathBuf,
    },
    /// A delete: the file now lives at `trashed_to`; undo restores to `original`.
    Trashed {
        original: PathBuf,
        trashed_to: PathBuf,
    },
    /// Nothing to undo (read-only).
    None,
}
impl Snapshot for FsSnapshot {}

/// The filesystem capability. Stateless except for the last snapshot it produced;
/// the engine owns lifecycle ordering.
#[derive(Default)]
pub struct FileSystem;

impl FileSystem {
    pub fn new() -> Self {
        FileSystem
    }

    /// Preview a step without touching disk. Powers "see before you approve".
    pub fn dry_run(&self, step: &PlanStep) -> DryRun {
        match step.action.as_str() {
            "list" => {
                let path = param(step, "path");
                let exists = Path::new(&path).exists();
                DryRun {
                    summary: if exists {
                        format!("Read-only. Will inspect {path} (nothing changes).")
                    } else {
                        format!("Path not found: {path}")
                    },
                    reversible: true,
                    warnings: if exists {
                        vec![]
                    } else {
                        vec!["Target does not exist".into()]
                    },
                }
            }
            "rename" | "move" => {
                let from = param(step, "from");
                let to = resolve_target(step);
                let mut warnings = vec![];
                if !Path::new(&from).exists() {
                    warnings.push(format!("Source does not exist: {from}"));
                }
                if Path::new(&to).exists() {
                    warnings.push(format!(
                        "Target already exists and may be overwritten: {to}"
                    ));
                }
                DryRun {
                    summary: format!("Will {}: {from}  ->  {to}", step.action),
                    reversible: true,
                    warnings,
                }
            }
            "delete" => {
                let path = param(step, "path");
                let exists = Path::new(&path).exists();
                DryRun {
                    summary: if exists {
                        format!("Will DELETE: {path}")
                    } else {
                        format!("Nothing to delete - not found: {path}")
                    },
                    reversible: true, // safe delete: recoverable from trash
                    warnings: if exists {
                        vec!["Moved to cmdOS trash; can be restored".into()]
                    } else {
                        vec!["Target does not exist".into()]
                    },
                }
            }
            other => DryRun {
                summary: format!("No preview available for {other}"),
                reversible: true,
                warnings: vec![],
            },
        }
    }
}

// --- The reversible Resource implementation ---------------------------------

impl Resource for FileSystem {
    type Snap = FsSnapshot;

    fn simulate(&self, step: &PlanStep) -> Result<String, ResourceError> {
        Ok(self.dry_run(step).summary)
    }

    fn snapshot(&self, step: &PlanStep) -> Result<Option<Self::Snap>, ResourceError> {
        // Read-only actions take no snapshot.
        match step.action.as_str() {
            "list" => Ok(None),
            "rename" | "move" => {
                let original = PathBuf::from(param(step, "from"));
                let moved_to = PathBuf::from(resolve_target(step));
                Ok(Some(FsSnapshot::Moved { original, moved_to }))
            }
            "delete" => {
                // Compute the trash path now, so the snapshot the engine keeps
                // has everything needed to undo. execute() will reuse this exact
                // path by recomputing deterministically from the same inputs.
                let original = PathBuf::from(param(step, "path"));
                let trashed_to = trash_path_for(&original)?;
                Ok(Some(FsSnapshot::Trashed {
                    original,
                    trashed_to,
                }))
            }
            _ => Ok(None),
        }
    }

    fn execute(&mut self, step: &PlanStep) -> Result<(), ResourceError> {
        match step.action.as_str() {
            "list" => {
                let path = param(step, "path");
                fs::read_dir(&path).map_err(|e| fail(format!("list {path}: {e}")))?;
                Ok(())
            }
            "rename" | "move" => {
                let from = param(step, "from");
                if !Path::new(&from).exists() {
                    return Err(fail(format!("source does not exist: {from}")));
                }
                let to = resolve_target(step);
                // Moving a file into a folder that does not exist yet is an
                // ordinary request — "put this in pdf/" — so make the folder
                // rather than failing. Undo removes it again if it is left empty.
                if let Some(parent) = Path::new(&to).parent() {
                    if !parent.as_os_str().is_empty() && !parent.exists() {
                        fs::create_dir_all(parent)
                            .map_err(|e| fail(format!("create {}: {e}", parent.display())))?;
                    }
                }
                fs::rename(&from, &to)
                    .map_err(|e| fail(format!("{} {from}->{to}: {e}", step.action)))?;
                Ok(())
            }
            "delete" => {
                let path = param(step, "path");
                let path = Path::new(&path);
                if !path.exists() {
                    return Err(fail(format!("does not exist: {}", path.display())));
                }
                // Same deterministic trash path the snapshot computed.
                let trashed_to = trash_path_for(path)?;
                if let Some(parent) = trashed_to.parent() {
                    fs::create_dir_all(parent).map_err(|e| fail(format!("create trash: {e}")))?;
                }
                fs::rename(path, &trashed_to)
                    .map_err(|e| fail(format!("trash {}: {e}", path.display())))?;
                Ok(())
            }
            other => Err(fail(format!("unknown action '{other}'"))),
        }
    }

    fn verify(&self, step: &PlanStep) -> Result<bool, ResourceError> {
        match step.action.as_str() {
            "list" => Ok(true), // read-only verifies trivially
            "rename" | "move" => {
                let from = param(step, "from");
                let to = resolve_target(step);
                let new_exists = Path::new(&to).exists();
                let old_gone = !Path::new(&from).exists();
                Ok(new_exists && old_gone)
            }
            "delete" => {
                // The original path should be gone (moved to trash).
                let path = param(step, "path");
                Ok(!Path::new(&path).exists())
            }
            _ => Ok(true),
        }
    }

    fn restore(&mut self, snapshot: Self::Snap) -> Result<(), ResourceError> {
        match snapshot {
            FsSnapshot::Moved { original, moved_to } => {
                let created_dir = moved_to.parent().map(|p| p.to_path_buf());
                fs::rename(&moved_to, &original).map_err(|e| fail(format!("undo move: {e}")))?;
                // If the move made a folder and nothing else went into it, take
                // it away again — an undo should leave no trace.
                if let Some(dir) = created_dir {
                    if dir.is_dir()
                        && fs::read_dir(&dir)
                            .map(|mut d| d.next().is_none())
                            .unwrap_or(false)
                    {
                        let _ = fs::remove_dir(&dir);
                    }
                }
                Ok(())
            }
            FsSnapshot::Trashed {
                original,
                trashed_to,
            } => {
                fs::rename(&trashed_to, &original)
                    .map_err(|e| fail(format!("undo delete: {e}")))?;
                Ok(())
            }
            FsSnapshot::None => Ok(()),
        }
    }
}

// --- Helpers ----------------------------------------------------------------

fn fail(msg: String) -> ResourceError {
    ResourceError::Failed(msg)
}

/// Read a string parameter, defaulting to "".
fn param(step: &PlanStep, key: &str) -> String {
    step.parameters
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

/// Resolve the rename/move target: if `to` has no path separator, keep it in the
/// same directory as `from` (matching the prototype).
fn resolve_target(step: &PlanStep) -> String {
    let to = param(step, "to");
    if to.contains('/') || to.contains('\\') {
        return to;
    }
    let from = param(step, "from");
    match Path::new(&from).parent() {
        Some(dir) => dir.join(&to).to_string_lossy().into_owned(),
        None => to,
    }
}

/// Compute a deterministic trash path for a file:
/// `<parent>/.cmdos-trash/<name>`. Deterministic (no timestamp) so that
/// `snapshot` and `execute` compute the *same* path and undo can find the file.
/// If a name collision in trash is possible, a future version can disambiguate;
/// for now last-delete-wins within a session, matching the reversible model.
fn trash_path_for(path: &Path) -> Result<PathBuf, ResourceError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unnamed".into());
    Ok(parent.join(".cmdos-trash").join(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cmd_types::{Id, StepStatus};
    use std::collections::BTreeMap;
    use std::fs;
    use tempfile::tempdir;

    fn step(action: &str, params: &[(&str, &str)]) -> PlanStep {
        let mut p = BTreeMap::new();
        for (k, v) in params {
            p.insert((*k).to_string(), serde_json::json!(v));
        }
        PlanStep {
            id: Id::new(),
            description: action.into(),
            capability: "filesystem".into(),
            action: action.into(),
            parameters: p,
            depends_on: vec![],
            requires_permission: false,
            status: StepStatus::Pending,
            error: None,
        }
    }

    // Contract 1: dry-run of rename describes the change and touches nothing.
    #[test]
    fn dry_run_rename_is_read_only_and_truthful() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("a.txt");
        fs::write(&src, "hello").unwrap();

        let fs_cap = FileSystem::new();
        let s = step(
            "rename",
            &[("from", src.to_str().unwrap()), ("to", "b.txt")],
        );
        let preview = fs_cap.dry_run(&s);

        assert!(preview.summary.contains(src.to_str().unwrap()));
        assert!(preview.reversible);
        assert!(src.exists(), "dry-run must not modify the filesystem");
        assert!(!dir.path().join("b.txt").exists());
    }

    // Contract 2: dry-run warns when the source is missing.
    #[test]
    fn dry_run_warns_missing_source() {
        let dir = tempdir().unwrap();
        let fs_cap = FileSystem::new();
        let s = step(
            "rename",
            &[
                ("from", dir.path().join("ghost.txt").to_str().unwrap()),
                ("to", "x.txt"),
            ],
        );
        let preview = fs_cap.dry_run(&s);
        assert!(preview
            .warnings
            .iter()
            .any(|w| w.contains("does not exist")));
    }

    // Contract 3: rename execute → verify → undo restores exactly.
    #[test]
    fn rename_execute_verify_undo_roundtrip() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("report.txt");
        fs::write(&src, "q1").unwrap();
        let target = dir.path().join("report-final.txt");

        let mut fs_cap = FileSystem::new();
        let s = step(
            "rename",
            &[("from", src.to_str().unwrap()), ("to", "report-final.txt")],
        );

        let snap = fs_cap.snapshot(&s).unwrap().unwrap();
        fs_cap.execute(&s).unwrap();
        assert!(target.exists());
        assert!(!src.exists());
        assert!(fs_cap.verify(&s).unwrap());

        fs_cap.restore(snap).unwrap();
        assert!(src.exists(), "undo restores the original path");
        assert!(!target.exists());
    }

    // Contract 4: verify reports failure when the world doesn't match the claim.
    #[test]
    fn verify_fails_when_rename_never_happened() {
        let dir = tempdir().unwrap();
        let fs_cap = FileSystem::new();
        let s = step(
            "rename",
            &[
                ("from", dir.path().join("never-a.txt").to_str().unwrap()),
                ("to", "never-b.txt"),
            ],
        );
        assert!(!fs_cap.verify(&s).unwrap(), "verify must not rubber-stamp");
    }

    // Contract 5: list executes, verifies as read-only, takes no snapshot.
    #[test]
    fn list_is_read_only_no_snapshot() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("one.txt"), "1").unwrap();

        let mut fs_cap = FileSystem::new();
        let s = step("list", &[("path", dir.path().to_str().unwrap())]);

        assert!(fs_cap.snapshot(&s).unwrap().is_none());
        fs_cap.execute(&s).unwrap();
        assert!(fs_cap.verify(&s).unwrap());
    }

    // Contract 6: delete moves to trash (recoverable), original gone, restorable.
    #[test]
    fn moving_into_a_new_folder_creates_it_and_undo_removes_it() {
        // "Put this in pdf/" is an ordinary request even when pdf/ does not
        // exist yet. Before this, rename simply failed.
        let dir = tempdir().unwrap();
        let src = dir.path().join("report.pdf");
        fs::write(&src, b"x").unwrap();

        let step = step(
            "move",
            &[
                ("from", src.to_str().unwrap()),
                ("to", dir.path().join("pdf/report.pdf").to_str().unwrap()),
            ],
        );

        let mut fs_cap = FileSystem::new();
        let snap = fs_cap.snapshot(&step).unwrap().unwrap();
        fs_cap.execute(&step).unwrap();

        assert!(dir.path().join("pdf/report.pdf").exists());
        assert!(fs_cap.verify(&step).unwrap());

        fs_cap.restore(snap).unwrap();
        assert!(src.exists(), "the file came back");
        assert!(
            !dir.path().join("pdf").exists(),
            "and the folder it made is gone"
        );
    }

    #[test]
    fn delete_moves_to_trash_and_is_recoverable() {
        let dir = tempdir().unwrap();
        let victim = dir.path().join("old-notes.txt");
        fs::write(&victim, "keep me safe").unwrap();

        let mut fs_cap = FileSystem::new();
        let s = step("delete", &[("path", victim.to_str().unwrap())]);

        let preview = fs_cap.dry_run(&s);
        assert!(preview.reversible);

        // Snapshot BEFORE execute won't yet know the trash path; the real engine
        // captures it. Here we execute, then confirm original is gone and content
        // survives in the trash folder.
        fs_cap.execute(&s).unwrap();
        assert!(!victim.exists(), "original path is gone");
        assert!(fs_cap.verify(&s).unwrap());

        // The trash folder holds the content.
        let trash_dir = dir.path().join(".cmdos-trash");
        assert!(trash_dir.exists());
        let restored: Vec<_> = fs::read_dir(&trash_dir).unwrap().collect();
        assert_eq!(restored.len(), 1, "one file in trash");
    }

    // Contract 7: rename of a missing source fails and creates nothing.
    #[test]
    fn rename_missing_source_fails_cleanly() {
        let dir = tempdir().unwrap();
        let mut fs_cap = FileSystem::new();
        let s = step(
            "rename",
            &[
                ("from", dir.path().join("nope.txt").to_str().unwrap()),
                ("to", "yes.txt"),
            ],
        );
        assert!(fs_cap.execute(&s).is_err());
        assert!(!dir.path().join("yes.txt").exists(), "no partial effect");
    }

    // Contract 8: rename target keeps same directory when 'to' has no separator.
    #[test]
    fn rename_target_stays_in_same_directory() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("info.md");
        fs::write(&src, "# hi").unwrap();

        let mut fs_cap = FileSystem::new();
        let s = step(
            "rename",
            &[("from", src.to_str().unwrap()), ("to", "renamed.md")],
        );
        fs_cap.execute(&s).unwrap();

        assert!(dir.path().join("renamed.md").exists());
        assert!(!src.exists());
    }

    // End-to-end through the real engine: rename runs the full reversible pipeline
    // and records to the ledger, then an explicit undo restores the file.
    #[test]
    fn rename_through_engine_then_undo() {
        use cmd_ledger::Ledger;
        use cmd_transaction::TransactionEngine;
        use cmd_types::RiskClass;

        let dir = tempdir().unwrap();
        let src = dir.path().join("draft.txt");
        fs::write(&src, "v1").unwrap();
        let target = dir.path().join("final.txt");

        let mut ledger = Ledger::new();
        let mut fs_cap = FileSystem::new();
        let s = step(
            "rename",
            &[("from", src.to_str().unwrap()), ("to", "final.txt")],
        );

        // Capture the snapshot the engine will keep for undo, then run.
        let snap = fs_cap.snapshot(&s).unwrap().unwrap();
        let out = {
            let mut engine = TransactionEngine::new(&mut ledger);
            engine
                .run(&mut fs_cap, &s, RiskClass::R1Reversible)
                .unwrap()
        };
        assert!(target.exists());
        assert!(!src.exists());
        assert!(ledger.verify().is_ok());

        // User hits Undo.
        let mut engine = TransactionEngine::new(&mut ledger);
        engine
            .rollback(&mut fs_cap, out.transaction, snap, &s)
            .unwrap();
        assert!(src.exists(), "undo through the engine restores the file");
        assert!(!target.exists());
    }

    // The fix for the delete-snapshot gap: delete undo through the engine actually
    // restores the file from trash (snapshot computed the trash path deterministically).
    #[test]
    fn delete_through_engine_then_undo_restores_from_trash() {
        use cmd_ledger::Ledger;
        use cmd_transaction::TransactionEngine;
        use cmd_types::RiskClass;

        let dir = tempdir().unwrap();
        let victim = dir.path().join("keep.txt");
        fs::write(&victim, "precious").unwrap();

        let mut ledger = Ledger::new();
        let mut fs_cap = FileSystem::new();
        let s = step("delete", &[("path", victim.to_str().unwrap())]);

        let snap = fs_cap.snapshot(&s).unwrap().unwrap();
        let out = {
            let mut engine = TransactionEngine::new(&mut ledger);
            engine
                .run(&mut fs_cap, &s, RiskClass::R1Reversible)
                .unwrap()
        };
        assert!(!victim.exists(), "deleted (moved to trash)");

        // Undo brings it back to the original path with its content intact.
        let mut engine = TransactionEngine::new(&mut ledger);
        engine
            .rollback(&mut fs_cap, out.transaction, snap, &s)
            .unwrap();
        assert!(victim.exists(), "delete undo restores from trash");
        assert_eq!(fs::read_to_string(&victim).unwrap(), "precious");
    }
}
