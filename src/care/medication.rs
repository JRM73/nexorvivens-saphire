// =============================================================================
// care/medication.rs — Therapeutic medications
// =============================================================================
//
// Role: Therapeutic medications. Reuses the same phase system as drugs.rs
//        (pharmacology) but with therapeutic profiles:
//        antidepressants, anxiolytics, painkillers, stimulants, etc.
//
// Difference with drugs.rs:
//   - Medications are prescribed (voluntary, therapeutic)
//   - Explicit side effects
//   - Iatrogenic tolerance and dependence possible
//  - Long treatment duration (weeks/months)
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Medication category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MedicationCategory {
    /// SSRI — selective serotonin reuptake inhibitors
    Antidepressant,
    /// Benzodiazepines — rapid anxiety reduction
    Anxiolytic,
    /// Opioids or NSAIDs — pain relief
    Painkiller,
    /// Methylphenidate, modafinil — focus
    Stimulant,
    /// Antipsychotics — excessive dopamine reduction
    Neuroleptic,
    /// Lithium, valproate — mood cycle smoothing
    MoodStabilizer,
}

/// Possible side effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffects {
    /// Drowsiness (0.0-1.0)
    pub drowsiness: f64,
    /// Appetite loss (0.0-1.0)
    pub appetite_loss: f64,
    /// Emotional blunting (0.0-1.0)
    pub emotional_blunting: f64,
    /// Dependency risk (0.0-1.0)
    pub dependency_risk: f64,
    /// Weight gain (0.0-1.0)
    pub weight_gain: f64,
}

/// A medication currently being administered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medication {
    pub name: String,
    pub category: MedicationCategory,
    /// Chemical impact per cycle (therapeutic effect)
    pub effect: ChemistryAdjustment,
    /// Side effects
    pub side_effects: SideEffects,
    /// Cycles since the start of treatment
    pub cycles_on_medication: u64,
    /// Delay before full therapeutic effect manifests
    pub onset_delay_cycles: u64,
    /// Current efficacy (0.0 = not yet active, 1.0 = full effect)
    pub current_efficacy: f64,
    /// Accumulated tolerance (0.0 = none, 1.0 = ineffective)
    pub tolerance: f64,
    /// Currently being tapered off
    pub tapering: bool,
    /// Relative dose (1.0 = standard dose)
    pub dose: f64,
}

impl Medication {
    /// Updates the medication state at each cycle.
    pub fn tick(&mut self) {
        self.cycles_on_medication += 1;

        // Efficacy ramps up progressively to full effect
        if self.cycles_on_medication < self.onset_delay_cycles {
            self.current_efficacy = self.cycles_on_medication as f64
                / self.onset_delay_cycles as f64;
        } else {
            self.current_efficacy = 1.0;
        }

        // Slow long-term tolerance buildup
        self.tolerance = (self.tolerance + 0.0001).min(0.5);

        // If tapering, reduce dose progressively
        if self.tapering {
            self.dose = (self.dose - 0.005).max(0.0);
        }
    }

    /// Therapeutic chemical impact (modulated by efficacy, tolerance, dose).
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let factor = self.current_efficacy * (1.0 - self.tolerance) * self.dose;
        ChemistryAdjustment {
            dopamine: self.effect.dopamine * factor,
            cortisol: self.effect.cortisol * factor,
            serotonin: self.effect.serotonin * factor,
            adrenaline: self.effect.adrenaline * factor,
            oxytocin: self.effect.oxytocin * factor,
            endorphin: self.effect.endorphin * factor,
            noradrenaline: self.effect.noradrenaline * factor,
        }
    }

    /// Has the medication been completely stopped?
    pub fn is_stopped(&self) -> bool {
        self.tapering && self.dose <= 0.0
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "category": format!("{:?}", self.category),
            "cycles_on": self.cycles_on_medication,
            "efficacy": self.current_efficacy,
            "tolerance": self.tolerance,
            "dose": self.dose,
            "tapering": self.tapering,
            "stopped": self.is_stopped(),
        })
    }
}

/// Medication manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicationManager {
    pub medications: Vec<Medication>,
}

impl MedicationManager {
    pub fn new() -> Self {
        Self { medications: Vec::new() }
    }

    /// Prescribes a medication.
    pub fn prescribe(&mut self, med: Medication) {
        self.medications.push(med);
    }

    /// Updates all medications.
    pub fn tick(&mut self) {
        for med in &mut self.medications {
            med.tick();
        }
        // Remove completely stopped medications
        self.medications.retain(|m| !m.is_stopped());
    }

    /// Total chemical impact of all medications.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();
        for med in &self.medications {
            let a = med.chemistry_influence();
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

    /// Progressively tapers off a medication by name.
    pub fn taper(&mut self, name: &str) {
        for med in &mut self.medications {
            if med.name == name {
                med.tapering = true;
            }
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "medications": self.medications.iter().map(|m| m.to_json()).collect::<Vec<_>>(),
            "count": self.medications.len(),
        })
    }
}

impl Default for MedicationManager {
    fn default() -> Self { Self::new() }
}

