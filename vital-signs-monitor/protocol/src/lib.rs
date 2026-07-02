//! Shared wire protocol between `control-core` and `ui-dashboard`.
//!
//! YOUR TASK: implement the encode/decode logic for a simple binary framing
//! protocol. This crate is deliberately dependency-free (and could become
//! `no_std` later) so it could run unmodified on a real MCU someday.
//!
//! ## Frame format (little-endian), decide on this BEFORE coding:
//!
//! ```text
//! [0]      START   = 0xAA           (1 byte, a fixed marker)
//! [1]      TYPE    = u8             (which FrameType this is)
//! [2..4]   LEN     = u16            (length of PAYLOAD in bytes)
//! [4..N]   PAYLOAD = [u8; LEN]
//! [N..N+1] CHECKSUM = u8            (e.g. XOR of TYPE, LEN bytes, and payload)
//! ```
//!
//! Why a checksum at all? Because this will cross a process boundary (and
//! someday maybe a real UART) - corrupted bytes are possible and you want to
//! detect them rather than silently decode garbage.

#![deny(missing_docs)]

/// Marks the start of every frame on the wire.
pub const START_BYTE: u8 = 0xAA;

/// Maximum payload size in bytes. Pick something with headroom over your
/// largest actual payload (look at VitalsSample/ConfigUpdate sizes below).
pub const MAX_PAYLOAD: usize = 256;

/// The kind of frame being sent, encoded as the wire's TYPE byte.
///
/// HINT: `#[repr(u8)]` lets you cast `FrameType::VitalsSample as u8` to get
/// the literal wire byte value directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType {
    /// A batch of vitals samples (HR, SpO2, Temp) from control-core to UI.
    VitalsSample = 0x01,
    /// An alarm state transition from control-core to UI.
    AlarmState = 0x02,
    /// A configuration change from UI to control-core (thresholds, patient ID).
    ConfigUpdate = 0x03,
    /// A generic acknowledgement, either direction.
    Ack = 0x04,
}

impl FrameType {
    /// Convert a raw byte into a FrameType, if it's a known value.
    ///
    /// HINT: a `match` on `b` returning `Some(FrameType::X)` for each known
    /// byte and `_ => None` for anything else. This is the inverse of the
    /// `as u8` cast you'd use to go the other direction.
    pub fn from_u8(b: u8) -> Option<Self> {
        
        match b {
            0x01 => Some(FrameType::VitalsSample), 
            0x02 => Some(FrameType::AlarmState),
            0x03 => Some(FrameType::ConfigUpdate),
            0x04 => Some(FrameType::Ack),
            _ => None,
        }
    }
}

/// Severity level for the alarm state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlarmLevel {
    /// All vitals within normal thresholds.
    Normal = 0,
    /// At least one vital is outside the normal band but not critical.
    Warning = 1,
    /// At least one vital is in a critical range — requires immediate attention.
    Critical = 2,
}

impl AlarmLevel {
    /// Convert a raw byte into an AlarmLevel, defaulting to Normal on unknown input.
    ///
    /// HINT: unlike FrameType::from_u8, this one should NEVER fail — an
    /// unrecognized byte should fail safe to Normal... or should it fail
    /// safe to Critical? Think about which default is actually safer for a
    /// monitor, then justify your choice in a comment.
    pub fn from_u8(b: u8) -> Self {
        
        match b {
            0 => AlarmLevel::Normal,
            1 => AlarmLevel::Warning,
            2 => AlarmLevel::Critical,
            _ => AlarmLevel::Critical, // If you were to receive Junk Data, you should immediate check up on the patient and fix it
        }
    }
}

/// A single batch of vitals, as sent from control-core to the UI.
///
/// Field sizes already chosen for you — notice everything is a fixed-point
/// integer (e.g. spo2 as percent*10) rather than a float. Why might a wire
/// protocol prefer that over sending f32/f64 directly?
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VitalsSample {
    /// Milliseconds since control-core process start.
    pub timestamp_ms: u32, // 4 bytes
    /// Heart rate in beats per minute.
    pub heart_rate_bpm: u16, // 2
    /// Blood oxygen saturation, in percent * 10 (e.g. 975 == 97.5%).
    pub spo2_permille: u16, // 2 
    /// Body temperature in degrees Celsius * 100 (e.g. 3712 == 37.12C).
    pub temp_centi_c: u16, // 2
    /// Most recent raw ECG sample (for waveform plotting), signed millivolts * 100.
    pub ecg_sample_mv100: i16, // 2
}

impl VitalsSample {
    // HINT: add up the byte sizes of each field above (u32=4, u16=2, u16=2,
    // u16=2, i16=2) to get this constant right.
    const ENCODED_LEN: usize = 12; // u32(4) and u16(2) 

