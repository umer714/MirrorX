# MirrorX Android Capture Bridge

This local Expo module starts a user-approved Android screen-capture session through the platform-owned MediaProjection consent flow.

## Implemented

- Opens Android's system consent dialog only after the user taps the capture button.
- Starts a non-exported foreground service with the `mediaProjection` service type.
- Creates a VirtualDisplay and feeds it into an Android MediaCodec H.264 encoder.
- Normalizes MediaCodec H.264 codec-specific data (Annex-B, raw SPS/PPS, and AVCDecoderConfigurationRecord) into Annex-B before prefixing parameter sets to IDR frames. Unsupported/malformed codec configuration is surfaced as `INVALID_H264_CODEC_CONFIG` instead of silently killing the drain worker.\n- Tracks encoded access units, encoded byte count, keyframes, codec configuration bytes, and capture resolution.
- Validates output buffer bounds and counts encoded bytes directly from MediaCodec metadata without allocating a discarded per-frame copy.
- Shows an ongoing system notification while capture is active.
- Stops capture resources when the user requests Stop or Android revokes the projection.

## Not implemented yet

- Encoded H.264 access units are counted locally and released; they are not retained or transmitted.
- No LAN discovery, pairing, transport, receiver, or live playback is connected.
- The current status counters prove encoder output only; they do not prove that a desktop received video.

## Platform requirements

Android owns the screen-capture permission prompt. MirrorX does not bypass it. A fresh user-approved projection session is required according to Android's MediaProjection lifecycle rules. The foreground notification remains visible while capture is active.

The next milestone is authenticated local pairing and encrypted media transport, followed by a Windows receiver and real-device end-to-end testing. Screen content must not be sent to a cloud service by default.

## Native queue consumer lifecycle

The internal encoded-frame consumer is single-owner/single-consumer. Its `close()` call interrupts and joins the drain worker before returning, including when the calling thread is interrupted (the interrupt flag is restored afterward). The native queue handle must not be destroyed until `close()` has returned. Callback failures are contained so an application callback cannot bypass worker cleanup.
