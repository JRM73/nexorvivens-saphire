// =============================================================================
// conditions/medical.rs — General medical conditions
// =============================================================================
//
// Purpose: Models cancer, HIV/AIDS, autoimmune diseases, and immune
//          deficiencies. Each disease has phases, affects energy,
//          immunity, inflammation, and pain.
//
// Integration:
//   Impacts physiology (immunity, inflammation) and chemistry.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Type of general medical condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MedicalConditionType {
    /// Cancer (stages I-IV, treatment, remission, relapse)
    Cancer,
    /// HIV/AIDS (progressive immune destruction)
    Hiv,
    /// Autoimmune disease (flares and remissions)
    Autoimmune,
    /// Immune deficiency
    ImmuneDeficiency,
}

/// Cancer stage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CancerStage {
    StageI,
    StageII,
    StageIII,
    StageIV,
    Remission,
}

/// Autoimmune disease phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AutoimmunePhase {
    /// Inflammatory flare
    Flare,
    /// Remission
    Remission,
}

/// A general medical condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalCondition {
    pub condition_type: MedicalConditionType,
    /// Overall severity (0.0 = mild, 1.0 = critical)
    pub severity: f64,
    /// Energy drain per cycle (0.0 = none, 0.1 = tiring)
    pub energy_drain: f64,
    /// Pain level (0.0 = none, 1.0 = intense)
    pub pain_level: f64,
    /// Impact on immunity (0.0 = no impact, 1.0 = immunity destroyed)
    pub immune_impact: f64,
    /// Chronic inflammation (0.0 = none, 1.0 = severe)
    pub inflammation: f64,
    /// Under treatment
    pub under_treatment: bool,
    /// Cancer stage (if applicable)
    pub cancer_stage: Option<CancerStage>,
    /// Autoimmune phase (if applicable)
    pub autoimmune_phase: Option<AutoimmunePhase>,
    /// Cycles since onset
    pub cycles_since_onset: u64,
    /// Normalized CD4 count (HIV, 1.0 = normal, 0.0 = AIDS)
    pub cd4_level: f64,
}

impl MedicalCondition {
    pub fn cancer(stage: CancerStage) -> Self {
        let (severity, energy_drain, pain) = match stage {
            CancerStage::StageI => (0.2, 0.01, 0.1),
            CancerStage::StageII => (0.4, 0.02, 0.2),
            CancerStage::StageIII => (0.7, 0.04, 0.4),
            CancerStage::StageIV => (0.95, 0.06, 0.7),
            CancerStage::Remission => (0.1, 0.005, 0.05),
        };
        Self {
            condition_type: MedicalConditionType::Cancer,
            severity,
            energy_drain,
            pain_level: pain,
            immune_impact: severity * 0.3,
            inflammation: severity * 0.4,
            under_treatment: false,
            cancer_stage: Some(stage),
            autoimmune_phase: None,
            cycles_since_onset: 0,
            cd4_level: 1.0,
        }
    }

    pub fn hiv() -> Self {
        Self {
            condition_type: MedicalConditionType::Hiv,
            severity: 0.3,
            energy_drain: 0.01,
            pain_level: 0.05,
            immune_impact: 0.3,
            inflammation: 0.2,
            under_treatment: false,
            cancer_stage: None,
            autoimmune_phase: None,
            cycles_since_onset: 0,
            cd4_level: 0.8,
        }
    }

    pub fn autoimmune() -> Self {
        Self {
            condition_type: MedicalConditionType::Autoimmune,
            severity: 0.4,
            energy_drain: 0.02,
            pain_level: 0.3,
            immune_impact: 0.1,
            inflammation: 0.5,
            under_treatment: false,
            cancer_stage: None,
            autoimmune_phase: Some(AutoimmunePhase::Remission),
            cycles_since_onset: 0,
            cd4_level: 1.0,
        }
    }

    /// Updates the state.
    pub fn tick(&mut self) {
        self.cycles_since_onset += 1;

        match self.condition_type {
            MedicalConditionType::Hiv => {
                // CD4 declines slowly without treatment
                let rate = if self.under_treatment { 0.0001 } else { 0.001 };
                self.cd4_level = (self.cd4_level - rate).max(0.0);
                self.immune_impact = 1.0 - self.cd4_level;
                self.severity = self.immune_impact;

                // AIDS stage if CD4 < 0.2
                if self.cd4_level < 0.2 {
                    self.energy_drain = 0.05;
                    self.pain_level = 0.3;
                }
            }
            MedicalConditionType::Autoimmune => {
                // Alternate between flares and remissions (pseudo-random)
                let cycle_mod = self.cycles_since_onset % 100;
                if cycle_mod < 30 {
                    self.autoimmune_phase = Some(AutoimmunePhase::Flare);
                    self.pain_level = 0.5;
                    self.inflammation = 0.7;
                    self.energy_drain = 0.03;
                } else {
                    self.autoimmune_phase = Some(AutoimmunePhase::Remission);
                    self.pain_level = 0.1;
                    self.inflammation = 0.2;
                    self.energy_drain = 0.01;
                }
            }
            MedicalConditionType::Cancer => {
                // Treatment reduces pain and energy drain
                if self.under_treatment {
                    self.energy_drain = self.severity * 0.08; // Chemo = extreme fatigue
                    self.pain_level = (self.pain_level - 0.001).max(0.05);
                }
            }
            MedicalConditionType::ImmuneDeficiency => {
                self.immune_impact = self.severity;
            }
        }
    }

