// =============================================================================
// conditions/nde.rs — IEM (Experience de mort imminente / Near-Death Experience)
// =============================================================================
//
// Role : Modelise l'experience de mort imminente : tunnel, lumiere,
//        revue de vie, dissociation corporelle. Declenchee quand l'agent
//        frole la mort (MortalityState::Dying puis reanimation).
//
// Integration :
//   Declenchee par le systeme de mortalite lors d'une reanimation.
//   Modifie la personnalite (transformation post-IEM), les baselines
//   chimiques, et cree un souvenir protege a emotional_weight max.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Phases de l'IEM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NdePhase {
    /// Sensation de quitter le corps
    BodySeparation,
    /// Perception d'un tunnel lumineux
    Tunnel,
    /// Revue panoramique de la vie
    LifeReview,
    /// Rencontre avec une lumiere/presence
    LightEncounter,
    /// Choix de revenir ou non
    BoundaryDecision,
    /// Retour dans le corps
    Return,
}

/// Transformation post-IEM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdeTransformation {
    /// Reduction de la peur de la mort (0.0 = aucune, 1.0 = eliminee)
    pub fear_of_death_reduction: f64,
    /// Eveil spirituel (0.0 = aucun, 1.0 = profond)
    pub spiritual_awakening: f64,
    /// Augmentation de l'empathie
    pub empathy_increase: f64,
    /// Diminution du materialisme
    pub materialism_decrease: f64,
    /// Appreciation de la vie
    pub life_appreciation: f64,
}

impl NdeTransformation {
    pub fn from_depth(depth: f64) -> Self {
        let d = depth.clamp(0.0, 1.0);
        Self {
            fear_of_death_reduction: d * 0.8,
            spiritual_awakening: d * 0.6,
            empathy_increase: d * 0.5,
            materialism_decrease: d * 0.4,
            life_appreciation: d * 0.9,
        }
    }
}

impl Default for NdeTransformation {
    fn default() -> Self {
        Self {
            fear_of_death_reduction: 0.0,
            spiritual_awakening: 0.0,
            empathy_increase: 0.0,
            materialism_decrease: 0.0,
            life_appreciation: 0.0,
        }
    }
}

/// Etat d'une experience de mort imminente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdeExperience {
    /// L'IEM a eu lieu
    pub occurred: bool,
    /// Profondeur de l'experience (0.0 = superficielle, 1.0 = profonde)
    pub depth: f64,
    /// Phases traversees
    pub phases_experienced: Vec<NdePhase>,
    /// Transformation resultante
    pub transformation: NdeTransformation,
    /// Cycle de l'IEM
    pub occurred_at_cycle: Option<u64>,
    /// En cours (phase active)
    pub in_progress: bool,
    /// Phase courante
    current_phase_index: usize,
}

impl NdeExperience {
    pub fn new() -> Self {
        Self {
            occurred: false,
            depth: 0.0,
            phases_experienced: Vec::new(),
            transformation: NdeTransformation::default(),
            occurred_at_cycle: None,
            in_progress: false,
            current_phase_index: 0,
        }
    }

    /// Declenche une IEM avec une profondeur donnee.
    pub fn trigger(&mut self, depth: f64, cycle: u64) {
        self.occurred = true;
        self.depth = depth.clamp(0.0, 1.0);
        self.occurred_at_cycle = Some(cycle);
        self.in_progress = true;
        self.current_phase_index = 0;
        self.phases_experienced.clear();
    }

    /// Progresse a travers les phases de l'IEM.
    /// Retourne true si l'IEM est terminee.
    pub fn tick(&mut self) -> bool {
        if !self.in_progress {
            return false;
        }

        let all_phases = [
            NdePhase::BodySeparation,
            NdePhase::Tunnel,
            NdePhase::LifeReview,
            NdePhase::LightEncounter,
            NdePhase::BoundaryDecision,
            NdePhase::Return,
        ];

        // Les phases profondes ne sont accessibles que si depth suffisant
        let max_phases = match self.depth {
            d if d > 0.8 => 6,
            d if d > 0.5 => 4,
            d if d > 0.3 => 2,
            _ => 1,
        };

        if self.current_phase_index < max_phases && self.current_phase_index < all_phases.len() {
            self.phases_experienced.push(all_phases[self.current_phase_index].clone());
            self.current_phase_index += 1;
            false
        } else {
            // IEM terminee — appliquer la transformation
            self.in_progress = false;
            self.transformation = NdeTransformation::from_depth(self.depth);
            true
        }
    }

