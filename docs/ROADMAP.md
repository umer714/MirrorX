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

## Milestone 1 — Desktop shell

- [x] Establish initial app layout and connection-mode UI.
- [x] Add Rust command registration.
- [ ] Validate Tauri desktop app builds and launches on Windows.
- [ ] Add app icon, installer metadata, and version display.

## Milestone 2 — USB device discovery

- [x] Add local ADB device listing.
- [x] Parse device serial, model, product, and state.
- [x] Display detected devices and authorization states.
- [x] Add parser unit tests.
- [ ] Test on a Windows machine with a real Android phone.

## Milestone 3 — First scrcpy launch prototype

- [x] Add selected-device scrcpy launch command.
- [x] Verify device authorization before launching scrcpy.
- [x] Add a managed scrcpy child-process handle.
- [x] Add Stop Session command.
- [x] Add selected-device Start / Stop controls to the UI.
- [x] Add Windows GitHub Actions build and Rust test checks.
- [ ] Run the Windows CI workflow successfully.
- [ ] Test start, stop, disconnect, and app-exit behavior with a real phone.
- [ ] Embed the mirrored screen inside the MirrorX dashboard (currently separate scrcpy window).

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
