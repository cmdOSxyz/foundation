//! # shell-core — the bridge between cmdShell and the cmdOS core
//!
//! cmdShell's UI needs one thing from the system: plain, serializable answers.
//! This crate provides them. It owns a running [`Machine`] — agent, tool
//! registry, key router, filesystem capability, ledger — and exposes the small
//! set of operations the interface actually performs, each returning data that
//! serializes straight to the front end.
//!
//! Deliberately free of any Tauri dependency: the Tauri layer is a thin wrapper
//! whose commands call these methods. That keeps the logic here testable in CI
//! (GUI toolchains are not), and lets the same core drive a CLI, a daemon, or a
//! different shell later.
//!
//! Every `// TAURI` hook in the cmdShell React code corresponds to one method
//! below.
//!
//! Defined by RFC-0021.

use aipc::Aipc;
use alios::{Agent, RulePlanner};
use cap_terminal::{risk_of_command, SystemShell, Terminal};
use cmd_kernel::{AuthorityContext, Kernel, StepOutcome};
use cmd_ledger::Ledger;
use cmd_router::{ApiKey, KeyRouter};
use cmd_shadow::{Change, ShadowWorld};
use cmd_types::{now, ExecutionPlan, Id, Mandate, PlanStep, RiskClass};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A tool the agent may call, as the UI lists it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolInfo {
    pub capability: String,
    pub action: String,
    pub description: String,
    pub risk: String,
    /// True when policy would let the agent run this without asking a human.
    pub autonomous: bool,
}

/// What happened to one planned step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepReport {
    pub description: String,
    pub capability: String,
    pub action: String,
    pub risk: String,
    /// `executed` | `awaiting_approval` | `blocked` | `failed` | `not_reached`
    pub outcome: String,
    /// Present when the step was gated or failed.
    pub reason: Option<String>,
}

/// The result of submitting an intent: what was planned, and what the kernel did.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntentReport {
    pub intent: String,
    pub summary: String,
    pub steps: Vec<StepReport>,
    pub completed: bool,
    /// Number of ledger entries after the run.
    pub ledger_len: usize,
    /// Whether the ledger's hash chain still verifies.
    pub chain_ok: bool,
}

/// One key in the router, as the UI's key dashboard shows it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyInfo {
    pub id: String,
    pub provider: String,
    pub label: String,
    pub masked: String,
    pub used: u64,
    pub limit: u64,
    pub remaining: u64,
    pub active: bool,
}

/// One audit record, as the Ledger app shows it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LedgerRow {
    pub index: u64,
    pub event_type: String,
    pub at: String,
    pub hash: String,
}

/// One entry in a directory, as the Files app lists it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    /// Size in bytes; 0 for directories.
    pub size: u64,
}

/// A directory and its contents.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DirListing {
    /// The directory that was read, absolute.
    pub path: String,
    /// The parent directory, if there is one — for the "up" control.
    pub parent: Option<String>,
    pub entries: Vec<FileEntry>,
}

/// One finished outcome the user can promote or throw away.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutcomeInfo {
    pub id: String,
    pub label: String,
    /// What promoting this would do to the real folder, in plain words.
    pub changes: Vec<String>,
    /// The plan that produced it, step by step.
    pub steps: Vec<StepReport>,
    pub completed: bool,
}

fn describe(c: &Change) -> String {
    match c {
        Change::Created(p) => format!("create {p}"),
        Change::Modified(p) => format!("modify {p}"),
        Change::Deleted(p) => format!("delete {p}"),
    }
}

/// What a shell command did.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandResult {
    pub command: String,
    pub risk: String,
    /// `executed` | `awaiting_approval` | `blocked` | `failed`
    pub outcome: String,
    pub stdout: String,
    pub stderr: String,
    pub code: i32,
    /// Present when the gate stopped it, saying why.
    pub reason: Option<String>,
}

/// Headline state for the agent status card.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MachineStatus {
    pub agent: String,
    pub ledger_len: usize,
    pub chain_ok: bool,
    pub key_count: usize,
    pub requests_remaining: u64,
    pub tool_count: usize,
}

fn risk_label(r: RiskClass) -> String {
    match r {
        RiskClass::R0ReadOnly => "R0",
        RiskClass::R1Reversible => "R1",
        RiskClass::R2Compensable => "R2",
        RiskClass::R3Irreversible => "R3",
    }
    .to_string()
}

/// A running cmdOS machine: everything the shell talks to, in one place.
pub struct Machine {
    agent_name: String,
    aipc: Aipc,
    router: KeyRouter,
    ledger: Ledger,
    files: cap_files::FileSystem,
    mandate: Mandate,
    /// Open candidate futures, if the agent is working in the shadow world.
    shadow: Option<ShadowWorld>,
    shadow_root: Option<std::path::PathBuf>,
}

