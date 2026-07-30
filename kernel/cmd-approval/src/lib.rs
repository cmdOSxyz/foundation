//! # cmd-approval — an approval that means something
//!
//! Before this crate, an approval was a decision: a person said yes, and the
//! kernel remembered that a yes had happened. Nothing tied the yes to *what* was
//! approved. Three things could go wrong, and none of them were detectable:
//!
//! - **Substitution.** Approve "move 14 files into pdf/", execute something else.
//!   The approval had no idea which plan it belonged to.
//! - **Drift.** Approve a plan after seeing the folder, then the folder changes
//!   before the plan runs. The approval was granted against a world that no
//!   longer exists.
//! - **Replay.** Approve once, execute twice.
//!
//! So an approval here is bound to two digests — the exact plan, and the exact
//! pre-state that was on screen when the person said yes — plus an expiry and a
//! single-use guard. Any of those failing invalidates it. This is the rule the
//! README states as a contract:
//!
//! ```text
//! approval = exact plan hash + exact pre-state
//! state drift = approval invalidation
//! ```
//!
//! # What is digested, and what is deliberately not
//!
//! A plan digest covers the semantics of the work: for each step, the capability,
//! the action, the parameters, and the shape of its dependencies. It does **not**
//! cover step ids, descriptions, or status.
//!
//! That exclusion is load-bearing rather than convenient. Ids are freshly
//! generated every time a planner runs, so including them would mean no approval
//! could ever match the plan it was granted for. Descriptions are prose for the
//! human and carry no authority. Status changes as the plan executes, which is
//! precisely when the digest must stay stable.
//!
//! The consequence is stated plainly: two plans that differ only in wording
//! produce the same digest, and an approval for one is valid for the other. That
//! is correct — they authorize identical work — but it means the description
//! shown to a person must be derived from the parameters rather than supplied
//! alongside them, or the screen could lie about what is being approved. That
//! obligation belongs to the caller, and [`ApprovalRequest::describe`] exists so
//! the caller does not have to invent it.
//!
//! Defined by RFC-0023.

use chrono::Duration;
use cmd_types::{now, ExecutionPlan, Id, PlanStep, RiskClass, Timestamp};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};

/// A 32-byte digest, rendered as hex for logs and receipts.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Digest32([u8; 32]);

impl Digest32 {
    /// Wrap 32 raw bytes.
    ///
    /// Exists so other kernel crates can commit to their own material without
    /// defining a second digest type. A second type would mean two answers to
    /// "what is a plan hash", and they would drift.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Digest32(bytes)
    }

    /// Full hex, for receipts and comparison.
    pub fn hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// The first eight characters, for a screen or a log line.
    pub fn short(&self) -> String {
        self.hex()[..8].to_string()
    }

    pub fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Debug for Digest32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.short())
    }
}

impl std::fmt::Display for Digest32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hex())
    }
}

/// Write one length-prefixed field into a hash.
///
/// The length matters: without it, `capability = "file", action = "system.list"`
/// and `capability = "filesystem", action = "list"` produce identical bytes, and
/// whoever controls the strings gets to choose where the boundary falls.
fn field(h: &mut Sha256, bytes: &[u8]) {
    h.update((bytes.len() as u64).to_be_bytes());
    h.update(bytes);
}

/// Digest one step's authority-bearing content.
///
/// Field lengths are written into the hash alongside the fields themselves, so
/// that `capability = "file", action = "system.list"` cannot collide with
/// `capability = "filesystem", action = "list"`. Without the lengths the
/// concatenation is ambiguous and an attacker chooses the split.
fn hash_step(step: &PlanStep, index_of: &HashMap<Id, usize>) -> Digest32 {
    let mut h = Sha256::new();

    field(&mut h, step.capability.as_bytes());
    field(&mut h, step.action.as_bytes());

    // BTreeMap iterates in key order, so the same parameters always hash the
    // same way regardless of how they were inserted.
    h.update((step.parameters.len() as u64).to_be_bytes());
    for (key, value) in &step.parameters {
        field(&mut h, key.as_bytes());
        // to_string on a serde_json::Value is canonical for the value shapes a
        // plan carries: objects preserve insertion order, but plan parameters are
        // built from a BTreeMap, so ordering is already fixed upstream.
        field(&mut h, value.to_string().as_bytes());
    }

    // Dependencies are hashed as positions, not ids: the ids are regenerated on
    // every planning run, but the *shape* of the graph is what carries meaning.
    let mut positions: Vec<usize> = step
        .depends_on
        .iter()
        .filter_map(|id| index_of.get(id).copied())
        .collect();
    positions.sort_unstable();
    h.update((positions.len() as u64).to_be_bytes());
    for p in positions {
        h.update((p as u64).to_be_bytes());
    }

    h.update([u8::from(step.requires_permission)]);

    let out = h.finalize();
    let mut buf = [0u8; 32];
    buf.copy_from_slice(&out);
    Digest32(buf)
}

