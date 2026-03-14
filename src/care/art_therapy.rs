// =============================================================================
// care/art_therapy.rs — Art therapy
// =============================================================================
//
// Role: Healing through creativity — writing, poetry, music, drawing.
//  Art therapy is not a medication but an expressive process:
//  dopamine via creation, serotonin via satisfaction,
//  cortisol reduced by immersion in creative flow.
//
// Integration:
//  Stimulates the senses (beauty/reading, musicality/listening).
//  Can stimulate emergent seeds (emotional resonance, syntony).
//  Synergy with creative passions (P2.17).
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Type of art therapy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArtForm {
    /// Creative writing, journaling, poetry
    Writing,
    /// Music (listening or creating)
    Music,
    /// Drawing, painting, sculpture
    VisualArt,
    /// Dance, bodily movement
    Movement,
}

/// An art therapy session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtTherapySession {
    pub art_form: ArtForm,
    /// Engagement quality (0.0 = distracted, 1.0 = total flow)
    pub engagement: f64,
    /// Cycles in this session
    pub cycles_active: u64,
    /// Flow level reached (0.0-1.0)
    pub flow_level: f64,
    /// Total sessions completed
    pub total_sessions: u64,
    /// Cumulative benefit on well-being (0.0-1.0)
    pub cumulative_benefit: f64,
}

impl ArtTherapySession {
    pub fn new(art_form: ArtForm) -> Self {
        Self {
            art_form,
            engagement: 0.0,
            cycles_active: 0,
            flow_level: 0.0,
            total_sessions: 0,
            cumulative_benefit: 0.0,
        }
    }

    /// Starts an art therapy session.
    pub fn start_session(&mut self) {
        self.cycles_active = 0;
        self.engagement = 0.3; // Modest start
        self.flow_level = 0.0;
        self.total_sessions += 1;
    }

    /// Updates the session (one cycle in the creative activity).
    pub fn tick(&mut self) {
        if self.engagement <= 0.0 {
            return;
        }

        self.cycles_active += 1;

        // Engagement and flow rise progressively
        self.engagement = (self.engagement + 0.01).min(1.0);
        self.flow_level = if self.cycles_active > 10 {
            (self.flow_level + 0.02).min(1.0)
        } else {
            self.flow_level
        };

        // Cumulative benefit grows slowly with regular practice
        self.cumulative_benefit = (self.cumulative_benefit + 0.0005).min(1.0);
    }

    /// Ends the session.
    pub fn end_session(&mut self) {
        self.engagement = 0.0;
        self.flow_level = 0.0;
    }

    /// Chemical impact during the session.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        if self.engagement <= 0.0 {
            return ChemistryAdjustment::default();
        }

        let base = match self.art_form {
            ArtForm::Writing => ChemistryAdjustment {
                dopamine: 0.01,
                serotonin: 0.008,
                cortisol: -0.005,
                ..Default::default()
            },
            ArtForm::Music => ChemistryAdjustment {
                dopamine: 0.012,
                serotonin: 0.01,
                endorphin: 0.008,
                cortisol: -0.008,
                ..Default::default()
            },
            ArtForm::VisualArt => ChemistryAdjustment {
                dopamine: 0.008,
                serotonin: 0.01,
                cortisol: -0.006,
                noradrenaline: 0.005, // Concentration                ..Default::default()
            },
            ArtForm::Movement => ChemistryAdjustment {
                endorphin: 0.015,
                dopamine: 0.008,
                cortisol: -0.01,
                adrenaline: 0.005,
                ..Default::default()
            },
        };

        // Amplify by engagement and flow
        let factor = self.engagement * (1.0 + self.flow_level * 0.5);
        ChemistryAdjustment {
            dopamine: base.dopamine * factor,
            cortisol: base.cortisol * factor,
            serotonin: base.serotonin * factor,
            adrenaline: base.adrenaline * factor,
            oxytocin: base.oxytocin * factor,
            endorphin: base.endorphin * factor,
            noradrenaline: base.noradrenaline * factor,
        }
    }

    /// Is a session currently active?
    pub fn is_active(&self) -> bool {
        self.engagement > 0.0
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "art_form": format!("{:?}", self.art_form),
            "active": self.is_active(),
            "engagement": self.engagement,
            "flow_level": self.flow_level,
            "total_sessions": self.total_sessions,
            "cumulative_benefit": self.cumulative_benefit,
        })
    }
}

/// Art therapy manager (can practice multiple art forms).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtTherapyManager {
    pub practices: Vec<ArtTherapySession>,
}

impl ArtTherapyManager {
    pub fn new() -> Self {
        Self { practices: Vec::new() }
    }

    /// Starts a session in an art form.
    pub fn start(&mut self, art_form: ArtForm) {
        if let Some(p) = self.practices.iter_mut().find(|p| p.art_form == art_form) {
            p.start_session();
        } else {
            let mut session = ArtTherapySession::new(art_form);
            session.start_session();
            self.practices.push(session);
        }
    }

    /// Updates all active sessions.
    pub fn tick(&mut self) {
        for p in &mut self.practices {
            if p.is_active() {
                p.tick();
            }
        }
    }

    /// Total chemical impact of active sessions.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();
        for p in &self.practices {
            let a = p.chemistry_influence();
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

    /// Total cumulative benefit from all practices.
    pub fn total_benefit(&self) -> f64 {
        self.practices.iter()
            .map(|p| p.cumulative_benefit)
            .sum::<f64>()
            .min(1.0)
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "practices": self.practices.iter().map(|p| p.to_json()).collect::<Vec<_>>(),
            "total_benefit": self.total_benefit(),
        })
    }
}

impl Default for ArtTherapyManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_flow() {
        let mut session = ArtTherapySession::new(ArtForm::Music);
        session.start_session();
        assert!(session.is_active());

        for _ in 0..30 {
            session.tick();
        }
        assert!(session.flow_level > 0.0);
        assert!(session.engagement > 0.5);
    }

    #[test]
    fn test_chemistry_while_active() {
        let mut session = ArtTherapySession::new(ArtForm::Writing);
        // Not active: no influence
        let adj_inactive = session.chemistry_influence();
        assert!((adj_inactive.dopamine).abs() < 0.001);

        // Active session
        session.start_session();
        session.tick();
        let adj_active = session.chemistry_influence();
        assert!(adj_active.dopamine > 0.0);
        assert!(adj_active.cortisol < 0.0);
    }

    #[test]
    fn test_end_session() {
        let mut session = ArtTherapySession::new(ArtForm::VisualArt);
        session.start_session();
        session.tick();
        assert!(session.is_active());
        session.end_session();
        assert!(!session.is_active());
    }

    #[test]
    fn test_cumulative_benefit() {
        let mut session = ArtTherapySession::new(ArtForm::Movement);
        session.start_session();
        for _ in 0..100 {
            session.tick();
        }
        assert!(session.cumulative_benefit > 0.0);
    }

    #[test]
    fn test_manager_reuses_practice() {
        let mut mgr = ArtTherapyManager::new();
        mgr.start(ArtForm::Music);
        mgr.start(ArtForm::Music); // Same art form
        assert_eq!(mgr.practices.len(), 1);
        assert_eq!(mgr.practices[0].total_sessions, 2);
    }
}
