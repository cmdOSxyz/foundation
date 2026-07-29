<div align="center">

<img src="./assets/hero.svg" width="100%" alt="cmdOS — Safe execution for AI agents" />

<br/>

# cmdOS

### An AI agent built to act in the real world

**cmdOS simulates consequences, enforces limits, coordinates approval, executes authorized actions, and proves the final outcome.**

[![Status](https://img.shields.io/badge/status-active%20development-22c55e?style=for-the-badge&labelColor=07110b)](#project-status)
[![Architecture](https://img.shields.io/badge/architecture-local--first-22c55e?style=for-the-badge&labelColor=07110b)](#local-first-trust-boundary)
[![Safety](https://img.shields.io/badge/safety-transaction%20firewall-22c55e?style=for-the-badge&labelColor=07110b)](#the-product)
[![Evidence](https://img.shields.io/badge/results-verifiable-22c55e?style=for-the-badge&labelColor=07110b)](#cmdproof)

<br/>

[Product](#the-product) · [What Runs Today](#what-runs-today) · [Architecture](#architecture) · [Roadmap](#roadmap) · [Security](#security-model)

</div>

---

## The Product

cmdOS is a safety-first AI agent and an independent transaction firewall for autonomous actions.

It sits between AI agents and systems that can change real state:

```text
Codex / Claude / Copilot / LangGraph / Other Agents
                         │
                    MCP / A2A / API
                         │
                       cmdOS
                         │
        Files / Shell / Browser / SaaS / Wallets / Markets
```

An agent may propose an action. cmdOS decides whether that action is authorized, previews its consequences, executes it through a constrained capability, checks the real post-state, and records evidence.

```text
Intent
→ Effect Graph
→ Permission & Risk Check
→ Shadow Simulation
→ State Diff
→ Exact Approval
→ Commit
→ Observe Real State
→ Verify or Compensate
→ Proof Bundle
```

> **Agents plan. cmdOS makes consequences safe to commit.**

---

## Why This Exists

Agent frameworks already provide planning, tool use, handoffs, retries, memory, computer use, and human-in-the-loop workflows. Wallets and exchanges already provide payments, swaps, prediction markets, perpetuals, and transaction simulation.

The unsolved problem is what happens when a probabilistic agent is allowed to change real state across several independent systems.

A workflow checkpoint cannot recall an email, reverse a finalized blockchain transaction, or guarantee that a remote API reached the intended state. A permission prompt alone cannot detect several agents splitting one restricted action into smaller actions.

cmdOS focuses on this boundary:

- complete mediation of external side effects;
- cumulative authority across agent delegation chains;
- counterfactual preview of composite actions;
- approval bound to an exact plan and pre-state;
- truthful handling of reversible and irreversible effects;
- independent postcondition verification;
- portable evidence of authorization, execution, and outcome.

---

# Core Components

## cmdFirewall

A reference monitor outside the model that intercepts consequential actions before they reach the target system.

It is designed to enforce:

```text
child capability ⊆ parent capability
aggregate child exposure ≤ parent budget
approval = exact plan hash + exact pre-state
state drift = approval invalidation
no external write without a valid authority chain
```

Target protections include:

- privilege amplification through sub-agents;
- split transactions used to evade limits;
- stale or replayed approvals;
- duplicate side effects caused by retries;
- concurrent agents spending the same budget;
- unsafe tool composition;
- data exfiltration through an allowed tool;
- emergency revoke and kill switch.

## cmdShadow

A cross-domain state twin for previewing consequences before commit.

Every supported effect adapter should expose a common contract:

```text
snapshot()
simulate()
diff()
commit()
observe()
compensate()
```

cmdShadow does not claim that every real-world action can be rolled back. It classifies effects as:

- **Reversible** — a snapshot can restore the prior state;
- **Compensatable** — a later action can reduce or offset the effect;
- **Irreversible** — the effect cannot be honestly undone after commit.

Irreversible actions require stronger approval and are committed last whenever the dependency graph allows it.

## cmdProof

A task is not complete because an agent says it is complete.

cmdProof checks observable postconditions against the real system of record and produces a signed evidence bundle containing commitments to:

- normalized user intent;
- delegation and capability chain;
- policy version and decision;
- pre-state;
- simulation and state diff;
- approvals;
- exact action;
- executor and external receipts;
- actual post-state;
- verifier result;
- compensation or settlement status.

Sensitive content can remain local while hashes and selective evidence support independent verification.

## cmdMandate — Planned Crypto Control Plane

cmdOS will not build a wallet, exchange, bridge, order book, oracle, stablecoin, prediction market, or perpetual venue.

cmdMandate is planned as a venue-neutral capital firewall that compiles one high-level policy into controls supported by existing wallets, smart accounts, payment protocols, and trading venues.

Example policy:

```yaml
capital:
  maximum_total_loss: 500 USD
  maximum_daily_drawdown: 2%
  maximum_leverage: 2x
  maximum_btc_delta: 0.25 BTC

execution:
  maximum_slippage: 0.30%
  maximum_oracle_age: 15s
  forbid_unlimited_approval: true

control:
  new_contract: human_approval
  bridge: human_approval
  emergency_mode: close_only
  expires_in: 2h
```

The intended difference is global enforcement across wallets, chains, lending positions, perpetual venues, and prediction markets—not another trading interface.

Private keys must never enter the agent reasoning context.

## cmdSettle — Planned Verified Outcome Settlement

cmdSettle will use existing payment and escrow rails. Its responsibility is to release payment only after a machine-verifiable result satisfies a previously committed `TaskContract`.

Initial verifiable task classes may include:

- code build and test;
- deterministic data transformation;
- file and artifact diffs;
- deployment with health checks;
- onchain state transitions;
- SaaS actions with observable API postconditions.

cmdOS will not claim that every creative or subjective result can be verified automatically.

---

# What Runs Today

**On a developer machine, right now:**

- A Rust kernel with an object model, hash-chained ledger, transaction engine, policy gate, and dependency-ordered scheduler.
- `cmd-policy` with R0–R3 risk classes, mandates, and budgets.
- `cmd-types` with the core object model and `Resource` trait.
- `cmd-ledger` with a hash-chained, verifiable execution history.
- `cmd-auth` with credentials and expiring access keys.
- `cmd-transaction` with simulate, snapshot, execute, verify, and honest undo semantics.
- `cmd-kernel` for dependency-ordered execution plans.
- `aipc`, an MCP-style capability catalog with kernel-mediated calls.
- `cap-files`, a working reversible filesystem capability.
- `cap-terminal`, an allowlisted shell capability with unknown commands gated as R3.
- Shadow World copy-on-write forks with promote or discard.
- cmdShell, a Tauri desktop client driving the real kernel rather than a mock.
- Real data in the implemented Files, Ledger, Shadow, and Terminal screens.
- A bring-your-own-key API router with limits and key rotation.
- `cap-browser`, written and risk-classified, awaiting a real browser backend.

Around 170 tests, with CI green at the time of this README revision.

**Not built yet:** a complete threat model, Effect Manifest specification, Proof Bundle format, cross-framework firewall, real browser backend, packaged installer, crypto mandate engine, remote verifier network, and production security audit.

---

## What Happens to the Existing Work?

The product focus has narrowed, but the completed engineering remains useful.

| Existing component | Role in the focused product |
|---|---|
| `cmd-policy` | Non-bypassable firewall policy and risk decisions |
| `cmd-transaction` | World-state transaction and compensation coordinator |
| Shadow World | Counterfactual execution and state-diff engine |
| `cmd-ledger` | Foundation for proof bundles and tamper-evident evidence |
| `cmd-auth` | Principal identity, expiring authority, and revocation |
| `cmd-types` / `Resource` | Typed effect and capability contracts |
| `aipc` | Adapter surface for MCP-style tool mediation |
| `cmd-kernel` | Dependency planning and safe commit ordering |
| `cap-files` | Reference reversible effect adapter |
| `cap-terminal` | Reference high-risk execution adapter |
| `cap-browser` | Reference browser effect adapter after backend integration |
| `cmd-router` | Replaceable model integration, not a competitive moat |
| cmdShell | Local control center for diff, approval, execution, and evidence |

Existing code will not be removed merely because it is no longer a market differentiator. It will either become part of the transaction firewall or be maintained as an integration adapter.

---

# Build, Integrate, Do Not Build

## Build

cmdOS owns the parts that define its safety and verification guarantees:

- Effect Manifest and typed side-effect taxonomy;
- cross-agent transaction firewall;
- cumulative capability and budget enforcement;
- cross-domain Shadow World orchestration;
- exact-plan approval binding;
- postcondition verification;
- compensation planning;
- proof bundles;
- global crypto mandate and portfolio invariants;
- verified outcome contracts.

## Integrate

cmdOS should use replaceable adapters for existing infrastructure:

- frontier and local models;
- agent runtimes and orchestration frameworks;
- MCP and A2A;
- browsers, desktop drivers, and sandboxes;
- SaaS and developer tools;
- wallets, smart accounts, and external signers;
- transaction simulators;
- payment and escrow rails;
- DEX solvers, bridges, prediction markets, and perpetual venues;
- agent identity and reputation standards.

## Do Not Build

The following are outside the focused product:

- a general-purpose messenger or social network;
- voice/video calling infrastructure;
- another agent framework or visual workflow builder;
- a model marketplace;
- a generic agent marketplace;
- a wallet or custody service;
- a DEX aggregator, bridge, exchange, or liquidation engine;
- a prediction market or perpetual exchange;
- an oracle, stablecoin, token, or blockchain;
- a general GPU or compute marketplace.

---

# Architecture

```text
┌──────────────────────────────────────────────────────────────┐
│                   AGENTS & ORCHESTRATORS                     │
│  Codex · Claude · Copilot · LangGraph · Other runtimes       │
├──────────────────────────────────────────────────────────────┤
│                    PROTOCOL ADAPTERS                         │
│  MCP · A2A · CLI · API · RPC                                │
├──────────────────────────────────────────────────────────────┤
│                       cmdFirewall                            │
│  Identity · Delegation · Capability · Budget · Risk          │
├──────────────────────────────────────────────────────────────┤
│                        cmdShadow                             │
│  Snapshot · Simulate · Effect Graph · Diff · Compensation    │
├──────────────────────────────────────────────────────────────┤
│                     COMMIT COORDINATOR                       │
│  Exact Approval · Idempotency · Ordering · Recovery          │
├──────────────────────────────────────────────────────────────┤
│                      EFFECT ADAPTERS                         │
│  Files · Shell · Browser · SaaS · Cloud · Wallet · Venue     │
├──────────────────────────────────────────────────────────────┤
│                        cmdProof                              │
│  Observe · Postconditions · Evidence · Signed Result         │
└──────────────────────────────────────────────────────────────┘
```

---

# Local-First Trust Boundary

The user's device remains the primary trust boundary whenever practical.

- no mandatory account for local execution;
- local storage by default;
- cloud infrastructure remains optional;
- minimum necessary disclosure;
- explicit authorization for external transmission;
- secrets isolated from model context;
- user-controlled retention and deletion;
- no silent agent execution;
- replaceable external providers;
- independent kill switch and revocation.

Remote execution may be added through signed, scoped, expiring task envelopes. cmdOS does not need to invent a new P2P network to provide this guarantee.

---

# Security Model

cmdOS assumes that models, prompts, plugins, remote agents, external services, and network responses may be compromised or incorrect.

Every consequential action must be:

```text
authenticated
→ authorized
→ scoped
→ simulated where supported
→ approved at the required risk tier
→ executed through a mediated capability
→ observed against the real target
→ independently verified
→ recorded with tamper-evident evidence
```

## Risk Classes

- **R0 — Observe:** read-only state, quotes, and simulation;
- **R1 — Prepare:** plans, drafts, unsigned artifacts, and alerts;
- **R2 — Constrained Commit:** allowlisted, reversible, budget-limited actions;
- **R3 — Irreversible or High Risk:** external communication, destructive changes, new counterparties, secrets, bridges, leverage, financial commitments, and raw signatures.

## Threats in Scope

- prompt injection;
- malicious tools and plugins;
- privilege escalation through delegation;
- replay and stale approval;
- duplicated side effects;
- budget splitting;
- concurrent overspending;
- compromised devices or executors;
- data exfiltration;
- hallucinated completion;
- tampered tasks or results;
- state drift between approval and execution;
- unsafe financial exposure across multiple venues.

No production security or cryptographic claim should be made until it is documented, tested, and independently reviewed.

---

# Roadmap

The roadmap uses stage gates rather than a large feature list. A phase advances only when its safety claims are measurable.

## Gate 0 — Product Focus ✅

- [x] Separate completed implementation from future claims.
- [x] Define `Build / Integrate / Do Not Build` boundaries.
- [x] Remove messenger, exchange, wallet, marketplace, and token ambitions from the core roadmap.
- [x] Position existing File, Terminal, Browser, router, and model work as adapters.

## Gate 1 — Local Proof Loop 🔵

- [x] Kernel-mediated Files and Terminal capabilities.
- [x] R0–R3 policy gate.
- [x] Snapshot, simulation, execution, verification, and honest undo.
- [x] Shadow World copy-on-write branches.
- [x] Hash-chained ledger.
- [ ] Publish the threat model.
- [ ] Define Effect Manifest v0.
- [ ] Bind approvals to exact plan and pre-state hashes.
- [ ] Define Proof Bundle v0.
- [ ] Add protected postcondition verifiers.
- [ ] Add idempotency and compensation fault-injection tests.
- [ ] Complete the real browser backend.
- [ ] Ship the packaged desktop installer.

**Exit criteria:** every declared external side effect is mediated and receipted; supported postconditions match observed state at least 95% of the time; the red-team suite finds no policy bypass.

## Gate 2 — Framework-Independent Firewall

- [ ] MCP proxy.
- [ ] A2A delegation gateway.
- [ ] Adapters for at least three independent agent runtimes.
- [ ] Capability attenuation across sub-agents.
- [ ] Aggregate budget and exposure reservation.
- [ ] Split-action, stale-approval, replay, and concurrency detection.
- [ ] Paired-device approval and remote revoke.

**Exit criteria:** seeded privilege-amplification and budget-evasion tests are blocked; normal policy overhead stays below the declared performance budget; real design partners route production writes through the firewall.

## Gate 3 — cmdMandate Testnet

- [ ] Financial Policy IR.
- [ ] Financial Capability Capsule.
- [ ] External signer and smart-account adapters.
- [ ] Cross-wallet and cross-venue portfolio state.
- [ ] Transaction and strategy simulation adapters.
- [ ] Prediction resolution-risk analyzer.
- [ ] Perpetual risk governor and close-only mode.
- [ ] Emergency revoke independent of the agent.

**Exit criteria:** no private key enters agent context; no unauthorized testnet execution; at least 10,000 adversarial simulations; external security review; simulation/post-state mismatch remains below the declared threshold for supported actions.

## Gate 4 — Limited Real-World Commit

- [ ] Strict capital and action limits.
- [ ] Re-simulation immediately before signing.
- [ ] Independent post-state observers.
- [ ] Recovery and compensation playbooks.
- [ ] Signed remote execution results.
- [ ] Limited mainnet and production pilots after review.

## Gate 5 — Verified Outcome Settlement

- [ ] `TaskContract` and `ResultReceipt` schemas.
- [ ] Multiple independent verifier support.
- [ ] Existing payment and escrow rail integrations.
- [ ] Dispute, timeout, refund, and compensation rules.
- [ ] Receipt-grounded reputation adapters.
- [ ] Open verifier CLI and conformance tests.

---

# Project Status

cmdOS is under active development.

The current repository contains a working local kernel, policy system, transaction engine, Shadow World, ledger, desktop client, filesystem capability, terminal capability, and the initial browser capability contract.

The cross-framework firewall, Proof Bundle standard, crypto mandate engine, verified settlement layer, and production security guarantees are roadmap work and must not be represented as shipped.

Feature claims should be updated only when supported by:

- working code;
- automated and adversarial tests;
- reproducible demonstrations;
- documented limitations;
- security review;
- real post-state verification.

---

# Contributing

Before proposing a feature, classify it:

1. Is it a cmdOS safety or verification primitive?
2. Is it an adapter to an existing product or standard?
3. Is it a commodity feature that should not be built here?

Contributions to the trusted path must include:

- an explicit threat model;
- declared side effects;
- capability and risk classification;
- tests for denial and bypass attempts;
- postcondition verification;
- truthful reversibility or compensation behavior;
- no permission-boundary bypass.

---

# Security Disclosure

Do not publish exploitable vulnerabilities in public issues.

Before public testing, the repository should include `SECURITY.md`, a private reporting channel, supported-version policy, severity definitions, disclosure timelines, and response expectations.

---

<div align="center">

<img src="./assets/footer.svg" width="100%" alt="cmdOS footer animation" />

### Agents plan. cmdOS makes consequences safe to commit.

</div>
