import { parseSignalingMessage, SIGNALING_PROTOCOL_VERSION, type SignalingMessage } from "../../mobile/src/transport/signalingProtocol";

export type SignalSender = (message: SignalingMessage) => Promise<void> | void;
export type ReceiverCallbacks = {
  onPeerConnection?: (peer: RTCPeerConnection | null) => void;
  onState?: (state: RTCPeerConnectionState) => void;
  onError?: (message: string) => void;
};

/**
 * Desktop-side WebRTC receiver. Instantiate only after the QR ticket has been
 * authenticated and the user has approved the peer. The caller supplies the
 * authenticated signaling transport; this class never opens a public socket.
 */
export class DesktopReceiverSession {
  readonly peer: RTCPeerConnection;
  private closed = false;
  private remoteDescriptionReady = false;
  private pendingCandidates: RTCIceCandidateInit[] = [];

  constructor(
    private readonly sessionId: string,
    private readonly sendSignal: SignalSender,
    private readonly callbacks: ReceiverCallbacks = {},
    configuration: RTCConfiguration = { iceServers: [] },
  ) {
    if (!/^[A-Za-z0-9-]{1,64}$/.test(sessionId)) throw new Error("Invalid MirrorX session ID");
    this.peer = new RTCPeerConnection(configuration);
    this.peer.ontrack = (event) => {
      this.callbacks.onPeerConnection?.(this.peer);
      // The UI subscribes to this peer's track event and renders its stream.
      if (event.track.kind === "video") this.callbacks.onState?.(this.peer.connectionState);
    };
    this.peer.onicecandidate = (event) => {
      if (this.closed) return;
      const candidate = event.candidate;
      const message: SignalingMessage = candidate
        ? {
            type: "ice_candidate",
            version: SIGNALING_PROTOCOL_VERSION,
            session_id: this.sessionId,
            candidate: candidate.candidate,
            sdp_mid: candidate.sdpMid,
            sdp_mline_index: candidate.sdpMLineIndex,
          }
        : { type: "end_of_candidates", version: SIGNALING_PROTOCOL_VERSION, session_id: this.sessionId };
      void Promise.resolve(this.sendSignal(message)).catch((error) => {
        this.callbacks.onError?.(error instanceof Error ? error.message : String(error));
      });
    };
    this.peer.onconnectionstatechange = () => this.callbacks.onState?.(this.peer.connectionState);
  }

  async acceptSignal(input: unknown): Promise<void> {
    if (this.closed) throw new Error("MirrorX session is closed");
    const message = parseSignalingMessage(input);
    if (!message || message.session_id !== this.sessionId) {
      throw new Error("Invalid or mismatched MirrorX signaling message");
    }
    switch (message.type) {
      case "offer": {
        if (this.peer.signalingState !== "stable") throw new Error("Unexpected WebRTC offer state");
        await this.peer.setRemoteDescription({ type: "offer", sdp: message.sdp });
        this.remoteDescriptionReady = true;
        for (const candidate of this.pendingCandidates.splice(0)) {
          await this.peer.addIceCandidate(candidate);
        }
        const answer = await this.peer.createAnswer();
        await this.peer.setLocalDescription(answer);
        const localSdp = this.peer.localDescription?.sdp;
        if (!localSdp) throw new Error("WebRTC did not produce an answer");
        await this.sendSignal({
          type: "answer",
          version: SIGNALING_PROTOCOL_VERSION,
          session_id: this.sessionId,
          sdp: localSdp,
        });
        break;
      }
      case "ice_candidate": {
        const candidate: RTCIceCandidateInit = {
          candidate: message.candidate,
          sdpMid: message.sdp_mid,
          sdpMLineIndex: message.sdp_mline_index,
        };
        if (!this.remoteDescriptionReady) {
          if (this.pendingCandidates.length >= 256) throw new Error("Too many ICE candidates received before the remote offer");
          this.pendingCandidates.push(candidate);
        }
        else await this.peer.addIceCandidate(candidate);
        break;
      }
      case "end_of_candidates":
        if (this.remoteDescriptionReady) await this.peer.addIceCandidate();
        break;
      case "close":
        await this.close();
        break;
      case "answer":
        throw new Error("Desktop receiver is the answerer and cannot accept a second answer");
    }
  }

  async close(): Promise<void> {
    if (this.closed) return;
    this.closed = true;
    this.pendingCandidates = [];
    this.peer.ontrack = null;
    this.peer.onicecandidate = null;
    this.peer.onconnectionstatechange = null;
    this.peer.close();
    this.callbacks.onPeerConnection?.(null);
    this.callbacks.onState?.("closed");
  }
}
