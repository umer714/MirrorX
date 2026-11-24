# MirrorX Mobile Architecture

## Product scope

The mobile companion is intended to support two explicit modes:

1. **Sender:** phone screen to MirrorX desktop.
2. **Controller:** phone touchpad / keyboard / approved shortcut input to MirrorX desktop.

The desktop is the trusted session host. The mobile app is not a hidden remote-access agent.

## App structure

```
mobile/
  App.tsx
  app.json
  package.json
  modules/
    mirrorx-capture/
      android/       MediaProjection foreground service + H.264 encoder
      src/           Typed Expo module interface
  src/
    pairing/         Authenticated local pairing (planned)
    transport/       WebRTC + reliable data channel (planned)
    controls/        Touchpad and keyboard events (planned)
```

## Session lifecycle (target)

1. Desktop displays a one-time pairing challenge / QR code.
2. Phone scans or enters the challenge and verifies the desktop identity.
3. User chooses Sender or Controller.
4. Sender mode requests the operating system's screen-capture permission.
5. Controller mode requires explicit desktop approval before accepting input.
6. A direct local-network session is negotiated.
7. Stop, permission revocation, network loss, or desktop exit closes the session and invalidates ephemeral credentials.

Pairing and transport steps are target behavior, not yet implemented.

## Media and control planes (target)

- Media plane: encrypted WebRTC video stream; optional audio only after explicit user selection and permission.
- Control plane: authenticated, reliable data channel with strict message schema and rate limits.
- Pairing plane: short-lived challenge, authenticated session establishment, replay protection.
- Discovery: mDNS / QR fallback on the same local network; no unauthenticated public listener.

Do not invent a custom cryptographic protocol. Use established transport security and vetted libraries.

## Native platform adapters

### Android

Implemented foundation:
- Android-owned MediaProjection consent is requested after an explicit user action.
- A non-exported foreground service declares the mediaProjection service type and keeps an ongoing notification visible.
- A VirtualDisplay feeds a MediaCodec H.264 encoder.
- The service reports local access-unit, byte, keyframe, codec configuration, and resolution statistics.
- Stop and projection-revocation paths release capture resources.

Important boundary: encoded access units are counted from MediaCodec metadata and released without a per-frame copy. They are not retained or transmitted. No Android network sender, peer discovery, pairing, Windows receiver, or live playback is connected. Encoder counters are not evidence of a remote video stream.

### iOS

Not implemented. The planned approach is a ReplayKit Broadcast Upload Extension, subject to current iOS SDK constraints and App Store requirements. It requires macOS, Xcode, an appropriate Apple developer signing setup, and physical-device testing. Do not label the iOS app as a working sender until that build and test path exists.

## Remote-control safety

The controller sends only intentional pointer, key, and approved shortcut events to the paired desktop. No background input injection, hidden control, credential capture, or bypass of desktop consent. Desktop can revoke the mobile controller at any time.

## Current implementation boundary

The shared React Native interface and Android capture/encoder foundation exist. The Windows desktop currently has a separate ADB + scrcpy process prototype. These components are not integrated. Pairing, WebRTC transport, desktop receiver, remote input, and iOS native capture remain future milestones and require target-device testing.
