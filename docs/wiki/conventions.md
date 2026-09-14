# Conventions

Project skills live under `.cursor/skills/` — follow them on every change.

| Skill | Rule |
|---|---|
| `code-wiki` | Route via `docs/wiki/INDEX.md` first; keep wiki updated |
| `file-size-limit` | No source file > **200** lines (docs/README exempt) |
| `solid-principles` | SOLID everywhere; hexagonal for services |
| `design-patterns` | Ports/adapters, repository, use case, DTO, composition root |
| `million-tps` | Hot-path efficiency + **indexes with every new query** |

## Patterns in this repo

- Thin HTTP handlers; business logic in `app/`
- Traits in `ports/`; IO in `adapters/`
- Wire in `build_state` / `ems-server` only
- Structured `tracing` (`error`/`warn`/`info`/`debug`); `shared::logging::init()`

## When adding a feature

1. Update wiki (INDEX + topic) if paths/routes change  
2. Add migration/indexes with new queries  
3. Keep files ≤ 200 lines  
4. Prefer extending ports over growing god modules
