//! # cmd-proof — evidence, as distinct from a record
//!
//! A ledger entry says an execution happened. That is a record, and it is written
//! by the same system that performed the action: anyone who believes it already
//! believes the runtime. It settles nothing with a party who does not.
//!
//! A [`ProofBundle`] commits to each stage of one execution — intent, plan,
//! pre-state, approval, action, post-state, verdict — as a chain of hashes. The
//! chain is what makes the sequence itself part of the claim. A bundle whose
//! approval follows its action describes a very different system from one where it
//! precedes it, and after tampering the two are indistinguishable unless the order
//! is hashed.
//!
//! # Two different questions
//!
//! Checking a bundle and checking reality are separate operations, deliberately:
//!
//! - [`ProofBundle::verify_chain`] needs nothing but the bundle. It detects
//!   tampering, reordering, truncation, and insertion.
//! - [`ProofBundle::check_plan`] and [`ProofBundle::check_post_state`] need the
//!   original material, and answer whether the bundle is about *this* plan and
//!   *this* world.
//!
//! That split is the privacy property. A bundle can be published while the files,
//! balances, and parameters behind it stay local. Someone holding the originals can
//! prove correspondence; someone holding only the bundle can still detect tampering.
//!
//! # What v0 does not prove
//!
//! A bundle is tamper-**evident**, not authenticated. There is no signature, so
//! anyone can construct a well-formed bundle for an execution that never happened.
//! What it establishes is internal consistency and correspondence to material the
//! checker already holds — never that cmdOS produced it. No caller may present a v0
//! bundle as proof of origin. Signing is v1.
//!
//! Coverage is the caller's obligation, exactly as in RFC-0023: a post-state
//! observation that omits a file the plan deleted cannot detect that it is gone.
//! The digest is only ever as honest as the observation behind it.
//!
//! Defined by RFC-0025.

use cmd_approval::{Approval, Digest32, StateObservation};
use cmd_types::{ExecutionPlan, RiskClass};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Domain separator. Prefixed into every link so a digest computed here can never
/// be mistaken for one computed by another part of the system.
const DOMAIN: &[u8] = b"cmdos.proof.v1";

/// The "previous link" of the first stage.
const GENESIS: [u8; 32] = [0u8; 32];

/// Write one length-prefixed field into a hash.
///
/// The length is what stops the boundary from being attacker-chosen: without it
/// `stage = "Pre", digest = "State.."` and `stage = "PreState", digest = ".."`
/// produce identical bytes. Same reasoning as `field()` in cmd-approval.
fn field(h: &mut Sha256, bytes: &[u8]) {
    h.update((bytes.len() as u64).to_be_bytes());
    h.update(bytes);
}

fn finish(h: Sha256) -> Digest32 {
    let out = h.finalize();
    let mut buf = [0u8; 32];
    buf.copy_from_slice(&out);
    Digest32::from_bytes(buf)
}

/// The seven stages of one execution, in causal order.
///
/// The set is closed rather than open. A free-form stage list would let a bundle
/// describe a sequence that never occurs in the kernel, and a verifier would have
/// no way to notice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    Intent,
    Plan,
    PreState,
    Approval,
    Action,
    PostState,
    Verdict,
}

impl Stage {
    /// Canonical order. Building from this is what makes an out-of-order bundle
    /// unconstructible through the builder.
    pub const ORDER: [Stage; 7] = [
        Stage::Intent,
        Stage::Plan,
        Stage::PreState,
        Stage::Approval,
        Stage::Action,
        Stage::PostState,
        Stage::Verdict,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Stage::Intent => "intent",
            Stage::Plan => "plan",
            Stage::PreState => "pre_state",
            Stage::Approval => "approval",
            Stage::Action => "action",
            Stage::PostState => "post_state",
            Stage::Verdict => "verdict",
        }
    }
}

