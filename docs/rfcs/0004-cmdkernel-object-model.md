# RFC-0004: cmdKernel Object Model (cmd-types)

Version: 1.0
Status: Accepted
Category: Architecture
Author: Lead Architect
Implemented by: `kernel/cmd-types`

---

# 1. Summary

Defines the shared object model of cmdOS — the "nouns" every other crate depends
on. It ports the prototype's TypeScript `schemas/` into typed Rust and adds the
three Strategy-v2 primitives that have no prototype equivalent: **Transaction**,
**Mandate**, and **Budget**, plus the canonical **RiskClass** (R0–R3) and the
two-tier **Agent** model.

# 2. Motivation

Everything above the object model (scheduler, transactions, policy, ledger,
agent, shell) references these types. Fixing them once, correctly, prevents
churn later. The prototype `schemas/` proved the core shapes in production; this
RFC promotes them to the kernel language (Rust) and extends them for the v2
architecture.

# 3. Design

Modules in `cmd-types`:

- `common` — `Id` (UUID newtype), `Timestamp` (UTC), and **`RiskClass`**:
  `R0ReadOnly`, `R1Reversible`, `R2Compensable`, `R3Irreversible`.
  `may_be_autonomous()` returns false for R3 (load-bearing rule);
  `is_reversible()` true for R0/R1.
- `intent` — `Intent`, `IntentSource`, `IntentStatus`, `Objective`.
- `plan` — `ExecutionPlan`, `PlanStep`, `PlanStatus`, `StepStatus`;
  `ready_steps()` returns pending steps whose deps are satisfied.
- `capability` — `Capability`, `CapabilityAction` (contract shape only).
- `authority` — **`Budget`** (kernel-enforced money/action/time limits) and
  **`Mandate`** (signable grant of authority; AP2-compatible in spirit).
- `transaction` — **`Transaction`** and `TransactionPhase`
  (simulated → snapshotted → executed → verified → committed | rolled_back).
- `agent` — **`Agent`** with `AgentKind` (`User` | `Supervisor`), `TrustLevel`,
  and `PermissionRequest` / `PermissionDecision`.
- `event` — `Event`, `EventType` for the append-only ledger.

All types derive `Serialize`/`Deserialize`. Maps use `BTreeMap` for
deterministic serialization (stable hashing into the ledger).

# 4. Mapping: prototype schemas → cmd-types

| Prototype (`schemas/*.ts`) | cmd-types |
|---|---|
| `intent.ts` Intent/Source/Status | `intent` module |
| `execution-plan.ts` ExecutionPlan/PlanStep | `plan` module |
| `capability.ts` Capability/Action | `capability` module |
| `capability.ts` `RiskLevel` (read_only/reversible/destructive/external) | `common::RiskClass` (R0/R1/R2/R3) |
| `permission-request.ts` | `agent::PermissionRequest` |
| `event.ts` | `event` module |
| — (new) | `authority::Budget`, `authority::Mandate` |
| — (new) | `transaction::Transaction` |
| — (new) | `agent::Agent` (two-tier) |

# 5. Mapping: archived kernel managers → cmd-types + downstream

| Old kernel manager (archived roadmap) | New home |
|---|---|
| Object Manager | `cmd-types` object model + `cmd-kernel` registry |
| State Manager | `cmd-kernel` (uses these types) |
| Command Manager | `cmd-kernel` (semantic syscall dispatch) |
| Execution Planner | `agent/alios` + `cmd-kernel` |
| Transaction Manager | `cmd-transaction` (drives `Transaction`) |
| Consistency Manager | `cmd-transaction` verify phase |
| Lock Manager | `cmd-kernel` resource leases |
| Memory Manager | `agent/alios` + `services/semfs` |
| Cache Manager | deferred (H2) |

# 6. Testing

`cmd-types` ships 9 unit tests covering: R3-never-autonomous, budget ceilings
(including the no-money-authority case), mandate active/revoked, transaction
rollback requiring a snapshot, supervisor-vs-user distinction, plan dependency
readiness, and JSON round-trip. `cargo test -p cmd-types` is green.

# 7. Reversibility Impact

This RFC introduces no side effects; it defines the types through which all
reversibility (R0–R3, Transaction) is later expressed.

# 8. Next

`cmd-ledger` (consumes `Event`) then `cmd-transaction` (drives `Transaction`),
each validated against the prototype behavior contracts.