    /// Serialize into a fixed-size byte buffer.
    ///
    /// HINT: build a `[0u8; Self::ENCODED_LEN]` buffer, then for each field
    /// use `buf[start..end].copy_from_slice(&self.field.to_le_bytes())`.
    /// Each numeric type has a `.to_le_bytes()` method that returns a
    /// fixed-size array — that's what you're copying in.
    pub fn encode(&self) -> [u8; Self::ENCODED_LEN] {
        
        let mut buf = [0u8;Self::ENCODED_LEN];
        buf[0..4].copy_from_slice(&self.timestamp_ms.to_le_bytes());
        buf[4..6].copy_from_slice(&self.heart_rate_bpm.to_le_bytes());
        buf[6..8].copy_from_slice(&self.spo2_permille.to_le_bytes());
        buf[8..10].copy_from_slice(&self.temp_centi_c.to_le_bytes());
        buf[10..12].copy_from_slice(&self.ecg_sample_mv100.to_le_bytes());
        return buf;
    }

    


    /// Deserialize from a byte slice. Returns None if too short.
    ///
    /// HINT: the inverse of encode — `u32::from_le_bytes(buf[0..4].try_into().ok()?)`
    /// is the pattern for each field. The `?` on `try_into().ok()?` is what
    /// lets you bail out with `None` if a slice conversion ever fails.
    pub fn decode(buf: &[u8]) -> Option<Self> {

        let timestamp_ms : u32;
        let heart_rate_bpm : u16;
        let spo2_permille : u16;
        let temp_centi_c : u16;
        let ecg_sample_mv100 : i16;

        if buf.len() >= Self::ENCODED_LEN {
            timestamp_ms = u32::from_le_bytes(buf[0..4].try_into().ok()?);
            heart_rate_bpm = u16::from_le_bytes(buf[4..6].try_into().ok()?);
            spo2_permille = u16::from_le_bytes(buf[6..8].try_into().ok()?);
            temp_centi_c = u16::from_le_bytes(buf[8..10].try_into().ok()?);
            ecg_sample_mv100 = i16::from_le_bytes(buf[10..12].try_into().ok()?);
        } else {
            return None; 
        }
        return Some(Self {timestamp_ms, heart_rate_bpm, spo2_permille, temp_centi_c, ecg_sample_mv100});
    }
}

/// An alarm state transition event, sent from control-core to the UI.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlarmEvent {
    /// Milliseconds since control-core process start.
    pub timestamp_ms: u32, // 4 byte 
    /// New alarm level.
    pub level: AlarmLevel, // 1 byte 
    /// Which vital triggered this transition. 0=HR, 1=SpO2, 2=Temp, 255=cleared/none.
    pub source_vital: u8, // 1 byte 
}

impl AlarmEvent {
    const ENCODED_LEN: usize = 6; // TODO: fix this (timestamp_ms + level byte + source_vital byte)

    /// Serialize into a fixed-size byte buffer.
    pub fn encode(&self) -> [u8; Self::ENCODED_LEN] {
        // HINT: `self.level as u8` converts the enum to its wire byte.
        let mut buf = [0u8;Self::ENCODED_LEN];
        buf[0..4].copy_from_slice(&self.timestamp_ms.to_le_bytes());
        buf[4] = self.level as u8;
        buf[5] = self.source_vital;
        return buf; 
    }

    /// Deserialize from a byte slice. Returns None if too short.
    pub fn decode(buf: &[u8]) -> Option<Self> {
        // HINT: use AlarmLevel::from_u8(buf[4]) for the level field.
        let timestamp_ms : u32;
        let level : AlarmLevel;
        let source_vital : u8;
        if buf.len() >= Self::ENCODED_LEN {
            timestamp_ms = u32::from_le_bytes(buf[0..4].try_into().ok()?);
            level = AlarmLevel::from_u8(buf[4]);
            source_vital = u8::from(buf[5]).try_into().ok()?;
            
        } else {
            return None; 
        }
        return Some(Self {timestamp_ms, level, source_vital}); 
    }
}

/// A configuration update sent from the UI to control-core.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfigUpdate {
    /// Lower bound for "normal" heart rate, BPM.
    pub hr_low: u16, // 2 bytes 
    /// Upper bound for "normal" heart rate, BPM.
    pub hr_high: u16,
    /// Lower bound for "normal" SpO2, percent * 10.
    pub spo2_low_permille: u16,
    /// Lower bound for "normal" temperature, degrees C * 100.
    pub temp_low_centi_c: u16,
    /// Upper bound for "normal" temperature, degrees C * 100.
    pub temp_high_centi_c: u16,
}

impl ConfigUpdate {
    const ENCODED_LEN: usize = 10; // 2 x 5 total bytes 

