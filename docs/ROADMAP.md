# MirrorX — Development Roadmap

This roadmap is provisional. Milestones should be considered complete only after the feature is implemented and tested on a real Android device.

## Milestone 0 — Foundation

- [x] Create public GitHub repository.
- [x] Document product scope and initial architecture.
- [x] Record privacy and security principles.
- [ ] Select the project's own code license.
- [ ] Confirm Windows development environment and toolchain.

## Milestone 1 — Desktop shell

- [ ] Create Tauri 2 + React + TypeScript application.
- [ ] Add Rust command layer and typed UI events.
- [ ] Establish app layout, navigation, and device status area.
- [ ] Add accessible loading, empty, and error states.

## Milestone 2 — Device discovery

- [ ] Integrate ADB process management.
- [ ] Detect connected Android devices.
- [ ] Show device model/state where available.
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