/// What a verifier concluded.
///
/// [`Outcome::Unverified`] is a distinct value rather than an absent verdict
/// because "no verifier ran" and "a verifier ran and was satisfied" must never
/// collapse into each other. Collapsing them is precisely how a system comes to
/// report work it never checked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Verified,
    Failed { reason: String },
    Unverified,
}

impl Outcome {
    fn digest(&self) -> Digest32 {
        let mut h = Sha256::new();
        field(&mut h, b"cmdos.proof.verdict.v1");
        match self {
            Outcome::Verified => field(&mut h, b"verified"),
            Outcome::Unverified => field(&mut h, b"unverified"),
            Outcome::Failed { reason } => {
                field(&mut h, b"failed");
                field(&mut h, reason.as_bytes());
            }
        }
        finish(h)
    }
}

/// One stage's commitment. `None` means the stage did not occur.
///
/// Absence is recorded rather than omitted, and is covered by the chain, so a
/// stage cannot be made to appear or disappear after sealing. An R0 read genuinely
/// has no approval; forcing callers to invent one would put fiction in the
/// evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Commitment {
    pub stage: Stage,
    pub digest: Option<Digest32>,
}

/// Why a bundle does not hold up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Broken {
    /// The stage list is not the seven canonical stages in order.
    MalformedStages,
    /// A link does not follow from the one before it.
    ChainBroken {
        stage: Stage,
        expected: String,
        actual: String,
    },
    /// The seal does not match the chain it claims to close.
    SealMismatch { expected: String, actual: String },
    /// The bundle records nothing for a stage being checked.
    StageAbsent(Stage),
    /// The material does not match what the bundle committed to.
    Mismatch {
        stage: Stage,
        committed: String,
        actual: String,
    },
}

impl std::fmt::Display for Broken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Broken::MalformedStages => {
                write!(f, "the bundle does not carry the seven stages in order")
            }
            Broken::ChainBroken { stage, .. } => write!(
                f,
                "the chain breaks at the {} stage — the bundle was altered after sealing",
                stage.label()
            ),
            Broken::SealMismatch { .. } => {
                write!(f, "the seal does not match the bundle's contents")
            }
            Broken::StageAbsent(stage) => {
                write!(f, "the bundle records no {} stage", stage.label())
            }
            Broken::Mismatch { stage, .. } => write!(
                f,
                "the {} does not match what the bundle committed to",
                stage.label()
            ),
        }
    }
}

impl std::error::Error for Broken {}

/// Compute one link from the previous one.
///
/// The presence byte is what makes an absent stage unforgeable: without it,
/// dropping a digest would produce the same bytes as a stage that never had one.
fn link(prev: &[u8; 32], stage: Stage, digest: Option<&Digest32>) -> Digest32 {
    let mut h = Sha256::new();
    field(&mut h, DOMAIN);
    field(&mut h, prev);
    field(&mut h, stage.label().as_bytes());
    match digest {
        Some(d) => {
            h.update([1u8]);
            field(&mut h, d.bytes());
        }
        None => h.update([0u8]),
    }
    finish(h)
}

fn seal_of(links: &[Digest32]) -> Digest32 {
    let mut h = Sha256::new();
    field(&mut h, b"cmdos.proof.seal.v1");
    // The count is sealed so truncation cannot pass as a shorter honest chain.
    h.update((links.len() as u64).to_be_bytes());
    if let Some(last) = links.last() {
        field(&mut h, last.bytes());
    }
    finish(h)
}

/// Tamper-evident evidence for one execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofBundle {
    commitments: Vec<Commitment>,
    /// One link per stage; each covers the previous link.
    links: Vec<Digest32>,
    /// Closes the chain and the stage count.
    seal: Digest32,
    /// The highest risk class in the execution. Recorded, never enforced —
    /// enforcement lives in cmd-policy, and a second gate here could disagree
    /// with the first.
    pub ceiling: RiskClass,
}

