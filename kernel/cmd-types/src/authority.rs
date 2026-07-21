//! Authority primitives introduced in Strategy v2: **Budget** and **Mandate**.
//!
//! These have no equivalent in the original TypeScript schemas. They exist so
//! that authority sits *below* the agent, enforced by the kernel, where prompt
//! injection cannot reach it. An agent holds only what it was granted.

use crate::common::{Id, RiskClass, Timestamp};
use serde::{Deserialize, Serialize};

/// A hard, kernel-enforced spending/usage limit on an agent. Exhaustion pauses
/// the agent and forces it to request renewal — it can never overspend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Budget {
    pub id: Id,
    /// The agent this budget constrains.
    pub agent_id: Id,
    /// Money ceiling in the smallest currency unit (e.g. cents), if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub money_ceiling: Option<u64>,
    /// Money already spent, same unit.
    #[serde(default)]
    pub money_spent: u64,
    /// Currency code, e.g. `"USD"`. Only meaningful with a money ceiling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Maximum number of actions this budget authorizes, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_ceiling: Option<u64>,
    /// Actions already taken against this budget.
    #[serde(default)]
    pub actions_taken: u64,
    /// When this budget stops being valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<Timestamp>,
}

impl Budget {
    /// Whether spending `amount` more would stay within the money ceiling.
    /// `None` ceiling means "no money authority" -> always false for spending.
    pub fn can_spend(&self, amount: u64) -> bool {
        match self.money_ceiling {
            Some(ceiling) => self.money_spent.saturating_add(amount) <= ceiling,
            None => false,
        }
    }

    /// Whether at least one more action is authorized.
    pub fn has_action_headroom(&self) -> bool {
        match self.action_ceiling {
            Some(ceiling) => self.actions_taken < ceiling,
            None => true, // no action cap set
        }
    }

    /// Whether the budget is expired as of `at`.
    pub fn is_expired(&self, at: Timestamp) -> bool {
        matches!(self.expires_at, Some(exp) if at >= exp)
    }
}

/// A cryptographically-signable grant of authority from the user to an agent to
/// perform a *class* of actions within limits. AP2-compatible in spirit: it is
/// the proof that an agent was permitted to act (especially to spend).
///
/// The signature itself is added by `cmd-policy`/`cmdpay`; this type is the
/// signable content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mandate {
    pub id: Id,
    /// The agent granted this mandate.
    pub agent_id: Id,
    /// Human-readable scope, e.g. `"research subscriptions"`.
    pub scope: String,
    /// The capability names this mandate authorizes, e.g. `["cmdpay", "browser"]`.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// The highest risk class this mandate authorizes to run autonomously.
    /// A mandate can never authorize autonomous R3 (enforced by policy), but the
    /// field is kept explicit for auditing.
    pub max_autonomous_risk: RiskClass,
    /// Optional budget bound to this mandate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget_id: Option<Id>,
    pub granted_at: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<Timestamp>,
    /// Set when the user revokes the mandate; revoked mandates authorize nothing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<Timestamp>,
}

impl Mandate {
    /// Whether this mandate is currently active (granted, not expired, not revoked).
    pub fn is_active(&self, at: Timestamp) -> bool {
        if self.revoked_at.is_some() {
            return false;
        }
        !matches!(self.expires_at, Some(exp) if at >= exp)
    }

    /// Whether this mandate authorizes the named capability.
    pub fn authorizes_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }
}
