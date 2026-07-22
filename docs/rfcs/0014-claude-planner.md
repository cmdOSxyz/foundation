# RFC-0014: Model-Backed Planning (Claude)

Version: 1.0
Status: Accepted
Category: Architecture (agent)
Author: Lead Architect
Depends on: RFC-0013 (Agent / Planner trait)
Implemented by: `agent/alios` (module `claude_planner`)

---

# 1. Summary

Turns real natural language into an `ExecutionPlan` by asking Claude — the role
the prototype's `anthropic-planner` plays. It implements the same `Planner` trait
as the rule-based planner, so it drops into the agent unchanged.

# 2. Testability by Design

The impure network call is isolated behind a `ClaudeTransport` trait. Everything
else is pure and fully unit-tested with a fake transport:
- `parse_plan_response` — model JSON → plan, tolerant of ```json fences.
- Safe fallback — a null/empty/garbage reply degrades to a read-only `list`;
  the agent never invents a destructive action from a bad reply.
- `ClaudePlanner<T: ClaudeTransport>` — inject a fake transport to test the whole
  path with no network; inject a real HTTP transport in production.

This keeps the crate dependency-light (no HTTP client in the core) and CI-safe,
while leaving a clean seam for the real transport.

# 3. The Real Transport (developer machine)

A production `ClaudeTransport` performs the HTTPS POST to the Anthropic Messages
API (system prompt + user text → text reply). It needs an API key and network,
so it runs on a developer machine / in the app, not CI. Implementing the trait is
the only remaining step to live planning.

# 4. Safety

The `Planner` impl never fails: transport or parse errors degrade to the safe
read-only fallback plan. Combined with the kernel's policy gate, a bad or
malicious model reply cannot cause an unsafe action.

# 5. Testing

13 tests in `alios`, all green, no warnings: parser (well-formed, fences, null,
empty, garbage, intent-link, capability default), planner-over-fake-transport,
and degrade-on-transport-failure — plus the earlier rule-planner and end-to-end
tests.

# 6. Next

A real `ClaudeTransport` (HTTP) wired in the desktop app / a small service crate;
then the Machine (RFC-0006): auth, VM, the desktop the agent works inside.
