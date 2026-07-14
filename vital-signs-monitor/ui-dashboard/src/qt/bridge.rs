//! cxx-qt bridge: exposes live vitals/alarm state as a Qt-bindable QObject
//! ("dashboard" in QML) and (eventually) owns a background thread that
//! reads frames off the control-core Unix socket, reusing the exact same
//! decode loop you wrote in ui-dashboard/src/main.rs.
//!
//! This module is feature-gated behind `qt-ui` (see ui-dashboard/Cargo.toml)
//! since it requires Qt installed at build time — you'll need to be on the
//! reTerminal (or a dev machine with Qt5/Qt6 dev packages) to compile and
//! iterate on this file. Everything else in this repo deliberately doesn't
//! depend on it.
//!
//! YOUR TASK (once you have Qt available):
//! 1. Get the `#[cxx_qt::bridge]` macro below compiling with properties QML
//!    can read.
//! 2. Implement `send_config_update` to forward values into a channel.
//! 3. Add a background thread (started from wherever you construct the
//!    Dashboard) that connects to the socket and pushes decoded values into
//!    the QObject's properties using cxx-qt's generated setters — this is
//!    the trickiest part, because that thread is NOT the Qt event loop
//!    thread, so you need cxx-qt's mechanism for safely updating properties
//!    from another thread (look at `qobject::Dashboard::cxx_qt_thread()` /
//!    queued property updates in the cxx-qt docs — this detail is exactly
//!    why this file is left for you to work through once Qt is installed
//!    and you can iterate against real compiler errors).

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(i32, heart_rate)]
        #[qproperty(f64, spo2)]
        #[qproperty(f64, temperature)]
        #[qproperty(f64, ecg_sample)]
        #[qproperty(i32, alarm_level)]
        #[qproperty(QString, patient_id)]
        type Dashboard = super::DashboardRust;

        /// Invoked from QML's "Apply Thresholds" button. Should forward a
        /// ConfigUpdate frame to control-core over the IPC socket.
        #[qinvokable]
        fn send_config_update(
            self: Pin<&mut Dashboard>,
            hr_low: i32,
            hr_high: i32,
            spo2_low_permille: i32,
            temp_low_centi_c: i32,
            temp_high_centi_c: i32,
        );
    }
}

use core::pin::Pin;
use cxx_qt_lib::QString;
use protocol::ConfigUpdate;
use std::sync::mpsc::Sender;
use std::io::Read;


/// Backing Rust struct for the `Dashboard` QObject.
///
/// HINT: this needs at minimum the fields matching the qproperty list above
/// (heart_rate, spo2, temperature, ecg_sample, alarm_level, patient_id), plus
/// somewhere to stash a `Sender<ConfigUpdate>` once your background socket
/// thread exists, so `send_config_update` has something to send into.
#[derive(Default)]
pub struct DashboardRust {
    pub heart_rate: i32,
    pub spo2: f64,
    pub temperature: f64,
    pub ecg_sample: f64,
    pub alarm_level: i32,
    pub patient_id: QString,
    pub config_tx: Option<Sender<ConfigUpdate>>,
}

impl qobject::Dashboard {
    /// HINT: `self.rust()` gets you a reference to the DashboardRust struct
    /// (via the Pin). If `config_tx` is Some, build a ConfigUpdate from the
    /// i32 args (casting to u16 — note these came from QML as ints) and
    /// `.send(...)` it. If it's None (background thread hasn't started yet
    /// or failed), log a warning instead of panicking.
    fn send_config_update(
        self: Pin<&mut Self>,
        hr_low: i32,
        hr_high: i32,
        spo2_low_permille: i32,
        temp_low_centi_c: i32,
        temp_high_centi_c: i32,
    ) {
        let update = ConfigUpdate {
            hr_low: hr_low as u16,
            hr_high: hr_high as u16,
            spo2_low_permille: spo2_low_permille as u16,
            temp_low_centi_c: temp_low_centi_c as u16,
            temp_high_centi_c: temp_high_centi_c as u16,
        };
        if let Some(tx) = &self.rust().config_tx {
            let _ = tx.send(update);
        } else {
            eprintln!("send_config_update: background thread not running"); 
        }
    }
}

