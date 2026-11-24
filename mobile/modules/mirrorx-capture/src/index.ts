import { requireOptionalNativeModule } from "expo";

type CapturePermissionResult = {
  granted: boolean;
  status: "approved" | "denied";
};

export type CaptureStats = {
  active: boolean;
  frames: number;
  width: number;
  height: number;
};

type MirrorXCaptureNativeModule = {
  requestScreenCapturePermission(): Promise<CapturePermissionResult>;
  hasApprovedScreenCapturePermission(): boolean;
  clearApprovedScreenCapturePermission(): boolean;
  stopProjectionSession(): boolean;
  getCaptureStats(): CaptureStats;
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

export function stopProjectionSession(): boolean {
  return nativeModule?.stopProjectionSession() ?? false;
}

export function getCaptureStats(): CaptureStats {
  return nativeModule?.getCaptureStats() ?? { active: false, frames: 0, width: 0, height: 0 };
}
