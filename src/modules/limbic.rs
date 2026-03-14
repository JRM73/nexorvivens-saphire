// =============================================================================
// limbic.rs — Limbic module: emotions, reward, social bonds
// =============================================================================
//
// Role: This file implements Saphire's limbic module, inspired by the
// limbic system in Paul MacLean's triune brain model. It handles
// emotional reactions: fear (amygdala), pleasure (reward circuit),
// social attachment (oxytocin) and resilience (endorphins).
//
// Dependencies:
//   - crate::neurochemistry::NeuroChemicalState: chemical state (dopamine,
//     oxytocin, serotonin, endorphin influence processing)
//   - crate::stimulus::Stimulus: sensory input (danger, reward, social)
//   - super::BrainModule, ModuleSignal: common trait and output type
//
// Place in the architecture:
//   Second of the 3 brain modules. Its signal reflects the emotional
//   reaction to the stimulus. Its weight in the consensus increases when
//   dopamine, serotonin and oxytocin are high.
// =============================================================================

use crate::neurochemistry::NeuroChemicalState;
use crate::stimulus::Stimulus;
use super::{BrainModule, ModuleSignal};

/// The limbic system — emotional processing, empathy and reward circuit.
/// This module represents Saphire's emotional responses, including:
/// - The amygdala (fear reaction to danger)
/// - The reward circuit (attraction to pleasure)
/// - Social bonding (empathy, attachment)
/// - Emotional resilience (endorphins)
pub struct LimbicModule;

impl BrainModule for LimbicModule {
    /// Returns the module name: "Limbique".
    fn name(&self) -> &str {
        "Limbique"
    }

    /// Processes a stimulus from an emotional perspective.
    ///
    /// Algorithm with 4 components:
    /// 1. Amygdala: negative reaction proportional to danger (factor 0.8).
    /// 2. Reward circuit: attraction proportional to reward,
    ///    amplified by dopamine.
    /// 3. Social bonding: social component amplified by oxytocin.
    /// 4. Resilience: endorphins attenuate emotional pain.
    ///    The final signal also integrates serotonin (baseline well-being).
    ///
    /// # Parameters
    /// - `stimulus`: sensory input with its perceptual scores.
    /// - `chemistry`: chemical state (dopamine, oxytocin, serotonin,
    ///   endorphin modulate the response).
    ///
    /// # Returns
    /// A `ModuleSignal` with signal, confidence and explanatory reasoning.
    fn process(&self, stimulus: &Stimulus, chemistry: &NeuroChemicalState) -> ModuleSignal {
        // Amygdala: instinctive emotional reaction to danger.
        // The negative sign reflects rejection/fear. The 0.8 factor is
        // slightly below 1.0 because the limbic system is less "brutal"
        // than the reptilian brain when facing danger.
        let amygdala = -stimulus.danger * 0.8;

        // Reward circuit: dopamine amplifies the attraction of the
        // reward. When dopamine is high, even a modest reward becomes
        // very attractive (motivational bias).
        let reward = stimulus.reward * (1.0 + chemistry.dopamine);

        // Social bonding: oxytocin amplifies sensitivity to social
        // interactions. Base of 0.5 to guarantee a minimum of social receptivity.
        let social = stimulus.social * (0.5 + chemistry.oxytocin * 0.5);

        // Resilience: endorphins attenuate emotional pain
        // and add a slight positive bias (ability to absorb hits).
        let resilience = chemistry.endorphin * 0.2;

        // Raw signal: sum of the 4 components + baseline well-being (serotonin).
        // tanh() naturally bounds the result between -1 and +1.
        let raw = amygdala + reward + social + chemistry.serotonin * 0.3 + resilience;
        let signal = raw.tanh();

        // Fixed confidence at 0.7: the limbic system is always fairly confident
        // because emotions are by nature felt with certainty
        // (one does not doubt what one feels).
        let confidence = 0.7;

        // Reasoning: dynamic construction by listing significant components
        // (above their respective thresholds).
        let parts: Vec<String> = [
            if amygdala.abs() > 0.2 {
                Some(format!("Amygdale réactive ({:.2})", amygdala))
            } else { None },
            if reward > 0.3 {
                Some(format!("Récompense attirante ({:.2})", reward))
            } else { None },
            if social > 0.2 {
                Some(format!("Lien social ressenti ({:.2})", social))
            } else { None },
        ].into_iter().flatten().collect();

        let reasoning = if parts.is_empty() {
            "Le limbique est neutre — pas de forte émotion.".to_string()
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