impl ProofBundle {
    /// The bundle's identity: one digest covering the whole chain.
    pub fn seal(&self) -> Digest32 {
        self.seal
    }

    pub fn commitments(&self) -> &[Commitment] {
        &self.commitments
    }

    /// What the bundle committed to for a stage, if anything.
    pub fn digest_for(&self, stage: Stage) -> Option<Digest32> {
        self.commitments
            .iter()
            .find(|c| c.stage == stage)
            .and_then(|c| c.digest)
    }

    /// Recompute every link and the seal. Needs no original material.
    ///
    /// Detects tampering, reordering, truncation, and insertion — each of which
    /// changes a link, and every link after it.
    pub fn verify_chain(&self) -> Result<(), Broken> {
        if self.commitments.len() != Stage::ORDER.len()
            || self.links.len() != self.commitments.len()
        {
            return Err(Broken::MalformedStages);
        }
        for (c, expected) in self.commitments.iter().zip(Stage::ORDER.iter()) {
            if c.stage != *expected {
                return Err(Broken::MalformedStages);
            }
        }

        let mut prev = GENESIS;
        for (i, c) in self.commitments.iter().enumerate() {
            let expected = link(&prev, c.stage, c.digest.as_ref());
            if expected != self.links[i] {
                return Err(Broken::ChainBroken {
                    stage: c.stage,
                    expected: expected.hex(),
                    actual: self.links[i].hex(),
                });
            }
            prev = *expected.bytes();
        }

        let expected_seal = seal_of(&self.links);
        if expected_seal != self.seal {
            return Err(Broken::SealMismatch {
                expected: expected_seal.hex(),
                actual: self.seal.hex(),
            });
        }
        Ok(())
    }

    /// Is this the plan the bundle is about?
    pub fn check_plan(&self, plan: &ExecutionPlan) -> Result<(), Broken> {
        self.check(Stage::Plan, cmd_approval::plan_digest(plan))
    }

    /// Was the world what the bundle says it was before the plan ran?
    pub fn check_pre_state(&self, observed: &StateObservation) -> Result<(), Broken> {
        self.check(Stage::PreState, observed.digest())
    }

    /// Did the world end up the way the bundle claims?
    pub fn check_post_state(&self, observed: &StateObservation) -> Result<(), Broken> {
        self.check(Stage::PostState, observed.digest())
    }

    fn check(&self, stage: Stage, actual: Digest32) -> Result<(), Broken> {
        match self.digest_for(stage) {
            None => Err(Broken::StageAbsent(stage)),
            Some(committed) if committed == actual => Ok(()),
            Some(committed) => Err(Broken::Mismatch {
                stage,
                committed: committed.hex(),
                actual: actual.hex(),
            }),
        }
    }
}

/// Builds a bundle by filling fixed slots.
///
/// Order is structural rather than a rule the caller must remember: the slots are
/// emitted in [`Stage::ORDER`], so an out-of-order bundle cannot be produced
/// through this type at all. Any slot left unset is sealed as absent.
#[derive(Debug)]
pub struct BundleBuilder {
    intent: Option<Digest32>,
    plan: Option<Digest32>,
    pre_state: Option<Digest32>,
    approval: Option<Digest32>,
    action: Option<Digest32>,
    post_state: Option<Digest32>,
    verdict: Option<Digest32>,
    ceiling: RiskClass,
}

impl BundleBuilder {
    /// Start a bundle for an execution whose highest risk class is `ceiling`.
    ///
    /// The ceiling is required rather than defaulted: there is no safe default
    /// here, and a builder that quietly assumed R0 would understate every bundle
    /// whose author forgot to set it.
    pub fn new(ceiling: RiskClass) -> Self {
        BundleBuilder {
            intent: None,
            plan: None,
            pre_state: None,
            approval: None,
            action: None,
            post_state: None,
            verdict: None,
            ceiling,
        }
    }

