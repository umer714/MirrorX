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


/// Bounded, typed signaling messages exchanged before a WebRTC peer connection.
/// Signaling transports must authenticate the pairing ticket out-of-band and
/// enforce expiry/single-use; these structs only validate message shape.
pub const SIGNALING_PROTOCOL_VERSION: u8 = 1;
pub const MAX_SDP_BYTES: usize = 64 * 1024;
pub const MAX_CANDIDATE_BYTES: usize = 4 * 1024;
pub const MAX_SIGNALING_ID_BYTES: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SignalingMessage {
    Offer { version: u8, session_id: String, sdp: String },
    Answer { version: u8, session_id: String, sdp: String },
    IceCandidate { version: u8, session_id: String, candidate: String, sdp_mid: Option<String>, sdp_mline_index: Option<u16> },
    EndOfCandidates { version: u8, session_id: String },
    Close { version: u8, session_id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalingReject {
    UnsupportedVersion,
    InvalidSessionId,
    EmptyPayload,
    PayloadTooLarge,
    InvalidSdp,
    InvalidCandidate,
}

impl SignalingMessage {
    pub fn validate(&self) -> Result<(), SignalingReject> {
        let (version, session_id) = match self {
            Self::Offer { version, session_id, .. }
            | Self::Answer { version, session_id, .. }
            | Self::IceCandidate { version, session_id, .. }
            | Self::EndOfCandidates { version, session_id }
            | Self::Close { version, session_id } => (*version, session_id),
        };
        if version != SIGNALING_PROTOCOL_VERSION { return Err(SignalingReject::UnsupportedVersion); }
        if session_id.is_empty() || session_id.len() > MAX_SIGNALING_ID_BYTES
            || !session_id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
            return Err(SignalingReject::InvalidSessionId);
        }
        match self {
            Self::Offer { sdp, .. } | Self::Answer { sdp, .. } => {
                if sdp.is_empty() { return Err(SignalingReject::EmptyPayload); }
                if sdp.len() > MAX_SDP_BYTES { return Err(SignalingReject::PayloadTooLarge); }
                if !sdp.starts_with("v=0\r\n") && !sdp.starts_with("v=0\n") {
                    return Err(SignalingReject::InvalidSdp);
                }
            }
            Self::IceCandidate { candidate, sdp_mid, .. } => {
                if candidate.is_empty() { return Err(SignalingReject::EmptyPayload); }
                if candidate.len() > MAX_CANDIDATE_BYTES
                    || sdp_mid.as_ref().is_some_and(|mid| mid.len() > 32) {
                    return Err(SignalingReject::PayloadTooLarge);
                }
                if !candidate.starts_with("candidate:") { return Err(SignalingReject::InvalidCandidate); }
            }
            Self::EndOfCandidates { .. } | Self::Close { .. } => {}
        }
        Ok(())
    }
}

/// Enforces per-session signaling order without performing network I/O.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SignalingGate {
    offer_seen: bool,
    answer_seen: bool,
    closed: bool,
}

impl SignalingGate {
    pub fn accept(&mut self, message: &SignalingMessage) -> Result<(), SignalingReject> {
        message.validate()?;
        if self.closed { return Err(SignalingReject::InvalidSessionId); }
        match message {
            SignalingMessage::Offer { .. } if !self.offer_seen && !self.answer_seen => {
                self.offer_seen = true;
                Ok(())
            }
            SignalingMessage::Answer { .. } if self.offer_seen && !self.answer_seen => {
                self.answer_seen = true;
                Ok(())
            }
            SignalingMessage::IceCandidate { .. } if self.offer_seen && !self.closed => Ok(()),
            SignalingMessage::EndOfCandidates { .. } if self.offer_seen && !self.closed => Ok(()),
            SignalingMessage::Close { .. } => {
                self.closed = true;
                Ok(())
            }
            _ => Err(SignalingReject::InvalidSessionId),
        }
    }

