// =============================================================================
// neocortex.rs — Neocortex module: rational analysis, mental clarity
// =============================================================================
//
// Role: This file implements Saphire's neocortex module, inspired by the
// neocortex in Paul MacLean's triune brain model. It handles rational
// analysis, cost/benefit calculation and logical decision-making. It is
// the most "thoughtful" module but is vulnerable to stress which
// degrades its mental clarity.
//
// Dependencies:
//   - crate::neurochemistry::NeuroChemicalState: chemical state (cortisol and
//     adrenaline degrade clarity, noradrenaline improves it)
//   - crate::stimulus::Stimulus: sensory input (reward, danger, social, urgency)
//   - super::BrainModule, ModuleSignal: common trait and output type
//
// Place in the architecture:
//   Third of the 3 brain modules. Its signal reflects a weighted rational
//   analysis. Its weight in the consensus increases when serotonin and
//   noradrenaline are high (calm + focus), and decreases under stress
//   (high cortisol, adrenaline).
// =============================================================================

use crate::neurochemistry::NeuroChemicalState;
use crate::stimulus::Stimulus;
use super::{BrainModule, ModuleSignal};

/// The neocortex — rational thought, cost/benefit analysis.
/// The most evolved module, it analyzes the stimulus logically by
/// weighing benefits against risks. Its performance depends on
/// "mental clarity", which is itself influenced by neurochemistry.
pub struct NeocortexModule;

impl BrainModule for NeocortexModule {
    /// Returns the module name: "Neocortex".
    fn name(&self) -> &str {
        "Néocortex"
    }

    /// Processes a stimulus from a rational perspective (cost/benefit analysis).
    ///
    /// Algorithm:
    /// 1. Mental clarity calculation: penalized by stress (cortisol +
    ///    adrenaline), improved by focus (noradrenaline).
    /// 2. Cost/benefit analysis: reward and social as benefits,
    ///    danger and urgency as costs, all multiplied by clarity.
    /// 3. Confidence proportional to mental clarity.
    ///
    /// Mental clarity is bounded between 0.1 (vital minimum, even under
    /// extreme stress the neocortex functions a little) and 1.5 (optimal focus).
    ///
    /// # Parameters
    /// - `stimulus`: sensory input with its perceptual scores.
    /// - `chemistry`: chemical state (cortisol and adrenaline degrade clarity,
    ///   noradrenaline improves it).
    ///
    /// # Returns
    /// A `ModuleSignal` with signal, confidence and explanatory reasoning.
    fn process(&self, stimulus: &Stimulus, chemistry: &NeuroChemicalState) -> ModuleSignal {
        // Stress penalty: cortisol and adrenaline reduce mental clarity.
        // This simulates the well-documented effect of stress on executive
        // functions (difficulty thinking under pressure).
        let stress_penalty = (chemistry.cortisol + chemistry.adrenaline) * 0.5;

        // Focus bonus: noradrenaline improves concentration and
        // cognitive vigilance (stimulating effect on attention).
        let focus = chemistry.noradrenaline * 0.4;

        // Resulting mental clarity: 1.0 = normal, < 1.0 = degraded by
        // stress, > 1.0 = improved by focus. Bounded between 0.1
        // (vital minimum) and 1.5 (optimal performance).
        let clarity = (1.0 - stress_penalty + focus).clamp(0.1, 1.5);

        // Rational cost/benefit analysis:
        //   Benefits: reward (weight 0.7) + social (weight 0.3)
        //   Costs: danger (weight 0.6) + urgency (weight 0.2)
        // The result is multiplied by mental clarity: under stress,
        // the analysis is attenuated; with focus, it is amplified.
        let raw = (stimulus.reward * 0.7
            + stimulus.social * 0.3
            - stimulus.danger * 0.6
            - stimulus.urgency * 0.2)
            * clarity;

        // tanh() naturally bounds the signal between -1 and +1
        let signal = raw.tanh();

        // Confidence proportional to mental clarity: the neocortex
        // trusts its judgment only when it can think clearly.
        // clarity/1.5 normalizes between 0 and 1, bounded between 0.2 and 0.95.
        let confidence = (clarity / 1.5).clamp(0.2, 0.95);

        // Detailed reasoning with mental clarity state
        let reasoning = format!(
            "Analyse rationnelle (clarté={:.2}) : bénéfice={:.2}, risque={:.2}, social={:.2}. {}",
            clarity,
            stimulus.reward,
            stimulus.danger,
            stimulus.social,
            if clarity < 0.5 {
                "⚠ Clarté mentale dégradée par le stress."
            } else if clarity > 1.0 {
                "Focus élevé, analyse optimale."
            } else {
                "Clarté mentale normale."
            }
        );

        ModuleSignal {
            module: self.name().to_string(),
            signal: signal.clamp(-1.0, 1.0),
            confidence,
            reasoning,
        }
    }
}
