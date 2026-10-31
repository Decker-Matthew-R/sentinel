# Dev Setup — Sentinel

Step-by-step guide to getting a working local build on a fresh machine.

> **Platform:** These instructions are written for **macOS**. Windows setup instructions will be added in a future story.

---

## Prerequisites

### 1. Homebrew

[Homebrew](https://brew.sh) is the package manager used to install the remaining tools. If you don't have it:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Follow the prompts. When it finishes, follow any instructions it prints to add Homebrew to your PATH (Apple Silicon Macs require an extra step).

Verify:

```bash
brew --version
```

### 2. .NET SDK

The project targets [.NET 9](https://dotnet.microsoft.com/download/dotnet/9.0). Install it via Homebrew:

```bash
brew install dotnet@9
```

Add it to your PATH (add this line to your `~/.zshrc` or `~/.bashrc`):

```bash
export PATH="/opt/homebrew/opt/dotnet@9/bin:$PATH"
```

Reload your shell and verify:

```bash
dotnet --version
```

The version should match or exceed the one in `global.json` at the repo root. See [Toolchain versions](TECH_STACK.md#toolchain-versions).

### 3. gitleaks

[gitleaks](https://github.com/gitleaks/gitleaks) scans commits for secrets and credentials. The CI pipeline runs this on every PR; the local gates check runs it before you push.

```bash
brew install gitleaks
```

Verify:

```bash
gitleaks version
```

> If gitleaks is not installed, `ci-gates-check.sh` skips the secret scan with a warning rather than failing. Install it to get full parity with CI.

### 4. Protocol Buffers compiler

The Rust panel build uses [prost-build](https://docs.rs/prost-build) to generate Rust types from the `.proto` schema at compile time. It requires `protoc` (the Protobuf compiler) to be on your PATH:

```bash
brew install protobuf
```

Verify:

```bash
protoc --version
```

> The C# agent does not need this — `Grpc.Tools` bundles its own `protoc` for Windows, Linux, and macOS.

### 5. Rust

[Rust](https://www.rust-lang.org) is installed via [rustup](https://rustup.rs), the official toolchain manager:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the prompts and choose the default installation. When it finishes, restart your terminal (or run `source ~/.cargo/env`) and verify:

```bash
rustc --version
cargo --version
```

The project pins the Rust version in `panel/rust-toolchain.toml`. The first time you build, `rustup` will automatically download the pinned version along with the `rustfmt` and `clippy` components.

---

## Clone the repo

```bash
git clone https://github.com/Decker-Matthew-R/sentinel.git
cd sentinel
```

---

## Build

**Agent (C#):**

```bash
dotnet build agent/
```

**Panel (Rust):**

```bash
cargo build --manifest-path panel/Cargo.toml
```

The first panel build downloads and compiles all Rust dependencies — this takes a few minutes. Subsequent builds are fast.

---

## Run tests

**Agent:**

```bash
dotnet test agent/
```

**Panel:**

```bash
cargo test --manifest-path panel/Cargo.toml
```

Both should report all tests passing on a clean checkout.

---

## Launch the dev environment

To open the panel UI and connect the fake-agent in one step:

```bash
./tools/scripts/dev-launch.sh
```

This builds both crates, opens the panel window in the current terminal, and launches the fake-agent in a new Terminal window. The fake-agent connects automatically once the panel is ready and begins streaming metric data. Use this during UI development instead of hardware.

To launch the panel alone (no fake-agent):

```bash
cargo run --manifest-path panel/Cargo.toml
```

### Fake data scenarios

The fake-agent cycles through five named scenarios on an 8-second loop (40 seconds total), designed to exercise every visual state of the dashboard:

| Scenario | CPU % | CPU °C | GPU % | GPU °C | Hotspot °C | VRAM | RAM |
|---|---|---|---|---|---|---|---|
| **Idle** | 6 | 38 | 3 | 32 | 38 | 1.2 / 16 GB | 8.5 / 32 GB |
| **Gaming — Normal** | 45 | 62 | 78 | 72 | 84 | 8.5 / 16 GB | 14.2 / 32 GB |
| **Gaming — Warm** | 72 | 74 | 92 | 82 | 93 | 12.0 / 16 GB | 22.1 / 32 GB |
| **Gaming — Critical** | 95 | 91 | 99 | 93 | 105 | 15.8 / 16 GB | 28.4 / 32 GB |
| **No GPU** | 25 | 48 | — | — | — | — | 9.2 / 32 GB |

The warm and critical scenarios are tuned to straddle the threshold values used by the `ViewModels` layer, so UI threshold colours can be validated without hardware. The "No GPU" scenario drives all GPU and VRAM tiles to `--`, verifying that absent-sensor handling renders correctly.

The terminal window running the fake-agent prints the active scenario and tick number so you can follow along:

```
[Gaming — Critical] tick 3/8 — uptime 26s
```

---

## Android build (tablet deployment)

> **Note:** The Android build toolchain setup is covered by story #100. The steps below will be filled in once that story is complete. For local development and testing, the desktop build above is sufficient — the fake-agent connects to the panel running as a desktop window.

Requirements once #100 is done:
- Android SDK (via Android Studio or command-line tools)
- Android NDK (version to be pinned in #100)
- `cargo-ndk` Rust tool
- `aarch64-linux-android` Rust target

---

## Format and lint

Before pushing, run the CI gates check script — it auto-formats both halves, verifies linting, and runs all tests in one step:

```bash
./scripts/ci-gates-check.sh
```

This mirrors exactly what the CI pipeline enforces and gives you a fast local feedback loop before opening a PR. See [Dev Norms](DEV_NORMS.md) for the full workflow.

If you prefer to run steps individually:

**Agent:**

```bash
dotnet format agent/
```

**Panel:**

```bash
cargo fmt --manifest-path panel/Cargo.toml
cargo clippy --manifest-path panel/Cargo.toml -- -D warnings
```

