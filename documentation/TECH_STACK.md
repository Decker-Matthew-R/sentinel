# Tech Stack — Sentinel

> Status: **Proposed**. Each choice lists its pros, cons, and the alternatives considered, so future-us knows *why*.

## Stack at a glance

```mermaid
flowchart TB
    subgraph PCSIDE["Windows PC"]
        direction TB
        a1["C# / .NET — Generic Host, DI, BackgroundServices"]
        a2["LibreHardwareMonitorLib — CPU/GPU sensors"]
        a3["Google.Protobuf — generated messages"]
        a4["WinForms NotifyIcon — tray app"]
        a5["Task Scheduler — start at logon, elevated"]
        a6["xUnit — tests"]
    end

    subgraph LINK["Link"]
        direction TB
        l1["USB-C cable — power + data"]
        l2["Android USB tethering (RNDIS)"]
        l3["TCP + length-prefixed Protobuf"]
    end

    subgraph TABSIDE["Tablet"]
        direction TB
        t1["Android (Samsung Galaxy Tab A11)"]
        t2["Rust — tokio for networking"]
        t3["Slint — Android backend"]
        t4["prost — generated messages"]
        t5["cargo test + Slint testing backend"]
    end

    subgraph TOOLS["Tooling"]
        direction TB
        x1["Monorepo — proto / agent / panel / tools / deploy"]
        x2["GitHub Actions — build, test, release"]
        x3["Android NDK + cargo-ndk — aarch64-android builds"]
        x4["Fake agent — UI dev without hardware"]
    end

    PCSIDE --- LINK --- TABSIDE
    TOOLS -.-> PCSIDE
    TOOLS -.-> TABSIDE
```

## Toolchain versions

| Tool | Pinned version | Config file |
|---|---|---|
| .NET SDK | 9.0.306 | `global.json` (repo root) |
| Rust | 1.99.1 | `panel/rust-toolchain.toml` |
| Slint | 1.18.1 | `panel/Cargo.toml` |

To upgrade: update the version in the config file, run a local build to verify, then open a chore PR.

## Decisions

### 1. Hardware: Samsung Galaxy Tab A11 (Android)

**Chosen.** Samsung Galaxy Tab A11 running stock Android with the Sentinel APK deployed via ADB.

| Pros | Cons |
|---|---|
| Android USB tethering (RNDIS) enables single-cable power + data to the PC | USB tethering must be enabled manually on first setup |
| No bootloader unlock or custom OS — ships ready to use | Less low-level OS control than a custom Linux install |
| Samsung battery protection mode limits charging to 85% for always-plugged health | Samsung One UI adds some Android customisation overhead |
| Widely available, strong hardware for the price | |

**Alternatives considered:**
- *postmarketOS tablet (Xiaomi Pad 5 / PineTab2)* — more Linux control, but requires bootloader unlock, flashing, and community-supported drivers. Ruled out in favour of a working device with a reliable Android ecosystem.
- *Surface Go (x86 + linux-surface)* — excellent Linux support, but USB ports are host-only (no gadget/tethering mode) and picky about low-power USB-C charging.

### 2. Link: USB-C with Android USB tethering (RNDIS)

| Pros | Cons |
|---|---|
| One cable for power and data; no outlet needed | PC USB-C ports supply limited power (often 7.5–15 W) |
| No Wi-Fi or Bluetooth pairing; low latency, very reliable | Tablet battery stays plugged in permanently |
| Android USB tethering shows up on Windows as a normal network adapter | USB tethering must be enabled in Android settings |
| Plain TCP sockets: transferable networking knowledge | |

**Alternatives considered:**
- *Wi-Fi/LAN* — works with any device, but needs a separate power source anyway.
- *Bluetooth (BLE)* — no cable for data, but flaky pairing/reconnects and awkward Windows APIs. Kept as a **stretch goal** (second transport).
- *USB HID (tablet as keyboard)* — can send hotkeys but cannot receive metrics.

### 3. Connection direction: tablet listens, agent dials out

