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

    extern "RustQt" {
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

/// Backing Rust struct for the `Dashboard` QObject.
///
/// HINT: this needs at minimum the fields matching the qproperty list above
/// (heart_rate, spo2, temperature, ecg_sample, alarm_level, patient_id), plus
/// somewhere to stash a `Sender<ConfigUpdate>` once your background socket
/// thread exists, so `send_config_update` has something to send into.
#[derive(Default)]
pub struct DashboardRust {
    // TODO: fields here
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
        todo!()
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