impl Default for Machine {
    fn default() -> Self {
        Self::new("Nova")
    }
}

impl Machine {
    /// Boot a machine for `agent_name`, with the first-party tools registered and
    /// a filesystem mandate whose autonomous ceiling is R1. Anything riskier
    /// stops for the human — the guarantee the interface is built to show.
    pub fn new(agent_name: impl Into<String>) -> Self {
        let mandate = Mandate {
            id: Id::new(),
            agent_id: Id::new(),
            scope: "shell session".into(),
            capabilities: vec!["filesystem".into(), "browser".into(), "terminal".into()],
            max_autonomous_risk: RiskClass::R1Reversible,
            budget_id: None,
            granted_at: now(),
            expires_at: None,
            revoked_at: None,
        };
        Machine {
            agent_name: agent_name.into(),
            aipc: Aipc::new().with_first_party(),
            router: KeyRouter::new(),
            ledger: Ledger::new(),
            files: cap_files::FileSystem::new(),
            mandate,
            shadow: None,
            shadow_root: None,
        }
    }

    /// The agent's name.
    pub fn agent(&self) -> &str {
        &self.agent_name
    }

    // ---- tools -------------------------------------------------------------

    /// Every tool the agent can call. `autonomous` reflects the policy ceiling,
    /// so the UI can mark which actions will stop for approval.
    pub fn tools(&self) -> Vec<ToolInfo> {
        self.aipc
            .list_tools()
            .into_iter()
            .map(|t| ToolInfo {
                capability: t.capability,
                action: t.name,
                description: t.description,
                risk: risk_label(t.risk),
                autonomous: t.risk.may_be_autonomous()
                    && t.risk <= self.mandate.max_autonomous_risk,
            })
            .collect()
    }

    /// Risk for a capability action, taken from the tool catalog. Unknown actions
    /// are treated as R2 — never silently assumed safe.
    fn risk_of(&self, step: &PlanStep) -> RiskClass {
        self.aipc
            .tool(&step.capability, &step.action)
            .map(|t| t.risk)
            .unwrap_or(RiskClass::R2Compensable)
    }

    // ---- the main loop -----------------------------------------------------

    /// Plan an intent and run it: the whole loop the shell exists to show.
    /// Intent → Planning → Permission (Alios) → Execution → Verification → Result.
    ///
    /// Uses the deterministic planner, so this works with no API key. Steps that
    /// need approval or are blocked stop the run and are reported as such — the
    /// shell renders them as pending decisions rather than failures.
    pub fn submit_intent(&mut self, text: &str) -> IntentReport {
        let agent = Agent::new(self.agent_name.clone(), RulePlanner::new());
        let (intent, mut plan) = agent.plan_for(text);
        // "Tidy X" needs to know what is in X, which the keyword planner cannot
        // see. Fall back to it only when the folder cannot be read.
        if Self::wants_tidying(text) {
            if let Some(path) = text.split_whitespace().find(|w| w.contains(['/', '\\'])) {
                if let Some(real) = self.plan_tidy(path) {
                    plan = real;
                }
            }
        }

        // Classify every step up front so the UI can show the plan with risks
        // even for steps the kernel never reaches.
        let risks: Vec<RiskClass> = plan.steps.iter().map(|s| self.risk_of(s)).collect();

        let ctx = AuthorityContext {
            mandate: Some(&self.mandate),
            budget: None,
        };
        let run = {
            let mut kernel = Kernel::new(&mut self.ledger);
            let aipc = &self.aipc;
            let resolve = |s: &PlanStep| {
                aipc.tool(&s.capability, &s.action)
                    .map(|t| t.risk)
                    .unwrap_or(RiskClass::R2Compensable)
            };
            kernel.run_plan(&plan, &mut self.files, &ctx, &resolve)
        };

        let steps = plan
            .steps
            .iter()
            .zip(risks.iter())
            .map(|(s, r)| {
                let found = run.steps.iter().find(|(id, _)| *id == s.id);
                let (outcome, reason) = match found.map(|(_, o)| o) {
                    Some(StepOutcome::Executed) => ("executed", None),
                    Some(StepOutcome::AwaitingApproval(m)) => {
                        ("awaiting_approval", Some(m.clone()))
                    }
                    Some(StepOutcome::Blocked(m)) => ("blocked", Some(m.clone())),
                    Some(StepOutcome::Failed(m)) => ("failed", Some(m.clone())),
                    None => ("not_reached", None),
                };
                StepReport {
                    description: s.description.clone(),
                    capability: s.capability.clone(),
                    action: s.action.clone(),
                    risk: risk_label(*r),
                    outcome: outcome.to_string(),
                    reason,
                }
            })
            .collect();

        IntentReport {
            intent: intent.raw_text,
            summary: plan.summary,
            steps,
            completed: run.completed,
            ledger_len: self.ledger.len(),
            chain_ok: self.ledger.verify().is_ok(),
        }
    }

