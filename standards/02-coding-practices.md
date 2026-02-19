# Coding Practices Standards

> **Scope:** Code quality rules, design principles, file constraints, and naming conventions.

---

## Design Principles

### SOLID

Apply SOLID at the module and function level. Below is each principle with a concrete guideline:

| Principle | Guideline |
|---|---|
| **Single Responsibility** | A function does one thing. A file addresses one concept. If you struggle to name it, it does too much. |
| **Open/Closed** | Extend behavior through composition (new functions, strategy objects), not by modifying existing logic. |
| **Liskov Substitution** | Any implementation of an interface must be safely swappable without changing calling code. |
| **Interface Segregation** | Do not force consumers to depend on methods they do not use. Prefer small, focused interfaces. |
| **Dependency Inversion** | Business logic depends on abstractions (interfaces/types), never on concrete infrastructure. |

### Functional Core, Imperative Shell

Separate **pure logic** from **side effects**:

- **Functional Core** — Pure functions that take data in and return data out. No I/O, no mutations of external state, no database calls. These live in the `logic/` layer of each domain.
- **Imperative Shell** — The thin outer layer that handles I/O: reading from databases, calling APIs, writing to disk. These live in the `data/` and `api/` layers.

**Why this matters:**

1. Pure functions are trivially testable — no mocks required.
2. Side effects are isolated to a small surface area, making bugs easier to trace.
3. The core business rules become portable across different infrastructure.

---

## File Constraints

### Maximum File Length: 300 Lines

No single source file may exceed **300 lines** (including comments and blank lines).

**Why:** Cognitive load. Research on developer productivity consistently shows that comprehension degrades sharply in files beyond 200-300 lines. Smaller files are easier to review, test, and reason about.

**What to do when a file approaches the limit:**

1. Extract a coherent subset of functionality into a new file.
2. Name the new file after the concept it encapsulates, not after the file it was split from.
3. Update imports and ensure the public API surface remains clean.

### Maximum Function Length: 40 Lines

If a function exceeds **40 lines**, it is likely doing too much. Extract sub-steps into well-named helper functions within the same file before considering a new file.

---

## Naming Conventions

### General Rules

1. **Be descriptive, not abbreviated.** `calculateMonthlyRevenue` over `calcMonRev`.
2. **Use domain language.** If the business calls it an "Order", the code calls it an `Order`, not a `Purchase` or `Transaction` (unless those are distinct concepts).
3. **Boolean variables and functions** start with `is`, `has`, `can`, or `should`: `isActive`, `hasPermission`, `canRetry`.
4. **Constants** use `UPPER_SNAKE_CASE`: `MAX_RETRY_COUNT`, `DEFAULT_TIMEOUT_MS`.

### File Naming

| Type | Convention | Example |
|---|---|---|
| Source files | `kebab-case` | `payment-processor.ts` |
| Test files | `kebab-case.test` | `payment-processor.test.ts` |
| Type/Interface files | `kebab-case.types` | `payment.types.ts` |
| Constants files | `kebab-case.constants` | `payment.constants.ts` |

### Directory Naming

- All lowercase, `kebab-case`.
- Domain folders match the domain name: `order-management/`, `user-auth/`.

---

## Error Handling

1. **Fail fast.** Validate inputs at the boundary (API layer). Do not let invalid data propagate into business logic.
2. **Use typed errors.** Define domain-specific error types rather than throwing generic strings.
3. **Never swallow errors silently.** Every `catch` block must either handle the error meaningfully or re-throw it.
4. **Log at the boundary, not inside pure logic.** The imperative shell handles logging.

---

## Import and Dependency Rules

1. **No circular imports.** If module A imports from B and B imports from A, one of them needs restructuring.
2. **Absolute imports over relative imports** for cross-module references.
3. **Relative imports** within the same domain module are acceptable.
4. **No wildcard exports.** Every export must be explicit and intentional.

---

## Code Review Checklist

Before approving any code change, verify:

- [ ] No file exceeds 300 lines.
- [ ] No function exceeds 40 lines.
- [ ] Variable and function names are descriptive (no unexplained abbreviations).
- [ ] Pure logic has no side effects (no I/O in `logic/` files).
- [ ] Error handling is explicit — no empty catch blocks.
- [ ] No circular imports.
- [ ] Domain boundary rules are respected (see `01-architecture.md`).

---

## References

- [SOLID Principles — Robert C. Martin](https://en.wikipedia.org/wiki/SOLID)
- [Functional Core, Imperative Shell — Gary Bernhardt](https://www.destroyallsoftware.com/screencasts/catalog/functional-core-imperative-shell)
- Related standards: [01-architecture.md](./01-architecture.md), [03-documentation.md](./03-documentation.md)
