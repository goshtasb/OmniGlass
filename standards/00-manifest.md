# Engineering Standards — Master Manifest

> **Role:** This is the central referral index for all engineering standards.
> Every standard lives in its own focused file. This manifest tells you where to look.

---

## How to Use This System

### For AI Agents

Before starting **any** architectural, refactoring, or feature task:

1. **Read this manifest first** to identify which sub-standards apply.
2. **Read only the relevant sub-files** — do not load everything unless performing a full audit.
3. **Follow the rules** defined in each sub-file. They are non-negotiable unless the user explicitly overrides them.

### Task-to-Standard Routing

| Task Type | Read These Files |
|---|---|
| New feature / module | `01-architecture.md`, `02-coding-practices.md` |
| Refactoring existing code | `01-architecture.md`, `02-coding-practices.md` |
| Writing or updating docs | `03-documentation.md` |
| Code review | `02-coding-practices.md`, `01-architecture.md` |
| Setting up a new domain/module | `01-architecture.md`, `03-documentation.md` |
| Full project audit | All files in `standards/` |

---

## Standards Index

| File | Scope | Summary |
|---|---|---|
| [01-architecture.md](./01-architecture.md) | System Design | Vertical Slice Architecture, Modular Monolith, domain boundaries, public API contracts |
| [02-coding-practices.md](./02-coding-practices.md) | Code Quality | SOLID principles, Functional Core / Imperative Shell, file size limits, naming conventions |
| [03-documentation.md](./03-documentation.md) | Documentation | No God Docs rule, co-located docs, ADRs, required doc structure per module |

---

## Core Principles (Quick Reference)

These principles are expanded in detail within each sub-file:

1. **No God Files** — No single file (code or documentation) may exceed 300 lines.
2. **Modularity First** — Loose coupling, high cohesion. Every module owns its boundaries.
3. **Referral Over Repetition** — Link to the canonical source; never duplicate rules across files.
4. **Co-location** — Code and its documentation live together, not in separate trees.
5. **Explicitness** — Public APIs are declared, not implied. Dependencies are visible, not hidden.

---

## Extending This System

To add a new standard:

1. Create a new file following the naming convention: `NN-topic-name.md` (e.g., `04-testing.md`).
2. Keep the file under 300 lines — split into sub-files if needed.
3. Add an entry to the **Standards Index** table above.
4. Add routing rules to the **Task-to-Standard Routing** table.

---

## Version

| Field | Value |
|---|---|
| Created | 2026-02-19 |
| Last Updated | 2026-02-19 |
| Owner | Engineering Team |
