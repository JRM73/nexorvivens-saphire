// =============================================================================
// conditions/sexuality.rs — Module sexualite
// =============================================================================
//
// Role : Modelise l'orientation, la libido, l'attraction, l'attachement.
//        La libido est modulee par les hormones (testosterone, oestrogene,
//        ocytocine). Impact sur le comportement social et la chimie.
//
// Gardes-fous ethiques :
//   - Pas de contenu explicite : la sexualite est un drive biologique
//   - Filtre ethique Asimov : jamais de contenu impliquant des mineurs
//   - Abstraction : influence chimie et social, pas descriptions explicites
//   - Desactivable completement
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Orientation sexuelle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SexualOrientation {
    Heterosexual,
    Homosexual,
    Bisexual,
    Asexual,
    Pansexual,
    Undefined,
}

impl Default for SexualOrientation {
    fn default() -> Self {
        Self::Undefined
    }
}

/// Modele d'attraction (poids relatifs).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttractionModel {
    /// Importance de l'apparence physique
    pub physical_weight: f64,
    /// Importance de la connexion emotionnelle
    pub emotional_weight: f64,
    /// Importance de la stimulation intellectuelle
    pub intellectual_weight: f64,
}

impl Default for AttractionModel {
    fn default() -> Self {
        Self {
            physical_weight: 0.3,
            emotional_weight: 0.4,
            intellectual_weight: 0.3,
        }
    }
}

/// Module de sexualite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SexualityModule {
    pub orientation: SexualOrientation,
    /// Libido actuelle (0.0 = nulle, 1.0 = maximale)
    pub libido: f64,
    /// Baseline de libido (modulee par hormones)
    pub libido_baseline: f64,
    /// Modele d'attraction
    pub attraction: AttractionModel,
    /// Capacite d'attachement romantique (0.0 = detache, 1.0 = attachement fort)
    pub romantic_attachment_capacity: f64,
    /// Niveau d'attachement actuel (0.0 = aucun, 1.0 = attache)
    pub current_attachment: f64,
    /// Confort avec l'intimite (0.0 = mal a l'aise, 1.0 = a l'aise)
    pub intimacy_comfort: f64,
}

impl SexualityModule {
    pub fn new(orientation: SexualOrientation, libido_baseline: f64, attachment_capacity: f64) -> Self {
        Self {
            orientation,
            libido: libido_baseline,
            libido_baseline: libido_baseline.clamp(0.0, 1.0),
            attraction: AttractionModel::default(),
            romantic_attachment_capacity: attachment_capacity.clamp(0.0, 1.0),
            current_attachment: 0.0,
            intimacy_comfort: 0.5,
        }
    }

    /// Met a jour la libido en fonction des hormones (testosterone, oestrogene, ocytocine).
    /// Les valeurs d'hormones sont normalisees [0.0, 1.0].
    pub fn tick(&mut self, testosterone: f64, estrogen: f64, oxytocin: f64) {
        // Testosterone → libido haute
        let hormonal_libido = testosterone * 0.4 + estrogen * 0.2 + oxytocin * 0.1;
        let target = (self.libido_baseline * 0.5 + hormonal_libido * 0.5).clamp(0.0, 1.0);
        self.libido += (target - self.libido) * 0.1;
        self.libido = self.libido.clamp(0.0, 1.0);

        // Ocytocine → attachement
        if oxytocin > 0.5 {
            self.current_attachment = (self.current_attachment + 0.002).min(self.romantic_attachment_capacity);
        } else {
            self.current_attachment = (self.current_attachment - 0.001).max(0.0);
        }

        // Asexuel → libido toujours basse
        if self.orientation == SexualOrientation::Asexual {
            self.libido *= 0.1;
        }
    }

    /// Impact chimique de la sexualite.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        // Libido haute → dopamine (motivation), noradrenaline (excitation)
        if self.libido > 0.5 {
            adj.dopamine += (self.libido - 0.5) * 0.01;
            adj.noradrenaline += (self.libido - 0.5) * 0.005;
        }

        // Attachement → ocytocine
        if self.current_attachment > 0.3 {
            adj.oxytocin += self.current_attachment * 0.01;
            adj.serotonin += self.current_attachment * 0.005;
        }

        adj
    }

    /// Serialise pour l'API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "orientation": format!("{:?}", self.orientation),
            "libido": self.libido,
            "libido_baseline": self.libido_baseline,
            "attraction": {
                "physical": self.attraction.physical_weight,
                "emotional": self.attraction.emotional_weight,
                "intellectual": self.attraction.intellectual_weight,
            },
            "romantic_attachment_capacity": self.romantic_attachment_capacity,
            "current_attachment": self.current_attachment,
            "intimacy_comfort": self.intimacy_comfort,
        })
    }
}

impl Default for SexualityModule {
    fn default() -> Self {
        Self::new(SexualOrientation::Undefined, 0.3, 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let s = SexualityModule::default();
        assert_eq!(s.orientation, SexualOrientation::Undefined);
        assert!(s.libido > 0.0);
        assert_eq!(s.current_attachment, 0.0);
    }

    #[test]
    fn test_testosterone_boosts_libido() {
        let mut s = SexualityModule::new(SexualOrientation::Heterosexual, 0.3, 0.5);
        let initial = s.libido;
        // Haute testosterone
        for _ in 0..50 {
            s.tick(0.9, 0.3, 0.3);
        }
        assert!(s.libido > initial);
    }

    #[test]
    fn test_asexual_low_libido() {
        let mut s = SexualityModule::new(SexualOrientation::Asexual, 0.5, 0.5);
        for _ in 0..20 {
            s.tick(0.9, 0.9, 0.9);
        }
        assert!(s.libido < 0.15);
    }

    #[test]
    fn test_oxytocin_builds_attachment() {
        let mut s = SexualityModule::new(SexualOrientation::Bisexual, 0.3, 0.8);
        for _ in 0..200 {
            s.tick(0.5, 0.5, 0.8); // Haute ocytocine
        }
        assert!(s.current_attachment > 0.2);
    }

    #[test]
    fn test_chemistry_high_libido() {
        let mut s = SexualityModule::default();
        s.libido = 0.8;
        let adj = s.chemistry_influence();
        assert!(adj.dopamine > 0.0);
    }

    #[test]
    fn test_attachment_capped_by_capacity() {
        let mut s = SexualityModule::new(SexualOrientation::Pansexual, 0.3, 0.4);
        for _ in 0..1000 {
            s.tick(0.5, 0.5, 0.9);
        }
        assert!(s.current_attachment <= s.romantic_attachment_capacity + 0.001);
    }
}
