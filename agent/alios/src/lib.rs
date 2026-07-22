//! # alios — the user agent: planning and running
//!
//! This crate turns an [`Intent`] into an [`ExecutionPlan`] and runs it through
//! the kernel. It closes the canonical loop end to end:
//! **Intent → Planning** → Permission → Execution → Verification → Result.
//!
//! Planning is behind a [`Planner`] trait. This crate ships a deterministic,
//! rule-based planner (`RulePlanner`) so the whole pipeline runs and tests today
//! without an AI. A real model-backed planner (calling Claude, as the prototype's
//! `anthropic-planner` does) implements the same trait and drops in unchanged —
//! it just can't run in CI, so it lives behind the same interface.
//!
//! Reference: `prototype/kernel/mock-planner.ts`.
//!
//! Defined by RFC-0013.

use cmd_types::{now, ExecutionPlan, Id, Intent, IntentSource, PlanStatus, PlanStep, StepStatus};
use std::collections::BTreeMap;

pub mod claude_planner;
pub mod routed_transport;
pub use claude_planner::{ClaudePlanner, ClaudeTransport, PlanError};
pub use routed_transport::RoutedTransport;

/// Anything that can turn an intent into a plan. Deterministic implementations
/// are testable; model-backed ones satisfy the same contract.
pub trait Planner {
    /// Produce an execution plan for the given intent.
    fn plan(&self, intent: &Intent) -> ExecutionPlan;
}

/// A deterministic, rule-based planner. It reads simple keywords from the
/// intent's raw text and produces a matching filesystem plan. Not an AI — just
/// enough to prove and test the full pipeline, and a stand-in until a
/// model-backed planner is wired in.
#[derive(Default)]
pub struct RulePlanner;

impl RulePlanner {
    pub fn new() -> Self {
        RulePlanner
    }
}

impl Planner for RulePlanner {
    fn plan(&self, intent: &Intent) -> ExecutionPlan {
        let text = intent.raw_text.to_lowercase();
        let mut steps = Vec::new();

        // Very small rule set, matching the prototype's spirit. A real planner
        // replaces this with model reasoning; the output shape is identical.
        if text.contains("list") || text.contains("what's in") || text.contains("show") {
            steps.push(step(
                "List the target folder",
                "filesystem",
                "list",
                &[(
                    "path",
                    param_after(&text, "in").unwrap_or_else(|| ".".into()),
                )],
                vec![],
            ));
        }

        if text.contains("rename") {
            steps.push(step(
                "Rename the file",
                "filesystem",
                "rename",
                &[("from", "".into()), ("to", "".into())],
                vec![],
            ));
        }

        // Fallback: if nothing matched, produce a safe read-only list of ".".
        if steps.is_empty() {
            steps.push(step(
                "Inspect the current folder (fallback)",
                "filesystem",
                "list",
                &[("path", ".".into())],
                vec![],
            ));
        }

        ExecutionPlan {
            id: Id::new(),
            intent_id: intent.id,
            created_at: now(),
            status: PlanStatus::Draft,
            summary: format!("Plan for: {}", intent.raw_text),
            steps,
        }
    }
}

/// Convenience: build a [`PlanStep`].
fn step(
    description: &str,
    capability: &str,
    action: &str,
    params: &[(&str, String)],
    depends_on: Vec<Id>,
) -> PlanStep {
    let mut parameters = BTreeMap::new();
    for (k, v) in params {
        parameters.insert((*k).to_string(), serde_json::Value::String(v.clone()));
    }
    PlanStep {
        id: Id::new(),
        description: description.into(),
        capability: capability.into(),
        action: action.into(),
        parameters,
        depends_on,
        requires_permission: false,
        status: StepStatus::Pending,
        error: None,
    }
}

/// Extract the word following `marker` in `text` (a crude "path after 'in'"
/// heuristic for the rule planner; a real planner needs none of this).
fn param_after(text: &str, marker: &str) -> Option<String> {
    let mut words = text.split_whitespace();
    while let Some(w) = words.next() {
        if w == marker {
            return words.next().map(|s| s.to_string());
        }
    }
    None
}

