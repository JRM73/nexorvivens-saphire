// =============================================================================
// receptors.rs — Pharmacological model of neuronal receptors
// =============================================================================
//
// Role: Models synaptic receptors with sigmoidal dose-response curves
// (Hill equation), adaptation (up/down regulation), and subtypes per
// molecule. Each molecule acts via its specific receptors, and the actual
// effect depends on density and affinity.
//
// Scientific references:
//   - Hill equation (1910): response = Emax * [C]^n / (EC50^n + [C]^n)
//   - Up/down regulation: Stahl's Essential Psychopharmacology (2021)
//   - Subtypes: Goodman & Gilman's Pharmacological Basis (2023)
// =============================================================================

use serde::{Deserialize, Serialize};

/// Receptor subtype for a given molecule.
/// Each molecule can have multiple subtypes with different affinities
/// and effects (e.g., D1 excitatory vs D2 inhibitory for dopamine).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorSubtype {
    /// Subtype name (e.g., "D1", "5-HT1A", "alpha-1")
    pub name: String,
    /// EC50: concentration producing 50% of maximal effect [0.0-1.0]
    /// Lower = more sensitive (high affinity)
    pub ec50: f64,
    /// Hill coefficient: slope of the sigmoid
    /// 1.0 = hyperbolic (standard), >1 = cooperative (switch-like), <1 = gradual
    pub hill_coefficient: f64,
    /// Normalized maximal effect [-1.0, +1.0]
    /// Positive = excitatory, negative = inhibitory
    pub emax: f64,
    /// Receptor density [0.1, 2.0]
    /// 1.0 = normal, <1.0 = down-regulated, >1.0 = up-regulated
    pub density: f64,
    /// Accumulated tolerance [0.0, 1.0]
    /// Increases with prolonged exposure, reduces efficacy
    pub tolerance: f64,
}

impl ReceptorSubtype {
    /// Hill equation: sigmoidal response to a given concentration.
    /// Includes modulation by density and tolerance.
    ///
    /// response = emax * density * (1 - tolerance) * C^n / (EC50^n + C^n)
    pub fn response(&self, concentration: f64) -> f64 {
        if concentration <= 0.0 || self.ec50 <= 0.0 {
            return 0.0;
        }
        let c_n = concentration.powf(self.hill_coefficient);
        let ec50_n = self.ec50.powf(self.hill_coefficient);
        let hill_response = c_n / (ec50_n + c_n);
        self.emax * self.density * (1.0 - self.tolerance * 0.5) * hill_response
    }

    /// Receptor adaptation to prolonged exposure.
    /// High exposure -> down-regulation (density decreases, tolerance increases)
    /// Low exposure -> up-regulation (density increases, tolerance decreases)
    pub fn adapt(&mut self, concentration: f64, adaptation_rate: f64) {
        let deviation = concentration - self.ec50;
        if deviation > 0.1 {
            // High exposure: progressive down-regulation
            self.density = (self.density - adaptation_rate * 0.5).max(0.3);
            self.tolerance = (self.tolerance + adaptation_rate * 0.3).min(0.8);
        } else if deviation < -0.1 {
            // Low exposure: up-regulation (sensitization)
            self.density = (self.density + adaptation_rate * 0.3).min(2.0);
            self.tolerance = (self.tolerance - adaptation_rate * 0.5).max(0.0);
        } else {
            // Equilibrium: slow return to baseline
            self.density += (1.0 - self.density) * adaptation_rate * 0.1;
            self.tolerance = (self.tolerance - adaptation_rate * 0.1).max(0.0);
        }
    }
}

/// Receptor bank — contains all subtypes for the 9 molecules.
/// Each molecule has 1 to 4 subtypes with distinct pharmacological profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorBank {
    /// Dopaminergic receptors (D1 excitatory, D2 inhibitory)
    pub dopamine: Vec<ReceptorSubtype>,
    /// Corticosteroid receptors (GR glucocorticoid, MR mineralocorticoid)
    pub cortisol: Vec<ReceptorSubtype>,
    /// Serotonergic receptors (5-HT1A inhibitory, 5-HT2A excitatory)
    pub serotonin: Vec<ReceptorSubtype>,
    /// Adrenergic receptors (alpha-1 vasoconstrictor, beta-1 cardiac)
    pub adrenaline: Vec<ReceptorSubtype>,
    /// Oxytocinergic receptors (OTR single, high affinity)
    pub oxytocin: Vec<ReceptorSubtype>,
    /// Opioid receptors (mu analgesic, delta euphoriant)
    pub endorphin: Vec<ReceptorSubtype>,
    /// Noradrenergic receptors (alpha-2 autoreceptor, beta-1 excitatory)
    pub noradrenaline: Vec<ReceptorSubtype>,
    /// GABAergic receptors (GABA-A fast ionotropic, GABA-B slow metabotropic)
    pub gaba: Vec<ReceptorSubtype>,
    /// Glutamatergic receptors (NMDA, AMPA)
    pub glutamate: Vec<ReceptorSubtype>,
    /// Receptor adaptation rate (0.001 = slow, 0.01 = fast)
    pub adaptation_rate: f64,
}

