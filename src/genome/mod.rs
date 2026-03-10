// genome/ — Stub for the lite edition

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChemicalGenes {
    pub baseline_dopamine_offset: f64,
    pub baseline_serotonin_offset: f64,
    pub baseline_cortisol_offset: f64,
}

impl Default for ChemicalGenes {
    fn default() -> Self {
        Self {
            baseline_dopamine_offset: 0.0,
            baseline_serotonin_offset: 0.0,
            baseline_cortisol_offset: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genome {
    pub seed: u64,
    pub chemical: ChemicalGenes,
}

impl Genome {
    pub fn new(seed: u64) -> Self { Self { seed, chemical: ChemicalGenes::default() } }

    pub fn from_seed(seed: u64) -> Self { Self::new(seed) }

    pub fn log_summary(&self) {
        tracing::info!("Genome seed={}, dopamine_offset={:.3}, serotonin_offset={:.3}, cortisol_offset={:.3}",
            self.seed, self.chemical.baseline_dopamine_offset,
            self.chemical.baseline_serotonin_offset, self.chemical.baseline_cortisol_offset);
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "seed": self.seed,
            "chemical": {
                "baseline_dopamine_offset": self.chemical.baseline_dopamine_offset,
                "baseline_serotonin_offset": self.chemical.baseline_serotonin_offset,
                "baseline_cortisol_offset": self.chemical.baseline_cortisol_offset,
            }
        })
    }
}
