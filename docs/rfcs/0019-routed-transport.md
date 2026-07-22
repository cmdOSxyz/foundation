# RFC-0019: RoutedTransport — Planner over the Key Router

Version: 1.0
Status: Accepted
Category: Architecture (agent)
Author: Lead Architect
Depends on: RFC-0014 (Model planning), RFC-0018 (Key router)
Implemented by: `agent/alios` (module `routed_transport`)

---

# 1. Summary

Bridges the BYOK key router (RFC-0018) to model planning (RFC-0014).
`RoutedTransport` is a `ClaudeTransport` that, on each call, asks the `KeyRouter`
for the next usable key (round-robin, per-key limits), builds a per-key transport,
and delegates the real call. When every key is exhausted it returns a clear error
so the agent warns the user to add or replace a key — it never stops silently.

# 2. Design

- `RoutedTransport::new(router, make_transport)` — `make_transport(secret)` builds
  a real transport for the chosen key. The HTTP client (claude-http) stays out of
  this crate, so the routing path is testable with a fake.
- `send()` calls `router.next_key()`:
  - `Use(id)` → fetch the secret, build the inner transport, delegate.
  - `AllExhausted` → transport error: "add or replace a key to continue".
  - `Empty` → transport error: "no API keys added".
- `stats()` / `total_remaining()` expose the pool state for the app dashboard.

Because `ClaudeTransport::send` takes `&self` but key selection mutates the meter,
the router is held in a `RefCell`.

# 3. Result: the full BYOK planning path

User keys → router selection (round-robin, metered) → per-key transport → Claude →
plan. Combined with the planner's safe fallback, an exhausted pool or an
unreachable model degrades safely rather than producing an unsafe action.

# 4. Testing

4 module tests (17 in alios total), all green, no warnings: routes-and-meters,
round-robin rotation, clear error when exhausted, clear error when empty.

# 5. Next

A production `make_transport` returning claude-http keyed to each secret; surface
`stats()` in the app UI; token/cost metering later.
