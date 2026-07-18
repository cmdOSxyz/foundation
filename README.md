# cmdOS

<p align="center">
  <h3 align="center">The AI Execution Operating System</h3>
  <p align="center">Turn Intent Into Execution.</p>
</p>

---

## Vision

cmdOS is building a deterministic execution operating system for AI.

Instead of simply generating text, cmdOS transforms human intent into secure, observable and deterministic execution across applications, devices, cloud infrastructure and autonomous agents.

---

## Core Principles

- Intent → Command → Execution Plan → Execution
- Deterministic by design
- Security first
- Event-driven architecture
- Observable execution
- Capability-based permissions
- Provider agnostic
- AI model agnostic

---

# Architecture

```text
                User Intent
                     │
                     ▼
           Interaction Layer
                     │
                     ▼
              AI Cognition
                     │
                     ▼
          Intent Interpretation
                     │
                     ▼
           Semantic Validation
                     │
                     ▼
             Kernel Intent API
                     │
────────────────────────────────────────
                 cmdOS Kernel
────────────────────────────────────────
Object Manager
State Manager
Command Manager
Execution Planner
Transaction Manager
Consistency Manager
Lock Manager
Memory Manager
Cache Manager
Execution Engine
Scheduler
Runtime
Persistence
Recovery
Security
Observability
────────────────────────────────────────
                     │
                     ▼
          Capability System
                     │
                     ▼
Applications • Devices • APIs • Cloud
```

---

# Repository Structure

```text
cmdOS/
├── docs/            # Specification (RFCs). Single source of truth for architecture.
├── apps/            # User-facing applications
│   ├── desktop/     #   Primary product (Windows / macOS / Linux)
│   ├── mobile/      #   Companion product
│   └── cli/         #   Command-line surface
├── kernel/          # Kernel core (objects, state, commands, planner, engine, ...)
├── runtime/         # Runtime environments (desktop, browser, containers)
├── capabilities/    # Capability system + first-party capabilities
├── providers/       # AI provider adapters + external service integrations
├── sdk/             # agent-sdk / capability-sdk / plugin-sdk / api
├── services/        # Supporting cloud services: identity, sync, marketplace, billing
├── schemas/         # Canonical data schemas / IDL (interface contracts)
├── infrastructure/  # Deployment, containers, monitoring, automation
├── tools/           # Build, generators, debugging, scripts
└── tests/           # unit / integration / system / security / performance
```

---

# Documentation

The complete architecture is documented as RFC specifications under `docs/`.

---

# Long-Term Roadmap

The canonical roadmap lives in `docs/09-roadmap`. It is organized as five product
Stages (0–4). Engineering workstreams ("Phases") map into these Stages — see the
mapping table in `ROADMAP.md`.

- Stage 0 — Foundation
- Stage 1 — MVP (first end-to-end execution loop)
- Stage 2 — Agent Platform (SDK, plugins, marketplace)
- Stage 3 — Execution OS (desktop/mobile agents, cross-platform execution)
- Stage 4 — AI-Native OS (autonomous agents, enterprise, ecosystem)

---

# Design Philosophy

Every action follows:

```text
Intent
  ↓
Command
  ↓
Execution Plan
  ↓
Runtime
  ↓
Verification
  ↓
Observed Result
```

---

# License

MIT License

---

# cmdOS

**The AI Execution Operating System**