    /// The request as the user expressed it.
    pub fn intent(mut self, text: &str) -> Self {
        let mut h = Sha256::new();
        field(&mut h, b"cmdos.proof.intent.v1");
        field(&mut h, text.as_bytes());
        self.intent = Some(finish(h));
        self
    }

    pub fn plan(mut self, plan: &ExecutionPlan) -> Self {
        self.plan = Some(cmd_approval::plan_digest(plan));
        self
    }

    pub fn pre_state(mut self, observed: &StateObservation) -> Self {
        self.pre_state = Some(observed.digest());
        self
    }

    /// Commit to the approval that authorized this execution, including who
    /// granted it and what it was bound to.
    pub fn approval(mut self, approval: &Approval) -> Self {
        let mut h = Sha256::new();
        field(&mut h, b"cmdos.proof.approval.v1");
        field(&mut h, approval.request_id.to_string().as_bytes());
        field(&mut h, approval.plan.bytes());
        field(&mut h, approval.pre_state.bytes());
        field(&mut h, approval.granted_at.to_rfc3339().as_bytes());
        field(&mut h, approval.expires_at.to_rfc3339().as_bytes());
        field(&mut h, approval.approver.as_bytes());
        self.approval = Some(finish(h));
        self
    }

    /// Commit to what actually ran, one line per executed step.
    ///
    /// Kept separate from the plan digest: a plan is what was authorized, an
    /// action is what the capability reports having done, and a system that
    /// cannot tell them apart cannot detect a capability that did something else.
    pub fn action(mut self, executed: &[String]) -> Self {
        let mut h = Sha256::new();
        field(&mut h, b"cmdos.proof.action.v1");
        h.update((executed.len() as u64).to_be_bytes());
        for line in executed {
            field(&mut h, line.as_bytes());
        }
        self.action = Some(finish(h));
        self
    }

    pub fn post_state(mut self, observed: &StateObservation) -> Self {
        self.post_state = Some(observed.digest());
        self
    }

    pub fn verdict(mut self, outcome: &Outcome) -> Self {
        self.verdict = Some(outcome.digest());
        self
    }

