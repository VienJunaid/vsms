# Milestone Checklist — Vital Signs Monitor Simulator

Tracks remaining implementation work. Update checkboxes as you complete
items. Ordered by dependency: `protocol` blocks everything, `control-core`
blocks `ui-dashboard`'s live data, Qt bridge is last because it needs a Qt
toolchain.

Status legend: `[ ]` not started · `[~]` in progress / has bugs · `[x]` done

---

## Milestone 0 — `protocol` crate (shared wire format)

This crate currently does **not compile**. Several functions have a
`todo!()` followed by leftover draft code after it (which is dead/unreachable
and also has bugs of its own). Both problems need fixing.

- [~] `FrameType::from_u8` — draft match exists but is unreachable after
      `todo!()`; remove the `todo!()` so the match is what executes.
- [~] `AlarmLevel::from_u8` — same `todo!()` issue, **and** the match arms
      reference bare `Normal`/`Warning`/`Critical` instead of
      `AlarmLevel::Normal` etc. — won't compile until qualified.
- [~] `VitalsSample::ENCODED_LEN` — currently `12`, which is correct
      (4+2+2+2+2), but double check against the field list comment.
- [~] `VitalsSample::encode` — byte ranges are off by one for every field
      after the first (e.g. `buf[5..6]` for a `u16` should be `buf[4..6]`).
      Each field's slice must start exactly where the previous one ended.
- [~] `VitalsSample::decode` — slice ranges have the same off-by-one bug as
      encode, a variable `d5` is referenced but never defined, and the
      struct field names (`timestamp_ms`, `heart_rate_bpm`, ...) don't match
      the placeholder names (`d1`, `d2`, ...) used in `Some(Self { ... })`.
- [~] `AlarmEvent::ENCODED_LEN` — currently `6`; confirm against
      4 (timestamp) + 1 (level) + 1 (source_vital) = 6. Looks right already.
- [~] `AlarmEvent::encode` — `todo!()` makes the rest unreachable; also
      `[0u8;Self;ENCODED_LEN]` is invalid syntax (should be `Self::ENCODED_LEN`),
      and `buf[5].copy_from_slice(...)` is wrong — indexing a single byte
      (`buf[5]`) gives you a `u8`, not a slice, so `copy_from_slice` won't
      apply; a single byte field is a plain assignment instead.
- [~] `AlarmEvent::decode` — `todo!()` blocks the draft; also references
      undefined `d1`/`d2`/`d3` and a malformed `buf[6].try_into().ok?`
      (missing the `()` on `ok`, and indexing one byte where a slice is
      needed).
- [~] `ConfigUpdate::ENCODED_LEN` — `10` is correct (5 fields × 2 bytes).
- [~] `ConfigUpdate::encode` — `[u8; Self::ENCODED_LEN]` is invalid array
      syntax (needs an initial value, e.g. `[0u8; Self::ENCODED_LEN]`); byte
      ranges have the same off-by-one bug as `VitalsSample::encode`; also has
      a stray `todo!()` after the draft body.
- [~] `ConfigUpdate::decode` — same off-by-one slice bug, and the function
      never returns `Some(Self { ... })` at the end — it computes `d1..d5`
      and then falls off the end of the function.
- [ ] `checksum` — not started. XOR `frame_type`, both length bytes, and
      every payload byte together.
- [ ] `encode_frame` — not started. Validate sizes, then write
      `START_BYTE`, type, len, payload, checksum into `out` in order.
- [ ] `decode_frame` — not started. This is the trickiest function in the
      crate (the Incomplete/Invalid/Frame decision tree) — save it for last.
- [ ] Test: `roundtrip_vitals_sample`
- [ ] Test: `roundtrip_full_frame`
- [ ] Test: `detects_corrupted_checksum`
- [ ] Test: `incomplete_buffer_requests_more_data`
- [ ] `cargo test -p protocol` passes clean

