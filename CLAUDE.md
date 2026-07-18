# CLAUDE.md

Working agreement for anyone — human or AI — building cmdOS. This file is authoritative
for product identity, architecture rules, and engineering conventions. When it conflicts
with an older document, this file and the RFCs under `docs/` win.

---

## Project

**cmdOS — The AI Execution Operating System.**

## Mission

Build a deterministic AI execution operating system. cmdOS turns human intent into
real-world execution through AI Agents.

---

## Product Identity

cmdOS is a standalone, AI-native **desktop** operating system application. The primary
product is the **cmdOS Desktop Application**.

cmdOS is an execution environment where users delegate work to AI Agents. It is **not**:

- a chatbot or conversational assistant
- a web application or browser-based AI wrapper
- a DApp, Web3 app, or blockchain interface
- a SaaS dashboard or collection of admin panels

The user should feel: *"I have an AI system operating my computer,"* never *"I am
chatting with an assistant."*

---

## Product Architecture

```
User
  ↓
cmdOS Desktop Interface (Control Center)
  ↓
Agent Runtime
  ↓
Execution Engine
  ↓
Operating System Environment
```

The **Control Center** is the management surface (agents, models, connections,
permissions, memory, execution logs, security). It is a surface within the Interaction
Layer — not a separate runtime layer.

---

## Canonical Stack

```
Interaction Layer → Intelligence Layer → Agent Layer → Capability Layer
→ Communication Layer → Runtime Layer → [Security: cross-cutting] → Kernel
```

- **Interaction Layer** — captures user requests (Control Center + command input).
- **Intelligence Layer** — AI cognition, reasoning, and model routing (AI Router).
- **Agent Layer** — autonomous workers that plan and drive execution.
- **Capability Layer** — executable abilities an Agent invokes to act on the world.
- **Communication Layer** — messages, events, and agent-to-agent coordination.
- **Runtime Layer** — the environment where approved operations execute.
- **Security** — cross-cutting: identity, permission, policy, isolation, monitoring, audit.
- **Kernel** — the single deterministic execution authority. No execution bypasses it.

---

## Desktop-First Principle

Build the desktop application first. Do not begin with a web, browser, or DApp interface.

Build order (this is the Stage 1 / MVP component sequence):

1. Desktop Application Shell
2. Control Center
3. Agent Runtime
4. Execution Engine
5. Capability System
6. AI Router
7. Memory System

---

## Execution Model

Every cmdOS task follows one pipeline:

```
Intent → Understanding → Command → Execution Plan → Permission → Runtime → Verification → Result
```

The kernel-level expansion adds context assembly, reasoning, transaction, observation,
and memory consolidation, but the contract above is the invariant every feature supports.

---

## Agent Model

AI Agents are the primary execution units. Each Agent has identity, memory, planning,
capability access, execution control, and state.

```
Agent → Capability → Execution Runtime → Result
```

Rules:

- Agents **invoke Capabilities, never Plugins**.
- Agents **never mutate canonical state directly**. All actions pass through the
  Capability Layer → Permission System → Execution Runtime.
- Agents **cannot expand their own permissions**.
- Multi-agent work is coordinated by the kernel Agent Orchestrator; agents never modify
  another agent's execution state.

---

## Capability and Plugin

- **Capability** — the core execution primitive: a versioned interface contract plus an
  implementation, held in the Capability Registry. The only thing an Agent invokes at
  runtime.
- **Plugin** — a signed, versioned distribution package. On install, after security
  validation, it registers its Capabilities (and optionally Agents) into the Registry.
  Packaging and provenance, not a runtime concept.

There is **one Marketplace**, and it distributes Plugins.

---

## Local-First Architecture

Core components run on the user's machine whenever possible: Agent Runtime, Execution
Engine, Permission System, Memory Layer. Cloud is supporting infrastructure only (model
access, sync, marketplace, updates).

---

## Security by Default

Security is built into every layer, never added afterward. Required everywhere: identity,
permission control, policy evaluation, sandbox isolation, audit, and transparent actions.

Sensitive operations require explicit user approval before execution — for example
sending messages, deleting files, financial actions, and external communication. The
canonical security model lives in `docs/05-architecture/6-security`.

---

## Observability

Everything meaningful is observable. Every important operation has status, history, logs,
and a recovery path. Track agent actions, execution states, permissions, errors, and
system events.

---

## Core Rules

- Documentation first; RFC before implementation.
- State is authoritative. Cache is never authoritative. Memory is temporary. Events are
  immutable.
- Everything observable. Everything versioned. Secure by default.
- Define interfaces before implementation.
- Never bypass kernel contracts, modify canonical state directly, create hidden execution
  paths, or break architecture boundaries.
- Architectural changes update documentation before code.

---

## UI Direction

The interface is an AI execution runtime, not a chat app and not a traditional dashboard.
It is a command center where users delegate tasks and watch Agents execute.

Main layout: Top System Bar · Left Navigation · (Agent Panel | Execution Workspace |
Task History) · Command Input. The command input is an intent interface, not a chat box.
Every step of execution must be visible.

Visual language: dark, terminal-inspired, premium developer-tool feel; monospace type,
glass panels, subtle glow, status indicators, smooth transitions. Avoid chat bubbles,
social-messaging style, SaaS cards, and generic admin dashboards.

---

## Development Philosophy

Build in dependency order; never build isolated features:

```
Foundation → Runtime → Agents → Capabilities → Execution → Product → Ecosystem
```

Every feature must contribute to the AI Execution Operating System. The roadmap is
defined as five product Stages in `docs/09-roadmap`; engineering workstreams ("Phases")
map into those Stages via `ROADMAP.md`.

---

## Final Goal

```
Human Intent → cmdOS Desktop → AI Agents → Computer Execution → Completed Work
```

Humans express what they want. AI understands the goal. Agents execute the work.
Computers become intelligent execution partners.
