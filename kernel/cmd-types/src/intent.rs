//! The Intent: the first artifact in the canonical loop.
//!
//! Intent -> Understanding -> Planning -> Permission -> Execution -> Verification -> Result
//!
//! Mirrors `schemas/intent.ts`. An Intent is the structured, validated
//! representation of what the user wants.

use crate::common::{Id, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Where an intent came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentSource {
    UserCommand,
    Voice,
    Scheduled,
    /// Raised by an agent (e.g. a sub-task in a delegation).
    Agent,
}

/// Lifecycle status of an intent as it moves through the loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentStatus {
    /// Captured from the user, not yet understood.
    Received,
    /// Parsed and validated into a clear objective.
    Understood,
    /// An execution plan has been produced.
    Planned,
    /// Could not be understood or is not permitted.
    Rejected,
}

/// The structured objective cmdOS understood from the raw request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Objective {
    /// A short, normalized summary of what to achieve.
    pub summary: String,
    /// Optional structured parameters extracted from the request.
    /// `BTreeMap` (not `HashMap`) so serialization is deterministic — important
    /// because objectives get hashed into the audit ledger.
    #[serde(default)]
    pub parameters: BTreeMap<String, serde_json::Value>,
}

/// A single expressed goal from the user, plus cmdOS's understanding of it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intent {
    pub id: Id,
    /// The raw natural-language request, exactly as written.
    pub raw_text: String,
    pub source: IntentSource,
    pub created_at: Timestamp,
    pub status: IntentStatus,
    /// The structured objective; `None` until at least `Understood`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective: Option<Objective>,
    /// Populated only when `Rejected`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejection_reason: Option<String>,
}

impl Intent {
    /// Create a freshly received intent from raw user text.
    pub fn received(raw_text: impl Into<String>, source: IntentSource) -> Self {
        Intent {
            id: Id::new(),
            raw_text: raw_text.into(),
            source,
            created_at: crate::common::now(),
            status: IntentStatus::Received,
            objective: None,
            rejection_reason: None,
        }
    }
}
