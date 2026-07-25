<!--
  cmdOS Premium README
  GitHub-compatible: Markdown + safe HTML + Mermaid + Shields
-->

<div align="center">

<img width="100%" alt="cmdOS — The Operating System for AI Agents" src="https://capsule-render.vercel.app/api?type=waving&height=290&color=0:020604,45:06211a,100:00d084&text=cmdOS&fontColor=ffffff&fontSize=82&fontAlignY=38&desc=THE%20OPERATING%20SYSTEM%20FOR%20AI%20AGENTS&descAlignY=60&descSize=19&animation=fadeIn" />

<br />

<img alt="Hire your AI. Give it a computer. Choose the future it builds." src="https://readme-typing-svg.demolab.com?font=JetBrains+Mono&weight=600&size=20&duration=3000&pause=1100&color=00D084&center=true&vCenter=true&repeat=true&width=900&height=50&lines=Hire+your+AI.+Give+it+a+computer.;Turn+human+intent+into+verified+execution.;Plan.+Approve.+Execute.+Verify.+Undo.;Choose+the+future+it+builds." />

<br />

cmdOS turns natural-language intent into real, observable and policy-controlled execution across applications, files, services and devices.

<br />



<br />



<br />

Explore the vision ·See the architecture ·Run the prototype ·Read the roadmap

</div>

[!IMPORTANT]cmdOS is under active development. This repository contains architecture, specifications,behavior contracts, core modules and a runnable reference prototype. Product surfaces,interfaces and internal boundaries may change as the execution kernel matures.

[!NOTE]This README separates what exists today, what is being built, and what remains a long-term vision.

⚡ The idea in one sentence

cmdOS is an AI-native execution environment where users describe outcomes, agents plan the work, the system enforces trust boundaries, and every consequential action is observed, verified and reversible whenever technically possible.

🧭 Navigate

<table>
<tr>
<td width="33%" valign="top">

🧠 Understand

The vision

Why cmdOS exists

What cmdOS is

Core principles

</td>
<td width="33%" valign="top">

⚙️ Explore

Execution lifecycle

System architecture

Alios

Security model

</td>
<td width="33%" valign="top">

🛠️ Build

Repository map

Quick start

Development

Contributing

</td>
</tr>
</table>

🌌 The Vision

<div align="center">

From software people operate

to systems that understand intent and execute responsibly.

</div>

Today, users still translate goals into applications, menus, commands, tabs and workflows.AI can reason about the goal, but the human usually remains the execution engine.

cmdOS is built around a different model:

USER INTENT
     │
     ▼
AI UNDERSTANDING
     │
     ▼
STRUCTURED PLAN
     │
     ▼
POLICY + PERMISSION + RISK
     │
     ▼
REAL EXECUTION
     │
     ▼
VERIFIED RESULT
     │
     ▼
AUDIT + LEARNING + RECOVERY

The ambition is not to put a chatbot beside the operating system.The ambition is to make AI-native execution a system primitive.

The user says what should happen. cmdOS determines how it can happen safely.

🚀 Why cmdOS exists

Modern AI can:

write code;

reason over documents;

search and summarize information;

generate plans;

use tools;

coordinate agents.

But most products still end at the edge of a response.

They can tell a user how to complete a task, yet rarely own the full lifecycle:

Missing capability

Why it matters

Understand the real goal

A prompt is not always the same as the intended outcome.

Build an explicit plan

Complex work needs structure, dependencies and checkpoints.

Choose capabilities safely

Tool access should be controlled, not improvised.

Estimate risk before action

Reading a file and sending money are not equivalent.

Ask approval only where needed

Too many approvals destroy usability; too few destroy trust.

Execute across applications

Real work spans browsers, files, APIs, desktop apps and devices.

Verify the outcome

A successful API call is not always a successful real-world result.

Recover from failure

Partial execution must not leave hidden damage.

Create an audit trail

Users need to know what happened, when and why.

cmdOS exists to make that execution loop observable, governable and recoverable.

The shift

- User learns the interface
- User coordinates the workflow
- User checks every result manually

+ User describes the outcome
+ System constructs and controls the workflow
+ Agents execute through constrained capabilities
+ System verifies and reports the real result

