# ROADMAP

> cmdOS — The Operating System for AI Agents
> **Hire your AI. Give it a computer. Choose the future it builds.**

Canonical strategy: `docs/01-vision/strategy-v2.md`
Product form: **cmdOS Machine** (RFC-0006) — a personal cloud computer per user, where
the user's own agent works and **Alios** supervises it.
Flagship technology: **Shadow World Engine** (RFC-0005) — now at VM level: fork the whole
Machine, let the agent finish plans in a copy, pick the outcome you want.
Superseded roadmaps archived under `docs/archive/`.

---

# Positioning

**Old:** "The AI Execution Operating System" — an app that runs tasks.
**New:** "The Operating System for AI Agents" — you hire an AI employee, give it its own
computer, and a supervisor (Alios) keeps it safe.

Three sentences that define the product:
1. Download cmdOS, create your agent — name it, give it a face.
2. It gets its own cloud computer where it logs into your tools and works, even overnight.
3. Alios watches everything it does, scores every action, blocks what crosses the line —
   and you approve anything irreversible before it happens.

---

# Horizon 1 — cmdOS Machine (months 0-12)

Goal: **1,000 users whose agent does real weekly work for them.**

## H1.0 — Foundation Lock (weeks 0-2)
- Strategy v2 migration merged (prototype/, Rust workspace, behavior contracts)
- RFC-0004 Object Model, RFC-0005 Shadow, RFC-0006 Machine & Supervision accepted
- Wedge persona locked (who the first 1,000 are)

## H1.1 — The Machine You Can See (weeks 2-6)
Technology: VM template (Linux + desktop + streamed display via WebRTC/noVNC);
Auth service v1 (sign up / log in, TOTP 2FA); onboarding (create agent: name, avatar, personality).
Product: user signs up, creates an agent, watches a real cloud desktop boot in the app.
Demo #1: "meet your agent, on its own computer."
Gate: a user can sign up, create an agent, and see a live streamed Machine.

## H1.2 — The Agent Works (weeks 6-12)
Technology: user agent acts in the Machine (hybrid: MCP/API preferred, computer-use fallback);
session vault (log in once, persist securely); Alios Supervision v1 (pre-action R0-R3 gate + mandate check + ledger).
Product: wedge use case live for 50 alpha users; agent does one real recurring task.
Demo #2: "log in once, then tell it what to do."
Gate: an agent completes a real multi-app task; Alios gates and logs every action.

## H1.3 — Choose Your Future (months 3-5)
Technology: VM-level Shadow (fork whole Machine via qcow2/ZFS CoW); promote replays, discard is free; 2-3 parallel forks + comparison.
Product: "choose your future" picker spanning the whole computer; open beta, target 300 users.
Demo #3: three finished outcomes, pick one, it becomes real — the money shot.
Gate: users routinely choose between finished futures for a real task.

## H1.4 — Alios Gets Smart (months 5-7)
Technology: Supervision v2 (scope drift, injection signatures, anomaly, goal divergence + per-agent trust score + intervention ladder);
Alios in the control plane (cannot be disabled from inside a Machine); NIS v1 (local + cloud routing);
Rust strangler: cmd-ledger + cmd-transaction replace prototype.
Product: "a supervisor that never sleeps"; multi-agent (Finance, Research...); target 600 users.

## H1.5 — Overnight & Autonomous (months 7-12)
Technology: persistent always-on Machine; A2A signed Agent Cards; cmdPay v1 (x402 testnet + mandates + kernel budget gate) — LAST.
Product: overnight jobs; remote R3 approval from the user's phone.
Demo #4: agent spends $0.02 inside a $1 mandate — supervised, logged.
Exit: 1,000 weekly-active users; >40% week-4 retention; every R0/R1 shadowed/undoable;
all spending mandate-gated; Alios blocks 100% of out-of-scope R3.

---

# Horizon 2 — Platform & Distro (months 12-30)
Goal: **100,000 users; agent marketplace flywheel turning.**
- cmdOS Cloud GA (subscription); Machine cost optimization (suspend/resume)
- Shadow sinks into cmd-transaction as an OS primitive (every capability shadowed free)
- Agent SDK: build agents that inherit supervision, budgets, shadow for free
- Agent Marketplace: share/sell agents; A2A delegation under caps
- SemFS native; NIS NPU scheduling; cmdPay mainnet + regional rails (legal review per market)
- Bootable cmdOS Distro for power users / mini-PCs
- zk-receipts research (verifiable supervision without exposing agent data)

---

# Horizon 3 — Native (months 30+)
Goal: **the Android moment.**
- OEM devices shipping cmdOS as primary OS
- Multi-device agent mesh under one sovereign identity
- Enterprise agent workforce + compliance ledger
- Full agent economy on cmdPay

---

# Operating Rules
1. Capability follows safety — payments last, supervision first.
2. Strangler fig — Rust replaces prototype only after passing behavior contracts.
3. One wedge until 1,000 users.
4. Every milestone ships one 30-second demo.
5. Alios (supervisor) can never be disabled by a user agent.
6. R3 always human-gated, even inside a fully-authorized Machine.
7. RFC-first; superseded docs archived, never deleted.

# One-Line Plan
See the Machine -> the agent works -> choose the future -> Alios makes it safe ->
it works overnight within limits -> become the OS.
