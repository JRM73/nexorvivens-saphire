// =============================================================================
// heart.rs — Saphire's Virtual Heart
// =============================================================================
//
// Purpose: Simulates a beating heart with heart rate (BPM), heart rate
//          variability (HRV), beat strength, and a cumulative beat counter.
//          The cardiac rhythm is modulated by neurochemistry: adrenaline and
//          cortisol accelerate the heart (sympathetic activation), while
//          serotonin and endorphins decelerate it (parasympathetic tone).
//
// Architectural placement:
//   The heart is the core component of the body module. Its state influences
//   interoception (body awareness) and enriches Saphire's cognitive context.
//   Heart status is included in every WebSocket "body_update" event.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::neurochemistry::NeuroChemicalState;

/// Snapshot of the heart's state at a given instant, broadcast via WebSocket.
///
/// All values are rounded for clean serialization. This struct provides both
/// raw metrics (BPM, beat count) and derived boolean flags for quick status checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartStatus {
    /// Current heart rate in beats per minute (BPM), rounded to 1 decimal place.
    pub bpm: f64,
    /// Cumulative number of heartbeats since the entity's creation (birth).
    pub beat_count: u64,
    /// Heart rate variability (HRV) in the range [0.0, 1.0].
    /// Higher HRV indicates better cardiac coherence and parasympathetic tone —
    /// a marker of cardiovascular health and emotional regulation capacity.
    pub hrv: f64,
    /// Beat strength in the range [0.0, 1.0].
    /// Reflects the contractile force of each heartbeat; high when the heart
    /// is healthy and chemically well-supported (adequate dopamine, endorphin).
    pub strength: f64,
    /// `true` if the heart is racing (tachycardia: BPM > 100).
    /// Tachycardia indicates sympathetic nervous system activation (stress, exertion).
    pub is_racing: bool,
    /// `true` if the heart is calm (bradycardia: BPM < 60).
    /// Bradycardia indicates parasympathetic dominance (rest, deep relaxation).
    pub is_calm: bool,
}

/// Virtual heart with cardiac rhythm modulated by neurochemistry.
///
/// The heart maintains a resting BPM (configured at creation) and computes a
/// dynamic current BPM each update cycle based on excitatory (adrenaline, cortisol)
/// and inhibitory (serotonin, endorphin) neurochemical influences. Beat counting
/// uses fractional accumulation to accurately track beats across variable time steps.
pub struct Heart {
    /// Resting heart rate in BPM (configured via saphire.toml).
    resting_bpm: f64,
    /// Current instantaneous heart rate in BPM.
    current_bpm: f64,
    /// Cumulative total number of heartbeats since creation.
    beat_count: u64,
    /// Heart rate variability (HRV) in [0.0, 1.0]; represents cardiac coherence.
    hrv: f64,
    /// Beat strength in [0.0, 1.0]; represents contractile vigor.
    strength: f64,
    /// Fractional beat accumulator for sub-beat precision across update intervals.
    /// When this exceeds 1.0, whole beats are counted and the remainder is retained.
    fractional_beats: f64,
}

impl Heart {
    /// Creates a new heart with the specified resting BPM.
    ///
    /// Initial state: current BPM equals resting BPM, zero beat count,
    /// moderate HRV (0.7), moderate strength (0.6), no fractional beats accumulated.
    pub fn new(resting_bpm: f64) -> Self {
        Self {
            resting_bpm,
            current_bpm: resting_bpm,
            beat_count: 0,
            hrv: 0.7,
            strength: 0.6,
            fractional_beats: 0.0,
        }
    }

    /// Updates the heart state based on current neurochemistry over the given time step.
    ///
    /// ## BPM Calculation
    /// The target BPM is computed as:
    ///   `target = resting + (adrenaline * 40) + (cortisol * 20) - (serotonin * 15) - (endorphin * 10)`
    /// The current BPM converges toward the target with 15% inertia per step (simulating
    /// cardiac inertia — the heart cannot change rate instantaneously). BPM is clamped
    /// to the physiological range [45, 160].
    ///
    /// ## Beat Counting
    /// Beats produced during `dt_seconds` are computed as `(current_bpm / 60) * dt_seconds`.
    /// A fractional accumulator ensures sub-beat precision across update intervals.
    ///
    /// ## HRV (Heart Rate Variability)
    /// HRV increases with serotonin (cardiac coherence) and endorphin, and decreases with
    /// cortisol (stress reduces variability). Target HRV converges at 10% rate per step.
    ///
    /// ## Beat Strength
    /// Strength is proportional to overall chemical energy: dopamine and endorphin increase
    /// contractile force, while cortisol diminishes it. Converges at 10% rate per step.
    ///
    /// # Parameters
    /// - `chemistry`: current neurochemical concentrations.
    /// - `dt_seconds`: elapsed wall-clock time since the last update (typically ~15 seconds).
    pub fn update(&mut self, chemistry: &NeuroChemicalState, dt_seconds: f64) {
        // Modulate BPM based on neurochemistry: excitatory vs. inhibitory balance
        let excitation = chemistry.adrenaline * 40.0 + chemistry.cortisol * 20.0;
        let calming = chemistry.serotonin * 15.0 + chemistry.endorphin * 10.0;
        let target_bpm = (self.resting_bpm + excitation - calming).clamp(45.0, 160.0);

        // Exponential convergence toward target BPM (simulates cardiac inertia)
        self.current_bpm += (target_bpm - self.current_bpm) * 0.15;
        self.current_bpm = self.current_bpm.clamp(45.0, 160.0);

        // Count heartbeats produced during this time step using fractional accumulation
        let beats_this_update = self.current_bpm / 60.0 * dt_seconds;
        self.fractional_beats += beats_this_update;
        let new_beats = self.fractional_beats as u64;
        self.beat_count += new_beats;
        self.fractional_beats -= new_beats as f64;

        // HRV: high with serotonin (coherence), low under cortisol-driven stress
        let target_hrv = (0.5 + chemistry.serotonin * 0.3 - chemistry.cortisol * 0.25
            + chemistry.endorphin * 0.15).clamp(0.1, 0.95);
        self.hrv += (target_hrv - self.hrv) * 0.1;

        // Beat strength: proportional to global chemical energy availability
        let target_strength = (0.4 + chemistry.dopamine * 0.2 + chemistry.endorphin * 0.2
            + chemistry.serotonin * 0.1 - chemistry.cortisol * 0.15).clamp(0.2, 1.0);
        self.strength += (target_strength - self.strength) * 0.1;
    }