    /// Chemistry impact.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        // Pain -> cortisol + endorphins
        if self.pain_level > 0.1 {
            adj.cortisol += self.pain_level * 0.02;
            adj.endorphin += self.pain_level * 0.01;
        }

        // Fatigue -> low serotonin
        adj.serotonin -= self.energy_drain * 0.5;

        // Inflammation -> cortisol
        adj.cortisol += self.inflammation * 0.01;

        adj
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": format!("{:?}", self.condition_type),
            "severity": self.severity,
            "energy_drain": self.energy_drain,
            "pain_level": self.pain_level,
            "immune_impact": self.immune_impact,
            "inflammation": self.inflammation,
            "under_treatment": self.under_treatment,
            "cancer_stage": self.cancer_stage.as_ref().map(|s| format!("{:?}", s)),
            "autoimmune_phase": self.autoimmune_phase.as_ref().map(|p| format!("{:?}", p)),
            "cd4_level": self.cd4_level,
        })
    }
}

/// General medical condition manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalManager {
    pub conditions: Vec<MedicalCondition>,
}

impl MedicalManager {
    pub fn new() -> Self {
        Self { conditions: Vec::new() }
    }

    pub fn add(&mut self, condition: MedicalCondition) {
        self.conditions.push(condition);
    }

    pub fn tick(&mut self) {
        for c in &mut self.conditions {
            c.tick();
        }
    }

    /// Cumulative chemistry impact.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();
        for c in &self.conditions {
            let a = c.chemistry_influence();
            adj.dopamine += a.dopamine;
            adj.cortisol += a.cortisol;
            adj.serotonin += a.serotonin;
            adj.endorphin += a.endorphin;
        }
        adj
    }

    /// Total energy drain.
    pub fn total_energy_drain(&self) -> f64 {
        self.conditions.iter().map(|c| c.energy_drain).sum::<f64>().min(0.2)
    }

    /// Total immunity impact.
    pub fn immune_impact(&self) -> f64 {
        self.conditions.iter().map(|c| c.immune_impact).sum::<f64>().min(1.0)
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "conditions": self.conditions.iter().map(|c| c.to_json()).collect::<Vec<_>>(),
            "total_energy_drain": self.total_energy_drain(),
            "immune_impact": self.immune_impact(),
        })
    }
}

impl Default for MedicalManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancer_stages() {
        let c1 = MedicalCondition::cancer(CancerStage::StageI);
        let c4 = MedicalCondition::cancer(CancerStage::StageIV);
        assert!(c4.severity > c1.severity);
        assert!(c4.pain_level > c1.pain_level);
    }

    #[test]
    fn test_hiv_cd4_decline() {
        let mut hiv = MedicalCondition::hiv();
        let initial_cd4 = hiv.cd4_level;
        for _ in 0..200 {
            hiv.tick();
        }
        assert!(hiv.cd4_level < initial_cd4);
    }

    #[test]
    fn test_hiv_treatment_slows() {
        let mut treated = MedicalCondition::hiv();
        treated.under_treatment = true;
        let mut untreated = MedicalCondition::hiv();
        for _ in 0..200 {
            treated.tick();
            untreated.tick();
        }
        assert!(treated.cd4_level > untreated.cd4_level);
    }

    #[test]
    fn test_autoimmune_flares() {
        let mut ai = MedicalCondition::autoimmune();
        // Tick through a cycle to hit a flare
        let mut had_flare = false;
        for _ in 0..100 {
            ai.tick();
            if ai.autoimmune_phase == Some(AutoimmunePhase::Flare) {
                had_flare = true;
            }
        }
        assert!(had_flare);
    }

    #[test]
    fn test_chemistry_pain() {
        let c = MedicalCondition::cancer(CancerStage::StageIII);
        let adj = c.chemistry_influence();
        assert!(adj.cortisol > 0.0);
        assert!(adj.endorphin > 0.0);
    }
}
