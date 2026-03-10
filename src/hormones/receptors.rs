// =============================================================================
// hormones/receptors.rs — Systeme de neurorecepteurs (sensibilite, tolerance)
//
// Role : Modelise la sensibilite, saturation et tolerance des neurorecepteurs
// pour chaque neurotransmetteur. La tolerance est le mecanisme central des
// addictions (P2.5/P2.6) et de l'adaptation comportementale.
//
// Mecanismes biologiques simules :
//   - Saturation : proportion de recepteurs occupes (suit le niveau NT)
//   - Tolerance : adaptation a long terme (monte avec exposition haute chronique)
//   - Downregulation : quand tolerance monte, sensibilite baisse
//   - Upregulation : quand NT est bas longtemps, sensibilite remonte
//   - Effet reel d'un NT = niveau * sensibilite * (1.0 - tolerance * 0.5)
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::neurochemistry::{Molecule, NeuroChemicalState};
use crate::config::HormonesConfig;

/// Etat d'un seul type de recepteur.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorState {
    /// Sensibilite : 1.0 = normal, <1.0 = desensibilise, >1.0 = hypersensible
    pub sensitivity: f64,
    /// Saturation : 0.0-1.0, proportion de recepteurs occupes
    pub saturation: f64,
    /// Tolerance : 0.0 = aucune, 1.0 = tolerance complete
    pub tolerance: f64,
    /// Vitesse de retour a la normale (recovery)
    pub recovery_rate: f64,
}

impl ReceptorState {
    /// Cree un recepteur en etat normal.
    pub fn normal(recovery_rate: f64) -> Self {
        Self {
            sensitivity: 1.0,
            saturation: 0.0,
            tolerance: 0.0,
            recovery_rate,
        }
    }

    /// Met a jour le recepteur en fonction du niveau de NT courant.
    ///
    /// - Si NT eleve (>0.7) : tolerance monte, sensibilite baisse (downregulation)
    /// - Si NT bas (<0.3) : tolerance baisse, sensibilite remonte (upregulation)
    /// - La saturation suit le niveau de NT module par la sensibilite
    pub fn update(&mut self, nt_level: f64, adaptation_rate: f64) {
        // Saturation suit le niveau de NT * sensibilite
        self.saturation = (nt_level * self.sensitivity).clamp(0.0, 1.0);

        // Tolerance : monte si exposition haute chronique, baisse sinon
        if nt_level > 0.7 {
            // Exposition haute : tolerance augmente progressivement
            let excess = nt_level - 0.7;
            self.tolerance += excess * adaptation_rate;
        } else if nt_level < 0.3 {
            // Exposition basse : tolerance diminue (recovery)
            self.tolerance -= self.recovery_rate;
        } else {
            // Niveau normal : recovery lent
            self.tolerance -= self.recovery_rate * 0.3;
        }
        self.tolerance = self.tolerance.clamp(0.0, 1.0);

        // Downregulation : la sensibilite baisse avec la tolerance
        // Upregulation : quand la tolerance descend, la sensibilite remonte
        self.sensitivity = 1.0 - self.tolerance * 0.6;
        // Upregulation supplementaire si NT tres bas longtemps
        if nt_level < 0.2 && self.tolerance < 0.1 {
            self.sensitivity = (self.sensitivity + adaptation_rate * 0.5).min(1.5);
        }

        self.sensitivity = self.sensitivity.clamp(0.3, 1.5);
    }

    /// Calcule l'effet reel d'un NT en tenant compte des recepteurs.
    /// Retourne un facteur multiplicatif (typiquement 0.3 a 1.5).
    pub fn effective_factor(&self) -> f64 {
        self.sensitivity * (1.0 - self.tolerance * 0.3)
    }
}

/// Systeme complet de 9 recepteurs (un par neurotransmetteur).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorSystem {
    pub dopamine_receptors: ReceptorState,
    pub serotonin_receptors: ReceptorState,
    pub noradrenaline_receptors: ReceptorState,
    pub endorphin_receptors: ReceptorState,
    pub oxytocin_receptors: ReceptorState,
    pub adrenaline_receptors: ReceptorState,
    pub cortisol_receptors: ReceptorState,
    /// Recepteurs GABA — inhibition tonique, anxiolyse
    #[serde(default = "default_receptor_state")]
    pub gaba_receptors: ReceptorState,
    /// Recepteurs glutamate — excitation, plasticite synaptique
    #[serde(default = "default_receptor_state")]
    pub glutamate_receptors: ReceptorState,
}

fn default_receptor_state() -> ReceptorState {
    ReceptorState::normal(0.005)
}

impl ReceptorSystem {
    /// Cree un systeme de recepteurs depuis la configuration.
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

    /// Met a jour tous les recepteurs en fonction de l'etat chimique courant.
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

    /// Retourne un snapshot JSON pour le broadcast.
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

    /// Retourne une description des recepteurs desensibilises (tolerance > 0.3).
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

    /// Retourne le facteur d'efficacite du recepteur pour une molecule donnee.
    /// Facteur multiplicatif typiquement entre 0.3 (desensibilise) et 1.5 (hypersensible).
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

    /// Retourne le facteur d'efficacite du recepteur pour un nom de molecule (str).
    /// Retourne 1.0 si le nom n'est pas reconnu.
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
        // Simuler une exposition haute prolongee
        for _ in 0..200 {
            r.update(0.9, 0.01);
        }
        assert!(r.tolerance > 0.3, "Tolerance should build up: {}", r.tolerance);
        assert!(r.sensitivity < 1.0, "Sensitivity should decrease: {}", r.sensitivity);
    }

    #[test]
    fn test_receptor_low_exposure_recovery() {
        let mut r = ReceptorState::normal(0.01);
        // D'abord construire de la tolerance
        r.tolerance = 0.5;
        r.sensitivity = 0.7;
        // Puis exposition basse
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