🧠 What cmdOS is

cmdOS is an AI-native operating environment for autonomous and semi-autonomous agents.

A user can express an intent such as:

“Invoice my three clients for this week's hours, send each invoice by email, and show me anything that needs approval before it goes out.”

Alios, the resident Prime Agent, converts that intent into an execution graph.The system then coordinates capabilities, evaluates risk, enforces policy, previews sensitive effects,executes approved actions, verifies outcomes and records the complete transaction.

Product direction

<table>
<tr>
<td width="50%" valign="top">

🖥️ Horizon 1 — cmdOS Layer

A desktop application for Windows, macOS and Linux that adds AI-native execution to the computers and apps people already use.

Designed around:

natural-language intent;

visible execution plans;

permission-aware actions;

local-first control;

reversible workflows;

capability-based integrations.

</td>
<td width="50%" valign="top">

🌍 Long-term — Full cmdOS

A complete AI-native operating system built on a Linux foundation with a purpose-built userspace, execution kernel, agent runtime and capability ecosystem.

The strategic progression:

Layer on existing operating systems.

Own the agent runtime.

Own the execution model.

Own the AI-native userspace.

Become a full AI-native OS.

</td>
</tr>
</table>

✨ The cmdOS promise

<table>
<tr>
<td width="33%" align="center" valign="top">

🎯

Intent-first

Describe the outcome.Do not micromanage the interface.

</td>
<td width="33%" align="center" valign="top">

👁️

Observable

See plans, approvals,actions and results.

</td>
<td width="33%" align="center" valign="top">

🛡️

Governed

Permissions and limits livebelow the model.

</td>
</tr>
<tr>
<td width="33%" align="center" valign="top">

↩️

Reversible

Recover from mistakeswhenever technically possible.

</td>
<td width="33%" align="center" valign="top">

✅

Verified

Measure real outcomes,not just tool responses.

</td>
<td width="33%" align="center" valign="top">

🔌

Open

Extend through protocols,capabilities and agents.

</td>
</tr>
</table>

🔄 Execution lifecycle

cmdOS treats execution as a controlled state machine rather than an unconstrained chain of model calls.

flowchart LR
    I[User Intent] --> U[Understand]
    U --> P[Plan]
    P --> R[Classify Risk]
    R --> G{Permission Gate}
    G -->|Approved| S[Simulate + Snapshot]
    G -->|Rejected| X[Stop Safely]
    S --> E[Execute Capabilities]
    E --> V[Verify Outcome]
    V -->|Valid| C[Commit]
    V -->|Invalid| B[Rollback / Recover]
    C --> L[Audit Ledger]
    B --> L
    L --> O[Observable Result]
    O --> F[Learning]

Lifecycle stages

Stage

Responsibility

Example output

Intent

Capture what the user wants to achieve.

“Send the weekly KPI report.”

Understanding

Resolve context, scope and constraints.

Recipients, date range, source systems.

Planning

Create a structured execution graph.

Read → calculate → draft → approve → send.

Risk classification

Determine the trust level of every action.

Email send = R2. Payment = R3.

Policy check

Apply permissions, budgets and mandates.

Block unauthorized recipient or spend.

Simulation

Preview expected effects.

Final email, attachments and recipients.

Execution

Act through controlled capabilities.

Create report and send approved message.

Verification

Confirm the real-world result.

Report exists; email delivered.

Commit / rollback

Preserve or recover state.

Commit success or restore snapshot.

Ledger

Record actions, evidence and outcomes.

Auditable execution history.

🪄 Example execution

User intent

“Prepare the monthly product report, attach the latest analytics, send it to the leadership team and schedule a review next week.”

cmdOS response model

sequenceDiagram
    autonumber
    participant U as User
    participant A as Alios
    participant K as Execution Kernel
    participant C as Capabilities
    participant L as Audit Ledger

    U->>A: Prepare report, email it, schedule review
    A->>K: Submit structured execution graph
    K->>K: Classify risk and evaluate policy
    K-->>U: Preview recipients, files and meeting time
    U->>K: Approve consequential actions
    K->>C: Execute report, email and calendar steps
    C-->>K: Return results and evidence
    K->>K: Verify external state
    K->>L: Record transaction
    K-->>U: Return verified final result

