//! R-peak detection over the synthetic ECG stream, used to derive
//! instantaneous heart rate.
//!
//! YOUR TASK: implement a simple threshold + refractory-period peak
//! detector. The idea: an R-peak is any sample that crosses above a
//! threshold AND it's been "long enough" since the last detected peak
//! (the refractory period) — this stops one tall spike from being counted
//! as multiple beats due to noise wobbling around the threshold.

use crate::signals::ECG_SAMPLE_RATE_HZ;

/// Detects R-peaks in an ECG sample stream and tracks heart rate.
pub struct PeakDetector {
    // TODO: think about what state you need across calls to `feed_sample`:
    // - the threshold itself (passed in at construction)
    // - how many samples must elapse after a peak before another can count
    //   (the refractory period, e.g. 200ms worth of samples — max
    //   physiological heart rate is roughly 300bpm, which sets a floor on
    //   how close together two real beats can be)
    // - a counter of samples since the last detected peak
    // - a counter + accumulator for computing BPM over a rolling window
    //   (e.g. count beats over a ~3 second window, then convert to BPM,
    //   rather than estimating from a single beat-to-beat interval, which
    //   would be noisy)
    // - the most recently computed BPM value to return from current_bpm()

    threshold_mv: f32,
    refractory_samples: u32,
    samples_since_last_peak: u32,
    beats_in_window: u32,
    window_progress: u32,
    current_bpm: u16, 
}

impl PeakDetector {
    /// Create a new detector. `threshold_mv` should sit comfortably below the
    /// expected QRS peak amplitude but above baseline noise.
    ///
    /// HINT: refractory_samples = (ECG_SAMPLE_RATE_HZ as f32 * 0.2) as u32
    /// gives you a 200ms refractory window.
    pub fn new(threshold_mv: f32) -> Self {
        let refractory_samples = (ECG_SAMPLE_RATE_HZ as f32 * 0.2) as u32;
        return Self {
            threshold_mv, 
            refractory_samples, 
            samples_since_last_peak: 0, 
            beats_in_window: 0, 
            window_progress: 0,
            current_bpm: 0,
        };
    }

    /// Feed one ECG sample (millivolts) into the detector. Returns `true` if
    /// this sample was identified as an R-peak.
    ///
    /// HINT — the core check is roughly:
    /// `sample_mv >= self.threshold_mv && self.samples_since_last_peak >= self.refractory_samples`
    /// If both are true: it's a peak — reset samples_since_last_peak to 0,
    /// increment your beats-in-window counter, return true.
    /// Either way, increment samples_since_last_peak and your
    /// window-progress counter every call.
    ///
    /// Then: once your window-progress counter reaches a chosen window size
    /// (e.g. 3 seconds worth of samples), convert beats-in-window to BPM
    /// with `(beats as f32 / minutes_in_window).round() as u16`, store it,
    /// and reset both counters for the next window.
    pub fn feed_sample(&mut self, sample_mv: f32) -> bool {
        let mut is_peak = false; 
        if sample_mv >= self.threshold_mv && self.samples_since_last_peak >= self.refractory_samples {
            self.samples_since_last_peak = 0;
            self.beats_in_window += 1;
            is_peak = true;
        }
        self.samples_since_last_peak += 1;
        self.window_progress += 1;
        if self.window_progress >= ECG_SAMPLE_RATE_HZ * 3 {
            self.current_bpm = (self.beats_in_window as f32 / (3.0 / 60.0)).round() as u16;
            self.beats_in_window = 0;
            self.window_progress = 0; 
        }
        return is_peak;
    }

    /// Most recently computed heart rate in BPM. Zero until the first window completes.
    pub fn current_bpm(&self) -> u16 {
        return self.current_bpm;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signals::EcgGenerator;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    // TODO: feed a PeakDetector several seconds of EcgGenerator output at a
    // known target_bpm (e.g. 72) and assert current_bpm() ends up somewhere
    // close to that target. Use a generous tolerance — this is a synthetic,
    // noisy signal, not a precision instrument. `StdRng::seed_from_u64(42)`
    // gives you a reproducible RNG for the test.
    #[test]
    fn detects_approximately_correct_bpm() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut ecg = EcgGenerator::new(72.0);
        let mut detector = PeakDetector::new(0.5);

        for _ in 0..ECG_SAMPLE_RATE_HZ * 10 {
            let sample = ecg.next_sample(&mut rng);
            detector.feed_sample(sample);
        }

        let bpm = detector.current_bpm();
        assert!(bpm >= 60 && bpm <= 84, "expected ~72 BPm, got {}", bpm);
    }
}
