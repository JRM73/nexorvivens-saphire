// =============================================================================
// params.rs — Tunable brain parameters
//
// Role: This file defines the TunableParams structure containing all
// tunable coefficients for Saphire's brain. These parameters are modified
// by the auto-tuner (CoefficientTuner) and persisted in the database.
//
// Dependencies:
//   - serde: serialization/deserialization for persistence and API
//
// Place in architecture:
//   TunableParams is used by the brain (brain.rs) to weight the modules,
//   compute the consensus, and apply feedback.
//   The auto-tuner (tuning/mod.rs) modifies these parameters incrementally
//   to optimize the agent's satisfaction over time.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Parameters tunable by the auto-tuner.
/// Each parameter influences an aspect of Saphire's cognitive processing.
/// The default values represent a reasonable starting balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunableParams {
    // --- Brain module weights ---
    // These weights determine the relative influence of each "brain"
    // in the consensus computation.

    /// Base weight of the reptilian module (instinct, survival, reflexes).
    /// The reptilian reacts to danger and urgency.
    pub weight_base_reptilian: f64,
    /// Cortisol (stress hormone) multiplier factor on the reptilian weight.
    /// The higher the cortisol, the more influential the reptilian becomes.
    pub weight_cortisol_factor: f64,
    /// Adrenaline multiplier factor on the reptilian weight.
    /// Adrenaline amplifies survival reactions.
    pub weight_adrenaline_factor: f64,
    /// Base weight of the limbic module (emotions, emotional memory).
    /// The limbic reacts to reward and social bonds.
    pub weight_base_limbic: f64,
    /// Dopamine multiplier factor on the limbic weight.
    /// Dopamine amplifies reward sensitivity.
    pub weight_dopamine_factor: f64,
    /// Oxytocin multiplier factor on the limbic weight.
    /// Oxytocin amplifies sensitivity to social bonds.
    pub weight_oxytocin_factor: f64,
    /// Base weight of the neocortex module (reasoning, analysis, logic).
    /// The neocortex provides the most rational evaluation.
    pub weight_base_neocortex: f64,
    /// Noradrenaline multiplier factor on the neocortex weight.
    /// Noradrenaline increases concentration and attention.
    pub weight_noradrenaline_factor: f64,

    // --- Consensus thresholds ---
    // The consensus score is a number between -1.0 and +1.0.
    // The decision is determined by comparison with these thresholds.

    /// Threshold below which the decision is "No" (negative value).
    /// E.g.: -0.33 means any score < -0.33 yields "No".
    pub threshold_no: f64,
    /// Threshold above which the decision is "Yes" (positive value).
    /// E.g.: 0.33 means any score > 0.33 yields "Yes".
    /// Between the two thresholds, the decision is "Maybe".
    pub threshold_yes: f64,

    // --- Feedback rates ---
    // Feedback adjusts neurochemistry after each decision
    // based on the outcome.

    /// Dopamine boost applied after a satisfying "Yes" decision.
    /// Simulates the pleasure of obtained reward.
    pub feedback_dopamine_boost: f64,
    /// Cortisol reduction after a reassuring "No" decision.
    /// Simulates the relief of having avoided danger.
    pub feedback_cortisol_relief: f64,
    /// Stress increase (cortisol) after a "Maybe" decision.
    /// Simulates the discomfort of indecision.
    pub feedback_indecision_stress: f64,

    // --- Homeostasis rate ---

    /// Speed at which neurochemistry returns to baseline values.
    /// The higher the value, the faster the return to equilibrium.
    /// Simulates the natural regulation of neurotransmitters.
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

impl TunableParams {
    /// Clamps all parameters within safety bounds.
    /// Prevents aberrant values that could destabilize the system
    /// (for example a weight of 100 or a threshold of 0).
    ///
    /// Bounds are chosen to allow sufficient variation
    /// while guaranteeing stable behavior:
    /// - Module weights: [0.1, 5.0]
    /// - Thresholds: [0.05, 0.8] in absolute value
    /// - Feedback rates: [0.01, 0.5]
    /// - Homeostasis: [0.01, 0.2]
    pub fn clamp_all(&mut self) {
        // Brain module weights
        self.weight_base_reptilian = self.weight_base_reptilian.clamp(0.1, 5.0);
        self.weight_cortisol_factor = self.weight_cortisol_factor.clamp(0.1, 5.0);
        self.weight_adrenaline_factor = self.weight_adrenaline_factor.clamp(0.1, 5.0);
        self.weight_base_limbic = self.weight_base_limbic.clamp(0.1, 5.0);
        self.weight_dopamine_factor = self.weight_dopamine_factor.clamp(0.1, 5.0);
        self.weight_oxytocin_factor = self.weight_oxytocin_factor.clamp(0.1, 5.0);
        self.weight_base_neocortex = self.weight_base_neocortex.clamp(0.1, 5.0);
        self.weight_noradrenaline_factor = self.weight_noradrenaline_factor.clamp(0.1, 5.0);

        // Consensus thresholds
        self.threshold_no = self.threshold_no.clamp(-0.8, -0.05);
        self.threshold_yes = self.threshold_yes.clamp(0.05, 0.8);

        // Feedback rates
        self.feedback_dopamine_boost = self.feedback_dopamine_boost.clamp(0.01, 0.5);
        self.feedback_cortisol_relief = self.feedback_cortisol_relief.clamp(0.01, 0.3);
        self.feedback_indecision_stress = self.feedback_indecision_stress.clamp(0.01, 0.3);

        // Homeostasis
        self.homeostasis_rate = self.homeostasis_rate.clamp(0.01, 0.2);
    }
}
