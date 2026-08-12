# ui-dashboard

Two build targets live here:

## 1. `ui-dashboard-cli` (terminal client, no GUI toolkit required)

A terminal client that connects to control-core's Unix socket, decodes
frames using the shared `protocol` crate, and prints a live readout. This
exists so the protocol/IPC layer can be developed and tested end-to-end
without a GUI toolkit involved, and it doubles as a debugging tool later.

```sh
cargo run --bin ui-dashboard-cli
```

## 2. `ui-dashboard-slint` (the real dashboard UI)

The full dashboard described in ARCHITECTURE.md — alarm banner, scrolling
ECG waveform, HR/SpO2/Temp tiles, and a settings panel with threshold
sliders — built with [Slint](https://slint.dev), a pure-Rust UI toolkit.
Unlike the earlier cxx-qt/QML attempt, this needs no system Qt packages: it
builds and runs anywhere `cargo` does, including this sandbox.

```sh
cd ui-dashboard
cargo run --bin ui-dashboard-slint
```

### Layout

- `ui/*.slint` — the view layer, one file per component (mirrors the old
  QML breakdown): `main.slint` (top-level window), `alarm_banner.slint`,
  `vitals_panel.slint` / `vital_tile.slint`, `waveform.slint`,
  `settings_panel.slint` / `threshold_slider.slint`.
- `src/slint_main.rs` — the Rust side. Reuses the exact same
  connect-and-decode loop as `ui-dashboard-cli`'s `main.rs` on a background
  thread, and pushes decoded values into the window via
  `Weak::upgrade_in_event_loop` (Slint's version of "safely update the UI
  from a background thread").
- `build.rs` — compiles the `.slint` files via `slint_build::compile`.

### Why split it this way

Keeping `ui-dashboard-cli` toolkit-free means the protocol, IPC, alarm state
machine, and peak detection can all be developed, tested, and demoed without
touching any GUI dependency — which is exactly what happened during initial
development of this repo. The Slint layer is a presentation concern on top
of an already-proven data path, and reuses its decode loop verbatim.
