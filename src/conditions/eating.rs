// =============================================================================
// conditions/eating.rs — Troubles alimentaires
// =============================================================================
//
// Role : Modelise l'anorexie, la boulimie et l'hyperphagie.
//        Modifie la perception de la faim, la relation a la nourriture,
//        et impacte la chimie (cortisol, dopamine, serotonine).
//
// Integration :
//   Modifie le comportement de PrimaryNeeds (faim/soif) dans le pipeline.
//   L'anorexie sous-estime la faim, la boulimie alterne restriction/craving,
//   l'hyperphagie est declenchee par le stress.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Type de trouble alimentaire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EatingDisorderType {
    /// Restriction alimentaire, ignore la faim
    Anorexia,
    /// Cycles restriction → binge → purge
    Bulimia,
    /// Manger compulsivement sous stress
    BingeEating,
}

/// Etat du trouble alimentaire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EatingDisorder {
    /// Type de trouble
    pub disorder_type: EatingDisorderType,
    /// Severite (0.0 = leger, 1.0 = severe)
    pub severity: f64,
    /// Biais de perception de la faim (-1.0 = sous-estime, +1.0 = surestime)
    pub hunger_perception_bias: f64,
    /// Envie compulsive de manger (0.0 = aucune, 1.0 = irresistible)
    pub binge_craving: f64,
    /// Culpabilite apres avoir mange (0.0 = aucune, 1.0 = ecrasante)
    pub guilt_after_eating: f64,
    /// Compteur de cycles depuis le dernier repas
    pub cycles_since_meal: u64,
    /// Compteur de cycles en binge
    binge_cycles: u64,
}

impl EatingDisorder {
    /// Cree un trouble alimentaire.
    pub fn new(disorder_type: EatingDisorderType, severity: f64) -> Self {
        let bias = match &disorder_type {
            EatingDisorderType::Anorexia => -severity,
            EatingDisorderType::BingeEating => severity * 0.5,
            EatingDisorderType::Bulimia => 0.0, // Fluctue
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

    /// Met a jour l'etat a chaque cycle.
    pub fn tick(&mut self, actual_hunger: f64, cortisol: f64) {
        self.cycles_since_meal += 1;

        match self.disorder_type {
            EatingDisorderType::Anorexia => {
                // Ignore la faim → sous-estime progressivement
                self.hunger_perception_bias = (-self.severity * 0.8).clamp(-1.0, 0.0);
                // La culpabilite monte apres avoir mange
                self.guilt_after_eating = (self.guilt_after_eating - 0.01).max(0.0);
            }
            EatingDisorderType::Bulimia => {
                // Cycle : restriction → craving monte → binge → purge → culpabilite
                if self.binge_craving < 0.5 {
                    // Phase restriction : craving monte avec la vraie faim
                    self.binge_craving = (self.binge_craving + actual_hunger * 0.02).min(1.0);
                    self.hunger_perception_bias = -self.severity * 0.5;
                } else {
                    // Phase binge
                    self.binge_cycles += 1;
                    self.hunger_perception_bias = self.severity;
                    if self.binge_cycles > 5 {
                        // Purge / culpabilite
                        self.guilt_after_eating = self.severity * 0.8;
                        self.binge_craving = 0.0;
                        self.binge_cycles = 0;
                    }
                }
            }
            EatingDisorderType::BingeEating => {
                // Stress → envie de manger
                self.binge_craving = (cortisol * self.severity).clamp(0.0, 1.0);
                self.hunger_perception_bias = self.binge_craving * 0.5;
                // Culpabilite si craving eleve
                if self.binge_craving > 0.5 {
                    self.guilt_after_eating = (self.guilt_after_eating + 0.02).min(0.8);
                } else {
                    self.guilt_after_eating = (self.guilt_after_eating - 0.01).max(0.0);
                }
            }
        }
    }

    /// Faim percue (modifiee par le biais du trouble).
    pub fn perceived_hunger(&self, actual_hunger: f64) -> f64 {
        (actual_hunger + self.hunger_perception_bias).clamp(0.0, 1.0)
    }

    /// Impact chimique du trouble.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        // Culpabilite → cortisol
        adj.cortisol += self.guilt_after_eating * 0.02;
        adj.serotonin -= self.guilt_after_eating * 0.01;

        // Craving → dopamine (anticipation)
        adj.dopamine += self.binge_craving * 0.02;

        // Anorexie severe → cortisol chronique
        if self.disorder_type == EatingDisorderType::Anorexia && self.severity > 0.5 {
            adj.cortisol += self.severity * 0.01;
        }

        adj
    }

    /// Serialise pour l'API.
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
        ed.tick(0.3, 0.8); // Cortisol eleve
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
