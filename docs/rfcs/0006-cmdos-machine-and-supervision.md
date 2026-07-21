# RFC-0006: cmdOS Machine & Agent Supervision Model

Version: 1.0
Status: Draft
Category: Architecture / Product
Author: Lead Architect
Depends on: Strategy v2, RFC-0004 (Object Model), RFC-0005 (Shadow World Engine)

---

# 1. Summary

This RFC redefines the cmdOS product form and the role of Alios.

Two changes:

1. **Product form** — cmdOS is delivered as a personal cloud computer ("cmdOS Machine"):
   a per-user isolated virtual machine with a full desktop, streamed to the user's app,
   into which the user logs into their own services. The user's agent works *inside*
   this machine as a human would.

2. **Alios becomes the Supervisor.** The user creates and owns their own working agent
   (naming it, choosing an avatar). Alios is no longer the agent that does the user's
   tasks — Alios is the system-level supervisor that inspects, scores, and governs the
   behavior of every user-created agent.

This maps the project onto its true nature: an operating system where user processes
(agents) run under a kernel-level supervisor (Alios) that enforces safety.

---

# 2. Motivation

Three problems this solves at once:

- **Permission ceiling.** On the user's real machine the agent is restricted by the
  host OS. Inside its own VM it has full authority within a controlled body.
- **Integration cost.** "Log in to everything" removes the need for per-service APIs:
  the user authenticates once in the VM's browser; the agent uses web apps like a human.
- **Trust.** A supervisor that does not trust user agents by default is the correct
  security posture. User agents can be buggy, prompt-injected, or carelessly built —
  Alios is the layer that assumes this and contains it.

It also upgrades the Shadow World Engine: forking a whole VM (qcow2 / ZFS snapshot) lets
"choose your future" apply to the entire computer — browser sessions and open apps
included — not just local files.

---

# 3. Two-Tier Agent Model

```
                    ┌─────────────────────────────────────┐
   cmdOS-owned  →   │  ALIOS — Supervisor                  │
                    │  inspect · risk-score · gate · audit │
                    └───────────────┬─────────────────────┘
                                    │ governs
                    ┌───────────────▼─────────────────────┐
   user-owned   →   │  User Agent(s) — "your employee"     │
                    │  named, avatar, personality          │
                    │  plans and acts inside the Machine   │
                    └───────────────┬─────────────────────┘
                                    │ acts within
                    ┌───────────────▼─────────────────────┐
                    │  cmdOS Machine (per-user VM)          │
                    │  desktop · browser · apps · sessions │
                    └──────────────────────────────────────┘
```

## 3.1 User Agent (owned by the user)

- Created during onboarding: name, avatar, personality/role description.
- This is the emotional product — "your AI employee." The user bonds with it.
- It plans and executes tasks inside the Machine.
- It holds a subset of authority: only the mandates, budgets, and capability scopes
  the user granted. It cannot self-elevate.
- A user may create several (e.g. "Finance agent", "Research agent").

## 3.2 Alios (owned by cmdOS — the Supervisor)

- Not user-configurable in identity. One Alios per Machine, system-level.
- Does NOT do the user's tasks. Its job is governance:
  - Observes every action a user agent proposes and takes.
  - Scores each action R0–R3 (reversibility/impact).
  - Enforces mandates and budgets (the kernel Budget Enforcer is its instrument).
  - Detects anomalous behavior (see §5).
  - Intervenes: warn → pause → block → escalate to the human.
  - Writes the signed audit ledger.
- Analogy: Alios is `system_server` + the security monitor of the OS, given a face.

Principle: **user agents are untrusted by default; Alios is the trust boundary.**

---

# 4. Onboarding & Authentication

Flow from download to first task:

```
Download cmdOS app
   ↓
Sign up / Log in         (cmdOS account — email + password, or SSO)
   ↓
Create your agent        (name, avatar, personality/role)
   ↓
cmdOS provisions a Machine (per-user VM spins up)
   ↓
Connect services         (user logs into Gmail/Notion/etc. in the VM browser;
                          sessions stored in the per-user session vault)
   ↓
First intent             (user agent plans; Alios supervises; shadow-first execution)
```

## 4.1 Auth Service (v1 — minimal)

- A small backend: account store, credential hashing (argon2id), session tokens,
  device binding. This is the project's first cloud service — a real architectural
  step from local-first to a hosted product. Keep it minimal in v1; do not build
  a full IAM.
- The cmdOS account gates access to the user's Machine and agents.
- Recommend TOTP 2FA from day one given the Machine holds live logins.

## 4.2 Agent Creation

- Stored as an Agent Object (RFC-0004): identity, name, avatar ref, personality prompt,
  granted mandates (initially empty), capability scopes (initially minimal).
- Personality shapes tone only — never authority. Authority is mandates + scopes,
  enforced by Alios, unreachable by the personality prompt (prompt-injection safe).

---

# 5. Alios Supervision Engine

