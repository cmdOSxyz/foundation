# RFC-0008: cmd-transaction — Reversible Execution Engine

Version: 1.0
Status: Accepted
Category: Architecture (core)
Author: Lead Architect
Depends on: RFC-0004 (Object Model), RFC-0007 (Ledger)
Implemented by: `kernel/cmd-transaction`

---

# 1. Summary

The engine that makes cmdOS reversible. Every side-effecting action runs the
pipeline: simulate → snapshot → execute → verify → commit | rollback. Dry-run,
undo, and shadowing are all expressed through it.

The engine is **abstract**: it drives a `Resource` trait and knows nothing about
files, APIs, or VMs. The same engine will later shadow a filesystem
(`capabilities/files`) or an entire Machine (RFC-0006) unchanged.

# 2. Motivation

Reversibility is cmdOS's core promise. Building it as a generic engine over a
`Resource` interface — rather than hard-wiring it to the filesystem — keeps the
architecture modular and lets shadowing scale from a file to a whole computer.

# 3. Design

`Resource` trait (interface): `simulate`, `snapshot` (returns `Option` — read-only
steps take none), `execute`, `verify`, `restore`.

`TransactionEngine`: borrows a `Ledger` and, per step, produces a `Transaction`
record while writing an `Event` at each phase. Failure policy: **verify-fail
auto-rolls-back** to the captured pre-state. R0 read-only steps skip snapshotting.
An explicit `rollback` implements user-initiated Undo and refuses non-reversible
transactions.

This crate is also the first place two kernel crates compose: `cmd-transaction`
records into `cmd-ledger` using `cmd-types::Event`.

# 4. Reference & Contracts

The behavior mirrors the prototype's `filesystem.ts`
(dry-run/execute/verify/undo). This RFC ships the engine plus an in-memory
`MemoryResource` for testing; the real filesystem `Resource` lands in
`capabilities/files` and must pass the 8 behavior contracts in
`prototype/tests/filesystem.behavior.test.ts`.

# 5. Testing

7 unit tests, all green: successful commit changes state; verify-fail auto-
rollback restores pre-state; read-only takes no snapshot; execute-fail does not
commit; explicit rollback restores + records; rollback refused when irreversible;
ledger hash-chain stays intact across transactions. `cargo test -p cmd-transaction`
is green.

# 6. Reversibility Impact

This crate *is* the reversibility machinery. R0/R1 fully reversible; R2 compensable
(future resources); R3 never auto-run (enforced upstream by policy, RFC-0006).

# 7. Next

`capabilities/files`: a real filesystem `Resource` validated against the prototype
behavior contracts — the first Rust component to replace a prototype capability.
