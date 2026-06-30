# ui-dashboard

Two build targets live here:

## 1. `ui-dashboard-cli` (works right now, no Qt required)

A terminal client that connects to control-core's Unix socket, decodes
frames using the shared `protocol` crate, and prints a live readout. This
exists so the protocol/IPC layer can be developed and tested end-to-end
without Qt installed, and it doubles as a debugging tool later.

```sh
cargo run --bin ui-dashboard-cli
```

## 2. `ui-dashboard-qt` (the real reTerminal UI, needs Qt)

The actual QML dashboard described in ARCHITECTURE.md. This is feature-gated
behind `qt-ui` because `cxx-qt` requires Qt5 or Qt6 dev packages at build
time, which the reTerminal's Debian-based BSP provides but a generic dev
sandbox usually doesn't.

### On the reTerminal (or any Linux box with Qt installed)

```sh
# Install Qt dev packages (Debian/Ubuntu-based, which the reTerminal's BSP is)
sudo apt-get install qtbase5-dev qtdeclarative5-dev qml-module-qtquick2 \
    qml-module-qtquick-controls2 qml-module-qtquick-layouts

cd ui-dashboard
cargo run --features qt-ui --bin ui-dashboard-qt
```

### What's implemented vs. what's left

- ✅ QML views (`qml/main.qml`, `Waveform.qml`, `VitalsPanel.qml`,
  `VitalTile.qml`, `AlarmBanner.qml`, `SettingsPanel.qml`,
  `ThresholdSlider.qml`) — these are complete and ready to render once a
  `dashboard` context object exists.
- ✅ `src/qt/bridge.rs` — the `cxx-qt` `QObject` bridge exposing
  `heartRate`, `spo2`, `temperature`, `ecgSample`, `alarmLevel`,
  `patientId` as Qt properties, and `sendConfigUpdate(...)` as a QML-callable
  method that forwards a `ConfigUpdate` frame to control-core.
- ⏳ **Remaining work**: the background socket-reading thread that
  connects to `/tmp/vital-signs-monitor.sock`, decodes `VitalsSample`/
  `AlarmState` frames (logic identical to `ui-dashboard-cli`'s `main.rs`),
  and pushes the decoded values into the `Dashboard` QObject's properties
  via cxx-qt's queued property setters (so updates land safely on the Qt
  event loop thread rather than racing it). This needs a real Qt
  toolchain to iterate on and is the natural next step once you're on
  the reTerminal.
- ⏳ A `build.rs` using `cxx_qt_build::CxxQtBuilder` to register
  `src/qt/bridge.rs` and the QML module — a few lines, but again easiest
  to get exactly right with Qt actually installed to compile against.

### Why split it this way

Keeping `ui-dashboard-cli` Qt-free means the protocol, IPC, alarm state
machine, and peak detection can all be developed, tested, and demoed
without ever touching a Qt toolchain — which is exactly what happened
during initial development of this repo. The QML/cxx-qt layer is a
presentation concern on top of an already-proven data path.
