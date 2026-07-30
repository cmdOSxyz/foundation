# RFC-0025: Proof Bundle v0 — Evidence a Third Party Can Check

Version: 1.0
Status: Accepted
Category: Kernel
Author: Lead Architect
Depends on: RFC-0004 (object model), RFC-0007 (cmd-ledger), RFC-0023 (approval binding)
Implemented by: `kernel/cmd-proof`

---

# 1. Summary

An execution currently leaves behind a ledger entry saying it happened. That is a record,
not evidence: it is written by the same system that performed the action, and anyone who
believes the record already believes the system.

A Proof Bundle is the artifact that closes the gap. It commits to each stage of one
execution — intent, plan, pre-state, approval, action, post-state, verdict — as a chain of
hashes, so a party who trusts nothing about the runtime can still check three things:

1. the stages are internally consistent and nothing was inserted, removed, or reordered
2. the plan that ran is the plan that was approved
3. the post-state is the one the verifier actually observed

# 2. Motivation

RFC-0023 bound an approval to an exact plan and an exact pre-state. That binding is checked
at execution time and then discarded. Nothing carries it forward, so after the fact there
is no way to demonstrate that the check ever happened.

This matters most in the case cmdOS exists to handle: an agent reports success. Today the
report and the evidence for the report have the same author. A Proof Bundle separates them
— the digests are computed from observed material, and the verdict is recorded next to the
observation that justifies it rather than in place of it.

# 3. Design

## 3.1 Stages

A bundle is an ordered sequence of stages, each a `(label, digest)` pair. v0 fixes seven:

```text
Intent → Plan → PreState → Approval → Action → PostState → Verdict
```

The order is the causal order of RFC-0023 plus execution. It is fixed rather than
free-form because the sequence itself is a claim: a bundle whose approval follows its
action describes a different and much worse system than one where it precedes it.

Stages may be **absent**. An R0 read has no approval and nothing to undo, so requiring an
approval stage would push callers to fabricate one. An absent stage is recorded as absent
and is covered by the chain, so absence cannot be added or removed after the fact.

## 3.2 The chain

Each link covers the previous link's digest, the stage label, and the stage digest:

```text
link[0] = H("cmdos.proof.v1" || GENESIS   || label[0] || digest[0])
link[i] = H("cmdos.proof.v1" || link[i-1] || label[i] || digest[i])
```

Every field is length-prefixed before hashing, following `field()` in cmd-approval: without
lengths, `label = "Pre", digest = "State…"` and `label = "PreState", digest = "…"` hash
identically and whoever controls the strings chooses the split.

The bundle's identity is the final link. Truncation is detected because the count is hashed
into the seal alongside it.

## 3.3 What a verifier needs

Verification of the chain requires only the bundle. Verification that the bundle describes
*reality* requires the original material, and the two are deliberately different
operations:

- `verify_chain()` — recomputes every link. Detects tampering, reordering, truncation, and
  insertion. Needs no secrets and no original data.
- `check_plan(plan)` — recomputes `plan_digest` from a plan and compares. Answers "is this
  the plan the bundle is about?"
- `check_post_state(observation)` — recomputes a `StateObservation` digest and compares.
  Answers "did the world end up the way the bundle claims?"

This split is the privacy property. A bundle can be published while the underlying files,
balances, and parameters stay local; a holder of the original material can prove
correspondence, and a holder of only the bundle can still detect tampering.

## 3.4 Verdict

The verdict stage commits to an outcome enum — `Verified`, `Failed`, `Unverified` — and,
when the outcome is `Failed`, to the reason. `Unverified` exists because "no verifier ran"
must be distinguishable from "a verifier ran and was satisfied". Collapsing them is how a
system ends up reporting hallucinated completion.

# 4. Interfaces

```rust
pub struct Commitment { pub stage: Stage, pub digest: Option<Digest32> }
pub struct ProofBundle { /* commitments + links + seal + ceiling */ }

pub struct BundleBuilder;          // one slot per stage; emitted in Stage::ORDER
impl ProofBundle {
    pub fn seal(&self) -> Digest32;
    pub fn verify_chain(&self) -> Result<(), Broken>;
    pub fn check_plan(&self, plan: &ExecutionPlan) -> Result<(), Broken>;
    pub fn check_pre_state(&self, observed: &StateObservation) -> Result<(), Broken>;
    pub fn check_post_state(&self, observed: &StateObservation) -> Result<(), Broken>;
}
```

Order is structural rather than enforced by a typestate chain: the builder fills named slots
and emits them in `Stage::ORDER`, so an out-of-order bundle cannot be constructed at all.
A typestate chain would enforce the same invariant while also forcing callers to supply
stages they do not have, which is how absent stages turn into invented ones.

`BundleBuilder::new` requires the risk ceiling instead of defaulting it. There is no safe
default: a builder that quietly assumed R0 would understate every bundle whose author
forgot to set it.

`Digest32` and `StateObservation` are reused from cmd-approval rather than redefined. A
second digest type would be a second definition of what a plan hash means, and the two
would drift.

# 5. Security model

Untrusted: the agent, the planner, the model, the capability, and any party transmitting a
bundle.

Properties that hold:

- a modified stage breaks every link after it, and therefore the seal
- a reordered or removed stage changes the chain
- an appended stage changes the seal, and the stage count is sealed
- a bundle for plan A cannot be presented as evidence for plan B

Properties that explicitly **do not** hold in v0, and must not be claimed:

- **No signature.** A bundle is tamper-*evident*, not authenticated. Anyone can construct a
  well-formed bundle for a fabricated execution. What a bundle proves is internal
  consistency and correspondence to material the checker already holds — not that cmdOS
  produced it. Signing is v1, and until then no interface may describe a bundle as proof of
  origin.
- **No freshness.** A bundle carries timestamps but nothing prevents replaying an old one.
- **Coverage is the caller's.** A pre-state observation that omits a file the plan deletes
  cannot detect that the file changed. The digest is only as honest as the observation, and
  that obligation sits with the caller, exactly as in RFC-0023.

# 6. Reversibility impact (R0-R3)

None. Building or verifying a bundle performs no side effect and is R0 throughout. The
bundle records a risk ceiling for the execution it describes but does not enforce it —
enforcement stays in cmd-policy, and duplicating it here would create a second gate that
can disagree with the first.

# 7. Testing

- chain verifies on a well-formed bundle
- a mutated digest at every position breaks verification
- reordering two stages breaks verification
- truncating the final stage breaks verification
- appending a stage changes the seal
- absent stages are covered: flipping present to absent breaks the chain
- `check_plan` accepts the real plan and rejects a modified one
- `check_post_state` accepts the observed state and rejects a drifted one
- a bundle built for one plan fails `check_plan` for another
- `Unverified` and `Verified` produce different digests
- an R0 bundle with no approval stage still verifies
- the seal is stable across serialization round-trips

# 8. Next

v1 adds a signature over the seal and a freshness nonce, at which point a bundle proves
origin as well as consistency. Independent verifier support and the open verifier CLI
(Phase 5) consume this format.
