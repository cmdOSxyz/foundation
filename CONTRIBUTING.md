# CONTRIBUTING

Thank you for your interest in contributing to cmdOS.

cmdOS is an AI Execution Operating System designed around deterministic execution, security, observability and extensibility.

---

# Development Principles

All contributions should follow these principles:

- Deterministic by design
- Security first
- Observable behavior
- Versioned interfaces
- RFC-driven development
- Backward compatibility where practical
- Minimal hidden state

---

# Contribution Workflow

1. Fork the repository.
2. Create a feature branch.
3. Implement your changes.
4. Update or create RFC documentation when architecture changes.
5. Add or update tests.
6. Open a Pull Request.

---

# Documentation First

Architecture changes must be documented before implementation.

Every major subsystem should include:

- Purpose
- Responsibilities
- Non-responsibilities
- Lifecycle
- Security
- Recovery
- Observability

---

# Code Standards

- Small, focused commits
- Clear naming
- No undocumented behavior
- Avoid breaking public contracts
- Keep modules loosely coupled

---

# Pull Request Checklist

- [ ] Documentation updated
- [ ] Tests added or updated
- [ ] RFC updated (if required)
- [ ] Backward compatibility considered
- [ ] Security implications reviewed

---

# Communication

Please discuss major architectural proposals through RFCs before implementation.

Thank you for helping build cmdOS.
