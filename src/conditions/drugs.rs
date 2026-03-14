// =============================================================================
// conditions/drugs.rs — Drogues et effets pharmacologiques
// =============================================================================
//
// Role : Modelise les effets pharmacologiques de substances sur la chimie.
//        Chaque drogue a des phases (onset, peak, comedown, after_effects)
//        avec des modifiers chimiques specifiques.
//
// Integration :
//   Modifie la chimie a chaque cycle selon la phase active.
//   Notifie le systeme d'addictions (P2.5) pour l'exposition.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Phase d'effet d'une drogue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PhaseType {
    /// Montee des effets
    Onset,
    /// Pic d'effet maximal
    Peak,
    /// Maintien de l'effet
    Plateau,
    /// Descente progressive
    Comedown,
    /// Apres-effets (lendemain, hangover)
    AfterEffects,
}

/// Phase individuelle d'une drogue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugPhase {
    pub phase_type: PhaseType,
    /// Duree de cette phase en cycles
    pub duration_cycles: u32,
    /// Impact chimique durant cette phase
    pub chemistry: ChemistryAdjustment,
}

/// Profil pharmacologique d'une substance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugProfile {
    pub name: String,
    /// Potentiel addictif (0.0 = aucun, 1.0 = extreme)
    pub addiction_potential: f64,
    /// Phases d'effets ordonnees
    pub phases: Vec<DrugPhase>,
}

/// Une dose active en cours de metabolisation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveDose {
    pub drug_name: String,
    /// Index de la phase courante
    pub current_phase: usize,
    /// Cycles ecoules dans la phase courante
    pub cycles_in_phase: u32,
    /// Profil de la drogue
    profile: DrugProfile,
}

impl ActiveDose {
    pub fn new(profile: DrugProfile) -> Self {
        Self {
            drug_name: profile.name.clone(),
            current_phase: 0,
            cycles_in_phase: 0,
            profile,
        }
    }

    /// Met a jour et retourne l'impact chimique courant.
    /// Retourne None si toutes les phases sont terminees.
    pub fn tick(&mut self) -> Option<ChemistryAdjustment> {
        if self.current_phase >= self.profile.phases.len() {
            return None;
        }

        let phase = &self.profile.phases[self.current_phase];
        let adj = phase.chemistry.clone();
        self.cycles_in_phase += 1;

        // Passer a la phase suivante si duree ecoulee
        if self.cycles_in_phase >= phase.duration_cycles {
            self.current_phase += 1;
            self.cycles_in_phase = 0;
        }

        Some(adj)
    }

    /// Phase courante (pour affichage).
    pub fn current_phase_type(&self) -> Option<&PhaseType> {
        self.profile.phases.get(self.current_phase).map(|p| &p.phase_type)
    }

    /// Est-ce que l'effet est termine ?
    pub fn is_finished(&self) -> bool {
        self.current_phase >= self.profile.phases.len()
    }
}

/// Gestionnaire de drogues actives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugManager {
    /// Doses actives en cours
    pub active_doses: Vec<ActiveDose>,
}

impl DrugManager {
    pub fn new() -> Self {
        Self {
            active_doses: Vec::new(),
        }
    }

    /// Administre une dose.
    pub fn administer(&mut self, profile: DrugProfile) {
        self.active_doses.push(ActiveDose::new(profile));
    }

    /// Met a jour toutes les doses actives.
    pub fn tick(&mut self) -> ChemistryAdjustment {
        let mut total = ChemistryAdjustment::default();

        // Tick chaque dose et accumuler l'impact
        for dose in &mut self.active_doses {
            if let Some(adj) = dose.tick() {
                total.dopamine += adj.dopamine;
                total.cortisol += adj.cortisol;
                total.serotonin += adj.serotonin;
                total.adrenaline += adj.adrenaline;
                total.oxytocin += adj.oxytocin;
                total.endorphin += adj.endorphin;
                total.noradrenaline += adj.noradrenaline;
            }
        }

        // Retirer les doses terminees
        self.active_doses.retain(|d| !d.is_finished());

        total
    }

