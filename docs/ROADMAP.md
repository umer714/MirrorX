# MirrorX — Development Roadmap

A milestone is complete only after its features are implemented and tested. A successful Vercel web build does not mean the desktop app or live mirroring engine is complete.

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
- [x] Run the latest Windows CI workflow successfully.
- [ ] Test start, stop, disconnect, and app-exit behavior with a real phone.
- [x] Add ADB Wireless debugging pair and connect commands with IP/port validation.
- [x] Add desktop wireless pairing and connection form.
- [ ] Test wireless pairing and reconnect behavior with a real Android phone.
- [ ] Embed the mirrored screen inside the MirrorX dashboard (currently separate scrcpy window).

## Milestone 4 — Android capture foundation

- [x] Request Android's official MediaProjection consent.
- [x] Start a visible, non-exported mediaProjection foreground service.
- [x] Create a VirtualDisplay and H.264 MediaCodec encoder.
- [x] Report local encoder access-unit, byte, keyframe, and resolution statistics.
- [x] Surface native encoder failures to the mobile UI.
- [ ] Test capture start, stop, revocation, and device rotation on real Android hardware.

## Milestone 5 — Secure local transport and receiver

- [x] Select a shared Rust WebRTC transport for the prototype (ADR-0001); Android build/interoperability gate remains.
- [ ] Implement short-lived authenticated signaling and user-approved pairing flow.
- [ ] Implement local peer discovery / connection setup.
- [x] Add shared H.264 Annex-B normalization, IDR/SPS/PPS inspection, and bounded keyframe-aware queue primitives with desktop CI coverage.
- [x] Export a thread-safe C ABI and header for native H.264 queue access; test frame copying, queue-empty behavior, invalid inputs, and output-buffer backpressure.
- [ ] Build the shared media crate for Android ABIs and add the Kotlin/JNI binding.
- [ ] Connect Android MediaCodec output to the shared queue and WebRTC sender with a bounded three-frame queue, keyframe recovery, and backpressure.
- [ ] Implement Windows receiver and live playback.
- [ ] Add disconnect, reconnect, timeout, and session revocation handling.
- [ ] Test end-to-end on a real Android phone and Windows PC.

## Milestone 6 — Remote controller and iOS

- [ ] Implement explicitly approved, authenticated remote input.
- [ ] Add touchpad, keyboard, and constrained shortcut controls.
- [ ] Implement iOS ReplayKit broadcast extension.
- [ ] Build and test iOS with macOS, Xcode, and Apple signing.
- [ ] Verify platform-specific permissions and lifecycle behavior.

## Milestone 7 — Release preparation

- [ ] Choose and document project licensing.
- [ ] Audit third-party licenses and notices.
- [ ] Add automated tests and build checks.
- [ ] Package and test a Windows installer and Android build.
- [ ] Add release notes, support guidance, and security reporting instructions.

## Release rule

Do not describe a feature as available until it has been implemented, tested, and included in a published build.
