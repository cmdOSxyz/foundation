//! Capabilities: the core execution primitive.
//!
//! An agent invokes a Capability action; only the runtime executes it, under
//! permission control. This file defines the CONTRACT shape only — concrete
//! implementations live under `capabilities/` and are exposed as MCP servers.
//!
//! Mirrors `schemas/capability.ts`, with `RiskLevel` replaced by the canonical
//! [`RiskClass`](crate::common::RiskClass).

use crate::common::{Id, RiskClass};
use serde::{Deserialize, Serialize};

/// A single action a capability can perform. This is the contract the planner
/// reads to know what exists and what parameters it takes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityAction {
    /// Action name, unique within the capability, e.g. `"rename"`.
    pub name: String,
    pub description: String,
    /// Names of parameters this action expects.
    pub parameters: Vec<String>,
    /// Risk class of the action; drives the permission requirement.
    pub risk: RiskClass,
}

/// The contract for a capability. Groups related actions under one name,
/// e.g. `"filesystem"` with actions `list` / `read` / `rename` / `move`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub id: Id,
    /// Capability name used by plan steps, e.g. `"filesystem"`.
    pub name: String,
    /// Semantic version of this capability's contract, e.g. `"1.0.0"`.
    pub version: String,
    pub description: String,
    pub actions: Vec<CapabilityAction>,
}

impl Capability {
    /// Look up an action by name.
    pub fn action(&self, name: &str) -> Option<&CapabilityAction> {
        self.actions.iter().find(|a| a.name == name)
    }
}
