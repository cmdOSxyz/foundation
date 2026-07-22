//! # cmd-kernel — the Intent Scheduler
//!
//! The crate that makes cmdOS a *system* rather than a set of parts. It runs an
//! [`ExecutionPlan`] to completion by walking its steps in dependency order and,
//! for each one:
//!
//! 1. asks **cmd-policy** whether it may run (Alios's supervision),
//! 2. if allowed, runs it through **cmd-transaction** (reversibly), which records
//!    to **cmd-ledger**,
//! 3. if it needs approval or is blocked, stops and reports — never runs a gated
//!    action autonomously.
//!
//! This is the canonical loop in code:
//! Intent → Planning → **Permission → Execution → Verification** → Result.
//!
//! Defined by RFC-0011.

use cmd_ledger::Ledger;
use cmd_policy::{Decision, PolicyEngine, ProposedAction};
use cmd_transaction::{Resource, TransactionEngine};
use cmd_types::{Budget, ExecutionPlan, Id, Mandate, PlanStep, RiskClass, StepStatus};

/// How the kernel learns the risk class of a step. In the full system this comes
/// from the capability contract (`CapabilityAction::risk`); the kernel takes it
/// as a resolver so it stays decoupled from any capability registry.
pub type RiskResolver<'a> = dyn Fn(&PlanStep) -> RiskClass + 'a;

/// The authority context a plan runs under: the acting agent's mandate and budget.
pub struct AuthorityContext<'a> {
    pub mandate: Option<&'a Mandate>,
    pub budget: Option<&'a Budget>,
}

/// What happened to one step.
#[derive(Debug, Clone, PartialEq)]
pub enum StepOutcome {
    /// Ran and committed.
    Executed,
    /// Gated: stopped for human approval, with the reason.
    AwaitingApproval(String),
    /// Refused by policy, with the reason.
    Blocked(String),
    /// Ran but failed (execution or verification).
    Failed(String),
}

/// The result of running a whole plan.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanRun {
    /// Per-step outcome, in the order the kernel processed them.
    pub steps: Vec<(Id, StepOutcome)>,
    /// True if every step executed successfully.
    pub completed: bool,
}

/// The kernel: schedules and runs plans. Borrows the ledger every run records to.
pub struct Kernel<'a> {
    ledger: &'a mut Ledger,
}

impl<'a> Kernel<'a> {
    /// A kernel that records into `ledger`.
    pub fn new(ledger: &'a mut Ledger) -> Self {
        Kernel { ledger }
    }

