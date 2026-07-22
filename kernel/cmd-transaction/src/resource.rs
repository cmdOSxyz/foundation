//! The [`Resource`] trait: the abstraction the engine drives.
//!
//! The transaction engine knows nothing about files, APIs, or VMs. It only
//! knows that a resource can be snapshotted, executed against, verified, and
//! restored. This is what lets the same engine later shadow a filesystem
//! (`capabilities/files`) or an entire Machine (RFC-0006) without changing.

use cmd_types::PlanStep;

/// A captured pre-state of a resource, opaque to the engine. Concrete resources
/// define what a snapshot actually contains.
pub trait Snapshot: Send + Sync {}

/// Anything that can take part in a reversible transaction.
///
/// The lifecycle the engine calls, in order:
/// `simulate` (optional preview) → `snapshot` → `execute` → `verify` →
/// then either commit (do nothing more) or `restore(snapshot)` to roll back.
pub trait Resource {
    /// The snapshot type this resource produces.
    type Snap: Snapshot;

    /// Dry-run the step against a shadow of the resource and describe the effect,
    /// without touching real state. Returns a human-readable summary.
    fn simulate(&self, step: &PlanStep) -> Result<String, ResourceError>;

    /// Capture the current pre-state so the step can be undone. Read-only (R0)
    /// steps may return `None` — nothing to restore.
    fn snapshot(&self, step: &PlanStep) -> Result<Option<Self::Snap>, ResourceError>;

    /// Apply the step to real state.
    fn execute(&mut self, step: &PlanStep) -> Result<(), ResourceError>;

    /// Check that real state now matches the step's intended outcome.
    fn verify(&self, step: &PlanStep) -> Result<bool, ResourceError>;

    /// Restore the resource from a snapshot (the undo path).
    fn restore(&mut self, snapshot: Self::Snap) -> Result<(), ResourceError>;
}

/// Errors a resource can raise during any lifecycle phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    /// The step could not be performed; carries a reason.
    Failed(String),
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceError::Failed(m) => write!(f, "resource error: {m}"),
        }
    }
}

impl std::error::Error for ResourceError {}
