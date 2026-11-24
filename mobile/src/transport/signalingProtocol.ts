/**
 * MirrorX bounded WebRTC signaling contract.
 * Shape validation only: it does not authenticate peers, issue pairing tickets,
 * create a peer connection, or send messages over a network.
 */
export const SIGNALING_PROTOCOL_VERSION = 1 as const;
export const MAX_SDP_BYTES = 64 * 1024;
export const MAX_CANDIDATE_BYTES = 4 * 1024;

export type SignalingMessage =
  | { type: "offer"; version: 1; session_id: string; sdp: string }
  | { type: "answer"; version: 1; session_id: string; sdp: string }
  | { type: "ice_candidate"; version: 1; session_id: string; candidate: string; sdp_mid: string | null; sdp_mline_index: number | null }
  | { type: "end_of_candidates"; version: 1; session_id: string }
  | { type: "close"; version: 1; session_id: string };

function exactKeys(value: Record<string, unknown>, keys: string[]): boolean {
  const actual = Object.keys(value).sort();
  const expected = [...keys].sort();
  return actual.length === expected.length && actual.every((key, i) => key === expected[i]);
}

function validSessionId(value: unknown): value is string {
  return typeof value === "string" && value.length > 0 && value.length <= 64 &&
    /^[a-zA-Z0-9-]+$/.test(value);
}

export function parseSignalingMessage(input: unknown): SignalingMessage | null {
  if (typeof input !== "object" || input === null || Array.isArray(input)) return null;
  const m = input as Record<string, unknown>;
  if (m.version !== SIGNALING_PROTOCOL_VERSION || !validSessionId(m.session_id) || typeof m.type !== "string") return null;
  switch (m.type) {
    case "offer":
    case "answer":
      if (!exactKeys(m, ["type", "version", "session_id", "sdp"])) return null;
      if (typeof m.sdp !== "string" || m.sdp.length === 0 || new TextEncoder().encode(m.sdp).byteLength > MAX_SDP_BYTES) return null;
      if (!m.sdp.startsWith("v=0\r\n") && !m.sdp.startsWith("v=0\n")) return null;
      return m as SignalingMessage;
    case "ice_candidate":
      if (!exactKeys(m, ["type", "version", "session_id", "candidate", "sdp_mid", "sdp_mline_index"])) return null;
      if (typeof m.candidate !== "string" || !m.candidate.startsWith("candidate:") ||
          new TextEncoder().encode(m.candidate).byteLength > MAX_CANDIDATE_BYTES) return null;
      if (m.sdp_mid !== null && (typeof m.sdp_mid !== "string" || m.sdp_mid.length > 32)) return null;
      if (m.sdp_mline_index !== null && (!Number.isInteger(m.sdp_mline_index) || (m.sdp_mline_index as number) < 0 || (m.sdp_mline_index as number) > 65535)) return null;
      return m as SignalingMessage;
    case "end_of_candidates":
    case "close":
      if (!exactKeys(m, ["type", "version", "session_id"])) return null;
      return m as SignalingMessage;
    default:
      return null;
  }
}

export class SignalingSequenceGuard {
  private offerSeen = false;
  private answerSeen = false;
  private closed = false;

  accept(message: SignalingMessage): boolean {
    if (this.closed) return false;
    switch (message.type) {
      case "offer":
        if (this.offerSeen || this.answerSeen) return false;
        this.offerSeen = true;
        return true;
      case "answer":
        if (!this.offerSeen || this.answerSeen) return false;
        this.answerSeen = true;
        return true;
      case "ice_candidate":
      case "end_of_candidates":
        return this.offerSeen;
      case "close":
        this.closed = true;
        return true;
    }
  }

  get isClosed(): boolean { return this.closed; }
}
