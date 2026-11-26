export const MIRRORX_PAIRING_VERSION = 1;
export const MAX_PAIRING_TICKET_LIFETIME_SECONDS = 300;

export type PairingQrPayload = {
  version: 1;
  sessionId: string;
  ticket: string;
  expiresAt: number;
};

/**
 * Validate the data carried by a QR code before handing it to the native
 * authenticated pairing adapter. This does not authenticate a peer by itself.
 * The ticket must be checked server-side/in the desktop process, consumed
 * once, and exchanged only over an authenticated encrypted channel.
 */
export function parsePairingQrPayload(input: unknown, nowSeconds = Math.floor(Date.now() / 1000)): PairingQrPayload {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    throw new Error("Invalid MirrorX pairing QR payload");
  }
  const value = input as Record<string, unknown>;
  const keys = Object.keys(value).sort();
  if (keys.join(",") !== "expiresAt,sessionId,ticket,version") {
    throw new Error("Unexpected fields in MirrorX pairing QR payload");
  }
  if (value.version !== MIRRORX_PAIRING_VERSION) {
    throw new Error("Unsupported MirrorX pairing QR version");
  }
  if (typeof value.sessionId !== "string" || !/^[A-Za-z0-9-]{1,64}$/.test(value.sessionId)) {
    throw new Error("Invalid pairing session identifier");
  }
  if (typeof value.ticket !== "string" || !/^[A-Za-z0-9_-]{43}$/.test(value.ticket)) {
    throw new Error("Invalid pairing ticket");
  }
  if (typeof value.expiresAt !== "number" || !Number.isSafeInteger(value.expiresAt)) {
    throw new Error("Invalid pairing ticket expiry");
  }
  const remaining = value.expiresAt - nowSeconds;
  if (remaining <= 0 || remaining > MAX_PAIRING_TICKET_LIFETIME_SECONDS) {
    throw new Error("Pairing QR code has expired or has an invalid lifetime");
  }
  return {
    version: MIRRORX_PAIRING_VERSION,
    sessionId: value.sessionId,
    ticket: value.ticket,
    expiresAt: value.expiresAt,
  };
}
