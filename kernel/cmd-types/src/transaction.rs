//! The Transaction: the v2 primitive that makes execution reversible.
//!
//! Every side-effecting action runs as a transaction:
//! simulate/shadow -> snapshot -> execute -> verify -> commit | rollback.
//!
//! This type is the object-model record of that lifecycle. The engine that
//! drives it lives in `cmd-transaction`; the behavior it must satisfy is
//! specified by `prototype/tests/filesystem.behavior.test.ts`.

use crate::common::{Id, Timestamp};
use serde::{Deserialize, Serialize};

/// The phase a transaction is in. Ordered to match the pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionPhase {
    /// Ran in a shadow world; no real effect yet.
    Simulated,
    /// Pre-state of affected resources captured.
    Snapshotted,
    /// Applied to real state.
    Executed,
    /// Post-state checked against the intended outcome.
    Verified,
    /// Made permanent (still undoable via the snapshot within the retention window).
    Committed,
    /// Reverted using the snapshot; real state restored.
    RolledBack,
}

/// A record of one reversible unit of execution, tied to a plan step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Id,
    /// The plan this transaction belongs to.
    pub plan_id: Id,
    /// The specific step this transaction executes.
    pub step_id: Id,
    pub phase: TransactionPhase,
    /// Opaque handle to the captured pre-state (e.g. a content-addressed snapshot
    /// id), if one was taken. `None` for read-only (R0) actions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_ref: Option<String>,
    /// Whether an undo path is registered and currently valid.
    pub reversible: bool,
    pub created_at: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Transaction {
    /// Whether this transaction can still be rolled back: it has a snapshot,
    /// is marked reversible, and hasn't already been rolled back.
    pub fn can_roll_back(&self) -> bool {
        self.reversible
            && self.snapshot_ref.is_some()
            && self.phase != TransactionPhase::RolledBack
    }
}
