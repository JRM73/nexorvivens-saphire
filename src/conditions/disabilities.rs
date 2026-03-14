// =============================================================================
// conditions/disabilities.rs — Handicaps
// =============================================================================
//
// Role : Modelise les handicaps (cecite, surdite, paraplegique, brule).
//        Affecte les sens (reduction/suppression), declenche la compensation
//        sensorielle, et impacte l'identite.
//
// Integration :
//   Modifie les intensites des sens dans le Sensorium.
//   Les sens restants sont renforces (compensation).
//   L'adaptation progresse avec le temps.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Type de handicap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DisabilityType {
    /// Perte de vue (affecte ReadingSense)
    Blind,
    /// Perte d'ouie (affecte ListeningSense)
    Deaf,
    /// Paralysie des membres inferieurs
    Paraplegic,
    /// Grand brule (affecte ContactSense, douleur chronique)
    BurnSurvivor,
    /// Muet (affecte la parole)
    Mute,
}

/// Origine du handicap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DisabilityOrigin {
    /// Ne avec
    Congenital,
    /// Acquis pendant la vie
    Acquired,
}

/// Un handicap individuel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disability {
    pub disability_type: DisabilityType,
    pub origin: DisabilityOrigin,
    /// Severite (0.0 = leger, 1.0 = total)
    pub severity: f64,
    /// Adaptation au handicap (0.0 = debut, 1.0 = pleinement adapte)
    pub adapted: f64,
    /// Cycle d'acquisition (si acquis)
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

    /// Progression de l'adaptation.
    pub fn tick(&mut self, adaptation_rate: f64) {
        self.adapted = (self.adapted + adaptation_rate).min(1.0);
    }
}

/// Gestionnaire de handicaps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisabilityManager {
    pub disabilities: Vec<Disability>,
    /// Facteur de compensation sensorielle (ex: 1.3 = +30% pour les sens restants)
    pub compensation_factor: f64,
    /// Taux d'adaptation par cycle
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

    /// Met a jour l'adaptation de tous les handicaps.
    pub fn tick(&mut self) {
        let rate = self.adaptation_rate;
        for d in &mut self.disabilities {
            d.tick(rate);
        }
    }

    /// Facteur de reduction pour un sens donne (0.0 = supprime, 1.0 = normal).
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

    /// Facteur de compensation pour les sens NON affectes.
    pub fn compensation_for(&self, sense_name: &str) -> f64 {
        if self.disabilities.is_empty() {
            return 1.0;
        }

        // Si ce sens n'est pas affecte, il est renforce
        let this_factor = self.sense_factor(sense_name);
        if this_factor >= 0.99 && self.has_any_sensory_disability() {
            // Compensation proportionnelle a l'adaptation moyenne
            let avg_adapted: f64 = self.disabilities.iter()
                .map(|d| d.adapted)
                .sum::<f64>() / self.disabilities.len() as f64;
            1.0 + (self.compensation_factor - 1.0) * avg_adapted
        } else {
            1.0
        }
    }

    /// Y a-t-il un handicap qui affecte un sens ?
    fn has_any_sensory_disability(&self) -> bool {
        self.disabilities.iter().any(|d| matches!(
            d.disability_type,
            DisabilityType::Blind | DisabilityType::Deaf | DisabilityType::BurnSurvivor
        ))
    }

    /// Douleur chronique (brulures).
    pub fn chronic_pain(&self) -> f64 {
        self.disabilities.iter()
            .filter(|d| d.disability_type == DisabilityType::BurnSurvivor)
            .map(|d| d.severity * (1.0 - d.adapted * 0.5)) // L'adaptation reduit la douleur
            .sum::<f64>()
            .min(1.0)
    }

    /// Serialise pour l'API.
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
        d.adapted = 1.0; // Pleinement adapte
        mgr.add(d);
        // L'ecoute doit etre renforcee
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
