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

## Main branch protection

`main` is protected — you cannot push to it directly. All changes go through a pull request. The ruleset enforces:

- **PRs required** — no direct pushes
- **Linear history** — no merge commits; squash merges only
- **No force pushes or deletions**

CI status checks will also be required once they exist (issue #17).

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

## Issues and story structure

All work is tracked as GitHub issues using one of three templates.

### Feature

A small, vertical slice of user-visible value. Key fields:

| Field | What goes there |
|---|---|
| **User story** | Who wants this, what they want, and why — `As a / I want / so that` |
| **Acceptance criteria** | Testable `Given / when / then` conditions — these become your first tests |
| **Out of scope** | What this story deliberately skips — keeps the slice small |
| **Size** | S, M, or L (see below) |
| **Components** | Which parts of the system are touched |

### Chore

Setup, CI, tooling, refactors, docs, or research. Key fields:

| Field | What goes there |
|---|---|
| **Goal** | What this unlocks for the developer — `As the developer / I want / so that` |
| **Kind** | Category: Setup/tooling, CI/CD, Refactor, Docs, Research, Dependency update |
| **Done when** | Concrete, checkable outcomes — the definition of done |
| **Size** | S, M, or L (see below) |
| **Components** | Which parts of the system are touched |

### Bug

Something broken that should work. Key fields:

| Field | What goes there |
|---|---|
| **What happened** | One or two sentences describing the problem |
| **Steps to reproduce** | Numbered steps to trigger the bug |
| **Expected behavior** | What should have happened instead |
| **Severity** | High (crash / data wrong / core feature unusable), Medium (degraded, workaround exists), Low (cosmetic) |

### Size guide

| Size | Meaning |
|---|---|
| S | Small, single focus |
| M | A few moving parts |
| L | Too big — split before starting |

`L` is never the right answer. If a story is `L`, break it into smaller slices before opening the branch.

---

## PR conventions

- Title: `Type[#NNN]: short description` (matches the squash commit message)
- Body: include `Closes #N` so merging closes the issue and moves the card to Done
- Squash and merge only — the repo is configured to reject other merge strategies
