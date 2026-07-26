<div align="center">

<img src="./assets/hero-dark-3d.svg" width="100%" alt="cmdOS — the ai execution operating system" />

<br />

[![website](https://img.shields.io/badge/website-cmdos.xyz-080b12?style=for-the-badge)](https://cmdos.xyz/)
[![x](https://img.shields.io/badge/x-@cmdOS__xyz-080b12?style=for-the-badge)](https://x.com/cmdOS_xyz)
[![telegram](https://img.shields.io/badge/telegram-cmdOS__xyz-080b12?style=for-the-badge)](https://t.me/cmdOS_xyz)
[![email](https://img.shields.io/badge/email-hello@cmdos.xyz-080b12?style=for-the-badge)](mailto:hello@cmdos.xyz)

<br />

**intent → understanding → planning → permission → execution → verification**

cmdOS is an ai-native execution layer that turns natural-language intent into secure, observable and verifiable action across applications, agents, devices and digital environments.

[vision](#vision) · [platform](#platform) · [architecture](#architecture) · [product](#product-experience) · [roadmap](#roadmap) · [community](#community)

</div>

---

## vision

most ai products stop at the answer.

cmdOS is designed for the next step: **execution**.

a user describes the desired outcome without needing to know commands, apis, integrations or workflow syntax. cmdOS interprets the intent, creates a plan, requests permission where required, coordinates specialized agents, executes across tools and verifies the final result.

```text
user intent
    ↓
ai understanding
    ↓
dynamic planning
    ↓
permission and policy
    ↓
agent execution
    ↓
evidence and verification
    ↓
trusted result
```

> the goal is not to make every application conversational.  
> the goal is to make intent executable.

---

## platform

cmdOS is positioned between the user and the environments where work happens.

| layer | responsibility |
|---|---|
| **experience** | desktop, mobile, web, chat, voice and command interfaces |
| **intent** | understand goals, context, constraints and success criteria |
| **orchestration** | plan tasks, route models, assign agents and recover from failure |
| **trust** | identity, permissions, policies, secrets, isolation and audit |
| **runtime** | execute through browser, terminal, desktop, mobile, cloud and edge agents |
| **verification** | collect evidence, validate outcomes and return a trusted result |

### core product surfaces

| product surface | purpose |
|---|---|
| **command workspace** | describe outcomes and inspect the generated plan |
| **execution timeline** | follow agents, approvals, retries, tool calls and results |
| **cmdOS connect** | bring people and ai agents into the same operational conversation |
| **agent runtime** | run specialized capabilities across trusted environments |
| **workflow studio** | turn successful executions into reusable systems |
| **security center** | control permissions, devices, secrets and execution policies |

---

## why cmdOS

### ai that acts

traditional ai generates content. cmdOS is designed to coordinate real execution.

### control before autonomy

sensitive actions stay visible, scoped and permissioned before they run.

### local-first by design

private tasks should execute locally whenever the required capability is available.

### observable execution

the user can inspect what is happening, which agent is acting, what data is used and what remains blocked.

### verification, not assumption

an attempted action is not treated as success. cmdOS gathers evidence and validates the outcome.

### human and agent collaboration

people and agents can share conversations, files, workflows, decisions and execution context.

---

## architecture

<img src="./assets/architecture-dark-3d.svg" width="100%" alt="cmdOS architecture" />

### experience layer

captures intent through desktop, mobile, web, command, chat and voice interfaces.

### intent and orchestration layer

interprets goals, builds dynamic plans, routes models, delegates work and manages recovery.

### permission and security layer

evaluates identity, policies, approval requirements, data boundaries and secrets.

### agent runtime and mesh

coordinates specialized agents across local devices, browsers, terminals, cloud services and trusted peers.

### execution and verification layer

performs actions, records evidence, validates outcomes and returns a trusted result.

```mermaid
flowchart LR
    U[user intent] --> I[intent engine]
    I --> P[planning engine]
    P --> G[permission gate]
    G --> R[agent runtime]
    R --> T[apps · apis · devices]
    T --> V[verification engine]
    V --> O[trusted outcome]
    V -. recovery .-> P
```

---

## product experience

<img src="./assets/product-dark-3d.svg" width="100%" alt="cmdOS product experience" />

the core interaction is deliberately simple:

```text
1. describe the outcome
2. review the interpreted goal
3. inspect the execution plan
4. approve sensitive actions
5. watch agents execute
6. receive a verified result
7. save the workflow when useful
```

### example intent

```text
“prepare the launch report, identify blockers,
email the team and schedule a review meeting tomorrow.”
```

### example plan

```yaml
goal: prepare_and_distribute_launch_report

constraints:
  - use approved internal sources
  - exclude customer-identifying information
  - require approval before external delivery

steps:
  - collect_project_updates
  - identify_blockers
  - draft_report
  - verify_sources
  - request_delivery_approval
  - send_report
  - schedule_review
  - confirm_delivery
```

---

## workflow studio

<img src="./assets/workflow-dark-3d.svg" width="100%" alt="cmdOS workflow studio" />

successful executions can become reusable workflows containing:

- intent definitions;
- agent assignments;
- tool calls;
- permission gates;
- conditions;
- verification steps;
- final result delivery.

### workflow blocks

| block | role |
|---|---|
| **intent** | defines the desired outcome and constraints |
| **agent** | assigns a specialized capability |
| **tool** | connects an application, api, device or service |
| **permission** | pauses execution for authorization |
| **condition** | selects the next path based on state or evidence |
| **verification** | confirms whether the expected outcome was achieved |
| **result** | stores, shares or delivers the verified output |

---

## trusted execution

every meaningful workflow follows the same trust model:

1. **understand** — capture the goal, context, constraints and definition of success.
2. **orchestrate** — select the models, agents, tools and fallback paths.
3. **authorize** — expose high-impact actions and request permission.
4. **execute** — run scoped operations through trusted runtimes.
5. **verify** — confirm the required outcome with evidence.

### permission request

```text
permission required

action:
send “weekly product report” to leadership@example.com

agent:
communication agent

data leaving device:
1 approved pdf

scope:
one-time send

[reject] [review details] [approve]
```

### execution receipt

```json
{
  "goal": "send the approved weekly report to leadership",
  "status": "verified",
  "executed_by": "communication-agent",
  "approved_by": "user",
  "evidence": [
    "approval-receipt",
    "document-hash",
    "mail-provider-delivery-id"
  ]
}
```

---

## cmdOS connect

cmdOS connect is the communication layer for agent-native teams.

### human conversations

- direct messages;
- private and public spaces;
- files and threads;
- voice and video;
- shared decisions;
- execution history.

### agent participation

agents can:

- join approved spaces;
- respond to mentions;
- summarize decisions;
- create tasks;
- execute authorized workflows;
- provide evidence;
- request human approval.

```text
discussion
   ↓
decision
   ↓
agent assignment
   ↓
permission
   ↓
execution
   ↓
verified update in the same conversation
```

---

## agent runtime

the runtime manages the lifecycle of specialized execution agents.

### runtime targets

| target | example capability |
|---|---|
| **browser** | navigate websites and operate web applications |
| **desktop** | interact with local applications and files |
| **terminal** | run commands, tests, builds and developer workflows |
| **mobile** | perform approved device-level actions |
| **cloud** | access remote apis and infrastructure |
| **edge** | execute near devices or private environments |

### agent state

```text
idle
  ↓
assigned
  ↓
planning
  ↓
waiting for permission
  ↓
executing
  ↓
verifying
  ↓
completed
```

possible terminal states:

- completed;
- partially completed;
- blocked;
- rejected;
- cancelled;
- verification failed;
- rolled back.

---

## ai router

no single model should handle every task.

the ai router selects models according to:

- task type;
- reasoning depth;
- latency;
- cost;
- privacy;
- context length;
- tool support;
- reliability;
- user policy.

| workload | preferred route |
|---|---|
| sensitive local task | local model |
| fast classification | lightweight model |
| deep planning | strong reasoning model |
| coding | code-specialized model |
| vision | multimodal model |
| high reliability | ensemble or verifier route |

---

## secure agent mesh

the secure agent mesh enables approved agents and devices to collaborate without exposing unrestricted control.

### mesh principles

- explicit device trust;
- agent identity;
- capability negotiation;
- scoped delegation;
- encrypted transport;
- signed results;
- revocation;
- auditability.

```text
mobile intent
   ↓
desktop planner
   ↓
workstation execution agent
   ↓
verifier agent
   ↓
signed result
   ↓
mobile notification
```

---

## security model

cmdOS follows a permission-first, zero-trust execution model.

### planned controls

- scoped permissions;
- one-time approvals;
- capability isolation;
- sandboxed execution;
- policy evaluation;
- encrypted secrets;
- signed agents and plugins;
- trusted device registry;
- execution limits;
- emergency stop;
- immutable audit events;
- result verification.

### data principles

- collect the minimum required data;
- keep private execution local when possible;
- separate credentials from model context;
- expose data movement before approval;
- make permissions revocable;
- avoid treating model output as trusted by default.

> security controls described here represent the intended architecture and must be independently validated before production deployment.

---

## developer platform

cmdOS is intended to support an ecosystem of agents, tools, connectors and reusable workflows.

### agent sdk

defines planning, execution, verification, recovery and lifecycle contracts.

### tool sdk

provides permissioned interfaces to applications, apis, files, devices and services.

### workflow sdk

packages repeatable execution systems with explicit inputs, controls and expected outcomes.

```yaml
agent:
  name: release-verifier
  version: 0.1.0

capabilities:
  - inspect_build
  - run_tests
  - verify_release

permissions:
  - repository:read
  - deployment:approve

outputs:
  - verification_report
  - evidence_bundle
```

---

## roadmap

<img src="./assets/roadmap-dark-3d.svg" width="100%" alt="cmdOS roadmap" />

### 01 · foundation

- canonical product architecture;
- execution lifecycle;
- permission model;
- design system;
- public documentation.

### 02 · local runtime

- desktop command workspace;
- local agents;
- browser and terminal tools;
- local model integration;
- execution timeline.

### 03 · cmdOS connect

- human and agent workspaces;
- mentions and delegated tasks;
- shared execution context;
- mobile approvals.

### 04 · secure agent mesh

- trusted devices;
- agent identity;
- encrypted transport;
- capability negotiation;
- signed results.

### 05 · distributed execution

- cross-device scheduling;
- remote capabilities;
- workload routing;
- recovery and retry;
- multi-agent verification.

### 06 · ecosystem

- agent sdk;
- tool sdk;
- workflow sdk;
- marketplace and distribution;
- enterprise policy controls.

> roadmap items describe product direction and are not commitments to a specific release date.

---

## project status

cmdOS is currently in the product architecture and early development stage.

| area | status |
|---|---|
| brand and positioning | active |
| product architecture | in development |
| public website | available |
| desktop runtime | planned |
| mobile companion | planned |
| cmdOS connect | planned |
| secure agent mesh | research |
| developer sdk | planned |

---

## repository structure

```text
cmdOS/
├── apps/
│   ├── desktop/
│   ├── mobile/
│   ├── web/
│   └── connect/
├── packages/
│   ├── agent-runtime/
│   ├── ai-router/
│   ├── permission-engine/
│   ├── verification-engine/
│   ├── workflow-sdk/
│   ├── tool-sdk/
│   └── ui/
├── services/
│   ├── orchestration/
│   ├── identity/
│   ├── mesh/
│   └── audit/
├── docs/
├── examples/
└── README.md
```

this structure represents the intended platform organization rather than a claim that every component is already implemented.

---

## frequently asked questions

<details>
<summary><strong>is cmdOS a traditional operating system?</strong></summary>

not in the kernel sense. cmdOS is an ai-native execution layer designed to coordinate models, agents, tools, applications, devices, permissions and verified outcomes.

</details>

<details>
<summary><strong>is cmdOS only for crypto?</strong></summary>

no. cmdOS is designed as a general-purpose execution platform for productivity, development, research, operations, communication, devices and digital services.

</details>

<details>
<summary><strong>does cmdOS replace existing applications?</strong></summary>

no. it is designed to operate across existing applications and services through a unified intent, permission, execution and verification layer.

</details>

<details>
<summary><strong>can cmdOS work locally?</strong></summary>

local-first execution is a core design principle. local models, agents and storage should be used whenever practical.

</details>

<details>
<summary><strong>are all features shown here available?</strong></summary>

no. this readme communicates the product vision, architecture and intended experience. the project status and roadmap separate current work from planned capabilities.

</details>

---

## community

<div align="center">

<img src="./assets/footer-dark-3d.svg" width="100%" alt="cmdOS footer" />

[website](https://cmdos.xyz/) · [x](https://x.com/cmdOS_xyz) · [telegram](https://t.me/cmdOS_xyz) · [email](mailto:hello@cmdos.xyz)

<br />

<sub>cmdOS · the ai execution operating system</sub>

</div>
