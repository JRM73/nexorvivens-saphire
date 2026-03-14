// =============================================================================
// conditions/motion_sickness.rs — Cinetose (mal des transports)
// =============================================================================
//
// Role : Modelise la cinetose : mal de l'air, mal de mer, vertige,
//        barotraumatisme. Provoque par un conflit sensoriel entre les sens.
//
// Mecanique :
//   - Mesure le conflit sensoriel (ecart entre les sens)
//   - Genere nausee si conflit > seuil * susceptibilite
//   - Impact chimique : cortisol +, confort --, concentration --
//   - Adaptation progressive (accoutumance) avec expositions repetees
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Type de mal des transports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MotionType {
    /// Mal de l'air (altitude, turbulences)
    Air,
    /// Mal de mer (mouvements rythmiques)
    Sea,
    /// Mal de terre (apres un long voyage)
    Land,
    /// Vertige (hauteur, rotation)
    Vertigo,
    /// Barotraumatisme (pression, profondeur)
    Barotrauma,
}

/// Etat de cinetose (mal des transports).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionSickness {
    /// Predisposition a la cinetose (0.0 = immunise, 1.0 = tres sensible)
    pub susceptibility: f64,
    /// Niveau de nausee actuel (0.0 = aucune, 1.0 = incapacitant)
    pub current_nausea: f64,
    /// Mesure du conflit sensoriel courant (0.0 = coherent, 1.0 = conflit total)
    pub sensory_conflict: f64,
    /// Niveau d'adaptation / accoutumance (0.0 = novice, 1.0 = agueri)
    pub adaptation: f64,
    /// Type de cinetose active (None si aucune)
    pub active_type: Option<MotionType>,
    /// Nombre total d'episodes
    pub total_episodes: u64,
}

impl MotionSickness {
    /// Cree un nouvel etat avec une susceptibilite donnee.
    pub fn new(susceptibility: f64) -> Self {
        Self {
            susceptibility: susceptibility.clamp(0.0, 1.0),
            current_nausea: 0.0,
            sensory_conflict: 0.0,
            adaptation: 0.0,
            active_type: None,
            total_episodes: 0,
        }
    }

    /// Evalue le conflit sensoriel a partir des intensites des 5 sens.
    ///
    /// Un conflit eleve se produit quand certains sens sont tres actifs
    /// et d'autres pas du tout (incoherence perceptuelle).
    pub fn evaluate_conflict(&mut self, sense_intensities: &[f64; 5]) {
        // Calcul de la variance des intensites
        let mean: f64 = sense_intensities.iter().sum::<f64>() / 5.0;
        let variance: f64 = sense_intensities.iter()
            .map(|&s| (s - mean).powi(2))
            .sum::<f64>() / 5.0;

        // Conflit = variance normalisee (max theorique ~0.25)
        self.sensory_conflict = (variance * 4.0).clamp(0.0, 1.0);
    }

    /// Declenche un episode de type specifique (via API ou scenario).
    pub fn trigger(&mut self, motion_type: MotionType) {
        self.active_type = Some(motion_type);
        self.sensory_conflict = 0.7; // Conflit artificiel eleve
        self.total_episodes += 1;
    }

    /// Met a jour la nausee en fonction du conflit et de la susceptibilite.
    ///
    /// Appele a chaque cycle quand la cinetose est activee.
    pub fn tick(&mut self) {
        // Nausee = conflit * susceptibilite * (1 - adaptation)
        let effective_susceptibility = self.susceptibility * (1.0 - self.adaptation * 0.7);
        let target_nausea = (self.sensory_conflict * effective_susceptibility).clamp(0.0, 1.0);

        // Convergence douce vers la cible
        self.current_nausea += (target_nausea - self.current_nausea) * 0.15;
        self.current_nausea = self.current_nausea.clamp(0.0, 1.0);

        // Adaptation progressive (exposition repetee → accoutumance)
        if self.sensory_conflict > 0.3 {
            self.adaptation = (self.adaptation + 0.001).min(1.0);
        }

        // Decroissance naturelle du conflit sensoriel
        self.sensory_conflict = (self.sensory_conflict - 0.02).max(0.0);

        // Si le conflit est resolu, la nausee disparait
        if self.sensory_conflict < 0.05 {
            self.active_type = None;
        }
    }