// =============================================================================
// Pre-defined medication catalog
// =============================================================================
/// Creates a standard medication from the catalog.
pub fn medication_catalog(name: &str) -> Option<Medication> {
    match name {
        "ssri" | "isrs" => Some(Medication {
            name: "ISRS (antidepresseur)".into(),
            category: MedicationCategory::Antidepressant,
            effect: ChemistryAdjustment {
                serotonin: 0.015,
                ..Default::default()
            },
            side_effects: SideEffects {
                drowsiness: 0.2, appetite_loss: 0.1, emotional_blunting: 0.3,
                dependency_risk: 0.15, weight_gain: 0.2,
            },
            cycles_on_medication: 0,
            onset_delay_cycles: 100, // ~2 weeks before full effect
            current_efficacy: 0.0,
            tolerance: 0.0,
            tapering: false,
            dose: 1.0,
        }),
        "benzodiazepine" | "benzo" => Some(Medication {
            name: "Benzodiazepine (anxiolytique)".into(),
            category: MedicationCategory::Anxiolytic,
            effect: ChemistryAdjustment {
                cortisol: -0.02,
                adrenaline: -0.015,
                serotonin: 0.005,
                ..Default::default()
            },
            side_effects: SideEffects {
                drowsiness: 0.5, appetite_loss: 0.0, emotional_blunting: 0.2,
                dependency_risk: 0.6, weight_gain: 0.0,
            },
            cycles_on_medication: 0,
            onset_delay_cycles: 5, // Fast-acting
            current_efficacy: 0.0,
            tolerance: 0.0,
            tapering: false,
            dose: 1.0,
        }),
        "opioid" | "opioide" => Some(Medication {
            name: "Opioide (antidouleur)".into(),
            category: MedicationCategory::Painkiller,
            effect: ChemistryAdjustment {
                endorphin: 0.03,
                dopamine: 0.01,
                cortisol: -0.01,
                ..Default::default()
            },
            side_effects: SideEffects {
                drowsiness: 0.4, appetite_loss: 0.2, emotional_blunting: 0.3,
                dependency_risk: 0.8, weight_gain: 0.0,
            },
            cycles_on_medication: 0,
            onset_delay_cycles: 3,
            current_efficacy: 0.0,
            tolerance: 0.0,
            tapering: false,
            dose: 1.0,
        }),
        "methylphenidate" | "ritaline" => Some(Medication {
            name: "Methylphenidate (stimulant)".into(),
            category: MedicationCategory::Stimulant,
            effect: ChemistryAdjustment {
                dopamine: 0.02,
                noradrenaline: 0.015,
                ..Default::default()
            },
            side_effects: SideEffects {
                drowsiness: 0.0, appetite_loss: 0.4, emotional_blunting: 0.1,
                dependency_risk: 0.3, weight_gain: 0.0,
            },
            cycles_on_medication: 0,
            onset_delay_cycles: 5,
            current_efficacy: 0.0,
            tolerance: 0.0,
            tapering: false,
            dose: 1.0,
        }),
        "neuroleptic" | "antipsychotique" => Some(Medication {
            name: "Antipsychotique (neuroleptique)".into(),
            category: MedicationCategory::Neuroleptic,
            effect: ChemistryAdjustment {
                dopamine: -0.02,
                ..Default::default()
            },
            side_effects: SideEffects {
                drowsiness: 0.4, appetite_loss: 0.0, emotional_blunting: 0.5,
                dependency_risk: 0.1, weight_gain: 0.5,
            },
            cycles_on_medication: 0,
            onset_delay_cycles: 30,
            current_efficacy: 0.0,
            tolerance: 0.0,
            tapering: false,
            dose: 1.0,
        }),
        "lithium" => Some(Medication {
            name: "Lithium (stabilisateur humeur)".into(),
            category: MedicationCategory::MoodStabilizer,
            effect: ChemistryAdjustment {
                // Smooths extremes: attenuates dopamine and cortisol
                dopamine: -0.005,
                cortisol: -0.005,
                serotonin: 0.005,
                ..Default::default()
            },
            side_effects: SideEffects {
                drowsiness: 0.2, appetite_loss: 0.1, emotional_blunting: 0.3,
                dependency_risk: 0.1, weight_gain: 0.3,
            },
            cycles_on_medication: 0,
            onset_delay_cycles: 70, // Slow-acting
            current_efficacy: 0.0,
            tolerance: 0.0,
            tapering: false,
            dose: 1.0,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onset_delay() {
        let mut med = medication_catalog("ssri").unwrap();
        assert!(med.current_efficacy < 0.01);
        for _ in 0..50 {
            med.tick();
        }
        // Halfway through the 100-cycle delay
        assert!(med.current_efficacy > 0.4);
        assert!(med.current_efficacy < 0.6);
    }

    #[test]
    fn test_full_efficacy() {
        let mut med = medication_catalog("benzo").unwrap();
        for _ in 0..10 {
            med.tick();
        }
        assert!((med.current_efficacy - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_tapering() {
        let mut med = medication_catalog("ssri").unwrap();
        for _ in 0..150 {
            med.tick();
        }
        med.tapering = true;
        for _ in 0..300 {
            med.tick();
        }
        assert!(med.is_stopped());
    }

    #[test]
    fn test_chemistry_modulated_by_dose() {
        let mut med = medication_catalog("ssri").unwrap();
        // Simulate full effect
        med.current_efficacy = 1.0;
        med.dose = 1.0;
        let adj_full = med.chemistry_influence();

        med.dose = 0.5;
        let adj_half = med.chemistry_influence();
        assert!(adj_full.serotonin > adj_half.serotonin);
    }

    #[test]
    fn test_manager_cleanup() {
        let mut mgr = MedicationManager::new();
        let mut med = medication_catalog("benzo").unwrap();
        med.tapering = true;
        med.dose = 0.01;
        mgr.prescribe(med);
        for _ in 0..10 {
            mgr.tick();
        }
        assert_eq!(mgr.medications.len(), 0);
    }

    #[test]
    fn test_catalog_unknown() {
        assert!(medication_catalog("unobtainium").is_none());
    }
}