<details>
<summary><strong>Expand the step-by-step execution</strong></summary>

Interpret the objectiveIdentify period, systems, recipients and scheduling constraints.

Build the execution graphRead analytics, calculate metrics, generate report, validate numbers, draft email and inspect calendars.

Classify riskReading approved analytics may be R0. Sending email and creating a leadership meeting may be R2.

Preview consequential actionsShow the final message, attachments, recipients and meeting details.

Execute approved stepsUse constrained capabilities through the runtime.

Verify the resultConfirm report creation, attachment integrity, email delivery and calendar event state.

Record the transactionPreserve evidence, status and recoverability information in the ledger.

</details>

🌟 Why cmdOS is different

Conversation is not execution

Category

Primary interface

Who owns execution?

Policy layer

Recovery

Verification

Traditional software

GUI

Human

App-specific

Limited

Human

AI chatbot

Conversation

Human

Prompt-level

Rare

Usually absent

Workflow automation

Rules and flows

Mixed

Workflow-level

Partial

Connector-dependent

cmdOS

Intent + execution graph

System + agent

Below model

Architectural goal

First-class

Five defining properties

01 — Reversible by architecture

simulate → snapshot → execute → verify → commit / rollback

Undo is treated as a system responsibility, not a UI convenience.

02 — Risk-proportional trust

Autonomy is appropriate where errors are cheap and recoverable.Explicit approval is required where consequences are material or irreversible.

03 — Limits enforced below the agent

Permissions, budgets, payment mandates and capability scopes are enforced in infrastructure the model cannot rewrite.

04 — Open by protocol

MCP serves as the capability ABI. A2A provides a direction for agent identity, delegation and coordination.

05 — Local-first ownership

The user's environment remains the center of identity, trust, policy and control.

📐 Core principles

Principle

Meaning

Intent over interface

The user states the outcome instead of operating every tool manually.

Execution over suggestion

The system performs work rather than only describing it.

Control at the right boundary

Human approval appears where risk, cost or ambiguity justify it.

Reversibility by design

Recovery is designed before execution, not after failure.

Observability over opacity

Plans, actions and results remain visible.

Policy below prompts

Critical rules cannot depend only on model behavior.

Capability isolation

Tools are scoped, constrained and independently governable.

Protocol openness

The ecosystem grows without forcing everything into one monolith.

Gradual autonomy

Autonomy expands only when trust and evidence justify it.

Reality-based verification

The system checks the actual result, not just the model's belief.

🏗️ System architecture

flowchart TB
    subgraph UX[Experience Layer]
        USER[User]
        SHELL[cmdShell]
    end

    subgraph INTELLIGENCE[Intelligence Layer]
        ALIOS[Alios Prime Agent]
        ROUTER[NIS AI Router]
    end

    subgraph CORE[Execution Kernel]
        SCHED[Scheduler]
        TX[Transaction Engine]
        POLICY[Policy Engine]
        RISK[Risk Classifier]
        VERIFY[Verification Engine]
        LEDGER[Audit Ledger]
    end

    subgraph RUNTIME[Capability Runtime]
        MCP[MCP Capabilities]
        A2A[A2A Agents]
        IPC[AIPC]
    end

    subgraph SERVICES[System Services]
        SEMFS[semfs]
        PAY[cmdPay]
        STORE[Storage]
        ID[Identity]
    end

    subgraph WORLD[External Environment]
        DESKTOP[Desktop Apps]
        BROWSER[Browser]
        CLOUD[Cloud Services]
        DEVICES[Devices]
    end

    USER --> SHELL
    SHELL --> ALIOS
    ALIOS --> ROUTER
    ALIOS --> SCHED
    SCHED --> TX
    TX --> POLICY
    POLICY --> RISK
    RISK --> VERIFY
    VERIFY --> LEDGER
    TX --> MCP
    TX --> A2A
    TX --> IPC
    MCP --> DESKTOP
    MCP --> BROWSER
    A2A --> CLOUD
    IPC --> DEVICES
    TX --> SEMFS
    TX --> PAY
    TX --> STORE
    POLICY --> ID

