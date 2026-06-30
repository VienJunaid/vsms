//! A minimal terminal-based client for `control-core`'s IPC socket.
//!
//! YOUR TASK: connect to the Unix socket, read bytes in a loop, decode
//! frames using the shared `protocol` crate, and print a live readout.
//! This is the same decode loop you'll eventually need in the real QML/
//! cxx-qt bridge — building it here first, without Qt in the picture,
//! makes it much easier to get right.

use protocol::{decode_frame, AlarmEvent, AlarmLevel, DecodeResult, FrameType, VitalsSample};
use std::io::Read;
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
    todo!()
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
    todo!()
}

/// Try connecting to a Unix socket a few times before giving up, since
/// control-core may not have started listening yet.
///
/// HINT: loop `attempts` times, try `UnixStream::connect(path)`, return Ok
/// immediately on success; on failure, sleep briefly (e.g. 250ms) and retry.
/// After the loop, return the last error you saw.
fn retry_connect(path: &str, attempts: u32) -> std::io::Result<UnixStream> {
    todo!()
}
