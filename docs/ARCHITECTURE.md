# MirrorX — Initial Architecture

## 1. Product boundary

MirrorX is a desktop interface for connecting to and interacting with Android devices. The first release target is Windows and the first connection path is USB.

The desktop application orchestrates local tools. The initial design does not require a cloud backend for screen mirroring.

## 2. High-level flow

```text
React + TypeScript UI
        |
        | Tauri invoke command
        v
Rust application core
        |
        +---- ADB: list and inspect Android devices
        |
        +---- scrcpy: planned session manager
        |
        v
Android device (USB initially)
```

## 3. Responsibilities

### Desktop UI

- Show detected devices and their connection state.
- Let the user start and stop a session when that feature is implemented.
- Display understandable errors and troubleshooting guidance.
- Keep controls disabled when the required device state is unavailable.

### Rust core

- Validate requested actions.
- Launch local executables using argument arrays, not shell command strings.
- Manage child-process lifecycle and shutdown.
- Return structured device information to the UI.
- Treat command output and device names as untrusted display data.

### ADB device discovery (first prototype)

The `list_android_devices` Tauri command runs `adb devices -l` locally. It parses serial, state, model, product, and device fields. It does not change device settings, bypass authorization, or transmit device information to a server.

The frontend displays authorized and unauthorized states. If ADB is not installed or not available in PATH, the UI reports an actionable error. This prototype still needs a Windows build and real-device testing.

### scrcpy (planned)

- Provide the screen mirroring engine and supported interaction features.
- Be invoked as a managed local process.
- Have its version and license notices documented for every packaged release.

## 4. Security and privacy

- USB is the default connection path for the first prototype.
- Wireless debugging must be opt-in and explain local-network implications.
- Do not send screen frames, device identifiers, or phone data to a remote service by default.
- Do not store ADB pairing credentials or other secrets in source control.
- Validate device identifiers before passing them to later commands.
- Stop child processes cleanly when a session ends or the app exits.
- Do not attempt to bypass Android authorization or device security controls.

## 5. Cloud services

Vercel may host a public website, documentation, and release information. It is not part of the live screen transport path.

Supabase is not required for local mirroring. It may be considered later for optional accounts, license entitlements, or update metadata, with data collection documented and minimized.

## 6. Decisions still open

- Exact scrcpy distribution and update strategy.
- Whether scrcpy binaries are bundled or separately installed.
- MirrorX's own source-code license.
- Update/signing strategy for Windows installers.
- Supported Windows versions and minimum hardware requirements.
