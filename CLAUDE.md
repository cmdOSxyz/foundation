# CLAUDE.md

## Project
cmdOS — The AI Execution Operating System

## Mission
Build a deterministic execution operating system.

## Rules
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

## Documentation
Each RFC must include:
- Purpose
- Responsibilities
- Design Principles
- Security
- Recovery
- Observability
- Summary

## AI Assistant
- Never bypass kernel contracts.
- Never modify canonical state directly.
- Keep architecture consistent.
- Update documentation with architectural changes.

## Goal
Build cmdOS into a universal AI Execution Operating System.
