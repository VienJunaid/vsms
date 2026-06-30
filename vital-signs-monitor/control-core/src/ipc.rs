//! Unix domain socket server. `control-core` listens here; `ui-dashboard`
//! connects as a client. Frames flow both directions using the shared
//! `protocol` crate's wire format.
//!
//! YOUR TASK: accept multiple client connections, broadcast outgoing frames
//! to all of them, and receive incoming ConfigUpdate frames from any client.
//!
//! IMPORTANT DESIGN CONSTRAINT, worth re-reading before you write a line:
//! a UI client disconnecting, crashing, or never connecting at all must
//! NEVER block or slow the control loop. That means: writes to clients
//! should not be allowed to stall the broadcaster, and a dead/erroring
//! client should just get dropped from the list rather than retried.

use protocol::{decode_frame, encode_frame, ConfigUpdate, DecodeResult, FrameType};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

/// Default socket path used by both control-core and ui-dashboard.
pub const SOCKET_PATH: &str = "/tmp/vital-signs-monitor.sock";

// HINT: you need a way for multiple threads (the accept loop, and your main
// broadcast calls) to share the list of connected client streams safely.
// `Arc<Mutex<Vec<UnixStream>>>` is the standard tool for "shared, mutable,
// thread-safe list."
type ClientList = Arc<Mutex<Vec<UnixStream>>>;

/// Owns the listening socket and the set of connected UI clients. Cloning
/// this handle is cheap; the underlying client list is shared.
#[derive(Clone)]
pub struct IpcServer {
    clients: ClientList,
}

impl IpcServer {
    /// Bind the socket (removing any stale socket file first) and start
    /// accepting client connections on a background thread. Incoming
    /// `ConfigUpdate` frames from any client are forwarded to the returned
    /// `Receiver`.
    ///
    /// HINT — rough shape:
    /// 1. If a file already exists at `path`, remove it first — Unix sockets
    ///    fail to bind if a stale socket file from a previous run is still
    ///    there (`std::fs::remove_file`, ignore the error if it doesn't exist).
    /// 2. `UnixListener::bind(path)?`
    /// 3. Create your shared `ClientList` and an mpsc `channel::<ConfigUpdate>()`.
    /// 4. Spawn a thread that loops over `listener.incoming()`, and for each
    ///    successfully accepted `stream`: spawn ANOTHER thread (see
    ///    `spawn_client_reader` below) to read ConfigUpdates from that one
    ///    client, then push the stream into the shared client list (you'll
    ///    need `stream.try_clone()` since one copy goes to the reader thread
    ///    and the original goes into the list for writing).
    /// 5. Return `(IpcServer { clients }, rx)`.
    pub fn bind_and_serve(path: impl AsRef<Path>) -> std::io::Result<(Self, Receiver<ConfigUpdate>)> {
        todo!()
    }

    /// Encode and broadcast a frame's payload to every connected client.
    /// Clients that error out on write are dropped from the list.
    ///
    /// HINT: `encode_frame(frame_type, payload, &mut buf)` gives you the
    /// bytes to send. Lock the client list, then use `Vec::retain_mut` to
    /// iterate: for each client, try `client.write_all(&buf[..n])`; keep the
    /// client (return true) if the write succeeded, drop it (return false,
    /// maybe with an eprintln! first) if it errored.
    pub fn broadcast(&self, frame_type: FrameType, payload: &[u8]) {
        todo!()
    }
}

/// Reads frames from one client connection in a loop, forwarding any
/// `ConfigUpdate` frames to `tx`. Returns (the thread ends) once the client
/// disconnects or the receiving end of `tx` is gone.
///
/// HINT — rough shape:
/// 1. Keep a growable `Vec<u8>` as your "bytes read so far but not yet
///    decoded" buffer, plus a small fixed-size `[u8; N]` chunk to read into.
/// 2. Loop: `stream.read(&mut chunk)` — 0 bytes means the client closed the
///    connection, break. Otherwise append the new bytes to your buffer.
/// 3. Inner loop: call `decode_frame(&buf)` repeatedly:
///    - `DecodeResult::Frame { frame_type, payload, consumed }`: if it's a
///      ConfigUpdate, decode the payload and `tx.send(...)` it (break the
///      whole function if that errors — means the other end is gone), then
///      `buf.drain(0..consumed)` to remove the consumed bytes and loop again
///      in case there's a second frame already buffered.
///    - `DecodeResult::Incomplete`: break the inner loop, go read more bytes.
///    - `DecodeResult::Invalid { skip }`: `buf.drain(0..skip)` and keep
///      trying to resync.
fn spawn_client_reader(mut stream: UnixStream, tx: Sender<ConfigUpdate>) {
    thread::spawn(move || {
        todo!()
    });
}
