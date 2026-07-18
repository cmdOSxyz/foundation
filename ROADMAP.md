# ROADMAP

> The AI Execution Operating System

The canonical product roadmap is defined in `docs/09-roadmap` as five **Stages**.
This file exists only to map engineering **workstreams** ("Phases") onto those Stages.
Phases are workstream identifiers, not a separate timeline.

---

## Stages (canonical — see `docs/09-roadmap`)

| Stage | Name | Outcome |
|-------|------|---------|
| 0 | Foundation | Architecture, standards, repository, initial AI infrastructure |
| 1 | MVP | First end-to-end execution loop on the desktop |
| 2 | Agent Platform | SDK, plugins, marketplace, developer ecosystem |
| 3 | Execution OS | Desktop/mobile agents, cross-platform execution |
| 4 | AI-Native OS | Autonomous agents, enterprise, ecosystem |

---

## Phase → Stage mapping

| Engineering workstream (Phase) | Delivers into Stage |
|-------------------------------|---------------------|
| P0 Foundation | Stage 0 Foundation |
| P1 Kernel | Stage 1 MVP |
| P2 Execution Engine | Stage 1 MVP |
| P3 Runtime | Stage 1 MVP → Stage 3 Execution OS |
| P4 Capability System | Stage 1 MVP → Stage 2 Agent Platform |
| P5 AI Integration | Stage 1 MVP |
| P6 Desktop Agent | Stage 3 Execution OS |
| P7 Distributed cmdOS | Stage 3 Execution OS → Stage 4 AI-Native OS |
| P8 SDK | Stage 2 Agent Platform |
| P9 Enterprise | Stage 4 AI-Native OS |

The "Phase" label is a workstream identifier only. It never appears as a standalone
timeline.

---

## Stage 1 (MVP) component build order

The MVP is sequenced desktop-first (see `CLAUDE.md`):

1. Desktop Application Shell
2. Control Center
3. Agent Runtime
4. Execution Engine
5. Capability System
6. AI Router
7. Memory System

---

## Success Criteria

- Deterministic execution
- Secure by default
- Observable operations
- Provider independent
- AI model agnostic
- Extensible capability platform
