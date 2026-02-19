# Omni-Glass — AI Agent Instructions

> **MANDATORY:** ALWAYS review `standards/00-manifest.md` before starting any architectural,
> refactoring, or feature implementation task. The manifest will route you to the specific
> sub-standards relevant to your task. Do not skip this step.

---

## Standards System

This project uses a modular engineering standards system located in `standards/`.

- **Master Manifest:** `standards/00-manifest.md` — Start here. It indexes all standards and tells you which files to read based on your task type.
- **Architecture:** `standards/01-architecture.md`
- **Coding Practices:** `standards/02-coding-practices.md`
- **Documentation:** `standards/03-documentation.md`

---

## Key Rules (Quick Reference)

1. **No God Files** — No file (code or docs) may exceed 300 lines.
2. **Vertical Slice Architecture** — Features are self-contained domain folders, not horizontal layers.
3. **Cross-domain imports only through `api/`** — Never import from another domain's `logic/`, `data/`, or `ui/`.
4. **Co-located documentation** — Each module has its own `README.md`.
5. **Functional Core, Imperative Shell** — Pure business logic has zero I/O dependencies.

---

## Build Commands

```bash
# Install dependencies
npm install

# Development server
npm run dev

# Production build
npm run build

# Type checking
npm run typecheck
```

## Test Commands

```bash
# Run all tests
npm test

# Run tests in watch mode
npm run test:watch

# Run tests with coverage
npm run test:coverage
```

## Lint Commands

```bash
# Lint check
npm run lint

# Lint fix
npm run lint:fix
```

---

## Project Structure (Target)

```
Omni-Glass/
  CLAUDE.md              # This file — AI agent instructions
  standards/             # Engineering standards (modular, referral-based)
  src/
    <domain>/            # One folder per business domain
      api/               # Public API (the only importable surface)
      logic/             # Pure business logic
      data/              # Data access / infrastructure
      ui/                # UI components (if applicable)
      __tests__/         # Domain-scoped tests
      README.md          # Module documentation
    shared/              # Cross-cutting utilities and types
```
