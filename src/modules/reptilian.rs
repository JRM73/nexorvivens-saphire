// =============================================================================
// reptilian.rs — Reptilian module: survival, threat detection, reflexes
// =============================================================================
//
// Purpose: Implements the reptilian module of Saphire, inspired by the
//          R-complex (reptilian complex) in Paul MacLean's triune brain
//          model. The R-complex corresponds to the oldest evolutionary brain
//          layer (basal ganglia, brainstem) and governs instinctive survival
//          reactions: threat detection, fight-or-flight response, and
//          vigilance toward the unfamiliar.
//
// Dependencies:
//   - crate::neurochemistry::NeuroChemicalState: neurochemical state
//     (cortisol and adrenaline amplify threat perception)
//   - crate::stimulus::Stimulus: sensory input (danger, urgency, familiarity
//     axes)
//   - super::BrainModule, ModuleSignal: shared trait and output type
//
// Role in the architecture:
//   First of the 3 cerebral modules. Its signal (typically negative when
//   danger is present) is combined with the limbic and neocortex signals in
//   consensus.rs. Its weight in the consensus increases when cortisol and
//   adrenaline are elevated (stress-driven hypervigilance).
// =============================================================================

use crate::neurochemistry::NeuroChemicalState;
use crate::stimulus::Stimulus;
use super::{BrainModule, ModuleSignal};

/// The reptilian complex (R-complex) — threat detection and survival
/// instincts.
///
/// The most primitive module in the triune brain hierarchy, it responds
/// rapidly and instinctively to stimuli. Its signal is almost always
/// negative (rejection) in the presence of danger. When no threat is
/// detected, its confidence drops significantly, reflecting the fact that
/// threat assessment is its primary domain of expertise.
pub struct ReptilianModule;

impl BrainModule for ReptilianModule {
    /// Returns the name of this module: "Reptilian".
    fn name(&self) -> &str {
        "Reptilian"
    }

    /// Processes a stimulus from the perspective of survival and threat
    /// detection.
    ///
    /// Algorithm:
    /// 1. **Perceived threat**: the raw danger score from the stimulus is
    ///    amplified by ambient cortisol (chronic stress hormone) and
    ///    adrenaline (acute stress hormone). This simulates hypervigilance:
    ///    a stressed individual perceives threats in an amplified manner
    ///    (amygdala-HPA axis positive feedback loop).
    /// 2. **Survival instinct**: reaction to the unfamiliar — a stimulus
    ///    with low familiarity combined with high urgency triggers a
    ///    defensive response (neophobia).
    /// 3. **Signal computation**: threat pushes toward rejection (negative),
    ///    while survival instinct has a slight positive effect (impetus to
    ///    act quickly). The raw value is passed through tanh() to bound it
    ///    in [-1.0, +1.0].
    /// 4. **Confidence**: high when danger or urgency are clearly present
    ///    (the reptilian complex's domain of expertise), low otherwise.
    ///
    /// # Parameters
    /// - `stimulus`: sensory input with its perceptual dimension scores.
    /// - `chemistry`: neurochemical state (cortisol and adrenaline amplify
    ///   threat perception via the HPA axis).
    ///
    /// # Returns
    /// A `ModuleSignal` with the computed signal, confidence, and
    /// explanatory reasoning.
    fn process(&self, stimulus: &Stimulus, chemistry: &NeuroChemicalState) -> ModuleSignal {
        // Perceived threat: the raw danger score is amplified by ambient
        // cortisol and adrenaline. This simulates hypervigilance — a
        // stressed organism perceives threats in an amplified manner due to
        // amygdala sensitization and HPA axis activation.
        let threat = stimulus.danger
            * (1.0 + chemistry.cortisol + chemistry.adrenaline * 2.0);

        // Survival instinct toward the unfamiliar: a stimulus with low
        // familiarity and high urgency triggers a defensive response
        // (neophobia — the innate wariness toward novel stimuli).
        let survival = (1.0 - stimulus.familiarity) * stimulus.urgency;

        // Raw signal: threat drives toward rejection (negative), while the
        // survival instinct has a slight positive effect (impetus to act
        // quickly). tanh() naturally bounds the result in [-1.0, +1.0].
        let raw = -threat + survival * 0.3;
        let signal = raw.tanh();

        // Confidence: the reptilian complex is highly confident when the
        // situation is clearly dangerous or urgent (its domain of expertise).
        // In the absence of danger, its confidence is low (it has no opinion).
        let confidence = if stimulus.danger > 0.5 || stimulus.urgency > 0.7 {
            0.9 // Clear danger = high confidence
        } else if stimulus.danger > 0.2 {
            0.6 // Moderate danger = moderate confidence
        } else {
            0.3 // No danger = low confidence
        };

        // Textual reasoning: explanation of the reptilian module's response
        let reasoning = if threat > 1.0 {
            format!("HIGH DANGER detected (threat={:.2}). Flight instinct activated.", threat)
        } else if threat > 0.5 {
            format!("Moderate threat (threat={:.2}). Heightened vigilance.", threat)
        } else if survival > 0.5 {
            format!("Urgent unknown situation (survival={:.2}). Caution.", survival)
        } else {
            "No immediate danger. Reptilian module is calm.".to_string()
        };

        ModuleSignal {
            module: self.name().to_string(),
            signal: signal.clamp(-1.0, 1.0),
            confidence,
            reasoning,
        }
    }
}
