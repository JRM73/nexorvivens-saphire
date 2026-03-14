// =============================================================================
// conditions/medical.rs — Maladies generales
// =============================================================================
//
// Role : Modelise cancer, VIH/SIDA, maladies auto-immunes, maladies
//        immunitaires. Chaque maladie a des phases, affecte l'energie,
//        l'immunite, l'inflammation, la douleur.
//
// Integration :
//   Impacte la physiologie (immunite, inflammation) et la chimie.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Type de maladie generale.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MedicalConditionType {
    /// Cancer (stades I-IV, traitement, remission, rechute)
    Cancer,
    /// VIH/SIDA (destruction immunitaire progressive)
    Hiv,
    /// Maladie auto-immune (poussees et remissions)
    Autoimmune,
    /// Deficience immunitaire
    ImmuneDeficiency,
}

/// Phase d'un cancer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CancerStage {
    StageI,
    StageII,
    StageIII,
    StageIV,
    Remission,
}

/// Phase d'une maladie auto-immune.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AutoimmunePhase {
    /// Poussee inflammatoire
    Flare,
    /// Remission
    Remission,
}

/// Une maladie generale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalCondition {
    pub condition_type: MedicalConditionType,
    /// Severite generale (0.0 = leger, 1.0 = critique)
    pub severity: f64,
    /// Drain d'energie par cycle (0.0 = aucun, 0.1 = fatiguant)
    pub energy_drain: f64,
    /// Niveau de douleur (0.0 = aucune, 1.0 = intense)
    pub pain_level: f64,
    /// Impact sur l'immunite (0.0 = pas d'impact, 1.0 = immunite detruite)
    pub immune_impact: f64,
    /// Inflammation chronique (0.0 = aucune, 1.0 = severe)
    pub inflammation: f64,
    /// Sous traitement
    pub under_treatment: bool,
    /// Phase cancer (si applicable)
    pub cancer_stage: Option<CancerStage>,
    /// Phase auto-immune (si applicable)
    pub autoimmune_phase: Option<AutoimmunePhase>,
    /// Cycles depuis le debut
    pub cycles_since_onset: u64,
    /// Compteur CD4 normalise (VIH, 1.0 = normal, 0.0 = SIDA)
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

    /// Met a jour l'etat.
    pub fn tick(&mut self) {
        self.cycles_since_onset += 1;

        match self.condition_type {
            MedicalConditionType::Hiv => {
                // CD4 decroit lentement sans traitement
                let rate = if self.under_treatment { 0.0001 } else { 0.001 };
                self.cd4_level = (self.cd4_level - rate).max(0.0);
                self.immune_impact = 1.0 - self.cd4_level;
                self.severity = self.immune_impact;

                // Stade SIDA si CD4 < 0.2
                if self.cd4_level < 0.2 {
                    self.energy_drain = 0.05;
                    self.pain_level = 0.3;
                }
            }
            MedicalConditionType::Autoimmune => {
                // Alterner poussees et remissions (pseudo-aleatoire)
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
                // Traitement reduit la douleur et l'energie drain
                if self.under_treatment {
                    self.energy_drain = self.severity * 0.08; // Chimio = fatigue extreme
                    self.pain_level = (self.pain_level - 0.001).max(0.05);
                }
            }
            MedicalConditionType::ImmuneDeficiency => {
                self.immune_impact = self.severity;
            }
        }
    }

    /// Impact chimique.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        // Douleur → cortisol + endorphines
        if self.pain_level > 0.1 {
            adj.cortisol += self.pain_level * 0.02;
            adj.endorphin += self.pain_level * 0.01;
        }

        // Fatigue → serotonine basse
        adj.serotonin -= self.energy_drain * 0.5;

        // Inflammation → cortisol
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

/// Gestionnaire de maladies generales.
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

    /// Impact chimique cumule.
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

    /// Drain d'energie total.
    pub fn total_energy_drain(&self) -> f64 {
        self.conditions.iter().map(|c| c.energy_drain).sum::<f64>().min(0.2)
    }

    /// Impact total sur l'immunite.
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
