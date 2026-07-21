# RFC-0007: cmd-ledger — Append-Only Hash-Chained Audit Ledger

Version: 1.0
Status: Accepted
Category: Architecture
Author: Lead Architect
Depends on: RFC-0004 (Object Model)
Implemented by: `kernel/cmd-ledger`

---

# 1. Summary

The audit ledger records every action, score, and intervention as an immutable,
hash-chained sequence of `Event`s. Entries can be appended and read, never
modified or deleted. Each entry's hash covers the previous entry's hash, so
altering any past entry breaks the chain from that point on — tampering is
detectable by anyone, without a secret.

# 2. Motivation

Trust in an autonomous agent requires proof of exactly what it did. Reversibility
(cmd-transaction) needs a record of what to reverse. Supervision (Alios) needs an
immutable place to write its judgments. All three rest on this ledger, so it is
built before any write-capable component.

# 3. Reference Behavior

Two prototype files define the behavior this crate reproduces and extends:
- `prototype/kernel/event-log.ts` — append-only, immutable, reads return copies.
- `prototype/apps/desktop/receipt-store.cjs` — a SHA-256 content hash makes
  tampering detectable.

`cmd-ledger` unifies both and adds hash **chaining** (each entry binds the prior
entry's hash), turning a set of independently-hashed records into a
self-verifying chain — the "signed ledger" of Strategy v2.

# 4. Design

- `LedgerEntry { index, prev_hash, hash, event }`.
  `hash = SHA256(index || prev_hash || event_json)`. Event JSON is deterministic
  (BTreeMaps), so hashes are stable across runs.
- `Ledger`: `append` (returns a copy), `all` (copy), `len` / `is_empty`,
  `by_type`, `by_plan`, and `verify` (walks the chain, returns
  `Err(BrokenAt(index))` at the first inconsistency).
- No API mutates or removes a stored entry. Reads return clones so callers cannot
  reach in and rewrite history.

# 5. Testing

6 unit tests, all green: sequential indexing + hash linking, empty-ledger verify,
intact-chain verify, **tampering detection**, reads-return-copies,
filter-by-type/plan. `cargo test -p cmd-ledger` is green.

# 6. Reversibility Impact

None directly; the ledger has no side effects on user state. It is the record
against which reversible actions (R0–R3, cmd-transaction) are audited.

# 7. Later Work

Persistence (write the chain to disk / SemFS), cryptographic signing of the head
hash with the Machine key, and export for the user. In scope for H1.4+, not this
RFC.
