# RFC-0010: cmd-policy — Alios's Supervision Engine

Version: 1.0
Status: Accepted
Category: Architecture (core / supervision)
Author: Lead Architect
Depends on: RFC-0004 (Object Model), RFC-0006 (Machine & Supervision)
Implemented by: `kernel/cmd-policy`

---

# 1. Summary

The decision core of **Alios**, the supervisor. For every action a user agent
proposes, `cmd-policy` scores risk (R0–R3), checks the action against the agent's
mandate, checks any spend against the budget, and returns a `Decision`:
`Allow`, `NeedsApproval`, or `Block`. Where `cmd-transaction` decides *how to
undo*, `cmd-policy` decides *whether it may run*.

# 2. Load-Bearing Rules

Enforced here, below agent logic, so no prompt injection or agent code can bypass
them:
- **R3 is never autonomous** — always `NeedsApproval`, even if a mandate names R3.
- Actions beyond R0 require an **active mandate** covering the capability;
  absence of authority means no (untrusted by default).
- Action risk above the mandate's autonomous limit → `NeedsApproval`.
- Any spend must fit the **budget**; a spend with no budget is **blocked** — a
  prompt-injected "send money" cannot be signed.
- Expired/revoked mandates and expired budgets block.

# 3. Design

`ProposedAction { capability, risk, spend }` is the minimal fact set for a
decision. `PolicyEngine::evaluate(action, mandate, budget)` is a pure function of
the action and the authority presented, at an evaluation time `at` (for
expiry checks). `Decision::is_autonomous()` tells the runtime whether it may
proceed without a human.

The engine is stateless and side-effect free: it only judges. Recording the
judgment is the ledger's job (cmd-ledger); acting on it is the kernel's.

# 4. Testing

10 unit tests, all green: read-only always allowed; R3 always needs approval even
with a mandate; beyond-R0 blocked without mandate; blocked when mandate lacks the
capability; revoked mandate blocks; risk-above-limit needs approval; within-
mandate allowed; spend-within-budget allowed; spend-over-budget blocked; spend-
without-budget blocked. No warnings.

# 5. Reversibility Impact

None directly. cmd-policy gates actions before they run; reversibility of the
actions it allows is handled by cmd-transaction.

# 6. Next

`cmd-kernel` (Intent Scheduler) ties object model, policy, transactions, and
ledger together to run a plan end-to-end — completing the kernel.
