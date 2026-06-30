//! Alarm state machine: Normal -> Warning -> Critical, with hysteresis.
//!
//! YOUR TASK: given a sample of vitals, decide the alarm level, and apply
//! hysteresis so the level doesn't flicker when a vital sits right at a
//! threshold. Real alarm-management standards (e.g. IEC 60601-1-8) care a
//! lot about this — flickering alarms erode clinician trust.
//!
//! KEY DESIGN CHOICE to make explicit in your implementation: escalations
//! (Normal -> Warning -> Critical) should happen IMMEDIATELY — you never
//! want to delay telling someone things got worse. Downgrades should
//! require several consecutive "things look fine now" readings in a row
//! before you believe it — that's the hysteresis.

use protocol::AlarmLevel;

/// Configurable thresholds for each monitored vital.
#[derive(Debug, Clone, Copy)]
pub struct Thresholds {
    /// Heart rate warning band (BPM): below this or above `hr_warning_high` is a warning.
    pub hr_warning_low: u16,
    /// See `hr_warning_low`.
    pub hr_warning_high: u16,
    /// Heart rate critical band (BPM): below this or above `hr_critical_high` is critical.
    pub hr_critical_low: u16,
    /// See `hr_critical_low`.
    pub hr_critical_high: u16,
    /// SpO2 warning floor, percent * 10.
    pub spo2_warning_low: u16,
    /// SpO2 critical floor, percent * 10.
    pub spo2_critical_low: u16,
    /// Temperature warning band, degrees C * 100.
    pub temp_warning_low: u16,
    /// See `temp_warning_low`.
    pub temp_warning_high: u16,
    /// Temperature critical band, degrees C * 100.
    pub temp_critical_low: u16,
    /// See `temp_critical_low`.
    pub temp_critical_high: u16,
}

impl Default for Thresholds {
    /// HINT: pick clinically-plausible numbers. Some starting points:
    /// HR warning 50-110bpm, critical 40-140bpm. SpO2 warning <92.0%,
    /// critical <88.0%. Temp warning 36.0-38.5C, critical 35.0-39.5C.
    /// Remember the *_permille / *_centi_c fixed-point scaling from protocol.rs!
    fn default() -> Self {
        todo!()
    }
}

/// Identifies which vital most recently drove an alarm transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VitalSource {
    /// Heart rate.
    HeartRate,
    /// Blood oxygen saturation.
    SpO2,
    /// Body temperature.
    Temperature,
    /// No active alarm source (state is Normal).
    None,
}

impl VitalSource {
    /// Encode as the wire-protocol's `source_vital` byte.
    /// HINT: match each variant to the byte values used in AlarmEvent's docs
    /// (0=HR, 1=SpO2, 2=Temp, 255=cleared/none).
    pub fn to_wire_byte(self) -> u8 {
        todo!()
    }
}

/// Tracks the current alarm level and applies hysteresis (a minimum dwell
/// time) before allowing a downgrade, so the level doesn't flicker.
pub struct AlarmStateMachine {
    // TODO: you'll want to track:
    // - the active thresholds
    // - the current confirmed alarm level and its source
    // - a counter of how many consecutive evaluations have computed a LOWER
    //   level than the current confirmed one (the "downgrade streak")
    // - how many consecutive lower-readings are required before you actually
    //   downgrade (the hysteresis window, passed in at construction)
}

impl AlarmStateMachine {
    /// Create a new state machine with the given thresholds and hysteresis window.
    pub fn new(thresholds: Thresholds, downgrade_hysteresis: u32) -> Self {
        todo!()
    }

    /// Replace the active thresholds (e.g. from a UI ConfigUpdate frame).
    pub fn set_thresholds(&mut self, thresholds: Thresholds) {
        todo!()
    }

    /// Evaluate one sample of vitals. Returns `Some((level, source))` if the
    /// alarm state changed this call, or `None` if it stayed the same.
    ///
    /// HINT — pseudocode:
    /// ```text
    /// let (computed_level, computed_source) = self.compute_level(hr, spo2, temp);
    /// if computed_level is MORE severe than self.current_level {
    ///     update current_level/source immediately, reset downgrade streak, return Some(...)
    /// } else if computed_level is LESS severe {
    ///     increment downgrade streak
    ///     if streak >= hysteresis threshold { commit the downgrade, return Some(...) }
    ///     else { return None }
    /// } else {
    ///     // same level as before — reset any partial downgrade streak, return None
    /// }
    /// ```
    /// You'll need some way to compare severity — could be as simple as
    /// casting AlarmLevel to its underlying u8 value and comparing those.
    pub fn evaluate(
        &mut self,
        heart_rate_bpm: u16,
        spo2_permille: u16,
        temp_centi_c: u16,
    ) -> Option<(AlarmLevel, VitalSource)> {
        todo!()
    }

    /// Pure function: given a vitals sample and the current thresholds, what
    /// alarm level/source SHOULD this be, with no memory of past calls?
    ///
    /// HINT: check critical conditions first (since critical should "win"
    /// over warning if both are somehow true), then warning conditions, then
    /// fall through to Normal/VitalSource::None. Check each vital's critical
    /// band, then each vital's warning band — return as soon as you find a
    /// match rather than checking everything every time.
    fn compute_level(&self, hr: u16, spo2: u16, temp: u16) -> (AlarmLevel, VitalSource) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: construct an AlarmStateMachine, feed it a clearly-critical heart
    // rate (e.g. very low, like 20bpm), and assert evaluate() immediately
    // returns Some((AlarmLevel::Critical, VitalSource::HeartRate)) on the
    // very first call — escalation should never be delayed.
    #[test]
    fn escalates_immediately_on_critical_heart_rate() {
        todo!()
    }

    // TODO: trigger a critical alarm, then feed several consecutive *normal*
    // readings — fewer than your hysteresis window — and assert evaluate()
    // keeps returning None (the alarm hasn't cleared yet). Then feed one more
    // normal reading to cross the hysteresis threshold and assert you finally
    // get Some((AlarmLevel::Normal, VitalSource::None)).
    #[test]
    fn does_not_downgrade_until_hysteresis_satisfied() {
        todo!()
    }

    // TODO: feed two different but both-Normal readings in a row and assert
    // evaluate() returns None both times — no spurious "change" events when
    // the level genuinely hasn't moved.
    #[test]
    fn no_change_event_when_level_is_stable() {
        todo!()
    }
}
