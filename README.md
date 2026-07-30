<div align="center">

<img src="./assets/hero.svg" width="100%" alt="cmdOS safe execution for AI agents" />

<br/>

# cmdOS

### An AI agent built to act in the real world

**cmdOS simulates consequences, enforces limits, coordinates approval, executes authorized actions, and verifies the final outcome.**

[![Status](https://img.shields.io/badge/status-active%20development-22c55e?style=for-the-badge&labelColor=07110b)](#project-status)
[![Architecture](https://img.shields.io/badge/architecture-local--first-22c55e?style=for-the-badge&labelColor=07110b)](#local-first)
[![Safety](https://img.shields.io/badge/safety-transaction%20firewall-22c55e?style=for-the-badge&labelColor=07110b)](#cmdfirewall)
[![Evidence](https://img.shields.io/badge/results-verifiable-22c55e?style=for-the-badge&labelColor=07110b)](#cmdproof)

<br/>

[Overview](#overview) · [How It Works](#how-it-works) · [Capabilities](#core-capabilities) · [Available Today](#available-today) · [Architecture](#architecture) · [Roadmap](#roadmap) · [Security](#security)

</div>

## Overview

cmdOS is a local-first AI agent and execution control layer for actions that affect real systems.

It connects AI agents to files, terminals, browsers, applications, cloud services, wallets, and markets through constrained capabilities. Every sensitive action can be inspected, limited, approved, executed, and verified from one control surface.

```text
AI Agent
   ↓
cmdOS Safety and Execution Layer
   ↓
Files · Shell · Browser · SaaS · Cloud · Wallets · Markets
```

cmdOS is designed around one principle:

> **Agents plan. cmdOS makes consequences safe to commit.**

## User Experience

A user gives cmdOS an outcome instead of a sequence of clicks.

```text
Prepare the release, run the tests, deploy the approved build,
and confirm that production is healthy.
```

Before real state changes, cmdOS presents:

- the proposed execution plan
- the systems and resources involved
- requested capabilities
- expected state changes
- risk level and limits
- actions that require approval
- available recovery or compensation paths
- verification criteria

After execution, cmdOS checks the real target state and returns the result with supporting evidence.

## How It Works

```text
Intent
→ Effect Graph
→ Permission and Risk Check
→ Shadow Simulation
→ State Diff
→ Exact Approval
→ Commit
→ Observe Real State
→ Verify or Compensate
→ Proof Bundle
```

### 1. Intent

The user describes the desired result in natural language.

### 2. Effect Graph

cmdOS converts the request into typed actions, dependencies, capabilities, limits, and verification rules.

### 3. Permission and Risk Check

Every action passes through policy before execution. Sensitive or irreversible actions require stronger approval.

### 4. Shadow Simulation

Supported actions run against an isolated state or simulation backend. cmdOS calculates the expected state diff before commit.

### 5. Exact Approval

Approval is bound to the exact plan and the exact state that was reviewed. If relevant state changes, the plan must be checked again.

### 6. Commit

Authorized actions are executed through mediated capabilities with ordering, idempotency, timeouts, and resource limits.

### 7. Verification

cmdOS observes the real target and evaluates the required postconditions.

### 8. Proof Bundle

The final result includes tamper-evident evidence for the intent, policy decision, approval, execution, and verified post-state.

# Core Capabilities

## cmdFirewall

cmdFirewall is the control boundary between an AI agent and external systems.

It manages:

- user, device, agent, and sub-agent authority
- capability scopes
- R0 to R3 risk classes
- budgets and rate limits
- approval requirements
- task and session expiry
- replay protection
- duplicate action protection
- concurrent budget reservation
- emergency revoke
- kill switch

Delegated authority follows strict containment rules:

```text
child capability ⊆ parent capability
aggregate child exposure ≤ parent budget
approval = exact plan hash + exact pre-state
state drift = approval invalidation
```

## cmdShadow

cmdShadow previews the consequences of a plan before real execution.

A supported effect adapter exposes:

```text
snapshot()
simulate()
diff()
commit()
observe()
compensate()
```

Actions are classified by their real recovery properties:

- **Reversible:** a snapshot can restore the previous state
- **Compensatable:** a follow-up action can reduce or offset the effect
- **Irreversible:** the effect cannot be restored after commit

The approval screen clearly identifies the class of each action and the available recovery path.

## cmdProof

cmdProof verifies that an action reached its intended result.

Verification methods include:

- target-state inspection
- file existence and checksums
- structured assertions
- API confirmations
- delivery receipts
- screenshots and visual comparison
- unit and integration tests
- deployment health checks
- signed remote results
- blockchain and market post-state observation

A Proof Bundle can contain commitments to:

- normalized intent
- capability and delegation chain
- policy version and decision
- pre-state
- simulation and state diff
- approval signatures
- exact action
- external receipts
- actual post-state
- verifier result
- compensation or settlement status

Sensitive content can remain local while hashes and selective evidence support independent verification.

## cmdMandate (Planned)

cmdMandate is a venue-neutral crypto risk and policy control plane.

It applies one policy across connected wallets, smart accounts, chains, payment protocols, lending positions, perpetual venues, and prediction markets.

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

Planned controls include:

- cross-wallet and cross-venue exposure
- leverage and drawdown limits
- concentration and correlation limits
- liquidity and slippage requirements
- oracle freshness and divergence checks
- liquidation distance
- prediction resolution risk
- close-only emergency mode
- session expiry and immediate revoke
- external signing without exposing private keys to the agent

## cmdSettle (Planned)

cmdSettle coordinates payment after a machine-verifiable result satisfies a committed `TaskContract`.

Initial task classes include:

- code build and test
- deterministic data transformation
- file and artifact diffs
- deployment with health checks
- onchain state transitions
- SaaS actions with observable API postconditions

The settlement flow uses existing payment and escrow integrations while cmdOS verifies the required outcome.

## cmdCapital (Planned)

cmdCapital is the market the other five capabilities make possible: an onchain verification layer and marketplace for autonomous trading agents.

It answers two failures that the current market treats as unavoidable.

| Failure | Today | cmdCapital |
| --- | --- | --- |
| Fake track record | A screenshot of a PnL curve costs nothing to produce and nothing to fake. | Every order is signed inside a hardware enclave and the record is derived from venue fills, not reported by the agent. |
| Counterparty risk | Following a strategy usually means handing someone the ability to move your funds. | A delegated agent holds a session key scoped to trading only. The right to withdraw never leaves the wallet owner. |

The flagship agent ships under the commercial name **AuraCore**, with the message *"Your autonomous nexus"*.

Four planned layers, in the order value passes through them:

- **NexusKernel** — execution and hardware. The agent runs inside a TEE (Intel SGX or AMD SEV). The private key is generated inside the enclave and cannot be extracted. Every order carries a signature and a remote attestation proof.
- **Metric Engine** — audit and data. A decentralized indexer reads fills from the venues themselves and derives ROI, maximum drawdown, Sharpe, and win rate. The result is written onchain rather than published by the agent.
- **NexusShield** — user custody. Account Abstraction (ERC-4337) smart wallets, with email or social sign-in instead of a seed phrase. The agent receives a session key that permits trades and forbids withdrawals.
- **NexusEscrow** — settlement. A contract splits the performance fee at the end of each cycle. Built last and gated behind human approval, because a settlement bug is the one class of bug that cannot be rolled back.

Four surfaces sit on top of those layers:

- **Super App** — Telegram mini-app and web dashboard. Ask in plain language, read position and risk state back.
- **Marketplace** — reviewed agents on an onchain leaderboard, ranked on attested numbers rather than claims.
- **Smart Wallet** — an ERC-4337 account with no seed phrase to lose, non-custodial by construction.
- **Vaults** — passive allocation across top-ranked agents, rebalanced on a fixed cycle.

The omnichain loop runs in three beats:

- **Signal** — inside the enclave, the agent watches EVM, SVM, and MoveVM venues along with social flow, and decides.
- **Synchronize** — the TEE signs the raw transactions, bridges through Wormhole or LayerZero, and a relayer applies them across delegated wallets together so followers do not absorb the slippage of going last.
- **Yield** — performance fees split at the end of the cycle. The developer that built the agent is paid, and the treasury takes the remainder, part of which funds a buy-back and burn of the governance token.

The user path and the control at each step:

```text
Fund      → a smart wallet is created; deposit from an exchange or an existing wallet
Choose    → read attested performance in the marketplace, then decide the allocation
Delegate  → grant a trade-only session key and set the drawdown cutoff
Watch     → position, cash flow, and return stream back to the dashboard
Settle    → stop at any time; the contract splits the fee and returns the rest
```

Policy constants from the specification. These are settings, not results:

```yaml
agent_permissions: trade only, withdrawal denied
circuit_breaker:   agent suspended at the drawdown the owner sets
performance_fee:   20% of realised profit, charged on profit only
fee_split:         85% agent developer, 15% protocol treasury
settlement_gate:   R3, human approval required
```

Nothing in cmdCapital is built yet. Every surface stays locked and names its missing dependency until the dependency exists. The leaderboard requires the TEE attestation pipeline, the venue indexer and metric derivation, and at least one reviewed agent with a settled cycle. No performance figure is published before then, not even as an example.

Full specification: [`docs/01-vision/cmdcapital-spec.md`](docs/01-vision/cmdcapital-spec.md).

# Available Today

The repository currently includes:

- a Rust kernel with a typed object model
- `cmd-policy` with R0 to R3 risk classes, mandates, and budgets
- `cmd-types` with the core object model and `Resource` trait
- `cmd-ledger` with a hash-chained execution history
- `cmd-auth` with credentials and expiring access keys
- `cmd-transaction` with simulation, snapshot, execution, verification, and honest undo semantics
- `cmd-kernel` with dependency-ordered execution plans
- `cmd-approval` binding an approval to an exact plan digest and an exact pre-state
- `cmd-proof` with Proof Bundle v0: hash-chained evidence for one execution, verifiable without the original data
- `aipc`, an MCP-style capability catalog with kernel-mediated calls
- `cap-files`, a working reversible filesystem capability
- `cap-terminal`, an allowlisted shell capability with R3 gating for unknown commands
- Shadow World copy-on-write forks with promote or discard
- cmdShell, a Tauri desktop client connected to the real kernel
- real data in the implemented Files, Ledger, Shadow, and Terminal screens
- a bring-your-own-key API router with limits and key rotation
- `cap-browser`, with capability and risk contracts ready for a real browser backend

The project has 204 tests, all green when this README was updated.

## In Development

- complete threat model
- Effect Manifest v0
- signed Proof Bundles — v0 is tamper-evident but carries no signature, so it proves
  consistency rather than origin
- protected postcondition verifiers
- idempotency and compensation fault-injection tests
- real browser backend
- complete light and dark appearance parity
- agent management interface
- packaged desktop installer

## Planned

- MCP execution proxy
- A2A delegation gateway
- adapters for multiple agent runtimes
- paired-device approval
- remote revoke and signed task results
- cmdMandate testnet release
- independent verifier support
- verified outcome settlement
- open verifier CLI and conformance suite
- attested agent track records and the cmdCapital marketplace

# Integrations

cmdOS uses adapters to connect existing tools and infrastructure.

Target integration categories include:

- AI models and agent runtimes
- MCP and A2A
- browser and desktop automation backends
- developer tools and SaaS platforms
- local and cloud execution environments
- wallets, smart accounts, and external signers
- transaction simulation services
- payment and escrow services
- DEX solvers and bridges
- prediction markets and perpetual venues
- trusted execution environments and attestation services
- account abstraction wallets and session-key delegation
- identity, reputation, and verification standards

Adapters remain replaceable so policy, evidence, and local user control do not depend on one provider.

# Architecture

```text
┌──────────────────────────────────────────────────────────────┐
│                   AGENTS AND ORCHESTRATORS                   │
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

# Local-First

The user's device is the primary trust boundary whenever practical.

- no mandatory account for local execution
- local storage by default
- optional cloud infrastructure
- minimum necessary disclosure
- explicit authorization for external transmission
- secrets isolated from model context
- user-controlled retention and deletion
- no silent agent execution
- replaceable external providers
- independent kill switch and revocation

Remote execution uses signed, scoped, and expiring task envelopes.

# Security

cmdOS treats models, prompts, plugins, remote agents, external services, and network responses as untrusted inputs.

Every consequential action follows this path:

```text
authenticate
→ authorize
→ scope
→ simulate where supported
→ approve at the required risk tier
→ execute through a mediated capability
→ observe the real target
→ verify the required postconditions
→ record tamper-evident evidence
```

## Risk Classes

- **R0 Observe:** read-only state, quotes, and simulation
- **R1 Prepare:** plans, drafts, unsigned artifacts, and alerts
- **R2 Constrained Commit:** allowlisted, reversible, and budget-limited actions
- **R3 High Risk:** destructive changes, external communication, secrets, new counterparties, bridges, leverage, financial commitments, and raw signatures

## Threats in Scope

- prompt injection
- malicious tools and plugins
- privilege escalation through delegation
- replay and stale approval
- duplicated side effects
- budget splitting
- concurrent overspending
- compromised devices or executors
- data exfiltration
- hallucinated completion
- tampered tasks or results
- state drift between approval and execution
- unsafe financial exposure across multiple venues

Production security and cryptographic guarantees require documented protocols, adversarial testing, and independent review.

# Roadmap

## Phase 1: Local Proof Loop (Current)

- [x] Kernel-mediated Files and Terminal capabilities
- [x] R0 to R3 policy gate
- [x] Snapshot, simulation, execution, verification, and honest undo
- [x] Shadow World copy-on-write branches
- [x] Hash-chained ledger
- [x] Bind approval to exact plan and pre-state hashes
- [x] Define Proof Bundle v0
- [ ] Publish the threat model
- [ ] Define Effect Manifest v0
- [ ] Sign Proof Bundles so they prove origin, not only consistency
- [ ] Add protected postcondition verifiers
- [ ] Complete the real browser backend
- [ ] Ship the packaged desktop installer

Completion target:

- every declared external side effect is mediated and receipted
- supported postconditions match observed state at least 95% of the time
- the red-team suite finds no policy bypass

## Phase 2: Agent Interoperability

- [ ] MCP execution proxy
- [ ] A2A delegation gateway
- [ ] adapters for at least three independent agent runtimes
- [ ] capability attenuation across sub-agents
- [ ] aggregate budget and exposure reservation
- [ ] split-action, stale-approval, replay, and concurrency detection
- [ ] paired-device approval and remote revoke

## Phase 3: cmdMandate Testnet

- [ ] Financial Policy IR
- [ ] Financial Capability Capsule
- [ ] external signer and smart-account adapters
- [ ] cross-wallet and cross-venue portfolio state
- [ ] transaction and strategy simulation adapters
- [ ] prediction resolution-risk analyzer
- [ ] perpetual risk governor and close-only mode
- [ ] emergency revoke independent of the agent

Completion target:

- no private key enters agent context
- no unauthorized testnet execution
- at least 10,000 adversarial simulations
- independent security review
- simulation and post-state mismatch stays below the declared threshold for supported actions

## Phase 4: Verified Real-World Commit

- [ ] strict capital and action limits
- [ ] re-simulation immediately before signing
- [ ] independent post-state observers
- [ ] recovery and compensation playbooks
- [ ] signed remote execution results
- [ ] limited production and mainnet pilots after review

## Phase 5: Verified Outcome Settlement

- [ ] `TaskContract` and `ResultReceipt` schemas
- [ ] multiple independent verifier support
- [ ] payment and escrow integrations
- [ ] dispute, timeout, refund, and compensation rules
- [ ] receipt-grounded reputation adapters
- [ ] open verifier CLI and conformance tests

## Phase 6: cmdCapital

- [ ] TEE execution environment and enclave key generation
- [ ] remote attestation attached to every signed order
- [ ] venue indexer and derived metrics written onchain
- [ ] ERC-4337 smart wallets with trade-only session keys
- [ ] drawdown circuit breaker independent of the agent
- [ ] agent review process and the onchain leaderboard
- [ ] performance-fee escrow behind an R3 human gate

Completion target:

- no enclave key is extractable and no order is accepted without a valid attestation
- no delegated key can withdraw funds
- every published metric is derived from venue data and never reported by the agent
- the leaderboard stays locked until its named dependencies are live

# Project Status

cmdOS is under active development.

The local kernel, policy system, transaction engine, Shadow World, ledger, desktop client, filesystem capability, terminal capability, and initial browser capability contract are implemented.

The cross-framework firewall, Proof Bundle standard, cmdMandate, distributed verification, settlement layer, cmdCapital, and production security guarantees are under development or planned.

cmdCapital has no implementation in this repository. It is documented here as a design commitment so the dependency order stays visible: attested execution and derived metrics come before any marketplace surface, and settlement comes last.

# Contributing

Contributions to the trusted execution path should include:

- an explicit threat model
- declared side effects
- capability and risk classification
- tests for denial and bypass attempts
- postcondition verification
- truthful reversibility or compensation behavior
- no permission-boundary bypass

# Security Disclosure

Do not publish exploitable vulnerabilities in public issues.

Before public testing, the repository will provide `SECURITY.md`, a private reporting channel, supported-version policy, severity definitions, disclosure timelines, and response expectations.

<div align="center">

<img src="./assets/footer.svg" width="100%" alt="cmdOS footer animation" />

### Agents plan. cmdOS makes consequences safe to commit.

</div>
