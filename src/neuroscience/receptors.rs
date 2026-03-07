// =============================================================================
// receptors.rs — Modele pharmacologique des recepteurs neuronaux
// =============================================================================
//
// Role : Modelise les recepteurs synaptiques avec courbes dose-reponse
// sigmoidales (equation de Hill), adaptation (up/down regulation),
// et sous-types par molecule. Chaque molecule agit via ses recepteurs
// specifiques, et l'effet reel depend de la densite et de l'affinite.
//
// References scientifiques :
//   - Equation de Hill (1910) : response = Emax * [C]^n / (EC50^n + [C]^n)
//   - Up/down regulation : Stahl's Essential Psychopharmacology (2021)
//   - Sous-types : Goodman & Gilman's Pharmacological Basis (2023)
// =============================================================================

use serde::{Deserialize, Serialize};

/// Sous-type de recepteur pour une molecule donnee.
/// Chaque molecule peut avoir plusieurs sous-types avec des affinites
/// et effets differents (ex: D1 excitateur vs D2 inhibiteur pour la dopamine).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorSubtype {
    /// Nom du sous-type (ex: "D1", "5-HT1A", "alpha-1")
    pub name: String,
    /// EC50 : concentration produisant 50% de l'effet maximal [0.0-1.0]
    /// Plus bas = plus sensible (affinite elevee)
    pub ec50: f64,
    /// Coefficient de Hill : pente de la sigmoide
    /// 1.0 = hyperbolique (standard), >1 = cooperative (switch-like), <1 = graduel
    pub hill_coefficient: f64,
    /// Effet maximal normalise [-1.0, +1.0]
    /// Positif = excitateur, negatif = inhibiteur
    pub emax: f64,
    /// Densite de recepteurs [0.1, 2.0]
    /// 1.0 = normal, <1.0 = down-regulated, >1.0 = up-regulated
    pub density: f64,
    /// Tolerance accumulee [0.0, 1.0]
    /// Augmente avec exposition prolongee, reduit l'efficacite
    pub tolerance: f64,
}

impl ReceptorSubtype {
    /// Equation de Hill : reponse sigmoidale a une concentration donnee.
    /// Inclut la modulation par la densite et la tolerance.
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

    /// Adaptation des recepteurs a l'exposition prolongee.
    /// Exposition haute → down-regulation (densite baisse, tolerance monte)
    /// Exposition basse → up-regulation (densite monte, tolerance baisse)
    pub fn adapt(&mut self, concentration: f64, adaptation_rate: f64) {
        let deviation = concentration - self.ec50;
        if deviation > 0.1 {
            // Exposition elevee : down-regulation progressive
            self.density = (self.density - adaptation_rate * 0.5).max(0.3);
            self.tolerance = (self.tolerance + adaptation_rate * 0.3).min(0.8);
        } else if deviation < -0.1 {
            // Exposition faible : up-regulation (sensibilisation)
            self.density = (self.density + adaptation_rate * 0.3).min(2.0);
            self.tolerance = (self.tolerance - adaptation_rate * 0.5).max(0.0);
        } else {
            // Equilibre : retour lent vers la normale
            self.density += (1.0 - self.density) * adaptation_rate * 0.1;
            self.tolerance = (self.tolerance - adaptation_rate * 0.1).max(0.0);
        }
    }
}

/// Banque de recepteurs — contient tous les sous-types pour les 9 molecules.
/// Chaque molecule a 1 a 4 sous-types avec des profils pharmacologiques distincts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorBank {
    /// Recepteurs dopaminergiques (D1 excitateur, D2 inhibiteur)
    pub dopamine: Vec<ReceptorSubtype>,
    /// Recepteurs cortisoliques (GR glucocorticoide, MR mineralocorticoide)
    pub cortisol: Vec<ReceptorSubtype>,
    /// Recepteurs serotoninergiques (5-HT1A inhibiteur, 5-HT2A excitateur)
    pub serotonin: Vec<ReceptorSubtype>,
    /// Recepteurs adrenergiques (alpha-1 vasoconstricteur, beta-1 cardiaque)
    pub adrenaline: Vec<ReceptorSubtype>,
    /// Recepteurs ocytocinergiques (OTR unique, haute affinite)
    pub oxytocin: Vec<ReceptorSubtype>,
    /// Recepteurs opioides (mu analgesique, delta euphorisant)
    pub endorphin: Vec<ReceptorSubtype>,
    /// Recepteurs noradrenergiques (alpha-2 autorecepteur, beta-1 excitateur)
    pub noradrenaline: Vec<ReceptorSubtype>,
    /// Recepteurs GABAergiques (GABA-A rapide ionotrope, GABA-B lent metabotrope)
    pub gaba: Vec<ReceptorSubtype>,
    /// Recepteurs glutamatergiques (NMDA, AMPA)
    pub glutamate: Vec<ReceptorSubtype>,
    /// Vitesse d'adaptation des recepteurs (0.001 = lent, 0.01 = rapide)
    pub adaptation_rate: f64,
}

