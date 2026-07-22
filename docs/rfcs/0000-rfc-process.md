# RFC-0000: The cmdOS RFC Process

Status: Accepted
Category: Governance

Every cmdOS component begins as an RFC. Technical RFCs live in this directory;
governance RFCs live in `docs/00-governance/` (RFC-0001 Documentation Reconciliation
and RFC-0003 Open-Core Governance already exist there — numbering is shared and
continues across both locations, never restarted).

Lifecycle: Draft -> Review -> Accepted -> Implemented -> (Superseded)

Rules:
1. No component code merges without an Accepted RFC.
2. RFC numbers are sequential across the whole project and never reused.
3. Superseded RFCs are marked, never deleted.
4. Each technical RFC states: motivation, design, interfaces, security model,
   reversibility class impact (R0-R3), and testing strategy.
5. The prototype under `prototype/` is admissible evidence in any RFC: cite its
   behavior as the reference specification where applicable.

Planned technical RFCs (continuing the existing sequence):
- 0004 cmdKernel object model — ACCEPTED, implemented in kernel/cmd-types (9 tests green)
- 0005 Shadow World Engine (accepted-draft; seed: prototype filesystem dry-run/verify/undo)
- 0006 cmdOS Machine & Agent Supervision (accepted-draft)
- 0005 Transaction Engine (seed: `prototype/capabilities/filesystem.ts` dry-run/verify/undo)
- 0006 Permission & Degradation Model (seed: `prototype/kernel/permission-gate.ts`; adds R0-R3, scopes, graceful degradation)
- 0007 cmd-ledger — ACCEPTED (6 tests green)
- 0008 cmd-transaction — ACCEPTED, reversible engine in kernel/cmd-transaction (7 tests green)
- 0009 cap-files — ACCEPTED, reversible filesystem capability (10 tests, 8 contracts + 2 e2e)
- 0010 cmd-policy — ACCEPTED, Alios supervision engine (10 tests green)
- 0011 cmd-kernel — ACCEPTED, Intent Scheduler + first vertical slice (4 tests green)
- 0012 aipc — ACCEPTED, MCP-style tool registry + permission-gated router (7 tests green)
- 0008 cmdPay mandates and budget enforcement
- 0009 SemFS
- 0010 NIS inference service
- 0013 alios — ACCEPTED, user agent planning loop + first agent-driven vertical slice (4 tests green)
- 0014 Model-backed planning (Claude) — ACCEPTED, transport-trait planner in agent/alios (13 tests green)
- 0015 cmd-cli — ACCEPTED, first runnable cmdOS runtime (binary cmdos, 4 tests green)
- 0016 claude-http — ACCEPTED, real ClaudeTransport (not CI-tested; needs API key + recent Rust)