Architecture layers

<table>
<tr>
<td width="50%" valign="top">

🖥️ Experience layer

cmdShell

intent input

plan preview

approval gates

execution timeline

result and recovery surfaces

🤖 Intelligence layer

intent understanding

planning

capability selection

model routing

orchestration

</td>
<td width="50%" valign="top">

⚙️ Execution kernel

scheduling

transactions

policy

risk

verification

ledger

🔌 Runtime and services

MCP capabilities

A2A delegation

storage

payments

identity

OS and app bridges

</td>
</tr>
</table>

[!IMPORTANT]The model may propose actions. The kernel determines what may happen, under which constraints, when approval is required, and how success must be verified.

🧱 The cmdOS stack

┌─────────────────────────────────────────────────────────────────────┐
│                            cmdShell                                 │
│      intent • plans • approvals • progress • results • recovery    │
├─────────────────────────────────────────────────────────────────────┤
│                             Alios                                   │
│       understanding • planning • orchestration • explanation       │
├─────────────────────────────────────────────────────────────────────┤
│                        Execution Kernel                             │
│  scheduler • transactions • policy • risk • verification • ledger │
├─────────────────────────────────────────────────────────────────────┤
│                    Capability + Agent Runtime                       │
│           MCP capabilities • A2A agents • AIPC bridges             │
├─────────────────────────────────────────────────────────────────────┤
│                         System Services                             │
│             semfs • NIS • cmdPay • storage • identity              │
├─────────────────────────────────────────────────────────────────────┤
│            Desktop • Browser • Cloud • Apps • Devices              │
└─────────────────────────────────────────────────────────────────────┘

🤖 Alios — the Prime Agent

Alios is the resident intelligence of cmdOS.

Alios is not simply a chatbot personality. It is the primary orchestration agent responsible for turning intent into structured execution.

<table>
<tr>
<td width="25%" align="center" valign="top">

🧩 Interpreter

Understands goals, context, constraints and ambiguity.

</td>
<td width="25%" align="center" valign="top">

🗺️ Planner

Builds explicit execution graphs with dependencies and checkpoints.

</td>
<td width="25%" align="center" valign="top">

🎛️ Orchestrator

Coordinates tools, capabilities, services and sub-agents.

</td>
<td width="25%" align="center" valign="top">

💬 Explainer

Communicates risk, approvals, progress and outcomes clearly.

</td>
</tr>
</table>

[!CAUTION]Alios does not replace permissions, policy, transaction control or payment limits.Those boundaries must remain outside the model.

🧾 Transactional execution

The core execution pattern is:

<div align="center">

SIMULATE → SNAPSHOT → EXECUTE → VERIFY → COMMIT / ROLLBACK

</div>

Transaction goals

Goal

System behavior

Preview before consequence

Show the user what sensitive actions will do.

Snapshot before mutation

Preserve recoverable state before changes.

Verify after action

Confirm real effects instead of trusting a tool response.

Commit on confidence

Finalize only when validation succeeds.

Recover on failure

Reverse, compensate or stop safely when execution is incomplete.

True reversal vs compensation

Not every external system supports perfect rollback. cmdOS distinguishes:

true rollback — restore the previous state;

compensating action — create a corrective action;

irreversible but auditable action — require stronger approval and preserve evidence.

🛡️ Security & trust model

cmdOS assumes that:

models can hallucinate;

prompts can be adversarial;

tool outputs can contain malicious instructions;

integrations can be over-permissioned;

external state can change mid-execution;

irreversible actions require stronger guarantees.

Risk levels

Level

Category

Typical examples

Default posture

R0

Observe

Read files, inspect state, query approved data

Autonomous with logging

R1

Low-impact change

Draft content, create local files, organize information

Usually autonomous

R2

Material action

Send email, schedule meetings, update shared records

Preview + approval

R3

High-risk action

Payments, destructive deletion, permission changes

Strong confirmation + safeguards

Security pillars

<table>
<tr>
<td width="33%" valign="top">

🔒 Policy below the model

