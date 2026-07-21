# CLAUDE.md — cmdOS Engineering Guide

cmdOS is **The Operating System for AI Agents**. A user creates their own agent
(named, with an avatar); it works inside a personal cloud computer (cmdOS Machine,
RFC-0006), logging into their tools. **Alios** is the system-level supervisor that
inspects, risk-scores (R0-R3), and governs every user agent — it never does the user's
tasks and can never be disabled by a user agent. User agents are untrusted by default;
R3 (irreversible) is always human-gated. Flagship tech: Shadow World Engine (RFC-0005),
at VM level. Canonical strategy: `docs/01-vision/strategy-v2.md`. Roadmap: `ROADMAP.md`.

Naming: **cmdOS** is the operating system; **Alios** is the agent living in it.
The Horizon-1 product is called **cmdOS Layer** (an app on the user's existing OS).

---

## Architecture Map

```
cmdos/
├── kernel/            Rust workspace — self-developed core
│   ├── cmd-types        Object model: Intent, Agent, Capability, Transaction, Mandate, Budget
│   ├── cmd-kernel       Intent Scheduler + semantic syscalls
│   ├── cmd-transaction  simulate → snapshot → execute → verify → commit/rollback
│   ├── cmd-policy       R0–R3 permissions, budget enforcement, mandate checks
│   └── cmd-ledger       Append-only signed audit ledger
├── services/          semfs · nis (AI Router) · aipc (MCP/A2A) · cmdpay
├── agent/alios/       Prime Agent (planning, memory, delegation)
├── capabilities/      First-party Rust MCP servers (files, browser, terminal, mail, calendar)
├── shell/             cmdShell (Tauri) — replaces the Electron prototype shell later
├── schemas/           TypeScript contracts (shared with shell; mirror of cmd-types)
├── prototype/         ⭐ REFERENCE IMPLEMENTATION (TypeScript/Electron) — runnable
│   ├── kernel/          executor, event-log, permission-gate (behavior references)
│   ├── capabilities/    filesystem.ts — dry-run/verify/undo reference for cmd-transaction
│   ├── apps/desktop/    Electron shell (npm start)
│   └── tests/           BEHAVIOR CONTRACTS — Rust ports must pass equivalent suites
└── docs/              Specs. RFCs: docs/rfcs/ (technical) + docs/00-governance/ (governance)
```

## Build & Test

```bash
cargo build --workspace          # Rust skeleton (must stay green)
cargo test  --workspace
npm install && npm test          # prototype behavior contracts (8 tests)
npm start                        # run the Electron prototype
```

## Hard Rules

1. **RFC-first.** No component code without an Accepted RFC. Numbering is shared across
   `docs/rfcs/` and `docs/00-governance/` and continues from RFC-0003 (next: RFC-0004).
2. **Strangler fig.** Rust components replace prototype parts only after passing the
   corresponding behavior contract in `prototype/tests/`. The prototype is never broken.
3. **All side effects go through transactions** (dry-run available, snapshot taken,
   verify after, undo path registered). No direct writes from agent code.
4. **All tool access goes through AIPC (MCP).** No ad-hoc integrations.
5. **All spending goes through cmdPay mandates.** No payment-capable keys in agent scope.
6. **Capability follows safety**: an action class ships only after the machinery
   controlling it exists. cmdpay is built LAST in Horizon 1.
7. Languages: Rust (kernel/services/capabilities), TypeScript (schemas/shell/prototype).
8. Superseded docs go to `docs/archive/` with a pointer header — never deleted.
9. **Alios is the supervisor** (cmdOS-owned), runs in the control plane, and can never
   be disabled by a user agent. User agents are untrusted by default. R3 is always
   human-gated, even inside a fully-authorized Machine.

## UI Direction

Dark terminal-inspired (#0a0a0f base), monospace, execution timeline always visible,
approval requests explicit, "AI is working for me" — never a chat clone.
