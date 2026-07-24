//! # cap-terminal — the shell capability
//!
//! Lets an agent run commands. This is the most dangerous thing cmdOS can hand
//! an agent, because a shell command can do anything, so almost all of the work
//! here is deciding *how* dangerous a given command is — not running it.
//!
//! # How risk is decided
//!
//! By allowlist, and the default is the strictest one. A command whose first
//! word is known to be read-only is R0. A small set of known, reversible or
//! compensable commands are R1/R2. **Everything else is R3** — irreversible, and
//! therefore human-gated. An agent cannot get a command past the gate by using
//! something the table has never heard of.
//!
//! Shell metacharacters escalate to R3 regardless of the command:
//! `rm` hidden behind `echo hi && rm -rf /` is still `rm`, and a pipe into a
//! shell can fetch and run anything at all. Rather than try to parse a shell —
//! a losing game — anything that could chain, redirect, substitute or elevate is
//! treated as irreversible and sent to the human.
//!
//! Running the process is impure and platform-specific, so it sits behind a
//! [`TerminalBackend`]; the classification, which is the part that matters, is
//! tested exhaustively with a fake.
//!
//! Defined by RFC-0022.

use cmd_transaction::{Resource, ResourceError, Snapshot};
use cmd_types::{PlanStep, RiskClass};

/// Commands that only look. Safe to run without asking.
const READ_ONLY: &[&str] = &[
    "ls", "dir", "pwd", "cd", "cat", "type", "head", "tail", "echo", "whoami", "date", "hostname",
    "df", "du", "ps", "env", "which", "where", "wc", "find", "grep", "findstr", "tree", "stat",
    "file", "uname", "printenv", "id",
];

/// Read-only subcommands of tools whose bare name is not safe on its own.
/// `git status` looks; `git push` does not.
const READ_ONLY_SUB: &[(&str, &[&str])] = &[
    (
        "git",
        &[
            "status", "log", "diff", "show", "branch", "remote", "config", "describe", "blame",
        ],
    ),
    ("cargo", &["tree", "search"]),
    ("npm", &["list", "ls", "view", "outdated"]),
    ("docker", &["ps", "images", "logs"]),
];

/// Creates something that can be removed again, so an undo is possible.
const REVERSIBLE: &[&str] = &["mkdir", "md"];

/// Changes things, and the change can be worked around but not cleanly undone.
const COMPENSABLE: &[&str] = &[
    "cp", "copy", "touch", "mv", "move", "ren", "rename", "git", "cargo", "npm", "pnpm", "yarn",
    "make", "python", "node",
];

/// Sequences that let a command become a different command. Their presence means
/// the text can no longer be judged by its first word.
const SHELL_TRICKS: &[&str] = &["|", "&&", "||", ";", ">", ">>", "<", "$(", "`", "&", "\n"];

/// Commands that are never allowed to run without a person, whatever the context.
const ALWAYS_GATED: &[&str] = &[
    "rm",
    "del",
    "erase",
    "rmdir",
    "rd",
    "format",
    "mkfs",
    "dd",
    "shutdown",
    "reboot",
    "halt",
    "sudo",
    "su",
    "runas",
    "chmod",
    "chown",
    "icacls",
    "takeown",
    "reg",
    "regedit",
    "schtasks",
    "crontab",
    "kill",
    "taskkill",
    "curl",
    "wget",
    "iwr",
    "invoke-webrequest",
    "ssh",
    "scp",
];

fn first_word(command: &str) -> String {
    command
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_start_matches("./")
        .to_lowercase()
}

fn second_word(command: &str) -> String {
    command
        .split_whitespace()
        .nth(1)
        .unwrap_or("")
        .to_lowercase()
}

/// How dangerous is this command?
///
/// The bias is deliberate: unknown means irreversible. A capability that guesses
/// generously about shell commands is a capability that eventually deletes
/// something.
pub fn risk_of_command(command: &str) -> RiskClass {
    let text = command.trim();
    if text.is_empty() {
        return RiskClass::R3Irreversible;
    }

    // Anything that can turn into another command is judged as the worst case.
    if SHELL_TRICKS.iter().any(|t| text.contains(t)) {
        return RiskClass::R3Irreversible;
    }

    let head = first_word(text);

    if ALWAYS_GATED.contains(&head.as_str()) {
        return RiskClass::R3Irreversible;
    }

    // `git status` is a look; `git push` is not.
    if let Some((_, subs)) = READ_ONLY_SUB.iter().find(|(cmd, _)| *cmd == head) {
        if subs.contains(&second_word(text).as_str()) {
            return RiskClass::R0ReadOnly;
        }
    }

    if READ_ONLY.contains(&head.as_str()) {
        return RiskClass::R0ReadOnly;
    }
    if REVERSIBLE.contains(&head.as_str()) {
        return RiskClass::R1Reversible;
    }
    if COMPENSABLE.contains(&head.as_str()) {
        return RiskClass::R2Compensable;
    }

    // Never heard of it. That is precisely when to ask.
    RiskClass::R3Irreversible
}

