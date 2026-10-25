# Dev Norms — Sentinel

Developer workflow, conventions, and automation for this repo.

---

## Board automation

Creating a branch moves its associated issue to **In progress** on the Sentinel project board automatically. No manual card dragging needed.

Two branch name formats are recognised:

| Format | Example | Notes |
|---|---|---|
| `type/NNN-description` | `chore/003-update-docs` | Project convention |
| `NNN-description` | `3-update-docs` | GitHub's default "Create a branch" name |

Branches with no issue number (e.g. `main`, `experiment/foo`) are silently ignored.

The automation uses a repository secret named `GH_PROJECT_AUTOMATION_TOKEN` — a personal access token with `repo` and `project` scopes. Never commit this token.

---

## Git workflow

1. `git switch main && git pull`
2. Create a branch: `git switch -c type/NNN-description`, or use **Create a branch** on the issue page then `git fetch`.
3. Commit freely on the branch.
4. `git push -u origin <branch>`
5. Open a PR titled `Type[#NNN]: short description` with `Closes #N` in the body.
6. **Squash and merge.** `main` gets one commit; individual commits are preserved in the PR.
7. Clean up: `git switch main && git pull && git branch -D <branch> && git fetch --prune`

> Use `git branch -D` (force delete) rather than `-d` — Git won't recognise a squashed branch as merged.

---

## Commit message format

```
Type[#issue]: short description
```

Examples: `Chore[#002]: add branch automation`, `Feature[#023]: show CPU temperature`

> If Git opens an editor, `#issue` inside the brackets won't cause problems. If you ever use a format with a leading `#`, set `git config core.commentChar ";"` so Git doesn't swallow the line as a comment.

---

## Branch naming

_To be defined in issue #6. This section will be updated when that story is complete._

---

## PR conventions

- Title: `Type[#NNN]: short description` (matches the squash commit message)
- Body: include `Closes #N` so merging closes the issue and moves the card to Done
- Squash and merge only — the repo is configured to reject other merge strategies
