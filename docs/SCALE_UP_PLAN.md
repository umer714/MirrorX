# MirrorX — Scale-Up Engineering Plan

This plan defines how MirrorX can grow from a Windows-first prototype into a reliable cross-platform product without pretending that planned capabilities already work.

## Product tracks

| Track | Initial target | Later expansion |
|---|---|---|
| Desktop host | Windows 10/11, Android over USB | Windows wireless, macOS host |
| Mobile sender | Android user-approved capture | iOS broadcast extension |
| Mobile controller | Explicitly paired local input | Tablet layouts and accessibility |
| Web | Product site, docs, downloads, status | Release notes and support portal |

The web application is not a relay for live screen media. The default media path remains peer-to-peer on the local network.

## Architecture boundaries

- **Presentation:** React / TypeScript on desktop; React Native / Expo shared mobile UI.
- **Desktop orchestration:** Tauri commands are thin, validated entry points. Device discovery, process lifecycle, pairing and session state belong in Rust modules.
- **Platform adapters:** ADB + scrcpy on Android/Windows; Android MediaProjection and iOS ReplayKit/ScreenCaptureKit behind separate native adapters.
- **Session transport:** WebRTC for media; a separate authenticated, reliable data channel for control messages.
- **Pairing:** QR-based, short-lived one-time challenge; authenticated key exchange; per-session credentials; replay protection.
- **Optional cloud:** account, licensing and update metadata only. Cloud is not required to establish a local mirroring session.

## Non-negotiable security requirements

1. Screen capture starts only after a visible, OS-approved user action.
2. Controller pairing requires explicit approval on the desktop.
3. No public unauthenticated control listener and no silent port exposure.
4. Bind local services to the minimum required interfaces; use authenticated session tokens.
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

### Gate B — Android sender
- Native MediaProjection permission flow and foreground-service notification.
- Explicit start/stop controls and immediate cleanup on permission revocation.
- Hardware encoder capability detection and adaptive resolution/frame rate.
- Battery, thermal and network-loss tests on physical devices.

### Gate C — Secure pairing and transport
- One-time pairing challenge with expiration and rate limiting.
- Mutual session authentication and key rotation between sessions.
- WebRTC connection on the same Wi-Fi network without a cloud media relay.
- Network interruption, reconnect and stale-session tests.

### Gate D — Controller
- Desktop approval, visible active-controller indicator and immediate revoke.
- Allowlisted pointer, keyboard and shortcut message types.
- Bounds checks, rate limiting and input focus rules.
- No credential capture, hidden control or bypass of OS security prompts.

### Gate E — iOS
- Native broadcast extension with system permission and visible system controls.
- Verify current Apple SDK support and App Store requirements before selecting implementation details.
- Physical-device test and signed archive on macOS/Xcode; CI type-check alone is not an iOS build.

### Gate F — Release
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
3. Implement Android native capture and a real local sender path.
4. Implement authenticated pairing and transport.
5. Add the approved controller path.
6. Add iOS native broadcast support.
7. Expand host platforms and public release operations.

A milestone is complete only when its acceptance tests pass on the target platform. A successful web deployment, TypeScript check or debug APK is not proof of native mirroring functionality.
