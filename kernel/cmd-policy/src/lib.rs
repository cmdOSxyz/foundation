//! # cmd-policy — Alios's supervision engine
//!
//! While `cmd-transaction` decides *how to undo* an action, `cmd-policy` decides
//! *whether it may run at all*. This is the decision core of **Alios**, the
//! supervisor: for every action a user agent proposes, it scores risk (R0–R3),
//! checks the action against the agent's granted mandate, checks any spend
//! against the budget, and returns a [`Decision`].
//!
//! The load-bearing rules, enforced here so no agent logic can bypass them:
//! - **R3 (irreversible) is never autonomous** — always needs the human.
//! - An action outside the agent's mandate is **blocked**.
//! - A spend beyond budget is **blocked**.
//! - User agents are **untrusted by default**: absence of authority means no,
//!   not yes.
//!
//! Defined by RFC-0010.

use cmd_types::{Budget, Mandate, RiskClass, Timestamp};

/// A proposed action, as the supervisor sees it. This is the minimal set of
/// facts needed to make a decision; the full `PlanStep` lives elsewhere.
#[derive(Debug, Clone)]
pub struct ProposedAction {
    /// Capability the action belongs to, e.g. `"filesystem"` or `"cmdpay"`.
    pub capability: String,
    /// Risk class of the action.
    pub risk: RiskClass,
    /// Money the action would spend, in the smallest unit (0 if none).
    pub spend: u64,
}

/// The supervisor's verdict on a proposed action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// May run autonomously (within policy). Carries the reason for the record.
    Allow { reason: String },
    /// Must be shown to the human for explicit approval before running.
    NeedsApproval { reason: String },
    /// Refused. The action will not run; the reason is recorded and surfaced.
    Block { reason: String },
}

impl Decision {
    /// Whether this decision permits autonomous execution without a human.
    pub fn is_autonomous(&self) -> bool {
        matches!(self, Decision::Allow { .. })
    }
}

/// The supervision engine. Stateless: every decision is a pure function of the
/// action and the authority presented, evaluated at time `at`.
pub struct PolicyEngine {
    /// Evaluation time, used for mandate/budget expiry checks.
    at: Timestamp,
}

impl PolicyEngine {
    /// A policy engine evaluating as of `at`.
    pub fn new(at: Timestamp) -> Self {
        PolicyEngine { at }
    }

    /// A policy engine evaluating as of now.
    pub fn now() -> Self {
        PolicyEngine {
            at: cmd_types::now(),
        }
    }

