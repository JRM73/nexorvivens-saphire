// =============================================================================
// conditions/nde.rs — NDE (Near-Death Experience)
// =============================================================================
//
// Purpose: Models the near-death experience: tunnel, light,
//          life review, out-of-body dissociation. Triggered when the agent
//          comes close to death (MortalityState::Dying then resuscitation).
//
// Integration:
//   Triggered by the mortality system during resuscitation.
//   Modifies personality (post-NDE transformation), chemistry
//   baselines, and creates a protected memory at maximum emotional_weight.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// NDE phases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NdePhase {
    /// Sensation of leaving the body
    BodySeparation,
    /// Perception of a luminous tunnel
    Tunnel,
    /// Panoramic life review
    LifeReview,
    /// Encounter with a light/presence
    LightEncounter,
    /// Choice to return or not
    BoundaryDecision,
    /// Return to the body
    Return,
}

/// Post-NDE transformation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdeTransformation {
    /// Reduction of fear of death (0.0 = none, 1.0 = eliminated)
    pub fear_of_death_reduction: f64,
    /// Spiritual awakening (0.0 = none, 1.0 = profound)
    pub spiritual_awakening: f64,
    /// Empathy increase
    pub empathy_increase: f64,
    /// Materialism decrease
    pub materialism_decrease: f64,
    /// Life appreciation
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

/// State of a near-death experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdeExperience {
    /// The NDE has occurred
    pub occurred: bool,
    /// Depth of the experience (0.0 = shallow, 1.0 = profound)
    pub depth: f64,
    /// Phases traversed
    pub phases_experienced: Vec<NdePhase>,
    /// Resulting transformation
    pub transformation: NdeTransformation,
    /// Cycle of the NDE
    pub occurred_at_cycle: Option<u64>,
    /// In progress (active phase)
    pub in_progress: bool,
    /// Current phase
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

    /// Triggers an NDE with a given depth.
    pub fn trigger(&mut self, depth: f64, cycle: u64) {
        self.occurred = true;
        self.depth = depth.clamp(0.0, 1.0);
        self.occurred_at_cycle = Some(cycle);
        self.in_progress = true;
        self.current_phase_index = 0;
        self.phases_experienced.clear();
    }

    /// Progresses through the NDE phases.
    /// Returns true if the NDE is complete.
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

        // Deeper phases are only accessible with sufficient depth
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
            // NDE complete — apply transformation
            self.in_progress = false;
            self.transformation = NdeTransformation::from_depth(self.depth);
            true
        }
    }

    /// Chemistry impact during the NDE.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        if !self.in_progress {
            return ChemistryAdjustment::default();
        }

        ChemistryAdjustment {
            endorphin: 0.08, // Deep peace
            serotonin: 0.04, // Serenity
            oxytocin: 0.03,  // Feeling of universal love
            cortisol: -0.03, // Reduced stress despite proximity to death
            dopamine: 0.02,  // Wonder
            ..Default::default()
        }
    }

    /// Baseline modification after the NDE (apply only once).
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

    /// Serializes for the API.
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
        // Traverse all phases
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
        assert!(adj.cortisol < 0.0); // Peace despite death
    }

    #[test]
    fn test_post_nde_baseline_shift() {
        let mut nde = NdeExperience::new();
        nde.trigger(0.9, 100);
        // Traverse all phases
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
