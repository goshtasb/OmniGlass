# Architecture Standards

> **Scope:** System design, module boundaries, dependency management, and structural patterns.

---

## Chosen Pattern: Vertical Slice / Modular Monolith

### What It Is

Each **feature or domain** is a self-contained vertical slice that owns all of its layers:
UI (or API surface), business logic, and data access. These slices live in dedicated
domain folders rather than being scattered across horizontal layers.

### Why We Use It

1. **Team Scalability** — Independent teams can own entire features without stepping on each other. Merge conflicts drop significantly when two teams never touch the same folder.
2. **Cognitive Load** — A developer working on "Payments" only needs to understand the `payments/` folder, not the entire codebase.
3. **Deployment Independence** — Slices can be extracted into standalone services later with minimal refactoring, because their boundaries are already clean.
4. **Testability** — Each slice can be tested in isolation. Integration tests target a single domain boundary, not a tangled web of cross-cutting dependencies.

---

## Directory Structure

A domain module must follow this structure:

```
src/
  <domain-name>/
    api/           # Public API surface (exports consumed by other domains)
    logic/         # Business rules, pure functions, domain models
    data/          # Data access, repositories, external service clients
    ui/            # UI components (if applicable)
    __tests__/     # Tests scoped to this domain
    README.md      # Module-level documentation (see 03-documentation.md)
```

### Rules

- Every domain folder **must** have an `api/` directory that explicitly exports its public interface.
- Internal files (inside `logic/`, `data/`, `ui/`) are **private** to the domain.
- No file outside the domain may import from `logic/`, `data/`, or `ui/` directly.

---

## Dependency Rules

### Rule 1: No Cross-Domain Direct Imports

Domains communicate **only** through each other's `api/` surface.

```
# CORRECT
import { createPayment } from '@/payments/api';

# VIOLATION
import { PaymentProcessor } from '@/payments/logic/processor';
```

### Rule 2: Shared Kernel

Code that genuinely belongs to multiple domains goes into a `shared/` module:

```
src/
  shared/
    types/         # Shared type definitions
    utils/         # Truly generic utilities (date formatting, etc.)
    constants/     # Application-wide constants
```

**Guard rail:** Before adding anything to `shared/`, ask: "Does this belong to a specific domain?" If yes, it stays in that domain's `api/`.

### Rule 3: Dependency Direction

Dependencies flow **inward** (infrastructure → application → domain), never outward.

```
External APIs / DB  →  data/  →  logic/  →  api/
                                              ↑
                                     Other domains consume this
```

The `logic/` layer must have **zero** infrastructure dependencies. It operates on plain data and interfaces, not on database clients or HTTP libraries.

---

## Module Boundary Checklist

Before merging code that adds or modifies a domain module, verify:

- [ ] The domain folder contains an `api/` directory with explicit exports.
- [ ] No external code imports from `logic/`, `data/`, or `ui/` directly.
- [ ] The `logic/` layer has no infrastructure imports (no DB clients, no HTTP).
- [ ] A `README.md` exists in the domain folder (see `03-documentation.md`).
- [ ] Cross-domain dependencies are documented in the README.

---

## Anti-Patterns to Avoid

| Anti-Pattern | Why It Is Harmful | Correct Approach |
|---|---|---|
| **God Module** — One domain folder doing everything | Defeats the purpose of modularity; becomes unmaintainable | Split by sub-domain or bounded context |
| **Horizontal Layers** — `controllers/`, `services/`, `models/` at root | Forces developers to touch multiple directories for one feature; high merge conflict rate | Group by feature/domain, not by layer |
| **Implicit Dependencies** — Importing internals without going through `api/` | Creates hidden coupling; refactoring one module breaks another unpredictably | Enforce `api/` as the only entry point |
| **Shared Dump** — Putting everything "reusable" into `shared/` | `shared/` grows into a God Module; everything depends on it | Only put genuinely cross-cutting concerns in `shared/` |

---

## References

- [Vertical Slice Architecture — Jimmy Bogard](https://www.jimmybogard.com/vertical-slice-architecture/)
- [Modular Monolith — Kamil Grzybek](https://www.kamilgrzybek.com/design/modular-monolith-primer/)
- Related standards: [02-coding-practices.md](./02-coding-practices.md), [03-documentation.md](./03-documentation.md)
