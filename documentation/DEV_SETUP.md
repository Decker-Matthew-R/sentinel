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

### 3. Rust

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

## Launch the Slint window

To open the panel UI on your desktop:

```bash
cargo run --manifest-path panel/Cargo.toml
```

A window will appear titled **Sentinel**. This is the development entry point for the panel UI — no hardware required.

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

