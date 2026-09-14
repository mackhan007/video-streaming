---
name: file-size-limit
description: >-
  Enforces a hard maximum of 200 lines per source file. Use whenever creating,
  editing, generating, or refactoring any code or config file in this project.
---

# File size limit

## Rule (verbatim)

no file should be more than 200 lines

## When this applies

- Creating new files
- Editing existing files
- Refactors, extractions modules, generated code
- Applies to **source code** and project config (`.rs`, `.ts`, `.tsx`, `.js`, `.py`, `.yml`, `.toml`, skill `SKILL.md`, etc.)
- Does **not** apply to lockfiles, generated build artifacts, vendored/registry caches, diagrams, or long-form docs (`README.md`, `docs/**`) unless those docs are being newly authored as code-adjacent modules

## How to comply

1. Before finishing an edit, check line count (`wc -l` or editor).
2. If a file would exceed **200** lines, split it:
   - Extract modules, helpers, types, adapters, or routes into new files
   - Prefer clear names over dumping leftovers into `util.rs` / `helpers.ts`
3. Never grow a file past 200 lines “just this once.”
4. When touching an already-oversized file, shrink it toward ≤200 as part of the change (split first if needed).

## Split patterns (Rust / this repo)

- `routes.rs` + `handlers_*.rs` instead of one fat API module
- One adapter file per integration (postgres / redis / s3 / kafka)
- One use-case file per app service
- Shared types in small `domain` / `dto` modules

## Checklist before done

- [ ] Every created or modified file is ≤ 200 lines
- [ ] Splits keep public APIs coherent (re-export from `mod.rs` if needed)
- [ ] No unrelated drive-by refactors beyond what’s needed to stay under the limit
