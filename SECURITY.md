# SECURITY

# Security Policy

Thank you for helping improve the security of cmdOS.

Security is a core design principle of the project.

---

## Supported Versions

| Version | Supported |
|---------|-----------|
| Main | ✅ |
| Development | ✅ |
| Legacy | ❌ |

---

## Reporting a Vulnerability

Please do **not** disclose security vulnerabilities publicly.

Instead:

1. Contact the maintainers privately.
2. Provide reproduction steps.
3. Include logs if available.
4. Allow time for investigation before public disclosure.

---

## Security Principles

cmdOS follows:

- Least Privilege
- Capability-Based Security
- Deterministic Execution
- Immutable Audit Events
- Zero Hidden State
- Secure Defaults
- Explicit Permissions

---

## Security Architecture

Every execution passes through:

Intent

↓

Validation

↓

Authorization

↓

Execution Plan

↓

Permission Check

↓

Runtime

↓

Verification

↓

Audit

---

## Scope

Security applies to:

- Kernel
- Runtime
- Capability System
- Provider Integrations
- Desktop Agent
- SDKs

---

## Goals

- Prevent privilege escalation
- Prevent unauthorized execution
- Protect secrets
- Ensure auditability
- Maintain deterministic behavior

---

Thank you for helping keep cmdOS secure.
