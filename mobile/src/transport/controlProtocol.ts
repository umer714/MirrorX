/**
 * Versioned MirrorX controller protocol.
 * This validates message shape only; transport authentication and desktop consent
 * must be completed before any validated event is dispatched.
 */
export const CONTROL_PROTOCOL_VERSION = 1 as const;

export type ApprovedShortcut =
  | "escape"
  | "enter"
  | "space"
  | "arrow_up"
  | "arrow_down"
  | "arrow_left"
  | "arrow_right";

export type ControlEvent =
  | { type: "pointer_move"; dx: number; dy: number }
  | { type: "pointer_button"; button: "primary" | "secondary" | "middle"; pressed: boolean }
  | { type: "key"; key_code: number; pressed: boolean }
  | { type: "shortcut"; shortcut: ApprovedShortcut };

export type ControlMessage = {
  version: typeof CONTROL_PROTOCOL_VERSION;
  sequence: number;
  event: ControlEvent;
};

const shortcuts = new Set<ApprovedShortcut>([
  "escape",
  "enter",
  "space",
  "arrow_up",
  "arrow_down",
  "arrow_left",
  "arrow_right",
]);

function hasExactKeys(value: Record<string, unknown>, keys: string[]): boolean {
  const actual = Object.keys(value).sort();
  const expected = [...keys].sort();
  return actual.length === expected.length && actual.every((key, index) => key === expected[index]);
}

export function parseControlMessage(input: unknown): ControlMessage | null {
  if (typeof input !== "object" || input === null || Array.isArray(input)) return null;
  const message = input as Record<string, unknown>;
  if (!hasExactKeys(message, ["version", "sequence", "event"])) return null;
  if (message.version !== CONTROL_PROTOCOL_VERSION) return null;
  if (!Number.isSafeInteger(message.sequence) || (message.sequence as number) < 1) return null;

  if (typeof message.event !== "object" || message.event === null || Array.isArray(message.event)) return null;
  const event = message.event as Record<string, unknown>;
  if (typeof event.type !== "string") return null;

  switch (event.type) {
    case "pointer_move":
      if (!hasExactKeys(event, ["type", "dx", "dy"])) return null;
      if (!Number.isInteger(event.dx) || !Number.isInteger(event.dy)) return null;
      if (Math.abs(event.dx as number) > 1200 || Math.abs(event.dy as number) > 1200) return null;
      break;
    case "pointer_button":
      if (!hasExactKeys(event, ["type", "button", "pressed"])) return null;
      if (!["primary", "secondary", "middle"].includes(String(event.button))) return null;
      if (typeof event.pressed !== "boolean") return null;
      break;
    case "key":
      if (!hasExactKeys(event, ["type", "key_code", "pressed"])) return null;
      if (!Number.isInteger(event.key_code) || (event.key_code as number) < 1 || (event.key_code as number) > 0xfe) return null;
      if (typeof event.pressed !== "boolean") return null;
      break;
    case "shortcut":
      if (!hasExactKeys(event, ["type", "shortcut"])) return null;
      if (!shortcuts.has(event.shortcut as ApprovedShortcut)) return null;
      break;
    default:
      return null;
  }

  return message as unknown as ControlMessage;
}
