// =============================================================================
// conditions/degenerative.rs — Degenerative diseases
// =============================================================================
//
// Purpose: Models Alzheimer's, Parkinson's, epilepsy, dementia, and major
//          depression. Each disease degrades specific abilities over time
//          and impacts chemistry.
//
// Integration:
//   Cognitive effects (memory, speed, motivation) are applied
//   in the pipeline via CognitiveEffects.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Type of degenerative disease.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DegenerativeType {
    /// Progressive memory and recognition loss
    Alzheimer,
    /// Motor slowness, tremors, rigidity
    Parkinson,
    /// Episodic seizures, post-seizure confusion
    Epilepsy,
    /// Degraded judgment, reasoning, and orientation
    Dementia,
    /// Anhedonia, loss of motivation, low energy
    MajorDepression,
}

/// Cumulative cognitive effects of diseases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveEffects {
    /// Memory retention multiplier (1.0 = normal, 0.0 = none)
    pub memory_retention: f64,
    /// Response speed multiplier (1.0 = normal)
    pub response_speed: f64,
    /// Reasoning clarity (1.0 = clear, 0.0 = confused)
    pub reasoning_clarity: f64,
    /// Motivation (1.0 = motivated, 0.0 = apathetic)
    pub motivation: f64,
    /// Seizure risk (0.0 = none, 1.0 = imminent)
    pub seizure_risk: f64,
}

impl Default for CognitiveEffects {
    fn default() -> Self {
        Self {
            memory_retention: 1.0,
            response_speed: 1.0,
            reasoning_clarity: 1.0,
            motivation: 1.0,
            seizure_risk: 0.0,
        }
    }
}

/// An individual degenerative condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegenerativeCondition {
    pub disease_type: DegenerativeType,
    /// Progression (0.0 = onset, 1.0 = terminal)
    pub progression: f64,
    /// Progression rate per cycle
    pub progression_rate: f64,
    /// Under treatment (slows progression)
    pub under_treatment: bool,
    /// Cycles since onset
    pub cycles_since_onset: u64,
    /// In crisis (epilepsy)
    pub in_crisis: bool,
    /// Crisis counter (epilepsy)
    pub crisis_count: u64,
}

impl DegenerativeCondition {
    pub fn new(disease_type: DegenerativeType, progression_rate: f64) -> Self {
        Self {
            disease_type,
            progression: 0.0,
            progression_rate,
            under_treatment: false,
            cycles_since_onset: 0,
            in_crisis: false,
            crisis_count: 0,
        }
    }

    /// Updates the progression.
    pub fn tick(&mut self) {
        self.cycles_since_onset += 1;

        let rate = if self.under_treatment {
            self.progression_rate * 0.3 // Treatment slows progression
        } else {
            self.progression_rate
        };

        self.progression = (self.progression + rate).min(1.0);

        // Epilepsy: random seizures based on progression
        if self.disease_type == DegenerativeType::Epilepsy {
            // Seizure if progression * hash_cycle gives a critical result
            let crisis_chance = self.progression * 0.02;
            let pseudo_random = ((self.cycles_since_onset * 17 + 31) % 100) as f64 / 100.0;
            self.in_crisis = pseudo_random < crisis_chance;
            if self.in_crisis {
                self.crisis_count += 1;
            }
        } else {
            self.in_crisis = false;
        }
    }

    /// Cognitive effects of this disease.
    pub fn cognitive_effects(&self) -> CognitiveEffects {
        let p = self.progression;
        match self.disease_type {
            DegenerativeType::Alzheimer => CognitiveEffects {
                memory_retention: 1.0 - p * 0.8,
                reasoning_clarity: 1.0 - p * 0.5,
                ..Default::default()
            },
            DegenerativeType::Parkinson => CognitiveEffects {
                response_speed: 1.0 - p * 0.6,
                ..Default::default()
            },
            DegenerativeType::Epilepsy => CognitiveEffects {
                seizure_risk: p * 0.5,
                reasoning_clarity: if self.in_crisis { 0.1 } else { 1.0 - p * 0.2 },
                ..Default::default()
            },
            DegenerativeType::Dementia => CognitiveEffects {
                reasoning_clarity: 1.0 - p * 0.7,
                memory_retention: 1.0 - p * 0.5,
                ..Default::default()
            },
            DegenerativeType::MajorDepression => CognitiveEffects {
                motivation: 1.0 - p * 0.8,
                response_speed: 1.0 - p * 0.3,
                ..Default::default()
            },
        }
    }

