use serde::{Deserialize, Serialize};

/// Transport lifecycle shared by desktop and mobile adapters.
/// This crate defines protocol state only; it does not open sockets or claim a peer is connected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionPhase {
    Idle,
    Pairing,
    Connected,
    Streaming,
    Closing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionRole {
    Sender,
    Controller,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionState {
    pub phase: SessionPhase,
    pub role: Option<SessionRole>,
    pub session_id: Option<String>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self { phase: SessionPhase::Idle, role: None, session_id: None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    InvalidSessionId,
    InvalidTransition,
}

impl SessionState {
    pub fn begin_pairing(&mut self, role: SessionRole, session_id: &str) -> Result<(), SessionError> {
        let valid = !session_id.is_empty()
            && session_id.len() <= 64
            && session_id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-');
        if !valid { return Err(SessionError::InvalidSessionId); }
        if self.phase != SessionPhase::Idle { return Err(SessionError::InvalidTransition); }
        self.phase = SessionPhase::Pairing;
        self.role = Some(role);
        self.session_id = Some(session_id.to_owned());
        Ok(())
    }

    pub fn mark_connected(&mut self) -> Result<(), SessionError> {
        if self.phase != SessionPhase::Pairing { return Err(SessionError::InvalidTransition); }
        self.phase = SessionPhase::Connected;
        Ok(())
    }

    pub fn mark_streaming(&mut self) -> Result<(), SessionError> {
        if self.phase != SessionPhase::Connected { return Err(SessionError::InvalidTransition); }
        self.phase = SessionPhase::Streaming;
        Ok(())
    }

    pub fn close(&mut self) {
        self.phase = SessionPhase::Closing;
        self.role = None;
        self.session_id = None;
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PointerButton { Primary, Secondary, Middle }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovedShortcut { Escape, Enter, Space, ArrowUp, ArrowDown, ArrowLeft, ArrowRight }

/// Deliberately narrow input allow-list. There is no shell, process, or arbitrary command event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlEvent {
    PointerMove { dx: i16, dy: i16 },
    PointerButton { button: PointerButton, pressed: bool },
    Key { key_code: u16, pressed: bool },
    Shortcut { shortcut: ApprovedShortcut },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlMessage {
    pub sequence: u64,
    pub event: ControlEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlReject {
    ReplayOrOutOfOrder,
    RateLimited,
    InvalidPointerDelta,
    InvalidKeyCode,
}

/// Per-session guard. Caller supplies monotonic milliseconds from a monotonic clock.
#[derive(Debug, Clone)]
pub struct ControlGate {
    last_sequence: Option<u64>,
    window_started_ms: Option<u64>,
    accepted_in_window: u8,
}

impl Default for ControlGate {
    fn default() -> Self {
        Self { last_sequence: None, window_started_ms: None, accepted_in_window: 0 }
    }
}

impl ControlGate {
    pub fn validate(&mut self, message: &ControlMessage, now_ms: u64) -> Result<(), ControlReject> {
        if self.last_sequence.is_some_and(|last| message.sequence <= last) {
            return Err(ControlReject::ReplayOrOutOfOrder);
        }
        match &message.event {
            ControlEvent::PointerMove { dx, dy } if dx.abs() > 1200 || dy.abs() > 1200 => {
                return Err(ControlReject::InvalidPointerDelta);
            }
            ControlEvent::Key { key_code, .. } if !(1..=0xFE).contains(key_code) => {
                return Err(ControlReject::InvalidKeyCode);
            }
            _ => {}
        }
        let start = self.window_started_ms.get_or_insert(now_ms);
        if now_ms.saturating_sub(*start) >= 1_000 {
            self.window_started_ms = Some(now_ms);
            self.accepted_in_window = 0;
        }
        if self.accepted_in_window >= 60 {
            return Err(ControlReject::RateLimited);
        }
        self.accepted_in_window += 1;
        self.last_sequence = Some(message.sequence);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_requires_ordered_explicit_transitions() {
        let mut state = SessionState::default();
        assert_eq!(state.mark_connected(), Err(SessionError::InvalidTransition));
        state.begin_pairing(SessionRole::Sender, "session-123").unwrap();
        assert_eq!(state.phase, SessionPhase::Pairing);
        state.mark_connected().unwrap();
        state.mark_streaming().unwrap();
        assert_eq!(state.phase, SessionPhase::Streaming);
        state.close();
        assert_eq!(state.phase, SessionPhase::Closing);
        state.reset();
        assert_eq!(state, SessionState::default());
    }

    #[test]
    fn rejects_unsafe_session_identifiers_and_duplicate_pairing() {
        let mut state = SessionState::default();
        assert_eq!(state.begin_pairing(SessionRole::Controller, "../host"), Err(SessionError::InvalidSessionId));
        state.begin_pairing(SessionRole::Controller, "abc123").unwrap();
        assert_eq!(state.begin_pairing(SessionRole::Sender, "second"), Err(SessionError::InvalidTransition));
    }

    #[test]
    fn control_gate_rejects_replay_and_extreme_pointer_delta() {
        let mut gate = ControlGate::default();
        let first = ControlMessage { sequence: 1, event: ControlEvent::PointerMove { dx: 12, dy: -8 } };
        gate.validate(&first, 100).unwrap();
        assert_eq!(gate.validate(&first, 110), Err(ControlReject::ReplayOrOutOfOrder));
        let invalid = ControlMessage { sequence: 2, event: ControlEvent::PointerMove { dx: 5000, dy: 0 } };
        assert_eq!(gate.validate(&invalid, 120), Err(ControlReject::InvalidPointerDelta));
    }

    #[test]
    fn limits_control_rate_to_sixty_events_per_second() {
        let mut gate = ControlGate::default();
        for sequence in 1..=60 {
            gate.validate(&ControlMessage { sequence, event: ControlEvent::Shortcut { shortcut: ApprovedShortcut::Enter } }, 50).unwrap();
        }
        assert_eq!(
            gate.validate(&ControlMessage { sequence: 61, event: ControlEvent::Shortcut { shortcut: ApprovedShortcut::Enter } }, 50),
            Err(ControlReject::RateLimited)
        );
        gate.validate(&ControlMessage { sequence: 61, event: ControlEvent::Shortcut { shortcut: ApprovedShortcut::Enter } }, 1_050).unwrap();
    }
}