Rules are enforced in infrastructure that prompt injection cannot rewrite.

</td>
<td width="33%" valign="top">

🧱 Capability isolation

Every tool receives only the permissions and scope it actually needs.

</td>
<td width="33%" valign="top">

👁️ Observable execution

Actions, evidence and outcomes are visible and auditable.

</td>
</tr>
<tr>
<td width="33%" valign="top">

✅ Verification

Success is confirmed in the external system, not assumed.

</td>
<td width="33%" valign="top">

💳 Budget control

Spending and payment mandates are enforced below the agent.

</td>
<td width="33%" valign="top">

🪪 Local-first identity

The user remains the root of agent identity, trust and delegation.

</td>
</tr>
</table>

🔌 Open protocol layer

cmdOS is designed to participate in an open agent ecosystem.

Interface

Role

MCP

Capability ABI and tool interoperability layer

A2A

Agent identity, delegation and multi-agent coordination direction

AIPC

Internal communication between agents, capabilities and services

Shared schemas

Stable contracts across shell, kernel and services

Why openness matters

capabilities can evolve independently;

third-party tools do not need to be hard-coded into the kernel;

users are not locked into one model or vendor;

the ecosystem can grow around stable execution contracts.

💼 Use cases

<table>
<tr>
<td width="50%" valign="top">

🧑‍💼 Knowledge work

prepare and distribute reports;

summarize meetings and create follow-ups;

reconcile data across systems;

draft, approve and send communications;

organize research with evidence.

🏢 Operations

run multi-application checklists;

update approved systems;

monitor workflows;

route exceptions to humans;

coordinate recurring processes.

</td>
<td width="50%" valign="top">

🧑‍💻 Personal execution

organize files;

schedule events;

plan travel;

manage administrative tasks;

coordinate apps on desktop and mobile.

🤖 AI-native workflows

long-running execution with checkpoints;

multi-agent delegation;

conditional tasks under policy;

auditable tool use;

recovery-aware automation.

</td>
</tr>
</table>

Intent examples

“Invoice my clients for this week's hours.”
“Prepare tomorrow's meeting brief.”
“Clean my downloads folder, but ask before deleting anything.”
“Send our weekly KPI update after I approve the final numbers.”
“Compare travel options and prepare the booking, but do not pay yet.”

📂 Repository map

cmdOS/
├── agent/
│   └── alios/                  # Prime Agent
├── capabilities/               # first-party MCP capability servers
├── docs/
│   ├── 00-governance/          # project governance
│   ├── 01-vision/              # strategy and positioning
│   └── rfcs/                   # architectural decisions
├── kernel/                     # Rust execution core
├── prototype/                  # runnable Electron reference implementation
├── schemas/                    # TypeScript contracts
├── services/
│   ├── semfs/                  # semantic filesystem direction
│   ├── nis/                    # AI model router
│   ├── aipc/                   # MCP / A2A communication
│   └── cmdpay/                 # payment and mandate layer
└── shell/                      # cmdShell desktop experience

Path

Purpose

kernel/

Types, scheduler, transactions, policy, verification and ledger

services/

Shared execution infrastructure

agent/alios/

Intent understanding and orchestration

capabilities/

Controlled external action surfaces

shell/

User-facing execution experience

schemas/

Shared data contracts

prototype/

Runnable reference implementation and behavior contracts

docs/

Strategy, RFCs, architecture and governance

🚀 Quick start

[!NOTE]The commands below target the current prototype/reference implementation.

Prototype

npm install
npm test
npm start

Rust workspace

cargo build --workspace

Recommended first path

README.md
   ↓
ROADMAP.md
   ↓
docs/01-vision/strategy-v2.md
   ↓
docs/rfcs/0000-rfc-process.md
   ↓
prototype behavior contracts
   ↓
kernel + services

🛠️ Development

cmdOS development is expected to be:

spec-led — architecture begins with explicit documents;

boundary-conscious — trust and ownership are defined clearly;

behavior-driven — execution semantics are testable;

security-aware — capability and policy implications are documented;

observable — system behavior can be inspected and explained.

Engineering workflow

