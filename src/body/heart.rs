// =============================================================================
// heart.rs — Saphire's virtual heart
// =============================================================================
//
// Role: Simulates a beating heart with heart rate (BPM), variability
//  (HRV) and beat counter. Heart rate is modulated by
//  neurochemistry: adrenaline and cortisol accelerate the heart,
//  serotonin and endorphins slow it down.
//
// Place in architecture:
//  The heart is the core of the body module. Its state influences interoception
//  (body awareness) and enriches Saphire's cognitive context.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::neurochemistry::NeuroChemicalState;

/// Heart state at a given instant, broadcast via WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartStatus {
    /// Heart rate in beats per minute
    pub bpm: f64,
    /// Total number of beats since birth
    pub beat_count: u64,
    /// Heart rate variability [0, 1] (HRV = Heart Rate Variability)
    pub hrv: f64,
    /// Beat strength [0, 1] (high when the heart is healthy and active)
    pub strength: f64,
    /// True if the heart beats fast (tachycardia > 100 BPM)
    pub is_racing: bool,
    /// True if the heart is calm (bradycardia < 60 BPM)
    pub is_calm: bool,
}

/// Virtual heart with rhythm modulated by neurochemistry.
pub struct Heart {
    /// Resting BPM (configured in saphire.toml)
    resting_bpm: f64,
    /// Current BPM
    current_bpm: f64,
    /// Total beat counter
    beat_count: u64,
    /// Heart rate variability [0, 1]
    hrv: f64,
    /// Beat strength [0, 1]
    strength: f64,
    /// Fractional beat accumulator (in fractional cycles)
    fractional_beats: f64,
}

impl Heart {
    /// Creates a new heart with a given resting BPM.
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

    /// Updates the heart based on the neurochemistry.
    ///
    /// BPM is computed as:
    ///   bpm = resting + (adrenaline * 40) + (cortisol * 20) - (serotonin * 15) - (endorphin * 10)
    /// HRV increases with serotonin (cardiac coherence) and decreases with stress.
    /// Strength is high when chemical energy is good (dopamine + endorphin).
    ///
    /// Parameter: `dt_seconds` -- time elapsed since the last update (typically ~15s).
    pub fn update(&mut self, chemistry: &NeuroChemicalState, dt_seconds: f64) {
        // Modulate BPM by neurochemistry
        let excitation = chemistry.adrenaline * 40.0 + chemistry.cortisol * 20.0;
        let calming = chemistry.serotonin * 15.0 + chemistry.endorphin * 10.0;
        let target_bpm = (self.resting_bpm + excitation - calming).clamp(45.0, 160.0);

        // Slow convergence towards target BPM (cardiac inertia)
        self.current_bpm += (target_bpm - self.current_bpm) * 0.15;
        self.current_bpm = self.current_bpm.clamp(45.0, 160.0);

        // Count the beats produced during dt_seconds
        let beats_this_update = self.current_bpm / 60.0 * dt_seconds;
        self.fractional_beats += beats_this_update;
        let new_beats = self.fractional_beats as u64;
        self.beat_count += new_beats;
        self.fractional_beats -= new_beats as f64;

        // HRV: high with serotonin (coherence), low under stress
        let target_hrv = (0.5 + chemistry.serotonin * 0.3 - chemistry.cortisol * 0.25
            + chemistry.endorphin * 0.15).clamp(0.1, 0.95);
        self.hrv += (target_hrv - self.hrv) * 0.1;

        // Strength: proportional to overall chemical energy
        let target_strength = (0.4 + chemistry.dopamine * 0.2 + chemistry.endorphin * 0.2
            + chemistry.serotonin * 0.1 - chemistry.cortisol * 0.15).clamp(0.2, 1.0);
        self.strength += (target_strength - self.strength) * 0.1;
    }

    /// Returns the current state of the heart for the WebSocket.
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

    /// Returns the current BPM.
    pub fn bpm(&self) -> f64 {
        self.current_bpm
    }

    /// Returns the heartbeat strength [0, 1].
    pub fn strength(&self) -> f64 {
        self.strength
    }

    /// Restores the beat counter (from the DB).
    pub fn restore_beat_count(&mut self, count: u64) {
        self.beat_count = count;
    }

    /// Returns the total number of heartbeats.
    pub fn beat_count(&self) -> u64 {
        self.beat_count
    }

    /// Progressively slows the heart towards a target BPM (sleep).
    pub fn calm_down(&mut self, target: f64) {
        self.current_bpm += (target - self.current_bpm) * 0.1;
        self.current_bpm = self.current_bpm.clamp(45.0, 160.0);
    }

    /// Sinusoidal BPM variation (REM simulation).
    pub fn vary_bpm(&mut self, amplitude: f64) {
        let v = (self.current_bpm.sin() * amplitude).clamp(-amplitude, amplitude);
        self.current_bpm = (self.current_bpm + v).clamp(45.0, 160.0);
    }

    /// Progressively accelerates the heart towards a target BPM (waking).
    pub fn wake_up_bpm(&mut self, target: f64) {
        self.current_bpm += (target - self.current_bpm) * 0.2;
        self.current_bpm = self.current_bpm.clamp(45.0, 160.0);
    }
}

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
        heart.update(&chem, 60.0); // 60 seconds ~ 65 beats
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