    /// Serialize into a fixed-size byte buffer.
    pub fn encode(&self) -> [u8; Self::ENCODED_LEN] {
        let mut buf = [0u8; Self::ENCODED_LEN];
        buf[0..2].copy_from_slice(&self.hr_low.to_le_bytes());
        buf[2..4].copy_from_slice(&self.hr_high.to_le_bytes());
        buf[4..6].copy_from_slice(&self.spo2_low_permille.to_le_bytes());
        buf[6..8].copy_from_slice(&self.temp_low_centi_c.to_le_bytes());
        buf[8..10].copy_from_slice(&self.temp_high_centi_c.to_le_bytes());
        return buf; 
    }

    /// Deserialize from a byte slice. Returns None if too short.
    pub fn decode(buf: &[u8]) -> Option<Self> {
        let hr_low : u16;
        let hr_high : u16;
        let spo2_low_permille : u16;
        let temp_low_centi_c : u16;
        let temp_high_centi_c: u16;

        
        if buf.len() >= Self::ENCODED_LEN {
            hr_low = u16::from_le_bytes(buf[0..2].try_into().ok()?);
            hr_high = u16::from_le_bytes(buf[2..4].try_into().ok()?);
            spo2_low_permille = u16::from_le_bytes(buf[4..6].try_into().ok()?);
            temp_low_centi_c = u16::from_le_bytes(buf[6..8].try_into().ok()?);
            temp_high_centi_c = u16::from_le_bytes(buf[8..10].try_into().ok()?);
        } else {
            return None; 
        }
        
        return Some(Self { hr_low, hr_high, spo2_low_permille, temp_low_centi_c, temp_high_centi_c})
    }
}


/// Compute the checksum over type byte, length bytes, and payload.
///
/// HINT: XOR is a common, cheap choice for this kind of thing — start with
/// `frame_type ^ len_bytes[0] ^ len_bytes[1]`, then XOR every payload byte
/// into that running value in a loop (`for b in payload { sum ^= b; }`).
fn checksum(frame_type: u8, len_bytes: [u8; 2], payload: &[u8]) -> u8 {
    let mut sum = frame_type ^ len_bytes[0] ^ len_bytes[1];
    for b in payload {
        sum ^= b;
    }
    return sum; 
}



/// Encode a complete frame (start byte, type, length, payload, checksum) into `out`.
/// Returns the number of bytes written, or None if the payload is too large
/// or doesn't fit in `out`.
///
/// HINT: total frame length = 1 (start) + 1 (type) + 2 (len) + payload.len() + 1 (checksum).
/// Check that against MAX_PAYLOAD and out.len() *before* writing anything.
pub fn encode_frame(frame_type: FrameType, payload: &[u8], out: &mut [u8]) -> Option<usize> {
    let total_len = 5 + payload.len();
    if payload.len() > MAX_PAYLOAD {
        return None;
    }
    out[0] = START_BYTE;
    out[1] = frame_type as u8;
    
    // Computing length bytes
    let len_bytes = (payload.len() as u16).to_le_bytes();
    out[2..4].copy_from_slice(&len_bytes);
    out[4..4+payload.len()].copy_from_slice(payload);
    out[4 + payload.len()] = checksum(frame_type as u8, len_bytes, payload);
    return Some(total_len);
}

/// Result of attempting to decode a frame from a buffer.
///
/// Why three variants instead of just `Option<Frame>` or `Result<Frame, Error>`?
/// Think about a caller that's reading bytes off a socket in a loop: it needs
/// to distinguish "wait for more bytes" (Incomplete) from "this data is junk,
/// skip past it and try again" (Invalid) from "I successfully read N bytes,
/// please remove them from your buffer" (Frame). A single Option/Result
/// can't express that distinction.
#[derive(Debug)]
pub enum DecodeResult<'a> {
    /// A complete, valid frame was found. `consumed` is how many bytes to drop from the buffer.
    Frame {
        /// The decoded frame type.
        frame_type: FrameType,
        /// The frame's payload bytes (borrowed from the input buffer).
        payload: &'a [u8],
        /// Total bytes consumed from the input buffer for this frame.
        consumed: usize,
    },
    /// Not enough bytes yet to decode a full frame — caller should read more and retry.
    Incomplete,
    /// Bytes were present but invalid (bad start byte, bad checksum, unknown type, etc).
    /// `skip` is how many bytes the caller should discard before retrying.
    Invalid {
        /// Bytes to skip before retrying decode.
        skip: usize,
    },
}

