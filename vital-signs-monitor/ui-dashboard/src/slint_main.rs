//! The reTerminal dashboard, built with Slint instead of cxx-qt/QML.
//!
//! Reuses the same connect-and-decode loop as `ui-dashboard-cli`'s
//! `main.rs`, running on a background thread so the socket never blocks the
//! UI event loop. Decoded frames are pushed into the window via
//! `Weak::upgrade_in_event_loop`, Slint's equivalent of cxx-qt's
//! `qt_thread().queue(...)`.

slint::include_modules!();

use protocol::{ConfigUpdate, DecodeResult, FrameType};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::mpsc::Receiver;
use std::time::Duration;

const SOCKET_PATH: &str = "/tmp/vital-signs-monitor.sock";
const ECG_BUFFER_LEN: usize = 600;

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;

    let (config_tx, config_rx) = std::sync::mpsc::channel::<ConfigUpdate>();

    ui.on_apply_thresholds(move |hr_low: f32, hr_high: f32, spo2_low_permille: f32, temp_low_centi_c: f32, temp_high_centi_c: f32| {
        let update = ConfigUpdate {
            hr_low: hr_low as u16,
            hr_high: hr_high as u16,
            spo2_low_permille: spo2_low_permille as u16,
            temp_low_centi_c: temp_low_centi_c as u16,
            temp_high_centi_c: temp_high_centi_c as u16,
        };
        let _ = config_tx.send(update);
    });

    let weak = ui.as_weak();
    std::thread::spawn(move || run_socket_thread(weak, config_rx));

    ui.run()
}

fn run_socket_thread(weak: slint::Weak<MainWindow>, config_rx: Receiver<ConfigUpdate>) {
    let mut stream = match retry_connect(SOCKET_PATH, 20) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("socket thread: failed to connect: {e}");
            return;
        }
    };

    let mut buf: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 256];
    let mut out_buf = [0u8; 300];
    let mut ecg_history: VecDeque<f32> = VecDeque::with_capacity(ECG_BUFFER_LEN);

    loop {
        while let Ok(update) = config_rx.try_recv() {
            let payload = update.encode();
            if let Some(n) = protocol::encode_frame(FrameType::ConfigUpdate, &payload, &mut out_buf) {
                let _ = stream.write_all(&out_buf[..n]);
            }
        }

        let n = match stream.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        buf.extend_from_slice(&chunk[..n]);

        loop {
            match protocol::decode_frame(&buf) {
                DecodeResult::Frame { frame_type, payload, consumed } => {
                    dispatch(&weak, frame_type, payload, &mut ecg_history);
                    buf.drain(..consumed);
                }
                DecodeResult::Incomplete => break,
                DecodeResult::Invalid { skip } => {
                    buf.drain(..skip);
                }
            }
        }
    }
    eprintln!("socket thread: disconnected.");
}

fn retry_connect(path: &str, attempts: u32) -> std::io::Result<UnixStream> {
    let mut last_err = None;
    for attempt in 1..=attempts {
        match UnixStream::connect(path) {
            Ok(s) => return Ok(s),
            Err(e) => {
                eprintln!("attempt {attempt}/{attempts} failed: {e}");
                last_err = Some(e);
                std::thread::sleep(Duration::from_millis(250));
            }
        }
    }
    Err(last_err.unwrap())
}

fn dispatch(
    weak: &slint::Weak<MainWindow>,
    frame_type: FrameType,
    payload: &[u8],
    ecg_history: &mut VecDeque<f32>,
) {
    match frame_type {
        FrameType::VitalsSample => {
            if let Some(s) = protocol::VitalsSample::decode(payload) {
                let heart_rate_text = if s.heart_rate_bpm > 0 {
                    s.heart_rate_bpm.to_string()
                } else {
                    "--".to_string()
                };
                let spo2_text = format!("{:.1}", s.spo2_permille as f32 / 10.0);
                let temperature_text = format!("{:.2}", s.temp_centi_c as f32 / 100.0);

                // ecg_sample_mv100 is millivolts * 100 fixed-point (see
                // control-core/src/main.rs); divide back down to millivolts
                // before plotting.
                let ecg_mv = s.ecg_sample_mv100 as f32 / 100.0;
                if ecg_history.len() == ECG_BUFFER_LEN {
                    ecg_history.pop_front();
                }
                ecg_history.push_back(ecg_mv);
                let ecg_path = build_ecg_path(ecg_history);

                let _ = weak.upgrade_in_event_loop(move |ui| {
                    ui.set_heart_rate_text(heart_rate_text.into());
                    ui.set_spo2_text(spo2_text.into());
                    ui.set_temperature_text(temperature_text.into());
                    ui.set_ecg_path(ecg_path.into());
                });
            }
        }
        FrameType::AlarmState => {
            if let Some(e) = protocol::AlarmEvent::decode(payload) {
                let level = e.level as i32;
                let _ = weak.upgrade_in_event_loop(move |ui| {
                    ui.set_alarm_level(level);
                });
            }
        }
        _ => {}
    }
}

/// Build SVG path data for the scrolling ECG trace, in the 600x300 viewbox
/// coordinate space that `ui/waveform.slint`'s Path element scales onto
/// whatever pixel size the widget actually renders at.
fn build_ecg_path(history: &VecDeque<f32>) -> String {
    const MID_Y: f32 = 150.0;
    const SCALE: f32 = 100.0;

    let mut path = String::with_capacity(history.len() * 12);
    for (i, sample) in history.iter().enumerate() {
        let x = i as f32;
        let y = MID_Y - sample * SCALE;
        if i == 0 {
            path.push_str(&format!("M {x} {y} "));
        } else {
            path.push_str(&format!("L {x} {y} "));
        }
    }
    path
}