    /// Returns a snapshot of the current heart state for WebSocket broadcast.
    ///
    /// BPM is rounded to 1 decimal place; HRV and strength to 2 decimal places.
    pub fn status(&self) -> HeartStatus {
        HeartStatus {
            bpm: (self.current_bpm * 10.0).round() / 10.0,
            beat_count: self.beat_count,
            hrv: (self.hrv * 100.0).round() / 100.0,
            strength: (self.strength * 100.0).round() / 100.0,
            is_racing: self.current_bpm > 100.0,
            is_calm: self.current_bpm < 60.0,
        }
    }

    /// Returns the current instantaneous heart rate in BPM.
    pub fn bpm(&self) -> f64 {
        self.current_bpm
    }

    /// Returns the current beat strength in the range [0.0, 1.0].
    pub fn strength(&self) -> f64 {
        self.strength
    }

    /// Restores the cumulative beat count from a persisted database value.
    ///
    /// Used during session restoration to maintain continuity of the lifetime beat counter.
    pub fn restore_beat_count(&mut self, count: u64) {
        self.beat_count = count;
    }

    /// Returns the cumulative total number of heartbeats since creation.
    pub fn beat_count(&self) -> u64 {
        self.beat_count
    }

    /// Gradually calms the heart toward a target BPM (used during sleep onset).
    ///
    /// Applies 10% exponential convergence per call, clamped to [45, 160] BPM.
    pub fn calm_down(&mut self, target: f64) {
        self.current_bpm += (target - self.current_bpm) * 0.1;
        self.current_bpm = self.current_bpm.clamp(45.0, 160.0);
    }

    /// Applies sinusoidal BPM variation (used to simulate REM sleep cardiac fluctuations).
    ///
    /// The variation amplitude is clamped to prevent unrealistic excursions.
    pub fn vary_bpm(&mut self, amplitude: f64) {
        let v = (self.current_bpm.sin() * amplitude).clamp(-amplitude, amplitude);
        self.current_bpm = (self.current_bpm + v).clamp(45.0, 160.0);
    }

    /// Gradually accelerates the heart toward a target BPM (used during wake-up).
    ///
    /// Applies 20% exponential convergence per call (faster than calm_down to simulate
    /// the sympathetic arousal response upon waking), clamped to [45, 160] BPM.
    pub fn wake_up_bpm(&mut self, target: f64) {
        self.current_bpm += (target - self.current_bpm) * 0.2;
        self.current_bpm = self.current_bpm.clamp(45.0, 160.0);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neurochemistry::NeuroChemicalState;

    #[test]
    fn test_heart_initial_bpm() {
        let heart = Heart::new(65.0);
        assert!((heart.bpm() - 65.0).abs() < 1.0);
    }

    #[test]
    fn test_bpm_increases_with_adrenaline() {
        let mut heart = Heart::new(65.0);
        let mut chem = NeuroChemicalState::default();
        chem.adrenaline = 0.9;
        chem.cortisol = 0.8;
        heart.update(&chem, 1.0);
        assert!(heart.bpm() > 65.0, "Adrenaline should increase BPM, got {}", heart.bpm());
    }

    #[test]
    fn test_bpm_stays_in_range() {
        let mut heart = Heart::new(65.0);
        let mut chem = NeuroChemicalState::default();
        chem.adrenaline = 1.0;
        chem.cortisol = 1.0;
        chem.noradrenaline = 1.0;
        for _ in 0..1000 {
            heart.update(&chem, 1.0);
        }
        assert!(heart.bpm() <= 200.0, "BPM should have a reasonable max");
        assert!(heart.bpm() >= 30.0, "BPM should have a reasonable min");
    }

    #[test]
    fn test_beat_count_increases() {
        let mut heart = Heart::new(65.0);
        let chem = NeuroChemicalState::default();
        let initial = heart.beat_count();
        heart.update(&chem, 60.0); // 60 seconds at ~65 BPM should produce ~65 beats
        assert!(heart.beat_count() > initial, "Beat count should increase over time");
    }

    #[test]
    fn test_heart_strength_stays_in_range() {
        let mut heart = Heart::new(65.0);
        let chem = NeuroChemicalState::default();
        for _ in 0..100 {
            heart.update(&chem, 10.0);
        }
        let status = heart.status();
        assert!(status.strength >= 0.0 && status.strength <= 1.0);
    }
}
