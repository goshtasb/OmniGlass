# Documentation Standards

> **Scope:** Documentation structure, co-location rules, Architecture Decision Records, and the "No God Docs" policy.

---

## The "No God Docs" Rule

No single documentation file may exceed **300 lines**.

**Why:** The same cognitive load argument that applies to code applies to documentation. A 1,000-line README becomes a write-only document — people write to it but no one reads it. Focused, modular docs are actually consumed.

**When a doc approaches the limit:**

1. Identify the distinct topics covered.
2. Split each topic into its own file.
3. Link from the original file to the new files, keeping the original as an index.

---

## Co-location: Documentation Lives Next to Code

Documentation about a module **must** reside inside that module's directory, not in a separate `docs/` tree at the root.

```
src/
  payments/
    README.md          # Module overview, usage, dependencies
    api/
    logic/
    data/
    adr/               # Architecture Decision Records for this module
      001-payment-gateway-selection.md
```

### Why Co-location

1. **Discoverability** — Developers find docs where they find code. No hunting through a separate directory tree.
2. **Ownership** — The team that owns the code owns the docs. Changes to logic and docs happen in the same PR.
3. **Staleness Prevention** — When docs live far from code, they drift. Co-located docs are visible during code review and get updated naturally.

---

## Required Documentation Per Module

Every domain module must include a `README.md` with these sections:

### 1. Overview

A 2-5 sentence description of what this module does and why it exists.

```markdown
## Overview

The Payments module handles all payment processing for the platform.
It integrates with Stripe for card payments and manages payment lifecycle
events (creation, capture, refund). Other modules interact with Payments
exclusively through its public API.
```

### 2. Public API

List the exports that other modules may consume. This serves as the contract.

```markdown
## Public API

| Export | Type | Description |
|---|---|---|
| `createPayment` | Function | Initiates a new payment intent |
| `refundPayment` | Function | Issues a full or partial refund |
| `PaymentStatus` | Type | Union type of all payment states |
```

### 3. Dependencies

List the other domain modules this module depends on, and what it uses from them.

```markdown
## Dependencies

| Module | Used Exports | Purpose |
|---|---|---|
| `user-auth` | `getCurrentUser` | Identify the paying user |
| `shared` | `formatCurrency` | Display formatting |
```

### 4. Architecture Decisions

Link to any ADRs relevant to this module.

```markdown
## Architecture Decisions

- [ADR-001: Payment Gateway Selection](./adr/001-payment-gateway-selection.md)
```

---

## Architecture Decision Records (ADRs)

When a non-trivial architectural choice is made, record it as an ADR.

### When to Write an ADR

- Choosing between two or more viable technical approaches.
- Introducing a new external dependency.
- Changing an established pattern or convention.
- Deciding **not** to do something (these are equally valuable).

### ADR Template

```markdown
# ADR-NNN: Title

## Status
Accepted | Superseded by ADR-XXX | Deprecated

## Context
What is the issue or decision that needs to be made? What forces are at play?

## Decision
What is the chosen approach?

## Consequences
What are the trade-offs? What becomes easier? What becomes harder?
```

### ADR Rules

1. ADRs are **immutable** once accepted. If a decision changes, write a new ADR that references and supersedes the old one.
2. ADRs live in the `adr/` folder of the relevant module, or in `standards/adr/` for project-wide decisions.
3. Number ADRs sequentially: `001-`, `002-`, etc.

---

## Project-Level Documentation

The following project-level docs are permitted at the repository root:

| File | Purpose |
|---|---|
| `README.md` | Project overview, setup instructions, quickstart |
| `CLAUDE.md` | AI agent instructions and project-specific commands |
| `CONTRIBUTING.md` | Contribution guidelines (optional) |
| `CHANGELOG.md` | Version history (optional) |

All other documentation lives inside `standards/` or inside the relevant module.

---

## Documentation Review Checklist

Before approving changes that affect a module's structure or behavior:

- [ ] The module's `README.md` is updated to reflect the change.
- [ ] Public API table matches the actual exports.
- [ ] Dependencies table is current.
- [ ] If an architectural decision was made, an ADR exists.
- [ ] No single doc file exceeds 300 lines.

---

## References

- [Diátaxis Documentation Framework](https://diataxis.fr/)
- [Architecture Decision Records — Michael Nygard](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
- Related standards: [01-architecture.md](./01-architecture.md), [02-coding-practices.md](./02-coding-practices.md)
