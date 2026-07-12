//! control-core: the real-time simulation + alarm process.
//!
//! YOUR TASK: wire together the signal generators, peak detector, alarm
//! state machine, IPC server, and audit logger into one ticking loop.

mod alarm;
mod ipc;
mod logger;
mod peak_detection;
mod signals;
// Dependencies
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
/// Currently the main implementation of the main.rs file should be good
/// We need to work on implementing the QML code for this project
fn try_set_realtime_priority() {
    let param = libc::sched_param { sched_priority: 50 };
    let result = unsafe { libc::sched_setscheduler(0, libc::SCHED_FIFO, &param) };
    if result != 0 {
        eprintln!("warning: could not set real-time priority (run as root for this)");
    }
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
    std::fs::create_dir_all("logs")?;
    let (ipc_server, config_rx) = ipc::IpcServer::bind_and_serve(ipc::SOCKET_PATH)?;
    let mut logger = AuditLogger::open("logs/audit.log")?;
    logger.log("control-core started");

    let mut rng = StdRng::seed_from_u64(42);
    let mut ecg = EcgGenerator::new(72.0);
    let mut spo2_gen = SpO2Generator::new(975.0);
    let mut temp_gen = TempGenerator::new(3700.0);
    let mut peak_detector = PeakDetector::new(0.5);
    let mut alarms = AlarmStateMachine::new(Thresholds::default(), 5);

    let mut current_spo2: u16 = 975;
    let mut current_temp: u16 = 3700;
    let mut sample_counter: u32 = 0;

    let tick_period = Duration::from_secs_f64(1.0 / ECG_SAMPLE_RATE_HZ as f64);
    let mut next_tick = Instant::now() + tick_period;
    
    // main loop
 
    loop {
        // Drain config updates
        while let Ok(cfg) = config_rx.try_recv() {
            alarms.set_thresholds(Thresholds {
            hr_warning_low: cfg.hr_low,
            hr_warning_high: cfg.hr_high,
            spo2_warning_low: cfg.spo2_low_permille,
            temp_warning_low: cfg.temp_low_centi_c,
            temp_warning_high: cfg.temp_high_centi_c,
            ..Thresholds::default()
            });
        }

        // Generate ECG sample, feed the peak detector 
        let ecg_sample = ecg.next_sample(&mut rng);
        peak_detector.feed_sample(ecg_sample);
        sample_counter += 1;

        // Once per second, update the spO2 and Temp
        if sample_counter >= ECG_SAMPLE_RATE_HZ {
            current_spo2 = spo2_gen.next_value(&mut rng);
            current_temp = temp_gen.next_value(&mut rng);
        }

        // Build and broadcast VitalsSamples
        let hr = peak_detector.current_bpm();
        let sample = VitalsSample {
            timestamp_ms: 0,
            heart_rate_bpm: hr,
            spo2_permille: current_spo2,
            temp_centi_c: current_temp,
            ecg_sample_mv100: (ecg_sample * 100.0) as i16,
        };
        ipc_server.broadcast(FrameType::VitalsSample, &sample.encode());

        // Evaluating Alarms
        if hr > 0 {
            if let Some((level, source)) = alarms.evaluate(hr, current_spo2, current_temp) {
                let event = AlarmEvent { timestamp_ms: 0, level, source_vital: source.to_wire_byte() };
                ipc_server.broadcast(FrameType::AlarmState, &event.encode());
                logger.log(&format!("alarm transition: level={:?} source={:?}", level, source));
            }
        }

        // Sleep until next tick
        let now = Instant::now();
        if now < next_tick {
            std::thread::sleep(next_tick - now);
        }
        next_tick += tick_period;

    }
}
