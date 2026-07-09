//! A minimal terminal-based client for `control-core`'s IPC socket.
//!
//! YOUR TASK: connect to the Unix socket, read bytes in a loop, decode
//! frames using the shared `protocol` crate, and print a live readout.
//! This is the same decode loop you'll eventually need in the real QML/
//! cxx-qt bridge — building it here first, without Qt in the picture,
//! makes it much easier to get right.

use protocol::{decode_frame, AlarmEvent, AlarmLevel, DecodeResult, FrameType, VitalsSample};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

const SOCKET_PATH: &str = "/tmp/vital-signs-monitor.sock";

fn main() -> std::io::Result<()> {
    // TODO: connect to SOCKET_PATH. control-core might not be up yet when
    // you start this binary, so consider retrying a few times with a short
    // sleep between attempts rather than failing immediately (see
    // `retry_connect` stub below).

    // TODO: main read loop:
    //   1. Read bytes into a small fixed chunk buffer, append them to a
    //      growable Vec<u8> that holds "received but not yet decoded" data.
    //      A read returning 0 bytes means control-core closed the connection.
    //   2. Inner loop: call decode_frame on your buffer repeatedly, handling
    //      each DecodeResult variant (Frame -> dispatch to handle_frame below
    //      and drain `consumed` bytes; Incomplete -> break and read more;
    //      Invalid -> drain `skip` bytes and keep resyncing).
    eprintln!("Connecting to contorl-core at {} ...", SOCKET_PATH);
    let mut stream = retry_connect(SOCKET_PATH, 10)?;
    eprintln!("Connected. Waiting for data...");
    
    // create the buffer that is a Vec>8u> 
    let mut buf: Vec<u8> = Vec::new();

    // chunk is a fixed array containing 256 unsigned 8-bit integers
    let mut chunk = [0u8; 256];

    // normal alarm level
    let mut alarm_level = AlarmLevel::Normal;

    loop {
        // read whatever is in the stream 
        let n = stream.read(&mut chunk)?;

        // A read returning 0 bytes - close the connections 
        if n == 0 {
            eprintln!("\ncontrol-core disconnected.");
            break;
        }
        buf.extend_from_slice(&chunk[..n]);

        loop {
            match decode_frame(&buf) {
                DecodeResult::Frame { frame_type, payload, consumed } => {
                    handle_frame(frame_type, payload, &mut alarm_level);
                    buf.drain(..consumed);
                }
                DecodeResult::Incomplete => break,
                DecodeResult::Invalid { skip } => {
                    buf.drain(..skip);
                }
            }
        }
    }
    Ok(())
}

/// Dispatch one decoded frame: update local state and print something useful.
///
/// HINT: match on `frame_type`. For VitalsSample, decode the payload and
/// print a one-line live readout (consider using `\r` instead of `\n` so it
/// overwrites in place rather than scrolling). For AlarmState, decode and
/// print a clearly different, attention-grabbing line — and remember to
/// update whatever you're tracking as "current alarm level" so subsequent
/// vitals printouts can reflect it.
fn handle_frame(frame_type: FrameType, payload: &[u8], alarm_level: &mut AlarmLevel) {
    match frame_type { 
        FrameType::VitalsSample => {
            if let Some(s) = VitalsSample::decode(payload) {
                let level_tag  = match alarm_level {
                    AlarmLevel::Normal => "OK",
                    AlarmLevel::Warning => "WARN",
                    AlarmLevel::Critical => "CRIT",
                };
                print!(
                    "\r[{:>9}ms] HR:{:>4} bpm | SpO2:{:>5.1}% | Temp:{:>6.2}C | ECG:{:>6} [{}]  ",
                    s.timestamp_ms,
                    s.heart_rate_bpm,
                    s.spo2_permille as f32 / 10.0,
                    s.temp_centi_c as f32 / 100.0,
                    s.ecg_sample_mv100,
                    level_tag,
                );
                let _ = std::io::stdout().flush();
            }
        }
        FrameType::AlarmState => {
            if let Some(e) = AlarmEvent::decode(payload) {
                *alarm_level = e.level;
                let source_name = match e.source_vital {
                    0 => "HR",
                    1 => "SpO2",
                    2 => "Temp",
                    255 => "cleared",
                    _ => "unknown",
                };
                let level_str = match e.level {
                    AlarmLevel::Normal => "NORMAL   ",
                    AlarmLevel::Warning => "WARNING ",
                    AlarmLevel::Critical => "CRITICAL!",
                };
                println!("\n[{:>9}ms] *** ALARM: {} (source: {}) ***", e.timestamp_ms, level_str, source_name);
            }
        }
        _ => {}
    }
}

/// Try connecting to a Unix socket a few times before giving up, since
/// control-core may not have started listening yet.
///
/// HINT: loop `attempts` times, try `UnixStream::connect(path)`, return Ok
/// immediately on success; on failure, sleep briefly (e.g. 250ms) and retry.
/// After the loop, return the last error you saw.
fn retry_connect(path: &str, attempts: u32) -> std::io::Result<UnixStream> {
    let mut last_err = None;
    for attempt in 1..=attempts {
        match UnixStream::connect(path) {
            Ok(s) => return Ok(s),
            Err(e) => {
                eprintln!(" attempt {}/{} failed: {}", attempt, attempts, e);
                last_err = Some(e);
                std::thread::sleep(Duration::from_millis(250));
            }
        }
    }
    Err(last_err.unwrap())
}
