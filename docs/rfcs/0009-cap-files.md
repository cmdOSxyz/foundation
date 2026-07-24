# RFC-0009: cap-files — The Reversible Filesystem Capability

Version: 1.0
Status: Accepted
Category: Capability
Author: Lead Architect
Depends on: RFC-0004 (Object Model), RFC-0008 (Transaction Engine)
Implemented by: `capabilities/files`

---

# 1. Summary

The first Rust component to replace a prototype capability. `cap-files` implements
the `cmd_transaction::Resource` trait so every file operation runs through the
reversible engine. This is the strangler-fig milestone: Rust reproducing the
prototype's filesystem behavior, validated against its 8 behavior contracts.

# 2. Actions

- `list` — R0 read-only; no snapshot, verifies trivially.
- `rename` / `move` — R1 reversible; undo renames back. `to` without a path
  separator stays in the source directory (matches the prototype).
- `delete` — R1; **safe delete**: the file is moved to a `.cmdos-trash` folder,
  so it is recoverable. Undo restores it to its original path.

# 3. Reversibility Design

The `FsSnapshot` enum records exactly what undo needs: `Moved { original,
moved_to }` for rename/move, `Trashed { original, trashed_to }` for delete. The
trash path is computed **deterministically** in `snapshot` (name-based, not
timestamped) so `snapshot` and `execute` agree and undo can find the file — a gap
found and closed during implementation, proven by an end-to-end
delete-through-engine-then-undo test.

# 4. Validation — the 8 Behavior Contracts

Reproduces `prototype/tests/filesystem.behavior.test.ts`:
dry-run is read-only and truthful; dry-run warns on missing source;
rename execute→verify→undo round-trip; verify fails when the rename never
happened; list is read-only with no snapshot; delete moves to trash and is
recoverable; rename of a missing source fails cleanly with no partial effect;
rename target stays in the same directory.

Plus 2 end-to-end tests driving the real `TransactionEngine`: rename-then-undo
and delete-then-undo (restore from trash), both recording an intact ledger chain.

# 5. Testing

`cargo test -p cap-files` — 10 tests green, no warnings.

# 6. Strangler-Fig Status

`cap-files` now reproduces the observable behavior of
`prototype/capabilities/filesystem.ts`. The prototype remains the runnable
reference; the Rust capability is ready to be wired into the runtime as the
replacement once AIPC/MCP exposure (RFC-0010+) lands.

# 7. Next

`cmd-policy` (R0–R3 gate + budgets + mandates) and `cmd-kernel` (Intent
Scheduler) to complete the kernel, then AIPC to expose capabilities over MCP and
run the first full end-to-end intent in Rust.

## Amendment — move creates its destination

`fs::rename` does not create the target folder, so "move this into pdf/" failed
whenever pdf/ did not already exist — an ordinary request. Move now creates the
destination folder, and undo removes it again if nothing else went into it, so a
reversal leaves no trace. Found while making the Shadow app able to show a real
tidy.
