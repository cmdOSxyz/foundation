# CLAUDE.md — cmdOS Engineering Guide

**Positioning is undecided as of 2026-08-04.** The "operating system for AI agents"
framing is retired — see `docs/archive/README.md` for what was retired and why. The
agent-facing components (services, capabilities, the Alios agent, the shell, the intent
scheduler) were removed with it; recover any of them from the `pre-cmdcapital-trim` tag.

What remains is a safety and evidence core that does not depend on any positioning:
transactions with rollback, risk classification with budgets, a signed audit ledger,
approval binding, and proof bundles.

Do not add a product claim to any surface until a direction is chosen.

---

## Architecture Map

```
foundation/
├── kernel/            Rust workspace
│   ├── cmd-types        Object model: Intent, Agent, Capability, Transaction, Mandate, Budget
│   ├── cmd-transaction  simulate → snapshot → execute → verify → commit/rollback
│   ├── cmd-policy       R0–R3 permissions, budget enforcement, mandate checks
│   ├── cmd-ledger       Append-only signed audit ledger
│   ├── cmd-approval     Approval binding (RFC-0023)
│   ├── cmd-proof        Proof Bundle v0 (RFC-0025)
│   └── cmd-shadow       Fork state, run alternatives to completion, choose one (RFC-0005)
├── schemas/           TypeScript contracts — mirror of cmd-types
├── prototype/         ⭐ REFERENCE IMPLEMENTATION (TypeScript/Electron) — runnable
│   └── tests/           BEHAVIOR CONTRACTS — Rust ports must pass equivalent suites
├── sandbox/           Test fixtures
├── tools/ci/          check-docs.sh
└── docs/              Specs. RFCs: docs/rfcs/ + docs/00-governance/. Retired: docs/archive/
```

## Build & Test

```bash
cargo build --workspace          # must stay green
cargo test  --workspace          # 29 tests
bash tools/ci/check-docs.sh      # documentation invariants
npm install && npm test          # prototype behavior contracts
```

## Hard Rules

1. **RFC-first.** No component code without an Accepted RFC. Numbering is shared across
   `docs/rfcs/` and `docs/00-governance/`.
2. **Strangler fig.** Rust components replace prototype parts only after passing the
   corresponding behavior contract in `prototype/tests/`. The prototype is never broken.
3. **All side effects go through transactions** (dry-run available, snapshot taken,
   verify after, undo path registered).
4. **Capability follows safety**: an action class ships only after the machinery
   controlling it exists. Payments are built last.
5. **R3 (irreversible) is always human-gated.** Bridging, settlement and any transfer of
   principal are R3 and never inherit a lower class from the action that motivated them.
6. Languages: Rust (kernel), TypeScript (schemas/prototype).
7. Superseded docs go to `docs/archive/` with a pointer header — never deleted.

## Removed on 2026-08-04

`kernel/cmd-kernel`, `services/*`, `agent/alios`, `capabilities/*`, `cli`, `shell/` —
5,847 lines tied to the agent-OS direction. All recoverable:

```bash
git checkout pre-cmdcapital-trim -- <path>
```
