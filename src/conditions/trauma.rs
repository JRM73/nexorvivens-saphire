// =============================================================================
// conditions/trauma.rs — Traumatic experiences and PTSD
// =============================================================================
//
// Purpose: Models traumas (grief, accident, emotional neglect, torture,
//          hostage taking, childhood trauma). Creates flashbacks,
//          hypervigilance, avoidance, dissociation.
//
// Integration:
//   Modifies chemistry baselines (chronic cortisol, adrenaline).
//   Text triggers cause flashbacks with chemistry spikes.
//   The HealingOrchestrator can progressively integrate traumas.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Type of trauma.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TraumaType {
    /// Loss of a loved one
    Grief,
    /// Serious accident
    Accident,
    /// Prolonged emotional neglect
    EmotionalNeglect,
    /// Childhood trauma
    ChildhoodTrauma,
    /// Physical or psychological torture
    Torture,
    /// Hostage taking, confinement
    Hostage,
}

/// An individual traumatic event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraumaticEvent {
    pub trauma_type: TraumaType,
    /// Severity (0.0 = mild, 1.0 = devastating)
    pub severity: f64,
    /// Cycle when the trauma occurred
    pub occurred_at_cycle: u64,
    /// Processing/integration level (0.0 = raw, 1.0 = integrated)
    pub processing_level: f64,
    /// Keywords that trigger flashbacks
    pub trigger_keywords: Vec<String>,
    /// Number of flashbacks triggered
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

/// Overall PTSD state (manages multiple traumas).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtsdState {
    pub traumas: Vec<TraumaticEvent>,
    /// Hypervigilance (0.0 = normal, 1.0 = extreme)
    pub hypervigilance: f64,
    /// Stress threshold for dissociation (0.0 = easy, 1.0 = resilient)
    pub dissociation_threshold: f64,
    /// Currently dissociated
    pub dissociated: bool,
    /// Flashback active this cycle
    pub flashback_active: bool,
    /// Healing rate per cycle
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

    /// Adds a trauma.
    pub fn add_trauma(&mut self, event: TraumaticEvent) {
        // Hypervigilance increases with each trauma
        self.hypervigilance = (self.hypervigilance + event.severity * 0.2).min(1.0);
        self.traumas.push(event);
    }

    /// Scans text for flashback triggers.
    /// Returns true if a flashback is triggered.
    pub fn scan_for_triggers(&mut self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        self.flashback_active = false;

        for trauma in &mut self.traumas {
            // The more processed the trauma, the less likely a flashback
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

    /// Updates the state each cycle.
    pub fn tick(&mut self, cortisol: f64) {
        // Progressive healing of traumas
        for trauma in &mut self.traumas {
            if trauma.processing_level < 1.0 {
                trauma.processing_level = (trauma.processing_level + self.healing_rate).min(1.0);
            }
        }

        // Hypervigilance decays slowly if no flashback
        if !self.flashback_active {
            self.hypervigilance = (self.hypervigilance - 0.0002).max(0.0);
        }

        // Dissociation if stress exceeds threshold
        self.dissociated = cortisol > self.dissociation_threshold;

        // Reset active flashback
        self.flashback_active = false;
    }

    /// Chemistry impact of traumas.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        // Hypervigilance -> chronic cortisol + adrenaline
        adj.cortisol += self.hypervigilance * 0.02;
        adj.adrenaline += self.hypervigilance * 0.01;

        // Active flashback -> massive spike
        if self.flashback_active {
            adj.cortisol += 0.05;
            adj.adrenaline += 0.04;
            adj.noradrenaline += 0.03;
        }

        // Dissociation -> endorphins (protective mechanism)
        if self.dissociated {
            adj.endorphin += 0.03;
            // Emotions are "cut off"
            adj.serotonin -= 0.01;
        }

        // Grief -> low oxytocin
        let has_grief = self.traumas.iter().any(|t|
            t.trauma_type == TraumaType::Grief && t.processing_level < 0.5
        );
        if has_grief {
            adj.oxytocin -= 0.01;
            adj.serotonin -= 0.01;
        }

        adj
    }

    /// Total unprocessed severity of traumas.
    pub fn unprocessed_severity(&self) -> f64 {
        self.traumas.iter()
            .map(|t| t.severity * (1.0 - t.processing_level))
            .sum::<f64>()
            .min(1.0)
    }

    /// Serializes for the API.
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
        trauma.processing_level = 0.9; // Nearly integrated
        ptsd.traumas.push(trauma);
        assert!(!ptsd.scan_for_triggers("la voiture rouge"));
    }
}
