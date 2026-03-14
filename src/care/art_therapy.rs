// =============================================================================
// care/art_therapy.rs — Art therapie
// =============================================================================
//
// Role : Guerison par la creativite — ecriture, poesie, musique, dessin.
//        L'art therapie n'est pas un medicament mais un processus expressif :
//        dopamine via la creation, serotonine via la satisfaction,
//        cortisol reduit par l'immersion dans le flow creatif.
//
// Integration :
//   Stimule les sens (beaute/lecture, musicalite/ecoute).
//   Peut stimuler les graines emergentes (resonance emotionnelle, syntonie).
//   Synergie avec les passions creatives (P2.17).
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Type d'art therapie.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArtForm {
    /// Ecriture creative, journaling, poesie
    Writing,
    /// Musique (ecoute ou creation)
    Music,
    /// Dessin, peinture, sculpture
    VisualArt,
    /// Danse, mouvement corporel
    Movement,
}

/// Une session d'art therapie.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtTherapySession {
    pub art_form: ArtForm,
    /// Qualite de l'engagement (0.0 = distrait, 1.0 = flow total)
    pub engagement: f64,
    /// Cycles dans cette session
    pub cycles_active: u64,
    /// Niveau de flow atteint (0.0-1.0)
    pub flow_level: f64,
    /// Total de sessions effectuees
    pub total_sessions: u64,
    /// Benefice cumule sur le bien-etre (0.0-1.0)
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

    /// Demarre une session d'art therapie.
    pub fn start_session(&mut self) {
        self.cycles_active = 0;
        self.engagement = 0.3; // Debut modeste
        self.flow_level = 0.0;
        self.total_sessions += 1;
    }

    /// Met a jour la session (un cycle dans l'activite creative).
    pub fn tick(&mut self) {
        if self.engagement <= 0.0 {
            return;
        }

        self.cycles_active += 1;

        // L'engagement et le flow montent progressivement
        self.engagement = (self.engagement + 0.01).min(1.0);
        self.flow_level = if self.cycles_active > 10 {
            (self.flow_level + 0.02).min(1.0)
        } else {
            self.flow_level
        };

        // Benefice cumule grandit lentement avec la pratique reguliere
        self.cumulative_benefit = (self.cumulative_benefit + 0.0005).min(1.0);
    }

    /// Termine la session.
    pub fn end_session(&mut self) {
        self.engagement = 0.0;
        self.flow_level = 0.0;
    }

    /// Impact chimique pendant la session.
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
                noradrenaline: 0.005, // Concentration
                ..Default::default()
            },
            ArtForm::Movement => ChemistryAdjustment {
                endorphin: 0.015,
                dopamine: 0.008,
                cortisol: -0.01,
                adrenaline: 0.005,
                ..Default::default()
            },
        };

        // Amplifier par l'engagement et le flow
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

    /// Est-ce qu'une session est active ?
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

/// Gestionnaire d'art therapie (peut pratiquer plusieurs formes).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtTherapyManager {
    pub practices: Vec<ArtTherapySession>,
}

impl ArtTherapyManager {
    pub fn new() -> Self {
        Self { practices: Vec::new() }
    }

    /// Commence une session dans une forme d'art.
    pub fn start(&mut self, art_form: ArtForm) {
        if let Some(p) = self.practices.iter_mut().find(|p| p.art_form == art_form) {
            p.start_session();
        } else {
            let mut session = ArtTherapySession::new(art_form);
            session.start_session();
            self.practices.push(session);
        }
    }

    /// Met a jour toutes les sessions actives.
    pub fn tick(&mut self) {
        for p in &mut self.practices {
            if p.is_active() {
                p.tick();
            }
        }
    }

    /// Impact chimique total des sessions actives.
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

    /// Benefice cumule total de toutes les pratiques.
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
        // Pas active : pas d'influence
        let adj_inactive = session.chemistry_influence();
        assert!((adj_inactive.dopamine).abs() < 0.001);

        // Active
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
        mgr.start(ArtForm::Music); // Meme forme
        assert_eq!(mgr.practices.len(), 1);
        assert_eq!(mgr.practices[0].total_sessions, 2);
    }
}