impl Default for ReceptorBank {
    fn default() -> Self {
        Self::new()
    }
}

impl ReceptorBank {
    /// Creates a receptor bank with default pharmacological profiles.
    /// Based on data from Stahl's Essential Psychopharmacology.
    pub fn new() -> Self {
        Self {
            dopamine: vec![
                ReceptorSubtype {
                    name: "D1".into(), ec50: 0.4, hill_coefficient: 1.2,
                    emax: 1.0, density: 1.0, tolerance: 0.0,
                },
                ReceptorSubtype {
                    name: "D2".into(), ec50: 0.3, hill_coefficient: 1.5,
                    emax: -0.6, density: 1.0, tolerance: 0.0,
                },
            ],
            cortisol: vec![
                ReceptorSubtype {
                    name: "MR".into(), ec50: 0.2, hill_coefficient: 1.0,
                    emax: 0.8, density: 1.0, tolerance: 0.0,
                },
                ReceptorSubtype {
                    name: "GR".into(), ec50: 0.6, hill_coefficient: 1.8,
                    emax: 1.0, density: 1.0, tolerance: 0.0,
                },
            ],
            serotonin: vec![
                ReceptorSubtype {
                    name: "5-HT1A".into(), ec50: 0.3, hill_coefficient: 1.0,
                    emax: -0.7, density: 1.0, tolerance: 0.0,
                },
                ReceptorSubtype {
                    name: "5-HT2A".into(), ec50: 0.5, hill_coefficient: 1.3,
                    emax: 0.8, density: 1.0, tolerance: 0.0,
                },
            ],
            adrenaline: vec![
                ReceptorSubtype {
                    name: "alpha-1".into(), ec50: 0.4, hill_coefficient: 1.0,
                    emax: 0.9, density: 1.0, tolerance: 0.0,
                },
                ReceptorSubtype {
                    name: "beta-1".into(), ec50: 0.3, hill_coefficient: 1.2,
                    emax: 1.0, density: 1.0, tolerance: 0.0,
                },
            ],
            oxytocin: vec![
                ReceptorSubtype {
                    name: "OTR".into(), ec50: 0.35, hill_coefficient: 1.0,
                    emax: 1.0, density: 1.0, tolerance: 0.0,
                },
            ],
            endorphin: vec![
                ReceptorSubtype {
                    name: "mu".into(), ec50: 0.3, hill_coefficient: 1.5,
                    emax: 1.0, density: 1.0, tolerance: 0.0,
                },
                ReceptorSubtype {
                    name: "delta".into(), ec50: 0.5, hill_coefficient: 1.0,
                    emax: 0.6, density: 1.0, tolerance: 0.0,
                },
            ],
            noradrenaline: vec![
                ReceptorSubtype {
                    name: "alpha-2".into(), ec50: 0.25, hill_coefficient: 1.2,
                    emax: -0.5, density: 1.0, tolerance: 0.0,
                },
                ReceptorSubtype {
                    name: "beta-1-NA".into(), ec50: 0.4, hill_coefficient: 1.0,
                    emax: 0.9, density: 1.0, tolerance: 0.0,
                },
            ],
            gaba: vec![
                ReceptorSubtype {
                    name: "GABA-A".into(), ec50: 0.35, hill_coefficient: 2.0,
                    emax: -1.0, density: 1.0, tolerance: 0.0,
                },
                ReceptorSubtype {
                    name: "GABA-B".into(), ec50: 0.5, hill_coefficient: 1.0,
                    emax: -0.6, density: 1.0, tolerance: 0.0,
                },
            ],
            glutamate: vec![
                ReceptorSubtype {
                    name: "AMPA".into(), ec50: 0.3, hill_coefficient: 1.5,
                    emax: 1.0, density: 1.0, tolerance: 0.0,
                },
                ReceptorSubtype {
                    name: "NMDA".into(), ec50: 0.5, hill_coefficient: 2.0,
                    emax: 0.8, density: 1.0, tolerance: 0.0,
                },
            ],
            adaptation_rate: 0.002,
        }
    }

