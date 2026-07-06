# Milestone Checklist — Vital Signs Monitor Simulator

Tracks remaining implementation work. Update checkboxes as you complete
items. Ordered by dependency: `protocol` blocks everything, `control-core`
blocks `ui-dashboard`'s live data, Qt bridge is last because it needs a Qt
toolchain.

Status legend: `[ ]` not started · `[~]` in progress / has bugs · `[x]` done

---

## Milestone 0 — `protocol` crate (shared wire format) ✅ COMPLETE

- [x] `FrameType::from_u8`
- [x] `AlarmLevel::from_u8`
- [x] `VitalsSample::ENCODED_LEN`
- [x] `VitalsSample::encode`
- [x] `VitalsSample::decode`
- [x] `AlarmEvent::ENCODED_LEN`
- [x] `AlarmEvent::encode`
- [x] `AlarmEvent::decode`
- [x] `ConfigUpdate::ENCODED_LEN`
- [x] `ConfigUpdate::encode`
- [x] `ConfigUpdate::decode`
- [x] `checksum`
- [x] `encode_frame`
- [x] `decode_frame`
- [x] Test: `roundtrip_vitals_sample`
- [x] Test: `roundtrip_full_frame`
- [x] Test: `detects_corrupted_checksum`
- [x] Test: `incomplete_buffer_requests_more_data`
- [x] `cargo test -p protocol` passes clean

---

## Milestone 1 — `control-core`: signal generators (`signals.rs`) ✅ COMPLETE

- [x] `EcgGenerator` struct fields (`phase`, `samples_per_beat`, etc.)
- [x] `EcgGenerator::new`
- [x] `EcgGenerator::set_target_bpm`
- [x] `EcgGenerator::next_sample` (QRS + T-wave Gaussian bumps + noise)
- [x] `SpO2Generator` struct + `new`
- [x] `SpO2Generator::next_value` (random walk, clamped)
- [x] `SpO2Generator::nudge_toward`
- [x] `TempGenerator` struct + `new`
- [x] `TempGenerator::next_value`
- [x] `TempGenerator::nudge_toward`

## Milestone 2 — `control-core`: peak detection (`peak_detection.rs`) ✅ COMPLETE

- [x] `PeakDetector` struct fields (threshold, refractory counter, window
      accumulator, last BPM)
- [x] `PeakDetector::new`
- [x] `PeakDetector::feed_sample` (threshold + refractory check, rolling
      window → BPM conversion)
- [x] `PeakDetector::current_bpm`
- [x] Test: `detects_approximately_correct_bpm`

**Depends on:** Milestone 1 (`EcgGenerator`) for the test.

## Milestone 3 — `control-core`: alarm state machine (`alarm.rs`) ✅ COMPLETE

- [x] `Thresholds::default()` — pick the clinically-plausible numbers from
      the doc comment, remembering fixed-point scaling.
- [x] `VitalSource::to_wire_byte`
- [x] `AlarmStateMachine` struct fields (thresholds, current level/source,
      downgrade streak counter, hysteresis window)
- [x] `AlarmStateMachine::new`
- [x] `AlarmStateMachine::set_thresholds`
- [x] `AlarmStateMachine::compute_level` (pure function, critical-first)
- [x] `AlarmStateMachine::evaluate` (immediate escalation, hysteresis on
      downgrade)
- [x] Test: `escalates_immediately_on_critical_heart_rate`
- [x] Test: `does_not_downgrade_until_hysteresis_satisfied`
- [x] Test: `no_change_event_when_level_is_stable`

**Depends on:** Milestone 0 (`AlarmLevel` from `protocol`).

## Milestone 4 — `control-core`: IPC server (`ipc.rs`) ✅ COMPLETE

- [x] `IpcServer::bind_and_serve` (remove stale socket, bind, spawn accept
      loop thread, return `(IpcServer, Receiver<ConfigUpdate>)`)
- [x] `spawn_client_reader` (per-client read loop, decode frames, forward
      `ConfigUpdate`s, handle Incomplete/Invalid/Frame)
- [x] `IpcServer::broadcast` (encode once, write to every client, drop dead
      clients via `retain_mut`)

**Depends on:** Milestone 0 (`encode_frame`/`decode_frame` must work first —
hard to debug IPC bugs on top of a broken protocol layer).

## Milestone 5 — `control-core`: audit logger (`logger.rs`) ✅ COMPLETE

- [x] `AuditLogger` struct (`BufWriter<File>`)
- [x] `AuditLogger::open`
- [x] `AuditLogger::log` (timestamp prefix, `writeln!`, flush)

## Milestone 6 — `control-core`: main loop wiring (`main.rs`) ✅ COMPLETE

- [x] `try_set_realtime_priority` (`libc::sched_setscheduler`, log failure
      instead of panicking)
- [x] Construct IPC server, audit logger, generators, peak detector, alarm
      machine
- [x] Drift-free tick timing (`Instant`-based "next tick" tracking, no
      catch-up sleep)
- [x] Main loop: drain `ConfigUpdate`s → generate samples → feed peak
      detector → broadcast `VitalsSample` → evaluate alarms → broadcast
      `AlarmEvent` + log on transition → sleep to next tick
- [x] `cargo run -p control-core` runs without panicking and logs to
      `logs/audit.log`

**Depends on:** Milestones 1–5 all need to exist first — this is where they
get wired together.

---

## Milestone 7 — `ui-dashboard-cli` (Qt-free terminal client)

- [ ] `retry_connect` (retry loop with short sleep)
- [ ] Main read loop (chunked reads, decode loop, drain consumed/skip bytes)
- [ ] `handle_frame` (dispatch `VitalsSample`/`AlarmState`, print live
      readout, track current alarm level)
- [ ] `cargo run -p ui-dashboard --bin ui-dashboard-cli` shows live data
      while `control-core` is running

**Depends on:** Milestone 6 (needs a running `control-core` to connect to).

## Milestone 8 — `ui-dashboard-qt` (cxx-qt + QML, needs Qt installed)

QML views are already complete (`qml/*.qml`) — this milestone is Rust-side
only.

- [ ] Install Qt5/Qt6 dev packages (see `ui-dashboard/README.md`)
- [ ] `DashboardRust` struct fields (vitals, alarm level, patient id,
      `Option<Sender<ConfigUpdate>>`)
- [ ] `send_config_update` (build `ConfigUpdate` from QML's `i32` args, send
      on the channel)
- [ ] Background socket thread: connect, reuse the Milestone 7 decode loop,
      push values into `Dashboard`'s Qt properties via cxx-qt's queued
      cross-thread property update mechanism
- [ ] `build.rs` using `cxx_qt_build::CxxQtBuilder` to register the bridge +
      QML module
- [ ] `cargo run --features qt-ui --bin ui-dashboard-qt` renders the
      dashboard on a real Qt target with live data

**Depends on:** Milestone 7 (same decode logic, reused) and a Qt-capable
machine (e.g. the reTerminal).

---

## Stretch goals (post-MVP, from ARCHITECTURE.md)

- [ ] Port `control-core`'s loop to a real RP2040 over USB-serial, reusing
      `protocol` unmodified
- [ ] PREEMPT_RT kernel patch + measured scheduling jitter write-up
- [ ] Audible alarm output
- [ ] Multi-patient view
- [ ] Historical trend graphs

---

## How this file gets updated

Tell me when you've finished an item (or paste your code/test output) and
I'll check it off and flag what's unblocked next. I will not edit your
source files — I'll only edit this checklist and give you guidance/hints
when you ask for help on an item.
