// =============================================================================
// tuning/params.rs — Tunable brain parameters (lite version)
// =============================================================================
//
// In the lite version, the auto-tuner has been removed.
// TunableParams is retained with its default values because it is used
// by consensus::consensus() to weight the cerebral modules.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Fixed parameters for the cognitive pipeline.
///
/// In the lite version, these parameters are not automatically adjusted;
/// they remain at their default values throughout the agent's lifetime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunableParams {
    /// Base weight for the reptilian brain module. Higher values increase
    /// the influence of survival-oriented instinctive reactions.
    pub weight_base_reptilian: f64,
    /// Scaling factor applied to the cortisol level when computing
    /// the reptilian module's effective weight. Controls how strongly
    /// stress amplifies the reptilian response.
    pub weight_cortisol_factor: f64,
    /// Scaling factor applied to the adrenaline level when computing
    /// the reptilian module's effective weight. Controls how strongly
    /// arousal amplifies the reptilian response.
    pub weight_adrenaline_factor: f64,
    /// Base weight for the limbic brain module. Higher values increase
    /// the influence of emotion-driven processing.
    pub weight_base_limbic: f64,
    /// Scaling factor applied to the dopamine level when computing
    /// the limbic module's effective weight. Controls how strongly
    /// reward signals amplify the limbic response.
    pub weight_dopamine_factor: f64,
    /// Scaling factor applied to the oxytocin level when computing
    /// the limbic module's effective weight. Controls how strongly
    /// social bonding amplifies the limbic response.
    pub weight_oxytocin_factor: f64,
    /// Base weight for the neocortex brain module. Higher values increase
    /// the influence of rational, analytical processing.
    pub weight_base_neocortex: f64,
    /// Scaling factor applied to the noradrenaline level when computing
    /// the neocortex module's effective weight. Controls how strongly
    /// focused attention amplifies the neocortical response.
    pub weight_noradrenaline_factor: f64,

    /// Consensus threshold below which the decision is "No" (negative score).
    /// Typical range: -1.0 to 0.0.
    pub threshold_no: f64,
    /// Consensus threshold above which the decision is "Yes" (positive score).
    /// Typical range: 0.0 to 1.0.
    pub threshold_yes: f64,

    /// Dopamine boost applied as positive feedback after a successful decision.
    /// Typical range: 0.0 to 0.5.
    pub feedback_dopamine_boost: f64,
    /// Cortisol relief applied as positive feedback after resolving a stressor.
    /// Typical range: 0.0 to 0.2.
    pub feedback_cortisol_relief: f64,
    /// Stress increase applied when the consensus is indecisive (score between
    /// threshold_no and threshold_yes). Typical range: 0.0 to 0.2.
    pub feedback_indecision_stress: f64,

    /// Rate at which neurochemical levels return toward their baseline values
    /// each cycle (homeostatic regulation). Typical range: 0.01 to 0.3.
    pub homeostasis_rate: f64,
}

impl Default for TunableParams {
    fn default() -> Self {
        Self {
            weight_base_reptilian: 1.0,
            weight_cortisol_factor: 2.0,
            weight_adrenaline_factor: 3.0,
            weight_base_limbic: 1.0,
            weight_dopamine_factor: 1.5,
            weight_oxytocin_factor: 1.5,
            weight_base_neocortex: 1.5,
            weight_noradrenaline_factor: 1.5,
            threshold_no: -0.33,
            threshold_yes: 0.33,
            feedback_dopamine_boost: 0.15,
            feedback_cortisol_relief: 0.05,
            feedback_indecision_stress: 0.08,
            homeostasis_rate: 0.10,
        }
    }
}