    /// Computes the total effective response of a molecule via its receptors.
    /// Weighted sum of each subtype's response.
    /// Returns a normalized value in [-1.0, 1.0].
    pub fn effective_response(&self, molecule: &str, concentration: f64) -> f64 {
        let receptors = match molecule {
            "dopamine" => &self.dopamine,
            "cortisol" => &self.cortisol,
            "serotonin" => &self.serotonin,
            "adrenaline" => &self.adrenaline,
            "oxytocin" => &self.oxytocin,
            "endorphin" => &self.endorphin,
            "noradrenaline" => &self.noradrenaline,
            "gaba" => &self.gaba,
            "glutamate" => &self.glutamate,
            _ => return 0.0,
        };
        if receptors.is_empty() {
            return 0.0;
        }
        let total: f64 = receptors.iter().map(|r| r.response(concentration)).sum();
        (total / receptors.len() as f64).clamp(-1.0, 1.0)
    }

    /// Adapts all receptors to current exposure levels.
    /// Called each cognitive cycle to simulate neuronal adaptation.
    pub fn tick_adaptation(&mut self, chemistry: &crate::neurochemistry::NeuroChemicalState) {
        let rate = self.adaptation_rate;
        for r in &mut self.dopamine { r.adapt(chemistry.dopamine, rate); }
        for r in &mut self.cortisol { r.adapt(chemistry.cortisol, rate); }
        for r in &mut self.serotonin { r.adapt(chemistry.serotonin, rate); }
        for r in &mut self.adrenaline { r.adapt(chemistry.adrenaline, rate); }
        for r in &mut self.oxytocin { r.adapt(chemistry.oxytocin, rate); }
        for r in &mut self.endorphin { r.adapt(chemistry.endorphin, rate); }
        for r in &mut self.noradrenaline { r.adapt(chemistry.noradrenaline, rate); }
        for r in &mut self.gaba { r.adapt(chemistry.gaba, rate); }
        for r in &mut self.glutamate { r.adapt(chemistry.glutamate, rate); }
    }

    /// Serializes receptor state for persistence.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }

    /// Restores receptor state from persisted JSON.
    pub fn restore_from_json(json: &serde_json::Value) -> Option<Self> {
        serde_json::from_value(json.clone()).ok()
    }

    /// Summarizes receptor bank state for the dashboard.
    pub fn summary(&self) -> ReceptorSummary {
        let mut entries = Vec::new();
        let summarize = |name: &str, receptors: &[ReceptorSubtype]| -> ReceptorMoleculeSummary {
            let avg_density = if receptors.is_empty() { 1.0 }
                else { receptors.iter().map(|r| r.density).sum::<f64>() / receptors.len() as f64 };
            let avg_tolerance = if receptors.is_empty() { 0.0 }
                else { receptors.iter().map(|r| r.tolerance).sum::<f64>() / receptors.len() as f64 };
            ReceptorMoleculeSummary {
                molecule: name.to_string(),
                subtypes: receptors.len(),
                avg_density,
                avg_tolerance,
            }
        };
        entries.push(summarize("dopamine", &self.dopamine));
        entries.push(summarize("cortisol", &self.cortisol));
        entries.push(summarize("serotonin", &self.serotonin));
        entries.push(summarize("adrenaline", &self.adrenaline));
        entries.push(summarize("oxytocin", &self.oxytocin));
        entries.push(summarize("endorphin", &self.endorphin));
        entries.push(summarize("noradrenaline", &self.noradrenaline));
        entries.push(summarize("gaba", &self.gaba));
        entries.push(summarize("glutamate", &self.glutamate));
        ReceptorSummary { molecules: entries }
    }
}

/// Receptor summary for the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorSummary {
    pub molecules: Vec<ReceptorMoleculeSummary>,
}

/// Per-molecule summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorMoleculeSummary {
    pub molecule: String,
    pub subtypes: usize,
    pub avg_density: f64,
    pub avg_tolerance: f64,
}

/// Cross-interaction matrix between molecules.
/// Models the modulatory effects of one molecule on others.
/// Based on known pharmacological data.
#[derive(Debug, Clone)]
pub struct InteractionMatrix {
    /// Each entry: (source, target, coefficient)
    /// Positive coefficient = potentiation, negative = inhibition
    pub interactions: Vec<(String, String, f64)>,
}

impl Default for InteractionMatrix {
    fn default() -> Self {
        Self::new()
    }
}

