// =============================================================================
// conditions/motion_sickness.rs — Motion sickness (kinetosis)
// =============================================================================
//
// Purpose: Models motion sickness: air sickness, sea sickness, vertigo,
//          barotrauma. Caused by a sensory conflict between the senses.
//
// Mechanics:
//   - Measures sensory conflict (gap between senses)
//   - Generates nausea if conflict > threshold * susceptibility
//   - Chemistry impact: cortisol +, comfort --, concentration --
//   - Progressive adaptation (habituation) with repeated exposures
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Type of motion sickness.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MotionType {
    /// Air sickness (altitude, turbulence)
    Air,
    /// Sea sickness (rhythmic movements)
    Sea,
    /// Land sickness (after a long trip)
    Land,
    /// Vertigo (height, rotation)
    Vertigo,
    /// Barotrauma (pressure, depth)
    Barotrauma,
}

/// Motion sickness state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionSickness {
    /// Susceptibility to motion sickness (0.0 = immune, 1.0 = very sensitive)
    pub susceptibility: f64,
    /// Current nausea level (0.0 = none, 1.0 = incapacitating)
    pub current_nausea: f64,
    /// Current sensory conflict measurement (0.0 = coherent, 1.0 = total conflict)
    pub sensory_conflict: f64,
    /// Adaptation / habituation level (0.0 = novice, 1.0 = seasoned)
    pub adaptation: f64,
    /// Active motion sickness type (None if none)
    pub active_type: Option<MotionType>,
    /// Total number of episodes
    pub total_episodes: u64,
}

impl MotionSickness {
    /// Creates a new state with a given susceptibility.
    pub fn new(susceptibility: f64) -> Self {
        Self {
            susceptibility: susceptibility.clamp(0.0, 1.0),
            current_nausea: 0.0,
            sensory_conflict: 0.0,
            adaptation: 0.0,
            active_type: None,
            total_episodes: 0,
        }
    }

    /// Evaluates sensory conflict from the 5 sense intensities.
    ///
    /// A high conflict occurs when some senses are very active
    /// and others are not at all (perceptual incoherence).
    pub fn evaluate_conflict(&mut self, sense_intensities: &[f64; 5]) {
        // Compute the variance of intensities
        let mean: f64 = sense_intensities.iter().sum::<f64>() / 5.0;
        let variance: f64 = sense_intensities.iter()
            .map(|&s| (s - mean).powi(2))
            .sum::<f64>() / 5.0;

        // Conflict = normalized variance (theoretical max ~0.25)
        self.sensory_conflict = (variance * 4.0).clamp(0.0, 1.0);
    }

    /// Triggers an episode of a specific type (via API or scenario).
    pub fn trigger(&mut self, motion_type: MotionType) {
        self.active_type = Some(motion_type);
        self.sensory_conflict = 0.7; // Artificially high conflict
        self.total_episodes += 1;
    }

    /// Updates nausea based on conflict and susceptibility.
    ///
    /// Called each cycle when motion sickness is active.
    pub fn tick(&mut self) {
        // Nausea = conflict * susceptibility * (1 - adaptation)
        let effective_susceptibility = self.susceptibility * (1.0 - self.adaptation * 0.7);
        let target_nausea = (self.sensory_conflict * effective_susceptibility).clamp(0.0, 1.0);

        // Smooth convergence toward target
        self.current_nausea += (target_nausea - self.current_nausea) * 0.15;
        self.current_nausea = self.current_nausea.clamp(0.0, 1.0);

        // Progressive adaptation (repeated exposure -> habituation)
        if self.sensory_conflict > 0.3 {
            self.adaptation = (self.adaptation + 0.001).min(1.0);
        }

        // Natural decay of sensory conflict
        self.sensory_conflict = (self.sensory_conflict - 0.02).max(0.0);

        // If the conflict is resolved, nausea disappears
        if self.sensory_conflict < 0.05 {
            self.active_type = None;
        }
    }

    /// Chemistry impact: cortisol +, serotonin -, endorphin +.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        if self.current_nausea < 0.1 {
            return ChemistryAdjustment::default();
        }

        ChemistryAdjustment {
            cortisol: self.current_nausea * 0.03,
            serotonin: -self.current_nausea * 0.02,
            endorphin: self.current_nausea * 0.01, // analgesic response
            adrenaline: if self.active_type == Some(MotionType::Vertigo) {
                self.current_nausea * 0.04 // vertigo = panic
            } else {
                0.0
            },
            ..Default::default()
        }
    }

    /// Cognitive degradation due to nausea [0.0 - 0.3].
    pub fn cognitive_impact(&self) -> f64 {
        if self.current_nausea > 0.7 {
            0.3 // Incapacitating
        } else if self.current_nausea > 0.3 {
            self.current_nausea * 0.2
        } else {
            0.0
        }
    }

    /// Serializes for the API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "susceptibility": self.susceptibility,
            "current_nausea": self.current_nausea,
            "sensory_conflict": self.sensory_conflict,
            "adaptation": self.adaptation,
            "active_type": self.active_type.as_ref().map(|t| format!("{:?}", t)),
            "total_episodes": self.total_episodes,
            "cognitive_impact": self.cognitive_impact(),
        })
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_conflict_no_nausea() {
        let mut ms = MotionSickness::new(0.8);
        // All senses at the same intensity = no conflict
        ms.evaluate_conflict(&[0.5, 0.5, 0.5, 0.5, 0.5]);
        ms.tick();
        assert!(ms.current_nausea < 0.01);
    }

    #[test]
    fn test_high_conflict_causes_nausea() {
        let mut ms = MotionSickness::new(0.8);
        // Large gap between senses
        ms.evaluate_conflict(&[1.0, 0.0, 1.0, 0.0, 0.5]);
        ms.tick();
        assert!(ms.current_nausea > 0.0);
        assert!(ms.sensory_conflict > 0.3);
    }

    #[test]
    fn test_low_susceptibility_resists() {
        let mut ms = MotionSickness::new(0.1);
        ms.evaluate_conflict(&[1.0, 0.0, 1.0, 0.0, 0.5]);
        ms.tick();
        // Same conflict but low susceptibility -> reduced nausea
        assert!(ms.current_nausea < 0.1);
    }

    #[test]
    fn test_adaptation_reduces_nausea() {
        let mut ms = MotionSickness::new(0.8);
        ms.adaptation = 0.9; // Already well adapted
        ms.evaluate_conflict(&[1.0, 0.0, 1.0, 0.0, 0.5]);
        ms.tick();
        assert!(ms.current_nausea < 0.1);
    }

    #[test]
    fn test_trigger_episode() {
        let mut ms = MotionSickness::new(0.5);
        ms.trigger(MotionType::Vertigo);
        assert!(ms.sensory_conflict > 0.5);
        assert_eq!(ms.total_episodes, 1);
        ms.tick();
        assert!(ms.current_nausea > 0.0);
    }

    #[test]
    fn test_chemistry_influence_vertigo() {
        let mut ms = MotionSickness::new(0.8);
        ms.trigger(MotionType::Vertigo);
        // Several ticks for nausea to converge above threshold
        for _ in 0..10 {
            ms.tick();
            // Re-inject the conflict as it decays
            ms.sensory_conflict = 0.7;
        }
        let adj = ms.chemistry_influence();
        assert!(adj.cortisol > 0.0);
        assert!(adj.adrenaline > 0.0); // Vertigo = panic
    }
}