    /// Serialise pour l'API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "active_doses": self.active_doses.iter().map(|d| serde_json::json!({
                "drug": d.drug_name,
                "phase": d.current_phase_type().map(|p| format!("{:?}", p)),
                "finished": d.is_finished(),
            })).collect::<Vec<_>>(),
            "count": self.active_doses.len(),
        })
    }
}

impl Default for DrugManager {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Catalogue de drogues pre-definies
// =============================================================================

/// Cree le profil pharmacologique d'une substance connue.
pub fn drug_catalog(name: &str) -> Option<DrugProfile> {
    match name {
        "caffeine" => Some(DrugProfile {
            name: "caffeine".into(),
            addiction_potential: 0.2,
            phases: vec![
                DrugPhase { phase_type: PhaseType::Onset, duration_cycles: 5,
                    chemistry: ChemistryAdjustment { dopamine: 0.01, cortisol: 0.005, noradrenaline: 0.01, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::Peak, duration_cycles: 15,
                    chemistry: ChemistryAdjustment { dopamine: 0.02, noradrenaline: 0.015, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::Comedown, duration_cycles: 20,
                    chemistry: ChemistryAdjustment { dopamine: -0.005, cortisol: 0.005, ..Default::default() } },
            ],
        }),
        "alcohol" => Some(DrugProfile {
            name: "alcohol".into(),
            addiction_potential: 0.5,
            phases: vec![
                DrugPhase { phase_type: PhaseType::Onset, duration_cycles: 5,
                    chemistry: ChemistryAdjustment { dopamine: 0.02, serotonin: 0.01, endorphin: 0.01, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::Peak, duration_cycles: 20,
                    chemistry: ChemistryAdjustment { dopamine: 0.03, serotonin: 0.02, endorphin: 0.02, cortisol: -0.01, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::Comedown, duration_cycles: 15,
                    chemistry: ChemistryAdjustment { dopamine: -0.01, serotonin: -0.02, cortisol: 0.02, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::AfterEffects, duration_cycles: 30,
                    chemistry: ChemistryAdjustment { dopamine: -0.005, serotonin: -0.01, cortisol: 0.015, ..Default::default() } },
            ],
        }),
        "nicotine" => Some(DrugProfile {
            name: "nicotine".into(),
            addiction_potential: 0.7,
            phases: vec![
                DrugPhase { phase_type: PhaseType::Onset, duration_cycles: 2,
                    chemistry: ChemistryAdjustment { dopamine: 0.015, noradrenaline: 0.01, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::Peak, duration_cycles: 5,
                    chemistry: ChemistryAdjustment { dopamine: 0.02, noradrenaline: 0.015, cortisol: -0.005, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::Comedown, duration_cycles: 10,
                    chemistry: ChemistryAdjustment { dopamine: -0.01, cortisol: 0.01, noradrenaline: -0.005, ..Default::default() } },
            ],
        }),
        "cannabis" => Some(DrugProfile {
            name: "cannabis".into(),
            addiction_potential: 0.15,
            phases: vec![
                DrugPhase { phase_type: PhaseType::Onset, duration_cycles: 3,
                    chemistry: ChemistryAdjustment { dopamine: 0.01, serotonin: 0.01, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::Peak, duration_cycles: 15,
                    chemistry: ChemistryAdjustment { dopamine: 0.015, serotonin: 0.015, endorphin: 0.01, cortisol: -0.01, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::Comedown, duration_cycles: 20,
                    chemistry: ChemistryAdjustment { dopamine: -0.005, serotonin: -0.005, ..Default::default() } },
            ],
        }),
        "mdma" => Some(DrugProfile {
            name: "mdma".into(),
            addiction_potential: 0.4,
            phases: vec![
                DrugPhase { phase_type: PhaseType::Onset, duration_cycles: 5,
                    chemistry: ChemistryAdjustment { serotonin: 0.03, oxytocin: 0.02, dopamine: 0.02, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::Peak, duration_cycles: 20,
                    chemistry: ChemistryAdjustment { serotonin: 0.06, oxytocin: 0.04, dopamine: 0.03, endorphin: 0.02, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::Comedown, duration_cycles: 15,
                    chemistry: ChemistryAdjustment { serotonin: -0.04, dopamine: -0.02, cortisol: 0.03, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::AfterEffects, duration_cycles: 50,
                    chemistry: ChemistryAdjustment { serotonin: -0.02, cortisol: 0.01, dopamine: -0.01, ..Default::default() } },
            ],
        }),
        "cocaine" => Some(DrugProfile {
            name: "cocaine".into(),
            addiction_potential: 0.85,
            phases: vec![
                DrugPhase { phase_type: PhaseType::Onset, duration_cycles: 1,
                    chemistry: ChemistryAdjustment { dopamine: 0.05, noradrenaline: 0.03, adrenaline: 0.02, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::Peak, duration_cycles: 8,
                    chemistry: ChemistryAdjustment { dopamine: 0.08, noradrenaline: 0.04, adrenaline: 0.03, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::Comedown, duration_cycles: 10,
                    chemistry: ChemistryAdjustment { dopamine: -0.05, cortisol: 0.04, serotonin: -0.03, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::AfterEffects, duration_cycles: 30,
                    chemistry: ChemistryAdjustment { dopamine: -0.02, cortisol: 0.02, serotonin: -0.01, ..Default::default() } },
            ],
        }),
        "heroin" => Some(DrugProfile {
            name: "heroin".into(),
            addiction_potential: 0.95,
            phases: vec![
                DrugPhase { phase_type: PhaseType::Onset, duration_cycles: 2,
                    chemistry: ChemistryAdjustment { endorphin: 0.08, dopamine: 0.04, cortisol: -0.03, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::Peak, duration_cycles: 15,
                    chemistry: ChemistryAdjustment { endorphin: 0.1, dopamine: 0.05, cortisol: -0.04, serotonin: 0.02, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::Comedown, duration_cycles: 20,
                    chemistry: ChemistryAdjustment { endorphin: -0.04, dopamine: -0.03, cortisol: 0.04, ..Default::default() } },
                DrugPhase { phase_type: PhaseType::AfterEffects, duration_cycles: 40,
                    chemistry: ChemistryAdjustment { endorphin: -0.02, cortisol: 0.03, dopamine: -0.02, noradrenaline: 0.02, ..Default::default() } },
            ],
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caffeine_cycle() {
        let profile = drug_catalog("caffeine").unwrap();
        let mut dose = ActiveDose::new(profile);
        assert!(!dose.is_finished());

        let mut total_dopamine = 0.0;
        let mut cycles = 0;
        while !dose.is_finished() {
            if let Some(adj) = dose.tick() {
                total_dopamine += adj.dopamine;
            }
            cycles += 1;
            if cycles > 100 { break; } // Securite
        }
        assert!(dose.is_finished());
        assert!(cycles == 40); // 5 + 15 + 20
    }

    #[test]
    fn test_manager_cleanup() {
        let mut mgr = DrugManager::new();
        mgr.administer(drug_catalog("nicotine").unwrap());
        assert_eq!(mgr.active_doses.len(), 1);

        // Faire tourner jusqu'a la fin
        for _ in 0..100 {
            mgr.tick();
        }
        assert_eq!(mgr.active_doses.len(), 0);
    }

    #[test]
    fn test_cocaine_high_dopamine() {
        let profile = drug_catalog("cocaine").unwrap();
        let mut dose = ActiveDose::new(profile);
        // Passer la phase onset
        dose.tick();
        // Phase peak
        if let Some(adj) = dose.tick() {
            assert!(adj.dopamine > 0.05);
        }
    }

    #[test]
    fn test_heroin_high_endorphin() {
        let profile = drug_catalog("heroin").unwrap();
        let mut dose = ActiveDose::new(profile);
        // Onset
        dose.tick(); dose.tick();
        // Peak
        if let Some(adj) = dose.tick() {
            assert!(adj.endorphin > 0.05);
        }
    }

    #[test]
    fn test_unknown_drug_returns_none() {
        assert!(drug_catalog("unobtainium").is_none());
    }

    #[test]
    fn test_multiple_drugs_stack() {
        let mut mgr = DrugManager::new();
        mgr.administer(drug_catalog("caffeine").unwrap());
        mgr.administer(drug_catalog("nicotine").unwrap());
        let adj = mgr.tick();
        // Les deux contribuent a dopamine pendant l'onset
        assert!(adj.dopamine > 0.01);
    }
}