impl InteractionMatrix {
    /// Creates the interaction matrix based on known pharmacology.
    pub fn new() -> Self {
        Self {
            interactions: vec![
                // Cortisol suppresses dopamine (HPA axis -> mesolimbic)
                ("cortisol".into(), "dopamine".into(), -0.12),
                // Cortisol reduces serotonin (tryptophan depletion)
                ("cortisol".into(), "serotonin".into(), -0.08),
                // GABA inhibits glutamate (excitation/inhibition balance)
                ("gaba".into(), "glutamate".into(), -0.15),
                // Glutamate excites the system (global arousal)
                ("glutamate".into(), "noradrenaline".into(), 0.08),
                ("glutamate".into(), "adrenaline".into(), 0.05),
                // GABA calms the system (anxiolytic)
                ("gaba".into(), "cortisol".into(), -0.10),
                ("gaba".into(), "adrenaline".into(), -0.08),
                // Endorphins buffer cortisol (stress analgesia)
                ("endorphin".into(), "cortisol".into(), -0.12),
                // Oxytocin provides social buffering of cortisol
                ("oxytocin".into(), "cortisol".into(), -0.08),
                // Dopamine and noradrenaline share a pathway (tyrosine hydroxylase)
                ("dopamine".into(), "noradrenaline".into(), 0.04),
                // Serotonin brakes dopamine (5-HT brake on VTA)
                ("serotonin".into(), "dopamine".into(), -0.05),
                // Noradrenaline potentiates glutamate (cortical arousal)
                ("noradrenaline".into(), "glutamate".into(), 0.06),
                // Adrenaline stimulates cortisol (HPA loop)
                ("adrenaline".into(), "cortisol".into(), 0.06),
                // Glutamate facilitates dopamine (mesolimbic pathway)
                ("glutamate".into(), "dopamine".into(), 0.05),
            ],
        }
    }

    /// Computes interaction deltas for a given chemical state.
    /// Returns an array of 9 deltas (one per molecule).
    /// Order: dopamine, cortisol, serotonin, adrenaline, oxytocin,
    ///        endorphin, noradrenaline, gaba, glutamate
    pub fn compute_deltas(&self, chemistry: &crate::neurochemistry::NeuroChemicalState) -> [f64; 9] {
        let mut deltas = [0.0f64; 9];
        let mol_index = |name: &str| -> Option<usize> {
            match name {
                "dopamine" => Some(0),
                "cortisol" => Some(1),
                "serotonin" => Some(2),
                "adrenaline" => Some(3),
                "oxytocin" => Some(4),
                "endorphin" => Some(5),
                "noradrenaline" => Some(6),
                "gaba" => Some(7),
                "glutamate" => Some(8),
                _ => None,
            }
        };
        let concentrations = chemistry.to_vec9();

        for (source, target, coeff) in &self.interactions {
            if let (Some(src_idx), Some(tgt_idx)) = (mol_index(source), mol_index(target)) {
                // The effect depends on the source concentration
                // and is proportional to the interaction coefficient
                let effect = concentrations[src_idx] * coeff;
                deltas[tgt_idx] += effect;
            }
        }
        deltas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hill_equation_at_ec50() {
        let receptor = ReceptorSubtype {
            name: "test".into(), ec50: 0.5, hill_coefficient: 1.0,
            emax: 1.0, density: 1.0, tolerance: 0.0,
        };
        let response = receptor.response(0.5);
        assert!((response - 0.5).abs() < 0.01, "A EC50, la reponse doit etre 50% de Emax");
    }

    #[test]
    fn test_hill_equation_saturation() {
        let receptor = ReceptorSubtype {
            name: "test".into(), ec50: 0.3, hill_coefficient: 2.0,
            emax: 1.0, density: 1.0, tolerance: 0.0,
        };
        let response_high = receptor.response(0.9);
        let response_low = receptor.response(0.1);
        assert!(response_high > response_low, "Concentration haute → reponse plus forte");
        assert!(response_high > 0.8, "Pres de saturation a haute concentration");
    }

    #[test]
    fn test_tolerance_reduces_response() {
        let mut receptor = ReceptorSubtype {
            name: "test".into(), ec50: 0.5, hill_coefficient: 1.0,
            emax: 1.0, density: 1.0, tolerance: 0.0,
        };
        let response_clean = receptor.response(0.5);
        receptor.tolerance = 0.5;
        let response_tolerant = receptor.response(0.5);
        assert!(response_tolerant < response_clean, "La tolerance reduit la reponse");
    }

    #[test]
    fn test_adaptation_down_regulation() {
        let mut receptor = ReceptorSubtype {
            name: "test".into(), ec50: 0.3, hill_coefficient: 1.0,
            emax: 1.0, density: 1.0, tolerance: 0.0,
        };
        // High exposure for 100 cycles
        for _ in 0..100 {
            receptor.adapt(0.9, 0.01);
        }
        assert!(receptor.density < 1.0, "Exposition prolongee → down-regulation");
        assert!(receptor.tolerance > 0.0, "Exposition prolongee → tolerance accrue");
    }

    #[test]
    fn test_bank_effective_response() {
        let bank = ReceptorBank::new();
        let response = bank.effective_response("dopamine", 0.5);
        assert!(response.abs() > 0.0, "Reponse non nulle a concentration non nulle");
        assert!(response >= -1.0 && response <= 1.0, "Reponse normalisee");
    }
}
