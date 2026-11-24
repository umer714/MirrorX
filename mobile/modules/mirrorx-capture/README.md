# MirrorX Android Capture Bridge

This local Expo module requests Android's system screen-capture consent through
MediaProjectionManager and reports whether the user approved or declined.

## Current scope

- Opens the Android-owned consent dialog only after the user taps the request button.
- Keeps the approved result token in module memory for the current app process.
- Exposes a status check and a clear operation to the React Native layer.
- Does not start a MediaProjection foreground service, create a VirtualDisplay,
  encode frames, or transmit screen content yet.

A successful consent result is **not** a live mirroring session. The next native
milestone is a user-visible mediaProjection foreground service with lifecycle
and revocation handling, followed by frame encoding and authenticated desktop
transport. The app must request consent again for each new projection session,
as required by Android.

No screen content is uploaded or transmitted by this permission bridge.
