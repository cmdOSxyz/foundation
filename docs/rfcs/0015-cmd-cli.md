# RFC-0015: cmd-cli — The Runtime CLI

Version: 1.0
Status: Accepted
Category: Runtime
Author: Lead Architect
Depends on: RFC-0013 (Agent), RFC-0011 (Kernel), RFC-0009 (cap-files)
Implemented by: `cli` (binary `cmdos`)

---

# 1. Summary

The first *runnable* cmdOS. `cmd-cli` wires the whole agent stack into one
terminal program: give it a request, the agent (Alios) plans it, the kernel runs
the plan under supervision against real files, and it prints the execution
timeline and the audit ledger.

It uses the deterministic `RulePlanner`, so it runs with no API key — proving the
end-to-end pipeline as a real program, not just tests. Swapping in the
model-backed planner (RFC-0014) is a one-line change once a real `ClaudeTransport`
is wired.

# 2. Usage

```
cmdos "list files in ."
cmdos --dir /path/to/workspace "rename draft.txt to final.txt"
```

# 3. What it demonstrates

A single command exercises the full stack: Intent → Planning (RulePlanner) →
Permission (cmd-policy grants a filesystem mandate) → Execution (cmd-transaction
via cap-files) → Verification → Result, with the ledger printed and its hash
chain verified. This is the vertical slice as a program you can run.

# 4. Safety

Every step is policy-gated and every executed step is reversible and recorded —
the same guarantees the library crates enforce. When the planner cannot resolve a
concrete safe action (e.g. a rename with no path), the capability refuses rather
than guessing.

# 5. Testing

4 unit tests (arg parsing, dir flag, empty-request error, risk table), all green,
no warnings. Manual run verified end-to-end: a `list` completes with an intact
4-entry ledger.

# 6. Next

Wire a real `ClaudeTransport` so the CLI plans from natural language live; then
the Machine (RFC-0006) turns this runtime into the product (auth, VM, desktop).
