# MirrorX — Development Roadmap

A milestone is complete only after its features are implemented and tested. A successful Vercel web build does not mean the Windows desktop app or mirroring engine is complete.

## Milestone 0 — Repository and web foundation

- [x] Create public GitHub repository.
- [x] Document product scope and initial architecture.
- [x] Record privacy and security principles.
- [x] Create React + TypeScript frontend scaffold.
- [x] Create Tauri 2 / Rust shell scaffold.
- [x] Deploy the public web frontend to Vercel.
- [ ] Select the project's own code license.
- [ ] Confirm Windows development environment and toolchain.

## Milestone 1 — Desktop shell

- [x] Establish initial app layout and connection-mode UI.
- [ ] Validate Tauri desktop app builds and launches on Windows.
- [ ] Add Rust command layer and typed UI events.
- [ ] Add accessible loading, empty, and error states.
- [ ] Add app icon, installer metadata, and version display.

## Milestone 2 — Device discovery

- [ ] Integrate ADB process management.
- [ ] Detect connected Android devices.
- [ ] Show device model and connection state where available.
- [ ] Explain Android USB debugging authorization.
- [ ] Handle disconnected and unauthorized devices safely.

## Milestone 3 — First mirroring prototype

- [ ] Integrate a documented scrcpy version.
- [ ] Start and stop a session for a selected device.
- [ ] Surface process errors and logs in a user-friendly way.
- [ ] Test session cleanup and app shutdown.

## Milestone 4 — Interaction and reliability

- [ ] Verify mouse and keyboard interaction.
- [ ] Add reconnect guidance.
- [ ] Test multiple connected devices and explicit device selection.
- [ ] Test common USB connection failures.

## Milestone 5 — Optional capabilities

- [ ] Evaluate wireless connection workflow.
- [ ] Evaluate screenshots and recording.
- [ ] Evaluate clipboard support.
- [ ] Evaluate multi-device sessions.
- [ ] Review privacy implications before enabling each feature.

## Milestone 6 — Release preparation

- [ ] Choose and document project licensing.
- [ ] Audit third-party licenses and notices.
- [ ] Add automated tests and build checks.
- [ ] Package and test a Windows installer.
- [ ] Add release notes, support guidance, and security reporting instructions.

## Release rule

Do not describe a feature as available until it has been implemented, tested, and included in a published build.
