# RFC-0024: cmdCapital — Attested Trading Agents

Version: 0.1
Status: Draft
Category: Product / Ecosystem
Author: Lead Architect
Depends on: RFC-0010 (cmd-policy), RFC-0008 (cmd-transaction), RFC-0023 (Approval Binding)
Specification: `docs/01-vision/cmdcapital-spec.md`
Implemented by: nothing yet — this RFC blocks all cmdCapital code

---

# 1. Summary

cmdCapital is an onchain verification layer and marketplace for autonomous trading
agents. Its one idea is that a track record must be **derived, not reported**: the agent
runs in a TEE, every order carries a remote attestation proof, and an indexer computes
performance from venue fills rather than from anything the agent publishes.

The custody half is the same idea applied to funds. An ERC-4337 session key lets a
delegated agent trade and forbids it from withdrawing, so a fully compromised agent still
cannot move principal out.

This RFC does not accept that design. It states what cmdCapital maps onto in the existing
kernel, and it names the decisions that must be made before any component is written.

# 2. Motivation

Two failures define the market this enters:

- **Fake track record.** A screenshot of a PnL curve costs nothing to produce and nothing
  to fake.
- **Counterparty risk.** Following a strategy usually means handing someone the ability to
  move your funds.

Both are already cmdOS problems under different names. A track record is a claim about a
past action, which is what cmdProof exists to verify. A session key with a spending bound
is a capability with a budget, which is what cmdFirewall exists to enforce. cmdCapital is
those two primitives pointed at trading venues, not a new safety model.

# 3. Mapping onto existing primitives

| cmdCapital layer | cmdOS primitive | Status |
| --- | --- | --- |
| NexusKernel (TEE execution, attestation) | cmdProof evidence, signed remote results | Proof Bundle v0 not defined |
| Metric Engine (venue indexer, derived metrics) | cmdProof post-state observation | no venue adapters exist |
| NexusShield (session key, withdrawal denied) | cmd-policy capability + budget, cmdMandate | cmdMandate is unimplemented |
| NexusEscrow (performance fee split) | cmdSettle, cmdPay mandates | payments are built last |
| Circuit breaker | cmd-policy budget exhaustion + revoke | policy engine exists (RFC-0010) |

The dependency order is not negotiable: attested execution and derived metrics come before
any marketplace surface, and settlement comes last. Building the marketplace first would
produce exactly the unverified leaderboard the project exists to replace.

# 4. Risk classification

- Reading market data, quotes, and leaderboard state → **R0**.
- Producing an unsigned strategy plan or a proposed allocation → **R1**.
- A trade inside an accepted mandate, under budget, on an allowlisted venue → **R2**.
- Granting a session key, raising a limit, adding a venue, bridging, and every settlement
  or fee transfer → **R3**, always human-gated.

Bridging deserves emphasis. A cross-chain transfer is irreversible and the failure mode is
total loss of the bridged amount, so it cannot inherit R2 from the trade that motivated it.

# 5. Blocking decisions

These are unresolved in the specification. Each one changes the design, so none can be
deferred to implementation.

## 5.1 Who operates the TEE, and what does attestation actually cover?

Attestation proves that specific code ran in a genuine enclave. It does not prove that the
operator listed every agent it ran. If cmdCapital hosts the enclaves and also curates the
marketplace, it can silently drop losing agents and the surviving leaderboard is still
fully attested — survivorship bias that the hardware cannot detect.

**Decision needed:** either agent registration is committed onchain *before* the first
trade, so the set of agents is fixed in advance and dropouts are visible, or the operator
role is separated from the curator role. Attestation without an enrolment commitment does
not deliver Proof of PnL.

## 5.2 Who controls the relayer?

Executing across thousands of copy-trading wallets in milliseconds means the relayer sees
trade intent before it lands. That is a front-running position over every follower, and it
is a more valuable one than any individual strategy.

**Decision needed:** the trust model for the relayer, and whether intent is encrypted to it
or merely trusted to it. Until this is answered, the "avoid slippage" benefit and a
privileged extraction channel are the same mechanism.

## 5.3 What does the circuit breaker measure?

A -15% threshold is meaningless without a source. Mark price or realised PnL? Which oracle,
and what staleness bound? For a thin-liquidity position, forced liquidation at the
threshold can itself cause the loss it was meant to cap.

**Decision needed:** the metric, the oracle and its freshness requirement, and the
unwind policy — immediate market exit, or close-only with a time limit. cmd-policy already
enforces budgets; this is a question about the input, not the enforcement.

## 5.4 Is there a high-water mark?

The specification charges 20% on realised profit but never mentions a high-water mark.
Without one, an agent that loses 30% and then gains 30% collects a performance fee on the
recovery while the user is still down.

**Decision needed:** per-user high-water mark accounting, its reset conditions, and its
behaviour across deposits and withdrawals mid-cycle. This is a correctness property of
NexusEscrow, not a pricing preference.

## 5.5 What is the governance token?

The buy-back and burn has no token behind it. No name, no supply, no issuance mechanism, no
statement of what the token confers.

**Decision needed:** whether a token exists at all. If it does, it needs its own RFC —
the treasury flow cannot be specified against an undefined asset. If it does not, remove
buy-back and burn from the specification rather than leaving it as an implied commitment.

# 6. Security model

The threat model must treat as untrusted: the agent, the model behind it, the relayer, the
venue, the bridge, the oracle, and the indexer. The properties that must hold under all of
them being hostile:

- no private key leaves the enclave
- no delegated key can withdraw principal
- no order is accepted without a valid, fresh attestation
- no published metric originates from the agent
- revoke and close-only take effect without the agent's cooperation

The last property is the one most easily lost in implementation. If the emergency path runs
through the same relayer as normal execution, a compromised relayer can suppress it.

# 7. Testing strategy

No cmdCapital code merges without:

- attestation verification tests including expired, replayed, and forged quotes
- session-key tests proving withdrawal is rejected at the wallet, not merely unused
- circuit-breaker tests under oracle staleness, gaps, and manipulated marks
- high-water-mark accounting tests across loss, recovery, deposit, and mid-cycle exit
- adversarial simulations against every actor in section 6 assumed hostile
- a red-team suite that finds no path from a compromised agent to principal

Mainnet exposure requires an independent security review, consistent with the Phase 3
completion target.

# 8. What must not be built yet

- the marketplace leaderboard, until attestation and metric derivation exist
- NexusEscrow, under rule 6: capability follows safety, and payments are built last
- any surface displaying a performance number, real or illustrative

Locked states must name their missing dependency. An empty screen that does not say what is
missing reads as a broken product and, worse, invites placeholder data.

# 9. Next

Resolve section 5, then split this RFC into implementable units: attestation verification,
venue indexing and metric derivation, and session-key delegation. Each gets its own RFC
number and its own acceptance.