// TODO: the socket-reading background thread. Plan:
// - Connect to ipc::SOCKET_PATH (you'll need to either depend on control-core's
//   constant or just hardcode/share the path string here).
// - Reuse the exact decode loop from ui-dashboard/src/main.rs's CLI client.
// - Instead of println!, push decoded values into the Dashboard's Qt
//   properties. Look up how cxx-qt expects cross-thread property updates to
//   be done safely (hint: it's NOT as simple as just calling the generated
//   setter directly from a non-Qt thread — there's a queued-invoke mechanism
//   for this in cxx-qt's docs).
//


// This function takes in a CxxQtThread<Dashboard> handle and immediately spawns a background
// OS thread.
// Inside the therad:
// Calls retry_connect() to connect to the Unix Socket that control-core is broadcasting on.
// It tries up to 20 times btw 
//
// Then it runs a infinite read loop - (same structure pattern as CLI client in main.rs) 
//  Reads raw bytes into chunks, appends to a buffer, then repeatedly calls decode_frame until the
//  buffer is exhausted.
//
//
//  For each successfuly decoded frame it calls dispatch(). If the connection drops (reads return 0
//  or errors) the thread exits cleanly 
//
//  Key reasoning for separate threads: Qt's event loop blocks the main thread. If you tried to do
//  socket I/O there, the entire UI would freeze waiting for data (data race)


pub fn spawn_socket_thread(qt_thread: cxx_qt::CxxQtThread<qobject::Dashboard>) {
    std::thread::spawn(move || {
        // create a stream
        // doing the same implementation as the retry_connect() function
        retry_connect(); 

        let mut stream = match retry_connect("/tmp/vital-signs-monitor.sock", 20) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("socket thread: failed to connect: {e}");
                return;
            }
        };
        // create an array to rate the stream 
        let mut buf: Vec<u8> = Vec::new();
        let mut chunk = [0u8; 256];
        
        // start reading the stream 
        loop {
            let n = match stream.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => n,
            };
            buf.extend_from_slice(&chunk[..n]);
            
            loop {
                match protocol::decode_frame(&buf) {
                    protocol::DecodeResult::Frame { frame_type, payload, consumed } => {
                        dispatch(&qt_thread, frame_type, payload);
                        buf.drain(..consumed);
                    }
                    protocol::DecodeResult::Incomplete => break,
                    protocol::DecodeResult::Invalid { skip } => { buf.drain(..skip); }
                }
            }
        }
        eprintln!("socket thread: disconnected.");
    });
}

// Retry_connect function
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

fn dispatch(qt_thread: &cxx_qt::CxxQtThread<qobject::Dashboard>, frame_type:
    protocol::FrameType, payload: &[u8]) {
    match frame_type {
        protocol::FrameType::VitalsSample => {
            if let Some(s) = protocol::VitalsSample::decode(payload) {
                let hr = s.heart_rate_bpm as i32;
                let spo2 = s.spo2_permille as f64 / 10.0;
                let temp = s.temp_centic_c as f64 / 100.0;
                let ecg = s.ecg_sample_mv100 as f64; 
                let _ = qt_thread.queue(move |mut dashboard| {
                    dashboard.as_mut().set_heart_rate(hr);
                    dashboard.as_mut().set_spo2(spo2);
                    dashboard.as_mut().set_temperature(temp);
                    dashboard.as_mut().set_ecg_sample(ecg);
                });
            }
        }
        // 
        protocol::FrameType::AlarmState => {
            if let Some(e) = protocol::AlarmEvent::decode(payload) {
                let level = e.level as i32;
                let _ = qt_thread.queue(move |mut dashboard| {
                    dashboard.as_mut().set_alarm_level(level);
                });
            }
        }
        // send a blank if failure 
        _ => {}
    }
}