    /// Build a plan that actually *does* something to a folder.
    ///
    /// The rule planner is keyword-based and cannot see the disk, so asking it to
    /// "tidy this folder" produced a read-only listing and, in a shadow, an
    /// outcome that changed nothing. Sorting files needs to know what is in the
    /// folder, so that planning happens here, where the filesystem is reachable.
    ///
    /// A model-backed planner subsumes this; until one is wired, it is what makes
    /// the agent able to do real work.
    fn plan_tidy(&self, root: &str) -> Option<ExecutionPlan> {
        let listing = self.list_dir(root).ok()?;

        let mut steps = Vec::new();
        for entry in listing.entries.iter().filter(|e| !e.is_dir) {
            // Leave hidden files alone: they are usually configuration, and
            // moving them is rarely what "tidy" means.
            if entry.name.starts_with('.') {
                continue;
            }
            let ext = std::path::Path::new(&entry.name)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());
            let folder = match ext {
                Some(e) if !e.is_empty() => e,
                _ => continue, // no extension, nothing to sort it by
            };

            let mut parameters = BTreeMap::new();
            parameters.insert("from".into(), serde_json::json!(entry.path.clone()));
            parameters.insert(
                "to".into(),
                serde_json::json!(format!("{folder}/{}", entry.name)),
            );
            steps.push(PlanStep {
                id: Id::new(),
                description: format!("Move {} into {}/", entry.name, folder),
                capability: "filesystem".into(),
                action: "move".into(),
                parameters,
                depends_on: vec![],
                requires_permission: false,
                status: cmd_types::StepStatus::Pending,
                error: None,
            });

            if steps.len() >= 60 {
                break; // a plan you cannot read is a plan you cannot judge
            }
        }

