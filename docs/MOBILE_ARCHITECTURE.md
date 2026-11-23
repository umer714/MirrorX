# MirrorX Mobile Architecture

## Product scope

The mobile companion supports two explicit modes:

1. **Sender:** phone screen to MirrorX desktop.
2. **Controller:** phone touchpad / keyboard / shortcut input to MirrorX desktop.

The desktop is the trusted session host. The mobile app is not a hidden remote-access agent.

## App structure

```
mobile/
  App.tsx                 Shared React Native interface
  app.json                Android/iOS identifiers
  package.json            Expo SDK and React Native dependencies
  native/
    android/              MediaProjection adapter (planned)
    ios/                  Screen-sharing adapter (planned)
  src/
    pairing/              Authenticated local pairing (planned)
    transport/            WebRTC + reliable data channel (planned)
    controls/             Touchpad and keyboard events (planned)
```

## Session lifecycle

1. Desktop displays a one-time pairing challenge / QR code.
2. Phone scans or enters the challenge and verifies the desktop identity.
3. User chooses Sender or Controller.
4. Sender mode requests the operating system's screen-capture permission.
5. Controller mode requires explicit desktop approval before accepting input.
6. A direct local-network session is negotiated.
7. Stop, permission revocation, network loss, or desktop exit closes the session and invalidates ephemeral credentials.

## Media and control planes

- Media plane: WebRTC video stream, optional audio only after explicit user selection and permission.
- Control plane: authenticated, reliable data channel with strict message schema and rate limits.
- Pairing plane: short-lived challenge, authenticated key exchange, replay protection.
- Discovery: mDNS / QR fallback on the same local network; do not expose an unauthenticated public listener.

## Native platform adapters

### Android

MediaProjection requires a user-approved capture grant. Use a foreground service with the correct media-projection service type and visible notification. Handle callback revocation and release all virtual-display / encoder resources. Never assume a grant can be reused for a later capture session.

### iOS

Use the Apple screen-sharing API available for the deployment target and SDK used by the build. On current supported releases, evaluate ScreenCaptureKit; retain an isolated compatibility adapter if older supported iOS releases require ReplayKit. System permission and stop controls must remain authoritative.

## Remote-control safety

The controller sends only intentional pointer, key, and approved shortcut events to the paired desktop. No background input injection, hidden control, credential capture, or bypass of desktop consent. Desktop can revoke the mobile controller at any time.

## Current implementation boundary

The initial mobile foundation commit includes the shared UI, app metadata, and this architecture. It does **not** yet include native capture, pairing, WebRTC, or desktop-side control endpoints. Those are separate implementation milestones and must be tested on physical devices before claiming support.
