# Dependencies

## C#

| Package | Version | Used in | Why | License |
|---|---|---|---|---|
| `xunit` | 2.9.2 | `Sentinel.Agent.Tests` | Test framework | Apache 2.0 |
| `xunit.runner.visualstudio` | 2.8.2 | `Sentinel.Agent.Tests` | Allows `dotnet test` to discover and run xUnit tests | MIT |
| `Microsoft.NET.Test.Sdk` | 17.12.0 | `Sentinel.Agent.Tests` | Required infrastructure for `dotnet test` | MIT |
| `coverlet.collector` | 6.0.2 | `Sentinel.Agent.Tests` | Code coverage collection during test runs | MIT |

## Rust

| Crate | Version | Used in | Why | License |
|---|---|---|---|---|
| `slint` | 1.18.1 | `sentinel-panel` | UI framework for the panel's fullscreen dashboard | GPL v3 |
| `slint-build` | 1.18.1 | `sentinel-panel` (build) | Compiles `.slint` UI files into Rust code at build time | GPL v3 |
| `cargo-audit` | 0.22.2 | `panel-gates.yml`, `ci-gates-check.sh` | Scans `Cargo.lock` against the RustSec advisory database for known CVEs | Apache 2.0 / MIT |

## GitHub Actions

| Action | Version | Used in | Why | License |
|---|---|---|---|---|
| `actions/checkout` | v4 | `move-to-in-progress.yml`, `agent-gates.yml`, `panel-gates.yml` | Checks out the repo so workflows can access repo files | MIT |
| `actions/setup-dotnet` | v4 | `agent-gates.yml` | Installs the .NET SDK version pinned in `global.json` on the CI runner | MIT |
| `Swatinem/rust-cache` | v2 | `panel-gates.yml` | Caches Cargo registry and build artifacts to speed up Rust CI runs | MIT |
