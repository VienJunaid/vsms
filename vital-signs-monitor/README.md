# Vital Signs Monitor Simulator

A two-tier embedded systems project simulating a medical patient monitor,
built to mirror real-world architectural patterns used in regulated
medical device firmware: **strict separation between a real-time safety
core and a non-real-time HMI/UI layer.**

This is a portfolio/learning project aimed at demonstrating systems design
practices relevant to the medical device industry (control loop isolation,
deterministic timing, structured logging/audit trails, and memory-safe
firmware), not a certified or clinically validated device.

## Architecture at a glance

```
┌─────────────────────────┐         UART/USB         ┌──────────────────────────────┐
│      Control Core        │  ── structured frames ──▶│        UI Dashboard           │
│  (RTIC, RP2040/STM32)    │  ◀── config/ack frames ──│  (Rust + Slint,               │
│                          │                           │   runs on reTerminal Linux)   │
│  - ECG/SpO2/Temp sim     │                           │  - Live waveform              │
│  - Peak detection (HR)   │                           │  - Vitals readout              │
│  - Alarm state machine   │                           │  - Alarm banner                │
│  - Hard timing guarantees│                           │  - Settings panel               │
└─────────────────────────┘                           └──────────────────────────────┘
```

See [ARCHITECTURE.md](./ARCHITECTURE.md) for the full design doc.

## Why this project exists

Medical device software architecture commonly isolates the part of the
system that must never lag or fail (control/monitoring loops, alarms)
from the part that can be slower or even crash without immediate patient
risk (the touchscreen UI). This project recreates that boundary using:

- **Rust** on both sides of the boundary, for memory safety in both the
  real-time firmware and the UI application.
- A **defined wire protocol** between the two tiers, so the UI process
  can never stall or corrupt the control core's behavior.
- A basic **audit-trail style logging** layer, gesturing at the kind of
  data integrity/traceability expectations found in standards like
  IEC 62304 and 21 CFR Part 11 (again: not a compliance claim, just a
  design exercise informed by those expectations).

## Repo layout

```
vital-signs-monitor/
├── control-core/      Rust firmware (RTIC), runs on RP2040 or STM32
├── ui-dashboard/       Rust + Slint app, runs on the reTerminal
├── protocol/           Shared protocol definitions (frame format, crate used by both sides)
├── docs/               Design notes, timing budgets, state machine diagrams
├── logs/                Sample/example audit-trail logs (gitignored at runtime)
├── ARCHITECTURE.md
└── README.md
```

## Status

🚧 Early scaffolding — architecture and protocol design phase.

## Hardware targets

- **Control core:** Raspberry Pi Pico (RP2040) — primary target, cheapest
  and best Rust embedded support via `embassy`/`rtic`. STM32 Nucleo as a
  stretch goal/secondary target.
- **UI host:** Seeed reTerminal (CM4-based, runs Debian Linux).

## License

MIT (see LICENSE) — this is a learning/portfolio project, not a medical
device, and must not be used for any clinical or patient-facing purpose.
