// =============================================================================
// care/surgery.rs — Chirurgie et operations
// =============================================================================
//
// Role : Modelise les interventions chirurgicales avec leurs phases :
//        pre-operatoire (anxiete), operation (anesthesie), post-operatoire
//        (douleur + recuperation). Chaque phase a un impact chimique distinct.
//
// Integration :
//   Affecte la physiologie (energie, douleur, immunite).
//   Interagit avec les maladies (P2.3) comme traitement curatif.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Phase d'une operation chirurgicale.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SurgeryPhase {
    /// Attente avant l'operation (anxiete, preparation)
    PreOp,
    /// Sous anesthesie (inconscience, pas de douleur)
    UnderAnesthesia,
    /// Reveil post-operatoire immediat (douleur, confusion)
    PostOpImmediate,
    /// Convalescence (guerison progressive)
    Recovery,
    /// Completement retabli
    Recovered,
}

/// Une operation chirurgicale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Surgery {
    /// Description de l'operation
    pub description: String,
    /// Condition traitee (id texte libre)
    pub target_condition: String,
    /// Phase courante
    pub phase: SurgeryPhase,
    /// Gravite de l'operation (0.0 = mineure, 1.0 = lourde)
    pub severity: f64,
    /// Cycles dans la phase courante
    pub cycles_in_phase: u64,
    /// Succes de l'operation (determine en sortie d'anesthesie)
    pub success: bool,
    /// Progres de guerison post-op (0.0-1.0)
    pub recovery_progress: f64,
    /// Niveau de douleur post-op (decroit avec la guerison)
    pub pain_level: f64,
}

impl Surgery {
    pub fn new(description: &str, target: &str, severity: f64) -> Self {
        Self {
            description: description.to_string(),
            target_condition: target.to_string(),
            phase: SurgeryPhase::PreOp,
            severity: severity.clamp(0.0, 1.0),
            cycles_in_phase: 0,
            success: true,
            recovery_progress: 0.0,
            pain_level: 0.0,
        }
    }

    /// Fait avancer l'operation d'un cycle.
    pub fn tick(&mut self) {
        self.cycles_in_phase += 1;

        match self.phase {
            SurgeryPhase::PreOp => {
                // 20 cycles de preparation/attente
                if self.cycles_in_phase >= 20 {
                    self.phase = SurgeryPhase::UnderAnesthesia;
                    self.cycles_in_phase = 0;
                }
            }
            SurgeryPhase::UnderAnesthesia => {
                // Duree proportionnelle a la gravite
                let op_duration = (self.severity * 30.0) as u64 + 10;
                if self.cycles_in_phase >= op_duration {
                    self.phase = SurgeryPhase::PostOpImmediate;
                    self.cycles_in_phase = 0;
                    self.pain_level = self.severity * 0.8;
                }
            }
            SurgeryPhase::PostOpImmediate => {
                // 30 cycles de reveil douloureux
                self.pain_level = (self.pain_level - 0.005).max(self.severity * 0.3);
                if self.cycles_in_phase >= 30 {
                    self.phase = SurgeryPhase::Recovery;
                    self.cycles_in_phase = 0;
                }
            }
            SurgeryPhase::Recovery => {
                // Guerison progressive
                let recovery_rate = 0.002 * (1.0 - self.severity * 0.5);
                self.recovery_progress = (self.recovery_progress + recovery_rate).min(1.0);
                self.pain_level = (self.pain_level - 0.002).max(0.0);

                if self.recovery_progress >= 1.0 {
                    self.phase = SurgeryPhase::Recovered;
                    self.pain_level = 0.0;
                }
            }
            SurgeryPhase::Recovered => {
                // Rien a faire
            }
        }
    }

