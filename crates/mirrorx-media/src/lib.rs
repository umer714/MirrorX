use std::collections::VecDeque;

/// Errors returned while validating or normalizing encoded H.264 access units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum H264Error {
    EmptyAccessUnit,
    MissingNalLengthSize,
    InvalidNalLengthSize,
    TruncatedNalLength,
    ZeroLengthNal,
    NalExceedsAccessUnit,
}

const ANNEX_B_START_CODE: [u8; 4] = [0, 0, 0, 1];

/// Convert a length-prefixed H.264 access unit to Annex-B.
///
/// If the input already begins with a 3- or 4-byte Annex-B start code, it is
/// returned unchanged. For AVCC-style input, `nal_length_size` must be the
/// number of big-endian bytes used for each NAL length (1 through 4).
pub fn to_annex_b(
    access_unit: &[u8],
    nal_length_size: Option<usize>,
) -> Result<Vec<u8>, H264Error> {
    if access_unit.is_empty() {
        return Err(H264Error::EmptyAccessUnit);
    }

    if access_unit.starts_with(&[0, 0, 1]) || access_unit.starts_with(&ANNEX_B_START_CODE) {
        return Ok(access_unit.to_vec());
    }

    let length_size = nal_length_size.ok_or(H264Error::MissingNalLengthSize)?;
    if !(1..=4).contains(&length_size) {
        return Err(H264Error::InvalidNalLengthSize);
    }

    let mut output = Vec::with_capacity(access_unit.len() + 16);
    let mut cursor = 0usize;

    while cursor < access_unit.len() {
        let length_end = cursor
            .checked_add(length_size)
            .filter(|end| *end <= access_unit.len())
            .ok_or(H264Error::TruncatedNalLength)?;

        let mut nal_length = 0usize;
        for byte in &access_unit[cursor..length_end] {
            nal_length = nal_length
                .checked_mul(256)
                .and_then(|value| value.checked_add(*byte as usize))
                .ok_or(H264Error::NalExceedsAccessUnit)?;
        }
        if nal_length == 0 {
            return Err(H264Error::ZeroLengthNal);
        }

        let nal_end = length_end
            .checked_add(nal_length)
            .filter(|end| *end <= access_unit.len())
            .ok_or(H264Error::NalExceedsAccessUnit)?;

        output.extend_from_slice(&ANNEX_B_START_CODE);
        output.extend_from_slice(&access_unit[length_end..nal_end]);
        cursor = nal_end;
    }

    if output.is_empty() {
        return Err(H264Error::EmptyAccessUnit);
    }
    Ok(output)
}

/// An already encoded H.264 access unit. The payload is owned so it can safely
/// cross the native capture/transport boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct H264Frame {
    pub data: Vec<u8>,
    pub keyframe: bool,
    pub pts_micros: u64,
}

/// A latency-oriented bounded queue. On overflow it drops stale queued frames
/// and waits for a fresh keyframe before accepting predictive frames again.
#[derive(Debug)]
pub struct BoundedH264Queue {
    capacity: usize,
    frames: VecDeque<H264Frame>,
    waiting_for_keyframe: bool,
    dropped_frames: u64,
}

impl BoundedH264Queue {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            frames: VecDeque::new(),
            waiting_for_keyframe: false,
            dropped_frames: 0,
        }
    }

    /// Returns true when the frame was queued; false when it was discarded.
    pub fn push(&mut self, frame: H264Frame) -> bool {
        if self.waiting_for_keyframe && !frame.keyframe {
            self.dropped_frames = self.dropped_frames.saturating_add(1);
            return false;
        }

        if self.frames.len() >= self.capacity {
            self.dropped_frames = self
                .dropped_frames
                .saturating_add(self.frames.len() as u64);
            self.frames.clear();
            self.waiting_for_keyframe = true;

            if !frame.keyframe {
                self.dropped_frames = self.dropped_frames.saturating_add(1);
                return false;
            }
        }

        if frame.keyframe {
            self.waiting_for_keyframe = false;
        }
        self.frames.push_back(frame);
        true
    }

    pub fn pop(&mut self) -> Option<H264Frame> {
        self.frames.pop_front()
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn dropped_frames(&self) -> u64 {
        self.dropped_frames
    }

    pub fn waiting_for_keyframe(&self) -> bool {
        self.waiting_for_keyframe
    }
}

#[cfg(test)]
mod tests {
    use super::{to_annex_b, BoundedH264Queue, H264Error, H264Frame};

    fn frame(keyframe: bool, pts_micros: u64) -> H264Frame {
        H264Frame {
            data: vec![0, 0, 0, 1, 0x65],
            keyframe,
            pts_micros,
        }
    }

    #[test]
    fn preserves_annex_b_access_units() {
        let input = [0, 0, 1, 0x65, 0x88];
        assert_eq!(to_annex_b(&input, None).unwrap(), input);
    }

    #[test]
    fn converts_multiple_length_prefixed_nals() {
        let input = [2, 0x67, 0x64, 1, 0x68];
        let expected = [0, 0, 0, 1, 0x67, 0x64, 0, 0, 0, 1, 0x68];
        assert_eq!(to_annex_b(&input, Some(1)).unwrap(), expected);
    }

    #[test]
    fn rejects_malformed_h264_lengths() {
        assert_eq!(to_annex_b(&[], Some(4)), Err(H264Error::EmptyAccessUnit));
        assert_eq!(to_annex_b(&[1, 0x65], None), Err(H264Error::MissingNalLengthSize));
        assert_eq!(to_annex_b(&[1, 0x65], Some(0)), Err(H264Error::InvalidNalLengthSize));
        assert_eq!(to_annex_b(&[0], Some(2)), Err(H264Error::TruncatedNalLength));
        assert_eq!(to_annex_b(&[0], Some(1)), Err(H264Error::ZeroLengthNal));
        assert_eq!(to_annex_b(&[3, 0x65], Some(1)), Err(H264Error::NalExceedsAccessUnit));
    }

    #[test]
    fn queue_is_bounded_and_recovers_at_keyframe() {
        let mut queue = BoundedH264Queue::new(2);
        assert!(queue.push(frame(true, 1)));
        assert!(queue.push(frame(false, 2)));
        assert!(!queue.push(frame(false, 3)));
        assert_eq!(queue.len(), 0);
        assert!(queue.waiting_for_keyframe());
        assert!(!queue.push(frame(false, 4)));
        assert!(queue.push(frame(true, 5)));
        assert!(!queue.waiting_for_keyframe());
        assert_eq!(queue.pop().unwrap().pts_micros, 5);
        assert_eq!(queue.dropped_frames(), 3);
    }

    #[test]
    fn keyframe_overflow_replaces_stale_queue() {
        let mut queue = BoundedH264Queue::new(1);
        assert!(queue.push(frame(true, 1)));
        assert!(queue.push(frame(true, 2)));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop().unwrap().pts_micros, 2);
        assert_eq!(queue.dropped_frames(), 1);
    }
}
