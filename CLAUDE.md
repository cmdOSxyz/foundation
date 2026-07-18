# CLAUDE.md

# Project

cmdOS — The AI Execution Operating System


# Mission

Build a deterministic AI execution operating system.

cmdOS transforms human intent into real-world execution through AI Agents.


# Product Identity

cmdOS is a standalone AI-native desktop operating system application.

The primary product is:

cmdOS Desktop Application


cmdOS is NOT:

- A DApp
- A Web3 application
- A blockchain interface
- A browser-based AI wrapper
- A chatbot application
- A SaaS dashboard


cmdOS is an AI execution environment where users delegate work to AI Agents.


# Product Architecture

The core product architecture:

```
User

↓

cmdOS Desktop Interface

↓

Admin Runtime

↓

Agent Runtime

↓

Execution Engine

↓

Operating System Environment
```


cmdOS is designed as an AI control center that allows users to:

- Create intents
- Manage Agents
- Approve actions
- Monitor executions
- Control AI operations


# Desktop First Principle

cmdOS must be built as a desktop application first.

Do not begin with:

- Web application
- Browser interface
- DApp interface


Development priority:

1. Desktop Application Shell
2. Admin Control Center
3. Agent Runtime
4. Execution Engine
5. Capability System
6. AI Router
7. Memory System


# Admin Desktop

Admin Desktop is the management and control layer of cmdOS.

It is responsible for:

- Agent lifecycle management
- Permission management
- AI model configuration
- API connections
- Memory management
- Execution monitoring
- Security controls
- System configuration


## cmdOS User Interface Direction

The cmdOS interface represents an AI Execution Runtime.

It is not a chat application.

It is not a traditional dashboard.

It is a command center where users delegate tasks and monitor AI Agent execution.

The UI must communicate:

- AI is active
- Agents are working
- Tasks are being executed
- Permissions are controlled
- Results are verifiable


# Interface Concept

The core experience:

User Intent

↓

Agent Understanding

↓

Execution Planning

↓

Permission Approval

↓

Task Execution

↓

Completed Result


The interface should make every step visible.

---

# Main Layout

The cmdOS Desktop interface consists of four major areas:

```
------------------------------------------------

Top System Bar

------------------------------------------------

Left Navigation

| Agent Panel | Execution Workspace | Task History |

------------------------------------------------

Command Input

------------------------------------------------
```

---

# 1. Top System Bar

Purpose:

Provide system-level information.

Displays:

- cmdOS identity
- Current runtime status
- Active workspace
- Active AI model
- Resource status
- Connection status


Example:

```
cmdOS

agent runtime

/workspace

model: Claude Opus

online
```

The top bar represents the operating system state.

---

# 2. Left Navigation Panel

Purpose:

Access cmdOS system modules.

Required modules:

- Command
- Home
- AI Router
- API Connections
- Executions
- Memory
- Marketplace
- Security
- Settings


The navigation represents OS-level capabilities.

It is not application navigation.

---

# 3. Agent Runtime Panel

Purpose:

Display the currently active Agent.

The Agent panel shows:

- Agent identity
- Agent status
- Runtime state
- Control actions


Agent states:

- Online
- Thinking
- Planning
- Executing
- Waiting approval
- Completed
- Failed


Example:

```
AGENT

cmdOS Agent

● online

[Turn off]
```

---

# 4. AI Model Panel

Purpose:

Display AI intelligence configuration.

Shows:

- Current model
- Available models
- Context usage
- Session information


Example:

```
MODEL

Claude Opus

context 200K

ALL MODELS

Claude Opus
Claude Sonnet
GPT
Gemini
Llama
```

The user should understand that cmdOS can route between multiple AI models.

---

# 5. Execution Workspace

Purpose:

The central area where Agent execution happens.

This is the core of cmdOS.

It displays:

- User intent
- AI understanding
- Execution planning
- Actions
- Results


Example:

User:

"Book a 30-minute sync tomorrow afternoon."


cmdOS displays:

```
understanding

find free 30-minute slot


planning

checking calendars

evaluating availability


permission required

Calendar.CreateEvent

Approve / Deny


execution

event created
```

---

# 6. Permission Gate

Purpose:

Protect user control.

Sensitive actions require approval.

The permission UI must clearly show:

