// =============================================================================
// hormones/receptors.rs — Neuroreceptor system (sensitivity, tolerance)
//
// Purpose: Models the sensitivity, saturation and tolerance of neuroreceptors
// for each neurotransmitter. Tolerance is the central mechanism behind
// addictions (P2.5/P2.6) and behavioral adaptation.
//
// Simulated biological mechanisms:
//   - Saturation: proportion of occupied receptors (follows NT level)
//   - Tolerance: long-term adaptation (increases with chronic high exposure)
//   - Downregulation: when tolerance rises, sensitivity decreases
//   - Upregulation: when NT is low for extended periods, sensitivity recovers
//   - Effective NT impact = level * sensitivity * (1.0 - tolerance * 0.5)
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::neurochemistry::{Molecule, NeuroChemicalState};
use crate::config::HormonesConfig;

/// State of a single receptor type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorState {
    /// Sensitivity: 1.0 = normal, <1.0 = desensitized, >1.0 = hypersensitive
    pub sensitivity: f64,
    /// Saturation: 0.0-1.0, proportion of occupied receptors
    pub saturation: f64,
    /// Tolerance: 0.0 = none, 1.0 = complete tolerance
    pub tolerance: f64,
    /// Rate of return to normal (recovery)
    pub recovery_rate: f64,
}

impl ReceptorState {
    /// Creates a receptor in normal state.
    pub fn normal(recovery_rate: f64) -> Self {
        Self {
            sensitivity: 1.0,
            saturation: 0.0,
            tolerance: 0.0,
            recovery_rate,
        }
    }

    /// Updates the receptor based on the current NT level.
    ///
    /// - If NT is high (>0.7): tolerance rises, sensitivity decreases (downregulation)
    /// - If NT is low (<0.3): tolerance decreases, sensitivity recovers (upregulation)
    /// - Saturation follows the NT level modulated by sensitivity
    pub fn update(&mut self, nt_level: f64, adaptation_rate: f64) {
        // Saturation follows NT level * sensitivity
        self.saturation = (nt_level * self.sensitivity).clamp(0.0, 1.0);

        // Tolerance: rises with chronic high exposure, decreases otherwise
        if nt_level > 0.7 {
            // High exposure: tolerance increases progressively
            let excess = nt_level - 0.7;
            self.tolerance += excess * adaptation_rate;
        } else if nt_level < 0.3 {
            // Low exposure: tolerance decreases (recovery)
            self.tolerance -= self.recovery_rate;
        } else {
            // Normal level: slow recovery
            self.tolerance -= self.recovery_rate * 0.3;
        }
        self.tolerance = self.tolerance.clamp(0.0, 1.0);

        // Downregulation: sensitivity decreases with tolerance
        // Upregulation: when tolerance drops, sensitivity recovers
        self.sensitivity = 1.0 - self.tolerance * 0.6;
        // Additional upregulation if NT is very low for extended periods
        if nt_level < 0.2 && self.tolerance < 0.1 {
            self.sensitivity = (self.sensitivity + adaptation_rate * 0.5).min(1.5);
        }

        self.sensitivity = self.sensitivity.clamp(0.3, 1.5);
    }

    /// Computes the effective impact of a NT accounting for receptors.
    /// Returns a multiplicative factor (typically 0.3 to 1.5).
    pub fn effective_factor(&self) -> f64 {
        self.sensitivity * (1.0 - self.tolerance * 0.3)
    }
}

/// Complete system of 9 receptors (one per neurotransmitter).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorSystem {
    pub dopamine_receptors: ReceptorState,
    pub serotonin_receptors: ReceptorState,
    pub noradrenaline_receptors: ReceptorState,
    pub endorphin_receptors: ReceptorState,
    pub oxytocin_receptors: ReceptorState,
    pub adrenaline_receptors: ReceptorState,
    pub cortisol_receptors: ReceptorState,
    /// GABA receptors — tonic inhibition, anxiolysis
    #[serde(default = "default_receptor_state")]
    pub gaba_receptors: ReceptorState,
    /// Glutamate receptors — excitation, synaptic plasticity
    #[serde(default = "default_receptor_state")]
    pub glutamate_receptors: ReceptorState,
}

fn default_receptor_state() -> ReceptorState {
    ReceptorState::normal(0.005)
}

