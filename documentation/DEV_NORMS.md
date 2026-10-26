# Dev Norms — Sentinel

Developer workflow, conventions, and automation for this repo.

---

## Dev setup

See [Dev Setup](DEV_SETUP.md) for step-by-step instructions to install the required tools, build both halves, run tests, and launch the panel UI on your desktop.

---

## Board automation

Creating a branch moves its associated issue to **In progress** on the Sentinel project board automatically. No manual card dragging needed.

The branch name format `NNN-issue-title` (GitHub's default) is parsed for the issue number. Branches with no leading number (e.g. `main`, `experiment/foo`) are silently ignored.

The automation uses a repository secret named `GH_PROJECT_AUTOMATION_TOKEN` — a personal access token with `repo` and `project` scopes. Never commit this token.

---

## Main branch protection

`main` is protected — you cannot push to it directly. All changes go through a pull request. The ruleset enforces:

- **PRs required** — no direct pushes
- **Linear history** — no merge commits; squash merges only
- **No force pushes or deletions**

The following CI checks are required to pass before a PR can be merged:

| Check | Workflow | What it enforces |
|---|---|---|
| `agent-gates` | `agent-gates.yml` | Format, CVE scan, build, and test the agent |
| `panel-gates` | `panel-gates.yml` | Format, lint, CVE scan, build, and test the panel |
| `Require Issue Link` | `require-issue-link.yml` | PR body contains `Closes #N` |

---

## Git workflow

1. `git switch main && git pull`
2. Use **Create a branch** on the GitHub issue page, then `git fetch && git switch NNN-issue-title`.
3. Commit freely on the branch.
4. Run `./scripts/ci-gates-check.sh` — auto-formats, lints, and tests both halves locally before pushing. Catches failures before they consume pipeline time.
5. `git push -u origin <branch>`
6. Open a PR titled `Type[#NNN]: short description` with `Closes #N` in the body.
7. **Squash and merge.** `main` gets one commit; individual commits are preserved in the PR.
8. Clean up: `git switch main && git pull && git branch -D <branch> && git fetch --prune`

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

Always create branches using **"Create a branch"** on the GitHub issue page. This generates the branch name and links it to the issue automatically.

GitHub's generated format: `NNN-issue-title` (e.g. `6-chore-document-branch-and-commit-conventions`)

If you must create a branch locally, link it to the issue on GitHub before pushing — otherwise the board automation won't fire and the card won't move to In progress.

> The branch automation (`move-to-in-progress.yml`) parses the leading issue number from this format. Branches with no leading number (e.g. `main`, `experiment/foo`) are silently ignored.

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

> `Closes #N` is enforced by CI (`require-issue-link.yml`) — PRs without it will fail the check. `Fixes #N` and `Resolves #N` are also accepted.

---

## Testing strategy

**C# (agent):** Tests live in `agent/tests/`. Framework is xUnit. Run with `dotnet test agent/`. Write a failing test first, then make it pass.

**Rust (panel):** Tests live inline in the same file as the code, inside a `#[cfg(test)] mod tests` block. Run with `cargo test --manifest-path panel/Cargo.toml`. Same TDD approach.

**UI testing:** Slint's `slint::testing` module supports headless component testing. Deferred until M4 when view models with real logic are introduced — there is nothing meaningful to assert against pure layout code.

**CI:** Both halves run automatically on PRs via `agent-gates.yml` and `panel-gates.yml` (format, lint, CVE scan, build, test). Run `./scripts/ci-gates-check.sh` locally before pushing for a faster feedback loop.

---

## Board and PR automation

The full story lifecycle is automated:

| Event | What happens |
|---|---|
| Branch created (`NNN-*`) | Issue card moves to **In progress** |
| PR opened with `Closes #N` | Issue card moves to **In review** |
| PR merged | Issue closed, card moves to **Done** |

Workflows involved: `move-to-in-progress.yml`, `require-issue-link.yml`, and GitHub's built-in project workflows.

CI quality gates run on every PR via `agent-gates.yml` and `panel-gates.yml` (format, lint, CVE scan, build, test).
