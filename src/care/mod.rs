// =============================================================================
// care/mod.rs — Saphire's care system
// =============================================================================
//
// Role: Groups all healing and care mechanisms:
//        psychological therapy, medications, surgery, art therapy.
//        The CareSystem orchestrates these 4 aspects and produces a global
//        chemical impact that accelerates condition healing (P2.2/P2.3/P2.7/P2.10).
//
// Place in architecture:
//   Consulted by the cognitive pipeline. Care modifies chemistry,
//   accelerates trauma processing and phobia desensitization,
//   and can generate side effects.
// =============================================================================

pub mod therapy;
pub mod medication;
pub mod surgery;
pub mod art_therapy;

pub use therapy::{TherapyManager, TherapyType};
pub use medication::{MedicationManager, Medication, medication_catalog, MedicationCategory};
pub use surgery::{SurgeryManager, Surgery};
pub use art_therapy::{ArtTherapyManager, ArtForm};

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Forced rest state (bed rest, activity stop).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestState {
    /// Currently in forced rest
    pub resting: bool,
    /// Accumulated rest cycles
    pub cycles_resting: u64,
    /// Healing boost (increases with rest)
    pub healing_boost: f64,
}

impl RestState {
    pub fn new() -> Self {
        Self {
            resting: false,
            cycles_resting: 0,
            healing_boost: 0.0,
        }
    }

    pub fn start_rest(&mut self) {
        self.resting = true;
    }

    pub fn stop_rest(&mut self) {
        self.resting = false;
        self.cycles_resting = 0;
        self.healing_boost = 0.0;
    }

    pub fn tick(&mut self) {
        if self.resting {
            self.cycles_resting += 1;
            // Boost increases with rest (asymptote at 0.5)
            self.healing_boost = (1.0 - (-0.005 * self.cycles_resting as f64).exp()) * 0.5;
        }
    }

    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        if !self.resting {
            return ChemistryAdjustment::default();
        }
        ChemistryAdjustment {
            cortisol: -0.005,
            serotonin: 0.003,
            ..Default::default()
        }
    }
}

impl Default for RestState {
    fn default() -> Self { Self::new() }
}

/// Saphire's complete care system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CareSystem {
    pub therapy: TherapyManager,
    pub medication: MedicationManager,
    pub surgery: SurgeryManager,
    pub art_therapy: ArtTherapyManager,
    pub rest: RestState,
}

impl CareSystem {
    pub fn new() -> Self {
        Self {
            therapy: TherapyManager::new(),
            medication: MedicationManager::new(),
            surgery: SurgeryManager::new(),
            art_therapy: ArtTherapyManager::new(),
            rest: RestState::new(),
        }
    }

    /// Updates all care components.
    /// Returns healing bonuses per targeted condition.
    pub fn tick(&mut self) -> Vec<(String, f64)> {
        let therapy_bonuses = self.therapy.tick();
        self.medication.tick();
        self.surgery.tick();
        self.art_therapy.tick();
        self.rest.tick();
        therapy_bonuses
    }

    /// Total chemical impact of all active care.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();
        let sources = [
            self.therapy.chemistry_influence(),
            self.medication.chemistry_influence(),
            self.surgery.chemistry_influence(),
            self.art_therapy.chemistry_influence(),
            self.rest.chemistry_influence(),
        ];
        for a in &sources {
            adj.dopamine += a.dopamine;
            adj.cortisol += a.cortisol;
            adj.serotonin += a.serotonin;
            adj.adrenaline += a.adrenaline;
            adj.oxytocin += a.oxytocin;
            adj.endorphin += a.endorphin;
            adj.noradrenaline += a.noradrenaline;
        }
        adj
    }

    /// Total pain from care (mainly surgery).
    pub fn total_pain(&self) -> f64 {
        self.surgery.total_pain()
    }

    /// Is at least one care component active?
    pub fn has_active_care(&self) -> bool {
        !self.therapy.active_therapies.is_empty()
            || !self.medication.medications.is_empty()
            || !self.surgery.surgeries.is_empty()
            || self.art_therapy.practices.iter().any(|p| p.is_active())
            || self.rest.resting
    }

    /// Serializes for the API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "therapy": self.therapy.to_json(),
            "medication": self.medication.to_json(),
            "surgery": self.surgery.to_json(),
            "art_therapy": self.art_therapy.to_json(),
            "rest": {
                "resting": self.rest.resting,
                "cycles": self.rest.cycles_resting,
                "healing_boost": self.rest.healing_boost,
            },
            "has_active_care": self.has_active_care(),
            "total_pain": self.total_pain(),
        })
    }
}

impl Default for CareSystem {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_care_system_empty() {
        let care = CareSystem::new();
        assert!(!care.has_active_care());
        let adj = care.chemistry_influence();
        assert!((adj.dopamine).abs() < 0.001);
    }

    #[test]
    fn test_care_system_with_therapy() {
        let mut care = CareSystem::new();
        care.therapy.start(TherapyType::Cbt, "phobia:test");
        assert!(care.has_active_care());
        let bonuses = care.tick();
        // First cycle = very small but present bonus
        assert!(!bonuses.is_empty() || care.therapy.active_therapies.is_empty());
    }

    #[test]
    fn test_care_system_chemistry() {
        let mut care = CareSystem::new();
        care.therapy.start(TherapyType::Emdr, "trauma:test");
        care.rest.start_rest();
        let adj = care.chemistry_influence();
        // EMDR start + rest -> cortisol should be modified
        assert!(adj.cortisol != 0.0 || adj.serotonin != 0.0);
    }

    #[test]
    fn test_rest_healing_boost() {
        let mut rest = RestState::new();
        rest.start_rest();
        for _ in 0..200 {
            rest.tick();
        }
        assert!(rest.healing_boost > 0.3);
    }
}
