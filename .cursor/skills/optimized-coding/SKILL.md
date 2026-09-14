---
name: optimized-coding
description: >-
  Enforces incremental, minimal edits for new requirements instead of rewriting
  whole files or modules. Use whenever creating, editing, refactoring, or
  extending application code after a new requirement, feature ask, or follow-up
  change in this project.
---

# Optimized coding (incremental changes)

## Rule (verbatim)

do optimized coding, not to change complete code every time i give new requirements

## Apply on every change

Prefer the **smallest diff** that satisfies the new requirement. Do not rewrite
working modules, handlers, use cases, or UI components from scratch when an
extension, patch, or small extraction is enough.

### Do

- Read the existing code first; extend what already works
- Add/adjust only the functions, fields, routes, or files the requirement needs
- Prefer surgical edits (`StrReplace` / targeted inserts) over full-file rewrites
- Split out a new small module when a file would otherwise grow past the limit —
  leave the rest of the file intact
- Reuse existing ports, DTOs, hooks, and patterns instead of inventing parallel ones
- Keep unrelated behavior, comments, naming, and formatting unchanged

### Do not

- Replace an entire file “to be clean” when only a few lines need to change
- Re-implement a use case, adapter, or component that already meets most of the need
- Drive-by refactors, renames, or style churn outside the requirement
- Recreate fakes/tests wholesale — update only assertions and constructor args
- “Start fresh” on follow-up asks unless the user explicitly requests a rewrite

### Decision check (before editing)

1. What is the **delta** vs current behavior?
2. Which **existing** types/files already own that concern?
3. Can this be done with **≤ a handful of focused edits**?
4. Only if the current design cannot absorb the change — introduce a **new**
   small module and wire it in, without rewriting neighbors.

## Checklist before done

- [ ] Diff is limited to what the new requirement needs
- [ ] No full-file rewrite of healthy code without a hard constraint (e.g. line limit split)
- [ ] Existing public APIs / behavior preserved unless the requirement changes them
- [ ] Still respects SOLID, design-patterns, code-decorum, and ≤200-line skills