    /// Run a plan to completion (or until a step is gated/blocked/fails) against
    /// a single resource, under the given authority, using `risk_of` to classify
    /// each step.
    ///
    /// Dependency order: a step runs only once all steps it depends on have
    /// executed. Processing stops at the first step that awaits approval, is
    /// blocked, or fails — the kernel never runs a gated action autonomously and
    /// never proceeds past an unmet dependency.
    pub fn run_plan<R: Resource>(
        &mut self,
        plan: &ExecutionPlan,
        resource: &mut R,
        authority: &AuthorityContext,
        risk_of: &RiskResolver,
    ) -> PlanRun {
        let policy = PolicyEngine::now();
        let mut completed_ids: Vec<Id> = Vec::new();
        let mut outcomes: Vec<(Id, StepOutcome)> = Vec::new();

        // Work on a mutable copy of step statuses so we can track readiness.
        let mut plan = plan.clone();

        loop {
            // Find the next ready step (deps satisfied, still pending).
            let next = plan.ready_steps(&completed_ids).first().map(|s| s.id);
            let step_id = match next {
                Some(id) => id,
                None => break, // nothing left runnable
            };
            // Index of that step in the plan.
            let idx = plan.steps.iter().position(|s| s.id == step_id).unwrap();
            let step = plan.steps[idx].clone();

            let risk = risk_of(&step);
            let action = ProposedAction {
                capability: step.capability.clone(),
                risk,
                spend: 0, // spend integration arrives with cmdpay
            };

            match policy.evaluate(&action, authority.mandate, authority.budget) {
                Decision::Block { reason } => {
                    outcomes.push((step_id, StepOutcome::Blocked(reason)));
                    break; // do not proceed past a blocked step
                }
                Decision::NeedsApproval { reason } => {
                    outcomes.push((step_id, StepOutcome::AwaitingApproval(reason)));
                    break; // stop and wait for the human
                }
                Decision::Allow { .. } => {
                    let mut engine = TransactionEngine::new(self.ledger);
                    match engine.run(resource, &step, risk) {
                        Ok(_) => {
                            plan.steps[idx].status = StepStatus::Succeeded;
                            completed_ids.push(step_id);
                            outcomes.push((step_id, StepOutcome::Executed));
                        }
                        Err(e) => {
                            plan.steps[idx].status = StepStatus::Failed;
                            outcomes.push((step_id, StepOutcome::Failed(e.to_string())));
                            break; // stop on failure
                        }
                    }
                }
            }
        }

        let completed = outcomes.len() == plan.steps.len()
            && outcomes.iter().all(|(_, o)| *o == StepOutcome::Executed);

        PlanRun {
            steps: outcomes,
            completed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cmd_types::{now, ExecutionPlan, PlanStatus, PlanStep, RiskClass, StepStatus};
    use std::collections::BTreeMap;

    // A trivial resource that always succeeds, to test scheduling/policy flow
    // without touching disk.
    struct OkResource;
    struct OkSnap;
    impl cmd_transaction::Snapshot for OkSnap {}
    impl Resource for OkResource {
        type Snap = OkSnap;
        fn simulate(&self, _s: &PlanStep) -> Result<String, cmd_transaction::ResourceError> {
            Ok("ok".into())
        }
        fn snapshot(
            &self,
            _s: &PlanStep,
        ) -> Result<Option<Self::Snap>, cmd_transaction::ResourceError> {
            Ok(Some(OkSnap))
        }
        fn execute(&mut self, _s: &PlanStep) -> Result<(), cmd_transaction::ResourceError> {
            Ok(())
        }
        fn verify(&self, _s: &PlanStep) -> Result<bool, cmd_transaction::ResourceError> {
            Ok(true)
        }
        fn restore(&mut self, _s: Self::Snap) -> Result<(), cmd_transaction::ResourceError> {
            Ok(())
        }
    }

    fn step(id: Id, cap: &str, action: &str, deps: Vec<Id>) -> PlanStep {
        PlanStep {
            id,
            description: action.into(),
            capability: cap.into(),
            action: action.into(),
            parameters: BTreeMap::new(),
            depends_on: deps,
            requires_permission: false,
            status: StepStatus::Pending,
            error: None,
        }
    }

    fn plan(steps: Vec<PlanStep>) -> ExecutionPlan {
        ExecutionPlan {
            id: Id::new(),
            intent_id: Id::new(),
            created_at: now(),
            status: PlanStatus::Approved,
            steps,
            summary: "test".into(),
        }
    }

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
    fn runs_all_allowed_steps_in_dependency_order() {
        let a = Id::new();
        let b = Id::new();
        let p = plan(vec![
            step(b, "filesystem", "rename", vec![a]), // depends on a; listed first
            step(a, "filesystem", "list", vec![]),
        ]);
        let m = mandate_for(&["filesystem"]);
        let ctx = AuthorityContext {
            mandate: Some(&m),
            budget: None,
        };

        let mut ledger = Ledger::new();
        let mut res = OkResource;
        let run = {
            let mut k = Kernel::new(&mut ledger);
            k.run_plan(&p, &mut res, &ctx, &|_s| RiskClass::R1Reversible)
        };

        assert!(run.completed);
        assert_eq!(run.steps.len(), 2);
        // a ran before b (dependency order), even though b was listed first.
        assert_eq!(run.steps[0].0, a);
        assert_eq!(run.steps[1].0, b);
        assert!(ledger.verify().is_ok());
    }

    #[test]
    fn stops_at_a_step_needing_approval() {
        let a = Id::new();
        let p = plan(vec![step(a, "cmdpay", "pay", vec![])]);
        let m = mandate_for(&["cmdpay"]);
        let ctx = AuthorityContext {
            mandate: Some(&m),
            budget: None,
        };

        let mut ledger = Ledger::new();
        let mut res = OkResource;
        // Classify the step as R3 → must need approval.
        let run = {
            let mut k = Kernel::new(&mut ledger);
            k.run_plan(&p, &mut res, &ctx, &|_s| RiskClass::R3Irreversible)
        };

        assert!(!run.completed);
        assert!(matches!(run.steps[0].1, StepOutcome::AwaitingApproval(_)));
    }

    #[test]
    fn blocks_a_step_outside_the_mandate() {
        let a = Id::new();
        let p = plan(vec![step(a, "filesystem", "rename", vec![])]);
        // Mandate covers only "browser", not "filesystem".
        let m = mandate_for(&["browser"]);
        let ctx = AuthorityContext {
            mandate: Some(&m),
            budget: None,
        };

        let mut ledger = Ledger::new();
        let mut res = OkResource;
        let run = {
            let mut k = Kernel::new(&mut ledger);
            k.run_plan(&p, &mut res, &ctx, &|_s| RiskClass::R1Reversible)
        };

        assert!(!run.completed);
        assert!(matches!(run.steps[0].1, StepOutcome::Blocked(_)));
    }

    // The vertical slice: a real intent runs end-to-end through the whole kernel
    // against the real filesystem capability. Intent → Permission → Execution →
    // Verification → Result, all in Rust.
    #[test]
    fn vertical_slice_rename_a_real_file_end_to_end() {
        use cap_files::FileSystem;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let src = dir.path().join("draft.txt");
        std::fs::write(&src, "hello").unwrap();

        let mut params = BTreeMap::new();
        params.insert("from".into(), serde_json::json!(src.to_str().unwrap()));
        params.insert("to".into(), serde_json::json!("final.txt"));
        let s = PlanStep {
            id: Id::new(),
            description: "rename draft to final".into(),
            capability: "filesystem".into(),
            action: "rename".into(),
            parameters: params,
            depends_on: vec![],
            requires_permission: false,
            status: StepStatus::Pending,
            error: None,
        };
        let p = plan(vec![s]);

        let m = mandate_for(&["filesystem"]);
        let ctx = AuthorityContext {
            mandate: Some(&m),
            budget: None,
        };

        let mut ledger = Ledger::new();
        let mut fs_cap = FileSystem::new();
        let run = {
            let mut k = Kernel::new(&mut ledger);
            k.run_plan(&p, &mut fs_cap, &ctx, &|_s| RiskClass::R1Reversible)
        };

        // The whole system worked: the plan completed, the file was renamed,
        // and the ledger holds an intact, verifiable record of it.
        assert!(run.completed, "vertical slice should complete");
        assert!(dir.path().join("final.txt").exists());
        assert!(!src.exists());
        assert!(ledger.verify().is_ok());
        assert!(ledger.len() >= 4, "ledger recorded the transaction phases");
    }
}
