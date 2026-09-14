# Conventions

Project skills live under `.cursor/skills/` — follow them on every change.

| Skill | Rule |
|---|---|
| `code-wiki` | Route via `docs/wiki/INDEX.md` first; keep wiki updated |
| `file-size-limit` | No source file > **200** lines (docs/README exempt) |
| `solid-principles` | SOLID everywhere; hexagonal for services |
| `design-patterns` | Ports/adapters, repository, use case, DTO, composition root |
| `million-tps` | Hot-path efficiency + **indexes with every new query** |
| `optimized-coding` | **Incremental diffs** — do not rewrite whole modules on each new requirement |
| `code-decorum` | Keep **code decorum** — consistent style, tidy diffs, match house conventions |
| `commit-past-changes` | **One commit per task**; never add `Co-authored-by` unless asked |

Commit messages: [commitlint](https://commitlint.js.org/) — see [../commitlint.md](../commitlint.md).

**Commit past changes:** when asked to commit session/past work, make **one commit per task** — see [commit-past-changes.md](commit-past-changes.md). Never add `Co-authored-by`.

## Patterns in this repo

- Thin HTTP handlers; business logic in `app/`
- Traits in `ports/`; IO in `adapters/`
- Wire in `build_state` / `ems-server` only
- Structured `tracing` (`error`/`warn`/`info`/`debug`); `shared::logging::init()` → stdout + `logs/<server>.log`

## When adding a feature

1. Update wiki (INDEX + topic) if paths/routes change  
2. Add migration/indexes with new queries  
3. Keep files ≤ 200 lines  
4. Prefer extending ports over growing god modules
