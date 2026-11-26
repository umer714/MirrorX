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
    TruncatedAvcConfiguration,
    InvalidAvcConfigurationVersion,
    InvalidAvcConfigurationReservedBits,
    MissingSps,
    MissingPps,
    UnexpectedParameterSetType,
    TrailingAvcConfigurationData,
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


/// Parsed AVCDecoderConfigurationRecord (ISO/IEC 14496-15).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvcDecoderConfiguration {
    /// Number of bytes used by each NAL length in AVCC access units.
    pub nal_length_size: usize,
    /// SPS and PPS NAL units, including their NAL headers and without start codes.
    pub parameter_sets: Vec<Vec<u8>>,
}

/// Parse an AVCDecoderConfigurationRecord (the common MP4/MediaCodec AVCC CSD form).
///
/// The parser is deliberately strict: it validates reserved bits, version, every
/// length field, and the presence of at least one SPS and PPS. This prevents
/// malformed codec configuration from being forwarded as if it were Annex-B.
pub fn parse_avc_decoder_configuration(
    data: &[u8],
) -> Result<AvcDecoderConfiguration, H264Error> {
    if data.len() < 7 {
        return Err(H264Error::TruncatedAvcConfiguration);
    }
    if data[0] != 1 {
        return Err(H264Error::InvalidAvcConfigurationVersion);
    }
    if data[4] & 0xfc != 0xfc || data[5] & 0xe0 != 0xe0 {
        return Err(H264Error::InvalidAvcConfigurationReservedBits);
    }

    let nal_length_size = ((data[4] & 0x03) as usize) + 1;
    let sps_count = (data[5] & 0x1f) as usize;
    if sps_count == 0 {
        return Err(H264Error::MissingSps);
    }

    let mut cursor = 6usize;
    let mut parameter_sets = Vec::new();
    for _ in 0..sps_count {
        let nal = read_avc_parameter_set(data, &mut cursor)?;
        if nal[0] & 0x1f != 7 {
            return Err(H264Error::UnexpectedParameterSetType);
        }
        parameter_sets.push(nal);
    }

    let pps_count = *data.get(cursor).ok_or(H264Error::TruncatedAvcConfiguration)? as usize;
    cursor += 1;
    if pps_count == 0 {
        return Err(H264Error::MissingPps);
    }
    for _ in 0..pps_count {
        let nal = read_avc_parameter_set(data, &mut cursor)?;
        if nal[0] & 0x1f != 8 {
            return Err(H264Error::UnexpectedParameterSetType);
        }
        parameter_sets.push(nal);
    }

    // High-profile records may append chroma/bit-depth extension fields. They
    // are not needed for baseline SPS/PPS extraction; reject unexplained tails
    // rather than silently accepting corrupt/truncated records.
    if cursor != data.len() {
        return Err(H264Error::TrailingAvcConfigurationData);
    }

    Ok(AvcDecoderConfiguration { nal_length_size, parameter_sets })
}