    /// Chemistry impact.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let p = self.progression;
        let mut adj = ChemistryAdjustment::default();

        match self.disease_type {
            DegenerativeType::Alzheimer | DegenerativeType::Dementia => {
                adj.cortisol += p * 0.01;
                adj.serotonin -= p * 0.01;
            }
            DegenerativeType::Parkinson => {
                adj.dopamine -= p * 0.02; // Dopaminergic deficit
            }
            DegenerativeType::Epilepsy => {
                if self.in_crisis {
                    adj.adrenaline += 0.05;
                    adj.cortisol += 0.04;
                    adj.noradrenaline += 0.03;
                }
            }
            DegenerativeType::MajorDepression => {
                adj.serotonin -= p * 0.03;
                adj.dopamine -= p * 0.02;
                adj.cortisol += p * 0.02;
                adj.noradrenaline -= p * 0.01;
            }
        }

        adj
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": format!("{:?}", self.disease_type),
            "progression": self.progression,
            "under_treatment": self.under_treatment,
            "cycles_since_onset": self.cycles_since_onset,
            "in_crisis": self.in_crisis,
            "crisis_count": self.crisis_count,
        })
    }
}

/// Degenerative disease manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegenerativeManager {
    pub conditions: Vec<DegenerativeCondition>,
}

impl DegenerativeManager {
    pub fn new() -> Self {
        Self { conditions: Vec::new() }
    }

    pub fn add(&mut self, condition: DegenerativeCondition) {
        self.conditions.push(condition);
    }

    pub fn tick(&mut self) {
        for c in &mut self.conditions {
            c.tick();
        }
    }

    /// Cumulative cognitive effects (multiplicative).
    pub fn cumulative_effects(&self) -> CognitiveEffects {
        let mut effects = CognitiveEffects::default();
        for c in &self.conditions {
            let e = c.cognitive_effects();
            effects.memory_retention *= e.memory_retention;
            effects.response_speed *= e.response_speed;
            effects.reasoning_clarity *= e.reasoning_clarity;
            effects.motivation *= e.motivation;
            effects.seizure_risk = (effects.seizure_risk + e.seizure_risk).min(1.0);
        }
        effects
    }

    /// Cumulative chemistry impact.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();
        for c in &self.conditions {
            let a = c.chemistry_influence();
            adj.dopamine += a.dopamine;
            adj.cortisol += a.cortisol;
            adj.serotonin += a.serotonin;
            adj.adrenaline += a.adrenaline;
            adj.noradrenaline += a.noradrenaline;
            adj.endorphin += a.endorphin;
        }
        adj
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "conditions": self.conditions.iter().map(|c| c.to_json()).collect::<Vec<_>>(),
            "count": self.conditions.len(),
        })
    }
}

impl Default for DegenerativeManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alzheimer_memory_loss() {
        let mut c = DegenerativeCondition::new(DegenerativeType::Alzheimer, 0.01);
        for _ in 0..50 {
            c.tick();
        }
        let effects = c.cognitive_effects();
        assert!(effects.memory_retention < 0.8);
    }

    #[test]
    fn test_depression_low_motivation() {
        let mut c = DegenerativeCondition::new(DegenerativeType::MajorDepression, 0.02);
        for _ in 0..30 {
            c.tick();
        }
        let effects = c.cognitive_effects();
        assert!(effects.motivation < 0.6);
        let adj = c.chemistry_influence();
        assert!(adj.serotonin < 0.0);
    }

    #[test]
    fn test_treatment_slows_progression() {
        let mut treated = DegenerativeCondition::new(DegenerativeType::Parkinson, 0.01);
        treated.under_treatment = true;
        let mut untreated = DegenerativeCondition::new(DegenerativeType::Parkinson, 0.01);
        for _ in 0..100 {
            treated.tick();
            untreated.tick();
        }
        assert!(treated.progression < untreated.progression);
    }

    #[test]
    fn test_cumulative_effects() {
        let mut mgr = DegenerativeManager::new();
        let mut alz = DegenerativeCondition::new(DegenerativeType::Alzheimer, 0.01);
        alz.progression = 0.5;
        let mut dep = DegenerativeCondition::new(DegenerativeType::MajorDepression, 0.01);
        dep.progression = 0.5;
        mgr.add(alz);
        mgr.add(dep);
        let effects = mgr.cumulative_effects();
        assert!(effects.memory_retention < 0.7);
        assert!(effects.motivation < 0.7);
    }
}