    /// Decide the fate of a proposed action given the agent's mandate and budget.
    ///
    /// `mandate` / `budget` are `None` when the agent holds none — which, under
    /// "untrusted by default", means it has no authority for anything beyond
    /// read-only R0 actions.
    pub fn evaluate(
        &self,
        action: &ProposedAction,
        mandate: Option<&Mandate>,
        budget: Option<&Budget>,
    ) -> Decision {
        // Rule 1 — R0 read-only is always safe: it changes nothing, spends nothing.
        if action.risk == RiskClass::R0ReadOnly && action.spend == 0 {
            return Decision::Allow {
                reason: "read-only action".into(),
            };
        }

        // Rule 2 — R3 is NEVER autonomous, regardless of mandate. Human-gated.
        if action.risk == RiskClass::R3Irreversible {
            return Decision::NeedsApproval {
                reason: "irreversible action (R3) requires explicit approval".into(),
            };
        }

        // Rule 3 — anything beyond R0 needs an active mandate covering the
        // capability. No mandate → no authority (untrusted by default).
        let mandate = match mandate {
            Some(m) if m.is_active(self.at) => m,
            Some(_) => {
                return Decision::Block {
                    reason: "mandate is expired or revoked".into(),
                }
            }
            None => {
                return Decision::Block {
                    reason: "no mandate grants this action".into(),
                }
            }
        };
        if !mandate.authorizes_capability(&action.capability) {
            return Decision::Block {
                reason: format!(
                    "mandate does not authorize capability '{}'",
                    action.capability
                ),
            };
        }

        // Rule 4 — the action's risk must not exceed what the mandate allows to
        // run autonomously.
        if action.risk > mandate.max_autonomous_risk {
            return Decision::NeedsApproval {
                reason: "action risk exceeds the mandate's autonomous limit".into(),
            };
        }

        // Rule 5 — any spend must fit the budget (both a money ceiling and an
        // action-count headroom, if set).
        if action.spend > 0 {
            match budget {
                Some(b) if !b.is_expired(self.at) => {
                    if !b.can_spend(action.spend) {
                        return Decision::Block {
                            reason: "spend would exceed the budget ceiling".into(),
                        };
                    }
                    if !b.has_action_headroom() {
                        return Decision::Block {
                            reason: "no action headroom left in the budget".into(),
                        };
                    }
                }
                Some(_) => {
                    return Decision::Block {
                        reason: "budget is expired".into(),
                    }
                }
                None => {
                    return Decision::Block {
                        reason: "spend requires a budget, but none is granted".into(),
                    }
                }
            }
        }

        // Passed every gate: allowed to run autonomously, within policy.
        Decision::Allow {
            reason: "within mandate and budget".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cmd_types::{Id, RiskClass};

    fn action(cap: &str, risk: RiskClass, spend: u64) -> ProposedAction {
        ProposedAction {
            capability: cap.into(),
            risk,
            spend,
        }
    }

    fn mandate(caps: &[&str], max_risk: RiskClass, budget_id: Option<Id>) -> Mandate {
        Mandate {
            id: Id::new(),
            agent_id: Id::new(),
            scope: "test".into(),
            capabilities: caps.iter().map(|c| c.to_string()).collect(),
            max_autonomous_risk: max_risk,
            budget_id,
            granted_at: cmd_types::now(),
            expires_at: None,
            revoked_at: None,
        }
    }

    fn budget(money_ceiling: Option<u64>, spent: u64) -> Budget {
        Budget {
            id: Id::new(),
            agent_id: Id::new(),
            money_ceiling,
            money_spent: spent,
            currency: Some("USD".into()),
            action_ceiling: None,
            actions_taken: 0,
            expires_at: None,
        }
    }

    #[test]
    fn read_only_is_always_allowed_without_any_authority() {
        let p = PolicyEngine::now();
        let d = p.evaluate(&action("filesystem", RiskClass::R0ReadOnly, 0), None, None);
        assert!(d.is_autonomous());
    }

    #[test]
    fn r3_always_needs_approval_even_with_a_mandate() {
        let p = PolicyEngine::now();
        let m = mandate(&["cmdpay"], RiskClass::R3Irreversible, None);
        let d = p.evaluate(
            &action("cmdpay", RiskClass::R3Irreversible, 0),
            Some(&m),
            None,
        );
        // Even though the mandate names R3, R3 is never autonomous.
        assert!(matches!(d, Decision::NeedsApproval { .. }));
        assert!(!d.is_autonomous());
    }

    #[test]
    fn action_beyond_r0_blocked_without_mandate() {
        let p = PolicyEngine::now();
        let d = p.evaluate(
            &action("filesystem", RiskClass::R1Reversible, 0),
            None,
            None,
        );
        assert!(matches!(d, Decision::Block { .. }));
    }

    #[test]
    fn action_blocked_when_mandate_lacks_capability() {
        let p = PolicyEngine::now();
        let m = mandate(&["browser"], RiskClass::R1Reversible, None);
        let d = p.evaluate(
            &action("filesystem", RiskClass::R1Reversible, 0),
            Some(&m),
            None,
        );
        assert!(matches!(d, Decision::Block { .. }));
    }

    #[test]
    fn revoked_mandate_blocks() {
        let p = PolicyEngine::now();
        let mut m = mandate(&["filesystem"], RiskClass::R1Reversible, None);
        m.revoked_at = Some(cmd_types::now());
        let d = p.evaluate(
            &action("filesystem", RiskClass::R1Reversible, 0),
            Some(&m),
            None,
        );
        assert!(matches!(d, Decision::Block { .. }));
    }

    #[test]
    fn risk_above_mandate_limit_needs_approval() {
        let p = PolicyEngine::now();
        // Mandate allows only up to R1 autonomously; action is R2.
        let m = mandate(&["mail"], RiskClass::R1Reversible, None);
        let d = p.evaluate(&action("mail", RiskClass::R2Compensable, 0), Some(&m), None);
        assert!(matches!(d, Decision::NeedsApproval { .. }));
    }

    #[test]
    fn within_mandate_and_no_spend_is_allowed() {
        let p = PolicyEngine::now();
        let m = mandate(&["filesystem"], RiskClass::R1Reversible, None);
        let d = p.evaluate(
            &action("filesystem", RiskClass::R1Reversible, 0),
            Some(&m),
            None,
        );
        assert!(d.is_autonomous());
    }

    #[test]
    fn spend_within_budget_allowed() {
        let p = PolicyEngine::now();
        let m = mandate(&["cmdpay"], RiskClass::R1Reversible, None);
        let b = budget(Some(100), 90);
        let d = p.evaluate(
            &action("cmdpay", RiskClass::R1Reversible, 10),
            Some(&m),
            Some(&b),
        );
        assert!(d.is_autonomous(), "90 + 10 == 100, within ceiling");
    }

    #[test]
    fn spend_over_budget_blocked() {
        let p = PolicyEngine::now();
        let m = mandate(&["cmdpay"], RiskClass::R1Reversible, None);
        let b = budget(Some(100), 95);
        let d = p.evaluate(
            &action("cmdpay", RiskClass::R1Reversible, 10),
            Some(&m),
            Some(&b),
        );
        assert!(matches!(d, Decision::Block { .. }), "95 + 10 > 100");
    }

    #[test]
    fn spend_without_budget_blocked() {
        let p = PolicyEngine::now();
        let m = mandate(&["cmdpay"], RiskClass::R1Reversible, None);
        let d = p.evaluate(
            &action("cmdpay", RiskClass::R1Reversible, 5),
            Some(&m),
            None,
        );
        // A prompt-injected "send money" with no budget cannot be signed.
        assert!(matches!(d, Decision::Block { .. }));
    }
}
