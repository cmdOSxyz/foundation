//! # cmd-types — the cmdOS object model
//!
//! The shared "nouns" of cmdOS. Every other crate depends on these and nothing
//! else in the kernel. Defined by **RFC-0004**.
//!
//! The model has two layers:
//!
//! - **Ported from the prototype** (the TypeScript `schemas/`, which remain the
//!   reference): [`Intent`], [`ExecutionPlan`]/[`PlanStep`], [`Capability`],
//!   [`PermissionRequest`], [`Event`].
//! - **New in Strategy v2**: [`RiskClass`] (R0–R3), [`Transaction`],
//!   [`Mandate`], [`Budget`], and the two-tier [`Agent`] model.
//!
//! Core loop these types serve:
//! Intent → Understanding → Planning → Permission → Execution → Verification → Result.
//!
//! Legacy kernel-manager mapping (from the archived roadmap) lives in RFC-0004.

pub mod agent;
pub mod authority;
pub mod capability;
pub mod common;
pub mod event;
pub mod intent;
pub mod plan;
pub mod transaction;

// Flat re-exports so downstream crates can `use cmd_types::Intent;`.
pub use agent::{Agent, AgentKind, PermissionDecision, PermissionRequest, TrustLevel};
pub use authority::{Budget, Mandate};
pub use capability::{Capability, CapabilityAction};
pub use common::{now, Id, RiskClass, Timestamp};
pub use event::{Event, EventType};
pub use intent::{Intent, IntentSource, IntentStatus, Objective};
pub use plan::{ExecutionPlan, PlanStatus, PlanStep, StepStatus};
pub use transaction::{Transaction, TransactionPhase};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn risk_class_governs_autonomy_and_reversibility() {
        assert!(RiskClass::R0ReadOnly.may_be_autonomous());
        assert!(RiskClass::R1Reversible.may_be_autonomous());
        assert!(RiskClass::R2Compensable.may_be_autonomous());
        // The load-bearing rule: R3 can never be autonomous.
        assert!(!RiskClass::R3Irreversible.may_be_autonomous());

        assert!(RiskClass::R0ReadOnly.is_reversible());
        assert!(RiskClass::R1Reversible.is_reversible());
        assert!(!RiskClass::R2Compensable.is_reversible());
        assert!(!RiskClass::R3Irreversible.is_reversible());
    }

    #[test]
    fn intent_starts_received_without_objective() {
        let i = Intent::received("clean my downloads", IntentSource::UserCommand);
        assert_eq!(i.status, IntentStatus::Received);
        assert!(i.objective.is_none());
    }

    #[test]
    fn budget_enforces_money_ceiling() {
        let mut b = Budget {
            id: Id::new(),
            agent_id: Id::new(),
            money_ceiling: Some(100),
            money_spent: 90,
            currency: Some("USD".into()),
            action_ceiling: None,
            actions_taken: 0,
            expires_at: None,
        };
        assert!(b.can_spend(10)); // 90 + 10 == 100, within ceiling
        assert!(!b.can_spend(11)); // would exceed
        b.money_spent = 100;
        assert!(!b.can_spend(1)); // no headroom
    }

    #[test]
    fn budget_without_money_ceiling_cannot_spend() {
        let b = Budget {
            id: Id::new(),
            agent_id: Id::new(),
            money_ceiling: None,
            money_spent: 0,
            currency: None,
            action_ceiling: None,
            actions_taken: 0,
            expires_at: None,
        };
        assert!(!b.can_spend(1));
    }

    #[test]
    fn mandate_active_unless_revoked_or_expired() {
        let now = now();
        let mut m = Mandate {
            id: Id::new(),
            agent_id: Id::new(),
            scope: "research".into(),
            capabilities: vec!["browser".into()],
            max_autonomous_risk: RiskClass::R1Reversible,
            budget_id: None,
            granted_at: now,
            expires_at: None,
            revoked_at: None,
        };
        assert!(m.is_active(now));
        assert!(m.authorizes_capability("browser"));
        assert!(!m.authorizes_capability("cmdpay"));
        m.revoked_at = Some(now);
        assert!(!m.is_active(now));
    }

    #[test]
    fn transaction_rollback_requires_snapshot() {
        let mut t = Transaction {
            id: Id::new(),
            plan_id: Id::new(),
            step_id: Id::new(),
            phase: TransactionPhase::Executed,
            snapshot_ref: Some("cas:abc123".into()),
            reversible: true,
            created_at: now(),
            error: None,
        };
        assert!(t.can_roll_back());
        t.snapshot_ref = None; // read-only action, nothing to restore
        assert!(!t.can_roll_back());
    }

    #[test]
    fn supervisor_is_distinguished_from_user_agent() {
        let nova = Agent::new_user("Nova");
        assert!(!nova.is_supervisor());
        assert_eq!(nova.trust, TrustLevel::Calm);

        let alios = Agent {
            id: Id::new(),
            kind: AgentKind::Supervisor,
            name: "Alios".into(),
            avatar: None,
            role: Some("supervisor".into()),
            trust: TrustLevel::Calm,
            created_at: now(),
        };
        assert!(alios.is_supervisor());
    }

    #[test]
    fn plan_ready_steps_respects_dependencies() {
        let a = Id::new();
        let step_a = PlanStep {
            id: a,
            description: "first".into(),
            capability: "filesystem".into(),
            action: "list".into(),
            parameters: Default::default(),
            depends_on: vec![],
            requires_permission: false,
            status: StepStatus::Pending,
            error: None,
        };
        let step_b = PlanStep {
            id: Id::new(),
            description: "second".into(),
            capability: "filesystem".into(),
            action: "rename".into(),
            parameters: Default::default(),
            depends_on: vec![a],
            requires_permission: false,
            status: StepStatus::Pending,
            error: None,
        };
        let plan = ExecutionPlan {
            id: Id::new(),
            intent_id: Id::new(),
            created_at: now(),
            status: PlanStatus::Approved,
            steps: vec![step_a, step_b],
            summary: "test".into(),
        };
        // Nothing completed yet: only step_a (no deps) is ready.
        let ready = plan.ready_steps(&[]);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, a);

        // After a completes (status flips to Succeeded), only step_b is ready:
        // step_a is no longer Pending so it drops out of the ready set.
        let mut plan = plan;
        plan.steps[0].status = StepStatus::Succeeded;
        let ready2 = plan.ready_steps(&[a]);
        assert_eq!(ready2.len(), 1);
        assert_ne!(ready2[0].id, a);
    }

    #[test]
    fn objects_roundtrip_through_json() {
        let i = Intent::received("hello", IntentSource::Voice);
        let json = serde_json::to_string(&i).unwrap();
        let back: Intent = serde_json::from_str(&json).unwrap();
        assert_eq!(i, back);
    }
}
