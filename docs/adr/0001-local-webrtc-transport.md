# ADR-0001: Local WebRTC media transport

- Status: Accepted for a prototype spike; production adoption is gated on Android cross-compilation and interoperability tests.
- Date: 2026-11-24

## Context

MirrorX has two disconnected foundations: Android captures the display with MediaProjection and MediaCodec (H.264), while the Windows desktop shell is a Tauri 2 application with a Rust core. A transport must carry real-time video and a separate, narrowly scoped control channel without sending screen contents through Vercel or Supabase.

A custom TCP/UDP protocol with home-grown encryption is not acceptable. Media must use an established WebRTC implementation so that DTLS-SRTP protects media and DTLS-SCTP protects data channels. Pairing/signaling must authenticate the intended peer and require visible user approval.

## Decision

Prototype a shared Rust transport core using **webrtc-rs** on Windows and Android:

- Windows: host the Rust peer connection in the existing Tauri Rust application.
- Android: build the same Rust transport crate for Android ABIs and expose a small JNI boundary to the existing Expo Kotlin capture module.
- Android MediaCodec remains the encoder. Pass complete encoded H.264 access units and codec configuration to Rust; do not decode/re-encode frames.
- Send H.264 as a WebRTC video track and use a separate WebRTC data channel for session metadata and, later, explicitly approved remote-input messages.
- Use WebRTC's ICE, DTLS-SRTP, congestion handling, and negotiated media/data-channel mechanisms. No custom encryption layer.
- Keep signaling separate from media. Initial prototype signaling should be local-network-first, short-lived, authenticated, and user-approved; no media relay or cloud video storage.
- Keep the native boundary small: start/stop session, exchange signaling messages, submit encoded access units, and report connection/transport state.

The webrtc-rs project is a Rust WebRTC implementation with runnable examples for H.264/H.265 media and data channels. Its documented H.264 sample path expects encoded media in Annex-B form; MirrorX must validate the Android encoder's output format and convert length-prefixed NAL units only when required. See the project examples and H.264 implementation notes in the references below.

## Bounded media pipeline

- Use a bounded queue of at most three pending video access units.
- If the sender falls behind, discard stale queued frames rather than growing memory or latency.
- After dropping data that makes a predictive chain unusable, wait for the next keyframe before resuming delivery.
- Preserve codec configuration (SPS/PPS) and ensure a decodable keyframe is available when a receiver joins or reconnects.
- Track queue depth, dropped frames, encoded bytes, keyframes, RTT, and estimated outbound bitrate.
- Stop and release the peer connection, JNI resources, and capture service on user stop, permission revocation, timeout, or app shutdown.

## Security requirements

- No unauthenticated peer is allowed to receive screen video or send control messages.
- Pairing credentials are short-lived, single-session, and revoked on disconnect/stop.
- The desktop must display the requesting phone and require explicit approval before accepting a session.
- Control messages use a versioned allow-listed schema; reject unknown commands, malformed payloads, stale sessions, and oversized messages.
- Screen capture remains visible through Android's system consent and foreground notification.
- Do not expose a general-purpose remote shell or arbitrary command execution interface.

## Alternatives considered

### libdatachannel through C/C++ FFI

It supports WebRTC media and data channels on Windows and Android and has Rust bindings. It is a credible fallback if webrtc-rs cannot be built reliably for Android or if its H.264 interop/performance is inadequate. It introduces a native C++ dependency and packaging/ABI work across Android and Windows, so it is not the first prototype choice.

### Separate Android libwebrtc and Rust desktop stack

This is possible but creates two different native APIs and more cross-platform integration work. Reconsider only if the shared Rust build proves impractical.

### Raw TCP/UDP with custom encryption

Rejected. It would require MirrorX to implement and maintain authentication, encryption, packetization, congestion control, retransmission, and lifecycle handling.

## Prototype exit criteria

Do not mark transport complete until all are true:

1. Windows and Android builds compile from the same transport crate.
2. A real Android phone and Windows PC establish an authenticated, user-approved peer session on the same LAN.
3. H.264 video renders on Windows continuously with bounded memory and observable frame drops.
4. Session stop, permission revocation, disconnect, and reconnect release resources correctly.
5. Unpaired peers cannot view video or send control messages.
6. Automated tests cover H.264 framing, bounded queue behavior, signaling validation, and session revocation.

## References

- [webrtc-rs repository](https://github.com/webrtc-rs/webrtc)
- [webrtc-rs H.26x playback example](https://github.com/webrtc-rs/webrtc/blob/master/examples/play-from-disk-h26x/play-from-disk-h26x.rs)
- [webrtc-rs media API and examples](https://github.com/webrtc-rs/webrtc/blob/master/examples/README.md)
- [libdatachannel project](https://github.com/paullouisageneau/libdatachannel)
