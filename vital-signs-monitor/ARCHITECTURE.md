# Architecture — Vital Signs Monitor Simulator (single-board edition)

## Overview

Both tiers run on the **Seeed reTerminal (CM4, Debian Linux)**, as two
separate OS processes connected by a local **Unix domain socket**. This
keeps the architectural lesson that mattered most — *the control/alarm
logic is isolated from the UI and cannot be stalled by it* — while
dropping the complexity of a second physical board. The boundary is now
a process boundary + IPC contract instead of a UART link, but the
design intent is identical to a real device: the safety-relevant loop
runs at a guaranteed cadence regardless of what the UI is doing.

```
┌────────────────────────────────────────────┐      Unix Domain Socket      ┌──────────────────────────────────────────┐
│              control-core (process)          │  ── structured frames ──▶  │            ui-dashboard (process)          │
│                                                │  ◀── config/ack frames ──  │                                              │
│  - Real-time-priority thread (SCHED_FIFO)     │                             │  - cxx-qt bridges Rust <-> QML               │
│  - ECG / SpO2 / Temp simulation                │                             │  - QML renders on reTerminal touchscreen     │
│  - Peak detection -> heart rate                │                             │  - Live waveform, vitals, alarm banner       │
│  - Alarm state machine (Normal/Warning/Crit)   │                             │  - Settings panel (thresholds, patient ID)   │
│  - Audit-trail logger (timestamped, append-only)│                            │                                              │
└────────────────────────────────────────────┘                             └──────────────────────────────────────────┘
```

## Why two processes instead of one binary

Even on a single board, splitting into two OS processes (rather than two
threads in one binary) buys a real property: a panic, deadlock, or
runaway allocation in the Qt/QML event loop **cannot** take down the
control loop's process. The control-core process keeps running and
logging regardless of UI health. This is a deliberately chosen
constraint, not an accident of the simpler hardware setup — it's the
core thing this project is meant to demonstrate.

The two processes communicate over a Unix domain socket using a small
length-prefixed binary frame format (defined in `protocol/`), the same
format that would be used over UART/USB on the two-board version. That
means swapping back to a real microcontroller later is mostly a
transport change, not a redesign.

## Component breakdown

### `protocol/` (shared crate, `no_std`-compatible)
- Defines `Frame`, a fixed binary wire format: `[start byte][type][len][payload][checksum]`.
- Frame types: `VitalsSample`, `AlarmState`, `ConfigUpdate`, `Ack`.
- Encoding/decoding logic shared by both processes so the contract can't drift.
- Deliberately `no_std` + `alloc`-free where possible, so this same crate
  could later run on a real MCU without modification.

### `control-core/` (binary, std Rust)
- **Signal generators**: synthetic ECG waveform (a sum-of-sinusoids approximation
  of a QRS-complex-like pattern), SpO2 (slow random walk in a healthy band with
  occasional excursions), temperature (slow drift).
- **Peak detection**: simple thresholding + refractory-period algorithm over
  the ECG stream to derive instantaneous heart rate (BPM).
- **Alarm state machine**: `Normal -> Warning -> Critical`, driven by
  configurable thresholds per vital, with hysteresis to avoid alarm
  chatter (a real concern in actual monitors — flickering alarms erode
  clinician trust and are explicitly addressed in alarm-management
  standards like IEC 60601-1-8).
- **Scheduling**: runs the sample/alarm loop on a dedicated OS thread set
  to `SCHED_FIFO` real-time priority via `libc`/`nix`, with a fixed tick
  interval (e.g., 4ms for a 250Hz ECG sample rate). This is "soft"
  real-time (Linux + PREEMPT_RT-friendly), not hard real-time like a bare
  metal MCU loop — documented clearly so the distinction isn't glossed over.
- **Audit logger**: every sample batch and every alarm transition is
  appended to a timestamped, append-only log file (`logs/`), gesturing at
  the kind of traceability expected of medical device data records.
- **IPC server**: listens on a Unix domain socket, pushes `VitalsSample`
  and `AlarmState` frames to any connected UI client, accepts
  `ConfigUpdate` frames (alarm thresholds, patient ID) from the UI.

### `ui-dashboard/` (binary, cxx-qt + QML)
- A Rust `QObject` bridge (via `cxx-qt`) exposes live vitals, alarm state,
  and settings as Qt properties/signals that QML binds to directly.
- A background Rust task owns the Unix socket client, decodes frames,
  and updates the bridge's properties — QML never touches the socket directly.
- QML views:
  - **Waveform.qml** — scrolling ECG trace (Canvas or a custom QML item).
  - **VitalsPanel.qml** — HR / SpO2 / Temp numeric readouts.
  - **AlarmBanner.qml** — color-coded banner (green/yellow/red) with audible
    alarm hook (stretch goal).
  - **SettingsPanel.qml** — patient ID entry, per-vital threshold sliders,
    sends `ConfigUpdate` frames back to control-core.

## Timing budget (initial targets)

| Task                         | Target cadence | Notes |
|-------------------------------|----------------|-------|
| ECG sample generation         | 250 Hz (4ms)   | Typical clinical ECG sampling rate floor |
| SpO2 / Temp sample            | 1 Hz           | These vitals don't need high sample rates |
| Peak detection / HR update    | Per ECG sample, HR recalculated every ~1s window |
| Alarm evaluation              | Every sample batch | Hysteresis window ~3-5s to avoid chatter |
| UI frame push (socket)        | 30-60 Hz batched | Decoupled from the 250Hz internal rate — UI doesn't need every raw sample |
| Audit log flush               | Every alarm transition + periodic heartbeat (e.g. every 5s) |

## What this project intentionally does NOT claim
- It is not a certified or validated medical device, and the simulated
  signals are not derived from real patient data.
- "Real-time" here means Linux soft real-time (SCHED_FIFO + careful loop
  design), not the deterministic hard real-time of bare-metal RTIC/Zephyr.
  That tradeoff is explicit and documented, not hidden.
- The audit log is a structural analogy to data-integrity practices, not
  an implementation of 21 CFR Part 11 or IEC 62304 — those standards
  involve far more than a timestamped log file.

## Future stretch goals
- Move `control-core`'s loop onto a real RP2040 over USB-serial, reusing
  the `protocol` crate unmodified, to demonstrate the original two-board
  hard-real-time design as a v2.
- Add PREEMPT_RT kernel patch on the reTerminal and measure/document
  actual scheduling jitter, with numbers, as a small write-up.
- Audible alarm output, multi-patient view, historical trend graphs.
