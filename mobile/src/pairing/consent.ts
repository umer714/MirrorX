/**
 * Local consent state mirrored by the mobile UI.
 * This is not authentication: only set approvals after explicit user/OS consent.
 */
export type ConsentSnapshot = Readonly<{
  peerApproved: boolean;
  captureApproved: boolean;
  controlApproved: boolean;
}>;

export const EMPTY_CONSENT: ConsentSnapshot = Object.freeze({
  peerApproved: false,
  captureApproved: false,
  controlApproved: false,
});

export function approvePeer(): ConsentSnapshot {
  return { peerApproved: true, captureApproved: false, controlApproved: false };
}

export function grantCapture(state: ConsentSnapshot): ConsentSnapshot {
  if (!state.peerApproved) throw new Error("Peer approval is required before screen capture.");
  return { ...state, captureApproved: true };
}

export function grantControl(state: ConsentSnapshot): ConsentSnapshot {
  if (!state.peerApproved) throw new Error("Peer approval is required before remote control.");
  return { ...state, controlApproved: true };
}

export function canSendVideo(state: ConsentSnapshot): boolean {
  return state.peerApproved && state.captureApproved;
}

export function canAcceptControl(state: ConsentSnapshot): boolean {
  return state.peerApproved && state.controlApproved;
}

export function revokeConsent(): ConsentSnapshot {
  return EMPTY_CONSENT;
}
