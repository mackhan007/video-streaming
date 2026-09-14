---
name: code-decorum
description: >-
  Maintains code decorum: consistent style, naming, layering, and professionalism
  matching the existing codebase. Use whenever creating, editing, reviewing, or
  extending application code in this project.
---

# Code decorum

## Rule (verbatim)

also maintain the code decurum

## Meaning

**Code decorum** = the codebase stays orderly, consistent, and respectful of
existing conventions. New work should look like it belongs here.

## Apply on every change

### Match the house style

- Follow neighboring files for naming, imports, error types, logging, and layout
- Keep hexagonal layers clear: `api` → `app` → `ports` ← `adapters`; no SQL/SDK in handlers
- Prefer the same patterns already used nearby (ports, use cases, DTOs, hooks)
- Keep comments only when they explain non-obvious intent; no noisy narration

### Keep the room tidy

- No leftover dead code, unused imports, or commented-out blocks from the change
- No half-renames or mixed naming styles in the same module
- No drive-by reformatting of untouched regions
- Wire new pieces at the composition root; do not smuggle globals or shortcuts

### Be a good neighbor

- Preserve public APIs and behavior unless the requirement changes them
- Update tests/fakes/wiki only for what you touched
- If you must split a file (e.g. 200-line limit), extract cleanly — do not leave a messy remnant
- Commit messages stay clean: no `Co-authored-by` or other trailers unless the user asks

## Checklist before done

- [ ] Reads like the surrounding code (names, structure, tone)
- [ ] Layers and patterns respected
- [ ] No clutter introduced (dead code, unused imports, random renames)
- [ ] Aligns with `optimized-coding`, SOLID, design-patterns, and ≤200-line skills
