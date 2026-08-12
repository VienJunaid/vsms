# Milestone Checklist — Vital Signs Monitor Simulator

Tracks remaining implementation work. Update checkboxes as you complete
items. Ordered by dependency: `protocol` blocks everything, `control-core`
blocks `ui-dashboard`'s live data, the Slint UI is last because it's the
top of the stack.

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

## Milestone 7 — `ui-dashboard-cli` (Qt-free terminal client) ✅ COMPLETE

- [x] `retry_connect` (retry loop with short sleep)
- [x] Main read loop (chunked reads, decode loop, drain consumed/skip bytes)
- [x] `handle_frame` (dispatch `VitalsSample`/`AlarmState`, print live
      readout, track current alarm level)
- [x] `cargo run -p ui-dashboard --bin ui-dashboard-cli` shows live data
      while `control-core` is running

**Depends on:** Milestone 6 (needs a running `control-core` to connect to).

## Milestone 8 — `ui-dashboard-slint` (Slint UI) ✅ COMPLETE

Switched from cxx-qt + QML to [Slint](https://slint.dev) — pure Rust, no
system Qt/C++ toolchain needed, which was the source of three separate
build failures on the reTerminal under the old approach.

- [x] `ui/*.slint` views, ported 1:1 from the old QML breakdown:
      `alarm_banner.slint`, `vitals_panel.slint` / `vital_tile.slint`,
      `waveform.slint`, `settings_panel.slint` / `threshold_slider.slint`,
      `main.slint`
- [x] `build.rs` using `slint_build::compile("ui/main.slint")`
- [x] `src/slint_main.rs`: background socket thread reusing the Milestone 7
      decode loop, pushes decoded values into the window via
      `Weak::upgrade_in_event_loop` (Slint's cross-thread-safe property
      update mechanism — the equivalent of cxx-qt's queued property setters)
- [x] `on_apply_thresholds` callback: builds a `ConfigUpdate` from the
      settings panel's slider values, sends on the channel
- [x] `Cargo.toml` second `[[bin]]` for `ui-dashboard-slint` — no feature
      gate needed, it just builds
- [x] `cargo run --bin ui-dashboard-slint` renders the dashboard with live
      data (verified against a running `control-core`)

**Depends on:** Milestone 7 (same decode logic, reused).

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
