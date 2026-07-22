# RFC-0018: cmd-router — Bring-Your-Own-Key API Router

Version: 1.0
Status: Accepted
Category: Services
Author: Lead Architect
Depends on: RFC-0004
Implemented by: `services/cmd-router`

---

# 1. Summary

cmdOS is a bring-your-own-key tool. The user adds their own API keys; `cmd-router`
manages them: it detects each key's provider, meters usage against a per-key
limit, rotates round-robin across keys with headroom, and moves to the next key
automatically when one is exhausted. When every key is spent it returns
`AllExhausted`, and the app warns the user to add or replace a key. The user then
supplies new keys of their own.

# 2. Positioning (important)

cmdOS neither owns, provides, nor sells API keys. It forwards the user's requests
to the keys the user supplied — like any API client or browser. The user is
responsible for using their keys within their providers' terms; the decision to
add and rotate keys is the user's. cmd-router is a management and transparency
layer over keys the user already holds, not a means of evading provider limits.
Product copy should frame it this way: efficient, transparent management of the
user's own keys — not "bypass free-tier limits."

# 3. Design

- `Provider::detect` recognizes Anthropic (`sk-ant-`), OpenAI (`sk-`), Google
  (`AIza`), else `Other`.
- `ApiKey` holds the secret (used only by the transport), a display label, a
  `request_limit`, and `requests_used`. `masked()` is what the UI shows; the raw
  secret is never displayed.
- `KeyRouter::next_key()` scans round-robin from a cursor for the next key with
  headroom, meters one request, advances the cursor, and returns `Use(id)`,
  `AllExhausted`, or `Empty`.
- `stats()` exposes per-key `{ provider, label, masked, used, limit, remaining,
  active }` for the app's key dashboard.
- `add` / `remove` let the user manage the pool.

Metering is by request count in this RFC (simple, deterministic, testable).
Token/cost metering — which needs usage figures parsed from each provider's
responses — is future work once real transports report them.

# 4. Integration

The router selects a key; the transport (e.g. claude-http) uses `secret_of(id)`
to send the actual request. The planner path is unchanged: router picks the key,
transport sends, planner parses.

# 5. Testing

8 tests, all green, no warnings: provider detection, secret masking, metering +
remaining, round-robin spread, move-to-next-on-exhaustion, warn-when-all-
exhausted, empty pool, and add/remove.

# 6. Next

Surface `stats()` in the app's key dashboard; wire the router into the planner's
transport selection; later, token/cost metering from provider usage fields.
