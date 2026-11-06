# ADB Deployment — Sentinel Panel

How to build, install, and launch the Sentinel Panel APK on the Galaxy Tab A11.

## One-time tablet setup

1. **Enable Developer Options** — Settings → About tablet → Software information → tap **Build number** 7 times.
2. **Disable Auto Blocker** — Settings → Security and privacy → Auto Blocker → toggle off.
3. **Enable USB Debugging** — Settings → Developer options → USB Debugging → toggle on.
4. Connect the tablet to the Mac via USB-C. When the tablet prompts "Allow USB debugging?", tap **Allow**.

Confirm ADB sees the device:

```bash
adb devices
```

Expected output: a line like `R5GL85P7NJN    device`. If the status is `unauthorized`, re-plug the cable and accept the prompt on the tablet.

## Build the APK

From the repo root, set the required environment variables (these should be in `~/.zshrc` — run once per shell session if not):

```bash
export ANDROID_HOME="$HOME/android-sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/27.2.12479018"
export ANDROID_NDK_ROOT="$ANDROID_NDK_HOME"
export ANDROID_JAR="$ANDROID_HOME/platforms/android-34/android.jar"
export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH"
```

Then build:

```bash
cd panel && cargo apk build --lib
```

The signed APK lands at `panel/target/debug/apk/sentinel-panel.apk`.

## Install and launch

```bash
adb install -r target/debug/apk/sentinel-panel.apk
adb shell am start -n "com.sentinel.panel/android.app.NativeActivity"
```

The `-r` flag reinstalls over an existing version, preserving app data.

The panel launches showing "Waiting for PC".

## Connect the fake-agent for testing

The tablet is a separate network namespace when connected via USB debugging. Forward the panel's listen port so the fake-agent on the Mac can reach it:

```bash
adb forward tcp:9000 tcp:9000
```

Then start the fake-agent from the repo root:

```bash
cargo run --manifest-path tools/fake-agent/Cargo.toml
```

The panel transitions from "Waiting for PC" to the live metrics dashboard.

## Notes

- The app runs in immersive sticky mode — system bars are hidden but reappear on swipe from the edge.
- For the production link (agent → panel over USB networking), the tablet must be plugged into the Windows PC, not the Mac. See `deploy/usb-networking.md`.
- ADB debugging and USB tethering use the same physical cable but are mutually exclusive connection modes — swap as needed.
