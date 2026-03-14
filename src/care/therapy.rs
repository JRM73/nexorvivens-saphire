// =============================================================================
// care/therapy.rs — Psychological therapies
// =============================================================================
//
// Role: Models psychological therapies that treat traumas, phobias, addictions.
//  Each therapy has a type, duration, progressive efficacy,
//        and modifies chemistry and targeted conditions.
//
// Integration:
//   Accelerates the healing_rate of traumas (P2.10),
//  boosts phobia desensitization (P2.7),
//  assists with addiction withdrawal (P2.5).
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Type of psychological therapy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TherapyType {
    /// Cognitive-behavioral therapy (phobias, anxiety, OCD)
    Cbt,
    /// EMDR — trauma reprocessing through eye movements
    Emdr,
    /// Psychoanalysis — deep exploration of the unconscious
    Psychoanalysis,
    /// Therapeutic hypnosis (addictions, phobias, pain)
    Hypnotherapy,
}

/// An ongoing therapy session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TherapySession {
    pub therapy_type: TherapyType,
    /// Targeted condition (free text id: "trauma:grief", "phobia:claustro", etc.)
    pub target_condition: String,
    /// Cycles elapsed in this therapy
    pub cycles_elapsed: u64,
    /// Total expected duration (in cycles)
    pub total_duration: u64,
    /// Treatment progress (0.0 = start, 1.0 = completed)
    pub progress: f64,
    /// Efficacy of this therapy on this condition (0.0-1.0)
    pub efficacy: f64,
    /// Completed sessions
    pub sessions_completed: u32,
}

impl TherapySession {
    pub fn new(therapy_type: TherapyType, target: &str) -> Self {
        let (duration, efficacy) = match therapy_type {
            TherapyType::Cbt => (300, 0.7),          // Long, good efficacy
            TherapyType::Emdr => (150, 0.8),          // Medium, high efficacy on trauma
            TherapyType::Psychoanalysis => (600, 0.5), // Very long, deep but slow effect
            TherapyType::Hypnotherapy => (100, 0.6),   // Short, moderate efficacy
        };
        Self {
            therapy_type,
            target_condition: target.to_string(),
            cycles_elapsed: 0,
            total_duration: duration,
            progress: 0.0,
            efficacy,
            sessions_completed: 0,
        }
    }

    /// Advances the therapy by one cycle.
    /// Returns the healing bonus to apply to the targeted condition.
    pub fn tick(&mut self) -> f64 {
        if self.progress >= 1.0 {
            return 0.0;
        }

        self.cycles_elapsed += 1;
        self.progress = (self.cycles_elapsed as f64 / self.total_duration as f64).min(1.0);

        // The healing bonus increases with progress (the further along, the more it helps)
        let healing_bonus = self.efficacy * self.progress * 0.001;

        // Session milestone every 50 cycles
        if self.cycles_elapsed % 50 == 0 {
            self.sessions_completed += 1;
        }

        healing_bonus
    }

    /// Chemical impact of the therapy (during the session).
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        if self.progress >= 1.0 {
            return ChemistryAdjustment::default();
        }

        match self.therapy_type {
            TherapyType::Cbt => ChemistryAdjustment {
                cortisol: -0.005 * self.progress, // Progressive stress reduction
                serotonin: 0.003 * self.progress, // Well-being improvement
                ..Default::default()
            },
            TherapyType::Emdr => ChemistryAdjustment {
                // EMDR can be taxing at first then liberating
                cortisol: if self.progress < 0.3 { 0.005 } else { -0.008 },
                endorphin: 0.003 * self.progress,
                ..Default::default()
            },
            TherapyType::Psychoanalysis => ChemistryAdjustment {
                // Slow but deep — stirs things up
                cortisol: if self.progress < 0.5 { 0.002 } else { -0.003 },
                serotonin: 0.001 * self.progress,
                oxytocin: 0.002, // Therapeutic bond
                ..Default::default()
            },
            TherapyType::Hypnotherapy => ChemistryAdjustment {
                cortisol: -0.005,
                endorphin: 0.005,
                serotonin: 0.002,
                ..Default::default()
            },
        }
    }

    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": format!("{:?}", self.therapy_type),
            "target": self.target_condition,
            "progress": self.progress,
            "efficacy": self.efficacy,
            "sessions_completed": self.sessions_completed,
            "complete": self.is_complete(),
        })
    }
}

/// Manager for active therapies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TherapyManager {
    pub active_therapies: Vec<TherapySession>,
}

impl TherapyManager {
    pub fn new() -> Self {
        Self { active_therapies: Vec::new() }
    }

    /// Starts a new therapy.
    pub fn start(&mut self, therapy_type: TherapyType, target: &str) {
        self.active_therapies.push(TherapySession::new(therapy_type, target));
    }

    /// Updates all therapies.
    /// Returns healing bonuses per targeted condition.
    pub fn tick(&mut self) -> Vec<(String, f64)> {
        let mut bonuses = Vec::new();
        for therapy in &mut self.active_therapies {
            let bonus = therapy.tick();
            if bonus > 0.0 {
                bonuses.push((therapy.target_condition.clone(), bonus));
            }
        }
        // Remove completed therapies
        self.active_therapies.retain(|t| !t.is_complete());
        bonuses
    }

    /// Total chemical impact.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();
        for t in &self.active_therapies {
            let a = t.chemistry_influence();
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

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "active": self.active_therapies.iter().map(|t| t.to_json()).collect::<Vec<_>>(),
            "count": self.active_therapies.len(),
        })
    }
}

impl Default for TherapyManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_therapy_progresses() {
        let mut session = TherapySession::new(TherapyType::Cbt, "phobia:arachno");
        assert_eq!(session.progress, 0.0);
        for _ in 0..150 {
            session.tick();
        }
        assert!(session.progress > 0.4);
        assert!(!session.is_complete());
    }

    #[test]
    fn test_therapy_completes() {
        let mut session = TherapySession::new(TherapyType::Hypnotherapy, "addiction:nicotine");
        for _ in 0..200 {
            session.tick();
        }
        assert!(session.is_complete());
    }

    #[test]
    fn test_emdr_chemistry_phases() {
        let mut session = TherapySession::new(TherapyType::Emdr, "trauma:accident");
        // Start: positive cortisol (taxing)
        session.tick();
        let adj_early = session.chemistry_influence();
        assert!(adj_early.cortisol > 0.0);

        // After 50%: negative cortisol (liberating)
        for _ in 0..100 {
            session.tick();
        }
        let adj_late = session.chemistry_influence();
        assert!(adj_late.cortisol < 0.0);
    }

    #[test]
    fn test_manager_cleanup() {
        let mut mgr = TherapyManager::new();
        mgr.start(TherapyType::Hypnotherapy, "test");
        assert_eq!(mgr.active_therapies.len(), 1);
        for _ in 0..200 {
            mgr.tick();
        }
        assert_eq!(mgr.active_therapies.len(), 0);
    }

    #[test]
    fn test_healing_bonus_grows() {
        let mut session = TherapySession::new(TherapyType::Cbt, "phobia:test");
        let bonus_early = session.tick();
        for _ in 0..200 {
            session.tick();
        }
        let bonus_late = session.tick();
        assert!(bonus_late > bonus_early);
    }
}
