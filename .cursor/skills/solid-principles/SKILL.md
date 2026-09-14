---
name: solid-principles
description: >-
  Enforces SOLID principles on all design and code changes. Use whenever
  creating, editing, refactoring, or reviewing application code in this project.
---

# SOLID principles

## Rule (verbatim)

SOLID Principals should be used every where

## Apply on every change

Use SOLID for new code and when touching existing code. Prefer small, focused modules (also respect the 200-line file limit skill).

### S — Single Responsibility

- One reason to change per type/module
- Split HTTP handlers, use cases, adapters, and domain logic
- Avoid god services / kitchen-sink `lib.rs`

### O — Open/Closed

- Extend via new types or trait impls, not by editing large `match`/`if` chains for every case
- Prefer ports (traits) so behavior can grow without rewriting callers

### L — Liskov Substitution

- Trait implementors must honor the trait contract (same pre/postconditions)
- Do not weaken errors or silently no-op required behavior in an impl

### I — Interface Segregation

- Small, role-specific traits (`VideoRepository`, `ObjectStore`, …) over one mega-port
- Callers depend only on methods they use

### D — Dependency Inversion

- High-level app/use-case code depends on traits (ports), not Postgres/S3/Redis/Kafka concretes
- Wire concretes in composition roots (`build_state`, `main`, gateway bootstrap) only

## Hexagonal layout (this repo)

```
api/        # HTTP DTOs + handlers (thin)
app/        # use cases — depend on ports only
domain/     # pure types, no IO
ports/      # traits
adapters/   # IO implementations of ports
```

Bins / `ems-server` only compose adapters → ports → router.

## Checklist before done

- [ ] New behavior lives in the right layer (not IO inside domain, not business rules in handlers)
- [ ] Dependencies point inward (adapters → ports ← app)
- [ ] Traits are small and purpose-built
- [ ] No new hard-wiring of SDK/DB clients inside use cases
- [ ] File stays ≤ 200 lines (split by responsibility if needed)
