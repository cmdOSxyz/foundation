# cmdOS

<p align="center">
  <h3 align="center">The AI Execution Operating System</h3>
  <p align="center">Turn Intent Into Execution.</p>
</p>

---

## Vision

cmdOS is a deterministic execution operating system for AI. Instead of only generating
text, it turns human intent into secure, observable, deterministic execution across
applications, devices, cloud infrastructure, and autonomous agents.

cmdOS is a standalone **desktop** application — not a chatbot, web app, DApp, or SaaS
dashboard. The experience is *"an AI system operating my computer,"* not *"chatting with
an assistant."*

---

## Core Principles

- Intent → Command → Execution Plan → Execution
- Deterministic by design
- Security first; authorization precedes every action
- Human authority is never removed
- Event-driven and observable
- Capability-based permissions
- Provider-agnostic and AI-model-agnostic
- Local-first execution

---

## Architecture

```text
                User Intent
                     │
                     ▼
             Interaction Layer          (Control Center + command input)
                     │
                     ▼
            Intelligence Layer           (AI cognition, reasoning, AI Router)
                     │
                     ▼
                Agent Layer              (autonomous workers)
                     │
                     ▼
             Capability Layer            (executable abilities)
                     │
                     ▼
           Communication Layer           (messages, events, coordination)
                     │
                     ▼
              Runtime Layer              (execution environment)
                     │
   ┌─────────────────┴─────────────────┐
   │        Security (cross-cutting)    │  identity · permission · policy
   │                                    │  isolation · monitoring · audit
   └─────────────────┬─────────────────┘
                     ▼
────────────────────────────────────────
                 cmdOS Kernel
────────────────────────────────────────
Object · State · Command · Validation · Cognition · Execution Planner
Execution Engine · Transactions · Scheduler · Agent Orchestrator
Memory · Knowledge · Security · Events · Recovery · Observability
────────────────────────────────────────
                     │
                     ▼
        Applications · Devices · APIs · Cloud
```

The Kernel is the single deterministic execution authority — no execution bypasses it.
Agents invoke Capabilities; they never mutate canonical state directly.

---

## Repository Structure

```text
cmdOS/
├── docs/            # Specification (RFCs). Single source of truth for architecture.
├── apps/            # desktop/ (primary), mobile/, cli/, web/ (non-primary)
├── kernel/          # core/ scheduler/ memory/ security/ resources/ interfaces/
├── runtime/         # execution/ workflow/ scheduler/ process/ monitoring/
├── ai/              # router/ models/ reasoning/ context/ memory/ inference/
├── agents/          # core/ lifecycle/ planning/ memory/ communication/ templates/
├── capabilities/    # core/ registry/ built-in/ external/ examples/
├── plugins/         # loader/ manifest/ signing/ validation/ registry/
├── providers/       # AI provider adapters + external service integrations
├── sdk/             # agent-sdk/ capability-sdk/ plugin-sdk/ api/
├── services/        # identity/ sync/ marketplace/ billing/ analytics/
├── schemas/         # canonical data schemas / IDL (interface contracts)
├── infrastructure/  # cloud/ deployment/ containers/ monitoring/ automation/
├── tools/           # build/ generators/ debugging/ scripts/
└── tests/           # unit/ integration/ system/ security/ performance/
```

---

## Capability and Plugin

A **Capability** is the core execution primitive — a versioned interface contract plus an
implementation, held in the Capability Registry, and the only thing an Agent invokes at
runtime.

A **Plugin** is a signed, versioned distribution package. On install, after security
validation, it registers its Capabilities (and optionally Agents) into the Registry.
There is one **Marketplace**, and it distributes Plugins.

---

## Roadmap

The canonical roadmap lives in `docs/09-roadmap`, organized as five product Stages.
Engineering workstreams ("Phases") map into these Stages — see `ROADMAP.md`.

- **Stage 0 — Foundation** — architecture, standards, repository, initial AI infrastructure
- **Stage 1 — MVP** — first end-to-end execution loop
- **Stage 2 — Agent Platform** — SDK, plugins, marketplace, developer ecosystem
- **Stage 3 — Execution OS** — desktop/mobile agents, cross-platform execution
- **Stage 4 — AI-Native OS** — autonomous agents, enterprise, ecosystem

---

## Design Philosophy

```text
Intent → Command → Execution Plan → Runtime → Verification → Observed Result
```

Documentation first. RFC before implementation. Every feature contributes to the AI
Execution Operating System.

---

## Documentation

The complete architecture is documented as RFC specifications under `docs/`. Start with
`docs/01-vision`, then `docs/05-architecture`. Contribution rules are in `CONTRIBUTING.md`;
security policy is in `SECURITY.md`.

---

## License

MIT License.

---

**cmdOS — The AI Execution Operating System.**
