# RFC-0020: cap-browser — The Browser Capability

Version: 1.0
Status: Accepted
Category: Capability
Author: Lead Architect
Depends on: RFC-0004, RFC-0008 (Transaction Engine)
Implemented by: `capabilities/browser`

---

# 1. Summary

Lets an agent act on the web — navigate, read, fill fields, submit forms —
implementing the `Resource` trait so browser steps run reversibly through the
engine. Most real digital work is on the web, so this is the highest-value
capability after the filesystem, and the one where risk classification matters
most.

# 2. Risk classification (the point)

- `navigate`, `read`, `screenshot` → **R0** (read-only).
- `fill`, `clear` → **R1** (fully reversible: clearing restores prior state; the
  engine can auto-undo).
- `submit`, `click_buy`, `confirm`, `pay` → **R3** (irreversible external effect;
  ALWAYS human-gated by policy — an agent can browse and stage input freely, but
  buying or submitting requires the user's approval).

This is where the R0–R3 model earns its keep: the agent works autonomously up to
the moment of an irreversible commitment, then stops for the human.

# 3. Design

- `BrowserBackend` trait is the impure surface (drive a real headless browser).
  Kept behind a trait so the capability's logic — parsing, risk, the reversible
  lifecycle — is fully tested with a fake backend; a real Chrome backend plugs in
  unchanged. (Real automation can't run in CI.)
- `Browser<B>` implements `Resource`: navigate verifies by URL; fill snapshots the
  field and undoes by clearing; R0 reads take no snapshot; R3 submits are not
  auto-reversible and are gated upstream.

# 4. Testing

7 tests, all green, no warnings: risk classification (incl. R3 submit not
autonomous), navigate execute+verify, fill reversible-by-clear, read is
read-only, submit executes but is R3, unknown action fails, and an end-to-end
fill-then-undo through the real TransactionEngine with an intact ledger.

# 5. Note found during implementation

`fill` was initially typed R2; the engine only auto-undoes R0/R1, and clearing a
field is a clean, complete undo — so `fill` is correctly R1. Reclassified.

# 6. Next

A real `BrowserBackend` (headless Chrome / CDP) behind this trait; then wiring the
capability into AIPC as tools so agents can call it.