/// Digest a whole plan: every step, in order, plus the step count.
///
/// The count is included so that a plan cannot be extended with extra steps
/// whose digests happen to be absorbed — a truncation or extension changes the
/// count and therefore the digest.
pub fn plan_digest(plan: &ExecutionPlan) -> Digest32 {
    let index_of: HashMap<Id, usize> = plan
        .steps
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id, i))
        .collect();

    let mut h = Sha256::new();
    h.update(b"cmdos.plan.v1");
    h.update((plan.steps.len() as u64).to_be_bytes());
    for step in &plan.steps {
        h.update(hash_step(step, &index_of).bytes());
    }
    let out = h.finalize();
    let mut buf = [0u8; 32];
    buf.copy_from_slice(&out);
    Digest32(buf)
}

/// Digest of the world as it was observed before approval.
///
/// What goes in is the caller's decision, because only the caller knows which
/// facts the plan depends on. The contract is that it must cover everything the
/// plan reads or overwrites: a digest that omits a file the plan will delete
/// cannot detect that the file changed.
#[derive(Debug, Clone, Default)]
pub struct StateObservation {
    facts: BTreeMap<String, String>,
}

impl StateObservation {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one observed fact — a path and its checksum, a row count, an
    /// account balance. Recording the same key twice replaces it.
    pub fn observe(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.facts.insert(key.into(), value.into());
        self
    }

    /// Record that something the plan touches does **not** exist. Absence is a
    /// fact: a plan that creates `report.pdf` must fail its approval if someone
    /// else created it in the meantime.
    pub fn observe_absent(self, key: impl Into<String>) -> Self {
        self.observe(key, "\u{0}absent")
    }

    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    pub fn len(&self) -> usize {
        self.facts.len()
    }

    pub fn digest(&self) -> Digest32 {
        let mut h = Sha256::new();
        h.update(b"cmdos.state.v1");
        h.update((self.facts.len() as u64).to_be_bytes());
        for (k, v) in &self.facts {
            field(&mut h, k.as_bytes());
            field(&mut h, v.as_bytes());
        }
        let out = h.finalize();
        let mut buf = [0u8; 32];
        buf.copy_from_slice(&out);
        Digest32(buf)
    }
}

/// What a person is asked to approve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: Id,
    pub plan: Digest32,
    pub pre_state: Digest32,
    /// The highest risk class anywhere in the plan.
    pub ceiling: RiskClass,
    pub requested_at: Timestamp,
    /// How long the person has before the request goes stale, in seconds.
    pub valid_for_secs: u64,
    /// One line per step, derived from the step itself.
    pub summary: Vec<String>,
}

impl ApprovalRequest {
    pub fn new(
        plan: &ExecutionPlan,
        state: &StateObservation,
        ceiling: RiskClass,
        valid_for_secs: u64,
    ) -> Self {
        ApprovalRequest {
            id: Id::new(),
            plan: plan_digest(plan),
            pre_state: state.digest(),
            ceiling,
            requested_at: now(),
            valid_for_secs,
            summary: plan.steps.iter().map(Self::describe).collect(),
        }
    }

    /// Describe a step from the fields that are actually hashed.
    ///
    /// The description a planner writes is not covered by the digest, so showing
    /// it to a person would mean approving one thing and authorizing another.
    /// This builds the line from the capability, action and parameters instead —
    /// the same material the approval is bound to.
    pub fn describe(step: &PlanStep) -> String {
        let args: Vec<String> = step
            .parameters
            .iter()
            .map(|(k, v)| {
                let shown = v
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| v.to_string());
                format!("{k}={shown}")
            })
            .collect();
        if args.is_empty() {
            format!("{}.{}", step.capability, step.action)
        } else {
            format!("{}.{} {}", step.capability, step.action, args.join(" "))
        }
    }
}