    /// Impact chimique selon la phase.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        match self.phase {
            SurgeryPhase::PreOp => ChemistryAdjustment {
                cortisol: 0.02 * self.severity,
                adrenaline: 0.01 * self.severity,
                noradrenaline: 0.01,
                ..Default::default()
            },
            SurgeryPhase::UnderAnesthesia => ChemistryAdjustment {
                // Anesthesie : tout est attenue
                endorphin: 0.02,
                ..Default::default()
            },
            SurgeryPhase::PostOpImmediate => ChemistryAdjustment {
                cortisol: self.pain_level * 0.03,
                endorphin: self.pain_level * 0.02,
                adrenaline: 0.01,
                serotonin: -0.01,
                ..Default::default()
            },
            SurgeryPhase::Recovery => ChemistryAdjustment {
                cortisol: self.pain_level * 0.01,
                endorphin: self.pain_level * 0.01,
                serotonin: self.recovery_progress * 0.005,
                ..Default::default()
            },
            SurgeryPhase::Recovered => ChemistryAdjustment {
                serotonin: 0.005, // Soulagement
                dopamine: 0.003,  // Satisfaction
                ..Default::default()
            },
        }
    }

    pub fn is_complete(&self) -> bool {
        self.phase == SurgeryPhase::Recovered
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "description": self.description,
            "target": self.target_condition,
            "phase": format!("{:?}", self.phase),
            "severity": self.severity,
            "success": self.success,
            "recovery_progress": self.recovery_progress,
            "pain_level": self.pain_level,
            "complete": self.is_complete(),
        })
    }
}

/// Gestionnaire de chirurgies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurgeryManager {
    pub surgeries: Vec<Surgery>,
    /// Historique des operations terminees
    pub history_count: u32,
}

impl SurgeryManager {
    pub fn new() -> Self {
        Self { surgeries: Vec::new(), history_count: 0 }
    }

    /// Planifie une operation.
    pub fn schedule(&mut self, surgery: Surgery) {
        self.surgeries.push(surgery);
    }

    /// Met a jour toutes les operations.
    pub fn tick(&mut self) {
        for s in &mut self.surgeries {
            s.tick();
        }
        let completed = self.surgeries.iter().filter(|s| s.is_complete()).count();
        self.history_count += completed as u32;
        self.surgeries.retain(|s| !s.is_complete());
    }

    /// Impact chimique total.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();
        for s in &self.surgeries {
            let a = s.chemistry_influence();
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

    /// Douleur totale des operations en cours.
    pub fn total_pain(&self) -> f64 {
        self.surgeries.iter().map(|s| s.pain_level).sum::<f64>().min(1.0)
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "active": self.surgeries.iter().map(|s| s.to_json()).collect::<Vec<_>>(),
            "history_count": self.history_count,
            "total_pain": self.total_pain(),
        })
    }
}

impl Default for SurgeryManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surgery_phases() {
        let mut s = Surgery::new("Appendicectomie", "appendicite", 0.4);
        assert_eq!(s.phase, SurgeryPhase::PreOp);

        // Pre-op (20 cycles)
        for _ in 0..20 {
            s.tick();
        }
        assert_eq!(s.phase, SurgeryPhase::UnderAnesthesia);

        // Operation (0.4*30 + 10 = 22 cycles)
        for _ in 0..22 {
            s.tick();
        }
        assert_eq!(s.phase, SurgeryPhase::PostOpImmediate);
        assert!(s.pain_level > 0.0);

        // Post-op immediat (30 cycles)
        for _ in 0..30 {
            s.tick();
        }
        assert_eq!(s.phase, SurgeryPhase::Recovery);
    }

    #[test]
    fn test_surgery_recovery() {
        let mut s = Surgery::new("Test", "test", 0.3);
        // Avancer jusqu'a la phase Recovery
        for _ in 0..100 {
            s.tick();
        }
        // Puis attendre longtemps
        for _ in 0..1000 {
            s.tick();
        }
        assert!(s.is_complete());
    }

    #[test]
    fn test_preop_anxiety() {
        let s = Surgery::new("Lourde", "test", 0.9);
        let adj = s.chemistry_influence();
        assert!(adj.cortisol > 0.0);
        assert!(adj.adrenaline > 0.0);
    }

    #[test]
    fn test_manager_cleanup() {
        let mut mgr = SurgeryManager::new();
        mgr.schedule(Surgery::new("Test", "test", 0.2));
        // Faire tourner longtemps
        for _ in 0..2000 {
            mgr.tick();
        }
        assert_eq!(mgr.surgeries.len(), 0);
        assert!(mgr.history_count > 0);
    }
}
