# CLAUDE.md — Sentinel

Guidance for AI agents working in this repo. Keep this file short; detailed design lives in `documentation/`.

## Start of every session

1. Read the newest files in `.context/` (gitignored summaries). Open them by path, since file search may skip ignored files. If a story is in progress, read its file first.
2. Skim `documentation/SPRINTS.md` to see which milestone is in progress.
3. If a request conflicts with the docs, point out the conflict before proposing changes.

## Project in brief

Sentinel is a touchscreen panel beside a Windows gaming PC. It shows hardware metrics and has one-touch launch buttons.

- **Sentinel Agent** (`agent/`): a C#/.NET tray app on Windows. It reads sensors (LibreHardwareMonitorLib), inspects the Steam library, and runs whitelisted actions.
- **Sentinel Panel** (`panel/`): a Rust + Slint fullscreen app on the device.
- **Link:** USB-C power plus USB networking. The panel listens and the agent dials out. Messages are length-prefixed Protobuf over TCP.

Design docs:

- `documentation/ARCHITECTURE.md`: components, protocol, security, failure modes
- `documentation/TECH_STACK.md`: technology choices and trade-offs
- `documentation/SPRINTS.md`: milestone roadmap

**Hardware is not chosen yet** (Raspberry Pi 4 + touch display, or a postmarketOS tablet). Don't hard-code OS assumptions such as init system or libc until it is chosen.

## Repo layout and naming

```
proto/           sentinel.proto (package sentinel.v1)
agent/           Sentinel.Agent C# solution (src/ + tests/), builds sentinel-agent.exe
panel/           sentinel-panel Rust crate + .slint UI files
tools/fake-agent stand-in agent for UI development
deploy/          init scripts, Task Scheduler config, device notes
documentation/   design docs
```

## Approval rules

- **No file changes without an explicit "yes."** This covers every file: code, tests, docs, config, CI, and `.context/`. Describe the proposed change first, then wait for the developer to answer "yes."
- **A "yes" covers only the change described.** If the work turns out to need anything more, stop and ask again.
- **Read-only actions need no approval:** reading files, searching, and running builds, tests, linters or `git status`/`git diff`/`git log`.
- **Never push to a remote, and never open PRs.** The developer is solely responsible for what goes to GitHub and is the only one who pushes. Commits are file changes, so they also need a "yes."

## How we work

- **This is a learning project.** The developer writes the code to learn. Explain the reasoning behind choices, and prefer guiding and reviewing over generating large amounts of code. When in doubt, ask whether he wants it written or explained.
- **One story at a time.** If you spot something outside the current story, suggest a new issue instead of doing it.
- **Test-driven:** write a failing test first, then make it pass. Bug fixes start with a test that reproduces the bug.
- **Never weaken, skip or delete a test** to make it pass without the developer's explicit OK.
- **Ask instead of guessing.** For anything uncertain, check the library's documentation or ask. Never invent an API.
- **Dependencies:** ask before adding any dependency (NuGet package, Rust crate, tool, GitHub Action). Every dependency is documented in `DEPENDENCIES.md` (create it when the first one is added) with what it is, why it's needed, and its license.
- **Small vertical slices:** each change delivers one thin piece end to end.
- **No timelines:** don't add dates, estimates or time boxes to docs or issues.
- **Issues** use the templates in `.github/ISSUE_TEMPLATE/`: Feature, Bug, Chore. Stories stay as lean as possible while still delivering value.
- **Branch and commit conventions:** _to be defined by the developer in a future iteration._

## Story lifecycle

- **A story is complete when it is merged into `main`** (which means it passed all CI checks). The developer will say when this has happened.
- **Before the developer opens the PR, check the docs for drift.** Review `README.md`, `CLAUDE.md`, `DEPENDENCIES.md` (if present), everything in `documentation/`, and the issue templates against the story's changes. Report what is out of date and propose minimal fixes (including ticking off roadmap items), then wait for a "yes." This keeps doc fixes in the same PR as the change.
- **When the story is complete, write its summary** to `.context/<issue#>-<short-slug>.md` (for example `.context/012-show-cpu-temp.md`): what changed, decisions made, anything learned, open items, and suggested next steps. This needs a "yes" like any other file change.
- **Pausing mid-story:** when the developer stops partway through, propose a short progress note in the same file: current state, next step, blockers.

## Rules that must not be broken

- **Protocol compatibility:** never reuse or renumber Protobuf field numbers; mark removed ones `reserved`. Add new features as new messages announced through capabilities.
- **No arbitrary execution:** the panel sends action IDs only. The agent maps IDs to commands defined on the PC, and message content never reaches a shell.
- **The agent opens no listening port.** It only dials out to the panel.
- **No pushes:** never push to a remote or open PRs.
- **No secrets in Git:** the pre-shared token and any keys stay out of the repo (see `.gitignore`).
- **The agent is a tray app, not a Windows Service.** Services can't launch apps on the user's desktop.

## Code conventions

- **C#:** `dotnet format` clean, nullable reference types enabled, xUnit tests. Put sensors and the filesystem behind interfaces (`ISensorSource`, `ISteamLibrary`) so they can be faked in tests.
- **Rust:** `cargo fmt` and `cargo clippy -- -D warnings` clean. Keep logic (state store, view models) free of UI dependencies so it is unit-testable. `Cargo.lock` is committed.
- **Errors:** handle and log them. The panel must never crash on a malformed message.

## Commands

_To be filled in during Milestone 0._

```
# agent
dotnet build agent/
dotnet test agent/

# panel
cargo build --manifest-path panel/Cargo.toml
cargo test --manifest-path panel/Cargo.toml
```