/// Attempt to decode one frame from the front of `buf`.
///
/// HINT — walk through this step by step, returning early at each check:
/// 1. `buf.is_empty()` -> Incomplete (no data at all yet)
/// 2. `buf[0] != START_BYTE` -> Invalid { skip: 1 } (resync by dropping one byte)
/// 3. `buf.len() < 4` -> Incomplete (don't even have the length field yet)
/// 4. read `len` from buf[2..4] via `u16::from_le_bytes(...)`; if `len > MAX_PAYLOAD`
///    -> Invalid { skip: 1 }
/// 5. compute `total_len = 4 + len + 1`; if `buf.len() < total_len` -> Incomplete
///    (we know the frame's size now, just don't have all of it yet)
/// 6. slice out the payload, recompute the checksum yourself, compare to the
///    byte actually in the buffer at the checksum position -> Invalid on mismatch
/// 7. convert the type byte via `FrameType::from_u8` -> Invalid on None
/// 8. otherwise return `Frame { frame_type, payload, consumed: total_len }`
pub fn decode_frame(buf: &[u8]) -> DecodeResult<'_> {
    if buf.is_empty() {
    return DecodeResult::Incomplete;
    }
    if buf[0] != START_BYTE {
        return DecodeResult::Invalid { skip: 1 };
    }
    if buf.len() < 4 {
        return DecodeResult::Incomplete;
        
    }
    let len = u16::from_le_bytes(buf[2..4].try_into().unwrap()) as usize;
    if len > MAX_PAYLOAD {
        return DecodeResult::Invalid { skip: 1 };
    }
    let total_len = 4 + len + 1;
    if buf.len() < total_len {
        return DecodeResult::Incomplete;
    }
    let payload = &buf[4..4 + len];
    let len_bytes = (len as u16).to_le_bytes();
    let expected = checksum(buf[1], len_bytes, payload);
    if buf[total_len - 1] != expected {
        return DecodeResult::Invalid { skip: 1 };
    }
    match FrameType::from_u8(buf[1]) {
        None => DecodeResult::Invalid { skip: 1 },
        Some(frame_type) => DecodeResult::Frame { frame_type, payload, consumed: total_len },
    }
} 

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: write a test that encodes a VitalsSample and decodes it back,
    // asserting the result equals the original. This is the most important
    // test in the whole crate — if this passes, your fixed-point encoding
    // math is correct.
    #[test]
    fn roundtrip_vitals_sample() {
        let original = VitalsSample {
            timestamp_ms: 1000,
            heart_rate_bpm: 72,
            spo2_permille: 975,
            temp_centi_c: 3712,
            ecg_sample_mv100: -42,
        };

        let bytes = original.encode();
        let decoded = VitalsSample::decode(&bytes).unwrap();
        assert_eq!(original, decoded);
    }

    // TODO: write a test that calls encode_frame(...) then decode_frame(...)
    // on the result and checks you get back a DecodeResult::Frame with the
    // right frame_type, payload, and consumed length.
    #[test]
    fn roundtrip_full_frame() {
        let sample = VitalsSample {
            timestamp_ms: 500,
            heart_rate_bpm: 80,
            spo2_permille: 900,
            temp_centi_c: 3600,
            ecg_sample_mv100: -42,
        };

        let payload = sample.encode();
        let mut out = [0u8; 256];
        let written = encode_frame(FrameType::VitalsSample, &payload, &mut out).unwrap();
        match decode_frame(&out[..written]) {
            DecodeResult::Frame { frame_type, payload: p, consumed } => {
                assert_eq!(frame_type, FrameType::VitalsSample);
                assert_eq!(p, &payload);
                assert_eq!(consumed, written);
            }
            other => panic!("Expected frame, got {:?}", other),
        }

        
    }

    // TODO: encode a frame, then flip a bit in its checksum byte, and assert
    // decode_frame(...) returns DecodeResult::Invalid. This is what proves
    // your checksum actually catches corruption instead of being decorative.
    #[test]
    fn detects_corrupted_checksum() {
        let payload = [1u8, 2, 3];
        let mut out = [0u8; 256];
        let written = encode_frame(FrameType::Ack, &payload, &mut out).unwrap();
        out[written - 1] ^= 0xFF;
        assert!(matches!(decode_frame(&out[..written]), DecodeResult::Invalid { ..}));
    }

    // TODO: encode a frame, then call decode_frame on a slice that's missing
    // its last byte, and assert you get DecodeResult::Incomplete (not Invalid!).
    // This distinction matters for a caller reading off a real socket.
    #[test]
    fn incomplete_buffer_requests_more_data() {
        let payload = [9u8, 8, 7];
        let mut out = [0u8; 256];
        let written = encode_frame(FrameType::Ack, &payload, &mut out).unwrap();
        assert!(matches!(decode_frame(&out[..written - 1]), DecodeResult::Incomplete));
    }
}