impl Default for ReceptorBank {
    fn default() -> Self {
        Self::new()
    }
}

impl ReceptorBank {
    /// Cree une banque de recepteurs avec les profils pharmacologiques par defaut.
    /// Basee sur les donnees de Stahl's Essential Psychopharmacology.
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

    /// Calcule la reponse effective totale d'une molecule via ses recepteurs.
    /// Somme ponderee des reponses de chaque sous-type.
    /// Retourne une valeur normalisee dans [-1.0, 1.0].
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

    /// Adaptation de tous les recepteurs a l'exposition courante.
    /// Appelee a chaque cycle cognitif pour simuler l'adaptation neuronale.
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

    /// Serialise l'etat des recepteurs pour persistance.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }

    /// Restaure l'etat des recepteurs depuis un JSON persiste.
    pub fn restore_from_json(json: &serde_json::Value) -> Option<Self> {
        serde_json::from_value(json.clone()).ok()
    }

    /// Resume l'etat de la banque de recepteurs pour le dashboard.
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

/// Resume des recepteurs pour le dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorSummary {
    pub molecules: Vec<ReceptorMoleculeSummary>,
}

/// Resume par molecule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorMoleculeSummary {
    pub molecule: String,
    pub subtypes: usize,
    pub avg_density: f64,
    pub avg_tolerance: f64,
}

/// Matrice d'interactions croisees entre molecules.
/// Modelise les effets modulateurs d'une molecule sur les autres.
/// Basee sur les donnees pharmacologiques connues.
#[derive(Debug, Clone)]
pub struct InteractionMatrix {
    /// Chaque entree : (source, cible, coefficient)
    /// Coefficient positif = potentialisation, negatif = inhibition
    pub interactions: Vec<(String, String, f64)>,
}

impl Default for InteractionMatrix {
    fn default() -> Self {
        Self::new()
    }
}

impl InteractionMatrix {
    /// Cree la matrice d'interactions basee sur la pharmacologie connue.
    pub fn new() -> Self {
        Self {
            interactions: vec![
                // Cortisol supprime dopamine (axe HPA → mesolimbique)
                ("cortisol".into(), "dopamine".into(), -0.12),
                // Cortisol reduit serotonine (depletion tryptophane)
                ("cortisol".into(), "serotonin".into(), -0.08),
                // GABA inhibe glutamate (equilibre excitation/inhibition)
                ("gaba".into(), "glutamate".into(), -0.15),
                // Glutamate excite le systeme (arousal global)
                ("glutamate".into(), "noradrenaline".into(), 0.08),
                ("glutamate".into(), "adrenaline".into(), 0.05),
                // GABA calme le systeme (anxiolytique)
                ("gaba".into(), "cortisol".into(), -0.10),
                ("gaba".into(), "adrenaline".into(), -0.08),
                // Endorphines tamponnent le cortisol (analgesie du stress)
                ("endorphin".into(), "cortisol".into(), -0.12),
                // Ocytocine tampon social du cortisol
                ("oxytocin".into(), "cortisol".into(), -0.08),
                // Dopamine et noradrenaline partagent une voie (tyrosine hydroxylase)
                ("dopamine".into(), "noradrenaline".into(), 0.04),
                // Serotonine freine la dopamine (frein 5-HT sur VTA)
                ("serotonin".into(), "dopamine".into(), -0.05),
                // Noradrenaline potentialise le glutamate (eveil cortical)
                ("noradrenaline".into(), "glutamate".into(), 0.06),
                // Adrenaline stimule le cortisol (boucle HPA)
                ("adrenaline".into(), "cortisol".into(), 0.06),
                // Glutamate facilite la dopamine (voie mesolimbique)
                ("glutamate".into(), "dopamine".into(), 0.05),
            ],
        }
    }

    /// Calcule les deltas d'interaction pour un etat chimique donne.
    /// Retourne un tableau de 9 deltas (un par molecule).
    /// Ordre : dopamine, cortisol, serotonin, adrenaline, oxytocin,
    ///         endorphin, noradrenaline, gaba, glutamate
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
                // L'effet depend de la concentration de la source
                // et est proportionnel au coefficient d'interaction
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
        // Exposition elevee pendant 100 cycles
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
