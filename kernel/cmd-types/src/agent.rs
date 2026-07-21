//! Agents and permission requests.
//!
//! The two-tier model from RFC-0006: a **User Agent** (owned by the user, does
//! the work, untrusted by default) and **Alios** (the cmdOS-owned supervisor).
//! `PermissionRequest` mirrors `schemas/permission-request.ts`.

use crate::common::{Id, RiskClass, Timestamp};
use serde::{Deserialize, Serialize};

/// Which tier an agent belongs to. These are never collapsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentKind {
    /// Created and owned by the user; does the work; untrusted by default.
    User,
    /// The cmdOS-owned supervisor. Exactly one per Machine. Cannot be disabled
    /// by a user agent.
    Supervisor,
}

/// Alios's running trust assessment of a user agent. Drives how tightly the
/// supervisor gates that agent's actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    /// Behaving within its role; normal gating.
    Calm,
    /// Some anomaly signals; tighter gating.
    Cautious,
    /// Suspended pending human review.
    Quarantined,
}

/// An agent: a User Agent or the Alios supervisor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Agent {
    pub id: Id,
    pub kind: AgentKind,
    /// Display name. For user agents this is user-chosen (e.g. "Nova").
    pub name: String,
    /// Avatar asset reference (e.g. "nova"). Optional for the supervisor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    /// Free-text role/personality. Shapes tone ONLY — never authority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Supervisor's current trust assessment. `Calm` for a fresh agent;
    /// not meaningful for the supervisor itself.
    pub trust: TrustLevel,
    pub created_at: Timestamp,
}

impl Agent {
    /// Create a new user-owned agent with default (calm) trust.
    pub fn new_user(name: impl Into<String>) -> Self {
        Agent {
            id: Id::new(),
            kind: AgentKind::User,
            name: name.into(),
            avatar: None,
            role: None,
            trust: TrustLevel::Calm,
            created_at: crate::common::now(),
        }
    }

    /// Whether this agent is the supervisor (and therefore governs others).
    pub fn is_supervisor(&self) -> bool {
        matches!(self.kind, AgentKind::Supervisor)
    }
}

/// The user's decision on a permission request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionDecision {
    Pending,
    Approved,
    Denied,
    Expired,
}

/// A request for the user to authorize one sensitive step before it runs.
/// Raised by Alios; R3 actions ALWAYS produce one of these. Mirrors
/// `schemas/permission-request.ts`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub id: Id,
    pub plan_id: Id,
    pub step_id: Id,
    /// The agent whose action is being gated.
    pub agent_id: Id,
    /// The capability being invoked, e.g. `"filesystem"`.
    pub capability: String,
    /// The action being invoked, e.g. `"delete"`.
    pub action: String,
    /// Risk class that triggered the request.
    pub risk: RiskClass,
    /// Human-readable explanation of exactly what will happen.
    pub explanation: String,
    pub decision: PermissionDecision,
    pub requested_at: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decided_at: Option<Timestamp>,
}