impl ReceptorSystem {
    /// Creates a receptor system from configuration.
    pub fn new(config: &HormonesConfig) -> Self {
        let rr = config.receptor_recovery_rate;
        Self {
            dopamine_receptors: ReceptorState::normal(rr),
            serotonin_receptors: ReceptorState::normal(rr),
            noradrenaline_receptors: ReceptorState::normal(rr),
            endorphin_receptors: ReceptorState::normal(rr),
            oxytocin_receptors: ReceptorState::normal(rr),
            adrenaline_receptors: ReceptorState::normal(rr),
            cortisol_receptors: ReceptorState::normal(rr),
            gaba_receptors: ReceptorState::normal(rr),
            glutamate_receptors: ReceptorState::normal(rr),
        }
    }

    /// Updates all receptors based on the current chemical state.
    pub fn tick(&mut self, chemistry: &NeuroChemicalState, config: &HormonesConfig) {
        let rate = config.receptor_adaptation_rate;
        self.dopamine_receptors.update(chemistry.dopamine, rate);
        self.serotonin_receptors.update(chemistry.serotonin, rate);
        self.noradrenaline_receptors.update(chemistry.noradrenaline, rate);
        self.endorphin_receptors.update(chemistry.endorphin, rate);
        self.oxytocin_receptors.update(chemistry.oxytocin, rate);
        self.adrenaline_receptors.update(chemistry.adrenaline, rate);
        self.cortisol_receptors.update(chemistry.cortisol, rate);
        self.gaba_receptors.update(chemistry.gaba, rate);
        self.glutamate_receptors.update(chemistry.glutamate, rate);
    }

    /// Returns a JSON snapshot for broadcast.
    pub fn to_snapshot_json(&self) -> serde_json::Value {
        serde_json::json!({
            "dopamine": receptor_json(&self.dopamine_receptors),
            "serotonin": receptor_json(&self.serotonin_receptors),
            "noradrenaline": receptor_json(&self.noradrenaline_receptors),
            "endorphin": receptor_json(&self.endorphin_receptors),
            "oxytocin": receptor_json(&self.oxytocin_receptors),
            "adrenaline": receptor_json(&self.adrenaline_receptors),
            "cortisol": receptor_json(&self.cortisol_receptors),
            "gaba": receptor_json(&self.gaba_receptors),
            "glutamate": receptor_json(&self.glutamate_receptors),
        })
    }

    /// Returns a description of desensitized receptors (tolerance > 0.3).
    pub fn describe_desensitized(&self) -> String {
        let mut desensitized = Vec::new();
        if self.dopamine_receptors.tolerance > 0.3 {
            desensitized.push(format!("dopamine ({:.0}%)", self.dopamine_receptors.tolerance * 100.0));
        }
        if self.serotonin_receptors.tolerance > 0.3 {
            desensitized.push(format!("serotonine ({:.0}%)", self.serotonin_receptors.tolerance * 100.0));
        }
        if self.noradrenaline_receptors.tolerance > 0.3 {
            desensitized.push(format!("noradrenaline ({:.0}%)", self.noradrenaline_receptors.tolerance * 100.0));
        }
        if self.endorphin_receptors.tolerance > 0.3 {
            desensitized.push(format!("endorphine ({:.0}%)", self.endorphin_receptors.tolerance * 100.0));
        }
        if self.oxytocin_receptors.tolerance > 0.3 {
            desensitized.push(format!("ocytocine ({:.0}%)", self.oxytocin_receptors.tolerance * 100.0));
        }
        if self.adrenaline_receptors.tolerance > 0.3 {
            desensitized.push(format!("adrenaline ({:.0}%)", self.adrenaline_receptors.tolerance * 100.0));
        }
        if self.cortisol_receptors.tolerance > 0.3 {
            desensitized.push(format!("cortisol ({:.0}%)", self.cortisol_receptors.tolerance * 100.0));
        }
        if self.gaba_receptors.tolerance > 0.3 {
            desensitized.push(format!("GABA ({:.0}%)", self.gaba_receptors.tolerance * 100.0));
        }
        if self.glutamate_receptors.tolerance > 0.3 {
            desensitized.push(format!("glutamate ({:.0}%)", self.glutamate_receptors.tolerance * 100.0));
        }
        desensitized.join(", ")
    }