    pub fn is_closed(&self) -> bool { self.closed }
}

    #[test]
    fn signaling_messages_are_bounded_and_shape_checked() {
        let offer = SignalingMessage::Offer {
            version: SIGNALING_PROTOCOL_VERSION,
            session_id: "mirrorx-123".into(),
            sdp: "v=0\r\no=- 1 1 IN IP4 127.0.0.1\r\n".into(),
        };
        assert_eq!(offer.validate(), Ok(()));
        let escaped_line_endings = SignalingMessage::Offer {
            version: 1, session_id: "mirrorx-123".into(), sdp: r"v=0\r\n".into(),
        };
        assert_eq!(escaped_line_endings.validate(), Err(SignalingReject::InvalidSdp));
        let invalid = SignalingMessage::IceCandidate {
            version: 1, session_id: "mirrorx-123".into(),
            candidate: "not-a-candidate".into(), sdp_mid: None, sdp_mline_index: None,
        };
        assert_eq!(invalid.validate(), Err(SignalingReject::InvalidCandidate));
        let oversized = SignalingMessage::Offer {
            version: 1, session_id: "mirrorx-123".into(), sdp: format!("v=0\n{}", "x".repeat(MAX_SDP_BYTES)),
        };
        assert_eq!(oversized.validate(), Err(SignalingReject::PayloadTooLarge));
    }

    #[test]
    fn signaling_gate_requires_offer_then_answer_and_stops_after_close() {
        let offer = SignalingMessage::Offer { version: 1, session_id: "s-1".into(), sdp: "v=0\n".into() };
        let answer = SignalingMessage::Answer { version: 1, session_id: "s-1".into(), sdp: "v=0\n".into() };
        let close = SignalingMessage::Close { version: 1, session_id: "s-1".into() };
        let mut gate = SignalingGate::default();
        assert_eq!(gate.accept(&answer), Err(SignalingReject::InvalidSessionId));
        gate.accept(&offer).unwrap();
        gate.accept(&answer).unwrap();
        gate.accept(&close).unwrap();
        assert!(gate.is_closed());
        assert_eq!(gate.accept(&offer), Err(SignalingReject::InvalidSessionId));
    }



#[cfg(test)]
mod signaling_tests {
    use super::*;


}


/// Short-lived, single-use pairing tickets.
///
/// The caller MUST generate ticket bytes with the platform CSPRNG. This store
/// keeps only in-memory tickets and compares secrets in constant time. It is
/// intentionally not a replacement for authenticated transport encryption.
#[derive(Debug, Clone)]
pub struct PairingTicketStore {
    tickets: Vec<StoredPairingTicket>,
    max_tickets: usize,
}

#[derive(Debug, Clone)]
struct StoredPairingTicket {
    secret: [u8; 32],
    session_id: String,
    expires_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairingTicketError {
    InvalidSessionId,
    InvalidLifetime,
    CapacityReached,
    DuplicateTicket,
    NotFoundOrExpired,
}

impl Default for PairingTicketStore {
    fn default() -> Self {
        Self::new(128)
    }
}

impl PairingTicketStore {
    pub fn new(max_tickets: usize) -> Self {
        Self { tickets: Vec::new(), max_tickets: max_tickets.max(1) }
    }

    /// Register a 256-bit secret created by a cryptographically secure RNG.
    /// Lifetimes are limited to five minutes to reduce QR-code replay exposure.
    pub fn issue(
        &mut self,
        secret: [u8; 32],
        session_id: &str,
        now_ms: u64,
        lifetime_ms: u64,
    ) -> Result<(), PairingTicketError> {
        self.tickets.retain(|ticket| ticket.expires_at_ms > now_ms);
        if session_id.is_empty() || session_id.len() > MAX_SIGNALING_ID_BYTES
            || !session_id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
            return Err(PairingTicketError::InvalidSessionId);
        }
        if !(5_000..=300_000).contains(&lifetime_ms) {
            return Err(PairingTicketError::InvalidLifetime);
        }
        if self.tickets.len() >= self.max_tickets {
            return Err(PairingTicketError::CapacityReached);
        }
        if self.tickets.iter().any(|ticket| constant_time_eq(&ticket.secret, &secret)) {
            return Err(PairingTicketError::DuplicateTicket);
        }
        self.tickets.push(StoredPairingTicket {
            secret,
            session_id: session_id.to_owned(),
            expires_at_ms: now_ms.saturating_add(lifetime_ms),
        });
        Ok(())
    }

    /// Consume a ticket exactly once. Expired and unknown tickets share the
    /// same error so callers cannot distinguish stored session identifiers.
    pub fn consume(
        &mut self,
        supplied_secret: &[u8; 32],
        now_ms: u64,
    ) -> Result<String, PairingTicketError> {
        let position = self.tickets.iter().position(|ticket| {
            constant_time_eq(&ticket.secret, supplied_secret)
        });
        let Some(position) = position else {
            self.tickets.retain(|ticket| ticket.expires_at_ms > now_ms);
            return Err(PairingTicketError::NotFoundOrExpired);
        };
        let ticket = self.tickets.remove(position);
        if ticket.expires_at_ms <= now_ms {
            return Err(PairingTicketError::NotFoundOrExpired);
        }
        Ok(ticket.session_id)
    }

    pub fn pending_count(&self, now_ms: u64) -> usize {
        self.tickets.iter().filter(|ticket| ticket.expires_at_ms > now_ms).count()
    }
}

