# MirrorX — Architecture

## 1. Product boundary

MirrorX is a local-first phone-to-PC mirroring and control project. The current desktop prototype is Windows-first and uses ADB plus scrcpy. The mobile project contains an Android MediaProjection/H.264 capture foundation. These are separate prototypes and are not yet connected end to end.

The public Vercel site is not the live screen transport path.

## 2. Current implementation

```text
Windows desktop prototype
React + TypeScript UI
        |
        | Tauri commands
        v
Rust application core
        |
        +---- ADB: local device discovery and authorization check
        |
        +---- scrcpy: managed child process
                     (opens a separate window)

Android mobile capture prototype
User taps Start
        |
Android system MediaProjection consent
        |
Non-exported foreground capture service
        |
VirtualDisplay -> MediaCodec H.264 encoder
        |
Local diagnostic counters
        |
        X  No network transport or PC receiver yet
```

## 3. Desktop responsibilities

- Show detected devices and their connection state.
- Allow explicit selection of an authorized device.
- Start and stop the scrcpy process launched by MirrorX.
- Validate device identifiers and pass them as process arguments, never shell command strings.
- Verify the selected device is in ADB's authorized `device` state before starting scrcpy.
- Pair Android Wireless debugging sessions with the six-digit code shown by Android using `adb pair`.
- Connect to the separately displayed wireless debugging endpoint using `adb connect`.
- Validate IP addresses and ports, and pass values as process arguments rather than shell strings.

The scrcpy window is separate from MirrorX at this stage; its video surface has not been embedded.

## 4. Android capture responsibilities

- Request the platform-owned MediaProjection consent after an explicit user action.
- Run capture in a non-exported foreground service with the mediaProjection service type.
- Feed a VirtualDisplay into a MediaCodec H.264 encoder.
- Track output access units, bytes, keyframes, codec configuration size, and resolution.
- Keep an ongoing notification visible and stop capture when the user or operating system ends the projection.

Encoded access units are currently counted from MediaCodec metadata without a per-frame copy, then released. They are not retained or transmitted. There is no LAN sender, pairing, receiver, or playback implementation yet.

## 5. Planned secure transport boundary

**Prototype decision:** use a shared Rust `webrtc-rs` transport core for Windows and Android, subject to Android ABI build and real-device interoperability gates. The Android MediaCodec encoder will feed encoded H.264 access units into a small JNI boundary; the Rust core will publish a WebRTC video track and a separate data channel. Windows will host the same Rust transport from the Tauri application. See [ADR-0001](adr/0001-local-webrtc-transport.md).

- Use authenticated, short-lived signaling only for connection setup; do not relay screen media through Vercel or Supabase.
- Use WebRTC DTLS-SRTP for media and its protected data channel for approved session messages; do not invent a custom encryption protocol.
- Bind remote-control messages to an approved paired session and a versioned allow-listed command schema.
- Reject unauthenticated peers and stale/replayed session credentials.
- Bound the video queue to three pending access units; discard stale frames under load and resume at a decodable keyframe.
- Preserve H.264 codec configuration (SPS/PPS) and validate Annex-B versus length-prefixed NAL framing.
- Provide visible session state and immediate Stop controls on both endpoints.
- Keep device discovery and media transport local-network-first.

The platform-independent `crates/mirrorx-media` library now contains tested H.264 Annex-B normalization and a bounded keyframe-aware queue, and the Windows Rust crate links it. This is reusable media groundwork only: no network transport, JNI bridge, pairing, video delivery, or real-device test is implemented yet. Android cross-compilation remains an explicit gate.

## 6. Security and privacy

- USB and Android's user-approved ADB Wireless debugging pairing are available as desktop connection paths. Wireless pairing requires the user to enable Developer options / Wireless debugging and enter the temporary code shown by Android.
- Android capture requires explicit OS permission and a visible foreground notification.
- Never build shell command strings from UI input.
- Do not attempt to bypass Android authorization or device security controls.
- Do not send screen frames, device identifiers, or phone data to a remote service by default.
- Remote input must be explicitly enabled and scoped to an approved paired session.

## 7. Cloud services

Vercel may host the public website, documentation, and release information. It is not part of the live screen transport path.

Supabase is not required for local mirroring. It may be considered later for optional accounts or license entitlements with data collection documented and minimized.

## 8. Open engineering decisions

- Exact scrcpy version, distribution strategy, and embedding approach.
- WebRTC library and signaling/discovery implementation for Android and Windows.
- QR pairing payload format, expiry, and session authorization lifecycle.
- Windows installer signing and update strategy.
- iOS ReplayKit implementation and Apple build/signing requirements.
- MirrorX's own source-code license and third-party notices.