    /// Returns the receptor efficiency factor for a given molecule.
    /// Multiplicative factor typically between 0.3 (desensitized) and 1.5 (hypersensitive).
    pub fn factor_for(&self, molecule: Molecule) -> f64 {
        match molecule {
            Molecule::Dopamine => self.dopamine_receptors.effective_factor(),
            Molecule::Cortisol => self.cortisol_receptors.effective_factor(),
            Molecule::Serotonin => self.serotonin_receptors.effective_factor(),
            Molecule::Adrenaline => self.adrenaline_receptors.effective_factor(),
            Molecule::Oxytocin => self.oxytocin_receptors.effective_factor(),
            Molecule::Endorphin => self.endorphin_receptors.effective_factor(),
            Molecule::Noradrenaline => self.noradrenaline_receptors.effective_factor(),
            Molecule::Gaba => self.gaba_receptors.effective_factor(),
            Molecule::Glutamate => self.glutamate_receptors.effective_factor(),
        }
    }

    /// Returns the receptor efficiency factor for a molecule name (str).
    /// Returns 1.0 if the name is not recognized.
    pub fn factor_for_str(&self, name: &str) -> f64 {
        match name {
            "dopamine" => self.dopamine_receptors.effective_factor(),
            "cortisol" => self.cortisol_receptors.effective_factor(),
            "serotonin" | "serotonine" => self.serotonin_receptors.effective_factor(),
            "adrenaline" => self.adrenaline_receptors.effective_factor(),
            "oxytocin" | "ocytocine" => self.oxytocin_receptors.effective_factor(),
            "endorphin" | "endorphine" => self.endorphin_receptors.effective_factor(),
            "noradrenaline" => self.noradrenaline_receptors.effective_factor(),
            "gaba" => self.gaba_receptors.effective_factor(),
            "glutamate" => self.glutamate_receptors.effective_factor(),
            _ => 1.0,
        }
    }
}

impl Default for ReceptorSystem {
    fn default() -> Self {
        Self {
            dopamine_receptors: ReceptorState::normal(0.005),
            serotonin_receptors: ReceptorState::normal(0.005),
            noradrenaline_receptors: ReceptorState::normal(0.005),
            endorphin_receptors: ReceptorState::normal(0.005),
            oxytocin_receptors: ReceptorState::normal(0.005),
            adrenaline_receptors: ReceptorState::normal(0.005),
            cortisol_receptors: ReceptorState::normal(0.005),
            gaba_receptors: ReceptorState::normal(0.005),
            glutamate_receptors: ReceptorState::normal(0.005),
        }
    }
}

fn receptor_json(r: &ReceptorState) -> serde_json::Value {
    serde_json::json!({
        "sensitivity": r.sensitivity,
        "saturation": r.saturation,
        "tolerance": r.tolerance,
        "effective_factor": r.effective_factor(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receptor_normal_state() {
        let r = ReceptorState::normal(0.005);
        assert_eq!(r.sensitivity, 1.0);
        assert_eq!(r.tolerance, 0.0);
        assert_eq!(r.saturation, 0.0);
    }

    #[test]
    fn test_receptor_high_exposure_builds_tolerance() {
        let mut r = ReceptorState::normal(0.005);
        // Simulate prolonged high exposure
        for _ in 0..200 {
            r.update(0.9, 0.01);
        }
        assert!(r.tolerance > 0.3, "Tolerance should build up: {}", r.tolerance);
        assert!(r.sensitivity < 1.0, "Sensitivity should decrease: {}", r.sensitivity);
    }

    #[test]
    fn test_receptor_low_exposure_recovery() {
        let mut r = ReceptorState::normal(0.01);
        // First build up some tolerance
        r.tolerance = 0.5;
        r.sensitivity = 0.7;
        // Then apply low exposure
        for _ in 0..100 {
            r.update(0.1, 0.01);
        }
        assert!(r.tolerance < 0.5, "Tolerance should decrease: {}", r.tolerance);
    }

    #[test]
    fn test_effective_factor_range() {
        let mut r = ReceptorState::normal(0.005);
        assert!((r.effective_factor() - 1.0).abs() < 0.01);

        r.tolerance = 1.0;
        r.sensitivity = 0.4;
        let factor = r.effective_factor();
        assert!(factor > 0.0 && factor < 1.0, "Factor should be reduced: {}", factor);
    }
}
