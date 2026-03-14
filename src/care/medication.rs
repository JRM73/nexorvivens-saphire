// =============================================================================
// care/medication.rs — Medicaments therapeutiques
// =============================================================================
//
// Role : Medicaments a usage therapeutique. Reutilise le meme systeme de phases
//        que drugs.rs (pharmacologie) mais avec des profils therapeutiques :
//        antidepresseurs, anxiolytiques, antidouleur, stimulants, etc.
//
// Difference avec drugs.rs :
//   - Les medicaments sont prescrits (volontaires, therapeutiques)
//   - Effets secondaires explicites
//   - Tolerance et dependance iatrogene possibles
//   - Duree de traitement longue (semaines/mois)
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Categorie de medicament.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MedicationCategory {
    /// ISRS — inhibiteurs selectifs de recapture de serotonine
    Antidepressant,
    /// Benzodiazepines — reduction anxiete rapide
    Anxiolytic,
    /// Opioides ou AINS — soulagement douleur
    Painkiller,
    /// Methylphenidate, modafinil — concentration
    Stimulant,
    /// Antipsychotiques — reduction dopamine excessive
    Neuroleptic,
    /// Lithium, valproate — lissage cycles humeur
    MoodStabilizer,
}

/// Effets secondaires possibles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffects {
    /// Somnolence (0.0-1.0)
    pub drowsiness: f64,
    /// Perte d'appetit (0.0-1.0)
    pub appetite_loss: f64,
    /// Emoussement emotionnel (0.0-1.0)
    pub emotional_blunting: f64,
    /// Risque de dependance (0.0-1.0)
    pub dependency_risk: f64,
    /// Prise de poids (0.0-1.0)
    pub weight_gain: f64,
}

/// Un medicament en cours de traitement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medication {
    pub name: String,
    pub category: MedicationCategory,
    /// Impact chimique par cycle (effet therapeutique)
    pub effect: ChemistryAdjustment,
    /// Effets secondaires
    pub side_effects: SideEffects,
    /// Cycles depuis le debut du traitement
    pub cycles_on_medication: u64,
    /// Delai avant que l'effet therapeutique plein se manifeste
    pub onset_delay_cycles: u64,
    /// Efficacite courante (0.0 = pas encore actif, 1.0 = plein effet)
    pub current_efficacy: f64,
    /// Tolerance accumulee (0.0 = aucune, 1.0 = inefficace)
    pub tolerance: f64,
    /// En cours d'arret progressif (tapering)
    pub tapering: bool,
    /// Dose relative (1.0 = dose standard)
    pub dose: f64,
}

impl Medication {
    /// Met a jour l'etat du medicament a chaque cycle.
    pub fn tick(&mut self) {
        self.cycles_on_medication += 1;

        // Efficacite monte progressivement jusqu'au plein effet
        if self.cycles_on_medication < self.onset_delay_cycles {
            self.current_efficacy = self.cycles_on_medication as f64
                / self.onset_delay_cycles as f64;
        } else {
            self.current_efficacy = 1.0;
        }

        // Tolerance lente sur le long terme
        self.tolerance = (self.tolerance + 0.0001).min(0.5);

        // Si tapering, reduire la dose progressivement
        if self.tapering {
            self.dose = (self.dose - 0.005).max(0.0);
        }
    }

    /// Impact chimique therapeutique (module par efficacite, tolerance, dose).
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

    /// Est-ce que le medicament est completement arrete ?
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

/// Gestionnaire de medicaments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicationManager {
    pub medications: Vec<Medication>,
}

impl MedicationManager {
    pub fn new() -> Self {
        Self { medications: Vec::new() }
    }

    /// Prescrit un medicament.
    pub fn prescribe(&mut self, med: Medication) {
        self.medications.push(med);
    }

    /// Met a jour tous les medicaments.
    pub fn tick(&mut self) {
        for med in &mut self.medications {
            med.tick();
        }
        // Retirer les medicaments completement arretes
        self.medications.retain(|m| !m.is_stopped());
    }

    /// Impact chimique total de tous les medicaments.
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

    /// Arrete progressivement un medicament par nom.
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
// Catalogue de medicaments pre-definis
// =============================================================================

/// Cree un medicament standard a partir du catalogue.
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
            onset_delay_cycles: 100, // ~2 semaines avant plein effet
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
            onset_delay_cycles: 5, // Effet rapide
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
                // Lisse les extremes : attenue dopamine et cortisol
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
            onset_delay_cycles: 70, // Effet lent
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
        // A mi-chemin du delai de 100 cycles
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
        // Simuler plein effet
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
