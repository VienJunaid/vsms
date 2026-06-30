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
        todo!("match b against each FrameType's wire value, return None for unknown bytes")
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
        todo!("match b -> AlarmLevel, with a sensible default for unknown values")
        match b {
            0 => Normal,
            1 => Warning,
            2 => Critical,
            _ => Critical, // If you were to receive Junk Data, you should immediate check up on the patient and fix it
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
        //todo!("write each field's to_le_bytes() into the right slice of buf")
        let mut buf = [0u8;Self::ENCODED_LEN];
        buf[0..4].copy_from_slice(&self.timestamp_ms.to_le_bytes());
        buf[5..6].copy_from_slice(&self.heart_rate_bpm.to_le_bytes());
        buf[7..8].copy_from_slice(&self.spo2_permille.to_le_bytes());
        buf[9..10].copy_from_slice(&self.temp_centi_c.to_le_bytes());
        buf[11..12].copy_from_slice(&self.ecg_sample_mv100.to_le_bytes());
        return buf;
    }


    /// Deserialize from a byte slice. Returns None if too short.
    ///
    /// HINT: the inverse of encode — `u32::from_le_bytes(buf[0..4].try_into().ok()?)`
    /// is the pattern for each field. The `?` on `try_into().ok()?` is what
    /// lets you bail out with `None` if a slice conversion ever fails.
    pub fn decode(buf: &[u8]) -> Option<Self> {
        //todo!("check buf.len() >= ENCODED_LEN, then from_le_bytes() each field back out")
        if buf.len() >= Self::ENCODED_LEN {
            let d1 = u32::from_le_bytes(buf[0..4].try_into().ok()?);
            let d2 = u16::from_le_bytes(buf[5..6].try_into().ok()?);
            let d3 = u16::from_le_bytes(buf[7..8].try_into().ok()?);
            let d4 = u16::from_le_bytes(buf[9..10].try_into().ok()?);
            let d6 = u16::from_le_bytes(buf[11..12].try_into().ok()?);
        } else {
            return None; 
        }
        return Some(Self {d1, d2, d3, d4, d5, d6});
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
        todo!()

        let mut buf = [0u8;Self;ENCODED_LEN];
        buf[0..4].copy_from_slice(&self.timestamp_ms.to_le_bytes());
        buf[5].copy_from_slice(&self.level.to_le_bytes());
        buf[6].copy_from_slice(&self.source_vital.to_le_bytes());
    }

    /// Deserialize from a byte slice. Returns None if too short.
    pub fn decode(buf: &[u8]) -> Option<Self> {
        // HINT: use AlarmLevel::from_u8(buf[4]) for the level field.
        todo!()
        if buf.len() >= Self::ENCODED_LEN {
            let d1 = u32::from_le_bytes(buf[0..4].try_into().ok()?);
            let d2 = AlarmLevel::from_u8(buf[5]);
            let d3 = u8::from_le_bytes(buf[6].try_into().ok?);
            
        } else {
            return None; 
        }
        return Some(Self {d1, d2, d3}); 
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
        let mut buf = [u8; Self::ENCODED_LEN];
        buf[0..2].copy_from_slice(&self.hr_low.to_le_bytes());
        buf[3..4].copy_from_slice(&self.hr_high.to_le_bytes());
        buf[5..6].copy_from_slice(&self.spo2_low_permille.to_le_bytes());
        buf[7..8].copy_from_slice(&self.temp_low_centi_c.to_le_bytes());
        buf[9..10].copy_from_slice(&self.temp_high_centi_c.to_le_bytes());
        todo!()
    }

    /// Deserialize from a byte slice. Returns None if too short.
    pub fn decode(buf: &[u8]) -> Option<Self> {
        //todo!()
        if buf.len() >= Self::ENCODED_LEN {
            let d1 = u16::from_le_bytes(buf[0..2].try_into().ok()?);
            let d2 = u16::from_le_bytes(buf[3..4].try_into().ok()?);
            let d3 = u16::from_le_bytes(buf[5..6].try_into().ok()?);
            let d4 = u16::from_le_bytes(buf[7..8].try_into().ok()?);
            let d5 = u16::from_le_bytes(buf[9..10].try_into().ok()?);
        } else {
            return None; 
        }
    }
}


/// Compute the checksum over type byte, length bytes, and payload.
///
/// HINT: XOR is a common, cheap choice for this kind of thing — start with
/// `frame_type ^ len_bytes[0] ^ len_bytes[1]`, then XOR every payload byte
/// into that running value in a loop (`for b in payload { sum ^= b; }`).
fn checksum(frame_type: u8, len_bytes: [u8; 2], payload: &[u8]) -> u8 {
    todo!()
}

/// Encode a complete frame (start byte, type, length, payload, checksum) into `out`.
/// Returns the number of bytes written, or None if the payload is too large
/// or doesn't fit in `out`.
///
/// HINT: total frame length = 1 (start) + 1 (type) + 2 (len) + payload.len() + 1 (checksum).
/// Check that against MAX_PAYLOAD and out.len() *before* writing anything.
pub fn encode_frame(frame_type: FrameType, payload: &[u8], out: &mut [u8]) -> Option<usize> {
    todo!("write START_BYTE, type, len (as_le_bytes), payload, then checksum(...) into out")
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
    todo!()
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
        todo!()
    }

    // TODO: write a test that calls encode_frame(...) then decode_frame(...)
    // on the result and checks you get back a DecodeResult::Frame with the
    // right frame_type, payload, and consumed length.
    #[test]
    fn roundtrip_full_frame() {
        todo!()
    }

    // TODO: encode a frame, then flip a bit in its checksum byte, and assert
    // decode_frame(...) returns DecodeResult::Invalid. This is what proves
    // your checksum actually catches corruption instead of being decorative.
    #[test]
    fn detects_corrupted_checksum() {
        todo!()
    }

    // TODO: encode a frame, then call decode_frame on a slice that's missing
    // its last byte, and assert you get DecodeResult::Incomplete (not Invalid!).
    // This distinction matters for a caller reading off a real socket.
    #[test]
    fn incomplete_buffer_requests_more_data() {
        todo!()
    }
}
