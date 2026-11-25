export type IceRecoveryState = "connected" | "disconnected" | "failed" | "closed";

export type IceRecoveryAction =
  | { type: "none" }
  | { type: "restart_ice"; attempt: number; delayMs: number }
  | { type: "give_up" };

/**
 * Bounded ICE restart policy. The transport adapter must schedule the delay,
 * call RTCPeerConnection.restartIce(), and exchange the resulting offer over
 * the already authenticated signaling channel.
 */
export class IceRecoveryPolicy {
  private attempts = 0;
  private nextRetryAt = 0;
  private closed = false;

  constructor(private readonly maxAttempts = 5) {
    if (!Number.isInteger(maxAttempts) || maxAttempts < 1 || maxAttempts > 10) {
      throw new Error("maxAttempts must be an integer from 1 to 10");
    }
  }

  onState(state: IceRecoveryState, nowMs: number): IceRecoveryAction {
    if (!Number.isFinite(nowMs) || nowMs < 0) return { type: "give_up" };
    if (state === "connected") {
      this.attempts = 0;
      this.nextRetryAt = 0;
      this.closed = false;
      return { type: "none" };
    }
    if (state === "closed") {
      this.closed = true;
      return { type: "give_up" };
    }
    if (this.closed || this.attempts >= this.maxAttempts) {
      return { type: "give_up" };
    }
    if (nowMs < this.nextRetryAt) return { type: "none" };

    this.attempts += 1;
    const delayMs = Math.min(1_000 * 2 ** Math.min(this.attempts - 1, 4), 16_000);
    this.nextRetryAt = nowMs + delayMs;
    return { type: "restart_ice", attempt: this.attempts, delayMs };
  }

  get retryCount(): number {
    return this.attempts;
  }
}
