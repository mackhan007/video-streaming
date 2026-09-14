---
name: design-patterns
description: >-
  Enforces appropriate, proven design patterns on all design and code changes.
  Use whenever creating, editing, refactoring, or reviewing application code.
---

# Good design patterns

## Rule (verbatim)

Good design patterns should be used

## Principles

- Prefer **known patterns** over ad-hoc structure when the problem matches
- Use the **simplest** pattern that fits — no pattern for its own sake
- Combine with SOLID and the ≤200-line file limit
- Name types/modules so the pattern is obvious (`*Repository`, `*Store`, `GetUploadUrl`, …)

## Preferred patterns (this repo)

| Pattern | Where |
|---|---|
| **Hexagonal / ports & adapters** | EMS/IMS services — `ports` traits, `adapters` IO |
| **Repository** | Persistence (`VideoRepository`) |
| **Gateway / anti-corruption** | External systems (S3, Redis, Kafka) behind small ports |
| **Use case / application service** | One orchestration per action (`GetUploadUrl`, `CompleteUpload`) |
| **DTO** | API request/response types separate from domain |
| **Composition root** | `build_state`, `main`, `ems-server` wire dependencies |
| **Strategy** (via traits) | Swappable behavior without changing callers |
| **Factory / builder** (light) | Complex object/client setup in adapters only |

## Avoid / use sparingly

- God objects, anemic dump modules, shotgun `utils`
- Premature abstract factories, visitors, deep inheritance
- Singleton mutable global state (prefer injected `Arc`/state)
- Copy-paste “pattern” layers with no behavior

## When adding a feature

1. Identify the use case → app layer
2. Depend on ports; add/extend a trait only if needed (ISP)
3. Implement in an adapter
4. Keep HTTP handlers thin (map DTO ↔ use case)
5. Compose in the binary/gateway, not inside domain

## Checklist before done

- [ ] Structure matches a clear pattern (or a deliberate simple alternative)
- [ ] Ports own IO boundaries; domain stays pure
- [ ] No new cross-layer shortcuts (handler → SQL/SDK)
- [ ] Naming reflects the pattern and responsibility
