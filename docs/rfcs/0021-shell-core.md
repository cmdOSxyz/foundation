# RFC-0021: shell-core — The Bridge Between cmdShell and the Core

Version: 1.0
Status: Accepted
Category: Services
Author: Lead Architect
Depends on: RFC-0011 (Kernel), RFC-0012 (AIPC), RFC-0013 (Agent), RFC-0018 (Router)
Implemented by: `services/shell-core`

---

# 1. Summary

cmdShell needs one thing from the system: plain, serializable answers.
`shell-core` provides them. It owns a running `Machine` — agent, tool registry,
key router, filesystem capability, ledger — and exposes the small set of
operations the interface actually performs, each returning data that serializes
straight to the front end.

# 2. Why a separate crate

The Tauri layer cannot be tested in CI: GUI toolchains are not available there.
Putting the logic in a plain library crate keeps it under test, and leaves the
Tauri commands as a thin marshalling layer. The same core can then drive a CLI, a
daemon, or a different shell.

Every `// TAURI` hook in the cmdShell React code maps to one method here.

# 3. API

- `Machine::new(agent)` — boots with the first-party tool catalog and a mandate
  whose autonomous ceiling is R1; anything riskier stops for the human.
- `submit_intent(text) -> IntentReport` — the whole loop: plan, gate, execute,
  verify. Reports every planned step with its risk and outcome
  (`executed` / `awaiting_approval` / `blocked` / `failed` / `not_reached`), plus
  ledger length and chain integrity. Gated steps are reported as pending
  decisions, not failures.
- `tools() -> Vec<ToolInfo>` — the tool surface, each marked `autonomous` against
  the mandate ceiling, so the UI can show which actions will stop for approval.
- `add_key` / `key_stats` / `remove_key` / `keys_exhausted` — the BYOK pool.
  Secrets are never returned; only masked forms.
- `ledger_rows()` / `status()` — the audit view and the status card.

Risk for each step is read from the AIPC tool catalog, making that catalog the
single source of truth; unknown actions default to R2 rather than being assumed
safe.

# 4. Bug found and fixed

Wiring the shell surfaced a real defect in `RulePlanner`: it matched keywords on
a lowercased copy of the request but also extracted the *path* from that copy,
so `list files in /Users/Admin/Docs` planned a step against
`/users/admin/docs` — broken on any case-sensitive filesystem. Parameters are now
read from the original text while keyword and marker matching stay
case-insensitive. Two regression tests added in `alios`.

# 5. Testing

8 tests in shell-core, all green, no warnings: boot state, autonomy marking
(navigate autonomous, click_buy never), intent run with ledger recording, safe
fallback for a vague intent, key add/meter/remove, exhaustion reporting, ledger
rows, and JSON round-trip of the report (the UI contract).

Workspace total after this change: 119 tests.

# 5b. Browsing (added)

`list_dir(path) -> DirListing` and `home_dir()` back the Files app. These take no
mandate and write no ledger entry, deliberately: the *user* looking at a folder
is not the *agent* acting on one. Everything the agent does to files still goes
through the capability and the kernel. Entries sort folders-first then
case-insensitively by name; unreadable entries are skipped rather than failing
the whole listing.

# 6. Next

Stream Alios verdicts and execution events to the UI instead of returning them in
one report; wire the remaining apps (Files, Ledger, Shadow) to the core; then the
Machine (RFC-0006) — the per-user VM the agent works inside.
