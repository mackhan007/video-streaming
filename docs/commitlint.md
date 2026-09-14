# Commitlint

This repo uses [commitlint](https://commitlint.js.org/) with [@commitlint/config-conventional](https://github.com/conventional-changelog/commitlint/tree/master/@commitlint/config-conventional).

## Setup (once per clone)

```bash
npm install
```

`prepare` runs `husky`, which installs the `commit-msg` hook.

## Message format

```
type(scope?): subject

# examples
feat(upload): add soft-delete endpoint
fix(ems): correct ready probe s3 check
docs: update local-setup smoke test
chore: add commitlint
```

Allowed types: `build`, `chore`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert`, `style`, `test`.

## Manual check

```bash
echo "feat: something" | npx commitlint
npx commitlint --from HEAD~1 --to HEAD --verbose
```

Config: `commitlint.config.js` · Hook: `.husky/commit-msg`
