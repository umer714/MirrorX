# MirrorX — Development Setup Notes

The repository contains a React + TypeScript frontend, a Tauri 2 / Rust desktop shell, local ADB device discovery, and an initial scrcpy process controller.

## Windows prerequisites

Install and verify:

- Node.js LTS and npm.
- Rust stable toolchain.
- Microsoft C++ Build Tools with the Windows SDK.
- Tauri 2 prerequisites for Windows.
- Android Platform Tools (ADB).
- scrcpy for Windows.

Make sure `adb.exe` and `scrcpy.exe` are available in PATH, then restart PowerShell and the desktop app.

## Verify tools

```powershell
node --version
npm --version
rustc --version
cargo --version
adb version
scrcpy --version
```

## Build and run

From the repository root:

```powershell
npm install
npm run build
npm run tauri dev
```

The USB device screen invokes ADB locally. Connect a phone with USB debugging enabled and approve the computer authorization prompt.

Select the authorized device and choose **Start mirroring**. scrcpy opens in a separate desktop window. Use **Stop session** in MirrorX to stop the process launched by the app.

The public website cannot access a computer's USB devices or launch local executables.

## Current testing status

Rust unit tests cover ADB output parsing and device-serial validation. A Windows GitHub Actions workflow has been added to run frontend build, Rust tests, and Rust backend checks. The workflow and real-device behavior still need to be observed after the latest commits.

## Development safety checklist

- Use argument arrays when launching ADB or scrcpy; do not build shell commands from raw UI input.
- Validate device identifiers before using them.
- Keep debug logs free of secrets and personal device content.
- Do not commit generated installers, signing keys, API keys, or local environment files.
- Test device disconnects and process cleanup.
- Do not attempt to bypass Android authorization or device security prompts.
