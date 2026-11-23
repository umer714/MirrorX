# MirrorX

**Your Phone. Your PC. One Experience.**

MirrorX is a Windows-first desktop application project focused on Android screen mirroring and device interaction. The goal is to provide a clear, approachable desktop experience built around established Android tooling.

> **Project status:** Early planning / foundation. This repository currently contains product documentation. A working desktop application has not been implemented yet.

## Planned capabilities

- Detect Android devices connected over USB.
- Start and stop a live mirrored screen.
- Support mouse and keyboard interaction where supported by the connected device and engine.
- Provide clear connection and troubleshooting states.
- Explore wireless connections, screenshots, recording, clipboard support, and multi-device workflows in later milestones.

## Technical direction

| Area | Planned technology | Responsibility |
|---|---|---|
| Desktop shell | Tauri 2 | Native desktop window and packaging |
| User interface | React + TypeScript | Screens, controls, device status |
| Native core | Rust | Process lifecycle and device orchestration |
| Android connection | Android Debug Bridge (ADB) | Device discovery and connection |
| Mirroring engine | scrcpy | Android screen mirroring and supported controls |

The initial target is **Windows**. Other desktop platforms may be considered after the first stable release.

## Repository layout (planned)

```text
MirrorX/
├── README.md
├── .gitignore
└── docs/
    ├── ARCHITECTURE.md
    ├── ROADMAP.md
    └── DEVELOPMENT.md
```

The source application directories will be added when implementation begins.

## Getting started

There is no runnable application in this repository yet. See:

- [Architecture](docs/ARCHITECTURE.md)
- [Roadmap](docs/ROADMAP.md)
- [Development notes](docs/DEVELOPMENT.md)

## Third-party components and licensing

MirrorX plans to integrate with scrcpy and may use other third-party components. Each dependency must retain its applicable copyright and license notices. Before distribution, the project must document the exact versions, notices, modifications, and any packaging obligations. See the upstream [scrcpy repository](https://github.com/Genymobile/scrcpy) for its current license and notices.

The license for MirrorX's own code has **not yet been selected**. Do not assume that the public visibility of this repository grants permission to reuse its code.

## Project principles

- Make device connection status and required permissions clear.
- Never collect phone contents or transmit screen data to a server by default.
- Keep mirroring traffic local between the computer and Android device.
- Explain any optional network features before enabling them.
- Prefer reliable, accessible workflows over unnecessary complexity.

## Contributing

The project is in its foundation stage. Design and implementation decisions may change as the first Windows prototype is built.
