// =============================================================================
// conditions/eating.rs — Eating disorders
// =============================================================================
//
// Purpose: Models anorexia, bulimia, and binge eating.
//          Modifies hunger perception, relationship with food,
//          and impacts chemistry (cortisol, dopamine, serotonin).
//
// Integration:
//   Modifies PrimaryNeeds behavior (hunger/thirst) in the pipeline.
//   Anorexia underestimates hunger, bulimia alternates restriction/craving,
//   binge eating is triggered by stress.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Type of eating disorder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EatingDisorderType {
    /// Food restriction, ignores hunger
    Anorexia,
    /// Restriction -> binge -> purge cycles
    Bulimia,
    /// Compulsive eating under stress
    BingeEating,
}

/// Eating disorder state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EatingDisorder {
    /// Type of disorder
    pub disorder_type: EatingDisorderType,
    /// Severity (0.0 = mild, 1.0 = severe)
    pub severity: f64,
    /// Hunger perception bias (-1.0 = underestimates, +1.0 = overestimates)
    pub hunger_perception_bias: f64,
    /// Compulsive eating urge (0.0 = none, 1.0 = irresistible)
    pub binge_craving: f64,
    /// Guilt after eating (0.0 = none, 1.0 = overwhelming)
    pub guilt_after_eating: f64,
    /// Cycle counter since last meal
    pub cycles_since_meal: u64,
    /// Binge cycle counter
    binge_cycles: u64,
}

impl EatingDisorder {
    /// Creates an eating disorder.
    pub fn new(disorder_type: EatingDisorderType, severity: f64) -> Self {
        let bias = match &disorder_type {
            EatingDisorderType::Anorexia => -severity,
            EatingDisorderType::BingeEating => severity * 0.5,
            EatingDisorderType::Bulimia => 0.0, // Fluctuates
        };
        Self {
            disorder_type,
            severity: severity.clamp(0.0, 1.0),
            hunger_perception_bias: bias,
            binge_craving: 0.0,
            guilt_after_eating: 0.0,
            cycles_since_meal: 0,
            binge_cycles: 0,
        }
    }

    /// Updates the state each cycle.
    pub fn tick(&mut self, actual_hunger: f64, cortisol: f64) {
        self.cycles_since_meal += 1;

        match self.disorder_type {
            EatingDisorderType::Anorexia => {
                // Ignores hunger -> progressively underestimates
                self.hunger_perception_bias = (-self.severity * 0.8).clamp(-1.0, 0.0);
                // Guilt increases after eating
                self.guilt_after_eating = (self.guilt_after_eating - 0.01).max(0.0);
            }
            EatingDisorderType::Bulimia => {
                // Cycle: restriction -> craving rises -> binge -> purge -> guilt
                if self.binge_craving < 0.5 {
                    // Restriction phase: craving rises with actual hunger
                    self.binge_craving = (self.binge_craving + actual_hunger * 0.02).min(1.0);
                    self.hunger_perception_bias = -self.severity * 0.5;
                } else {
                    // Binge phase
                    self.binge_cycles += 1;
                    self.hunger_perception_bias = self.severity;
                    if self.binge_cycles > 5 {
                        // Purge / guilt
                        self.guilt_after_eating = self.severity * 0.8;
                        self.binge_craving = 0.0;
                        self.binge_cycles = 0;
                    }
                }
            }
            EatingDisorderType::BingeEating => {
                // Stress -> urge to eat
                self.binge_craving = (cortisol * self.severity).clamp(0.0, 1.0);
                self.hunger_perception_bias = self.binge_craving * 0.5;
                // Guilt if craving is high
                if self.binge_craving > 0.5 {
                    self.guilt_after_eating = (self.guilt_after_eating + 0.02).min(0.8);
                } else {
                    self.guilt_after_eating = (self.guilt_after_eating - 0.01).max(0.0);
                }
            }
        }
    }

    /// Perceived hunger (modified by the disorder's bias).
    pub fn perceived_hunger(&self, actual_hunger: f64) -> f64 {
        (actual_hunger + self.hunger_perception_bias).clamp(0.0, 1.0)
    }

    /// Chemistry impact of the disorder.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        // Guilt -> cortisol
        adj.cortisol += self.guilt_after_eating * 0.02;
        adj.serotonin -= self.guilt_after_eating * 0.01;

        // Craving -> dopamine (anticipation)
        adj.dopamine += self.binge_craving * 0.02;

        // Severe anorexia -> chronic cortisol
        if self.disorder_type == EatingDisorderType::Anorexia && self.severity > 0.5 {
            adj.cortisol += self.severity * 0.01;
        }

        adj
    }

    /// Serializes for the API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": format!("{:?}", self.disorder_type),
            "severity": self.severity,
            "hunger_perception_bias": self.hunger_perception_bias,
            "binge_craving": self.binge_craving,
            "guilt_after_eating": self.guilt_after_eating,
            "perceived_vs_actual": self.hunger_perception_bias,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anorexia_underestimates_hunger() {
        let ed = EatingDisorder::new(EatingDisorderType::Anorexia, 0.8);
        assert!(ed.perceived_hunger(0.7) < 0.7);
    }

    #[test]
    fn test_binge_eating_stress_craving() {
        let mut ed = EatingDisorder::new(EatingDisorderType::BingeEating, 0.7);
        ed.tick(0.3, 0.8); // High cortisol
        assert!(ed.binge_craving > 0.3);
    }

    #[test]
    fn test_guilt_chemistry() {
        let mut ed = EatingDisorder::new(EatingDisorderType::Anorexia, 0.8);
        ed.guilt_after_eating = 0.7;
        let adj = ed.chemistry_influence();
        assert!(adj.cortisol > 0.0);
    }
}
