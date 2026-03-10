// =============================================================================
// grey_matter.rs — Substrat cerebral physique de Saphire
//
// Role : Simule la matiere grise (densite corticale), la myelinisation
// (vitesse de signal), la neuroplasticite (formation de connexions),
// la neurogenese (nouveaux neurones hippocampiques), la densite synaptique
// et le BDNF (Brain-Derived Neurotrophic Factor).
//
// Place dans l'architecture :
//   Pipeline cognitif etape 3p : tick + chemistry_influence.
//   Interactions croisees :
//     - tryptophane (nutrition) → BDNF
//     - densite synaptique → coherence biochamp (fields)
//     - matiere grise → amplitude brain_regions
//     - neuroplasticite → synaptogenese connectome
// =============================================================================

use serde::{Deserialize, Serialize};

use crate::config::GreyMatterConfig;
use crate::world::ChemistryAdjustment;

/// Systeme de matiere grise et substrat cerebral physique.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreyMatterSystem {
    pub enabled: bool,
    /// Volume/densite de matiere grise corticale (0-1). Affecte l'amplitude brain_regions.
    pub grey_matter_volume: f64,
    /// Niveau de myelinisation (0-1). Affecte la vitesse de traitement.
    pub myelination: f64,
    /// Neuroplasticite (0-1). Module la capacite de formation de connexions.
    pub neuroplasticity: f64,
    /// Taux de neurogenese hippocampique (0-1). Boostee par BDNF.
    pub neurogenesis_rate: f64,
    /// Densite synaptique (connexions par neurone, 0-1). Module phi/conscience.
    pub synaptic_density: f64,
    /// Niveau de BDNF (facteur neurotrophique) — derive de serotonine, exercice, apprentissage.
    pub bdnf_level: f64,
}

impl GreyMatterSystem {
    /// Cree un nouveau systeme depuis la config.
    pub fn new(config: &GreyMatterConfig) -> Self {
        Self {
            enabled: config.enabled,
            grey_matter_volume: 0.7,
            myelination: 0.6,
            neuroplasticity: 0.65,
            neurogenesis_rate: 0.5,
            synaptic_density: 0.6,
            bdnf_level: 0.5,
        }
    }

