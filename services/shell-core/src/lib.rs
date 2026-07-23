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
use cmd_kernel::{AuthorityContext, Kernel, StepOutcome};
use cmd_ledger::Ledger;
use cmd_router::{ApiKey, KeyRouter};
use cmd_types::{now, Id, Mandate, PlanStep, RiskClass};
use serde::{Deserialize, Serialize};

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
            capabilities: vec!["filesystem".into(), "browser".into()],
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
        let (intent, plan) = agent.plan_for(text);

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
