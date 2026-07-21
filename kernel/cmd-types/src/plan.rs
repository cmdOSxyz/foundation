//! The ExecutionPlan: an ordered graph of steps the runtime executes.
//!
//! The AI proposes it; the kernel executes it. The AI never executes directly.
//! Mirrors `schemas/execution-plan.ts`.

use crate::common::{Id, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Lifecycle status of a whole plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Draft,
    AwaitingPermission,
    Approved,
    Executing,
    Completed,
    Failed,
    RolledBack,
}

/// Lifecycle status of a single step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Skipped,
}

/// One unit of work in a plan. A step invokes exactly one capability action and
/// never mutates state directly — the runtime executes it under permission
/// control. Mirrors the `PlanStep` interface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: Id,
    pub description: String,
    /// Capability name, e.g. `"filesystem"`.
    pub capability: String,
    /// Action name within that capability, e.g. `"rename"`.
    pub action: String,
    #[serde(default)]
    pub parameters: BTreeMap<String, serde_json::Value>,
    /// Ids of steps that must complete before this one (the DAG edges).
    #[serde(default)]
    pub depends_on: Vec<Id>,
    pub requires_permission: bool,
    pub status: StepStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// The full plan derived from an Intent: an ordered set of steps plus metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub id: Id,
    /// The intent this plan fulfills.
    pub intent_id: Id,
    pub created_at: Timestamp,
    pub status: PlanStatus,
    pub steps: Vec<PlanStep>,
    pub summary: String,
}

impl ExecutionPlan {
    /// Steps whose dependencies are all satisfied by `completed`, and which are
    /// still pending. This is the set the scheduler may start next.
    ///
    /// Kept deliberately simple here (the object model); the real scheduling
    /// policy lives in `cmd-kernel`.
    pub fn ready_steps<'a>(&'a self, completed: &[Id]) -> Vec<&'a PlanStep> {
        self.steps
            .iter()
            .filter(|s| s.status == StepStatus::Pending)
            .filter(|s| s.depends_on.iter().all(|d| completed.contains(d)))
            .collect()
    }
}
