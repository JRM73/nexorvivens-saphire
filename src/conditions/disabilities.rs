// =============================================================================
// conditions/disabilities.rs — Disabilities
// =============================================================================
//
// Purpose: Models disabilities (blindness, deafness, paraplegia, burn survivor).
//          Affects senses (reduction/suppression), triggers sensory
//          compensation, and impacts identity.
//
// Integration:
//   Modifies sense intensities in the Sensorium.
//   Remaining senses are strengthened (compensation).
//   Adaptation progresses over time.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Type of disability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DisabilityType {
    /// Vision loss (affects ReadingSense)
    Blind,
    /// Hearing loss (affects ListeningSense)
    Deaf,
    /// Lower limb paralysis
    Paraplegic,
    /// Severe burn survivor (affects ContactSense, chronic pain)
    BurnSurvivor,
    /// Mute (affects speech)
    Mute,
}

/// Origin of the disability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DisabilityOrigin {
    /// Born with it
    Congenital,
    /// Acquired during life
    Acquired,
}

/// An individual disability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disability {
    pub disability_type: DisabilityType,
    pub origin: DisabilityOrigin,
    /// Severity (0.0 = mild, 1.0 = total)
    pub severity: f64,
    /// Adaptation to the disability (0.0 = beginning, 1.0 = fully adapted)
    pub adapted: f64,
    /// Acquisition cycle (if acquired)
    pub acquired_at_cycle: Option<u64>,
}

impl Disability {
    pub fn new(dtype: DisabilityType, origin: DisabilityOrigin, severity: f64) -> Self {
        Self {
            disability_type: dtype,
            origin,
            severity: severity.clamp(0.0, 1.0),
            adapted: 0.0,
            acquired_at_cycle: None,
        }
    }

    /// Adaptation progression.
    pub fn tick(&mut self, adaptation_rate: f64) {
        self.adapted = (self.adapted + adaptation_rate).min(1.0);
    }
}

/// Disability manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisabilityManager {
    pub disabilities: Vec<Disability>,
    /// Sensory compensation factor (e.g., 1.3 = +30% for remaining senses)
    pub compensation_factor: f64,
    /// Adaptation rate per cycle
    pub adaptation_rate: f64,
}

impl DisabilityManager {
    pub fn new(adaptation_rate: f64, compensation_factor: f64) -> Self {
        Self {
            disabilities: Vec::new(),
            compensation_factor,
            adaptation_rate,
        }
    }

    pub fn add(&mut self, disability: Disability) {
        self.disabilities.push(disability);
    }

    /// Updates adaptation for all disabilities.
    pub fn tick(&mut self) {
        let rate = self.adaptation_rate;
        for d in &mut self.disabilities {
            d.tick(rate);
        }
    }

    /// Reduction factor for a given sense (0.0 = suppressed, 1.0 = normal).
    pub fn sense_factor(&self, sense_name: &str) -> f64 {
        let mut factor = 1.0;
        for d in &self.disabilities {
            let affects = match (&d.disability_type, sense_name) {
                (DisabilityType::Blind, "lecture" | "reading") => true,
                (DisabilityType::Deaf, "ecoute" | "listening") => true,
                (DisabilityType::BurnSurvivor, "contact" | "touch") => true,
                _ => false,
            };
            if affects {
                factor *= 1.0 - d.severity;
            }
        }
        factor.clamp(0.0, 1.0)
    }

    /// Compensation factor for NON-affected senses.
    pub fn compensation_for(&self, sense_name: &str) -> f64 {
        if self.disabilities.is_empty() {
            return 1.0;
        }

        // If this sense is not affected, it is strengthened
        let this_factor = self.sense_factor(sense_name);
        if this_factor >= 0.99 && self.has_any_sensory_disability() {
            // Compensation proportional to average adaptation
            let avg_adapted: f64 = self.disabilities.iter()
                .map(|d| d.adapted)
                .sum::<f64>() / self.disabilities.len() as f64;
            1.0 + (self.compensation_factor - 1.0) * avg_adapted
        } else {
            1.0
        }
    }

    /// Is there any disability affecting a sense?
    fn has_any_sensory_disability(&self) -> bool {
        self.disabilities.iter().any(|d| matches!(
            d.disability_type,
            DisabilityType::Blind | DisabilityType::Deaf | DisabilityType::BurnSurvivor
        ))
    }

    /// Chronic pain (burns).
    pub fn chronic_pain(&self) -> f64 {
        self.disabilities.iter()
            .filter(|d| d.disability_type == DisabilityType::BurnSurvivor)
            .map(|d| d.severity * (1.0 - d.adapted * 0.5)) // Adaptation reduces pain
            .sum::<f64>()
            .min(1.0)
    }

    /// Serializes for the API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "disabilities": self.disabilities.iter().map(|d| serde_json::json!({
                "type": format!("{:?}", d.disability_type),
                "origin": format!("{:?}", d.origin),
                "severity": d.severity,
                "adapted": d.adapted,
            })).collect::<Vec<_>>(),
            "chronic_pain": self.chronic_pain(),
            "compensation_factor": self.compensation_factor,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blind_reduces_reading() {
        let mut mgr = DisabilityManager::new(0.001, 1.3);
        mgr.add(Disability::new(DisabilityType::Blind, DisabilityOrigin::Congenital, 1.0));
        assert_eq!(mgr.sense_factor("lecture"), 0.0);
        assert_eq!(mgr.sense_factor("ecoute"), 1.0);
    }

    #[test]
    fn test_compensation_for_hearing() {
        let mut mgr = DisabilityManager::new(0.001, 1.3);
        let mut d = Disability::new(DisabilityType::Blind, DisabilityOrigin::Congenital, 1.0);
        d.adapted = 1.0; // Fully adapted
        mgr.add(d);
        // Hearing should be strengthened
        assert!(mgr.compensation_for("ecoute") > 1.0);
    }

    #[test]
    fn test_burn_chronic_pain() {
        let mut mgr = DisabilityManager::new(0.001, 1.3);
        mgr.add(Disability::new(DisabilityType::BurnSurvivor, DisabilityOrigin::Acquired, 0.7));
        assert!(mgr.chronic_pain() > 0.0);
    }

    #[test]
    fn test_adaptation_progresses() {
        let mut mgr = DisabilityManager::new(0.01, 1.3);
        mgr.add(Disability::new(DisabilityType::Deaf, DisabilityOrigin::Congenital, 1.0));
        let initial = mgr.disabilities[0].adapted;
        for _ in 0..100 {
            mgr.tick();
        }
        assert!(mgr.disabilities[0].adapted > initial);
    }
}