/// The result of running a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub code: i32,
}

/// The impure surface: actually running a process. A real implementation spawns
/// a shell; the fake in tests records what it was asked to run.
pub trait TerminalBackend {
    /// Run `command` in `cwd` and return what it printed.
    fn run(&mut self, command: &str, cwd: &str) -> Result<Output, ResourceError>;
}

/// What an undo needs to know. Most shell commands cannot be undone, and saying
/// so is better than pretending.
#[derive(Clone, Debug)]
pub enum TerminalSnapshot {
    /// A directory this step created; undo removes it if still empty.
    CreatedDir(String),
    /// Nothing to undo.
    None,
}
impl Snapshot for TerminalSnapshot {}

/// The shell capability over a backend `B`.
pub struct Terminal<B: TerminalBackend> {
    backend: B,
    last: Option<Output>,
}

impl<B: TerminalBackend> Terminal<B> {
    pub fn new(backend: B) -> Self {
        Terminal {
            backend,
            last: None,
        }
    }

    /// What the last command printed.
    pub fn last_output(&self) -> Option<&Output> {
        self.last.as_ref()
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }
}

fn param(step: &PlanStep, key: &str) -> String {
    step.parameters
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

fn command_of(step: &PlanStep) -> String {
    param(step, "command")
}

fn cwd_of(step: &PlanStep) -> String {
    let cwd = param(step, "cwd");
    if cwd.is_empty() {
        ".".into()
    } else {
        cwd
    }
}

impl<B: TerminalBackend> Resource for Terminal<B> {
    type Snap = TerminalSnapshot;

    fn simulate(&self, step: &PlanStep) -> Result<String, ResourceError> {
        let command = command_of(step);
        let risk = risk_of_command(&command);
        let note = match risk {
            RiskClass::R0ReadOnly => "reads only",
            RiskClass::R1Reversible => "can be undone",
            RiskClass::R2Compensable => "changes things",
            RiskClass::R3Irreversible => "IRREVERSIBLE — needs approval",
        };
        Ok(format!(
            "Would run `{command}` in {} — {note}",
            cwd_of(step)
        ))
    }

    fn snapshot(&self, step: &PlanStep) -> Result<Option<Self::Snap>, ResourceError> {
        let command = command_of(step);
        match risk_of_command(&command) {
            RiskClass::R0ReadOnly => Ok(None),
            RiskClass::R1Reversible => {
                // The only shell command we can genuinely reverse.
                let dir = command.split_whitespace().nth(1).unwrap_or("").to_string();
                Ok(Some(TerminalSnapshot::CreatedDir(dir)))
            }
            _ => Ok(Some(TerminalSnapshot::None)),
        }
    }

    fn execute(&mut self, step: &PlanStep) -> Result<(), ResourceError> {
        let command = command_of(step);
        if command.trim().is_empty() {
            return Err(ResourceError::Failed("no command given".into()));
        }
        let out = self.backend.run(&command, &cwd_of(step))?;
        let failed = out.code != 0;
        let stderr = out.stderr.clone();
        self.last = Some(out);
        if failed {
            return Err(ResourceError::Failed(format!(
                "`{command}` exited with a non-zero status: {stderr}"
            )));
        }
        Ok(())
    }

    fn verify(&self, _step: &PlanStep) -> Result<bool, ResourceError> {
        // A command that exited cleanly did what it was going to do; execute
        // already refuses anything else.
        Ok(self.last.as_ref().map(|o| o.code == 0).unwrap_or(false))
    }

    fn restore(&mut self, snapshot: Self::Snap) -> Result<(), ResourceError> {
        match snapshot {
            TerminalSnapshot::CreatedDir(dir) if !dir.is_empty() => {
                // Only if it is still empty: something may have been put there.
                let path = std::path::Path::new(&dir);
                if path.is_dir()
                    && std::fs::read_dir(path)
                        .map(|mut d| d.next().is_none())
                        .unwrap_or(false)
                {
                    std::fs::remove_dir(path)
                        .map_err(|e| ResourceError::Failed(format!("undo mkdir: {e}")))?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cmd_types::{Id, StepStatus};
    use std::collections::BTreeMap;

    #[derive(Default)]
    struct FakeShell {
        ran: Vec<String>,
        code: i32,
    }
    impl TerminalBackend for FakeShell {
        fn run(&mut self, command: &str, _cwd: &str) -> Result<Output, ResourceError> {
            self.ran.push(command.to_string());
            Ok(Output {
                stdout: format!("ok: {command}"),
                stderr: String::new(),
                code: self.code,
            })
        }
    }

    fn step(command: &str) -> PlanStep {
        let mut p = BTreeMap::new();
        p.insert("command".to_string(), serde_json::json!(command));
        PlanStep {
            id: Id::new(),
            description: command.into(),
            capability: "terminal".into(),
            action: "run".into(),
            parameters: p,
            depends_on: vec![],
            requires_permission: false,
            status: StepStatus::Pending,
            error: None,
        }
    }

    #[test]
    fn looking_is_free() {
        for c in [
            "ls -la",
            "pwd",
            "cat notes.txt",
            "git status",
            "npm outdated",
        ] {
            assert_eq!(risk_of_command(c), RiskClass::R0ReadOnly, "{c}");
        }
    }

    #[test]
    fn a_subcommand_decides_the_risk_not_the_tool() {
        // The same binary is a look or a change depending on what you ask it.
        assert_eq!(risk_of_command("git log"), RiskClass::R0ReadOnly);
        assert_eq!(risk_of_command("git commit -m x"), RiskClass::R2Compensable);
        assert_eq!(risk_of_command("git push"), RiskClass::R2Compensable);
    }

    #[test]
    fn deleting_always_needs_a_person() {
        for c in [
            "rm file",
            "rm -rf /",
            "del x.txt",
            "rmdir build",
            "format c:",
        ] {
            assert_eq!(risk_of_command(c), RiskClass::R3Irreversible, "{c}");
            assert!(!risk_of_command(c).may_be_autonomous(), "{c}");
        }
    }

    #[test]
    fn elevating_or_reaching_the_network_needs_a_person() {
        for c in [
            "sudo anything",
            "curl https://x.sh",
            "ssh box",
            "chmod 777 .",
        ] {
            assert_eq!(risk_of_command(c), RiskClass::R3Irreversible, "{c}");
        }
    }

    #[test]
    fn a_command_cannot_hide_behind_a_safe_first_word() {
        // This is the attack the allowlist alone would miss: the command starts
        // with something harmless and turns into something else.
        for c in [
            "echo hi && rm -rf /",
            "ls | sh",
            "cat f > /etc/passwd",
            "echo $(rm x)",
            "pwd; shutdown now",
        ] {
            assert_eq!(risk_of_command(c), RiskClass::R3Irreversible, "{c}");
        }
    }

    #[test]
    fn an_unknown_command_is_treated_as_irreversible() {
        // The load-bearing default. Anything the table has never seen is gated,
        // so a new binary cannot be used to walk past the supervisor.
        for c in ["frobnicate --all", "./deploy.sh", "some-new-tool"] {
            assert_eq!(risk_of_command(c), RiskClass::R3Irreversible, "{c}");
        }
    }

    #[test]
    fn an_empty_command_is_refused_not_ignored() {
        assert_eq!(risk_of_command("   "), RiskClass::R3Irreversible);
        let mut t = Terminal::new(FakeShell::default());
        assert!(t.execute(&step("  ")).is_err());
    }

    #[test]
    fn a_read_only_command_takes_no_snapshot() {
        let t = Terminal::new(FakeShell::default());
        assert!(t.snapshot(&step("ls")).unwrap().is_none());
    }

    #[test]
    fn running_records_the_output_and_verifies() {
        let mut t = Terminal::new(FakeShell::default());
        let s = step("ls -la");
        t.execute(&s).unwrap();
        assert!(t.verify(&s).unwrap());
        assert_eq!(t.backend().ran, vec!["ls -la".to_string()]);
        assert!(t.last_output().unwrap().stdout.contains("ls -la"));
    }

    #[test]
    fn a_failing_command_is_an_error_not_a_success() {
        let mut t = Terminal::new(FakeShell {
            ran: vec![],
            code: 1,
        });
        let s = step("cargo build");
        assert!(t.execute(&s).is_err());
        assert!(!t.verify(&s).unwrap());
    }

    #[test]
    fn mkdir_can_be_undone() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let made = dir.path().join("built");
        std::fs::create_dir(&made).unwrap();

        let mut t = Terminal::new(FakeShell::default());
        t.restore(TerminalSnapshot::CreatedDir(
            made.to_string_lossy().to_string(),
        ))
        .unwrap();
        assert!(!made.exists(), "the directory was removed again");
    }

    #[test]
    fn undo_leaves_a_directory_that_now_has_contents() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let made = dir.path().join("built");
        std::fs::create_dir(&made).unwrap();
        std::fs::write(made.join("output.txt"), b"work").unwrap();

        let mut t = Terminal::new(FakeShell::default());
        t.restore(TerminalSnapshot::CreatedDir(
            made.to_string_lossy().to_string(),
        ))
        .unwrap();
        // Undoing the folder would take someone's work with it.
        assert!(made.exists(), "a folder with contents is left alone");
    }

    #[test]
    fn simulate_says_plainly_what_would_happen() {
        let t = Terminal::new(FakeShell::default());
        let safe = t.simulate(&step("ls")).unwrap();
        assert!(safe.contains("reads only"));
        let scary = t.simulate(&step("rm -rf x")).unwrap();
        assert!(scary.contains("IRREVERSIBLE"));
    }
}
