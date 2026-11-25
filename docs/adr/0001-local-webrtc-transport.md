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


## Implementation progress — 2026-11-25

The shared Rust crate now includes a bounded typed signaling message contract
(offer, answer, ICE candidate, end-of-candidates, close), payload size limits,
session identifier validation, and an ordering guard. The mobile TypeScript
layer has a matching shape validator and sequence guard.

An Android native `MirrorXEncodedFrameSource` now provides a single-consumer,
interruptible polling seam over the bounded Rust H.264 queue for a future native
transport adapter. It applies a small idle backoff and does not cross the React
Native bridge for each frame. It intentionally does not create a connection or
send media by itself.

These additions are protocol and adapter foundations only. They do not open a
signaling socket, authenticate a pairing ticket, establish a WebRTC peer
connection, exchange SDP over a network, or transmit video. Pairing-ticket
issuance, expiry/single-use enforcement, WebRTC adapter integration, and
real-device interoperability remain required before an end-to-end session can
be claimed.


### Verified WebRTC implementation target — 2026-11-25

The current upstream release inspected for the prototype is webrtc-rs **v0.17.3**. Its tagged H.264 example uses `MediaEngine::register_default_codecs`, `APIBuilder`, `RTCConfiguration`, `RTCPeerConnection::add_track`, and `TrackLocalStaticSample::write_sample`. The H.264 file reader yields Annex-B NAL units; the example writes samples through the H.264 static sample track. This is a verified API reference, not an integration in MirrorX. The upstream example's stdin/base64 signaling is demonstration-only and must not be copied as MirrorX's production pairing protocol.

Before integrating the Android encoder, MirrorX must form complete access-unit samples, preserve correct frame durations from MediaCodec presentation timestamps (not one sample per NAL with a guessed duration), normalize the NAL framing to Annex-B, and negotiate H.264 profile/packetization compatibility. Android codec-specific data must be parsed/normalized rather than blindly concatenated if the device reports a non-Annex-B representation. The sender must also consume RTCP feedback and expose transport/queue metrics.

Android capture telemetry now distinguishes encoded frames from native queue outcomes: accepted frames, queue-policy drops (including keyframe recovery), and invalid/push errors. This makes it possible to tell whether capture/encoding is working while the transport sender is still absent. Queue acceptance is not proof of network transmission or remote rendering.

### Desktop WebRTC sender foundation — 2026-11-25

The Windows/Tauri Rust core now pins webrtc-rs **v0.17.3** and includes `src-tauri/src/webrtc_transport.rs`. It creates a real `RTCPeerConnection`, registers the default codec/interceptor pipeline, adds an H.264 `TrackLocalStaticSample`, drains RTCP feedback, creates a gathered local SDP offer, accepts an SDP answer, and exposes a bounded Annex-B access-unit writer. Sample durations are derived from monotonic MediaCodec PTS deltas, with defensive bounds for irregular timestamps.

This is a transport implementation foundation, not an end-to-end product feature yet. It is not exposed through Tauri commands or connected to the Android JNI queue. Authenticated signaling, remote ICE candidate exchange, peer approval, Android-side WebRTC integration, Windows rendering, and real-device interoperability remain outstanding. Do not send SDP to an untrusted peer or enable video until the authenticated pairing/consent layer is wired in.