    /// Seal the bundle. Slots left unset are recorded as absent, not skipped.
    pub fn seal(self) -> ProofBundle {
        let digests = [
            self.intent,
            self.plan,
            self.pre_state,
            self.approval,
            self.action,
            self.post_state,
            self.verdict,
        ];
        let commitments: Vec<Commitment> = Stage::ORDER
            .iter()
            .zip(digests.iter())
            .map(|(stage, digest)| Commitment {
                stage: *stage,
                digest: *digest,
            })
            .collect();

        let mut links = Vec::with_capacity(commitments.len());
        let mut prev = GENESIS;
        for c in &commitments {
            let l = link(&prev, c.stage, c.digest.as_ref());
            prev = *l.bytes();
            links.push(l);
        }
        let seal = seal_of(&links);

        ProofBundle {
            commitments,
            links,
            seal,
            ceiling: self.ceiling,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cmd_types::{now, Id, PlanStatus, PlanStep, StepStatus};
    use std::collections::BTreeMap;

    fn step(capability: &str, action: &str, arg: &str) -> PlanStep {
        let mut parameters = BTreeMap::new();
        parameters.insert("path".to_string(), serde_json::json!(arg));
        PlanStep {
            id: Id::new(),
            capability: capability.to_string(),
            action: action.to_string(),
            parameters,
            depends_on: Vec::new(),
            description: String::new(),
            requires_permission: true,
            status: StepStatus::Pending,
            error: None,
        }
    }

    fn plan_of(steps: Vec<PlanStep>) -> ExecutionPlan {
        ExecutionPlan {
            id: Id::new(),
            intent_id: Id::new(),
            steps,
            created_at: now(),
            status: PlanStatus::Draft,
            summary: String::new(),
        }
    }

    fn a_plan() -> ExecutionPlan {
        plan_of(vec![step("files", "move", "report.pdf")])
    }

    fn a_state() -> StateObservation {
        StateObservation::new().observe("report.pdf", "sha256:abc")
    }

    fn post_state() -> StateObservation {
        StateObservation::new().observe("pdf/report.pdf", "sha256:abc")
    }

    fn an_approval(plan: &ExecutionPlan, pre: &StateObservation) -> Approval {
        let request = cmd_approval::ApprovalRequest::new(plan, pre, RiskClass::R2Compensable, 300);
        cmd_approval::ApprovalGate::new().grant(&request, "the user")
    }

    /// A bundle with all seven stages present. "Full" has to mean full: an
    /// earlier version of this helper skipped the approval, which silently turned
    /// the absent-stage test into a no-op.
    fn full_bundle() -> ProofBundle {
        let plan = a_plan();
        let pre = a_state();
        let approval = an_approval(&plan, &pre);
        BundleBuilder::new(RiskClass::R2Compensable)
            .intent("move the report into pdf/")
            .plan(&plan)
            .pre_state(&pre)
            .approval(&approval)
            .action(&["files.move path=report.pdf".to_string()])
            .post_state(&post_state())
            .verdict(&Outcome::Verified)
            .seal()
    }

    #[test]
    fn the_full_bundle_really_is_full() {
        // Guards the helper itself: every later test that mutates a stage depends
        // on that stage actually being there.
        let b = full_bundle();
        for stage in Stage::ORDER {
            assert!(
                b.digest_for(stage).is_some(),
                "{} is absent from the full bundle",
                stage.label()
            );
        }
    }

    #[test]
    fn a_well_formed_bundle_verifies() {
        assert!(full_bundle().verify_chain().is_ok());
    }

    #[test]
    fn mutating_any_stage_breaks_the_chain() {
        // Every position matters, not just the ones near the end.
        for i in 0..Stage::ORDER.len() {
            let mut b = full_bundle();
            b.commitments[i].digest = Some(Digest32::from_bytes([9u8; 32]));
            assert!(
                b.verify_chain().is_err(),
                "tampering at position {i} went undetected"
            );
        }
    }

    #[test]
    fn reordering_two_stages_breaks_the_chain() {
        let mut b = full_bundle();
        b.commitments.swap(1, 2);
        assert_eq!(b.verify_chain(), Err(Broken::MalformedStages));
    }

    #[test]
    fn truncating_the_chain_is_detected() {
        let mut b = full_bundle();
        b.commitments.pop();
        b.links.pop();
        assert_eq!(b.verify_chain(), Err(Broken::MalformedStages));
    }

    #[test]
    fn a_recomputed_seal_does_not_rescue_a_truncated_chain() {
        // The strongest version of the attack: drop the verdict, then rebuild the
        // links and the seal so everything is internally consistent. The fixed
        // stage count is what refuses it.
        let mut b = full_bundle();
        b.commitments.pop();
        let mut links = Vec::new();
        let mut prev = GENESIS;
        for c in &b.commitments {
            let l = link(&prev, c.stage, c.digest.as_ref());
            prev = *l.bytes();
            links.push(l);
        }
        b.seal = seal_of(&links);
        b.links = links;
        assert_eq!(b.verify_chain(), Err(Broken::MalformedStages));
    }

    #[test]
    fn flipping_a_present_stage_to_absent_breaks_the_chain() {
        let mut b = full_bundle();
        b.commitments[3].digest = None; // approval
        assert!(b.verify_chain().is_err());
    }

    #[test]
    fn forging_an_absent_stage_after_sealing_breaks_the_chain() {
        // An R0 bundle has no approval. Inventing one after the fact must fail.
        let r0 = BundleBuilder::new(RiskClass::R0ReadOnly)
            .intent("list the folder")
            .plan(&a_plan())
            .action(&["files.list path=.".to_string()])
            .verdict(&Outcome::Verified)
            .seal();
        let mut forged = r0.clone();
        forged.commitments[3].digest = Some(Digest32::from_bytes([7u8; 32]));
        assert!(r0.verify_chain().is_ok());
        assert!(forged.verify_chain().is_err());
    }

    #[test]
    fn an_r0_bundle_with_no_approval_still_verifies() {
        let b = BundleBuilder::new(RiskClass::R0ReadOnly)
            .intent("list the folder")
            .plan(&a_plan())
            .verdict(&Outcome::Verified)
            .seal();
        assert!(b.verify_chain().is_ok());
        assert_eq!(b.digest_for(Stage::Approval), None);
    }

    #[test]
    fn check_plan_accepts_the_real_plan_and_rejects_another() {
        let plan = a_plan();
        let b = BundleBuilder::new(RiskClass::R2Compensable)
            .plan(&plan)
            .seal();
        assert!(b.check_plan(&plan).is_ok());

        let other = plan_of(vec![step("files", "delete", "report.pdf")]);
        assert!(matches!(
            b.check_plan(&other),
            Err(Broken::Mismatch {
                stage: Stage::Plan,
                ..
            })
        ));
    }

    #[test]
    fn check_post_state_rejects_a_drifted_world() {
        let b = full_bundle();
        assert!(b.check_post_state(&post_state()).is_ok());

        let drifted = StateObservation::new().observe("pdf/report.pdf", "sha256:CHANGED");
        assert!(matches!(
            b.check_post_state(&drifted),
            Err(Broken::Mismatch {
                stage: Stage::PostState,
                ..
            })
        ));
    }

    #[test]
    fn checking_an_absent_stage_says_so_rather_than_passing() {
        let b = BundleBuilder::new(RiskClass::R0ReadOnly)
            .plan(&a_plan())
            .seal();
        assert_eq!(
            b.check_post_state(&post_state()),
            Err(Broken::StageAbsent(Stage::PostState))
        );
    }

    #[test]
    fn unverified_and_verified_are_different_commitments() {
        // The whole point of keeping Unverified as a value: it must not be able to
        // masquerade as a successful check.
        assert_ne!(
            Outcome::Verified.digest(),
            Outcome::Unverified.digest(),
            "an unverified outcome must not digest as a verified one"
        );
        assert_ne!(
            Outcome::Verified.digest(),
            Outcome::Failed {
                reason: "postcondition not met".into()
            }
            .digest()
        );
    }

    #[test]
    fn a_failure_reason_is_part_of_the_commitment() {
        let a = Outcome::Failed {
            reason: "file missing".into(),
        };
        let b = Outcome::Failed {
            reason: "permission denied".into(),
        };
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn bundles_for_different_plans_have_different_seals() {
        let one = BundleBuilder::new(RiskClass::R2Compensable)
            .plan(&a_plan())
            .seal();
        let two = BundleBuilder::new(RiskClass::R2Compensable)
            .plan(&plan_of(vec![step("files", "delete", "report.pdf")]))
            .seal();
        assert_ne!(one.seal(), two.seal());
    }

    #[test]
    fn the_seal_survives_a_serialization_round_trip() {
        let b = full_bundle();
        let json = serde_json::to_string(&b).expect("bundle serializes");
        let back: ProofBundle = serde_json::from_str(&json).expect("bundle deserializes");
        assert_eq!(back.seal(), b.seal());
        assert!(back.verify_chain().is_ok());
        assert_eq!(back, b);
    }

    #[test]
    fn stage_labels_cannot_collide_through_concatenation() {
        // Length-prefixing is what makes this hold; without it the pair
        // ("pre", "state..") and ("pre_state", "..") could hash alike.
        let d = Digest32::from_bytes([1u8; 32]);
        assert_ne!(
            link(&GENESIS, Stage::PreState, Some(&d)),
            link(&GENESIS, Stage::PostState, Some(&d))
        );
    }
}
