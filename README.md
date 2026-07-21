# cmdOS

### The Operating System for AI Agents

> **Hire your AI. Give it a computer. Choose the future it builds.**
> Give work to your AI employee in plain language. It does the work for real —
> shows every step, asks before anything risky, and everything it does can be undone.

---

## What is cmdOS?

cmdOS is an AI-native operating environment. You type an intent —
*"invoice my three clients for this week's hours"* — and **Alios**, the resident agent,
plans it, previews it, asks approval where it matters, executes, verifies, and records
everything in an auditable ledger. Mistakes are reversible.

The Horizon-1 product (**cmdOS Layer**) is a desktop app for Windows/macOS/Linux.
The long-term destination is a full AI-native OS — the Android playbook: Linux kernel
underneath, a fully self-developed userspace on top.
Strategy: `docs/01-vision/strategy-v2.md` · Roadmap: `ROADMAP.md`.

## Why it's different

- **Reversible by architecture** — every side effect runs as a transaction:
  simulate → snapshot → execute → verify → commit/rollback. Undo is a kernel property.
- **Risk-proportional trust (R0–R3)** — autonomy where mistakes are cheap to reverse,
  explicit approval where they aren't.
- **Enforced limits** — permissions, budgets, and payment mandates are checked *below*
  the agent, where prompt injection cannot reach.
- **Open by protocol** — MCP is the capability ABI; A2A 1.0 for agent identity and
  delegation. Compatible with the existing tool ecosystem on day one.
- **Yours** — local-first; the agent's identity is a signed card owned by the user.

## Repository at a glance

```
kernel/       Rust core: types · scheduler · transactions · policy · ledger
services/     semfs · nis (AI Router) · aipc (MCP/A2A) · cmdpay
agent/alios/  the Prime Agent
capabilities/ first-party MCP servers
shell/        cmdShell (Tauri)
schemas/      TypeScript contracts
prototype/    runnable reference implementation (Electron) + behavior contracts
docs/         specs & RFCs (docs/rfcs, docs/00-governance) · archive of superseded docs
```

## Quick start (prototype)

```bash
npm install
npm test      # 8 behavior contracts, all green
npm start     # launch the Electron reference app
```

## Development

Rust workspace: `cargo build --workspace`. Engineering rules live in `CLAUDE.md`.
Every component starts as an RFC (`docs/rfcs/0000-rfc-process.md`).

## License

MIT.

---

**cmdOS — The Operating System for AI Agents.**
