# RFC-0005: Shadow World Engine

Version: 1.0
Status: Draft
Category: Architecture (Flagship)
Author: Lead Architect
Depends on: RFC-0004 (Object Model)

---

# 1. Summary

Before touching the real environment, cmdOS forks it copy-on-write into a "shadow world."
The user's agent executes whole plans inside the fork. The user is shown finished
outcomes and promotes the one they want; discard is free.

Reversibility, dry-run, and undo are special cases of shadowing. At VM level (RFC-0006)
a shadow spans the entire computer — files, browser sessions, open apps.

# 2. Motivation

Every agent product today is propose-then-execute: the user approves a *plan* (text) and
only sees results after they happen. Shadowing inverts this: the user chooses among
*finished results*. This is the deepest change to human–agent interaction cmdOS makes,
and the kernel-first architecture (transactions, snapshots) makes it natural rather than
bolted-on.

# 3. Mechanism

- **Fork**: place a copy-on-write overlay over the real state (instant; no data copied).
- **Execute**: the agent runs normally; all writes are redirected into the shadow layer.
  The agent is unaware it is shadowed — capability backends are swapped beneath it.
- **Promote**: replay the shadow's change set atomically onto real state (snapshot taken,
  so still undoable afterward).
- **Discard**: drop the shadow layer at ~zero cost.

Multi-future: because forking is cheap, run 2–5 shadows in parallel (one strategy each)
and let the user pick.

# 4. Reversibility Classes

Shadowing coverage maps onto the R0–R3 model:
- R0/R1 — perfect shadow (local files, data).
- R2 — simulated shadow (external API/mail responses predicted from known data; the user
  sees the exact thing that will be sent).
- R3 — not shadowable (real external send/payment); the shadow ends at "here is exactly
  what will happen" and the real action stays human-gated.

# 5. Reference in the Prototype

`prototype/capabilities/filesystem.ts` already implements `dryRunFilesystemStep`
(shadow depth 1), `runFilesystemStep`, `verifyFilesystemStep`, and
`undoForFilesystemStep`. The behavior contracts in `prototype/tests/` are the
specification the Rust `cmd-transaction` shadow engine must satisfy.

# 6. Build Path

- v0.5: single shadow over user directories; before/after diff; Promote/Discard.
- v1: 2–3 parallel shadows; "choose your future" picker.
- v2: simulated shadow for API/mail/calendar with R2 preview.
- v3: sinks into `cmd-transaction` as an OS primitive; VM-level shadow (RFC-0006) so
  every capability is shadowed for free.
