//! The reversible-execution engine.
//!
//! For each step the engine runs the pipeline and produces a [`Transaction`]
//! record (from `cmd-types`). It writes an [`Event`] to the ledger at each
//! meaningful phase, so the audit trail is a byproduct of execution, not an
//! afterthought.
//!
//! Policy on failure: **verify-fail auto-rolls-back**. If a step executes but
//! fails verification, the engine restores the snapshot and reports the failure,
//! leaving real state as it was before the step. Read-only (R0) steps take no
//! snapshot and cannot roll back (there is nothing to undo).

use crate::resource::{Resource, ResourceError};
use cmd_ledger::Ledger;
use cmd_types::{Event, EventType, Id, PlanStep, RiskClass, Transaction, TransactionPhase};

/// Outcome of running one step through the engine.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    /// The transaction record, in its final phase.
    pub transaction: Transaction,
    /// Simulation summary produced before execution.
    pub simulation: String,
}

/// The engine. Borrows a ledger to record what happens.
pub struct TransactionEngine<'a> {
    ledger: &'a mut Ledger,
}

impl<'a> TransactionEngine<'a> {
    /// Build an engine that records into `ledger`.
    pub fn new(ledger: &'a mut Ledger) -> Self {
        TransactionEngine { ledger }
    }

    /// Run one step against a resource through the full reversible pipeline.
    ///
    /// - Read-only (`requires_permission == false` AND `risk == R0`) steps skip
    ///   snapshotting.
    /// - On verify failure the snapshot is restored and an error is returned.
    pub fn run<R: Resource>(
        &mut self,
        resource: &mut R,
        step: &PlanStep,
        risk: RiskClass,
    ) -> Result<Outcome, EngineError> {
        let plan_id = Id::new(); // in the full runtime this comes from the plan
        let mut txn = Transaction {
            id: Id::new(),
            plan_id,
            step_id: step.id,
            phase: TransactionPhase::Simulated,
            snapshot_ref: None,
            reversible: risk.is_reversible(),
            created_at: cmd_types::now(),
            error: None,
        };

        // 1. Simulate (shadow; no real effect).
        let simulation = resource.simulate(step).map_err(EngineError::Resource)?;
        self.record(EventType::PlanCreated, step, plan_id);

        // 2. Snapshot (skipped for read-only R0 steps).
        let snapshot = if risk == RiskClass::R0ReadOnly {
            None
        } else {
            let snap = resource.snapshot(step).map_err(EngineError::Resource)?;
            if snap.is_some() {
                txn.snapshot_ref = Some(format!("snap:{}", txn.id));
                txn.phase = TransactionPhase::Snapshotted;
            }
            snap
        };

        // 3. Execute.
        self.record(EventType::StepStarted, step, plan_id);
        if let Err(e) = resource.execute(step) {
            txn.error = Some(e.to_string());
            self.record(EventType::StepFailed, step, plan_id);
            return Err(EngineError::Resource(e));
        }
        txn.phase = TransactionPhase::Executed;

        // 4. Verify.
        let ok = resource.verify(step).map_err(EngineError::Resource)?;
        if ok {
            txn.phase = TransactionPhase::Verified;
            self.record(EventType::StepSucceeded, step, plan_id);
            // 5a. Commit.
            txn.phase = TransactionPhase::Committed;
            self.record(EventType::TransactionCommitted, step, plan_id);
            Ok(Outcome {
                transaction: txn,
                simulation,
            })
        } else {
            // 5b. Verify failed → auto rollback if we can.
            if let Some(snap) = snapshot {
                resource.restore(snap).map_err(EngineError::Resource)?;
                txn.phase = TransactionPhase::RolledBack;
                self.record(EventType::TransactionRolledBack, step, plan_id);
            }
            txn.error = Some("verification failed".to_string());
            self.record(EventType::StepFailed, step, plan_id);
            Err(EngineError::VerificationFailed(txn))
        }
    }

    /// Explicitly roll back a committed, reversible transaction (user undo).
    /// The caller supplies the same resource and the snapshot it kept.
    pub fn rollback<R: Resource>(
        &mut self,
        resource: &mut R,
        mut txn: Transaction,
        snapshot: R::Snap,
        step: &PlanStep,
    ) -> Result<Transaction, EngineError> {
        if !txn.can_roll_back() {
            return Err(EngineError::NotReversible);
        }
        resource.restore(snapshot).map_err(EngineError::Resource)?;
        txn.phase = TransactionPhase::RolledBack;
        self.record(EventType::TransactionRolledBack, step, txn.plan_id);
        Ok(txn)
    }

    /// Write one event to the ledger, tagged with the step and plan.
    fn record(&mut self, kind: EventType, step: &PlanStep, plan_id: Id) {
        let mut ev = Event::new(kind);
        ev.plan_id = Some(plan_id);
        ev.detail.insert(
            "step".to_string(),
            serde_json::json!({ "capability": step.capability, "action": step.action }),
        );
        self.ledger.append(ev);
    }
}

/// Errors the engine can return.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineError {
    /// A resource lifecycle call failed.
    Resource(ResourceError),
    /// The step executed but failed verification; state was restored if possible.
    /// Carries the rolled-back transaction record.
    VerificationFailed(Transaction),
    /// Rollback was requested on a transaction that cannot be reversed.
    NotReversible,
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::Resource(e) => write!(f, "{e}"),
            EngineError::VerificationFailed(_) => write!(f, "verification failed; rolled back"),
            EngineError::NotReversible => write!(f, "transaction is not reversible"),
        }
    }
}

impl std::error::Error for EngineError {}
