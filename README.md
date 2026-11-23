# MirrorX

**Your Phone. Your PC. One Experience.**

MirrorX is a Windows-first desktop application project focused on Android screen mirroring and device interaction.

> **Current status:** The React + TypeScript dashboard, Tauri 2 / Rust shell, and first local ADB device-discovery command are implemented in the repository. The ADB command has parser unit tests, but has not yet been compiled and tested on a Windows machine with a real Android phone. scrcpy session management and live screen mirroring are **not implemented yet**. The Vercel website is not the Windows desktop application.

## Project links

- **Website:** https://mirrorxapp.vercel.app
- **GitHub:** https://github.com/umer714/MirrorX

## Current prototype

The USB screen can call a native Tauri command that runs `adb devices -l` locally and displays device model, serial, and connection state. It reports missing ADB and unauthorized devices instead of attempting to bypass Android authorization.

To test this feature, build and run the Tauri desktop app on Windows, install Android Platform Tools, connect an Android phone with USB debugging enabled, and approve the computer authorization prompt. The public website cannot access USB devices.

## Planned capabilities

- Detect Android devices connected over USB.
- Start and stop a live mirrored screen using scrcpy.
- Support mouse and keyboard interaction where supported by the connected device and engine.
- Provide clear connection and troubleshooting states.
- Explore wireless connections, screenshots, recording, clipboard support, and multi-device workflows in later milestones.

## Technical direction

| Area | Technology | Responsibility |
|---|---|---|
| Desktop shell | Tauri 2 | Native desktop window and packaging |
| User interface | React + TypeScript | Screens, controls, device status |
| Native core | Rust | Process lifecycle and device orchestration |
| Android connection | Android Debug Bridge (ADB) | Device discovery and connection |
| Mirroring engine | scrcpy | Android screen mirroring and supported controls |

The initial target is **Windows**. Vercel hosts the public web frontend and documentation; it does not carry live screen frames. The mirroring implementation is intended to run locally between the PC and Android device.

## Repository layout

```text
MirrorX/
├── src/                 # React + TypeScript frontend
├── src-tauri/           # Tauri 2 / Rust desktop shell and commands
├── docs/
│   ├── ARCHITECTURE.md
│   ├── ROADMAP.md
│   └── DEVELOPMENT.md
├── package.json
├── vite.config.ts
└── README.md
```

## Development

The frontend build can be checked with:

```bash
npm install
npm run build
```

For native Windows development, install the Rust toolchain, Microsoft C++ Build Tools / Windows SDK, Tauri's Windows prerequisites, and Android Platform Tools. ADB must be available in PATH for USB discovery. A compatible scrcpy build will be required when mirroring is implemented.

See [Development notes](docs/DEVELOPMENT.md), [Architecture](docs/ARCHITECTURE.md), and [Roadmap](docs/ROADMAP.md).

## Third-party components and licensing

MirrorX plans to integrate with scrcpy and may use other third-party components. Each dependency must retain its applicable copyright and license notices. Before distribution, document exact versions, notices, modifications, and packaging obligations. See the upstream [scrcpy repository](https://github.com/Genymobile/scrcpy) for its current license and notices.

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
