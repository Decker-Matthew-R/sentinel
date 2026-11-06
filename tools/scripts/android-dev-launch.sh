#!/usr/bin/env bash
set -euo pipefail

# Starts the Android emulator, waits for boot, port-forwards, installs and
# launches the panel APK, then connects the fake-agent — all in one step.
#
# Requires: cargo-apk, Android SDK with emulator + system-images, AVD named
# 'sentinel'. See DEV_SETUP.md (Android emulator section) for setup steps.
#
# If you have multiple SDK installs (e.g. Android Studio), set
# SENTINEL_ANDROID_HOME in your shell to point to the right one:
#   export SENTINEL_ANDROID_HOME="$HOME/android-sdk"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

ANDROID_HOME="${SENTINEL_ANDROID_HOME:-${ANDROID_HOME:-$HOME/android-sdk}}"
export ANDROID_HOME
export ANDROID_NDK_ROOT="$ANDROID_HOME/ndk/27.2.12479018"

ADB="$ANDROID_HOME/platform-tools/adb"
EMULATOR="$ANDROID_HOME/emulator/emulator"
AVD_NAME="sentinel"
PANEL_PORT=9000

# ── Preflight ─────────────────────────────────────────────────────────────────
if [ ! -f "$EMULATOR" ]; then
    echo "Error: emulator not found at $EMULATOR"
    echo "Set SENTINEL_ANDROID_HOME or follow the emulator setup in DEV_SETUP.md."
    exit 1
fi

if [ ! -d "$HOME/.android/avd/${AVD_NAME}.avd" ]; then
    echo "Error: AVD '${AVD_NAME}' not found."
    echo "Follow the AVD creation steps in DEV_SETUP.md."
    exit 1
fi

if ! command -v cargo-apk &> /dev/null; then
    echo "Error: cargo-apk not installed. Run: cargo install cargo-apk"
    exit 1
fi

# ── Start emulator ────────────────────────────────────────────────────────────
echo "Starting Android emulator (${AVD_NAME})..."
osascript -e "tell application \"Terminal\" to do script \"ANDROID_HOME='$ANDROID_HOME' $EMULATOR -avd $AVD_NAME\""

# ── Wait for emulator to fully boot ──────────────────────────────────────────
echo "Waiting for emulator to boot (this takes 30–60 seconds)..."
"$ADB" wait-for-device
until [ "$("$ADB" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = "1" ]; do
    sleep 2
done
echo "Emulator ready."

# ── Port forward ──────────────────────────────────────────────────────────────
echo "Setting up ADB port forwarding (host:${PANEL_PORT} → emulator:${PANEL_PORT})..."
"$ADB" forward tcp:$PANEL_PORT tcp:$PANEL_PORT

# ── Build and install APK ─────────────────────────────────────────────────────
echo "Building APK..."
(cd "$REPO_ROOT/panel" && cargo apk build --lib)

echo "Installing APK..."
"$ADB" install -r "$REPO_ROOT/panel/target/debug/apk/sentinel-panel.apk"

echo "Launching panel..."
"$ADB" shell am start -n "com.sentinel.panel/android.app.NativeActivity"
echo "Panel launched on emulator."

# ── Start fake-agent ──────────────────────────────────────────────────────────
echo "Opening fake-agent in a new Terminal window..."
osascript -e "tell application \"Terminal\" to do script \"until nc -z 127.0.0.1 $PANEL_PORT 2>/dev/null; do sleep 0.1; done && cd '$REPO_ROOT' && cargo run --manifest-path tools/fake-agent/Cargo.toml\""

echo "Done. The fake-agent will connect once the panel is listening on port ${PANEL_PORT}."