fn read_avc_parameter_set<'a>(
    data: &'a [u8],
    cursor: &mut usize,
) -> Result<Vec<u8>, H264Error> {
    let length_bytes = data
        .get(*cursor..cursor.saturating_add(2))
        .ok_or(H264Error::TruncatedAvcConfiguration)?;
    let length = u16::from_be_bytes([length_bytes[0], length_bytes[1]]) as usize;
    *cursor += 2;
    if length == 0 {
        return Err(H264Error::EmptyNalUnit);
    }
    let end = cursor.checked_add(length).ok_or(H264Error::TruncatedAvcConfiguration)?;
    let nal = data.get(*cursor..end).ok_or(H264Error::TruncatedAvcConfiguration)?;
    *cursor = end;
    Ok(nal.to_vec())
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
    use super::{inspect_annex_b, parse_avc_decoder_configuration, to_annex_b, BoundedH264Queue, H264Error, H264Frame};

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
    fn parses_avcc_decoder_configuration_and_extracts_sps_pps() {
        // Version 1, four-byte NAL lengths, one SPS and one PPS.
        let record = [
            1, 100, 0, 31, 0xff, 0xe1,
            0, 3, 0x67, 0x64, 0x1f,
            1, 0, 2, 0x68, 0xee,
        ];
        let parsed = parse_avc_decoder_configuration(&record).unwrap();
        assert_eq!(parsed.nal_length_size, 4);
        assert_eq!(parsed.parameter_sets, vec![vec![0x67, 0x64, 0x1f], vec![0x68, 0xee]]);
    }

    #[test]
    fn rejects_invalid_or_incomplete_avcc_configuration() {
        assert_eq!(parse_avc_decoder_configuration(&[]), Err(H264Error::TruncatedAvcConfiguration));
        let mut bad_version = [1, 100, 0, 31, 0xff, 0xe1, 0, 1, 0x67, 1, 0, 1, 0x68];
        bad_version[0] = 2;
        assert_eq!(parse_avc_decoder_configuration(&bad_version), Err(H264Error::InvalidAvcConfigurationVersion));

        let no_sps = [1, 100, 0, 31, 0xff, 0xe0, 0, 1, 0x67, 1, 0, 1, 0x68];
        assert_eq!(parse_avc_decoder_configuration(&no_sps), Err(H264Error::MissingSps));

        let truncated_sps = [1, 100, 0, 31, 0xff, 0xe1, 0, 4, 0x67];
        assert_eq!(parse_avc_decoder_configuration(&truncated_sps), Err(H264Error::TruncatedAvcConfiguration));
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

/// Return the cumulative number of frames discarded by queue overflow/recovery.
#[no_mangle]
pub unsafe extern "C" fn mirrorx_h264_queue_dropped_frames(
    handle: *mut MirrorXH264QueueHandle,
) -> u64 {
    if handle.is_null() {
        return 0;
    }
    let Ok(queue) = (unsafe { &*handle }).inner.lock() else {
        return 0;
    };
    queue.dropped_frames()
}

/// Return 1 while the queue is waiting for a fresh keyframe, otherwise 0.
#[no_mangle]
pub unsafe extern "C" fn mirrorx_h264_queue_waiting_for_keyframe(
    handle: *mut MirrorXH264QueueHandle,
) -> i32 {
    if handle.is_null() {
        return 0;
    }
    let Ok(queue) = (unsafe { &*handle }).inner.lock() else {
        return 0;
    };
    if queue.waiting_for_keyframe() { 1 } else { 0 }
}

/// Destroy a queue handle created by mirrorx_h264_queue_new. Call exactly once.
#[no_mangle]
pub unsafe extern "C" fn mirrorx_h264_queue_free(handle: *mut MirrorXH264QueueHandle) {
    if !handle.is_null() {
        // SAFETY: caller must pass a live handle exactly once and never use it again.
        drop(unsafe { Box::from_raw(handle) });
    }
}


#[cfg(test)]
mod ffi_tests {
    use super::{
        mirrorx_h264_queue_free, mirrorx_h264_queue_new, mirrorx_h264_queue_pop,
        mirrorx_h264_queue_push, mirrorx_h264_queue_dropped_frames,
        mirrorx_h264_queue_waiting_for_keyframe, FFI_BUFFER_TOO_SMALL, FFI_EMPTY, FFI_INVALID, FFI_OK,
    };

    #[test]
    fn ffi_queue_copies_frames_and_retains_them_when_output_is_small() {
        unsafe {
            let queue = mirrorx_h264_queue_new(2);
            assert!(!queue.is_null());
            let payload = [0, 0, 0, 1, 0x65, 0xaa];
            assert_eq!(
                mirrorx_h264_queue_push(queue, payload.as_ptr(), payload.len(), true, 1234),
                FFI_OK
            );

            let mut required = 0usize;
            let mut keyframe = false;
            let mut pts = 0u64;
            let mut tiny = [0u8; 2];
            assert_eq!(
                mirrorx_h264_queue_pop(queue, tiny.as_mut_ptr(), tiny.len(), &mut required, &mut keyframe, &mut pts),
                FFI_BUFFER_TOO_SMALL
            );
            assert_eq!(required, payload.len());
            assert!(keyframe);
            assert_eq!(pts, 1234);

            let mut output = [0u8; 16];
            assert_eq!(
                mirrorx_h264_queue_pop(queue, output.as_mut_ptr(), output.len(), &mut required, &mut keyframe, &mut pts),
                FFI_OK
            );
            assert_eq!(&output[..required], &payload);
            assert_eq!(mirrorx_h264_queue_pop(queue, output.as_mut_ptr(), output.len(), &mut required, &mut keyframe, &mut pts), FFI_EMPTY);
            mirrorx_h264_queue_free(queue);
        }
    }

    #[test]
    fn ffi_queue_exposes_overflow_and_keyframe_recovery_metrics() {
        unsafe {
            let queue = mirrorx_h264_queue_new(1);
            assert_eq!(mirrorx_h264_queue_waiting_for_keyframe(queue), 0);
            assert_eq!(mirrorx_h264_queue_push(queue, [0x65u8].as_ptr(), 1, true, 1), FFI_OK);
            assert_eq!(mirrorx_h264_queue_push(queue, [0x41u8].as_ptr(), 1, false, 2), FFI_EMPTY);
            assert_eq!(mirrorx_h264_queue_dropped_frames(queue), 2);
            assert_eq!(mirrorx_h264_queue_waiting_for_keyframe(queue), 1);
            assert_eq!(mirrorx_h264_queue_push(queue, [0x65u8].as_ptr(), 1, true, 3), FFI_OK);
            assert_eq!(mirrorx_h264_queue_waiting_for_keyframe(queue), 0);
            mirrorx_h264_queue_free(queue);
        }
    }

    #[test]
    fn ffi_queue_rejects_invalid_inputs() {
        unsafe {
            let queue = mirrorx_h264_queue_new(1);
            assert_eq!(mirrorx_h264_queue_push(queue, std::ptr::null(), 1, false, 0), FFI_INVALID);
            assert_eq!(mirrorx_h264_queue_push(queue, [1u8].as_ptr(), 0, false, 0), FFI_INVALID);
            mirrorx_h264_queue_free(queue);
        }
    }
}
