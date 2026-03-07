// =============================================================================
// neocortex.rs — Neocortex module: rational analysis, mental clarity
// =============================================================================
//
// Purpose: Implements the neocortex module of Saphire, inspired by the
//          neocortex (neomammalian brain) in Paul MacLean's triune brain
//          model. The neocortex is the most recently evolved cortical
//          structure and is responsible for rational analysis, cost/benefit
//          evaluation, executive function, and logical decision-making. It
//          is the most "reflective" module but is vulnerable to stress,
//          which degrades its mental clarity — modeling the well-documented
//          effect of cortisol on prefrontal cortex executive functions.
//
// Dependencies:
//   - crate::neurochemistry::NeuroChemicalState: neurochemical state
//     (cortisol and adrenaline degrade clarity; noradrenaline improves it)
//   - crate::stimulus::Stimulus: sensory input (reward, danger, social,
//     urgency axes)
//   - super::BrainModule, ModuleSignal: shared trait and output type
//
// Role in the architecture:
//   Third of the 3 cerebral modules. Its signal reflects a rational,
//   weighted analysis of the stimulus. Its weight in the consensus
//   increases when serotonin and noradrenaline are elevated (calm + focus)
//   and decreases under stress (high cortisol and adrenaline).
// =============================================================================

use crate::neurochemistry::NeuroChemicalState;
use crate::stimulus::Stimulus;
use super::{BrainModule, ModuleSignal};

/// The neocortex — rational thought, cost/benefit analysis, executive
/// function.
///
/// The most evolutionarily advanced module, it analyzes the stimulus
/// logically by weighing benefits against risks. Its performance depends
/// on "mental clarity", which is itself modulated by the neurochemical
/// state: cortisol (stress hormone) impairs prefrontal function, while
/// noradrenaline (focus neurotransmitter) enhances attentional control.
pub struct NeocortexModule;

impl BrainModule for NeocortexModule {
    /// Returns the name of this module: "Neocortex".
    fn name(&self) -> &str {
        "Néocortex"
    }

    /// Processes a stimulus from a rational perspective (cost/benefit
    /// analysis).
    ///
    /// Algorithm:
    /// 1. **Mental clarity computation**: penalized by stress (cortisol +
    ///    adrenaline), improved by attentional focus (noradrenaline). This
    ///    models the well-documented impairment of prefrontal cortex
    ///    executive functions under acute stress (Arnsten, 2009).
    /// 2. **Cost/benefit analysis**: reward and social factors serve as
    ///    benefits; danger and urgency serve as costs. The result is scaled
    ///    by the current mental clarity.
    /// 3. **Confidence**: proportional to mental clarity — the neocortex
    ///    trusts its own judgment only when it can reason effectively.
    ///
    /// Mental clarity is bounded between 0.1 (minimum vital — even under
    /// extreme stress, the neocortex retains some residual function) and
    /// 1.5 (optimal focus).
    ///
    /// # Parameters
    /// - `stimulus`: sensory input with its perceptual dimension scores.
    /// - `chemistry`: neurochemical state (cortisol and adrenaline degrade
    ///   clarity; noradrenaline improves it).
    ///
    /// # Returns
    /// A `ModuleSignal` with the computed signal, confidence, and
    /// explanatory reasoning.
    fn process(&self, stimulus: &Stimulus, chemistry: &NeuroChemicalState) -> ModuleSignal {
        // Stress penalty: cortisol and adrenaline reduce mental clarity.
        // This simulates the well-documented effect of acute stress on
        // executive functions — difficulty reasoning under pressure
        // (prefrontal cortex hypoactivation during high cortisol states).
        let stress_penalty = (chemistry.cortisol + chemistry.adrenaline) * 0.5;

        // Focus bonus: noradrenaline improves concentration and cognitive
        // vigilance (stimulant effect on attentional networks, particularly
        // the locus coeruleus-noradrenergic system).
        let focus = chemistry.noradrenaline * 0.4;

        // Resulting mental clarity: 1.0 = normal baseline, < 1.0 = degraded
        // by stress, > 1.0 = enhanced by focus. Clamped between 0.1 (minimum
        // residual function) and 1.5 (optimal performance).
        let clarity = (1.0 - stress_penalty + focus).clamp(0.1, 1.5);

        // Rational cost/benefit analysis:
        //   Benefits: reward (weight 0.7) + social (weight 0.3)
        //   Costs:    danger (weight 0.6) + urgency (weight 0.2)
        // The result is multiplied by mental clarity: under stress the
        // analysis is attenuated; with focus it is amplified.
        let raw = (stimulus.reward * 0.7
            + stimulus.social * 0.3
            - stimulus.danger * 0.6
            - stimulus.urgency * 0.2)
            * clarity;

        // tanh() naturally bounds the signal in [-1.0, +1.0]
        let signal = raw.tanh();

        // Confidence is proportional to mental clarity: the neocortex trusts
        // its judgment only when it can reason clearly. clarity / 1.5
        // normalizes to [0, 1], clamped between 0.2 and 0.95.
        let confidence = (clarity / 1.5).clamp(0.2, 0.95);

        // Detailed reasoning including the current mental clarity state
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
