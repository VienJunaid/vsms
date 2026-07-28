use std::os::unix::net::UnixStream;
use std::time::Duration;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(i32, heart_rate)]
        #[qproperty(f64, spo2)]
        #[qproperty(f64, temperature)]
        #[qproperty(f64, ecg_sample)]
        #[qproperty(i32, alarm_level)]
        type Dashboard = super::DashboardRust;

        #[qinvokable]
        fn send_config_update(
            self: Pin<&mut Dashboard>,
            hr_low: i32,
            hr_high: i32,
            spo2_low_permille: i32,
            temp_low_centi_c: i32,
            temp_high_centi_c: i32,
        );
        #[qinvokable]
        fn initialize(self: Pin<&mut Dashboard>);
    }
}

use core::pin::Pin;
use protocol::ConfigUpdate;
use std::sync::mpsc::Sender;
use std::io::{Read, Write};

#[derive(Default)]
pub struct DashboardRust {
    pub heart_rate: i32,
    pub spo2: f64,
    pub temperature: f64,
    pub ecg_sample: f64,
    pub alarm_level: i32,
    pub config_tx: Option<Sender<ConfigUpdate>>,
}

impl qobject::Dashboard {
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

    fn initialize(mut self: Pin<&mut Self>) {
        let thread = self.qt_thread();
        let tx = spawn_socket_thread(thread);
        self.as_mut().rust_mut().config_tx = Some(tx);
    }
}

pub fn spawn_socket_thread(qt_thread: cxx_qt::CxxQtThread<qobject::Dashboard>) -> Sender<ConfigUpdate> {
    let (tx, rx) = std::sync::mpsc::channel::<ConfigUpdate>();

    std::thread::spawn(move || {
        let mut stream = match retry_connect("/tmp/vital-signs-monitor.sock", 20) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("socket thread: failed to connect: {e}");
                return;
            }
        };
        let mut buf: Vec<u8> = Vec::new();
        let mut chunk = [0u8; 256];
        let mut out_buf = [0u8; 300];

        loop {
            while let Ok(update) = rx.try_recv() {
                let payload = update.encode();
                if let Some(n) = protocol::encode_frame(
                    protocol::FrameType::ConfigUpdate,
                    &payload,
                    &mut out_buf,
                ) {
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
    tx
}

fn retry_connect(path: &str, attempts: u32) -> std::io::Result<UnixStream> {
    let mut last_err = None;
    for attempt in 1..=attempts {
        match UnixStream::connect(path) {
            Ok(s) => return Ok(s),
            Err(e) => {
                eprintln!("attempt {}/{} failed: {}", attempt, attempts, e);
                last_err = Some(e);
                std::thread::sleep(Duration::from_millis(250));
            }
        }
    }
    Err(last_err.unwrap())
}

fn dispatch(qt_thread: &cxx_qt::CxxQtThread<qobject::Dashboard>, frame_type: protocol::FrameType, payload: &[u8]) {
    match frame_type {
        protocol::FrameType::VitalsSample => {
            if let Some(s) = protocol::VitalsSample::decode(payload) {
                let hr = s.heart_rate_bpm as i32;
                let spo2 = s.spo2_permille as f64 / 10.0;
                let temp = s.temp_centi_c as f64 / 100.0;
                let ecg = s.ecg_sample_mv100 as f64;
                let _ = qt_thread.queue(move |mut dashboard| {
                    dashboard.as_mut().set_heart_rate(hr);
                    dashboard.as_mut().set_spo2(spo2);
                    dashboard.as_mut().set_temperature(temp);
                    dashboard.as_mut().set_ecg_sample(ecg);
                });
            }
        }
        protocol::FrameType::AlarmState => {
            if let Some(e) = protocol::AlarmEvent::decode(payload) {
                let level = e.level as i32;
                let _ = qt_thread.queue(move |mut dashboard| {
                    dashboard.as_mut().set_alarm_level(level);
                });
            }
        }
        _ => {}
    }
}
