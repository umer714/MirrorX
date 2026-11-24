# MirrorX — Scale-Up Engineering Plan

This plan defines how MirrorX can grow from its current prototypes into a reliable cross-platform product without pretending that planned capabilities already work.

## Product tracks

| Track | Initial target | Later expansion |
|---|---|---|
| Desktop host | Windows 10/11, Android over USB | Windows wireless, macOS host |
| Mobile sender | Android capture/encoder foundation | Android LAN sender; iOS ReplayKit broadcast |
| Mobile controller | Shared UI shell | Explicitly paired local input |
| Web | Product site, docs, downloads, status | Release notes and support portal |

The web application is not a relay for live screen media. The intended media path remains peer-to-peer on the local network.

## Current implementation snapshot

- React + TypeScript desktop UI and Tauri 2 / Rust shell exist.
- Desktop Rust code can discover ADB devices and launch/stop a local scrcpy child process. scrcpy opens in a separate window.
- Android mobile UI and a native MediaProjection foreground service exist.
- Android service creates a VirtualDisplay and H.264 MediaCodec encoder and reports local encoder statistics.
- Encoded H.264 access units are not transmitted; they are currently discarded after local accounting.
- The Android capture prototype is not connected to the Windows scrcpy prototype.
- Pairing, WebRTC, Windows video receiver, remote input, and iOS capture are not implemented.
- Physical-device end-to-end behavior remains unverified.

## Architecture boundaries

- **Presentation:** React / TypeScript on desktop; React Native / Expo shared mobile UI.
- **Desktop orchestration:** Tauri commands are thin, validated entry points. Device discovery, process lifecycle, pairing and session state belong in Rust modules.
- **Platform adapters:** ADB + scrcpy on Android/Windows; Android MediaProjection + MediaCodec; iOS ReplayKit Broadcast Upload Extension.
- **Session transport:** WebRTC for encrypted media; a separate authenticated, reliable data channel for control messages.
- **Pairing:** QR-based, short-lived one-time challenge; authenticated session establishment; replay protection; credentials scoped to a single session.
- **Optional cloud:** account, licensing and update metadata only. Cloud is not required to establish a local mirroring session.

## Non-negotiable security requirements

1. Screen capture starts only after a visible, OS-approved user action.
2. Controller pairing requires explicit approval on the desktop.
3. No public unauthenticated control listener and no silent port exposure.
4. Bind local services to the minimum required interfaces; use authenticated session credentials.
5. Validate every command at the Rust boundary, not only in the UI.
6. Apply strict message schemas, size limits, rate limits and replay protection.
7. Revoke session credentials on stop, disconnect, permission revocation or app exit.
8. Never collect screen frames, clipboard contents or keystrokes for analytics.
9. Keep diagnostics free of screen content, secrets and personal data.
10. Publish a security contact and responsible disclosure process before public release.

## Reliability and quality gates

### Gate A — Desktop foundation
- Windows CI compiles the Tauri backend and runs Rust unit tests.
- Device parser covers authorized, unauthorized, offline, emulator, malformed and empty ADB output.
- Process lifecycle covers duplicate start, stop-before-start, child exit and app shutdown.
- Windows installer launches on a clean Windows 10/11 environment.
- Manual tests cover at least two Android vendors and USB reconnect.

### Gate B — Android capture foundation
- MediaProjection permission and foreground-service lifecycle.
- H.264 encoder initialization, output-format handling, and safe output-buffer bounds checks.
- Visible start/stop controls and immediate cleanup on permission revocation.
- Hardware encoder capability detection and adaptive resolution/frame rate.
- Battery, thermal, rotation, and encoder-failure tests on physical devices.

### Gate C — Secure pairing and transport
- One-time pairing challenge with expiration and rate limiting.
- Authenticated peer/session establishment and per-session credentials.
- WebRTC connection on the same Wi-Fi network without a cloud media relay.
- Network interruption, reconnect and stale-session tests.
- Bounded media queues and backpressure.

### Gate D — Windows receiver
- Receive H.264 track from approved peer.
- Decode and render frames in the desktop window.
- Clear session states, errors, stop, and reconnect handling.
- Verify CPU/GPU usage and latency on supported Windows hardware.

### Gate E — Controller
- Desktop approval, visible active-controller indicator and immediate revoke.
- Allowlisted pointer, keyboard, and shortcut message types.
- Bounds checks, rate limiting and input focus rules.
- No credential capture, hidden control or bypass of OS security prompts.

### Gate F — iOS
- Native ReplayKit Broadcast Upload Extension with system permission and visible system controls.
- Verify current Apple SDK support and App Store requirements before implementation.
- Physical-device test and signed archive on macOS/Xcode; CI type-check alone is not an iOS build.

### Gate G — Release
- Reproducible release builds, signed installers and checksums.
- Third-party license and notice audit (including scrcpy and Android tooling).
- Privacy policy, threat model, support process and release notes.
- Crash reporting opt-in, privacy-reviewed and free of captured content.

## Performance targets to validate

These are engineering targets, not current measured results:
- Local USB session startup: under 5 seconds on supported hardware.
- Local Wi-Fi pairing after approval: under 10 seconds on a healthy network.
- Interactive controller latency: target under 100 ms on a healthy local network.
- Adaptive video: prioritize responsiveness and thermal stability over fixed resolution.
- Idle desktop CPU usage: minimize background polling; use event-driven discovery where possible.

## Delivery sequence

1. Stabilize Windows CI, process lifecycle and device diagnostics.
2. Produce and test a Windows installer.
3. Harden Android capture lifecycle and validate it on physical devices.
4. Implement authenticated pairing and encrypted local transport.
5. Implement Windows receiver and live playback.
6. Add the approved controller path.
7. Add iOS ReplayKit capture and physical-device validation.
8. Expand host platforms and public release operations.

A milestone is complete only when its acceptance tests pass on the target platform. A successful web deployment, TypeScript check or debug APK is not proof of native mirroring functionality.
