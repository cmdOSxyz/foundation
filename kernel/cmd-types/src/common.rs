//! Shared primitives used across the whole object model.
//!
//! Mirrors the `Id` and `Timestamp` aliases from the TypeScript schemas
//! (`schemas/intent.ts`) but gives them real types in Rust.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A stable unique identifier for any object in cmdOS.
///
/// Wraps a UUID v4. Newtype rather than a bare `Uuid` so the compiler stops
/// us from mixing, say, an intent id with an agent id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Id(pub Uuid);

impl Id {
    /// Generate a fresh random id.
    pub fn new() -> Self {
        Id(Uuid::new_v4())
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An ISO-8601 / RFC-3339 timestamp in UTC.
pub type Timestamp = chrono::DateTime<chrono::Utc>;

/// Convenience: the current UTC time.
pub fn now() -> Timestamp {
    chrono::Utc::now()
}

/// Reversibility / impact class of an action. This is the single most important
/// safety signal in cmdOS: it drives whether Alios lets an action run
/// autonomously, asks for approval, or blocks it.
///
/// Supersedes the string `RiskLevel` union from `schemas/capability.ts`:
/// - `read_only`  -> `R0`
/// - `reversible` -> `R1`
/// - `destructive`-> `R2`
/// - `external`   -> `R3`
///
/// The Rn naming is canonical from Strategy v2; the old names map onto it 1:1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskClass {
    /// R0 — observes state, changes nothing. Always safe to run.
    R0ReadOnly,
    /// R1 — changes state but is fully reversible (snapshot + undo).
    R1Reversible,
    /// R2 — changes state with side effects; compensable, not free to undo.
    R2Compensable,
    /// R3 — irreversible / affects the outside world. ALWAYS human-gated.
    R3Irreversible,
}

impl RiskClass {
    /// Whether an action of this class may ever run without explicit human
    /// approval. R3 can never be autonomous, by rule.
    pub fn may_be_autonomous(self) -> bool {
        !matches!(self, RiskClass::R3Irreversible)
    }

    /// Whether cmdOS can reverse an action of this class after the fact.
    pub fn is_reversible(self) -> bool {
        matches!(self, RiskClass::R0ReadOnly | RiskClass::R1Reversible)
    }
}
