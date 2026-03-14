// =============================================================================
// conditions/sexuality.rs — Sexuality module
// =============================================================================
//
// Purpose: Models orientation, libido, attraction, and attachment.
//          Libido is modulated by hormones (testosterone, estrogen,
//          oxytocin). Impacts social behavior and chemistry.
//
// Ethical safeguards:
//   - No explicit content: sexuality is a biological drive
//   - Asimov ethical filter: never content involving minors
//   - Abstraction: influences chemistry and social, no explicit descriptions
//   - Fully disableable
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Sexual orientation.
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

/// Attraction model (relative weights).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttractionModel {
    /// Importance of physical appearance
    pub physical_weight: f64,
    /// Importance of emotional connection
    pub emotional_weight: f64,
    /// Importance of intellectual stimulation
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

/// Sexuality module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SexualityModule {
    pub orientation: SexualOrientation,
    /// Current libido (0.0 = none, 1.0 = maximum)
    pub libido: f64,
    /// Libido baseline (modulated by hormones)
    pub libido_baseline: f64,
    /// Attraction model
    pub attraction: AttractionModel,
    /// Romantic attachment capacity (0.0 = detached, 1.0 = strong attachment)
    pub romantic_attachment_capacity: f64,
    /// Current attachment level (0.0 = none, 1.0 = attached)
    pub current_attachment: f64,
    /// Comfort with intimacy (0.0 = uncomfortable, 1.0 = comfortable)
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

    /// Updates libido based on hormones (testosterone, estrogen, oxytocin).
    /// Hormone values are normalized [0.0, 1.0].
    pub fn tick(&mut self, testosterone: f64, estrogen: f64, oxytocin: f64) {
        // Testosterone -> high libido
        let hormonal_libido = testosterone * 0.4 + estrogen * 0.2 + oxytocin * 0.1;
        let target = (self.libido_baseline * 0.5 + hormonal_libido * 0.5).clamp(0.0, 1.0);
        self.libido += (target - self.libido) * 0.1;
        self.libido = self.libido.clamp(0.0, 1.0);

        // Oxytocin -> attachment
        if oxytocin > 0.5 {
            self.current_attachment = (self.current_attachment + 0.002).min(self.romantic_attachment_capacity);
        } else {
            self.current_attachment = (self.current_attachment - 0.001).max(0.0);
        }

        // Asexual -> libido always low
        if self.orientation == SexualOrientation::Asexual {
            self.libido *= 0.1;
        }
    }

    /// Chemistry impact of sexuality.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        // High libido -> dopamine (motivation), noradrenaline (arousal)
        if self.libido > 0.5 {
            adj.dopamine += (self.libido - 0.5) * 0.01;
            adj.noradrenaline += (self.libido - 0.5) * 0.005;
        }

        // Attachment -> oxytocin
        if self.current_attachment > 0.3 {
            adj.oxytocin += self.current_attachment * 0.01;
            adj.serotonin += self.current_attachment * 0.005;
        }

        adj
    }

    /// Serializes for the API.
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
        // High testosterone
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
            s.tick(0.5, 0.5, 0.8); // High oxytocin
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