**Definition of done:** `cargo build -p protocol` succeeds with zero errors,
and all four tests above pass.

---

## Milestone 1 — `control-core`: signal generators (`signals.rs`)

- [ ] `EcgGenerator` struct fields (`phase`, `samples_per_beat`, etc.)
- [ ] `EcgGenerator::new`
- [ ] `EcgGenerator::set_target_bpm`
- [ ] `EcgGenerator::next_sample` (QRS + T-wave Gaussian bumps + noise)
- [ ] `SpO2Generator` struct + `new`
- [ ] `SpO2Generator::next_value` (random walk, clamped)
- [ ] `SpO2Generator::nudge_toward`
- [ ] `TempGenerator` struct + `new`
- [ ] `TempGenerator::next_value`
- [ ] `TempGenerator::nudge_toward`

## Milestone 2 — `control-core`: peak detection (`peak_detection.rs`)

- [ ] `PeakDetector` struct fields (threshold, refractory counter, window
      accumulator, last BPM)
- [ ] `PeakDetector::new`
- [ ] `PeakDetector::feed_sample` (threshold + refractory check, rolling
      window → BPM conversion)
- [ ] `PeakDetector::current_bpm`
- [ ] Test: `detects_approximately_correct_bpm`

**Depends on:** Milestone 1 (`EcgGenerator`) for the test.

## Milestone 3 — `control-core`: alarm state machine (`alarm.rs`)

- [ ] `Thresholds::default()` — pick the clinically-plausible numbers from
      the doc comment, remembering fixed-point scaling.
- [ ] `VitalSource::to_wire_byte`
- [ ] `AlarmStateMachine` struct fields (thresholds, current level/source,
      downgrade streak counter, hysteresis window)
- [ ] `AlarmStateMachine::new`
- [ ] `AlarmStateMachine::set_thresholds`
- [ ] `AlarmStateMachine::compute_level` (pure function, critical-first)
- [ ] `AlarmStateMachine::evaluate` (immediate escalation, hysteresis on
      downgrade)
- [ ] Test: `escalates_immediately_on_critical_heart_rate`
- [ ] Test: `does_not_downgrade_until_hysteresis_satisfied`
- [ ] Test: `no_change_event_when_level_is_stable`

**Depends on:** Milestone 0 (`AlarmLevel` from `protocol`).

## Milestone 4 — `control-core`: IPC server (`ipc.rs`)

- [ ] `IpcServer::bind_and_serve` (remove stale socket, bind, spawn accept
      loop thread, return `(IpcServer, Receiver<ConfigUpdate>)`)
- [ ] `spawn_client_reader` (per-client read loop, decode frames, forward
      `ConfigUpdate`s, handle Incomplete/Invalid/Frame)
- [ ] `IpcServer::broadcast` (encode once, write to every client, drop dead
      clients via `retain_mut`)

**Depends on:** Milestone 0 (`encode_frame`/`decode_frame` must work first —
hard to debug IPC bugs on top of a broken protocol layer).

## Milestone 5 — `control-core`: audit logger (`logger.rs`)

- [ ] `AuditLogger` struct (`BufWriter<File>`)
- [ ] `AuditLogger::open`
- [ ] `AuditLogger::log` (timestamp prefix, `writeln!`, flush)

## Milestone 6 — `control-core`: main loop wiring (`main.rs`)

- [ ] `try_set_realtime_priority` (`libc::sched_setscheduler`, log failure
      instead of panicking)
- [ ] Construct IPC server, audit logger, generators, peak detector, alarm
      machine
- [ ] Drift-free tick timing (`Instant`-based "next tick" tracking, no
      catch-up sleep)
- [ ] Main loop: drain `ConfigUpdate`s → generate samples → feed peak
      detector → broadcast `VitalsSample` → evaluate alarms → broadcast
      `AlarmEvent` + log on transition → sleep to next tick
- [ ] `cargo run -p control-core` runs without panicking and logs to
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
