// =============================================================================
// conditions/extreme.rs — Conditions extremes
// =============================================================================
//
// Role : Modelise les conditions de travail extreme : secouriste, militaire,
//        plongeur, astronaute. Stress chronique, adaptation physiologique,
//        resilience ou epuisement (burnout).
//
// Integration :
//   Modifie les baselines de cortisol/adrenaline, la physiologie (SpO2,
//   temperature), et peut declencher des traumas ou renforcer la resilience.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Type de condition extreme.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtremeConditionType {
    /// Confrontation a la mort, stress aigu repete
    Rescuer,
    /// Combat, hypervigilance, bruit
    Military,
    /// Pression, narcose, froid
    DeepSeaDiver,
    /// Isolation, gravite zero, radiation
    Astronaut,
}

/// Etat d'une condition extreme active.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtremeCondition {
    pub condition_type: ExtremeConditionType,
    /// Cycles depuis le debut de la condition
    pub duration_cycles: u64,
    /// Stress cumule (0.0 = frais, 1.0 = sature)
    pub stress_accumulation: f64,
    /// Adaptation physiologique (0.0 = non adapte, 1.0 = adapte)
    pub physiological_adaptation: f64,
    /// Resilience psychologique (0.0 = fragile, 1.0 = resilient)
    pub psychological_resilience: f64,
    /// Risque de burnout (0.0 = aucun, 1.0 = imminent)
    pub burnout_risk: f64,
}

impl ExtremeCondition {
    pub fn new(condition_type: ExtremeConditionType) -> Self {
        Self {
            condition_type,
            duration_cycles: 0,
            stress_accumulation: 0.0,
            physiological_adaptation: 0.0,
            psychological_resilience: 0.3,
            burnout_risk: 0.0,
        }
    }

    /// Met a jour a chaque cycle.
    pub fn tick(&mut self, cortisol: f64) {
        self.duration_cycles += 1;

        // Stress s'accumule, surtout si cortisol est eleve
        let stress_rate = match self.condition_type {
            ExtremeConditionType::Military => 0.003,
            ExtremeConditionType::Rescuer => 0.002,
            ExtremeConditionType::DeepSeaDiver => 0.002,
            ExtremeConditionType::Astronaut => 0.001,
        };
        self.stress_accumulation = (self.stress_accumulation + stress_rate * (1.0 + cortisol)).min(1.0);

        // Adaptation physiologique croît
        let adapt_rate = 0.001;
        self.physiological_adaptation = (self.physiological_adaptation + adapt_rate).min(1.0);

        // Resilience croît si stress pas trop eleve, decroit sinon
        if self.stress_accumulation < 0.7 {
            self.psychological_resilience = (self.psychological_resilience + 0.0005).min(1.0);
        } else {
            self.psychological_resilience = (self.psychological_resilience - 0.001).max(0.0);
        }

        // Burnout = stress eleve + resilience faible
        self.burnout_risk = ((self.stress_accumulation - self.psychological_resilience * 0.5) * 0.8)
            .clamp(0.0, 1.0);
    }

    /// Impact chimique de la condition.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        // Cortisol de base eleve en conditions extremes
        adj.cortisol += self.stress_accumulation * 0.02;

        // Adrenaline chronique (militaire, secouriste)
        match self.condition_type {
            ExtremeConditionType::Military | ExtremeConditionType::Rescuer => {
                adj.adrenaline += 0.01;
                adj.noradrenaline += 0.01;
            }
            _ => {}
        }

        // Endorphines comme mecanisme de coping
        adj.endorphin += self.psychological_resilience * 0.005;

        // Burnout → serotonine chute
        if self.burnout_risk > 0.5 {
            adj.serotonin -= self.burnout_risk * 0.02;
        }

        adj
    }

    /// Impact sur la physiologie (retourne des offsets).
    /// (spo2_offset, temperature_offset, bp_systolic_offset)
    pub fn physiology_impact(&self) -> (f64, f64, f64) {
        match self.condition_type {
            ExtremeConditionType::Astronaut => {
                // SpO2 reduit, temperature legèrement basse
                let spo2_penalty = -3.0 * (1.0 - self.physiological_adaptation * 0.5);
                (-spo2_penalty.abs(), -0.3, 0.0)
            }
            ExtremeConditionType::DeepSeaDiver => {
                // Pression → SpO2 fluctue, froid
                let depth_factor = 1.0 - self.physiological_adaptation * 0.5;
                (-2.0 * depth_factor, -0.5 * depth_factor, 10.0 * depth_factor)
            }
            ExtremeConditionType::Military => {
                // Stress → pression elevee
                (0.0, 0.0, self.stress_accumulation * 15.0)
            }
            ExtremeConditionType::Rescuer => {
                (0.0, 0.0, self.stress_accumulation * 10.0)
            }
        }
    }

    /// Serialise pour l'API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": format!("{:?}", self.condition_type),
            "duration_cycles": self.duration_cycles,
            "stress_accumulation": self.stress_accumulation,
            "physiological_adaptation": self.physiological_adaptation,
            "psychological_resilience": self.psychological_resilience,
            "burnout_risk": self.burnout_risk,
        })
    }
}

/// Gestionnaire de conditions extremes (une seule active a la fois).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtremeConditionManager {
    pub active: Option<ExtremeCondition>,
}

impl ExtremeConditionManager {
    pub fn new() -> Self {
        Self { active: None }
    }

    pub fn activate(&mut self, condition_type: ExtremeConditionType) {
        self.active = Some(ExtremeCondition::new(condition_type));
    }

    pub fn deactivate(&mut self) {
        self.active = None;
    }

    pub fn tick(&mut self, cortisol: f64) {
        if let Some(ref mut c) = self.active {
            c.tick(cortisol);
        }
    }

    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        self.active.as_ref()
            .map(|c| c.chemistry_influence())
            .unwrap_or_default()
    }

    pub fn to_json(&self) -> serde_json::Value {
        match &self.active {
            Some(c) => c.to_json(),
            None => serde_json::json!({ "active": false }),
        }
    }
}

impl Default for ExtremeConditionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stress_accumulates() {
        let mut ec = ExtremeCondition::new(ExtremeConditionType::Military);
        for _ in 0..100 {
            ec.tick(0.5);
        }
        assert!(ec.stress_accumulation > 0.0);
        assert!(ec.physiological_adaptation > 0.0);
    }

    #[test]
    fn test_burnout_risk() {
        let mut ec = ExtremeCondition::new(ExtremeConditionType::Military);
        ec.stress_accumulation = 0.9;
        ec.psychological_resilience = 0.1;
        ec.tick(0.8);
        assert!(ec.burnout_risk > 0.3);
    }

    #[test]
    fn test_astronaut_physiology_impact() {
        let ec = ExtremeCondition::new(ExtremeConditionType::Astronaut);
        let (spo2, temp, _bp) = ec.physiology_impact();
        assert!(spo2 < 0.0); // SpO2 reduit
        assert!(temp < 0.0);  // Temperature basse
    }

    #[test]
    fn test_manager_activate_deactivate() {
        let mut mgr = ExtremeConditionManager::new();
        assert!(mgr.active.is_none());
        mgr.activate(ExtremeConditionType::Rescuer);
        assert!(mgr.active.is_some());
        mgr.deactivate();
        assert!(mgr.active.is_none());
    }
}