/// A user agent: a name plus a planner. It turns a raw request into a plan.
pub struct Agent<P: Planner> {
    pub name: String,
    planner: P,
}

impl<P: Planner> Agent<P> {
    /// Create an agent with the given name and planner.
    pub fn new(name: impl Into<String>, planner: P) -> Self {
        Agent {
            name: name.into(),
            planner,
        }
    }

    /// Understand a raw request into an [`Intent`], then plan it.
    ///
    /// (Understanding is trivial here — wrap the text; a real system does NLU.)
    pub fn plan_for(&self, raw_request: &str) -> (Intent, ExecutionPlan) {
        let intent = Intent::received(raw_request, IntentSource::UserCommand);
        let plan = self.planner.plan(&intent);
        (intent, plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cmd_kernel::{AuthorityContext, Kernel, StepOutcome};
    use cmd_ledger::Ledger;
    use cmd_types::{Mandate, RiskClass};

    fn mandate_for(caps: &[&str]) -> Mandate {
        Mandate {
            id: Id::new(),
            agent_id: Id::new(),
            scope: "test".into(),
            capabilities: caps.iter().map(|c| c.to_string()).collect(),
            max_autonomous_risk: RiskClass::R1Reversible,
            budget_id: None,
            granted_at: now(),
            expires_at: None,
            revoked_at: None,
        }
    }

    #[test]
    fn planner_produces_a_list_step_for_a_list_intent() {
        let agent = Agent::new("Nova", RulePlanner::new());
        let (intent, plan) = agent.plan_for("list what's in my documents folder");
        assert_eq!(plan.intent_id, intent.id);
        assert!(plan.steps.iter().any(|s| s.action == "list"));
    }

    #[test]
    fn planner_falls_back_to_a_safe_read_only_step() {
        let agent = Agent::new("Nova", RulePlanner::new());
        let (_intent, plan) = agent.plan_for("do something vague");
        // Fallback is a read-only list — never a destructive guess.
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].action, "list");
    }

    #[test]
    fn plan_intent_id_links_back_to_the_intent() {
        let agent = Agent::new("Nova", RulePlanner::new());
        let (intent, plan) = agent.plan_for("show the folder");
        assert_eq!(plan.intent_id, intent.id);
        assert_eq!(intent.source, IntentSource::UserCommand);
    }

    // The full agent loop, end to end, on a real file: the agent plans a rename,
    // the kernel runs it under Alios supervision, the file changes, and the
    // ledger records an intact chain. Intent → Plan → Permission → Execution →
    // Verification → Result — all in Rust, driven by the agent.
    #[test]
    fn agent_plans_and_kernel_runs_a_real_rename_end_to_end() {
        use cap_files::FileSystem;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let src = dir.path().join("draft.txt");
        std::fs::write(&src, "hello").unwrap();

        // Build the plan by hand-parametrizing a rename (the rule planner does not
        // fill real paths; a model planner would). We use the agent's planner to
        // produce the shape, then set concrete parameters — this mirrors how a
        // real planner emits fully-specified steps.
        let agent = Agent::new("Nova", RulePlanner::new());
        let (_intent, mut plan) = agent.plan_for("rename the draft");
        // Point the rename at the real file.
        let s = plan
            .steps
            .iter_mut()
            .find(|s| s.action == "rename")
            .expect("rename step");
        s.parameters
            .insert("from".into(), serde_json::json!(src.to_str().unwrap()));
        s.parameters
            .insert("to".into(), serde_json::json!("final.txt"));

        let m = mandate_for(&["filesystem"]);
        let ctx = AuthorityContext {
            mandate: Some(&m),
            budget: None,
        };

        let mut ledger = Ledger::new();
        let mut fs_cap = FileSystem::new();
        let run = {
            let mut k = Kernel::new(&mut ledger);
            k.run_plan(&plan, &mut fs_cap, &ctx, &|_s| RiskClass::R1Reversible)
        };

        assert!(run.completed, "agent's plan should complete");
        assert!(run.steps.iter().all(|(_, o)| *o == StepOutcome::Executed));
        assert!(dir.path().join("final.txt").exists());
        assert!(!src.exists());
        assert!(ledger.verify().is_ok());
    }
}