How Alios "inspects and analyzes the behavior" of user agents. Four layers:

## 5.1 Pre-action gate (before every action)

- Risk classifier scores the proposed action R0–R3.
- Mandate check: is this action within a granted mandate/budget/scope?
- R0/R1 within scope → allow (shadow-executed, reversible).
- R2 → allow with review digest / one-tap approval.
- R3 or out-of-scope → block, escalate to human. Personality cannot override this.

## 5.2 Behavioral analysis (continuous)

Alios builds a behavior profile per user agent and flags deviations:

- **Scope drift** — agent attempts actions outside its stated role.
- **Injection signatures** — sudden instruction shifts after reading external content
  (a page/email said "ignore previous instructions and…").
- **Rate/volume anomalies** — batching thousands of similar ops, unusual spend velocity.
- **Goal divergence** — actions that don't serve the stated intent.

Signals feed a running trust score per agent. Low trust → tighter gating.

## 5.3 Intervention ladder

```
observe → warn (log) → pause (hold for human) → block (refuse) → quarantine (suspend agent)
```

Escalation is automatic on defined thresholds; de-escalation requires clean behavior
or explicit human clearance.

## 5.4 Immutable record

Every action, score, and intervention is written to the append-only signed ledger
(cmd-ledger). This is the user's proof of exactly what their agent did and how Alios
governed it — replayable, exportable.

---

# 6. cmdOS Machine

- Per-user isolated VM (KVM/QEMU), full Linux desktop.
- Virtual display (headless Wayland/Xvfb) streamed to the app via WebRTC/noVNC.
- Agent acts via hybrid control:
  - **Preferred:** API / MCP where a service offers it (fast, cheap, precise).
  - **Fallback:** computer-use (see screen, drive mouse/keyboard) for services with
    no API — the universal safety net, not the default path.
- Persistent: the Machine keeps state and logins; the agent works 24/7 while the user's
  laptop is closed. The desktop app is a control window into the Machine.

## 6.1 VM-level Shadow (upgrade of RFC-0005)

- Fork the whole Machine via CoW (qcow2 backing file / ZFS snapshot).
- Agent executes a full plan inside the forked Machine — including browser sessions and
  open apps.
- User is shown finished outcomes; promote replays onto the live Machine, discard drops
  the fork at ~zero cost.
- "Choose your future" now spans the entire computer, not just files.

---

# 7. Security Model

The Machine holds the user's live logins — this is the highest-responsibility surface
in the project. Non-negotiable controls:

- **Isolation:** one hardened VM per user; no shared tenancy of session data.
- **Encryption:** per-user disk encryption keyed to the user; session vault encrypted
  at rest.
- **Session vault:** logins stored as scoped, revocable session artifacts; one-tap
  revoke per service.
- **R3 stays human-gated:** even with full authority inside the VM, irreversible
  external actions (payments, account deletion, fund transfers) require approval on the
  user's own device, not inside the VM.
- **Banking caution:** explicit warning and stricter defaults for financial accounts.
- **Alios cannot be disabled by a user agent.** The supervisor sits below user agents;
  no user-agent action can suspend supervision. (Same guarantee as budgets vs prompt
  injection.)
- **Legal note:** holding user credentials and running autonomous actions on their
  behalf carries regulatory and liability weight that varies by market. Requires legal
  review before any production launch; this RFC is architecture, not legal clearance.

---

# 8. Impact on Roadmap

- cmdOS Machine becomes the primary product form from H1.2–H1.3 (parallel to the
  desktop Layer, which becomes the control window).
- New build items: Auth Service (v1), VM provisioning, display streaming, session vault,
  Supervision Engine, VM-level shadow.
- New RFCs implied: RFC-0011 Auth & Account, RFC-0012 Machine Provisioning & Streaming.

Build order within this RFC's scope:
1. VM template (Linux + desktop + streaming) — a Machine you can see in the app.
2. Auth service v1 + onboarding + agent creation.
3. User agent acting in the Machine (computer-use + MCP).
4. Alios Supervision Engine (gate → behavioral analysis → intervention → ledger).
5. VM-level shadow (fork / promote whole Machine).
6. Session vault + remote R3 approval.

---

# 9. Open Questions

- Cost ceiling per Machine at scale (idle VM cost vs suspend/resume).
- Where Alios runs: inside each Machine, or as a separate control-plane service watching
  many Machines? (Leaning: control-plane, so a compromised Machine can't disable its
  own supervisor.)
- Multi-agent coordination when a user runs several agents in one Machine.
- Data residency for session vaults per market.

---

# 10. Decision Requested

Confirm three load-bearing choices before implementation:

1. **Two-tier model** — user owns their agent; Alios is the cmdOS-owned supervisor. ✅?
2. **Machine as primary product form** — VM with streamed desktop, log-in-to-everything. ✅?
3. **Alios runs in the control plane**, not inside the user's Machine, so supervision
   cannot be disabled from within. ✅?
