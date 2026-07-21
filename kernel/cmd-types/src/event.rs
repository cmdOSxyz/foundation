//! Events: the append-only record of everything that happens.
//!
//! Every action, score, and intervention becomes an Event in the signed audit
//! ledger (implemented in `cmd-ledger`; the prototype reference is
//! `prototype/kernel/event-log.ts`). Mirrors `schemas/event.ts`.

use crate::common::{Id, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The kind of thing that happened. Kept as an open, explicit enum so the
/// ledger is self-describing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    IntentReceived,
    IntentUnderstood,
    PlanCreated,
    PermissionRequested,
    PermissionDecided,
    StepStarted,
    StepSucceeded,
    StepFailed,
    TransactionCommitted,
    TransactionRolledBack,
    /// Alios scored an action R0–R3.
    ActionScored,
    /// Alios intervened (warn/pause/block/quarantine).
    SupervisorIntervened,
}

/// One immutable entry in the audit ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: Id,
    #[serde(rename = "type")]
    pub event_type: EventType,
    pub at: Timestamp,
    /// The plan this event relates to, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<Id>,
    /// The agent this event relates to, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<Id>,
    /// Free-form structured detail. Deterministic ordering for stable hashing.
    #[serde(default)]
    pub detail: BTreeMap<String, serde_json::Value>,
}

impl Event {
    /// Create an event of the given type, stamped now.
    pub fn new(event_type: EventType) -> Self {
        Event {
            id: Id::new(),
            event_type,
            at: crate::common::now(),
            plan_id: None,
            agent_id: None,
            detail: BTreeMap::new(),
        }
    }
}
