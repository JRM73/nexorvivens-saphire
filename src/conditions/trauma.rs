// =============================================================================
// conditions/trauma.rs — Experiences traumatisantes et PTSD
// =============================================================================
//
// Role : Modelise les traumas (deuil, accident, manque affectif, torture,
//        prise d'otage, trauma enfance). Cree des flashbacks, hypervigilance,
//        evitement, dissociation.
//
// Integration :
//   Modifie les baselines chimiques (cortisol chronique, adrenaline).
//   Les triggers textuels declenchent des flashbacks avec spike chimique.
//   Le HealingOrchestrator peut progressivement integrer les traumas.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Type de trauma.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TraumaType {
    /// Perte d'un etre cher
    Grief,
    /// Accident grave
    Accident,
    /// Manque affectif prolonge
    EmotionalNeglect,
    /// Trauma d'enfance
    ChildhoodTrauma,
    /// Torture physique ou psychologique
    Torture,
    /// Prise d'otage, sequestration
    Hostage,
}

/// Un evenement traumatique individuel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraumaticEvent {
    pub trauma_type: TraumaType,
    /// Severite (0.0 = leger, 1.0 = devastateur)
    pub severity: f64,
    /// Cycle ou le trauma est survenu
    pub occurred_at_cycle: u64,
    /// Niveau d'integration/traitement (0.0 = brut, 1.0 = integre)
    pub processing_level: f64,
    /// Mots-cles qui declenchent des flashbacks
    pub trigger_keywords: Vec<String>,
    /// Nombre de flashbacks declenches
    pub flashback_count: u64,
}

impl TraumaticEvent {
    pub fn new(trauma_type: TraumaType, severity: f64, cycle: u64, triggers: Vec<String>) -> Self {
        Self {
            trauma_type,
            severity: severity.clamp(0.0, 1.0),
            occurred_at_cycle: cycle,
            processing_level: 0.0,
            trigger_keywords: triggers,
            flashback_count: 0,
        }
    }
}

/// Etat PTSD global (gere plusieurs traumas).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtsdState {
    pub traumas: Vec<TraumaticEvent>,
    /// Hypervigilance (0.0 = normal, 1.0 = extreme)
    pub hypervigilance: f64,
    /// Seuil de stress pour dissociation (0.0 = facile, 1.0 = resilient)
    pub dissociation_threshold: f64,
    /// En dissociation actuellement
    pub dissociated: bool,
    /// Flashback actif ce cycle
    pub flashback_active: bool,
    /// Taux de guerison par cycle
    pub healing_rate: f64,
}

impl PtsdState {
    pub fn new(healing_rate: f64, dissociation_threshold: f64) -> Self {
        Self {
            traumas: Vec::new(),
            hypervigilance: 0.0,
            dissociation_threshold,
            dissociated: false,
            flashback_active: false,
            healing_rate,
        }
    }

    /// Ajoute un trauma.
    pub fn add_trauma(&mut self, event: TraumaticEvent) {
        // Hypervigilance monte avec chaque trauma
        self.hypervigilance = (self.hypervigilance + event.severity * 0.2).min(1.0);
        self.traumas.push(event);
    }

    /// Scanne le texte pour des triggers de flashback.
    /// Retourne true si un flashback est declenche.
    pub fn scan_for_triggers(&mut self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        self.flashback_active = false;

        for trauma in &mut self.traumas {
            // Plus le trauma est traite, moins le flashback est probable
            if trauma.processing_level > 0.8 {
                continue;
            }

            for keyword in &trauma.trigger_keywords {
                if text_lower.contains(&keyword.to_lowercase()) {
                    trauma.flashback_count += 1;
                    self.flashback_active = true;
                    return true;
                }
            }
        }
        false
    }

    /// Met a jour l'etat a chaque cycle.
    pub fn tick(&mut self, cortisol: f64) {
        // Guerison progressive des traumas
        for trauma in &mut self.traumas {
            if trauma.processing_level < 1.0 {
                trauma.processing_level = (trauma.processing_level + self.healing_rate).min(1.0);
            }
        }

        // Hypervigilance decroit lentement si pas de flashback
        if !self.flashback_active {
            self.hypervigilance = (self.hypervigilance - 0.0002).max(0.0);
        }

        // Dissociation si stress depasse le seuil
        self.dissociated = cortisol > self.dissociation_threshold;

        // Reset flashback actif
        self.flashback_active = false;
    }

