//! control-core: the real-time simulation + alarm process.
//!
//! YOUR TASK: wire together the signal generators, peak detector, alarm
//! state machine, IPC server, and audit logger into one ticking loop.

mod alarm;
mod ipc;
mod logger;
mod peak_detection;
mod signals;

use alarm::{AlarmStateMachine, Thresholds};
use logger::AuditLogger;
use peak_detection::PeakDetector;
use protocol::{AlarmEvent, FrameType, VitalsSample};
use rand::rngs::StdRng;
use rand::SeedableRng;
use signals::{EcgGenerator, SpO2Generator, TempGenerator, ECG_SAMPLE_RATE_HZ};
use std::time::{Duration, Instant};

/// Attempt to raise this thread to SCHED_FIFO real-time priority.
///
/// This is "soft real-time" honesty: on most dev machines, without
/// CAP_SYS_NICE or root, this will FAIL — that's expected and fine, just
/// log a warning and keep running at normal priority. On a real embedded
/// target you might instead want this to be a hard failure.
///
/// HINT: `libc::sched_setscheduler(0, libc::SCHED_FIFO, &param)` where
/// `param` is a `libc::sched_param { sched_priority: 50 }` (pick a priority
/// value — higher = more priority among other SCHED_FIFO threads). This is
/// an `unsafe` call since it's a raw libc FFI binding. Check the return
/// value: 0 means success, nonzero means it failed.
fn try_set_realtime_priority() {
    todo!()
}

fn main() -> std::io::Result<()> {
    try_set_realtime_priority();

    // TODO: bind the IPC server (ipc::IpcServer::bind_and_serve), open the
    // audit log (logger::AuditLogger::open("logs/audit.log")), log a startup
    // event, and construct your generators/detector/alarm machine:
    //   - StdRng::seed_from_u64(some_seed) for reproducible randomness
    //   - EcgGenerator::new(72.0) (or whatever baseline bpm you want)
    //   - SpO2Generator::new(975.0), TempGenerator::new(3700.0)
    //   - PeakDetector::new(some_threshold_mv)
    //   - AlarmStateMachine::new(Thresholds::default(), some_hysteresis_count)

    // TODO: set up your timing. `Duration::from_secs_f64(1.0 / ECG_SAMPLE_RATE_HZ as f64)`
    // gives you the per-sample tick period. Track an `Instant` for "when
    // should the next tick fire" so you can `sleep` for exactly the
    // remaining time each iteration rather than just sleeping a fixed
    // amount (which would drift).

    // TODO: main loop. Each iteration should:
    //   1. Drain any pending ConfigUpdate frames from the IPC server's
    //      receiver (non-blocking `try_recv()` in a `while let Ok(cfg) = ...`
    //      loop) and apply them to your AlarmStateMachine via set_thresholds.
    //   2. Generate the next ECG sample, feed it to the PeakDetector.
    //   3. Roughly once per second (track a sample counter against
    //      ECG_SAMPLE_RATE_HZ), generate the next SpO2 and Temp values.
    //   4. Build a VitalsSample with the current values and
    //      ipc_server.broadcast(FrameType::VitalsSample, &sample.encode()).
    //   5. If you have a valid heart rate yet (current_bpm() > 0), call
    //      alarms.evaluate(hr, spo2, temp) — if it returns Some((level,
    //      source)), build an AlarmEvent, broadcast it as FrameType::AlarmState,
    //      and log the transition.
    //   6. Sleep until the next scheduled tick time. Deliberately do NOT
    //      try to "catch up" if you're behind — think about why silently
    //      absorbing missed deadlines would be the wrong behavior for a
    //      real-time control loop, versus making the lateness visible.
    todo!()
}
