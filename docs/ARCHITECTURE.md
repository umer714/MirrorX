# MirrorX — Initial Architecture

## 1. Product boundary

MirrorX is a desktop interface for connecting to and interacting with Android devices. The first release target is Windows and the first connection path is USB.

The desktop application should orchestrate local tools. The initial design does not require a cloud backend for screen mirroring.

## 2. High-level flow

```text
React + TypeScript UI
        |
        | Tauri commands / events
        v
Rust application core
        |
        +---- ADB: discover and inspect Android devices
        |
        +---- scrcpy: start and manage mirroring sessions
        |
        v
Android device (USB initially)
```

## 3. Responsibilities

### Desktop UI

- Show detected devices and their connection state.
- Let the user start and stop a session.
- Display understandable errors and troubleshooting guidance.
- Keep controls disabled when the required device state is unavailable.

### Rust core

- Validate requested actions.
- Locate and launch required local executables.
- Manage child-process lifecycle and shutdown.
- Stream structured status events to the UI.
- Avoid shell-string construction from untrusted input; use argument arrays and validate device identifiers.

### ADB

- Discover attached devices.
- Read device state.
- Select a device explicitly when more than one is connected.
- Report authorization and connection errors without attempting to bypass Android security prompts.

### scrcpy

- Provide the screen mirroring engine and supported interaction features.
- Be invoked as a managed local process.
- Have its version and license notices documented for every packaged release.

## 4. Security and privacy

- USB is the default connection path for the first prototype.
- Wireless debugging must be opt-in and explain the local-network implications.
- Do not send screen frames, device identifiers, or phone data to a remote service by default.
- Do not store ADB pairing credentials or other secrets in source control.
- Treat device names and command output as untrusted display data.
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