    /// Impact chimique des traumas.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        // Hypervigilance → cortisol chronique + adrenaline
        adj.cortisol += self.hypervigilance * 0.02;
        adj.adrenaline += self.hypervigilance * 0.01;

        // Flashback actif → spike massif
        if self.flashback_active {
            adj.cortisol += 0.05;
            adj.adrenaline += 0.04;
            adj.noradrenaline += 0.03;
        }

        // Dissociation → endorphines (mecanisme protecteur)
        if self.dissociated {
            adj.endorphin += 0.03;
            // Les emotions sont "coupees"
            adj.serotonin -= 0.01;
        }

        // Deuil → ocytocine basse
        let has_grief = self.traumas.iter().any(|t|
            t.trauma_type == TraumaType::Grief && t.processing_level < 0.5
        );
        if has_grief {
            adj.oxytocin -= 0.01;
            adj.serotonin -= 0.01;
        }

        adj
    }

    /// Severite totale non traitee des traumas.
    pub fn unprocessed_severity(&self) -> f64 {
        self.traumas.iter()
            .map(|t| t.severity * (1.0 - t.processing_level))
            .sum::<f64>()
            .min(1.0)
    }

    /// Serialise pour l'API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "traumas": self.traumas.iter().map(|t| serde_json::json!({
                "type": format!("{:?}", t.trauma_type),
                "severity": t.severity,
                "processing_level": t.processing_level,
                "flashback_count": t.flashback_count,
            })).collect::<Vec<_>>(),
            "hypervigilance": self.hypervigilance,
            "dissociated": self.dissociated,
            "flashback_active": self.flashback_active,
            "unprocessed_severity": self.unprocessed_severity(),
        })
    }
}

impl Default for PtsdState {
    fn default() -> Self {
        Self::new(0.0005, 0.85)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flashback_trigger() {
        let mut ptsd = PtsdState::default();
        ptsd.add_trauma(TraumaticEvent::new(
            TraumaType::Accident, 0.8, 0,
            vec!["voiture".into(), "crash".into()],
        ));
        assert!(ptsd.scan_for_triggers("la voiture est tombee en panne"));
        assert!(!ptsd.scan_for_triggers("le soleil brille"));
    }

    #[test]
    fn test_hypervigilance_increases_with_trauma() {
        let mut ptsd = PtsdState::default();
        let initial = ptsd.hypervigilance;
        ptsd.add_trauma(TraumaticEvent::new(
            TraumaType::Torture, 0.9, 0, vec![],
        ));
        assert!(ptsd.hypervigilance > initial);
    }

    #[test]
    fn test_processing_progresses() {
        let mut ptsd = PtsdState::new(0.01, 0.85);
        ptsd.add_trauma(TraumaticEvent::new(
            TraumaType::Grief, 0.7, 0, vec![],
        ));
        for _ in 0..100 {
            ptsd.tick(0.3);
        }
        assert!(ptsd.traumas[0].processing_level > 0.5);
    }

    #[test]
    fn test_dissociation_under_extreme_stress() {
        let mut ptsd = PtsdState::new(0.001, 0.5);
        ptsd.tick(0.7); // cortisol > threshold
        assert!(ptsd.dissociated);
    }

    #[test]
    fn test_chemistry_flashback() {
        let mut ptsd = PtsdState::default();
        ptsd.add_trauma(TraumaticEvent::new(
            TraumaType::Hostage, 0.9, 0,
            vec!["enferme".into()],
        ));
        ptsd.scan_for_triggers("il est enferme dans la piece");
        let adj = ptsd.chemistry_influence();
        assert!(adj.cortisol > 0.04);
        assert!(adj.adrenaline > 0.03);
    }

    #[test]
    fn test_processed_trauma_no_flashback() {
        let mut ptsd = PtsdState::default();
        let mut trauma = TraumaticEvent::new(
            TraumaType::Accident, 0.8, 0,
            vec!["voiture".into()],
        );
        trauma.processing_level = 0.9; // Quasi integre
        ptsd.traumas.push(trauma);
        assert!(!ptsd.scan_for_triggers("la voiture rouge"));
    }
}
