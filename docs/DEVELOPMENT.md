# MirrorX — Development Setup Notes

The repository contains a React + TypeScript frontend, a Tauri 2 / Rust desktop shell, and an initial ADB device-discovery command.

## Windows prerequisites

For native desktop development, install and verify:

- Node.js LTS and npm.
- Rust stable toolchain.
- Microsoft C++ Build Tools with the Windows SDK.
- Tauri 2 prerequisites for Windows.
- Android Platform Tools (ADB).
- A compatible scrcpy build when the mirroring milestone begins.

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

## Frontend build

From the repository root:

```powershell
npm install
npm run build
```

The frontend is also built by Vercel. This does not compile the native Tauri application.

## Run the desktop app

After the Windows prerequisites are installed:

```powershell
npm run tauri dev
```

The app's USB device scan invokes the local `adb devices -l` command. Connect a phone with USB debugging enabled and approve the computer authorization prompt. If ADB is not found, confirm Android Platform Tools are installed and that the folder containing `adb.exe` is in PATH, then restart the terminal and app.

The public website cannot access a computer's USB devices; device discovery only works inside the native desktop app.

## Current testing status

The Rust parser includes unit tests for authorized, unauthorized, and empty device lists. These tests have not yet been run in a Windows build environment. A real Android device test is still required.

## Development safety checklist

- Use argument arrays when launching ADB or scrcpy; do not build shell commands from raw UI input.
- Validate device identifiers before using them.
- Keep debug logs free of secrets and personal device content.
- Do not commit generated installers, signing keys, API keys, or local environment files.
- Test device disconnects and process cleanup.
- Do not attempt to bypass Android authorization or device security prompts.
