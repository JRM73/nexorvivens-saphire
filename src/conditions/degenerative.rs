// =============================================================================
// conditions/degenerative.rs — Maladies degeneratives
// =============================================================================
//
// Role : Modelise Alzheimer, Parkinson, epilepsie, demence, depression
//        profonde. Chaque maladie degrade des capacites specifiques au fil
//        du temps et impacte la chimie.
//
// Integration :
//   Les effets cognitifs (memoire, vitesse, motivation) sont appliques
//   dans le pipeline via CognitiveEffects.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Type de maladie degenerative.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DegenerativeType {
    /// Perte progressive de memoire, reconnaissance
    Alzheimer,
    /// Lenteur motrice, tremblements, rigidite
    Parkinson,
    /// Crises episodiques, confusion post-crise
    Epilepsy,
    /// Jugement, raisonnement, orientation degrades
    Dementia,
    /// Anhedonie, perte de motivation, energie basse
    MajorDepression,
}

/// Effets cognitifs cumules des maladies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveEffects {
    /// Multiplicateur de retention memoire (1.0 = normal, 0.0 = aucune)
    pub memory_retention: f64,
    /// Multiplicateur de vitesse de reponse (1.0 = normal)
    pub response_speed: f64,
    /// Clarte du raisonnement (1.0 = clair, 0.0 = confus)
    pub reasoning_clarity: f64,
    /// Motivation (1.0 = motivee, 0.0 = apathique)
    pub motivation: f64,
    /// Risque de crise epileptique (0.0 = aucun, 1.0 = imminent)
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

/// Une maladie degenerative individuelle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegenerativeCondition {
    pub disease_type: DegenerativeType,
    /// Progression (0.0 = debut, 1.0 = terminal)
    pub progression: f64,
    /// Taux de progression par cycle
    pub progression_rate: f64,
    /// Sous traitement (ralentit la progression)
    pub under_treatment: bool,
    /// Cycles depuis le debut
    pub cycles_since_onset: u64,
    /// En crise (epilepsie)
    pub in_crisis: bool,
    /// Compteur de crises (epilepsie)
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

    /// Met a jour la progression.
    pub fn tick(&mut self) {
        self.cycles_since_onset += 1;

        let rate = if self.under_treatment {
            self.progression_rate * 0.3 // Traitement ralentit
        } else {
            self.progression_rate
        };

        self.progression = (self.progression + rate).min(1.0);

        // Epilepsie : crises aleatoires basees sur la progression
        if self.disease_type == DegenerativeType::Epilepsy {
            // Crise si progression * hash_cycle donne un resultat critique
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

    /// Effets cognitifs de cette maladie.
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

    /// Impact chimique.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let p = self.progression;
        let mut adj = ChemistryAdjustment::default();

        match self.disease_type {
            DegenerativeType::Alzheimer | DegenerativeType::Dementia => {
                adj.cortisol += p * 0.01;
                adj.serotonin -= p * 0.01;
            }
            DegenerativeType::Parkinson => {
                adj.dopamine -= p * 0.02; // Deficit dopaminergique
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

/// Gestionnaire de maladies degeneratives.
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

    /// Effets cognitifs cumules (multiplicatifs).
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

    /// Impact chimique cumule.
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
