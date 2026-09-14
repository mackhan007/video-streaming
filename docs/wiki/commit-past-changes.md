# Commit past changes

When the user says **commit past changes** (or asks to commit work from this session / each task), split the working tree into **one commit per completed task** — not one giant dump.

## Procedure

1. `git status` · `git diff` · `git log -15 --oneline` (match [commitlint](../commitlint.md) style).
2. List unfinished tasks from the conversation (features, fixes, docs, skills).
3. For each task, stage **only** that task’s paths (and its wiki updates).
4. Commit with a focused conventional message (`feat` / `fix` / `perf` / `docs` / `chore` …).
5. Repeat until the working tree is clean (or only unrelated leftovers remain — ask before committing those).
6. Do **not** push unless asked.

## Rules of thumb

- Order by dependency (schema → backend → frontend → docs-only).
- Prefer path-scoped `git add` over `git add -A`.
- If one file mixes tasks, put it with the **primary** task; mention secondary work only if needed in the body.
- Never commit secrets (`.env`, credentials).
- Never amend / force-push unless the user explicitly asks and amend rules allow it.
- Keep subjects short; explain **why** in the body when helpful.
- **Never** add `Co-authored-by`, `Signed-off-by`, or other trailer lines unless the user explicitly asks. After each commit, verify `git log -1 --format=%B` has no trailers; strip them (rebase/msg-filter or amend) if a tool injected them.

## Example

```text
feat(upload): track pipeline steps in Postgres
feat(ims): encode ABR HLS from Kafka uploads
feat(frontend): poll pipeline stepper and play HLS
fix(ims): sniff media before FFmpeg
feat(upload): retry failed processing
docs: document commit-past-changes workflow
```

## Related

- [commitlint.md](../commitlint.md)
- [conventions.md](conventions.md)
