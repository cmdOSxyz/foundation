# RFC-0011: cmd-kernel — The Intent Scheduler

Version: 1.0
Status: Accepted
Category: Architecture (core)
Author: Lead Architect
Depends on: RFC-0004, RFC-0008 (Transaction), RFC-0010 (Policy), RFC-0007 (Ledger)
Implemented by: `kernel/cmd-kernel`

---

# 1. Summary

The crate that makes cmdOS a system rather than a set of parts. `cmd-kernel` runs
an `ExecutionPlan` to completion by walking its steps in dependency order and,
for each: asking cmd-policy whether it may run, executing allowed steps through
cmd-transaction (which records to cmd-ledger), and stopping at any step that needs
approval, is blocked, or fails. It never runs a gated action autonomously and
never proceeds past an unmet dependency.

This is the canonical loop in code:
Intent → Planning → **Permission → Execution → Verification** → Result.

# 2. Design

- `Kernel::run_plan(plan, resource, authority, risk_of)` drives the loop.
- Dependency order via `ExecutionPlan::ready_steps` (RFC-0004).
- `AuthorityContext { mandate, budget }` carries the acting agent's authority.
- `RiskResolver` classifies each step's risk; in the full system this comes from
  the capability contract, taken as a closure so the kernel stays decoupled from
  any capability registry.
- Per-step `StepOutcome`: Executed / AwaitingApproval / Blocked / Failed.
  Processing halts at the first non-executed outcome.

# 3. The Vertical Slice

The kernel's tests include the project's first end-to-end slice: a real rename
intent runs through the whole stack against the real `cap-files` filesystem
capability — policy gate → transaction engine → ledger — completing successfully,
renaming the file on disk, and leaving an intact, verifiable ledger chain. This
proves the five kernel/capability crates compose into a working system.

# 4. Testing

4 tests, all green, no warnings: dependency-order execution; stop-at-approval
(R3); block-outside-mandate; and the end-to-end vertical slice on a real file.

# 5. Reversibility & Safety

The kernel enforces the architecture's guarantees at the seam: nothing runs
without a policy Allow; R3 and out-of-scope actions never run autonomously;
every executed step is reversible (via cmd-transaction) and recorded (cmd-ledger).

# 6. Status: Kernel Complete

With cmd-kernel, the Horizon-1 kernel is functionally complete:
cmd-types, cmd-ledger, cmd-transaction, cmd-policy, cmd-kernel — plus the first
capability, cap-files. Next: AIPC (expose capabilities over MCP), then the Machine
(RFC-0006) and the agent/planning layer.
