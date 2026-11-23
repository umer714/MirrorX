# MirrorX — Development Setup Notes

The repository contains a React + TypeScript frontend scaffold and a Tauri 2 / Rust desktop shell. Android device integration is not implemented yet.

## Windows prerequisites

For native desktop development, install and verify:

- Node.js LTS and npm.
- Rust stable toolchain.
- Microsoft C++ Build Tools with the Windows SDK.
- Tauri 2 prerequisites for Windows.
- Android Platform Tools (ADB), for the upcoming device-discovery milestone.
- A compatible scrcpy build, for the upcoming mirroring milestone.

Use the official installation instructions for each tool and check their current requirements before setup.

## Verify tools

Open PowerShell and run:

```powershell
node --version
npm --version
rustc --version
cargo --version
```

When ADB is installed, also run:

```powershell
adb version
```

## Frontend build

From the repository root:

```powershell
npm install
npm run build
```

The Vercel production build has passed with the current TypeScript/Vite configuration. A local Windows Tauri build still needs to be tested on a Windows development machine.

## Development safety checklist

- Use argument arrays when launching ADB or scrcpy; do not build shell commands from raw UI input.
- Validate device identifiers before using them.
- Keep debug logs free of secrets and personal device content.
- Do not commit generated installers, signing keys, API keys, or local environment files.
- Test device disconnects and process cleanup.
- Do not attempt to bypass Android authorization or device security prompts.
