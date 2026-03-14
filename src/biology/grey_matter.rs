// =============================================================================
// grey_matter.rs — Saphire's physical brain substrate
//
// Role: Simulates grey matter (cortical density), myelination
// (signal speed), neuroplasticity (connection formation),
// neurogenesis (new hippocampal neurons), synaptic density
// and BDNF (Brain-Derived Neurotrophic Factor).
//
// Place in architecture:
//   Cognitive pipeline step 3p: tick + chemistry_influence.
//   Cross-interactions:
//   - Tryptophan (nutrition) → BDNF
//   - Synaptic density → biofield coherence (fields)
//   - Grey matter → brain_regions amplitude
//   - Neuroplasticity → connectome synaptogenesis
// =============================================================================

use serde::{Deserialize, Serialize};

use crate::config::GreyMatterConfig;
use crate::world::ChemistryAdjustment;

/// Grey matter and physical brain substrate system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreyMatterSystem {
    pub enabled: bool,
    /// Cortical grey matter volume/density (0-1). Affects brain_regions amplitude.
    pub grey_matter_volume: f64,
    /// Myelination level (0-1). Affects processing speed.
    pub myelination: f64,
    /// Neuroplasticity (0-1). Modulates the capacity for connection formation.
    pub neuroplasticity: f64,
    /// Hippocampal neurogenesis rate (0-1). Boosted by BDNF.
    pub neurogenesis_rate: f64,
    /// Synaptic density (connections per neuron, 0-1). Modulates phi/consciousness.
    pub synaptic_density: f64,
    /// BDNF level (neurotrophic factor) — derived from serotonin, exercise, learning.
    pub bdnf_level: f64,
}

impl GreyMatterSystem {
    /// Creates a new system from the config.
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

    /// Biological tick: updates all brain parameters.
    ///
    /// Input parameters from other systems:
    /// - `cortisol`, `serotonin`, `dopamine`: current chemistry
    /// - `soma_energy`: ATP reserves (nutrition)
    /// - `is_sleeping`: in sleep phase
    /// - `is_learning`: learning activity detected
    /// - `tryptophan`: precursor amino acid (nutrition → BDNF)
    /// - `novelty_detected`: novel stimulus detected (novelty > 0.7)
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
        // Neurotrophic factor derived from multiple biological signals.
        // Base: serotonin + energy + learning + tryptophan
        let learning_factor = if is_learning { 0.2 } else { 0.0 };
        let mut raw_bdnf = serotonin * 0.3 + soma_energy * 0.3 + learning_factor + tryptophan * 0.1;

        // Dopamine promotes BDNF (reward-driven neuroplasticity)
        raw_bdnf += dopamine * 0.15;

        // Novelty bonus: exploration stimulates BDNF production
        if novelty_detected {
            raw_bdnf += 0.1;
        }

        // Flow state bonus: high dopamine + low cortisol = optimal state
        if dopamine > 0.6 && cortisol < 0.4 {
            raw_bdnf += 0.1;
        }

        // Cortisol penalty: chronic stress inhibits BDNF
        if cortisol > 0.6 {
            raw_bdnf -= (cortisol - 0.6) * 0.4;
        }

        // Exponential smoothing
        self.bdnf_level = self.bdnf_level * 0.95 + raw_bdnf * 0.05;

        // Homeostasis toward baseline 0.5: slow pull towards equilibrium
        self.bdnf_level += (0.5 - self.bdnf_level) * 0.01;

        self.bdnf_level = self.bdnf_level.clamp(0.0, 1.0);

        // ─── Neurogenesis ────────────────────────────────────────────────────
        // BDNF boost, cortisol penalizes, sleep deficit penalizes
        let cortisol_penalty = if cortisol > 0.6 { (cortisol - 0.6) * 0.3 } else { 0.0 };
        let sleep_deficit = if !is_sleeping && soma_energy < 0.3 { 0.1 } else { 0.0 };
        let neurogenesis_target = (self.bdnf_level * 0.6 - cortisol_penalty - sleep_deficit).clamp(0.0, 1.0);
        self.neurogenesis_rate = self.neurogenesis_rate * 0.98 + neurogenesis_target * 0.02;

        // ─── Grey matter ──────────────────────────────────────────────────
        // Growth through learning, natural decline, BDNF boost
        let growth = if is_learning { config.growth_rate * 2.0 } else { config.growth_rate };
        let bdnf_bonus = if self.bdnf_level > config.bdnf_threshold {
            (self.bdnf_level - config.bdnf_threshold) * 0.0002
        } else { 0.0 };
        self.grey_matter_volume += growth + bdnf_bonus - config.decline_rate;
        self.grey_matter_volume = self.grey_matter_volume.clamp(0.1, 1.0);

        // ─── Myelination ──────────────────────────────────────────────────
        // Slow growth (with experience), very slow decline
        let myelin_growth = config.myelination_growth;
        self.myelination += myelin_growth - config.decline_rate * 0.5;
        self.myelination = self.myelination.clamp(0.1, 1.0);

        // ─── Neuroplasticity ────────────────────────────────────────────────
        // Base + dopamine (motivation) + BDNF - cortisol (rigidity)
        let plasticity_target = 0.3 + dopamine * 0.4 + self.bdnf_level * 0.2 - cortisol * 0.3;
        self.neuroplasticity = self.neuroplasticity * 0.97 + plasticity_target.clamp(0.0, 1.0) * 0.03;

        // ─── Synaptic density ─────────────────────────────────────────────
        // Pruning during sleep, growth during wakefulness
        if is_sleeping {
            // Pruning synaptique (SHY — Synaptic Homeostasis Hypothesis)
            let overshoot = (self.synaptic_density - config.optimal_synaptic_density).max(0.0);
            self.synaptic_density -= overshoot * config.pruning_rate;
        } else {
            // Synaptic growth during wakefulness (Hebbian)
            let growth_rate = 0.0003 * self.neuroplasticity;
            self.synaptic_density += growth_rate;
        }
        self.synaptic_density = self.synaptic_density.clamp(0.1, 1.0);
    }

    /// Influence of grey matter on neurochemistry.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();
        if !self.enabled { return adj; }

        // Low grey matter → stress (cortisol +)
        if self.grey_matter_volume < 0.3 {
            adj.cortisol += (0.3 - self.grey_matter_volume) * 0.03;
        }

        // High neurogenesis → endorphin + (neuronal well-being)
        if self.neurogenesis_rate > 0.6 {
            adj.endorphin += (self.neurogenesis_rate - 0.6) * 0.02;
        }

        // Synaptic density too high → overload → cortisol +
        if self.synaptic_density > 0.85 {
            adj.cortisol += (self.synaptic_density - 0.85) * 0.02;
        }

        adj
    }

    /// Serializes the state to JSON for persistence.
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

    /// Restores state from persisted JSON.
    pub fn restore_from_json(&mut self, v: &serde_json::Value) {
        self.grey_matter_volume = v["grey_matter_volume"].as_f64().unwrap_or(self.grey_matter_volume);
        self.myelination = v["myelination"].as_f64().unwrap_or(self.myelination);
        self.neuroplasticity = v["neuroplasticity"].as_f64().unwrap_or(self.neuroplasticity);
        self.neurogenesis_rate = v["neurogenesis_rate"].as_f64().unwrap_or(self.neurogenesis_rate);
        self.synaptic_density = v["synaptic_density"].as_f64().unwrap_or(self.synaptic_density);
        self.bdnf_level = v["bdnf_level"].as_f64().unwrap_or(self.bdnf_level);
    }
}
