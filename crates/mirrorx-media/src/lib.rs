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
    MissingAnnexBStartCode,
    EmptyNalUnit,
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


/// Metadata extracted from an Annex-B H.264 access unit.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct H264AccessUnitInfo {
    /// True when the access unit contains an IDR (NAL unit type 5).
    pub keyframe: bool,
    /// Sequence parameter sets (NAL unit payloads, including NAL header).
    pub sps: Vec<Vec<u8>>,
    /// Picture parameter sets (NAL unit payloads, including NAL header).
    pub pps: Vec<Vec<u8>>,
}

fn start_code_len(data: &[u8], offset: usize) -> Option<usize> {
    let remaining = data.get(offset..)?;
    if remaining.starts_with(&[0, 0, 0, 1]) {
        Some(4)
    } else if remaining.starts_with(&[0, 0, 1]) {
        Some(3)
    } else {
        None
    }
}

/// Inspect an Annex-B access unit for IDR, SPS, and PPS NAL units.
///
/// Returned SPS/PPS values exclude Annex-B start codes but include their NAL
/// headers, making them suitable for caching and re-sending with a keyframe.
pub fn inspect_annex_b(access_unit: &[u8]) -> Result<H264AccessUnitInfo, H264Error> {
    if access_unit.is_empty() {
        return Err(H264Error::EmptyAccessUnit);
    }

    let mut info = H264AccessUnitInfo::default();
    let mut cursor = 0usize;
    let mut found_start_code = false;

    while cursor < access_unit.len() {
        let Some(prefix_len) = start_code_len(access_unit, cursor) else {
            cursor += 1;
            continue;
        };
        found_start_code = true;
        let nal_start = cursor + prefix_len;
        let mut next = nal_start;
        while next < access_unit.len() && start_code_len(access_unit, next).is_none() {
            next += 1;
        }
        if nal_start == next {
            return Err(H264Error::EmptyNalUnit);
        }

        let nal = &access_unit[nal_start..next];
        match nal[0] & 0x1f {
            5 => info.keyframe = true,
            7 => info.sps.push(nal.to_vec()),
            8 => info.pps.push(nal.to_vec()),
            _ => {}
        }
        cursor = next;
    }

    if !found_start_code {
        return Err(H264Error::MissingAnnexBStartCode);
    }
    Ok(info)
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
    use super::{inspect_annex_b, to_annex_b, BoundedH264Queue, H264Error, H264Frame};

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
        assert_eq!(queue.dropped_frames(), 4);
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

use std::sync::Mutex;

const FFI_MAX_ACCESS_UNIT_BYTES: usize = 16 * 1024 * 1024;
const FFI_OK: i32 = 1;
const FFI_EMPTY: i32 = 0;
const FFI_INVALID: i32 = -1;
const FFI_BUFFER_TOO_SMALL: i32 = -2;

/// Opaque, thread-safe queue handle for JNI/NDK integrations.
pub struct MirrorXH264QueueHandle {
    inner: Mutex<BoundedH264Queue>,
}

/// Create a bounded H.264 queue. Capacity is clamped to at least one frame.
#[no_mangle]
pub extern "C" fn mirrorx_h264_queue_new(capacity: usize) -> *mut MirrorXH264QueueHandle {
    Box::into_raw(Box::new(MirrorXH264QueueHandle {
        inner: Mutex::new(BoundedH264Queue::new(capacity)),
    }))
}

/// Copy one encoded access unit into the queue.
/// Returns 1 when queued, 0 when dropped by keyframe recovery, and -1 on invalid input.
#[no_mangle]
pub unsafe extern "C" fn mirrorx_h264_queue_push(
    handle: *mut MirrorXH264QueueHandle,
    data: *const u8,
    len: usize,
    keyframe: bool,
    pts_micros: u64,
) -> i32 {
    if handle.is_null() || data.is_null() || len == 0 || len > FFI_MAX_ACCESS_UNIT_BYTES {
        return FFI_INVALID;
    }
    // SAFETY: caller promises data points to len readable bytes.
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    let Ok(mut queue) = (unsafe { &*handle }).inner.lock() else {
        return FFI_INVALID;
    };
    if queue.push(H264Frame {
        data: bytes.to_vec(),
        keyframe,
        pts_micros,
    }) {
        FFI_OK
    } else {
        FFI_EMPTY
    }
}

/// Copy the next queued frame into caller-owned memory.
/// Returns 1 on success, 0 when empty, -2 if output is too small (frame retained),
/// and -1 for invalid arguments or a poisoned queue.
#[no_mangle]
pub unsafe extern "C" fn mirrorx_h264_queue_pop(
    handle: *mut MirrorXH264QueueHandle,
    out: *mut u8,
    out_capacity: usize,
    out_len: *mut usize,
    out_keyframe: *mut bool,
    out_pts_micros: *mut u64,
) -> i32 {
    if handle.is_null() || out_len.is_null() || out_keyframe.is_null() || out_pts_micros.is_null() {
        return FFI_INVALID;
    }
    // SAFETY: handle must be a live handle created by mirrorx_h264_queue_new.
    let Ok(mut queue) = (unsafe { &*handle }).inner.lock() else {
        return FFI_INVALID;
    };
    let Some(frame) = queue.frames.front() else {
        return FFI_EMPTY;
    };
    unsafe {
        *out_len = frame.data.len();
        *out_keyframe = frame.keyframe;
        *out_pts_micros = frame.pts_micros;
    }
    if out.is_null() || out_capacity < frame.data.len() {
        return FFI_BUFFER_TOO_SMALL;
    }
    // SAFETY: caller supplies out_capacity writable bytes; length check ensures room.
    unsafe { std::ptr::copy_nonoverlapping(frame.data.as_ptr(), out, frame.data.len()) };
    queue.frames.pop_front();
    FFI_OK
}

/// Destroy a queue handle created by mirrorx_h264_queue_new. Call exactly once.
#[no_mangle]
pub unsafe extern "C" fn mirrorx_h264_queue_free(handle: *mut MirrorXH264QueueHandle) {
    if !handle.is_null() {
        // SAFETY: caller must pass a live handle exactly once and never use it again.
        drop(unsafe { Box::from_raw(handle) });
    }
}
