# MirrorX Mobile

Android and iOS companion app for MirrorX desktop.

## Product modes

- **Screen Sender:** explicitly share the phone screen to a paired MirrorX desktop.
- **Remote Controller:** use the phone as a touchpad, keyboard and shortcut remote for the paired desktop.

## Stack

- Expo SDK 58 / React Native 0.88 / TypeScript for shared mobile UI.
- Android native capture adapter: Kotlin + Android MediaProjection foreground service.
- iOS native capture adapter: Swift + Apple's current screen-sharing APIs, selected by deployment target and SDK availability.
- Local-network transport: authenticated WebRTC media stream and a separate reliable data channel for remote input.
- Desktop remains the session authority; no cloud relay is required for the initial same-network release.

## Current status

This commit is the **mobile foundation UI**, not a working screen-streaming release. Pairing discovery, native capture, WebRTC transport, and remote input are not implemented yet. The UI deliberately reports that desktop discovery is not connected rather than pretending to find devices.

## Development

Install dependencies:

```sh
cd mobile
npm install
npm run start
```

Build native development clients:

```sh
npx expo prebuild
npx expo run:android
npx expo run:ios
```

iOS native builds require macOS with Xcode. Android native builds require Android Studio / Android SDK.

## Privacy and permissions

- Never capture until the user starts a visible session and grants the OS permission.
- Stop capture immediately when permission is revoked or the session ends.
- Do not capture protected/secure content or attempt to bypass OS restrictions.
- Pairing must use a short-lived challenge and authenticated key exchange; do not trust a user-entered IP address alone.
- Remote input is disabled until the user explicitly approves the paired desktop session.
- Prefer direct local-network transport. Do not upload screen frames or input events to a cloud service.

## Platform notes

### Android
Use MediaProjection with the required foreground service type and persistent user-visible notification. Handle projection revocation and Android version-specific service requirements. A single-use projection grant must not be silently reused.

### iOS
Use Apple's currently supported screen-sharing framework for the app's minimum deployment target. Present Apple's system sharing UI and honor the user's selection and stop action. Native capture availability differs by iOS release; keep the iOS adapter isolated from shared React Native UI.

## Release gates

- Real Android device capture and stream test.
- Real iPhone capture test on supported iOS hardware and OS.
- Windows desktop pairing and authenticated session test.
- Remote input authorization and disconnect tests.
- Privacy review, app signing, store metadata, and platform policy review.
