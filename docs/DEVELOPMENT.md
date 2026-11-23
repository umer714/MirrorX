# MirrorX — Development Setup Notes

This document records the intended toolchain. The repository does not contain the application source yet.

## Planned prerequisites

For Windows development, install and verify:

- Node.js LTS and npm.
- Rust stable toolchain.
- Microsoft C++ Build Tools with the Windows SDK.
- Tauri 2 prerequisites for Windows.
- Android Platform Tools (ADB).
- A compatible scrcpy build for local development.

Use the official installation instructions for each tool and check their current requirements before setup.

## Verify tools

Open PowerShell and run:

```powershell
node --version
npm --version
rustc --version
cargo --version
adb version
```

If a command is not recognized, install the corresponding tool and reopen the terminal.

## Planned application setup

When implementation begins, the expected app will use:

- Tauri 2 for the desktop shell.
- React and TypeScript for the UI.
- Rust for native process and device management.

Exact project-generation commands and dependency versions will be added after the initial toolchain is confirmed.

## Development safety checklist

- Use argument arrays when launching ADB or scrcpy; do not build shell commands from raw UI input.
- Validate device identifiers before using them.
- Keep debug logs free of secrets and personal device content.
- Do not commit generated installers, signing keys, API keys, or local environment files.
- Test device disconnects and process cleanup.
