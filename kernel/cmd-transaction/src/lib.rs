//! # cmd-transaction — the reversible-execution engine
//!
//! The heart of cmdOS. Every side-effecting action runs through this engine:
//! simulate → snapshot → execute → verify → commit | rollback. Reversibility,
//! dry-run, and undo are all expressed here.
//!
//! The engine is abstract: it drives the [`Resource`] trait and knows nothing
//! about files or VMs. This crate ships an in-memory `MemoryResource` used only
//! for tests; the real filesystem resource (which must satisfy the prototype's
//! 8 behavior contracts in `prototype/tests/`) lands in `capabilities/files`.
//!
//! Defined by RFC-0008.

pub mod engine;
pub mod resource;

pub use engine::{EngineError, Outcome, TransactionEngine};
pub use resource::{Resource, ResourceError, Snapshot};

#[cfg(test)]
mod tests {
    use super::*;
    use cmd_ledger::Ledger;
    use cmd_types::{EventType, Id, PlanStep, RiskClass, StepStatus};
    use std::collections::BTreeMap;

    // ---- A minimal in-memory resource: a key/value "world" -----------------

    /// The snapshot is a full copy of the world map.
    #[derive(Clone)]
    struct MemSnapshot(BTreeMap<String, String>);
    impl Snapshot for MemSnapshot {}

    /// A tiny world the engine can act on. Steps carry `key`/`value` params and
    /// an action of `"set"` (write) or `"read"` (no-op). A step may be told to
    /// fail verification via a `"break_verify"` param, to exercise rollback.
    struct MemResource {
        world: BTreeMap<String, String>,
        /// If true, `execute` returns an error (to test the execute-failure path).
        fail_execute: bool,
    }

    impl MemResource {
        fn new() -> Self {
            MemResource {
                world: BTreeMap::new(),
                fail_execute: false,
            }
        }
    }

