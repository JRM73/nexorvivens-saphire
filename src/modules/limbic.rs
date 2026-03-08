// =============================================================================
// limbic.rs — Limbic module: emotions, reward circuitry, social bonding
// =============================================================================
//
// Purpose: Implements the limbic module of Saphire, inspired by the limbic
//          system in Paul MacLean's triune brain model. The limbic system
//          corresponds to the paleomammalian brain and is responsible for
//          emotional responses: fear (amygdala), pleasure (reward circuitry /
//          mesolimbic dopamine pathway), social attachment (oxytocin-mediated
//          bonding), and emotional resilience (endorphin-mediated pain
//          buffering).
//
// Dependencies:
//   - crate::neurochemistry::NeuroChemicalState: neurochemical state
//     (dopamine, oxytocin, serotonin, and endorphin modulate processing)
//   - crate::stimulus::Stimulus: sensory input (danger, reward, social axes)
//   - super::BrainModule, ModuleSignal: shared trait and output type
//
// Role in the architecture:
//   Second of the 3 cerebral modules. Its signal reflects the emotional
//   reaction to the stimulus. Its weight in the consensus increases when
//   dopamine, serotonin, and oxytocin levels are elevated.
// =============================================================================

use crate::neurochemistry::NeuroChemicalState;
use crate::stimulus::Stimulus;
use super::{BrainModule, ModuleSignal};

/// The limbic system — emotional processing, empathy, and reward circuitry.
///
/// This module represents Saphire's emotional responses, modeling key
/// structures and functions of the mammalian limbic system:
/// - **Amygdala**: rapid fear response to perceived danger.
/// - **Reward circuit** (nucleus accumbens / ventral tegmental area):
///   attraction toward pleasurable stimuli, amplified by dopamine.
/// - **Social bonding** (hypothalamic oxytocin pathway): empathy and
///   attachment to social stimuli.
/// - **Emotional resilience** (endorphin system): pain buffering that adds
///   a slight positive bias under stress.
pub struct LimbicModule;

impl BrainModule for LimbicModule {
    /// Returns the name of this module: "Limbic".
    fn name(&self) -> &str {
        "Limbic"
    }

    /// Processes a stimulus from an emotional perspective.
    ///
    /// The algorithm computes 4 components:
    /// 1. **Amygdala**: negative reaction proportional to the danger score
    ///    (factor 0.8). The coefficient is slightly below 1.0 because the
    ///    limbic system is less "brutal" than the reptilian complex when
    ///    facing danger — it tempers raw threat with emotional context.
    /// 2. **Reward circuit**: attraction proportional to the reward score,
    ///    amplified by dopamine level. High dopamine makes even moderate
    ///    rewards highly attractive (motivational bias).
    /// 3. **Social bonding**: social component amplified by oxytocin level.
    ///    A baseline of 0.5 ensures a minimum level of social receptivity.
    /// 4. **Resilience**: endorphins attenuate emotional pain and add a
    ///    slight positive bias (capacity to endure adversity).
    ///
    /// The final signal also incorporates serotonin as a background
    /// well-being factor. The result is passed through tanh() to naturally
    /// bound it in [-1.0, +1.0].
    ///
    /// # Parameters
    /// - `stimulus`: sensory input with its perceptual dimension scores.
    /// - `chemistry`: neurochemical state (dopamine, oxytocin, serotonin,
    ///   and endorphin modulate the emotional response).
    ///
    /// # Returns
    /// A `ModuleSignal` with the computed signal, confidence, and
    /// explanatory reasoning.
    fn process(&self, stimulus: &Stimulus, chemistry: &NeuroChemicalState) -> ModuleSignal {
        // Amygdala: instinctive emotional reaction to danger.
        // The negative sign encodes rejection/fear. The 0.8 coefficient is
        // intentionally lower than 1.0 because the limbic system processes
        // danger with more emotional nuance than the reptilian complex.
        let amygdala = -stimulus.danger * 0.8;

        // Reward circuit: dopamine amplifies the attractiveness of the
        // reward. When dopamine is elevated, even a modest reward becomes
        // highly attractive (motivational bias mediated by the mesolimbic
        // dopamine pathway).
        let reward = stimulus.reward * (1.0 + chemistry.dopamine);

        // Social bonding: oxytocin amplifies sensitivity to social
        // interactions. A baseline of 0.5 guarantees a minimum level of
        // social receptivity even when oxytocin is low.
        let social = stimulus.social * (0.5 + chemistry.oxytocin * 0.5);

        // Emotional resilience: endorphins attenuate emotional pain and
        // contribute a slight positive bias (capacity to absorb adversity).
        let resilience = chemistry.endorphin * 0.2;

        // Raw signal: sum of the 4 components plus background well-being
        // from serotonin. tanh() naturally bounds the result in [-1, +1].
        let raw = amygdala + reward + social + chemistry.serotonin * 0.3 + resilience;
        let signal = raw.tanh();

        // Confidence is fixed at 0.7: the limbic system is always fairly
        // confident because emotions are, by their nature, felt with
        // subjective certainty (one does not doubt what one feels).
        let confidence = 0.7;

        // Reasoning: dynamically constructed by listing the significant
        // components (those exceeding their respective thresholds).
        let parts: Vec<String> = [
            if amygdala.abs() > 0.2 {
                Some(format!("Reactive amygdala ({:.2})", amygdala))
            } else { None },
            if reward > 0.3 {
                Some(format!("Attractive reward ({:.2})", reward))
            } else { None },
            if social > 0.2 {
                Some(format!("Social bond felt ({:.2})", social))
            } else { None },
        ].into_iter().flatten().collect();

        let reasoning = if parts.is_empty() {
            "Limbic system is neutral — no strong emotion.".to_string()
        } else {
            parts.join(". ") + "."
        };

        ModuleSignal {
            module: self.name().to_string(),
            signal: signal.clamp(-1.0, 1.0),
            confidence,
            reasoning,
        }
    }
}
