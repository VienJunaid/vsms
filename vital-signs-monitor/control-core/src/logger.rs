//! A minimal append-only, timestamped logger gesturing at the kind of
//! data-integrity/traceability expectations found in medical device record
//! keeping. (Not a compliance implementation of any real standard — just a
//! structural analogy worth pointing at in a portfolio writeup.)
//!
//! YOUR TASK: open a file in append mode and write timestamped lines to it.

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Appends timestamped, structured lines to a log file.
pub struct AuditLogger {
    // TODO: you'll want a buffered writer over an open File. Buffering
    // matters here because you'll be calling log() frequently and don't
    // want a syscall on every single write.
    writer: BufWriter<File>,
}

impl AuditLogger {
    /// Open (or create) a log file in append mode.
    ///
    /// HINT: `OpenOptions::new().create(true).append(true).open(path)?`
    /// gives you a File that always appends rather than overwriting.
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path.as_ref())?;
        return Ok(Self { writer: BufWriter::new(file) });
    }

    /// Write one log line, prefixed with a Unix epoch millisecond timestamp.
    ///
    /// HINT: `SystemTime::now().duration_since(UNIX_EPOCH)` gives you a
    /// Duration since epoch (handle the Result — it can technically fail if
    /// the clock is set before 1970, treat that as "unknown timestamp").
    /// Then `.as_millis()` to get a u128 you can format into the line.
    /// Consider: should a failed audit-log write panic the control loop, or
    /// should it be best-effort and silently continue? What would a real
    /// medical device need to do here instead, and why is "silently
    /// continue" not actually acceptable in a real system?
    pub fn log(&mut self, event: &str) {
        let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis(); 

        let _ = writeln!(self.writer, "{} {}", ts, event);

        let _ = self.writer.flush();
    }
}