    fn param<'a>(step: &'a PlanStep, k: &str) -> Option<&'a str> {
        step.parameters.get(k).and_then(|v| v.as_str())
    }

    impl Resource for MemResource {
        type Snap = MemSnapshot;

        fn simulate(&self, step: &PlanStep) -> Result<String, ResourceError> {
            Ok(format!("would {} {:?}", step.action, step.parameters))
        }

        fn snapshot(&self, _step: &PlanStep) -> Result<Option<Self::Snap>, ResourceError> {
            Ok(Some(MemSnapshot(self.world.clone())))
        }

        fn execute(&mut self, step: &PlanStep) -> Result<(), ResourceError> {
            if self.fail_execute {
                return Err(ResourceError::Failed("execute forced to fail".into()));
            }
            if step.action == "set" {
                let k = param(step, "key").unwrap_or_default().to_string();
                let v = param(step, "value").unwrap_or_default().to_string();
                self.world.insert(k, v);
            }
            Ok(())
        }

        fn verify(&self, step: &PlanStep) -> Result<bool, ResourceError> {
            // A step can be told to fail verification to exercise rollback.
            // `break_verify` is a bool param, so check for the key's presence.
            if step.parameters.contains_key("break_verify") {
                return Ok(false);
            }
            if step.action == "set" {
                let k = param(step, "key").unwrap_or_default();
                let v = param(step, "value").unwrap_or_default();
                return Ok(self.world.get(k).map(|x| x.as_str()) == Some(v));
            }
            Ok(true) // read-only verifies trivially
        }

        fn restore(&mut self, snapshot: Self::Snap) -> Result<(), ResourceError> {
            self.world = snapshot.0;
            Ok(())
        }
    }

    // ---- Helpers -----------------------------------------------------------

    fn set_step(key: &str, value: &str) -> PlanStep {
        let mut p = BTreeMap::new();
        p.insert("key".into(), serde_json::json!(key));
        p.insert("value".into(), serde_json::json!(value));
        PlanStep {
            id: Id::new(),
            description: format!("set {key}={value}"),
            capability: "mem".into(),
            action: "set".into(),
            parameters: p,
            depends_on: vec![],
            requires_permission: false,
            status: StepStatus::Pending,
            error: None,
        }
    }

    fn read_step() -> PlanStep {
        PlanStep {
            id: Id::new(),
            description: "read".into(),
            capability: "mem".into(),
            action: "read".into(),
            parameters: BTreeMap::new(),
            depends_on: vec![],
            requires_permission: false,
            status: StepStatus::Pending,
            error: None,
        }
    }

    // ---- Tests -------------------------------------------------------------

    #[test]
    fn successful_step_commits_and_changes_state() {
        let mut ledger = Ledger::new();
        let mut res = MemResource::new();
        let step = set_step("a", "1");

        let out = {
            let mut engine = TransactionEngine::new(&mut ledger);
            engine
                .run(&mut res, &step, RiskClass::R1Reversible)
                .expect("should commit")
        };

        assert_eq!(
            out.transaction.phase,
            cmd_types::TransactionPhase::Committed
        );
        assert_eq!(res.world.get("a"), Some(&"1".to_string()));
        // Ledger recorded a committed transaction.
        assert_eq!(ledger.by_type(EventType::TransactionCommitted).len(), 1);
        assert!(ledger.verify().is_ok());
    }

    #[test]
    fn verify_failure_auto_rolls_back_to_pre_state() {
        let mut ledger = Ledger::new();
        let mut res = MemResource::new();
        res.world.insert("a".into(), "original".into());

        // A set step whose verification is forced to fail.
        let mut step = set_step("a", "changed");
        step.parameters
            .insert("break_verify".into(), serde_json::json!(true));

        let err = {
            let mut engine = TransactionEngine::new(&mut ledger);
            engine
                .run(&mut res, &step, RiskClass::R1Reversible)
                .unwrap_err()
        };

        // Engine reports verification failure...
        assert!(matches!(err, EngineError::VerificationFailed(_)));
        // ...and the world was restored to exactly its pre-state.
        assert_eq!(res.world.get("a"), Some(&"original".to_string()));
        assert_eq!(ledger.by_type(EventType::TransactionRolledBack).len(), 1);
    }

    #[test]
    fn read_only_step_takes_no_snapshot() {
        let mut ledger = Ledger::new();
        let mut res = MemResource::new();
        let step = read_step();

        let out = {
            let mut engine = TransactionEngine::new(&mut ledger);
            engine
                .run(&mut res, &step, RiskClass::R0ReadOnly)
                .expect("read commits")
        };
        // A read-only action takes no snapshot (there is nothing to restore).
        // It is still "reversible" in the trivial sense — R0 changed nothing —
        // but crucially it holds no snapshot_ref.
        assert!(out.transaction.snapshot_ref.is_none());
        assert_eq!(
            out.transaction.phase,
            cmd_types::TransactionPhase::Committed
        );
    }

    #[test]
    fn execute_failure_returns_error_without_commit() {
        let mut ledger = Ledger::new();
        let mut res = MemResource::new();
        res.fail_execute = true;
        let step = set_step("a", "1");

        let err = {
            let mut engine = TransactionEngine::new(&mut ledger);
            engine
                .run(&mut res, &step, RiskClass::R1Reversible)
                .unwrap_err()
        };
        assert!(matches!(err, EngineError::Resource(_)));
        // Nothing committed.
        assert_eq!(ledger.by_type(EventType::TransactionCommitted).len(), 0);
        assert!(res.world.get("a").is_none());
    }

    #[test]
    fn explicit_rollback_restores_and_records() {
        let mut ledger = Ledger::new();
        let mut res = MemResource::new();
        res.world.insert("a".into(), "before".into());
        let step = set_step("a", "after");

        // Commit a change, keeping the pre-state snapshot ourselves.
        let pre = MemSnapshot(res.world.clone());
        let out = {
            let mut engine = TransactionEngine::new(&mut ledger);
            engine
                .run(&mut res, &step, RiskClass::R1Reversible)
                .expect("commit")
        };
        assert_eq!(res.world.get("a"), Some(&"after".to_string()));

        // Now the user hits Undo.
        let mut engine = TransactionEngine::new(&mut ledger);
        let rolled = engine
            .rollback(&mut res, out.transaction, pre, &step)
            .expect("rollback ok");
        assert_eq!(rolled.phase, cmd_types::TransactionPhase::RolledBack);
        assert_eq!(res.world.get("a"), Some(&"before".to_string()));
    }

    #[test]
    fn rollback_refused_for_irreversible_transaction() {
        let mut ledger = Ledger::new();
        let mut res = MemResource::new();
        let step = read_step();

        // A read-only run yields a non-reversible transaction (no snapshot).
        let out = {
            let mut engine = TransactionEngine::new(&mut ledger);
            engine
                .run(&mut res, &step, RiskClass::R0ReadOnly)
                .expect("commit")
        };
        let mut engine = TransactionEngine::new(&mut ledger);
        let err = engine
            .rollback(
                &mut res,
                out.transaction,
                MemSnapshot(Default::default()),
                &step,
            )
            .unwrap_err();
        assert!(matches!(err, EngineError::NotReversible));
    }

    #[test]
    fn ledger_chain_stays_intact_across_transactions() {
        let mut ledger = Ledger::new();
        let mut res = MemResource::new();
        {
            let mut engine = TransactionEngine::new(&mut ledger);
            engine
                .run(&mut res, &set_step("a", "1"), RiskClass::R1Reversible)
                .unwrap();
            engine
                .run(&mut res, &set_step("b", "2"), RiskClass::R1Reversible)
                .unwrap();
        }
        // Every event across both transactions forms one intact hash chain.
        assert!(ledger.verify().is_ok());
        assert!(ledger.len() >= 8); // ~4 events per committed transaction
    }
}
