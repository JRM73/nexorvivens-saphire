// =============================================================================
// care/surgery.rs — Surgery and operations
// =============================================================================
//
// Role: Models surgical interventions with their phases:
//        pre-operative (anxiety), operation (anesthesia), post-operative
//        (pain + recovery). Each phase has a distinct chemical impact.
//
// Integration:
//   Affects physiology (energy, pain, immunity).
//   Interacts with diseases (P2.3) as curative treatment.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Phase of a surgical operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SurgeryPhase {
    /// Waiting before the operation (anxiety, preparation)
    PreOp,
    /// Under anesthesia (unconscious, no pain)
    UnderAnesthesia,
    /// Immediate post-operative awakening (pain, confusion)
    PostOpImmediate,
    /// Convalescence (progressive healing)
    Recovery,
    /// Completely recovered
    Recovered,
}

/// A surgical operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Surgery {
    /// Operation description
    pub description: String,
    /// Treated condition (free text id)
    pub target_condition: String,
    /// Current phase
    pub phase: SurgeryPhase,
    /// Operation severity (0.0 = minor, 1.0 = major)
    pub severity: f64,
    /// Cycles in the current phase
    pub cycles_in_phase: u64,
    /// Operation success (determined upon leaving anesthesia)
    pub success: bool,
    /// Post-op recovery progress (0.0-1.0)
    pub recovery_progress: f64,
    /// Post-op pain level (decreases with healing)
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

    /// Advances the operation by one cycle.
    pub fn tick(&mut self) {
        self.cycles_in_phase += 1;

        match self.phase {
            SurgeryPhase::PreOp => {
                // 20 cycles of preparation/waiting
                if self.cycles_in_phase >= 20 {
                    self.phase = SurgeryPhase::UnderAnesthesia;
                    self.cycles_in_phase = 0;
                }
            }
            SurgeryPhase::UnderAnesthesia => {
                // Duration proportional to severity
                let op_duration = (self.severity * 30.0) as u64 + 10;
                if self.cycles_in_phase >= op_duration {
                    self.phase = SurgeryPhase::PostOpImmediate;
                    self.cycles_in_phase = 0;
                    self.pain_level = self.severity * 0.8;
                }
            }
            SurgeryPhase::PostOpImmediate => {
                // 30 cycles of painful waking
                self.pain_level = (self.pain_level - 0.005).max(self.severity * 0.3);
                if self.cycles_in_phase >= 30 {
                    self.phase = SurgeryPhase::Recovery;
                    self.cycles_in_phase = 0;
                }
            }
            SurgeryPhase::Recovery => {
                // Progressive healing
                let recovery_rate = 0.002 * (1.0 - self.severity * 0.5);
                self.recovery_progress = (self.recovery_progress + recovery_rate).min(1.0);
                self.pain_level = (self.pain_level - 0.002).max(0.0);

                if self.recovery_progress >= 1.0 {
                    self.phase = SurgeryPhase::Recovered;
                    self.pain_level = 0.0;
                }
            }
            SurgeryPhase::Recovered => {
                // Nothing to do
            }
        }
    }

    /// Chemical impact according to the current phase.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        match self.phase {
            SurgeryPhase::PreOp => ChemistryAdjustment {
                cortisol: 0.02 * self.severity,
                adrenaline: 0.01 * self.severity,
                noradrenaline: 0.01,
                ..Default::default()
            },
            SurgeryPhase::UnderAnesthesia => ChemistryAdjustment {
                // Anesthesia: everything is dampened
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
                serotonin: 0.005, // Relief
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

/// Surgery manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurgeryManager {
    pub surgeries: Vec<Surgery>,
    /// History of completed operations
    pub history_count: u32,
}

impl SurgeryManager {
    pub fn new() -> Self {
        Self { surgeries: Vec::new(), history_count: 0 }
    }

    /// Schedules an operation.
    pub fn schedule(&mut self, surgery: Surgery) {
        self.surgeries.push(surgery);
    }

    /// Updates all operations.
    pub fn tick(&mut self) {
        for s in &mut self.surgeries {
            s.tick();
        }
        let completed = self.surgeries.iter().filter(|s| s.is_complete()).count();
        self.history_count += completed as u32;
        self.surgeries.retain(|s| !s.is_complete());
    }

    /// Total chemical impact.
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

    /// Total pain from ongoing operations.
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

        // Immediate post-op (30 cycles)
        for _ in 0..30 {
            s.tick();
        }
        assert_eq!(s.phase, SurgeryPhase::Recovery);
    }

    #[test]
    fn test_surgery_recovery() {
        let mut s = Surgery::new("Test", "test", 0.3);
        // Advance to the Recovery phase
        for _ in 0..100 {
            s.tick();
        }
        // Then wait a long time
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
        // Run for a long time
        for _ in 0..2000 {
            mgr.tick();
        }
        assert_eq!(mgr.surgeries.len(), 0);
        assert!(mgr.history_count > 0);
    }
}
