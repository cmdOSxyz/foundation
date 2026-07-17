# 04.01 Core Concepts

## Overview

This document defines the fundamental concepts used by cmdOS.

These concepts form the base vocabulary for all other terminology,
architecture, kernel, runtime, and application layers.

The purpose is to ensure that humans, AI agents, and system components
share the same understanding.

------------------------------------------------------------------------

# 1. Core Execution Model

cmdOS is built around the transformation:

Human Intent → Machine Execution

The complete flow:

Intent

↓

Goal

↓

Planning

↓

Decision

↓

Action

↓

Execution

↓

Result

Each stage has a specific responsibility.

------------------------------------------------------------------------

# 2. Intent

## Definition

Intent represents what the user wants to achieve.

Intent is expressed through natural language and does not require
technical commands.

Example:

User:

"Send an email to my manager explaining the project update."

cmdOS understands this as an intent to complete an communication task.

------------------------------------------------------------------------

# 3. Goal

## Definition

Goal represents the desired final outcome of an intent.

Intent describes the user's desire.

Goal describes the target state.

Example:

Intent:

"Book a flight to Tokyo."

Goal:

"Flight reservation successfully completed."

------------------------------------------------------------------------

# 4. Plan

## Definition

Plan is the structured strategy created to achieve a goal.

A plan contains:

-   Required steps
-   Dependencies
-   Resources
-   Execution order

Example:

Goal:

"Book a flight."

Plan:

1.  Search available flights
2.  Compare options
3.  Select flight
4.  Confirm payment
5.  Complete booking

------------------------------------------------------------------------

# 5. Decision

## Definition

Decision represents the selection process used to determine the best
execution path.

cmdOS decisions may include:

-   Selecting tools
-   Selecting agents
-   Selecting models
-   Selecting workflows

------------------------------------------------------------------------

# 6. Action

## Definition

Action represents an individual executable operation.

Examples:

-   Send email
-   Create calendar event
-   Call API
-   Open application
-   Modify file

Actions are the smallest execution units.

------------------------------------------------------------------------

# 7. Execution

## Definition

Execution is the process of performing actions through the cmdOS system.

Execution includes:

-   Validation
-   Permission checking
-   Resource allocation
-   Action processing
-   Result generation

------------------------------------------------------------------------

# 8. Result

## Definition

Result represents the outcome after execution.

Results may include:

-   Success
-   Failure
-   Partial completion
-   Required user confirmation

------------------------------------------------------------------------

# 9. State

## Definition

State represents the current condition of an object, process, or
execution.

Examples:

-   Task pending
-   Task running
-   Task completed
-   Task failed

State allows cmdOS to understand system conditions.

------------------------------------------------------------------------

# 10. Context

## Definition

Context represents information required to understand and execute an
intent.

Context may include:

-   User preferences
-   Previous interactions
-   Environment information
-   Available resources

------------------------------------------------------------------------

# 11. Object

## Definition

Object is a structured representation used internally by cmdOS.

Examples:

-   Intent Object
-   Goal Object
-   Execution Object
-   Agent Object

Objects allow AI decisions to become machine-operable structures.

------------------------------------------------------------------------

# 12. Relationship Between Concepts

The relationship:

Intent creates a Goal.

Goal creates a Plan.

Plan creates Decisions.

Decisions create Actions.

Actions create Execution.

Execution creates Results.

------------------------------------------------------------------------

# 13. Relationship With Architecture

Terminology mapping:

Core Concepts

↓

Architecture

↓

Kernel

↓

Runtime

↓

Applications

This document provides the conceptual foundation for future system
design.

------------------------------------------------------------------------

# Summary

The core concepts define the language of cmdOS.

Every advanced capability in cmdOS depends on these fundamental
concepts.

Understanding these concepts is required before understanding agents,
runtime, kernel, and execution systems.
