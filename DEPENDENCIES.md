# Dependencies

## C#

| Package | Version | Used in | Why | License |
|---|---|---|---|---|
| `Google.Protobuf` | 3.36.2 | `Sentinel.Agent` | Protobuf runtime — serialization and deserialization of generated message types | BSD 3-Clause |
| `Grpc.Tools` | 2.85.0 | `Sentinel.Agent` | Build-time code generator; compiles `sentinel.proto` into C# classes at `dotnet build` time. Bundles its own `protoc` — no system install needed | Apache 2.0 |
| `xunit` | 2.9.2 | `Sentinel.Agent.Tests` | Test framework | Apache 2.0 |
| `xunit.runner.visualstudio` | 2.8.2 | `Sentinel.Agent.Tests` | Allows `dotnet test` to discover and run xUnit tests | MIT |
| `Microsoft.NET.Test.Sdk` | 17.12.0 | `Sentinel.Agent.Tests` | Required infrastructure for `dotnet test` | MIT |
| `coverlet.collector` | 6.0.2 | `Sentinel.Agent.Tests` | Code coverage collection during test runs | MIT |

## Rust

| Crate | Version | Used in | Why | License |
|---|---|---|---|---|
| `prost` | 0.13.5 | `sentinel-panel`, `fake-agent` | Protobuf runtime — serialization and deserialization of generated message types | Apache 2.0 |
| `tokio` | 1.54.1 | `sentinel-panel`, `fake-agent` | Async runtime — TCP networking, async I/O traits used by the framing layer and LinkServer | MIT |
| `prost-build` | 0.13.5 | `sentinel-panel` (build), `fake-agent` (build) | Compiles `sentinel.proto` into Rust structs at `cargo build` time. Requires system `protoc` | Apache 2.0 |
| `slint` | 1.18.1 | `sentinel-panel` | UI framework for the panel's fullscreen dashboard | GPL v3 |
| `slint-build` | 1.18.1 | `sentinel-panel` (build) | Compiles `.slint` UI files into Rust code at build time | GPL v3 |
| `i-slint-backend-testing` | 1.18 | `sentinel-panel` (dev) | Headless Slint backend for UI component tests; lets `AppWindow` be created and asserted against without a display | GPL v3 |

## CLI Tools

| Tool | Version | Used in | Why | License |
|---|---|---|---|---|
| `gitleaks` | latest | `ci-gates-check.sh` | Scans commits for secrets and credentials locally, matching what `secret-scan.yml` enforces in CI | MIT |
| `protoc` | latest | `ci-gates-check.sh`, `panel-gates.yml` | Compiles `sentinel.proto` at build time for the Rust panel (via `prost-build`). Install via `brew install protobuf` | BSD 3-Clause |
| `cargo-audit` | 0.22.2 | `ci-gates-check.sh`, `panel-gates.yml`, `scheduled-cve-scan.yml` | Scans `Cargo.lock` against the RustSec advisory database for known CVEs | Apache 2.0 / MIT |
| `cargo-ndk` | latest | `ci-gates-check.sh`, `panel-gates.yml` | Cross-compiles the Rust panel for Android (`aarch64-linux-android`) by wiring up the Android NDK linker and sysroot | MIT |

## GitHub Actions

| Action | Version | Used in | Why | License |
|---|---|---|---|---|
| `actions/checkout` | v4 | `move-to-in-progress.yml`, `agent-gates.yml`, `panel-gates.yml`, `secret-scan.yml`, `scheduled-cve-scan.yml` | Checks out the repo so workflows can access repo files | MIT |
| `actions/setup-dotnet` | v4 | `agent-gates.yml`, `scheduled-cve-scan.yml` | Installs the .NET SDK version pinned in `global.json` on the CI runner | MIT |
| `Swatinem/rust-cache` | v2 | `panel-gates.yml`, `scheduled-cve-scan.yml` | Caches Cargo registry and build artifacts to speed up Rust CI runs | MIT |
| `gitleaks/gitleaks-action` | v2 | `secret-scan.yml` | Scans PR changesets for secrets and credentials; fails the check if any are found | MIT |
| `actions/upload-artifact` | v4 | `panel-gates.yml` | Uploads the Android build artifact (`.so` / binary) so it can be downloaded from the CI run | MIT |