- Action
- Target
- Details
- Risk


Example:

```
cmdOS wants your approval

Calendar.CreateEvent

Team Sync

15:00

Room A


Approve     Deny
```

---

# 7. Task History Panel

Purpose:

Provide execution observability.

Displays:

- Previous tasks
- Status
- Time
- Result


Task states:

- Completed
- Running
- Scheduled
- Failed
- Blocked
- Queued


Example:

```
Email Anna

weekly summary

done

2m
```

---

# 8. Command Input

Purpose:

Primary user interaction.

Users communicate through intent.

Example:

```
Tell cmdOS what to do next...
```

The input is not a chat box.

It is an intent command interface.

---

# Visual Design Language

The interface should follow:

- Dark operating system aesthetic
- Terminal-inspired design
- Premium developer tool feeling
- Minimal interface
- High information density
- Real-time updates


Use:

- Monospace typography
- Glass panels
- Subtle glow
- Status indicators
- Smooth transitions


Avoid:

- Chat bubbles
- Social messaging style
- SaaS cards
- Generic admin dashboards


---

# Product Feeling

The final experience should feel like:

An operating system for AI Agents.

A combination of:

- Terminal
- Task manager
- AI command center
- Developer runtime
- Personal AI assistant


The user should feel:

"I give instructions. My AI system executes."

```


The user experience should feel like:

"I have an AI system operating my computer."


Not:

"I am chatting with an AI assistant."


# Core Rules

- Documentation first
- RFC before implementation
- Intent → Command → Execution Plan → Runtime
- State is authoritative
- Cache is never authoritative
- Memory is temporary
- Events are immutable
- Security by default
- Everything observable
- Everything versioned


# Architecture Principles

cmdOS architecture must follow:

- Deterministic execution
- Explicit system boundaries
- Modular architecture
- Clear contracts
- Observable behavior
- Secure execution


Never:

- Bypass kernel contracts
- Modify canonical state directly
- Create hidden execution paths
- Break architecture boundaries


Always:

- Keep architecture consistent
- Update documentation with architectural changes
- Define interfaces before implementation


# Execution Model

Every cmdOS task follows:

```
Intent

↓

Understanding

↓

Command

↓

Execution Plan

↓

Permission

↓

Runtime

↓

Verification

↓

Result
```


# Agent Model

AI Agents are the primary execution units of cmdOS.

Each Agent contains:

- Identity
- Memory
- Planning
- Capability access
- Execution control
- State management


Agent execution model:

```
Agent

↓

Capability

↓

Execution Runtime

↓

Result
```


Agents must never directly manipulate system state.

All actions must pass through:

- Capability Layer
- Permission System
- Execution Runtime


# Local First Architecture

cmdOS follows a local-first execution philosophy.

Core components should run on the user's machine whenever possible.

Local components:

- Agent Runtime
- Execution Engine
- Permission System
- Memory Layer


Cloud services are supporting infrastructure only.

Cloud may provide:

- AI model access
- Synchronization
- Marketplace
- Updates


# Security Principles

Security is built into every layer.

Required:

- Permission control
- User approval
- Execution audit
- Data protection
- Transparent actions


Sensitive operations require explicit approval.

Examples:

- Sending messages
- File deletion
- Financial actions
- External communication


# Observability

Everything in cmdOS must be observable.

Track:

- Agent actions
- Execution states
- Permissions
- Errors
- System events


Every important operation must have:

- Status
- History
- Logs
- Recovery path


# Documentation

Documentation is part of development.

Every RFC must include:

- Purpose
- Responsibilities
- Design Principles
- Architecture
- Security
- Recovery
- Observability
- Summary


Architectural changes must update documentation before implementation.


# Development Philosophy

Build in this order:

Foundation

↓

Runtime

↓

Agents

↓

Capabilities

↓

Execution

↓

Product

↓

Ecosystem


Do not build isolated features.

Every feature must contribute to:

The AI Execution Operating System.


# Final Goal

Build cmdOS into a universal AI Execution Operating System.

The final computing model:

```
Human Intent

↓

cmdOS Desktop

↓

AI Agents

↓

Computer Execution

↓

Completed Work
```


The objective:

Humans express what they want.

AI understands the goal.

Agents execute the work.

Computers become intelligent execution partners.
