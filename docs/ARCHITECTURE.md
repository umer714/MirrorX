# MirrorX — Initial Architecture

## 1. Product boundary

MirrorX is a Windows-first desktop interface for connecting to and interacting with Android devices. USB is the first connection path.

The desktop application orchestrates local tools. Screen frames are not routed through a cloud backend.

## 2. High-level flow

```text
React + TypeScript UI
        |
        | Tauri invoke commands
        v
Rust application core
        |
        +---- ADB: list and verify Android devices
        |
        +---- scrcpy: managed local child process
        |
        v
Android device (USB initially)

scrcpy currently opens its own desktop window.
Embedding that video surface in MirrorX is a future milestone.
```

## 3. Responsibilities

### Desktop UI

- Show detected devices and their connection state.
- Allow explicit selection of an authorized device.
- Start and stop the scrcpy process launched by MirrorX.
- Display understandable errors and troubleshooting guidance.

### Rust core

- Validate device identifiers before using them.
- Launch local executables with argument arrays, never shell command strings.
- Verify the selected device is in ADB's authorized `device` state before starting scrcpy.
- Keep the child-process handle and provide a stop command.
- Return structured errors without exposing phone contents.

### ADB device discovery

The `list_android_devices` command runs `adb devices -l` locally. It parses serial, state, model, product, and device fields. It does not change device settings, bypass authorization, or transmit device information to a server.

### scrcpy process prototype

The `start_scrcpy` command verifies the selected serial with ADB and launches `scrcpy --serial <serial>` as a managed child process. The `stop_scrcpy` command terminates that child process. The scrcpy window is separate from MirrorX at this stage; the video surface has not been embedded.

Both `adb.exe` and `scrcpy.exe` must be installed and discoverable through PATH. The native process integration still needs a successful Windows build and real-device testing.

## 4. Security and privacy

- USB is the default connection path.
- Device identifiers are restricted to a conservative character set and passed as process arguments.
- Never build shell command strings from UI input.
- Wireless debugging must be opt-in and explain local-network implications.
- Do not send screen frames, device identifiers, or phone data to a remote service by default.
- Do not attempt to bypass Android authorization or device security controls.

## 5. Cloud services

Vercel may host a public website, documentation, and release information. It is not part of the live screen transport path.

Supabase is not required for local mirroring. It may be considered later for optional accounts or license entitlements with data collection documented and minimized.

## 6. Decisions still open

- Exact scrcpy version and distribution strategy.
- Whether scrcpy binaries are bundled or separately installed.
- Embedding the scrcpy video surface in the Tauri window.
- MirrorX's own source-code license.
- Update/signing strategy for Windows installers.
