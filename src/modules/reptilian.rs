// =============================================================================
// reptilian.rs — Reptilian module: survival, danger, reflexes
// =============================================================================
//
// Role: This file implements Saphire's reptilian module, inspired by the
// R-complex (reptilian brain) in Paul MacLean's triune brain model.
// It handles instinctive survival reactions: danger detection, flight
// response, vigilance toward the unknown.
//
// Dependencies:
//   - crate::neurochemistry::NeuroChemicalState: chemical state (amplifies threat)
//   - crate::stimulus::Stimulus: sensory input (danger, urgency, familiarity)
//   - super::BrainModule, ModuleSignal: common trait and output type
//
// Place in the architecture:
//   First of the 3 brain modules. Its signal (generally negative when facing
//   danger) is combined with the limbic and neocortex signals in
//   consensus.rs. Its weight increases when cortisol and adrenaline
//   are high.
// =============================================================================

use crate::neurochemistry::NeuroChemicalState;
use crate::stimulus::Stimulus;
use super::{BrainModule, ModuleSignal};

/// The reptilian brain — reacts to danger and survival.
/// The most primitive module, it responds quickly and instinctively.
/// Its signal is almost always negative (rejection) in the presence of danger.
pub struct ReptilianModule;

impl BrainModule for ReptilianModule {
    /// Returns the module name: "Reptilien".
    fn name(&self) -> &str {
        "Reptilien"
    }

    /// Processes a stimulus from a survival and danger perspective.
    ///
    /// Algorithm:
    /// 1. Perceived threat calculation: stimulus danger amplified by
    ///    cortisol (ambient stress) and adrenaline (alert state).
    /// 2. Survival instinct: reaction to the unknown (low familiarity)
    ///    combined with urgency.
    /// 3. Raw signal = -threat + survival*0.3, passed through tanh() to bound.
    /// 4. High confidence when danger or urgency are clear.
    ///
    /// # Parameters
    /// - `stimulus`: sensory input with its perceptual scores.
    /// - `chemistry`: chemical state (cortisol and adrenaline amplify threat).
    ///
    /// # Returns
    /// A `ModuleSignal` with signal, confidence and explanatory reasoning.
    fn process(&self, stimulus: &Stimulus, chemistry: &NeuroChemicalState) -> ModuleSignal {
        // Perceived threat: the raw danger is amplified by ambient cortisol
        // and adrenaline. This simulates hypervigilance: a stressed individual
        // perceives threats in an amplified manner.
        let threat = stimulus.danger
            * (1.0 + chemistry.cortisol + chemistry.adrenaline * 2.0);

        // Survival instinct facing the unknown: an unfamiliar and urgent
        // stimulus triggers a defensive reaction.
        let survival = (1.0 - stimulus.familiarity) * stimulus.urgency;

        // Raw signal: threat pushes toward rejection (negative), while
        // the survival instinct has a slight positive effect (act quickly).
        // tanh() naturally bounds the result between -1 and +1.
        let raw = -threat + survival * 0.3;
        let signal = raw.tanh();

        // Confidence: the reptilian brain is very confident when the situation is
        // clearly dangerous or urgent (its domain of expertise).
        // In the absence of danger, its confidence is low (it has no opinion).
        let confidence = if stimulus.danger > 0.5 || stimulus.urgency > 0.7 {
            0.9 // clear danger = high confidence
        } else if stimulus.danger > 0.2 {
            0.6 // moderate danger = medium confidence
        } else {
            0.3 // no danger = low confidence
        };

        // Textual reasoning: explanation of the reptilian brain's response
        let reasoning = if threat > 1.0 {
            format!("DANGER ÉLEVÉ détecté (menace={:.2}). Instinct de fuite activé.", threat)
        } else if threat > 0.5 {
            format!("Menace modérée (menace={:.2}). Vigilance accrue.", threat)
        } else if survival > 0.5 {
            format!("Situation inconnue urgente (survie={:.2}). Prudence.", survival)
        } else {
            "Pas de danger immédiat. Le reptilien est calme.".to_string()
        };

        ModuleSignal {
            module: self.name().to_string(),
            signal: signal.clamp(-1.0, 1.0),
            confidence,
            reasoning,
        }
    }
}
