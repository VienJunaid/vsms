//! Synthetic vitals signal generation.
//!
//! YOUR TASK: generate plausible-looking ECG/SpO2/Temp signals. These don't
//! need to be physiologically accurate — just good enough to drive peak
//! detection and exercise the alarm system.

use rand::Rng;

/// Sample rate for the ECG signal, in Hz. 250Hz is a common clinical floor.
pub const ECG_SAMPLE_RATE_HZ: u32 = 250;

/// Generates a synthetic ECG waveform as a stream of millivolt samples.
///
/// DESIGN IDEA: model one heartbeat as a repeating cycle of length
/// `samples_per_beat = (60.0 / bpm) * ECG_SAMPLE_RATE_HZ`. Track a `phase`
/// counter that increments each sample and wraps back to 0 every
/// `samples_per_beat` samples — that wraparound is "one heartbeat happened."
/// Within one cycle, a sharp narrow spike near the start approximates a QRS
/// complex; a smaller, wider bump later approximates a T-wave.
pub struct EcgGenerator {
    /// Current simulated heart rate target, in BPM.
    pub target_bpm: f32,
    // TODO: add the fields you need to track progress through a beat cycle,
    // e.g. a `phase: f32` counter and a precomputed `samples_per_beat: f32`.
}

impl EcgGenerator {
    /// Create a new generator at a given starting heart rate.
    pub fn new(target_bpm: f32) -> Self {
        todo!("compute samples_per_beat from target_bpm and ECG_SAMPLE_RATE_HZ, init phase to 0")
    }

    /// Update the target heart rate (e.g. from a "scenario" driving abnormal states).
    pub fn set_target_bpm(&mut self, bpm: f32) {
        todo!("update target_bpm and recompute samples_per_beat")
    }

    /// Advance the generator by one sample period and return the next ECG
    /// value in millivolts.
    ///
    /// HINT: a Gaussian bump centered at some phase fraction `c` with width
    /// `w` looks like: `(-((x - c).powi(2)) / (2.0 * w * w)).exp()`. Sum a
    /// tall narrow one (the QRS spike, e.g. center=0.06, width=0.015) with a
    /// shorter wider one later in the cycle (the T-wave, e.g. center=0.28,
    /// width=0.05), where `x = phase / samples_per_beat` is the
    /// fraction-of-cycle-complete. Add a little `rng.gen_range(-0.02..0.02)`
    /// noise on top so it doesn't look perfectly synthetic.
    pub fn next_sample(&mut self, rng: &mut impl Rng) -> f32 {
        todo!("increment phase (wrapping at samples_per_beat), compute the QRS+T-wave shape, add noise")
    }
}

/// Generates a slow-drifting SpO2 percentage (e.g. 95.0-99.0 normally).
///
/// DESIGN IDEA: a "random walk" — each tick, nudge the current value by a
/// small random delta, then clamp it to a plausible range so it can't drift
/// off into nonsense.
pub struct SpO2Generator {
    // TODO: track the current value, e.g. `current_permille: f32`.
}

impl SpO2Generator {
    /// Create a new generator starting at a healthy baseline.
    pub fn new(start_permille: f32) -> Self {
        todo!()
    }

    /// Advance by one tick (intended to be called ~1x/sec) and return the
    /// current value in percent * 10 (permille-style fixed point).
    ///
    /// HINT: `let step: f32 = rng.gen_range(-2.0..2.0);` then
    /// `self.current = (self.current + step).clamp(700.0, 1000.0);`
    /// then round and cast to u16.
    pub fn next_value(&mut self, rng: &mut impl Rng) -> u16 {
        todo!()
    }

    /// Force the value toward a target (used by scenario injection for demos/tests).
    /// HINT: exponential approach — `self.current += (target - self.current) * 0.1`
    /// moves 10% of the remaining distance each call, which looks like a
    /// smooth transition rather than a jump if called repeatedly.
    pub fn nudge_toward(&mut self, target_permille: f32) {
        todo!()
    }
}

/// Generates a slow-drifting body temperature.
pub struct TempGenerator {
    // TODO: same shape as SpO2Generator above.
}

impl TempGenerator {
    /// Create a new generator starting at a healthy baseline (e.g. 37.0C -> 3700 centi-C).
    pub fn new(start_centi_c: f32) -> Self {
        todo!()
    }

    /// Advance by one tick (intended to be called ~1x/sec) and return the
    /// current value in degrees C * 100.
    pub fn next_value(&mut self, rng: &mut impl Rng) -> u16 {
        todo!()
    }

    /// Force the value toward a target (used by scenario injection for demos/tests).
    pub fn nudge_toward(&mut self, target_centi_c: f32) {
        todo!()
    }
}
