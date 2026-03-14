// =============================================================================
// consensus.rs — Weighting and consensus of the 3 brain modules
// =============================================================================
//
// Role: This file implements Saphire's decision-making mechanism.
// It combines the signals from the 3 brain modules (reptilian, limbic,
// neocortex) into a single score via a weighted sum. The weight of
// each module varies dynamically according to the neurochemical state.
//
// Dependencies:
//   - serde : serialization / deserialization
//   - crate::neurochemistry::NeuroChemicalState : chemical state (for the weights)
//   - crate::modules::ModuleSignal : signals emitted by each brain module
//
// Place in architecture:
//   This module is the decision-making core. It is called after the 3 brain
//   modules have processed the stimulus. The result is then observed by
//   consciousness.rs and used for neurochemical feedback.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::neurochemistry::NeuroChemicalState;
use crate::modules::ModuleSignal;
use crate::tuning::params::TunableParams;

/// Brain decision — trivalent result (Yes / No / Maybe).
///
/// The decision is determined by comparing the weighted score with the thresholds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    /// Approval: the score exceeds the positive threshold
    Yes,
    /// Rejection: the score is below the negative threshold
    No,
    /// Indecision: the score is between the two thresholds
    Maybe,
}

impl Decision {
    /// Converts the decision to a French string.
    ///
    /// # Returns
    /// "Oui", "Non" or "Peut-être".
    pub fn as_str(&self) -> &str {
        match self {
            Decision::Yes => "Oui",
            Decision::No => "Non",
            Decision::Maybe => "Peut-être",
        }
    }

    /// Converts the decision to a signed integer.
    ///
    /// # Returns
    /// 1 (Yes), -1 (No) or 0 (Maybe).
    pub fn as_i8(&self) -> i8 {
        match self {
            Decision::Yes => 1,
            Decision::No => -1,
            Decision::Maybe => 0,
        }
    }
}

/// Consensus result — contains all information about the decision
/// made, the weights used, the individual signals, and the coherence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    /// Final weighted score [-1, +1]: sum(weight_i * signal_i)
    pub score: f64,
    /// Trivalent decision derived from the score and thresholds
    pub decision: Decision,
    /// Normalized module weights [reptilian, limbic, neocortex].
    /// Sum = 1.0. Vary dynamically according to neurochemistry.
    pub weights: [f64; 3],
    /// Individual signals from the 3 brain modules (reptilian, limbic, neocortex)
    pub signals: Vec<ModuleSignal>,
    /// Coherence between modules [0, 1]: measures agreement between signals.
    /// 1.0 = perfect unanimity, 0.0 = maximum disagreement.
    pub coherence: f64,
}

/// Decision thresholds — define the boundaries between Yes, No, and Maybe.
///
/// If score > threshold_yes => Yes
/// If score < threshold_no => No
/// Otherwise => Maybe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusThresholds {
    /// Positive threshold above which the decision is "Yes" (default: 0.33)
    pub threshold_yes: f64,
    /// Negative threshold below which the decision is "No" (default: -0.33)
    pub threshold_no: f64,
}

impl Default for ConsensusThresholds {
    /// Default thresholds: "Maybe" zone between -0.33 and +0.33,
    /// covering approximately the central third of the decision space.
    fn default() -> Self {
        Self {
            threshold_yes: 0.33,
            threshold_no: -0.33,
        }
    }
}

/// Computes the dynamic weights of brain modules according to the neurochemistry.
///
/// The weights determine the relative influence of each module in the decision:
/// - Reptilian: takes over in stress/danger situations (high cortisol,
///   adrenaline). Endorphin slightly attenuates its influence.
/// - Limbic: dominant when positive emotions prevail (high dopamine,
///   serotonin, oxytocin).
/// - Neocortex: has a high base (reasoning by default) but is
///   inhibited by stress. Noradrenaline (focus) reinforces it.
///
/// The weights are normalized so their sum equals 1.0, with a minimum
/// guaranteed of 0.05 per module (no module is ever completely silenced).
///
/// # Parameters
/// - `chemistry` : current neurochemical state.
///
/// # Returns
/// Array [reptilian_weight, limbic_weight, neocortex_weight] with sum = 1.0.
pub fn compute_weights(chemistry: &NeuroChemicalState, params: &TunableParams) -> [f64; 3] {
    // Reptilian: amplified by cortisol (stress) and adrenaline (urgency).
    // Endorphin (resilience) slightly reduces its influence.
    let w_r = params.weight_base_reptilian
        + chemistry.cortisol * params.weight_cortisol_factor
        + chemistry.adrenaline * params.weight_adrenaline_factor
        - chemistry.endorphin * 0.5;

    // Limbic: amplified by "social and emotional" molecules.
    // Dopamine (motivation), serotonin (well-being), and oxytocin (social bonding).
    let w_l = params.weight_base_limbic
        + chemistry.dopamine * params.weight_dopamine_factor
        + chemistry.serotonin * 1.0
        + chemistry.oxytocin * params.weight_oxytocin_factor;

    // Neocortex: high base since rational reasoning is the
    // default mode. Stress (cortisol + adrenaline) degrades it,
    // while serotonin (calm) and noradrenaline (focus) improve it.
    let w_n = params.weight_base_neocortex
        - chemistry.cortisol * 1.5
        - chemistry.adrenaline * 2.0
        + chemistry.serotonin * 0.5
        + chemistry.noradrenaline * params.weight_noradrenaline_factor;

    // Guarantee a minimum of 0.05 per module — no module should be
    // completely silenced, even in extreme situations
    let w_r = w_r.max(0.05);
    let w_l = w_l.max(0.05);
    let w_n = w_n.max(0.05);

    // Normalize so that the sum = 1.0
    let total = w_r + w_l + w_n;
    [w_r / total, w_l / total, w_n / total]
}

