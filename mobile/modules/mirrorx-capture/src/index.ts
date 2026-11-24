import { requireOptionalNativeModule } from "expo";

type CapturePermissionResult = {
  granted: boolean;
  status: "approved" | "denied";
};

type MirrorXCaptureNativeModule = {
  requestScreenCapturePermission(): Promise<CapturePermissionResult>;
  hasApprovedScreenCapturePermission(): boolean;
  clearApprovedScreenCapturePermission(): boolean;
};

const nativeModule =
  requireOptionalNativeModule<MirrorXCaptureNativeModule>("MirrorXCapture");

export function isScreenCapturePermissionAvailable(): boolean {
  return nativeModule !== null;
}

export async function requestScreenCapturePermission(): Promise<CapturePermissionResult> {
  if (!nativeModule) {
    throw new Error("Android screen-capture permission is only available in a native MirrorX build.");
  }
  return nativeModule.requestScreenCapturePermission();
}

export function hasApprovedScreenCapturePermission(): boolean {
  return nativeModule?.hasApprovedScreenCapturePermission() ?? false;
}

export function clearApprovedScreenCapturePermission(): boolean {
  return nativeModule?.clearApprovedScreenCapturePermission() ?? false;
}
