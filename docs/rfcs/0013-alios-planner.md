# RFC-0013: alios — The User Agent (Planning Loop)

Version: 1.0
Status: Accepted
Category: Architecture (agent)
Author: Lead Architect
Depends on: RFC-0004, RFC-0011 (Kernel)
Implemented by: `agent/alios`

---

# 1. Summary

The agent layer. `alios` turns an `Intent` into an `ExecutionPlan` and runs it
through the kernel, closing the canonical loop end to end:
**Intent → Planning** → Permission → Execution → Verification → Result.

Planning sits behind a `Planner` trait. This crate ships a deterministic,
rule-based `RulePlanner` so the whole pipeline runs and is testable today without
any AI. A model-backed planner (calling Claude, as the prototype's
`anthropic-planner` does) implements the same trait and drops in unchanged.

# 2. Design

- `Planner` trait: `plan(&Intent) -> ExecutionPlan`.
- `RulePlanner`: reads simple keywords from the intent text and emits a matching
  filesystem plan. Its **fallback is always a read-only `list`** — never a
  destructive guess. This is a stand-in, not the product; the output shape is
  identical to what a model planner emits.
- `Agent<P: Planner> { name, planner }`: `plan_for(raw_request)` produces
  `(Intent, ExecutionPlan)`.

Reference: `prototype/kernel/mock-planner.ts`.

# 3. The Full Agent Loop

The tests include the first agent-driven end-to-end run: the agent plans a
rename, the kernel executes it under Alios supervision against the real
`cap-files` capability, the file changes on disk, and the ledger holds an intact
chain. The agent — not hand-written test code — drives the whole system.

# 4. Model-Backed Planning (later)

A real planner calls a model to turn natural language into a fully-specified
plan. It implements the same `Planner` trait, so nothing downstream changes. It
cannot run in CI (needs an API key / network / is non-deterministic), so it lives
behind the interface and is exercised on a developer machine — exactly how the
prototype's Anthropic planner works.

# 5. Testing

4 tests, all green, no warnings: list-intent produces a list step; vague intent
falls back to a safe read-only step; plan links back to its intent; and the
agent-plans-and-kernel-runs end-to-end rename on a real file.

# 6. Next

A model-backed `Planner` (Claude) on the developer machine; then the Machine
(RFC-0006) — auth, VM, the desktop the agent works inside — and Alios's
behavioral supervision (v2).