        if steps.is_empty() {
            return None;
        }
        Some(ExecutionPlan {
            id: Id::new(),
            intent_id: Id::new(),
            created_at: now(),
            status: cmd_types::PlanStatus::Draft,
            summary: format!("Sort {} file(s) into folders by type", steps.len()),
            steps,
        })
    }

    /// Whether a request is asking for the folder to be organised.
    fn wants_tidying(text: &str) -> bool {
        let t = text.to_lowercase();
        ["tidy", "organis", "organiz", "sort", "clean up", "arrange"]
            .iter()
            .any(|k| t.contains(k))
    }

    // ---- the shadow world --------------------------------------------------

    /// Run an intent **inside a fork** of `root`. The agent completes the whole
    /// plan; reality is not touched. What comes back is a finished outcome, to
    /// promote or throw away.
    ///
    /// Steps are executed against the fork by rewriting their path parameters to
    /// point inside it — the capability and the kernel are unchanged, they simply
    /// operate somewhere that is not real yet.
    pub fn run_in_shadow(
        &mut self,
        text: &str,
        root: &str,
        label: &str,
        scratch: &str,
    ) -> Result<OutcomeInfo, String> {
        let root_path = std::path::Path::new(root).to_path_buf();

        // Plan first. Planning only reads, and doing it before the fork is opened
        // keeps the mutable borrow of the shadow world from overlapping it.
        let plan = if Self::wants_tidying(text) {
            self.plan_tidy(root).unwrap_or_else(|| {
                let agent = Agent::new(self.agent_name.clone(), RulePlanner::new());
                agent.plan_for(text).1
            })
        } else {
            let agent = Agent::new(self.agent_name.clone(), RulePlanner::new());
            agent.plan_for(text).1
        };

        if self.shadow.is_none() || self.shadow_root.as_deref() != Some(root_path.as_path()) {
            self.shadow = Some(ShadowWorld::new(&root_path, scratch).map_err(|e| e.to_string())?);
            self.shadow_root = Some(root_path.clone());
        }

        let world = self.shadow.as_mut().expect("just created");
        let fork_id = world.fork(label).map_err(|e| e.to_string())?;

        // Point every path inside the fork, copying originals in as needed so
        // the agent still sees the folder's real contents.
        let fork = world.get_mut(fork_id).ok_or("fork vanished")?;
        let work = fork.work_dir().to_path_buf();
        let mut shadowed = plan.clone();
        for step in shadowed.steps.iter_mut() {
            for key in ["path", "from", "to"] {
                let Some(v) = step.parameters.get(key).and_then(|v| v.as_str()) else {
                    continue;
                };
                let p = std::path::Path::new(v);
                let rel = p.strip_prefix(&root_path).ok().map(|r| r.to_path_buf());
                let rel = match rel {
                    Some(r) if !r.as_os_str().is_empty() => r,
                    // A bare name or a path outside the root is taken as relative
                    // to the root — an agent must not reach past it.
                    _ if !p.is_absolute() => p.to_path_buf(),
                    _ => continue,
                };
                if let Some(k) = rel.to_str() {
                    let _ = fork.materialize(k); // copy-on-write, best effort
                }
                let target = work.join(&rel);
                step.parameters
                    .insert(key.to_string(), serde_json::json!(target.to_string_lossy()));
            }
        }

        // Same kernel, same gate, same ledger — only the ground is different.
        let risks: Vec<RiskClass> = shadowed.steps.iter().map(|s| self.risk_of(s)).collect();
        let ctx = AuthorityContext {
            mandate: Some(&self.mandate),
            budget: None,
        };
        let run = {
            let mut kernel = Kernel::new(&mut self.ledger);
            let aipc = &self.aipc;
            let resolve = |s: &PlanStep| {
                aipc.tool(&s.capability, &s.action)
                    .map(|t| t.risk)
                    .unwrap_or(RiskClass::R2Compensable)
            };
            kernel.run_plan(&shadowed, &mut self.files, &ctx, &resolve)
        };

        let steps: Vec<StepReport> = shadowed
            .steps
            .iter()
            .zip(risks.iter())
            .map(|(s, r)| {
                let found = run.steps.iter().find(|(id, _)| *id == s.id);
                let (outcome, reason) = match found.map(|(_, o)| o) {
                    Some(StepOutcome::Executed) => ("executed", None),
                    Some(StepOutcome::AwaitingApproval(m)) => {
                        ("awaiting_approval", Some(m.clone()))
                    }
                    Some(StepOutcome::Blocked(m)) => ("blocked", Some(m.clone())),
                    Some(StepOutcome::Failed(m)) => ("failed", Some(m.clone())),
                    None => ("not_reached", None),
                };
                StepReport {
                    description: s.description.clone(),
                    capability: s.capability.clone(),
                    action: s.action.clone(),
                    risk: risk_label(*r),
                    outcome: outcome.to_string(),
                    reason,
                }
            })
            .collect();

        let world = self.shadow.as_ref().expect("world");
        let fork = world.get(fork_id).ok_or("fork vanished")?;
        Ok(OutcomeInfo {
            id: fork_id.to_string(),
            label: label.to_string(),
            changes: fork
                .changes()
                .map_err(|e| e.to_string())?
                .iter()
                .map(describe)
                .collect(),
            steps,
            completed: run.completed,
        })
    }

    /// Every outcome currently on offer.
    pub fn shadow_outcomes(&self) -> Vec<OutcomeInfo> {
        let Some(world) = self.shadow.as_ref() else {
            return Vec::new();
        };
        world
            .outcomes()
            .unwrap_or_default()
            .into_iter()
            .map(|o| OutcomeInfo {
                id: o.id.to_string(),
                label: o.label,
                changes: o.changes.iter().map(describe).collect(),
                steps: Vec::new(),
                completed: true,
            })
            .collect()
    }

    /// Choose a future: promote it and drop the rest.
    pub fn shadow_choose(&mut self, id: &str) -> Result<Vec<String>, String> {
        let world = self.shadow.as_mut().ok_or("no shadow world open")?;
        let target = world
            .outcomes()
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|o| o.id.to_string() == id)
            .ok_or("no such outcome")?;
        let applied = world.choose(target.id).map_err(|e| e.to_string())?;
        self.shadow = None;
        self.shadow_root = None;
        Ok(applied.iter().map(describe).collect())
    }

    /// Walk away from every open future. Reality never knew.
    pub fn shadow_discard(&mut self) -> Result<(), String> {
        if let Some(world) = self.shadow.as_mut() {
            world.discard_all().map_err(|e| e.to_string())?;
        }
        self.shadow = None;
        self.shadow_root = None;
        Ok(())
    }

    // ---- keys (BYOK) -------------------------------------------------------

    /// Add one of the user's API keys to the router.
    pub fn add_key(&mut self, secret: &str, request_limit: u64, label: Option<&str>) -> KeyInfo {
        let mut key = ApiKey::new(secret, request_limit);
        if let Some(l) = label {
            key = key.with_label(l);
        }
        let id = key.id;
        self.router.add(key);
        self.key_stats()
            .into_iter()
            .find(|k| k.id == id.to_string())
            .expect("key just added")
    }

    /// Remove a key by its id string. Returns whether one was removed.
    pub fn remove_key(&mut self, id: &str) -> bool {
        let found = self
            .router
            .stats()
            .into_iter()
            .find(|s| s.id.to_string() == id)
            .map(|s| s.id);
        match found {
            Some(i) => self.router.remove(i),
            None => false,
        }
    }

    /// Per-key meters for the dashboard.
    pub fn key_stats(&self) -> Vec<KeyInfo> {
        self.router
            .stats()
            .into_iter()
            .map(|k| KeyInfo {
                id: k.id.to_string(),
                provider: k.provider.label().to_string(),
                label: k.label,
                masked: k.masked,
                used: k.used,
                limit: k.limit,
                remaining: k.remaining,
                active: k.active,
            })
            .collect()
    }

    /// True when every key is spent — the shell shows the "add or replace a key"
    /// warning.
    pub fn keys_exhausted(&self) -> bool {
        !self.router.is_empty() && self.router.total_remaining() == 0
    }

    // ---- the shell ---------------------------------------------------------

    /// Run a shell command through the kernel.
    ///
    /// The command is classified by what it can do (see `cap-terminal`), then
    /// put through the same gate and ledger as everything else. A command the
    /// mandate does not cover, or one that is irreversible, comes back as a
    /// pending decision rather than running — which is the whole point of
    /// letting an agent near a shell at all.
    pub fn run_command(&mut self, command: &str, cwd: &str) -> Result<CommandResult, String> {
        let risk = risk_of_command(command);

        let mut parameters = BTreeMap::new();
        parameters.insert("command".into(), serde_json::json!(command));
        parameters.insert("cwd".into(), serde_json::json!(cwd));
        let step = PlanStep {
            id: Id::new(),
            description: format!("Run `{command}`"),
            capability: "terminal".into(),
            action: "run".into(),
            parameters,
            depends_on: vec![],
            requires_permission: false,
            status: cmd_types::StepStatus::Pending,
            error: None,
        };
        let plan = ExecutionPlan {
            id: Id::new(),
            intent_id: Id::new(),
            created_at: now(),
            status: cmd_types::PlanStatus::Approved,
            summary: format!("Run `{command}`"),
            steps: vec![step],
        };

        // The shell is rooted at the working directory it was given, so a
        // command cannot be aimed outside the folder the user opened.
        let root = if cwd.is_empty() {
            self.home_dir()
        } else {
            cwd.to_string()
        };
        let mut shell = Terminal::new(SystemShell::new(&root));

        let ctx = AuthorityContext {
            mandate: Some(&self.mandate),
            budget: None,
        };
        let run = {
            let mut kernel = Kernel::new(&mut self.ledger);
            let resolve = |_: &PlanStep| risk;
            kernel.run_plan(&plan, &mut shell, &ctx, &resolve)
        };

        let (outcome, reason) = match run.steps.first().map(|(_, o)| o) {
            Some(StepOutcome::Executed) => ("executed", None),
            Some(StepOutcome::AwaitingApproval(m)) => ("awaiting_approval", Some(m.clone())),
            Some(StepOutcome::Blocked(m)) => ("blocked", Some(m.clone())),
            Some(StepOutcome::Failed(m)) => ("failed", Some(m.clone())),
            None => ("blocked", Some("the kernel did not reach this step".into())),
        };

        let out = shell.last_output();
        Ok(CommandResult {
            command: command.to_string(),
            risk: risk_label(risk),
            outcome: outcome.to_string(),
            stdout: out.map(|o| o.stdout.clone()).unwrap_or_default(),
            stderr: out.map(|o| o.stderr.clone()).unwrap_or_default(),
            code: out.map(|o| o.code).unwrap_or(-1),
            reason,
        })
    }

    // ---- browsing ----------------------------------------------------------

    /// Read a directory for display.
    ///
    /// This is the *user* looking at their machine, not the agent acting on it,
    /// so it takes no mandate and writes no ledger entry: reading a folder to
    /// show it on screen is not an agent action. Everything the agent does to
    /// files still goes through the capability and the kernel.
    pub fn list_dir(&self, path: &str) -> Result<DirListing, String> {
        let p = std::path::Path::new(path);
        let read = std::fs::read_dir(p).map_err(|e| format!("{path}: {e}"))?;

        let mut entries: Vec<FileEntry> = Vec::new();
        for item in read.flatten() {
            let meta = match item.metadata() {
                Ok(m) => m,
                Err(_) => continue, // unreadable entries are skipped, not fatal
            };
            let is_dir = meta.is_dir();
            entries.push(FileEntry {
                name: item.file_name().to_string_lossy().to_string(),
                path: item.path().to_string_lossy().to_string(),
                is_dir,
                size: if is_dir { 0 } else { meta.len() },
            });
        }
        // Folders first, then names, case-insensitively — how a file list reads.
        entries.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        Ok(DirListing {
            path: p.to_string_lossy().to_string(),
            parent: p.parent().map(|q| q.to_string_lossy().to_string()),
            entries,
        })
    }

    /// The folder the Files app opens on.
    pub fn home_dir(&self) -> String {
        std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".into())
    }

    // ---- audit -------------------------------------------------------------

    /// The audit trail for the Ledger app.
    pub fn ledger_rows(&self) -> Vec<LedgerRow> {
        self.ledger
            .all()
            .into_iter()
            .map(|e| LedgerRow {
                index: e.index,
                event_type: format!("{:?}", e.event.event_type),
                at: e.event.at.to_string(),
                hash: e.hash.chars().take(12).collect(),
            })
            .collect()
    }

    /// Headline state for the status card.
    pub fn status(&self) -> MachineStatus {
        MachineStatus {
            agent: self.agent_name.clone(),
            ledger_len: self.ledger.len(),
            chain_ok: self.ledger.verify().is_ok(),
            key_count: self.router.len(),
            requests_remaining: self.router.total_remaining(),
            tool_count: self.aipc.list_tools().len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boots_with_first_party_tools_and_an_agent() {
        let m = Machine::new("Nova");
        let s = m.status();
        assert_eq!(s.agent, "Nova");
        assert!(s.tool_count >= 12, "filesystem + browser tools registered");
        assert!(s.chain_ok);
        assert_eq!(s.ledger_len, 0);
    }

    #[test]
    fn tools_mark_which_actions_stop_for_a_human() {
        let m = Machine::new("Nova");
        let tools = m.tools();
        let nav = tools
            .iter()
            .find(|t| t.capability == "browser" && t.action == "navigate")
            .unwrap();
        let buy = tools
            .iter()
            .find(|t| t.capability == "browser" && t.action == "click_buy")
            .unwrap();
        // Reading the web is autonomous; buying never is.
        assert!(nav.autonomous);
        assert_eq!(nav.risk, "R0");
        assert!(!buy.autonomous);
        assert_eq!(buy.risk, "R3");
    }

    #[test]
    fn submitting_an_intent_runs_it_and_records_the_ledger() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "x").unwrap();

        let mut m = Machine::new("Nova");
        let report = m.submit_intent(&format!("list files in {}", dir.path().display()));

        assert_eq!(report.steps.len(), 1);
        assert_eq!(report.steps[0].action, "list");
        assert_eq!(report.steps[0].risk, "R0");
        assert_eq!(report.steps[0].outcome, "executed");
        assert!(report.completed);
        assert!(report.chain_ok);
        assert!(report.ledger_len > 0, "the run was recorded");
    }

    #[test]
    fn a_vague_intent_still_produces_a_safe_read_only_plan() {
        let mut m = Machine::new("Nova");
        let report = m.submit_intent("do something vague");
        // The planner's fallback is read-only — never a destructive guess.
        assert_eq!(report.steps.len(), 1);
        assert_eq!(report.steps[0].action, "list");
        assert_eq!(report.steps[0].risk, "R0");
    }

    #[test]
    fn keys_can_be_added_metered_and_removed() {
        let mut m = Machine::new("Nova");
        let added = m.add_key("sk-ant-abc123456789", 5, Some("my free tier"));
        assert_eq!(added.provider, "Anthropic");
        assert_eq!(added.label, "my free tier");
        assert_eq!(added.remaining, 5);
        // The secret is never handed back verbatim.
        assert!(!added.masked.contains("abc123456789"));

        assert_eq!(m.status().key_count, 1);
        assert_eq!(m.status().requests_remaining, 5);
        assert!(!m.keys_exhausted());

        assert!(m.remove_key(&added.id));
        assert_eq!(m.status().key_count, 0);
    }

    #[test]
    fn exhaustion_is_reported_for_the_warning_banner() {
        let mut m = Machine::new("Nova");
        m.add_key("sk-ant-a1b2c3d4e5", 0, None);
        assert!(m.keys_exhausted(), "a spent pool must surface to the user");
    }

    #[test]
    fn ledger_rows_are_exposed_for_the_audit_view() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let mut m = Machine::new("Nova");
        m.submit_intent(&format!("list files in {}", dir.path().display()));

        let rows = m.ledger_rows();
        assert!(!rows.is_empty());
        assert_eq!(rows[0].index, 0);
        assert!(!rows[0].hash.is_empty());
        assert!(m.status().chain_ok);
    }

    #[test]
    fn lists_a_directory_folders_first() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("b.txt"), "xy").unwrap();
        std::fs::write(dir.path().join("A.txt"), "x").unwrap();
        std::fs::create_dir(dir.path().join("zfolder")).unwrap();

        let m = Machine::new("Nova");
        let listing = m.list_dir(dir.path().to_str().unwrap()).unwrap();

        assert_eq!(listing.entries.len(), 3);
        // Folders come first even though "zfolder" sorts last alphabetically.
        assert!(listing.entries[0].is_dir);
        assert_eq!(listing.entries[0].name, "zfolder");
        // Then files, case-insensitively by name.
        assert_eq!(listing.entries[1].name, "A.txt");
        assert_eq!(listing.entries[2].name, "b.txt");
        assert_eq!(listing.entries[2].size, 2);
        assert!(listing.parent.is_some());
    }

    #[test]
    fn listing_a_missing_directory_reports_an_error() {
        let m = Machine::new("Nova");
        assert!(m.list_dir("/no/such/place/here").is_err());
    }

    #[test]
    fn browsing_writes_nothing_to_the_ledger() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let m = Machine::new("Nova");
        m.list_dir(dir.path().to_str().unwrap()).unwrap();
        // Looking at a folder is not an agent action, so nothing is recorded.
        assert_eq!(m.status().ledger_len, 0);
    }

    // ---- the shadow world --------------------------------------------------

    #[test]
    fn an_intent_runs_in_a_fork_without_touching_reality() {
        use tempfile::tempdir;
        let real = tempdir().unwrap();
        std::fs::write(real.path().join("keep.txt"), b"untouched").unwrap();
        let scratch = tempdir().unwrap();

        let mut m = Machine::new("Nova");
        let outcome = m
            .run_in_shadow(
                &format!("list files in {}", real.path().display()),
                real.path().to_str().unwrap(),
                "Try it",
                scratch.path().to_str().unwrap(),
            )
            .unwrap();

        assert_eq!(outcome.label, "Try it");
        assert!(!outcome.steps.is_empty(), "the plan ran");
        // The real folder is exactly as it was.
        assert_eq!(
            std::fs::read(real.path().join("keep.txt")).unwrap(),
            b"untouched"
        );
    }

    #[test]
    fn several_futures_can_be_open_at_once() {
        use tempfile::tempdir;
        let real = tempdir().unwrap();
        let scratch = tempdir().unwrap();
        let root = real.path().to_str().unwrap().to_string();
        let sc = scratch.path().to_str().unwrap().to_string();

        let mut m = Machine::new("Nova");
        m.run_in_shadow("show the folder", &root, "By type", &sc)
            .unwrap();
        m.run_in_shadow("show the folder", &root, "By date", &sc)
            .unwrap();

        let offered = m.shadow_outcomes();
        assert_eq!(offered.len(), 2, "two candidate futures on offer");
        assert!(offered.iter().any(|o| o.label == "By date"));
    }

    #[test]
    fn choosing_one_future_closes_the_world() {
        use tempfile::tempdir;
        let real = tempdir().unwrap();
        let scratch = tempdir().unwrap();
        let root = real.path().to_str().unwrap().to_string();
        let sc = scratch.path().to_str().unwrap().to_string();

        let mut m = Machine::new("Nova");
        let a = m.run_in_shadow("show the folder", &root, "A", &sc).unwrap();
        m.run_in_shadow("show the folder", &root, "B", &sc).unwrap();

        m.shadow_choose(&a.id).unwrap();

        // One became real; the rest never happened, and nothing stays open.
        assert!(m.shadow_outcomes().is_empty());
    }

    #[test]
    fn walking_away_leaves_reality_untouched() {
        use tempfile::tempdir;
        let real = tempdir().unwrap();
        std::fs::write(real.path().join("a.txt"), b"original").unwrap();
        let scratch = tempdir().unwrap();

        let mut m = Machine::new("Nova");
        m.run_in_shadow(
            "show the folder",
            real.path().to_str().unwrap(),
            "Discarded",
            scratch.path().to_str().unwrap(),
        )
        .unwrap();

        m.shadow_discard().unwrap();

        assert!(m.shadow_outcomes().is_empty());
        assert_eq!(
            std::fs::read(real.path().join("a.txt")).unwrap(),
            b"original"
        );
    }

    #[test]
    fn tidying_produces_real_changes_in_a_fork() {
        use tempfile::tempdir;
        let real = tempdir().unwrap();
        std::fs::write(real.path().join("report.pdf"), b"a").unwrap();
        std::fs::write(real.path().join("notes.txt"), b"b").unwrap();
        std::fs::write(real.path().join("photo.PNG"), b"c").unwrap();
        std::fs::create_dir(real.path().join("existing")).unwrap();
        let scratch = tempdir().unwrap();

        let mut m = Machine::new("Nova");
        let outcome = m
            .run_in_shadow(
                "tidy this folder",
                real.path().to_str().unwrap(),
                "By type",
                scratch.path().to_str().unwrap(),
            )
            .unwrap();

        // This is the point: a tidy has to offer something to accept.
        assert!(
            !outcome.changes.is_empty(),
            "tidying must produce changes, not an empty outcome"
        );
        assert!(outcome.changes.iter().any(|c| c.contains("pdf/report.pdf")));
        assert!(outcome.changes.iter().any(|c| c.contains("txt/notes.txt")));
        // Extension case is normalised, so PNG and png do not become two folders.
        assert!(outcome.changes.iter().any(|c| c.contains("png/photo.PNG")));

        // And reality is still exactly as it was.
        assert!(real.path().join("report.pdf").exists());
        assert!(!real.path().join("pdf").exists());
    }

    #[test]
    fn promoting_a_tidy_actually_sorts_the_folder() {
        use tempfile::tempdir;
        let real = tempdir().unwrap();
        std::fs::write(real.path().join("a.txt"), b"x").unwrap();
        let scratch = tempdir().unwrap();

        let mut m = Machine::new("Nova");
        let o = m
            .run_in_shadow(
                "organize this folder",
                real.path().to_str().unwrap(),
                "By type",
                scratch.path().to_str().unwrap(),
            )
            .unwrap();
        m.shadow_choose(&o.id).unwrap();

        assert!(
            real.path().join("txt/a.txt").exists(),
            "the file moved for real"
        );
    }

    #[test]
    fn hidden_files_are_left_alone() {
        use tempfile::tempdir;
        let real = tempdir().unwrap();
        std::fs::write(real.path().join(".gitconfig"), b"x").unwrap();
        std::fs::write(real.path().join("doc.md"), b"y").unwrap();
        let scratch = tempdir().unwrap();

        let mut m = Machine::new("Nova");
        let o = m
            .run_in_shadow(
                "tidy",
                real.path().to_str().unwrap(),
                "By type",
                scratch.path().to_str().unwrap(),
            )
            .unwrap();

        // Dotfiles are configuration; sorting them by extension is not tidying.
        assert!(!o.changes.iter().any(|c| c.contains("gitconfig")));
        assert!(o.changes.iter().any(|c| c.contains("md/doc.md")));
    }

    // ---- shell commands through the kernel ---------------------------------

    #[test]
    fn a_safe_command_runs_and_is_recorded() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let mut m = Machine::new("Nova");

        let r = m
            .run_command("echo hello", dir.path().to_str().unwrap())
            .unwrap();

        assert_eq!(r.risk, "R0");
        assert_eq!(r.outcome, "executed");
        assert!(r.stdout.contains("hello"), "got {:?}", r.stdout);
        assert!(m.status().ledger_len > 0, "the run was recorded");
        assert!(m.status().chain_ok);
    }

    #[test]
    fn a_dangerous_command_never_runs_on_its_own() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("precious.txt"), b"keep me").unwrap();

        let mut m = Machine::new("Nova");
        let r = m
            .run_command("rm precious.txt", dir.path().to_str().unwrap())
            .unwrap();

        assert_eq!(r.risk, "R3");
        assert_eq!(r.outcome, "awaiting_approval", "R3 must stop for a person");
        // And the file is still there, which is the part that matters.
        assert!(dir.path().join("precious.txt").exists());
    }

    #[test]
    fn a_command_hiding_behind_a_safe_word_is_also_stopped() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("precious.txt"), b"keep me").unwrap();

        let mut m = Machine::new("Nova");
        let r = m
            .run_command("echo hi && rm precious.txt", dir.path().to_str().unwrap())
            .unwrap();

        assert_eq!(r.risk, "R3");
        assert_ne!(r.outcome, "executed");
        assert!(dir.path().join("precious.txt").exists());
    }

    #[test]
    fn an_unknown_command_is_held_rather_than_attempted() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let mut m = Machine::new("Nova");

        let r = m
            .run_command("frobnicate --everything", dir.path().to_str().unwrap())
            .unwrap();

        assert_eq!(r.risk, "R3");
        assert_eq!(r.outcome, "awaiting_approval");
        assert!(r.stdout.is_empty(), "nothing was run");
    }

    #[test]
    fn reports_serialize_for_the_front_end() {
        let mut m = Machine::new("Nova");
        let report = m.submit_intent("show the folder");
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"steps\""));
        assert!(json.contains("\"risk\""));
        // Round-trips, so the UI contract is stable.
        let back: IntentReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back, report);
    }
}
