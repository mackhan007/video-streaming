---
name: commit-past-changes
description: >-
  Split session work into one conventional commit per completed task when the
  user says "commit past changes". Never add Co-authored-by trailers. Use
  whenever committing past/session work or multiple finished tasks.
---

# Commit past changes

## Rule (verbatim)

when i say commit past changes, create a commit for each task and commit it accordingly — never add Co-authored-by

## When to use

User says **commit past changes**, or asks to commit each task / session work separately.

## Do

1. Follow **`docs/wiki/commit-past-changes.md`**
2. One commitlint-valid commit per completed task; path-scoped `git add`
3. After every commit, check `git log -1 --format=%B` — if `Co-authored-by:` (or other unsolicited trailers) appear, **strip them** before moving on (msg-filter / amend with a clean `-F` file). Do not leave them in history.
4. Do not push unless asked

## Do not

- Add `Co-authored-by:`, `Signed-off-by:`, or similar trailers unless the user explicitly requests them
- Squash all tasks into one commit
- Force-push to main/master unless the user explicitly asks to rewrite remote history
