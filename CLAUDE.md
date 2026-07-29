# CLAUDE.md — Project Notes for AI Assistants

## Project

Vital Signs Monitor Simulator — two-process Rust app running on a Seeed reTerminal CM4
(aarch64 Debian Linux). `control-core` generates simulated vitals and broadcasts over a
Unix socket. `ui-dashboard-qt` renders them via cxx-qt + QML on the reTerminal's 1280×720
touchscreen.

Workspace root: `vital-signs-monitor/`
Milestone tracker: `vital-signs-monitor/MILESTONES.md`

---

## Known Bugs / Build Issues (reTerminal CM4)

### Bug 1 — Virtual manifest cannot have `[[bin]]` sections

**Error:**
```
error: This virtual manifest specifies a bin section, which is not allowed
```

**Cause:** Stray `[[bin]]` entries were accidentally added to the workspace root
`vital-signs-monitor/Cargo.toml`. A workspace-only (virtual) manifest cannot define
binaries — those belong in the individual crate's `Cargo.toml`.

**Fix:** Remove any `[[bin]]` blocks from `vital-signs-monitor/Cargo.toml`. The correct
`[[bin]]` entries already exist in `vital-signs-monitor/ui-dashboard/Cargo.toml`.

---

### Bug 2 — Linker error: undefined references to cxx-qt bridge symbols

**Error:**
```
error: linking with `cc` failed: exit status: 1
  ...
  error: undefined reference to 'cxxbridge1$...$Dashboard$heart_rate'
  error: undefined reference to 'cxx_qt_dashboard$...$create_rs_dashboard_rust'
```

**Cause:** The binary entry point (`src/qt_main.rs`) did not declare `mod bridge;`, so
`bridge.rs` was never compiled into the binary. The C++ side (compiled by `build.rs`) had
symbols referencing Rust functions that didn't exist in the output.

**Fix:** Add `mod bridge;` as the first line of `src/qt_main.rs`.

---

### Bug 3 — cxx-qt proc macro expansion fails with `found /` and `include` in foreign item position

**Errors:**
```
error: expected one of `!`, `(`, `+`, `::`, `<`, `>`, or `as`, found `/`
  --> ui-dashboard/src/bridge.rs:31:1
   |
31 | #[cxx_qt::bridge]
   | ^^^^^^^^^^^^^^^^^ expected one of 7 possible tokens
   |
   = note: this error originates in the attribute macro `cxx_qt::bridge`

error: non-foreign item macro in foreign item position: include
```

**Cause (part A):** The bridge file was originally at `src/qt/bridge.rs` (two directory
levels deep). cxx-qt 0.6 derives internal identifiers from the source file's path. The two
`/` separators in `src/qt/bridge.rs` were leaking into the proc macro's generated Rust as
bare division tokens, producing a syntax error.

**Fix (part A):** Moved the bridge and Qt entry point to flat paths:
- `src/qt/bridge.rs` → `src/bridge.rs`
- `src/qt/main.rs`   → `src/qt_main.rs`
- Updated `build.rs` `rust_files` from `"src/qt/bridge.rs"` to `"src/bridge.rs"`
- Updated `Cargo.toml` `[[bin]]` path from `src/qt/main.rs` to `src/qt_main.rs`

**Cause (part B):** The bridge originally had an `unsafe extern "C++"` block with
`include!("cxx-qt-lib/qstring.h")` and `type QString = cxx_qt_lib::QString` for a
`patient_id: QString` property. The cxx-qt 0.6.1 proc macro failed to expand this
combination on the reTerminal.

**Fix (part B):** Removed the `patient_id` property and the entire `unsafe extern "C++"`
block. `patient_id` was never populated from Rust anyway (`ConfigUpdate` has no patient ID
field), and `SettingsPanel.qml` has its own `property string patientId: ""` default.

---

## Running the App on the reTerminal

```sh
# Terminal 1 — start the data source
cd ~/vsms/vital-signs-monitor
cargo run -p control-core

# Terminal 2 — start the Qt UI (wait for Terminal 1 to be running first)
cd ~/vsms/vital-signs-monitor/ui-dashboard
cargo run --features qt-ui --bin ui-dashboard-qt
```

Qt dev packages required on the reTerminal (one-time setup):
```sh
sudo apt-get install qtbase5-dev qtdeclarative5-dev \
    qml-module-qtquick2 qml-module-qtquick-controls2 qml-module-qtquick-layouts \
    build-essential pkg-config libssl-dev
```