/// A granted approval, bound to what it approved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approval {
    pub request_id: Id,
    pub plan: Digest32,
    pub pre_state: Digest32,
    pub granted_at: Timestamp,
    pub expires_at: Timestamp,
    /// Who said yes. Not verified here; carried so the ledger can record it.
    pub approver: String,
}

/// Why an approval does not authorize this execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invalid {
    /// The plan is not the plan that was approved.
    DifferentPlan { approved: String, actual: String },
    /// The world changed between approval and execution.
    StateDrifted { approved: String, actual: String },
    /// The approval is past its expiry.
    Expired {
        expired_at: Timestamp,
        now: Timestamp,
    },
    /// This approval has already been used.
    AlreadyUsed,
    /// The approval belongs to a different request.
    WrongRequest,
    /// Nothing was observed, so drift cannot be detected at all.
    NoStateObserved,
}

impl std::fmt::Display for Invalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Invalid::DifferentPlan { approved, actual } => write!(
                f,
                "this is not the plan that was approved (approved {}, got {})",
                &approved[..8],
                &actual[..8]
            ),
            Invalid::StateDrifted { approved, actual } => write!(
                f,
                "the system changed since approval (was {}, now {}) — approve again",
                &approved[..8],
                &actual[..8]
            ),
            Invalid::Expired { .. } => write!(f, "the approval has expired"),
            Invalid::AlreadyUsed => write!(f, "the approval has already been used"),
            Invalid::WrongRequest => write!(f, "the approval belongs to a different request"),
            Invalid::NoStateObserved => {
                write!(f, "no pre-state was observed, so drift cannot be detected")
            }
        }
    }
}

impl std::error::Error for Invalid {}

/// Grants approvals and decides whether one authorizes an execution.
///
/// Single-use is enforced here rather than by the caller, because "did we already
/// spend this approval" is exactly the kind of bookkeeping a caller forgets under
/// pressure.
#[derive(Debug, Default)]
pub struct ApprovalGate {
    used: Vec<Id>,
}

impl ApprovalGate {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a person's yes against a specific request.
    pub fn grant(&self, request: &ApprovalRequest, approver: impl Into<String>) -> Approval {
        Approval {
            request_id: request.id,
            plan: request.plan,
            pre_state: request.pre_state,
            granted_at: now(),
            expires_at: request.requested_at + Duration::seconds(request.valid_for_secs as i64),
            approver: approver.into(),
        }
    }

    /// Does this approval authorize running `plan` against the world as it is
    /// now? Every check is a separate failure so the caller can tell the person
    /// which one it was — "approve again" and "that is the wrong plan" call for
    /// very different responses.
    pub fn authorize(
        &self,
        approval: &Approval,
        plan: &ExecutionPlan,
        current_state: &StateObservation,
        at: Timestamp,
    ) -> Result<(), Invalid> {
        if self.used.contains(&approval.request_id) {
            return Err(Invalid::AlreadyUsed);
        }
        if at > approval.expires_at {
            return Err(Invalid::Expired {
                expired_at: approval.expires_at,
                now: at,
            });
        }

        let actual_plan = plan_digest(plan);
        if actual_plan != approval.plan {
            return Err(Invalid::DifferentPlan {
                approved: approval.plan.hex(),
                actual: actual_plan.hex(),
            });
        }

        // An empty observation would make every state match, which is worse than
        // no check at all because it looks like one.
        if current_state.is_empty() {
            return Err(Invalid::NoStateObserved);
        }

        let actual_state = current_state.digest();
        if actual_state != approval.pre_state {
            return Err(Invalid::StateDrifted {
                approved: approval.pre_state.hex(),
                actual: actual_state.hex(),
            });
        }

        Ok(())
    }

    /// Spend the approval. Call this once execution begins, so a crash between
    /// authorizing and running cannot leave it reusable.
    pub fn consume(&mut self, approval: &Approval) {
        if !self.used.contains(&approval.request_id) {
            self.used.push(approval.request_id);
        }
    }

