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

> **Note:** The Android build is only needed to deploy to the Galaxy Tab A11. For local development and testing, the desktop build above is sufficient — the fake-agent connects to the panel running as a desktop window.

### 1. Android command-line tools

Go to [developer.android.com/studio#command-line-tools-only](https://developer.android.com/studio#command-line-tools-only) and download the macOS package. Then unzip it into a new SDK directory (the `latest` subdirectory name is required by `sdkmanager`):

```bash
mkdir -p ~/android-sdk/cmdline-tools/latest
unzip ~/Downloads/commandlinetools-mac_*.zip -d /tmp/cmdline-tools
mv /tmp/cmdline-tools/cmdline-tools/* ~/android-sdk/cmdline-tools/latest/
```

Add to your `~/.zshrc`:

```bash
export ANDROID_HOME="$HOME/android-sdk"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/27.2.12479018"
export ANDROID_NDK_ROOT="$ANDROID_NDK_HOME"
export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$PATH"
```

Reload your shell and verify:

```bash
source ~/.zshrc
sdkmanager --version
```

### 2. Android NDK r27c

```bash
sdkmanager "ndk;27.2.12479018"
```

Verify:

```bash
ls $ANDROID_NDK_HOME
```

### 3. cargo-ndk

```bash
cargo install cargo-ndk
```

Verify:

```bash
cargo ndk --version
```

### 4. aarch64-linux-android Rust target

```bash
rustup target add aarch64-linux-android
```

### Build

```bash
cd panel
cargo ndk -t arm64-v8a build
```

The binary lands at `panel/target/aarch64-linux-android/debug/sentinel-panel`, confirming the Rust code compiles for Android.

---

## Android emulator (optional — Apple Silicon only)

> **Note:** The ARM64 Android system image runs natively only on Apple Silicon (M1/M2/M3/M4). On Intel Macs the emulator falls back to x86_64 and cannot run an arm64-v8a APK. Skip this section if you are on Intel hardware.

Use this workflow to confirm the panel looks correct on Android without deploying to the physical tablet. For day-to-day iteration, the desktop build is faster.

Once the one-time setup below is complete, the full workflow can be launched in one step from the repo root:

```bash
./tools/scripts/android-dev-launch.sh
```

This starts the emulator, waits for boot, port-forwards, installs the APK, and connects the fake-agent automatically. If you have Android Studio installed, add this to your `~/.zshrc` so the script uses the right SDK instead of Android Studio's:

```bash
export SENTINEL_ANDROID_HOME="$HOME/android-sdk"
```

### 1. cargo-apk

`cargo-apk` bundles the compiled `cdylib` into an installable APK. It reads the `[package.metadata.android]` section from `panel/Cargo.toml` for the package name and SDK targets.

```bash
cargo install cargo-apk
```

Verify:

```bash
cargo apk version
```

### 2. Android emulator and system image

```bash
sdkmanager "emulator" "platform-tools" "platforms;android-34" \
           "system-images;android-34;google_apis;arm64-v8a" \
           "build-tools;34.0.0"
```

Create an AVD named `sentinel`:

```bash
avdmanager create avd -n sentinel \
  -k "system-images;android-34;google_apis;arm64-v8a"
```

When asked "Do you wish to create a custom hardware profile?", answer `no`.

Then set the screen dimensions to match the Galaxy Tab A11 in portrait mode (1200×1900 @ 206 dpi). The interactive setup leaves low-res defaults, so replace them:

```bash
sed -i '' '/^hw\.lcd\.width=/d' ~/.android/avd/sentinel.avd/config.ini
sed -i '' '/^hw\.lcd\.height=/d' ~/.android/avd/sentinel.avd/config.ini
sed -i '' '/^hw\.lcd\.density=/d' ~/.android/avd/sentinel.avd/config.ini
cat >> ~/.android/avd/sentinel.avd/config.ini <<'EOF'
hw.lcd.width=1200
hw.lcd.height=1900
hw.lcd.density=206
hw.initialOrientation=portrait
EOF
```

Verify:

```bash
grep hw.lcd ~/.android/avd/sentinel.avd/config.ini
```

You should see `width=1200`, `height=1900`, `density=206` with no other values for those three keys.

Add the emulator and platform-tools to your PATH (add to `~/.zshrc`):

```bash
export PATH="$ANDROID_HOME/emulator:$ANDROID_HOME/platform-tools:$PATH"
```

### 3. Start the emulator

Run the emulator in a separate terminal (or background it with `&`). Use the full path to avoid conflicts if Android Studio is also installed:

```bash
$ANDROID_HOME/emulator/emulator -avd sentinel
```

Wait 30–60 seconds for Android to finish booting. Then confirm the device is ready:

```bash
adb devices
```

You should see a line like `emulator-5554   device`. If it shows `offline`, wait a few more seconds and run it again. Do not proceed to the next step until the status is `device`.

### 4. Build and deploy the APK

From the `panel/` directory, build the APK, install it, and launch it:

```bash
cd panel
cargo apk build --lib
adb install -r target/debug/apk/sentinel-panel.apk
adb shell am start -n "com.sentinel.panel/android.app.NativeActivity"
```

The panel appears on the emulator showing "Waiting for PC".

> Note: use `cargo apk build --lib` rather than `cargo apk run --lib` here — `run` blocks the terminal streaming device output, preventing the next steps from running.

### 5. Connect the fake-agent

The emulator is a separate network namespace. Forward the panel's listen port so `fake-agent` can reach it:

```bash
adb forward tcp:9000 tcp:9000
```

Then start the fake-agent from the repo root:

```bash
cargo run --manifest-path tools/fake-agent/Cargo.toml
```

The panel should transition from "Waiting for PC" to the live metrics dashboard. The fake-agent cycles through the same scenarios as in the desktop workflow.

---

## Physical device deployment

To deploy to the Galaxy Tab A11 instead of the emulator, see [`deploy/adb-deploy.md`](../deploy/adb-deploy.md) for the full step-by-step guide covering:

- One-time tablet setup (Developer Options, USB Debugging, Auto Blocker)
- Building and installing the APK via ADB
- Connecting the fake-agent over a forwarded port for on-device testing

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