/// Computes the consensus from the signals of the 3 brain modules.
///
/// Algorithm:
/// 1. Compute dynamic weights according to neurochemistry.
/// 2. Weighted score: sum(weight_i * signal_i), bounded between -1 and +1.
/// 3. Trivalent decision: comparison of the score with the thresholds.
/// 4. Coherence: 1 - variance of signals (measures agreement between modules).
///
/// # Parameters
/// - `signals` : array of 3 signals [reptilian, limbic, neocortex].
/// - `chemistry` : current chemical state (for weight computation).
/// - `thresholds` : Yes/No decision thresholds.
///
/// # Returns
/// A `ConsensusResult` containing the score, decision, weights,
/// signals, and coherence.
pub fn consensus(
    signals: &[ModuleSignal; 3],
    chemistry: &NeuroChemicalState,
    thresholds: &ConsensusThresholds,
    params: &TunableParams,
) -> ConsensusResult {
    let weights = compute_weights(chemistry, params);

    // Weighted score: linear combination of signals by weights
    let score = weights[0] * signals[0].signal
        + weights[1] * signals[1].signal
        + weights[2] * signals[2].signal;
    let score = score.clamp(-1.0, 1.0);

    // Trivalent decision by comparison with thresholds
    let decision = if score > thresholds.threshold_yes {
        Decision::Yes
    } else if score < thresholds.threshold_no {
        Decision::No
    } else {
        Decision::Maybe
    };

    // Coherence: measures the concordance of signals between modules.
    // If all 3 modules agree (close signals), the variance is
    // low and coherence is high. In case of deep disagreement
    // (e.g. reptilian says No, limbic says Yes), coherence drops.
    let signals_vec = [signals[0].signal, signals[1].signal, signals[2].signal];
    let mean = signals_vec.iter().sum::<f64>() / 3.0;
    let variance = signals_vec.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / 3.0;
    let coherence = (1.0 - variance).clamp(0.0, 1.0);

    ConsensusResult {
        score,
        decision,
        weights,
        signals: signals.to_vec(),
        coherence,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neurochemistry::NeuroChemicalState;
    use crate::modules::ModuleSignal;
    use crate::tuning::params::TunableParams;

    #[test]
    fn test_weights_sum_to_one() {
        let chem = NeuroChemicalState::default();
        let params = TunableParams::default();
        let weights = compute_weights(&chem, &params);
        let sum: f64 = weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10, "Weights should sum to 1.0, got {}", sum);
    }

    #[test]
    fn test_consensus_produces_decision() {
        let chem = NeuroChemicalState::default();
        let params = TunableParams::default();
        let signals = [
            ModuleSignal { module: "Reptilien".into(), signal: 0.5, confidence: 0.8, reasoning: "".into() },
            ModuleSignal { module: "Limbique".into(), signal: 0.6, confidence: 0.7, reasoning: "".into() },
            ModuleSignal { module: "Neocortex".into(), signal: 0.8, confidence: 0.9, reasoning: "".into() },
        ];
        let thresholds = ConsensusThresholds::default();
        let result = consensus(&signals, &chem, &thresholds, &params);
        assert!(matches!(result.decision, Decision::Yes | Decision::No | Decision::Maybe));
    }

    #[test]
    fn test_coherence_in_range() {
        let chem = NeuroChemicalState::default();
        let params = TunableParams::default();
        let signals = [
            ModuleSignal { module: "R".into(), signal: 0.5, confidence: 0.8, reasoning: "".into() },
            ModuleSignal { module: "L".into(), signal: 0.5, confidence: 0.8, reasoning: "".into() },
            ModuleSignal { module: "N".into(), signal: 0.5, confidence: 0.8, reasoning: "".into() },
        ];
        let thresholds = ConsensusThresholds::default();
        let result = consensus(&signals, &chem, &thresholds, &params);
        assert!(result.coherence >= 0.0 && result.coherence <= 1.0);
    }

    #[test]
    fn test_aligned_signals_give_yes() {
        let chem = NeuroChemicalState::default();
        let params = TunableParams::default();
        let signals = [
            ModuleSignal { module: "R".into(), signal: 0.9, confidence: 1.0, reasoning: "".into() },
            ModuleSignal { module: "L".into(), signal: 0.9, confidence: 1.0, reasoning: "".into() },
            ModuleSignal { module: "N".into(), signal: 0.9, confidence: 1.0, reasoning: "".into() },
        ];
        let thresholds = ConsensusThresholds::default();
        let result = consensus(&signals, &chem, &thresholds, &params);
        assert!(matches!(result.decision, Decision::Yes), "Strongly positive signals should give Yes");
    }

    #[test]
    fn test_negative_signals_give_no() {
        let chem = NeuroChemicalState::default();
        let params = TunableParams::default();
        let signals = [
            ModuleSignal { module: "R".into(), signal: -0.9, confidence: 1.0, reasoning: "".into() },
            ModuleSignal { module: "L".into(), signal: -0.9, confidence: 1.0, reasoning: "".into() },
            ModuleSignal { module: "N".into(), signal: -0.9, confidence: 1.0, reasoning: "".into() },
        ];
        let thresholds = ConsensusThresholds::default();
        let result = consensus(&signals, &chem, &thresholds, &params);
        assert!(matches!(result.decision, Decision::No), "Strongly negative signals should give No");
    }
}
