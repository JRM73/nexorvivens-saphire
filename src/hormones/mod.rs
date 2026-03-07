// =============================================================================
// hormones/ — Minimal stub for the lite edition
// =============================================================================
//
// Purpose: Only the HormonalState structure is defined here, used by
//          body/physiology.rs for computing vital parameters.
//          The full hormonal system (receptors, cycles, interactions)
//          is not ported in the lite edition.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Simplified hormonal state — 8 hormones normalized to [0.0, 1.0].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HormonalState {
    /// Cortisol (hormonal level, distinct from neurochemical cortisol)
    pub cortisol_h: f64,
    /// Melatonin (sleep/wake cycle regulation)
    pub melatonin: f64,
    /// Epinephrine (adrenaline, fight-or-flight)
    pub epinephrine: f64,
    /// Testosterone (drive, assertiveness)
    pub testosterone: f64,
    /// Estrogen (mood modulation, neuroprotection)
    pub estrogen: f64,
    /// Oxytocin (hormonal level, social bonding)
    pub oxytocin_h: f64,
    /// Insulin (energy regulation, glucose metabolism)
    pub insulin: f64,
    /// Thyroid hormones (metabolic rate, energy levels)
    pub thyroid: f64,
}

impl Default for HormonalState {
    fn default() -> Self {
        Self {
            cortisol_h: 0.3,
            melatonin: 0.2,
            epinephrine: 0.2,
            testosterone: 0.4,
            estrogen: 0.4,
            oxytocin_h: 0.3,
            insulin: 0.5,
            thyroid: 0.5,
        }
    }
}