| Pros | Cons |
|---|---|
| PC exposes no listening port (smaller attack surface) | Reconnect logic lives in the agent, which must handle the tablet rebooting |
| Tablet address on the USB link is fixed — no discovery needed | Slightly less conventional than "PC is the server" |

### 4. Protocol: length-prefixed Protobuf over TCP

| Pros | Cons |
|---|---|
| One schema, generated code for both languages | Binary format is harder to debug than JSON (mitigate with a debug dump tool) |
| Strongly typed, compact, handles binary payloads (cover art) | Must design framing ourselves |
| Built-in forward/backward compatibility rules | Code generation step in both builds |
| Good learning: framing, serialization, versioning | |

**Alternatives considered:** JSON over WebSocket (easier to debug, weaker typing, base64 for images); gRPC (heavier, HTTP/2 machinery is overkill on a point-to-point link).

### 5. PC agent: C# / .NET

| Pros | Cons |
|---|---|
| LibreHardwareMonitorLib is a native .NET library and handles the kernel driver needed for CPU temps on Windows | A second language in the project |
| First-class Windows APIs (Core Audio, media sessions, networking) for future features | Single-file self-contained exe is relatively large |
| Generic Host + DI make clean, testable services | |
| Mature tooling and testing (xUnit) | |

**Alternatives considered:** Rust everywhere (one language, but CPU temps on Windows require a kernel driver — painful without LHM); Python (fast to write, weaker for a long-running elevated tray app).

### 6. Agent hosting: tray app at logon (not a Windows Service)

| Pros | Cons |
|---|---|
| Runs in the user's session, so it **can** open Steam and other apps on the desktop | Only runs while a user is logged in (fine for this use case) |
| Visible status via tray icon | Elevated start needs a Task Scheduler entry |

### 7. Tablet app: Rust

| Pros | Cons |
|---|---|
| Single small native binary, no runtime | Steeper learning curve (ownership, async) |
| Memory safety for a long-running appliance | Android NDK cross-compile adds build setup |
| tokio is a mature async networking stack | |
| Strong learning value | |

### 8. UI toolkit: Slint (Android backend)

| Pros | Cons |
|---|---|
| Declarative `.slint` markup, good for polished UIs | Smaller ecosystem than Qt |
| Same app runs as a desktop window on macOS/Windows for fast UI iteration | Fewer ready-made chart widgets (we build sparklines ourselves) |
| Android backend renders via Android's native surface — no compositor needed | APK packaging requires a thin Java `Activity` shim |

**Fallback:** Kotlin + Jetpack Compose — native Android UI, larger ecosystem, but requires rewriting all existing Rust panel code.

### 9. Repo and CI: monorepo + GitHub Actions

| Pros | Cons |
|---|---|
| Protocol changes and both consumers change in one PR | CI must build two toolchains |
| Single place for docs, issues, releases | Larger repo |
| Actions can run tests, build the Android APK, and publish release artifacts | Self-hosted runner needed if we ever want automatic ADB deploy to the tablet |

## Proposed repo layout

```
sentinel/
├── proto/            # sentinel.proto — single source of truth for the protocol
├── agent/            # Sentinel.Agent — C# solution (src/ + tests/)
├── panel/            # sentinel-panel — Rust crate + .slint UI files
├── tools/
│   └── fake-agent/   # streams fake metrics for UI development
├── deploy/           # Task Scheduler XML, ADB deploy notes, device notes
├── documentation/    # ARCHITECTURE.md, TECH_STACK.md, SPRINTS.md
└── .github/workflows/
```

## Risks to watch

| Risk | Impact | Mitigation |
|---|---|---|
| Slint Android backend rendering issues | UI glitches or won't render | Fall back to Kotlin + Jetpack Compose (full rewrite) |
| Samsung battery protection mode unreliable | Battery wear / swelling over time | Lower brightness; use an external USB power limiter |
| Insufficient USB power from PC | Tablet slowly drains | Use the PC's strongest USB-C header; dim the screen |
| LHM sensor names differ by hardware | Missing or mislabeled metrics | Map sensors by type, not name; hide missing tiles |
