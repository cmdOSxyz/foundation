# cmdCapital

**A marketplace for autonomous trading agents, where the track record is derived
rather than reported and a delegated agent can trade but never withdraw.**

> **Status: specification.** No cmdCapital code exists in this repository yet.
> [RFC-0024](docs/rfcs/0024-cmdcapital.md) is a Draft that deliberately blocks
> implementation until the decisions in its section 5 are resolved. Nothing below is
> a claim about something that runs today.

---

## The two failures this exists to remove

**Fake track record.** A screenshot of a PnL curve costs nothing to produce and nothing
to fake. Nobody can tell a good strategy from a well-edited image.

**Counterparty risk.** Following a strategy usually means handing someone the ability to
move your funds.

## The approach

**Performance is computed, not claimed.** An indexer reads fills from the venue and
derives ROI, maximum drawdown and win rate. The number never originates from the agent.

**Delegation without custody.** The agent receives a session key scoped to trading.
Withdrawal is rejected at the wallet, not merely left unused, so a fully compromised
agent still cannot move principal out.

**Bounded loss.** A circuit breaker suspends the agent at a drawdown the owner sets.

## Product surface

| | |
| --- | --- |
| **Marketplace** | Agents listed with derived ROI, win rate and maximum drawdown |
| **Smart wallet** | Account-abstraction wallet, social or email sign-in, no seed phrase. Withdrawal rights stay entirely with the owner |
| **Copy trade** | Pick an agent, set an amount, set a drawdown limit |
| **Vaults** | For passive allocation across top-performing agents |
| **Super App** | Telegram Mini-App and web dashboard |

Performance fee is charged on realised profit only.

## Architecture

Four layers, specified in [`docs/01-vision/cmdcapital-spec.md`](docs/01-vision/cmdcapital-spec.md):

- **NexusKernel** — agent execution with signed orders and attestation
- **Metric Engine** — venue indexer that derives the published metrics
- **NexusShield** — session-key custody, withdrawal denied
- **NexusEscrow** — settlement and performance-fee split

## What must be decided before any code is written

[RFC-0024](docs/rfcs/0024-cmdcapital.md) names five open decisions and refuses to accept
the design until they are answered. They are load-bearing, not details:

1. **Enrolment.** Attestation proves code ran in an enclave. It does not prove the
   operator listed every agent it ran. Without an on-chain enrolment commitment made
   *before* the first trade, an operator can silently drop losing agents and the
   surviving leaderboard is fully attested and still false.
2. **Relayer trust.** Executing across many follower wallets means the relayer sees trade
   intent before it lands — a front-running position over every follower.
3. **Circuit breaker input.** Which price, which oracle, what staleness bound, and what
   unwind policy. A threshold without a source is not a control.
4. **High-water mark.** Without one, an agent that loses 30% and then gains 30% charges a
   performance fee while the investor is still down. This is a correctness property.
5. **The token.** Buy-back and burn is specified against an asset with no name, no supply
   and no stated rights.

The dependency order is fixed: attested execution and derived metrics come before any
marketplace surface, and settlement is built last. Building the marketplace first would
produce exactly the unverified leaderboard this exists to replace.

## What is in the repository today

A Rust workspace of seven crates — the safety and evidence core cmdCapital builds on:

| Crate | Role | Used by cmdCapital for |
| --- | --- | --- |
| `cmd-types` | Object model | Foundation for the rest |
| `cmd-policy` | R0–R3 risk classes, budgets, mandates | Session-key scope, circuit breaker |
| `cmd-ledger` | Append-only hash-chained record | Order and attribution history |
| `cmd-proof` | Proof Bundle v0 | Evidence a third party can check |
| `cmd-approval` | Binds approval to an exact plan digest | Human gate on settlement |
| `cmd-transaction` | simulate → verify → commit/rollback | — |
| `cmd-shadow` | Copy-on-write forks | — |

Not yet built: the venue indexer, the session-key wallet, settlement, and every
user-facing surface.

## Build

```bash
cargo build --workspace
cargo test  --workspace          # 29 tests
bash tools/ci/check-docs.sh      # documentation invariants
npm install && npm test          # prototype behavior contracts
```

## History

This repository previously targeted an operating system for AI agents. That direction
was retired on 2026-08-04 and the components serving it were removed; the reasoning and
the recovery instructions are in [`docs/archive/README.md`](docs/archive/README.md).

## Risk classes

Reading market data is R0. An unsigned plan is R1. A trade inside an accepted mandate,
under budget, on an allowlisted venue is R2. Granting a session key, raising a limit,
adding a venue, bridging, and every settlement or fee transfer are **R3 — always
human-gated**. Bridging never inherits R2 from the trade that motivated it: it is
irreversible and the failure mode is total loss of the bridged amount.

## License

Apache-2.0. See [`LICENSE`](LICENSE). Security reporting: [`SECURITY.md`](SECURITY.md).