    /// Tick biologique : met a jour tous les parametres cerebraux.
    ///
    /// Parametres d'entree depuis d'autres systemes :
    /// - `cortisol`, `serotonin`, `dopamine` : chimie actuelle
    /// - `soma_energy` : ATP reserves (nutrition)
    /// - `is_sleeping` : en phase de sommeil
    /// - `is_learning` : activite d'apprentissage detectee
    /// - `tryptophan` : acide amine precurseur (nutrition → BDNF)
    /// - `novelty_detected` : stimulus novel detecte (novelty > 0.7)
    pub fn tick(
        &mut self,
        config: &GreyMatterConfig,
        cortisol: f64,
        serotonin: f64,
        dopamine: f64,
        soma_energy: f64,
        is_sleeping: bool,
        is_learning: bool,
        tryptophan: f64,
        novelty_detected: bool,
    ) {
        if !self.enabled { return; }

        // ─── BDNF ───────────────────────────────────────────────────────────
        // Facteur neurotrophique derive de multiples signaux biologiques.
        // Base : serotonine + energie + apprentissage + tryptophane
        let learning_factor = if is_learning { 0.2 } else { 0.0 };
        let mut raw_bdnf = serotonin * 0.3 + soma_energy * 0.3 + learning_factor + tryptophan * 0.1;

        // Dopamine promotes BDNF (reward-driven neuroplasticity)
        raw_bdnf += dopamine * 0.15;

        // Novelty bonus : l'exploration stimule la production de BDNF
        if novelty_detected {
            raw_bdnf += 0.1;
        }

        // Flow state bonus : dopamine elevee + cortisol bas = etat optimal
        if dopamine > 0.6 && cortisol < 0.4 {
            raw_bdnf += 0.1;
        }

        // Cortisol penalty : le stress chronique inhibe le BDNF
        if cortisol > 0.6 {
            raw_bdnf -= (cortisol - 0.6) * 0.4;
        }

        // Lissage exponentiel
        self.bdnf_level = self.bdnf_level * 0.95 + raw_bdnf * 0.05;

        // Homeostasis toward baseline 0.5 : tirage lent vers l'equilibre
        self.bdnf_level += (0.5 - self.bdnf_level) * 0.01;

        self.bdnf_level = self.bdnf_level.clamp(0.0, 1.0);

        // ─── Neurogenese ────────────────────────────────────────────────────
        // BDNF boost, cortisol penalise, deficit de sommeil penalise
        let cortisol_penalty = if cortisol > 0.6 { (cortisol - 0.6) * 0.3 } else { 0.0 };
        let sleep_deficit = if !is_sleeping && soma_energy < 0.3 { 0.1 } else { 0.0 };
        let neurogenesis_target = (self.bdnf_level * 0.6 - cortisol_penalty - sleep_deficit).clamp(0.0, 1.0);
        self.neurogenesis_rate = self.neurogenesis_rate * 0.98 + neurogenesis_target * 0.02;

        // ─── Matiere grise ──────────────────────────────────────────────────
        // Croissance par apprentissage, decline naturel, boost BDNF
        let growth = if is_learning { config.growth_rate * 2.0 } else { config.growth_rate };
        let bdnf_bonus = if self.bdnf_level > config.bdnf_threshold {
            (self.bdnf_level - config.bdnf_threshold) * 0.0002
        } else { 0.0 };
        self.grey_matter_volume += growth + bdnf_bonus - config.decline_rate;
        self.grey_matter_volume = self.grey_matter_volume.clamp(0.1, 1.0);

        // ─── Myelinisation ──────────────────────────────────────────────────
        // Croissance lente (avec l'experience), decline tres lent
        let myelin_growth = config.myelination_growth;
        self.myelination += myelin_growth - config.decline_rate * 0.5;
        self.myelination = self.myelination.clamp(0.1, 1.0);

        // ─── Neuroplasticite ────────────────────────────────────────────────
        // Base + dopamine (motivation) + BDNF - cortisol (rigidite)
        let plasticity_target = 0.3 + dopamine * 0.4 + self.bdnf_level * 0.2 - cortisol * 0.3;
        self.neuroplasticity = self.neuroplasticity * 0.97 + plasticity_target.clamp(0.0, 1.0) * 0.03;

        // ─── Densite synaptique ─────────────────────────────────────────────
        // Pruning pendant le sommeil, croissance pendant l'eveil
        if is_sleeping {
            // Pruning synaptique (SHY — Synaptic Homeostasis Hypothesis)
            let overshoot = (self.synaptic_density - config.optimal_synaptic_density).max(0.0);
            self.synaptic_density -= overshoot * config.pruning_rate;
        } else {
            // Croissance synaptique pendant l'eveil (Hebbian)
            let growth_rate = 0.0003 * self.neuroplasticity;
            self.synaptic_density += growth_rate;
        }
        self.synaptic_density = self.synaptic_density.clamp(0.1, 1.0);
    }

    /// Influence de la matiere grise sur la neurochimie.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();
        if !self.enabled { return adj; }

        // Matiere grise basse → stress (cortisol +)
        if self.grey_matter_volume < 0.3 {
            adj.cortisol += (0.3 - self.grey_matter_volume) * 0.03;
        }

        // Neurogenese elevee → endorphine + (bien-etre neuronal)
        if self.neurogenesis_rate > 0.6 {
            adj.endorphin += (self.neurogenesis_rate - 0.6) * 0.02;
        }

        // Densite synaptique trop elevee → surcharge → cortisol +
        if self.synaptic_density > 0.85 {
            adj.cortisol += (self.synaptic_density - 0.85) * 0.02;
        }

        adj
    }

    /// Serialise l'etat en JSON pour persistance.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "grey_matter_volume": self.grey_matter_volume,
            "myelination": self.myelination,
            "neuroplasticity": self.neuroplasticity,
            "neurogenesis_rate": self.neurogenesis_rate,
            "synaptic_density": self.synaptic_density,
            "bdnf_level": self.bdnf_level,
        })
    }

    /// Restaure l'etat depuis le JSON persiste.
    pub fn restore_from_json(&mut self, v: &serde_json::Value) {
        self.grey_matter_volume = v["grey_matter_volume"].as_f64().unwrap_or(self.grey_matter_volume);
        self.myelination = v["myelination"].as_f64().unwrap_or(self.myelination);
        self.neuroplasticity = v["neuroplasticity"].as_f64().unwrap_or(self.neuroplasticity);
        self.neurogenesis_rate = v["neurogenesis_rate"].as_f64().unwrap_or(self.neurogenesis_rate);
        self.synaptic_density = v["synaptic_density"].as_f64().unwrap_or(self.synaptic_density);
        self.bdnf_level = v["bdnf_level"].as_f64().unwrap_or(self.bdnf_level);
    }
}
