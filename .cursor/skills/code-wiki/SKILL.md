---
name: code-wiki
description: >-
  Routes codebase questions through docs/wiki instead of blind repo search.
  Use at the start of any task that needs to find files, understand EMS/IMS
  layout, APIs, env, infra, or data model in this streaming-service project.
---

# Code wiki router

## Rule (verbatim)

Also create a code wiki so you dont have to dig always you can always route with this

## Always do this first

1. Open **`docs/wiki/INDEX.md`** (repo root relative).
2. Follow the routing table to the matching wiki page.
3. Open only the listed source paths — do not broad-scan the tree unless the wiki is missing the answer.
4. After meaningful structural changes (new crate, route, env, migration, adapter), **update the wiki in the same change**.

## When the wiki is wrong or incomplete

- Fix the wiki page + INDEX row.
- Then implement the code change.

## Do not

- Re-discover crate layout by globbing when INDEX already maps it
- Duplicate long design prose already in README — wiki is a **map**, README is the story