    /// Impact sur la chimie : cortisol +, serotonine -, endorphine +.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        if self.current_nausea < 0.1 {
            return ChemistryAdjustment::default();
        }

        ChemistryAdjustment {
            cortisol: self.current_nausea * 0.03,
            serotonin: -self.current_nausea * 0.02,
            endorphin: self.current_nausea * 0.01, // reponse analgesique
            adrenaline: if self.active_type == Some(MotionType::Vertigo) {
                self.current_nausea * 0.04 // vertige = panique
            } else {
                0.0
            },
            ..Default::default()
        }
    }

    /// Degradation cognitive due a la nausee [0.0 - 0.3].
    pub fn cognitive_impact(&self) -> f64 {
        if self.current_nausea > 0.7 {
            0.3 // Incapacitant
        } else if self.current_nausea > 0.3 {
            self.current_nausea * 0.2
        } else {
            0.0
        }
    }

    /// Serialise pour l'API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "susceptibility": self.susceptibility,
            "current_nausea": self.current_nausea,
            "sensory_conflict": self.sensory_conflict,
            "adaptation": self.adaptation,
            "active_type": self.active_type.as_ref().map(|t| format!("{:?}", t)),
            "total_episodes": self.total_episodes,
            "cognitive_impact": self.cognitive_impact(),
        })
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_conflict_no_nausea() {
        let mut ms = MotionSickness::new(0.8);
        // Tous les sens a la meme intensite = pas de conflit
        ms.evaluate_conflict(&[0.5, 0.5, 0.5, 0.5, 0.5]);
        ms.tick();
        assert!(ms.current_nausea < 0.01);
    }

    #[test]
    fn test_high_conflict_causes_nausea() {
        let mut ms = MotionSickness::new(0.8);
        // Grand ecart entre les sens
        ms.evaluate_conflict(&[1.0, 0.0, 1.0, 0.0, 0.5]);
        ms.tick();
        assert!(ms.current_nausea > 0.0);
        assert!(ms.sensory_conflict > 0.3);
    }

    #[test]
    fn test_low_susceptibility_resists() {
        let mut ms = MotionSickness::new(0.1);
        ms.evaluate_conflict(&[1.0, 0.0, 1.0, 0.0, 0.5]);
        ms.tick();
        // Meme conflit mais faible susceptibilite → nausee reduite
        assert!(ms.current_nausea < 0.1);
    }

    #[test]
    fn test_adaptation_reduces_nausea() {
        let mut ms = MotionSickness::new(0.8);
        ms.adaptation = 0.9; // Deja tres adapte
        ms.evaluate_conflict(&[1.0, 0.0, 1.0, 0.0, 0.5]);
        ms.tick();
        assert!(ms.current_nausea < 0.1);
    }

    #[test]
    fn test_trigger_episode() {
        let mut ms = MotionSickness::new(0.5);
        ms.trigger(MotionType::Vertigo);
        assert!(ms.sensory_conflict > 0.5);
        assert_eq!(ms.total_episodes, 1);
        ms.tick();
        assert!(ms.current_nausea > 0.0);
    }

    #[test]
    fn test_chemistry_influence_vertigo() {
        let mut ms = MotionSickness::new(0.8);
        ms.trigger(MotionType::Vertigo);
        // Plusieurs ticks pour que la nausee converge au-dessus du seuil
        for _ in 0..10 {
            ms.tick();
            // Re-injecter le conflit car il decroit
            ms.sensory_conflict = 0.7;
        }
        let adj = ms.chemistry_influence();
        assert!(adj.cortisol > 0.0);
        assert!(adj.adrenaline > 0.0); // Vertige = panique
    }
}
