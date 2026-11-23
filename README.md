# MirrorX

**Your Phone. Your PC. One Experience.**

MirrorX is a Windows-first desktop application project focused on Android screen mirroring and device interaction.

> **Current status:** The React + TypeScript dashboard, Tauri 2 / Rust shell, local ADB device discovery, and initial scrcpy process controls are in the repository. The app can request a local scrcpy launch for a selected authorized device and stop the process it launched. This has not yet been compiled and tested on a Windows machine with a real Android phone. The screen appears in scrcpy's separate window; it is not embedded in the MirrorX dashboard yet. The Vercel website is not the Windows desktop application.

## Project links

- **Website:** https://mirrorxapp.vercel.app
- **GitHub:** https://github.com/umer714/MirrorX

## Current prototype

Inside the native Tauri desktop app:

1. The USB screen calls ADB locally and lists connected devices.
2. The user selects an authorized device.
3. Start mirroring launches `scrcpy --serial <device>` as a local process.
4. Stop session terminates the scrcpy process started by MirrorX.

Install Android Platform Tools and scrcpy on the Windows computer and make sure both `adb.exe` and `scrcpy.exe` are available in PATH. Enable USB debugging and approve the computer authorization prompt on the phone.

The public website cannot access USB devices or launch local executables. The scrcpy window is currently separate from the MirrorX dashboard.

## Planned capabilities

- Embed the mirrored screen inside the MirrorX desktop window.
- Verify mouse and keyboard interaction on supported Android devices.
- Improve session lifecycle, reconnect behavior, and diagnostics.
- Explore wireless connections, screenshots, recording, clipboard support, and multi-device workflows in later milestones.

## Technical direction

| Area | Technology | Responsibility |
|---|---|---|
| Desktop shell | Tauri 2 | Native desktop window and packaging |
| User interface | React + TypeScript | Screens, controls, device status |
| Native core | Rust | Process lifecycle and device orchestration |
| Android connection | Android Debug Bridge (ADB) | Device discovery and verification |
| Mirroring engine | scrcpy | Android screen mirroring and supported controls |

Vercel hosts the public web frontend and documentation; it does not carry live screen frames. Mirroring is intended to run locally between the PC and Android device.

## Development

Frontend build:

```bash
npm install
npm run build
```

Native Windows development also requires Rust, Microsoft C++ Build Tools / Windows SDK, Tauri's Windows prerequisites, Android Platform Tools, and scrcpy.

See [Development notes](docs/DEVELOPMENT.md), [Architecture](docs/ARCHITECTURE.md), and [Roadmap](docs/ROADMAP.md).

## CI

GitHub Actions runs the frontend build, Rust unit tests, and Rust backend check on Windows for pushes and pull requests to `main`.

## Third-party components and licensing

MirrorX integrates with external tools including ADB and scrcpy. Before distribution, document exact versions, notices, modifications, and packaging obligations. See the upstream [scrcpy repository](https://github.com/Genymobile/scrcpy) for its current license and notices.

The license for MirrorX's own code has **not yet been selected**. Public repository visibility does not itself grant permission to reuse the code.

## Project principles

- Make device connection status and required permissions clear.
- Never collect phone contents or transmit screen data to a server by default.
- Keep mirroring traffic local between the computer and Android device.
- Explain optional network features before enabling them.
- Use safe process management and explicit device selection.
- Do not bypass Android authorization or device security prompts.

## Release rule

Do not describe a feature as available until it has been implemented, tested on a real Android device, and included in a published Windows build.