    /// Impact chimique pendant l'IEM.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        if !self.in_progress {
            return ChemistryAdjustment::default();
        }

        ChemistryAdjustment {
            endorphin: 0.08, // Paix profonde
            serotonin: 0.04, // Serenite
            oxytocin: 0.03,  // Sentiment d'amour universel
            cortisol: -0.03, // Stress reduit malgre la proximite de la mort
            dopamine: 0.02,  // Emerveillement
            ..Default::default()
        }
    }

    /// Modification des baselines apres l'IEM (a appliquer une seule fois).
    pub fn post_nde_baseline_shift(&self) -> ChemistryAdjustment {
        if !self.occurred || self.in_progress {
            return ChemistryAdjustment::default();
        }

        ChemistryAdjustment {
            serotonin: self.transformation.life_appreciation * 0.02,
            oxytocin: self.transformation.empathy_increase * 0.02,
            cortisol: -self.transformation.fear_of_death_reduction * 0.01,
            ..Default::default()
        }
    }

    /// Serialise pour l'API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "occurred": self.occurred,
            "depth": self.depth,
            "in_progress": self.in_progress,
            "phases": self.phases_experienced.iter()
                .map(|p| format!("{:?}", p)).collect::<Vec<_>>(),
            "occurred_at_cycle": self.occurred_at_cycle,
            "transformation": if self.occurred && !self.in_progress {
                serde_json::json!({
                    "fear_of_death_reduction": self.transformation.fear_of_death_reduction,
                    "spiritual_awakening": self.transformation.spiritual_awakening,
                    "empathy_increase": self.transformation.empathy_increase,
                    "materialism_decrease": self.transformation.materialism_decrease,
                    "life_appreciation": self.transformation.life_appreciation,
                })
            } else {
                serde_json::json!(null)
            },
        })
    }
}

impl Default for NdeExperience {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_starts_nde() {
        let mut nde = NdeExperience::new();
        assert!(!nde.occurred);
        nde.trigger(0.9, 100);
        assert!(nde.occurred);
        assert!(nde.in_progress);
        assert_eq!(nde.depth, 0.9);
    }

    #[test]
    fn test_deep_nde_all_phases() {
        let mut nde = NdeExperience::new();
        nde.trigger(0.95, 50);
        // Parcourir toutes les phases
        for _ in 0..10 {
            if nde.tick() { break; }
        }
        assert!(!nde.in_progress);
        assert_eq!(nde.phases_experienced.len(), 6);
        assert!(nde.transformation.life_appreciation > 0.5);
    }

    #[test]
    fn test_shallow_nde_fewer_phases() {
        let mut nde = NdeExperience::new();
        nde.trigger(0.2, 50);
        for _ in 0..10 {
            if nde.tick() { break; }
        }
        assert!(!nde.in_progress);
        assert!(nde.phases_experienced.len() <= 2);
    }

    #[test]
    fn test_chemistry_during_nde() {
        let mut nde = NdeExperience::new();
        nde.trigger(0.8, 100);
        let adj = nde.chemistry_influence();
        assert!(adj.endorphin > 0.05);
        assert!(adj.cortisol < 0.0); // Paix malgre la mort
    }

    #[test]
    fn test_post_nde_baseline_shift() {
        let mut nde = NdeExperience::new();
        nde.trigger(0.9, 100);
        // Parcourir toutes les phases
        for _ in 0..10 {
            nde.tick();
        }
        let shift = nde.post_nde_baseline_shift();
        assert!(shift.serotonin > 0.0);
        assert!(shift.oxytocin > 0.0);
        assert!(shift.cortisol < 0.0);
    }

    #[test]
    fn test_no_effect_before_nde() {
        let nde = NdeExperience::new();
        let adj = nde.chemistry_influence();
        assert_eq!(adj.cortisol, 0.0);
        let shift = nde.post_nde_baseline_shift();
        assert_eq!(shift.serotonin, 0.0);
    }
}