    /// Authorize and spend in one step, which is what most callers want.
    pub fn authorize_once(
        &mut self,
        approval: &Approval,
        plan: &ExecutionPlan,
        current_state: &StateObservation,
        at: Timestamp,
    ) -> Result<(), Invalid> {
        self.authorize(approval, plan, current_state, at)?;
        self.consume(approval);
        Ok(())
    }

    pub fn used_count(&self) -> usize {
        self.used.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cmd_types::{PlanStatus, StepStatus};

    fn step(capability: &str, action: &str, params: &[(&str, &str)]) -> PlanStep {
        let mut parameters = BTreeMap::new();
        for (k, v) in params {
            parameters.insert(k.to_string(), serde_json::json!(v));
        }
        PlanStep {
            id: Id::new(),
            description: "whatever the planner wrote".into(),
            capability: capability.into(),
            action: action.into(),
            parameters,
            depends_on: vec![],
            requires_permission: false,
            status: StepStatus::Pending,
            error: None,
        }
    }

    fn plan(steps: Vec<PlanStep>) -> ExecutionPlan {
        ExecutionPlan {
            id: Id::new(),
            intent_id: Id::new(),
            created_at: now(),
            status: PlanStatus::Draft,
            summary: "a summary that carries no authority".into(),
            steps,
        }
    }

    fn folder_with(name: &str, sum: &str) -> StateObservation {
        StateObservation::new().observe(name, sum)
    }

    // ---- the digest -------------------------------------------------------

    #[test]
    fn the_same_work_digests_the_same_despite_fresh_ids() {
        // Ids and descriptions are regenerated on every planning run. If they
        // were hashed, no approval could ever match the plan it was granted for.
        let a = plan(vec![step(
            "filesystem",
            "move",
            &[("from", "a.txt"), ("to", "txt/a.txt")],
        )]);
        let mut b = plan(vec![step(
            "filesystem",
            "move",
            &[("from", "a.txt"), ("to", "txt/a.txt")],
        )]);
        b.steps[0].description = "completely different prose".into();
        b.steps[0].status = StepStatus::Running;

        assert_eq!(plan_digest(&a), plan_digest(&b));
    }

    #[test]
    fn changing_a_parameter_changes_the_digest() {
        let a = plan(vec![step("filesystem", "delete", &[("path", "draft.txt")])]);
        let b = plan(vec![step(
            "filesystem",
            "delete",
            &[("path", "payroll.xlsx")],
        )]);
        assert_ne!(plan_digest(&a), plan_digest(&b));
    }

    #[test]
    fn changing_the_action_changes_the_digest() {
        let a = plan(vec![step("filesystem", "list", &[("path", "/home")])]);
        let b = plan(vec![step("filesystem", "delete", &[("path", "/home")])]);
        assert_ne!(plan_digest(&a), plan_digest(&b));
    }

    #[test]
    fn a_field_boundary_cannot_be_moved() {
        // Without length prefixes these two hash the same, and an attacker gets
        // to choose where one field ends and the next begins.
        let a = plan(vec![step("file", "system.list", &[])]);
        let b = plan(vec![step("filesystem", "list", &[])]);
        assert_ne!(plan_digest(&a), plan_digest(&b));
    }

    #[test]
    fn adding_a_step_changes_the_digest() {
        let one = plan(vec![step("filesystem", "list", &[("path", "/home")])]);
        let two = plan(vec![
            step("filesystem", "list", &[("path", "/home")]),
            step("filesystem", "delete", &[("path", "/home/keep.txt")]),
        ]);
        assert_ne!(plan_digest(&one), plan_digest(&two));
    }

    #[test]
    fn reordering_steps_changes_the_digest() {
        // Order is semantic: deleting then writing is not writing then deleting.
        let s1 = step("filesystem", "write", &[("path", "a")]);
        let s2 = step("filesystem", "delete", &[("path", "a")]);
        let a = plan(vec![s1.clone(), s2.clone()]);
        let b = plan(vec![s2, s1]);
        assert_ne!(plan_digest(&a), plan_digest(&b));
    }

    #[test]
    fn dependency_shape_is_covered() {
        let mut a = plan(vec![
            step("filesystem", "mkdir", &[("path", "out")]),
            step("filesystem", "move", &[("from", "x"), ("to", "out/x")]),
        ]);
        let before = plan_digest(&a);

        // Make the move wait for the mkdir. Same steps, different plan.
        let first = a.steps[0].id;
        a.steps[1].depends_on.push(first);
        assert_ne!(plan_digest(&a), before);
    }

    // ---- substitution -----------------------------------------------------

    #[test]
    fn an_approval_does_not_authorize_a_different_plan() {
        // The attack this crate exists to stop: approve something harmless, then
        // execute something else under the same yes.
        let approved = plan(vec![step("filesystem", "list", &[("path", "/home")])]);
        let swapped = plan(vec![step("filesystem", "delete", &[("path", "/home")])]);
        let state = folder_with("/home", "abc123");

        let request = ApprovalRequest::new(&approved, &state, RiskClass::R3Irreversible, 300);
        let gate = ApprovalGate::new();
        let approval = gate.grant(&request, "admin");

        assert!(gate.authorize(&approval, &approved, &state, now()).is_ok());

        let err = gate
            .authorize(&approval, &swapped, &state, now())
            .unwrap_err();
        assert!(matches!(err, Invalid::DifferentPlan { .. }), "got {err:?}");
    }

    // ---- drift ------------------------------------------------------------

    #[test]
    fn state_drift_invalidates_the_approval() {
        let p = plan(vec![step(
            "filesystem",
            "move",
            &[("from", "a.txt"), ("to", "txt/a.txt")],
        )]);
        let seen = folder_with("a.txt", "sum-at-approval");
        let request = ApprovalRequest::new(&p, &seen, RiskClass::R2Compensable, 300);
        let gate = ApprovalGate::new();
        let approval = gate.grant(&request, "admin");

        // Someone edited the file between the yes and the run.
        let changed = folder_with("a.txt", "sum-after-someone-edited-it");
        let err = gate.authorize(&approval, &p, &changed, now()).unwrap_err();
        assert!(matches!(err, Invalid::StateDrifted { .. }), "got {err:?}");
    }

    #[test]
    fn something_appearing_is_also_drift() {
        // A plan that creates a file must not run if the file now exists.
        let p = plan(vec![step("filesystem", "write", &[("path", "report.pdf")])]);
        let seen = StateObservation::new().observe_absent("report.pdf");
        let request = ApprovalRequest::new(&p, &seen, RiskClass::R1Reversible, 300);
        let gate = ApprovalGate::new();
        let approval = gate.grant(&request, "admin");

        let now_exists = folder_with("report.pdf", "someone-else-wrote-this");
        assert!(matches!(
            gate.authorize(&approval, &p, &now_exists, now())
                .unwrap_err(),
            Invalid::StateDrifted { .. }
        ));
    }

    #[test]
    fn an_empty_observation_is_refused_rather_than_passing() {
        // An empty state matches nothing and everything. Treating it as a match
        // would be worse than having no check, because it looks like one.
        let p = plan(vec![step("filesystem", "list", &[("path", "/x")])]);
        let seen = folder_with("/x", "sum");
        let request = ApprovalRequest::new(&p, &seen, RiskClass::R0ReadOnly, 300);
        let gate = ApprovalGate::new();
        let approval = gate.grant(&request, "admin");

        let nothing = StateObservation::new();
        assert_eq!(
            gate.authorize(&approval, &p, &nothing, now()).unwrap_err(),
            Invalid::NoStateObserved
        );
    }

    // ---- expiry and replay ------------------------------------------------

    #[test]
    fn an_approval_expires() {
        let p = plan(vec![step("filesystem", "list", &[("path", "/x")])]);
        let state = folder_with("/x", "sum");
        let request = ApprovalRequest::new(&p, &state, RiskClass::R0ReadOnly, 60);
        let gate = ApprovalGate::new();
        let approval = gate.grant(&request, "admin");

        let later = request.requested_at + Duration::seconds(61);
        assert!(matches!(
            gate.authorize(&approval, &p, &state, later).unwrap_err(),
            Invalid::Expired { .. }
        ));
    }

    #[test]
    fn an_approval_is_single_use() {
        let p = plan(vec![step("filesystem", "delete", &[("path", "once.txt")])]);
        let state = folder_with("once.txt", "sum");
        let request = ApprovalRequest::new(&p, &state, RiskClass::R3Irreversible, 300);
        let mut gate = ApprovalGate::new();
        let approval = gate.grant(&request, "admin");

        gate.authorize_once(&approval, &p, &state, now()).unwrap();
        assert_eq!(
            gate.authorize_once(&approval, &p, &state, now())
                .unwrap_err(),
            Invalid::AlreadyUsed
        );
        assert_eq!(gate.used_count(), 1);
    }

    #[test]
    fn checks_are_ordered_so_the_person_gets_the_useful_message() {
        // A spent approval reports as spent even if the plan also changed —
        // "you already used this" is the actionable half.
        let approved = plan(vec![step("filesystem", "list", &[("path", "/x")])]);
        let other = plan(vec![step("filesystem", "delete", &[("path", "/x")])]);
        let state = folder_with("/x", "sum");
        let request = ApprovalRequest::new(&approved, &state, RiskClass::R0ReadOnly, 300);
        let mut gate = ApprovalGate::new();
        let approval = gate.grant(&request, "admin");
        gate.consume(&approval);

        assert_eq!(
            gate.authorize(&approval, &other, &state, now())
                .unwrap_err(),
            Invalid::AlreadyUsed
        );
    }

    // ---- what the person is shown -----------------------------------------

    #[test]
    fn the_summary_comes_from_what_is_hashed_not_from_the_prose() {
        // If the screen showed the planner's description, a plan could say
        // "tidying up" while authorizing a delete. The summary is built from the
        // same fields the digest covers.
        let mut s = step("filesystem", "delete", &[("path", "/home/payroll.xlsx")]);
        s.description = "tidy up some old files".into();
        let p = plan(vec![s]);
        let state = folder_with("/home/payroll.xlsx", "sum");
        let request = ApprovalRequest::new(&p, &state, RiskClass::R3Irreversible, 300);

        let line = &request.summary[0];
        assert!(line.contains("filesystem.delete"), "got {line}");
        assert!(line.contains("payroll.xlsx"), "got {line}");
        assert!(
            !line.contains("tidy up"),
            "the prose must not be what is shown"
        );
    }

    #[test]
    fn a_request_carries_the_ceiling_and_both_digests() {
        let p = plan(vec![step(
            "terminal",
            "run",
            &[("command", "rm -rf build")],
        )]);
        let state = folder_with("build/", "sum");
        let request = ApprovalRequest::new(&p, &state, RiskClass::R3Irreversible, 300);

        assert_eq!(request.ceiling, RiskClass::R3Irreversible);
        assert_eq!(request.plan, plan_digest(&p));
        assert_eq!(request.pre_state, state.digest());
        assert_eq!(request.plan.hex().len(), 64);
        assert_eq!(request.plan.short().len(), 8);
    }

    #[test]
    fn observing_the_same_key_twice_replaces_it() {
        let a = StateObservation::new()
            .observe("f", "old")
            .observe("f", "new");
        let b = StateObservation::new().observe("f", "new");
        assert_eq!(a.digest(), b.digest());
        assert_eq!(a.len(), 1);
    }

    #[test]
    fn observation_order_does_not_matter() {
        let a = StateObservation::new().observe("a", "1").observe("b", "2");
        let b = StateObservation::new().observe("b", "2").observe("a", "1");
        assert_eq!(a.digest(), b.digest());
    }

    #[test]
    fn the_happy_path_works() {
        let p = plan(vec![
            step("filesystem", "mkdir", &[("path", "txt")]),
            step(
                "filesystem",
                "move",
                &[("from", "a.txt"), ("to", "txt/a.txt")],
            ),
        ]);
        let state = StateObservation::new()
            .observe("a.txt", "sum-a")
            .observe_absent("txt");

        let request = ApprovalRequest::new(&p, &state, RiskClass::R1Reversible, 300);
        assert_eq!(request.summary.len(), 2);

        let mut gate = ApprovalGate::new();
        let approval = gate.grant(&request, "admin");
        assert_eq!(approval.request_id, request.id);

        gate.authorize_once(&approval, &p, &state, now()).unwrap();
    }
}
