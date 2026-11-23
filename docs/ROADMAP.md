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
- [x] Add initial Rust command registration.
- [ ] Validate Tauri desktop app builds and launches on Windows.
- [ ] Add app icon, installer metadata, and version display.

## Milestone 2 — USB device discovery

- [x] Add a local ADB `devices -l` command.
- [x] Parse device serial, model, product, and state.
- [x] Display detected devices and authorization states in the UI.
- [x] Add parser unit tests.
- [ ] Compile and run the native app on Windows.
- [ ] Test with a real authorized Android device.
- [ ] Test unauthorized, disconnected, and multiple-device cases.
- [ ] Improve ADB discovery and diagnostics based on real-device tests.

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