fn constant_time_eq(left: &[u8; 32], right: &[u8; 32]) -> bool {
    left.iter().zip(right.iter()).fold(0u8, |diff, (a, b)| diff | (a ^ b)) == 0
}

/// ICE connection recovery policy. It only decides when to retry; adapters
/// must perform ICE restart and signaling over the authenticated session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IceLinkState {
    Connected,
    Disconnected,
    Failed,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IceRecoveryAction {
    None,
    RestartIce { attempt: u8, delay_ms: u64 },
    GiveUp,
}

#[derive(Debug, Clone)]
pub struct IceRecoveryPolicy {
    attempts: u8,
    max_attempts: u8,
    next_retry_at_ms: Option<u64>,
    state: IceLinkState,
}

impl Default for IceRecoveryPolicy {
    fn default() -> Self {
        Self { attempts: 0, max_attempts: 5, next_retry_at_ms: None, state: IceLinkState::Connected }
    }
}

impl IceRecoveryPolicy {
    pub fn on_state(&mut self, state: IceLinkState, now_ms: u64) -> IceRecoveryAction {
        self.state = state;
        match state {
            IceLinkState::Connected => {
                self.attempts = 0;
                self.next_retry_at_ms = None;
                IceRecoveryAction::None
            }
            IceLinkState::Closed => {
                self.next_retry_at_ms = None;
                IceRecoveryAction::GiveUp
            }
            IceLinkState::Disconnected | IceLinkState::Failed => {
                if self.attempts >= self.max_attempts {
                    self.next_retry_at_ms = None;
                    return IceRecoveryAction::GiveUp;
                }
                let delay_ms = (1_000u64.saturating_mul(1u64 << self.attempts.min(4))).min(16_000);
                let due = self.next_retry_at_ms.unwrap_or(now_ms);
                if now_ms < due {
                    return IceRecoveryAction::None;
                }
                self.attempts += 1;
                self.next_retry_at_ms = Some(now_ms.saturating_add(delay_ms));
                IceRecoveryAction::RestartIce { attempt: self.attempts, delay_ms }
            }
        }
    }

    pub fn attempts(&self) -> u8 { self.attempts }
}


#[cfg(test)]
mod pairing_and_recovery_tests {
    use super::*;

    #[test]
    fn pairing_ticket_is_single_use_and_expires() {
        let mut store = PairingTicketStore::default();
        let secret = [0x5au8; 32];
        store.issue(secret, "mx-session-1", 10_000, 30_000).unwrap();
        assert_eq!(store.consume(&secret, 10_001), Ok("mx-session-1".into()));
        assert_eq!(store.consume(&secret, 10_002), Err(PairingTicketError::NotFoundOrExpired));

        let expired = [0x33u8; 32];
        store.issue(expired, "mx-session-2", 20_000, 5_000).unwrap();
        assert_eq!(store.consume(&expired, 25_000), Err(PairingTicketError::NotFoundOrExpired));
    }

    #[test]
    fn pairing_ticket_rejects_unsafe_lifetime_duplicate_and_bad_session() {
        let mut store = PairingTicketStore::new(1);
        let secret = [7u8; 32];
        assert_eq!(store.issue(secret, "../bad", 0, 10_000), Err(PairingTicketError::InvalidSessionId));
        assert_eq!(store.issue(secret, "s-1", 0, 999), Err(PairingTicketError::InvalidLifetime));
        store.issue(secret, "s-1", 0, 10_000).unwrap();
        assert_eq!(store.issue(secret, "s-2", 0, 10_000), Err(PairingTicketError::DuplicateTicket));
        assert_eq!(store.issue([8u8; 32], "s-3", 0, 10_000), Err(PairingTicketError::CapacityReached));
    }

    #[test]
    fn ice_recovery_uses_bounded_backoff_and_resets_on_reconnect() {
        let mut policy = IceRecoveryPolicy::default();
        assert_eq!(policy.on_state(IceLinkState::Disconnected, 100), IceRecoveryAction::RestartIce { attempt: 1, delay_ms: 1_000 });
        assert_eq!(policy.on_state(IceLinkState::Failed, 500), IceRecoveryAction::None);
        assert_eq!(policy.on_state(IceLinkState::Failed, 1_100), IceRecoveryAction::RestartIce { attempt: 2, delay_ms: 2_000 });
        assert_eq!(policy.on_state(IceLinkState::Connected, 1_200), IceRecoveryAction::None);
        assert_eq!(policy.attempts(), 0);
        assert_eq!(policy.on_state(IceLinkState::Closed, 1_300), IceRecoveryAction::GiveUp);
    }
}
