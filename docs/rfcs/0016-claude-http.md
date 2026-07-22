# RFC-0016: claude-http — Real ClaudeTransport (HTTP)

Version: 1.0
Status: Accepted
Category: Services
Author: Lead Architect
Depends on: RFC-0014 (Model-backed planning)
Implemented by: `services/claude-http`

---

# 1. Summary

A real `ClaudeTransport` (the trait from `alios`) that performs an HTTPS POST to
the Anthropic Messages API. This is what makes natural-language planning *live*:
`ClaudePlanner::new(HttpTransport::new(key))` turns a user's words into a plan.

# 2. Why a separate crate

The HTTP dependency (reqwest + TLS) is heavy and pulls a large dependency tree.
Isolating it in `services/claude-http` keeps the agent core (`alios`, kernel)
dependency-light and unaffected: the core builds and tests without ever compiling
an HTTP client. Only programs that actually want live planning depend on this
crate.

# 3. Design

`HttpTransport { api_key, model, max_tokens }` implements `ClaudeTransport::send`:
POST to `/v1/messages` with `x-api-key` and `anthropic-version` headers, parse the
`content[].text` blocks, and return the concatenated text. Non-2xx responses and
empty bodies become `PlanError`, which the planner degrades to a safe fallback.

# 4. Build/Test Note (honest status)

This crate needs network + an API key, so it is **not exercised in CI** and was
not compiled in the author's sandbox (an old toolchain there cannot build
reqwest's dependency tree). It requires a recent Rust (1.80+). The planner logic
it feeds is already fully tested in `alios` with a fake transport; this crate adds
only the wire call. First real validation happens on a developer machine with
`ANTHROPIC_API_KEY` set.

# 5. Next

Wire `HttpTransport` into the `cmdos` CLI behind a flag so the CLI plans from
natural language when a key is present; then the Machine (RFC-0006).
