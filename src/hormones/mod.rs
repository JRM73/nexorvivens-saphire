// =============================================================================
// hormones/ — Stub minimal pour la version lite
//
// Seule la structure HormonalState est definie ici, utilisee par
// body/physiology.rs pour le calcul des parametres vitaux.
// Le systeme hormonal complet (recepteurs, cycles, interactions)
// n'est pas porte dans la version lite.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Etat hormonal simplifie — 8 hormones normalisees [0.0, 1.0].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HormonalState {
    pub cortisol_h: f64,
    pub melatonin: f64,
    pub epinephrine: f64,
    pub testosterone: f64,
    pub estrogen: f64,
    pub oxytocin_h: f64,
    pub insulin: f64,
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