flowchart LR
    I[Idea] --> D[Discussion]
    D --> R[RFC Draft]
    R --> V[Review]
    V --> A[Accepted Direction]
    A --> M[Implementation]
    M --> T[Tests + Behavior Contracts]
    T --> DOC[Documentation Update]

[!TIP]Architectural work should begin with docs/rfcs/0000-rfc-process.md.

🧪 Technology direction

Area

Direction

Core kernel

Rust

Desktop shell

Tauri direction; Electron reference prototype exists

Shared contracts

TypeScript schemas

Capability interface

MCP

Agent coordination

A2A direction

Model routing

NIS multi-model router

Storage

Local-first and semantic filesystem direction

Payments

cmdPay mandate and budget layer

[!NOTE]Technology choices and internal interfaces may evolve as architecture is validated.

🌍 Project horizons

flowchart LR
    H1[Horizon 1<br/>cmdOS Layer] --> H2[Horizon 2<br/>Own Agent Runtime]
    H2 --> H3[Horizon 3<br/>Own AI-native Userspace]
    H3 --> H4[Horizon 4<br/>Full AI-native OS]

Horizon 1 — cmdOS Layer

Goal: Deliver a real desktop product on top of Windows, macOS and Linux.

natural-language execution;

controlled capabilities;

approval and preview surfaces;

observable workflows;

reversible actions where possible;

local-first user control.

Horizon 2 — Runtime ownership

Goal: Own the agent substrate and execution semantics.

execution graph runtime;

policy and risk engine;

capability ecosystem;

verification architecture;

agent identity and delegation.

Horizon 3 — AI-native userspace

Goal: Move beyond “an app with agents” toward a complete AI-first interaction layer.

Horizon 4 — Full operating system

Goal: Build a Linux-based foundation where AI execution is a native operating-system capability.

📊 Project status

Area

Status

Notes

Vision and positioning

🟢 Defined

Core project direction established

Architecture documentation

🟢 Active

Continues to mature through RFCs

Reference prototype

🟡 Available

Runnable prototype and behavior contracts

Rust execution kernel

🟡 In progress

Core boundaries and semantics evolving

Alios orchestration

🟡 In design / development

Prime Agent behavior being refined

Capability ecosystem

🟡 In progress

First-party and protocol-based direction

Security and policy engine

🟡 In design

Core system requirement

Full AI-native userspace

🔵 Planned

Later horizon

Complete AI-native OS

🟣 Vision

Long-term destination

Legend

🟢 Defined / available

🟡 Active development

🔵 Planned

🟣 Long-term vision

📚 Documentation

Topic

Location

Strategy

docs/01-vision/strategy-v2.md

Roadmap

ROADMAP.md

RFC process

docs/rfcs/0000-rfc-process.md

Governance

docs/00-governance/

Specifications

docs/

Documentation philosophy

One concept → one canonical definition
One architecture decision → one explicit rationale
Superseded ideas → archived, not silently duplicated
Security assumptions → visible, reviewable and testable

🤝 Contributing

Contributions are welcome from people working on:

agent runtimes;

AI infrastructure;

systems programming;

capability protocols;

local-first software;

trust and security architecture;

desktop automation;

developer experience;

AI-native interaction design.

Before opening a pull request

Read the relevant architecture documents.

Check whether the change needs an RFC.

Make security and permission implications explicit.

Add or update behavior contracts where appropriate.

Keep terminology consistent with canonical docs.

Update documentation alongside implementation.

Contribution standard

Clarity over cleverness. Explicit boundaries over hidden coupling. Durable infrastructure over short-term prompt hacks.

🌐 Community

<div align="center">



</div>

📜 License

Distributed under the MIT License. See LICENSE for details.

<div align="center">

<img width="100%" alt="cmdOS footer" src="https://capsule-render.vercel.app/api?type=waving&height=180&section=footer&color=0:00d084,55:06211a,100:020604&animation=fadeIn" />

The future of AI is not only about models that can answer.

It is about systems that can understand intent, execute responsibly, verify reality and recover when things go wrong.

<br />

cmdOS

The Operating System for AI Agents

<sub>Built for observable execution, proportional trust and user-owned intelligence.</sub>

</div>
