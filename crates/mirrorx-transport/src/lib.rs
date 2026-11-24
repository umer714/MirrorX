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
    pub version: u8,
    pub sequence: u64,
    pub event: ControlEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlReject {
    ReplayOrOutOfOrder,
    RateLimited,
    InvalidPointerDelta,
    InvalidKeyCode,
    UnsupportedVersion,
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
        if message.version != 1 { return Err(ControlReject::UnsupportedVersion); }
        if self.last_sequence.is_some_and(|last| message.sequence <= last) {
            return Err(ControlReject::ReplayOrOutOfOrder);
        }
        match &message.event {
            ControlEvent::PointerMove { dx, dy } if dx.unsigned_abs() > 1200 || dy.unsigned_abs() > 1200 => {
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


/// Explicit consent gates shared by desktop and mobile adapters.
///
/// This is an authorization state model, not peer authentication. Adapters must
/// call the grant methods only after the corresponding user/OS approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ConsentGate {
    peer_approved: bool,
    capture_approved: bool,
    control_approved: bool,
}

impl ConsentGate {
    pub fn approve_peer(&mut self) { self.peer_approved = true; }

    pub fn grant_capture(&mut self) -> Result<(), ConsentError> {
        if !self.peer_approved { return Err(ConsentError::PeerNotApproved); }
        self.capture_approved = true;
        Ok(())
    }

    pub fn grant_control(&mut self) -> Result<(), ConsentError> {
        if !self.peer_approved { return Err(ConsentError::PeerNotApproved); }
        self.control_approved = true;
        Ok(())
    }

    pub fn can_send_video(&self) -> bool {
        self.peer_approved && self.capture_approved
    }

    pub fn can_accept_control(&self) -> bool {
        self.peer_approved && self.control_approved
    }

    /// Revoke all permissions immediately when either side ends the session.
    pub fn revoke(&mut self) { *self = Self::default(); }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentError { PeerNotApproved }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consent_requires_peer_approval_and_is_revocable() {
        let mut consent = ConsentGate::default();
        assert_eq!(consent.grant_capture(), Err(ConsentError::PeerNotApproved));
        assert_eq!(consent.grant_control(), Err(ConsentError::PeerNotApproved));
        assert!(!consent.can_send_video());
        assert!(!consent.can_accept_control());

        consent.approve_peer();
        consent.grant_capture().unwrap();
        assert!(consent.can_send_video());
        assert!(!consent.can_accept_control());

        consent.grant_control().unwrap();
        assert!(consent.can_accept_control());
        consent.revoke();
        assert!(!consent.can_send_video());
        assert!(!consent.can_accept_control());
    }

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
        let first = ControlMessage { version: 1, sequence: 1, event: ControlEvent::PointerMove { dx: 12, dy: -8 } };
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
