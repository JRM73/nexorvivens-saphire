// =============================================================================
// modules/mod.rs — The 3 cerebral modules of Saphire
// =============================================================================
//
// Purpose: This file declares the cerebral sub-modules and defines the shared
//          types used by all 3 modules: `ModuleSignal` (output of a module)
//          and the `BrainModule` trait (common interface).
//
// Dependencies:
//   - serde: serialization / deserialization
//   - crate::neurochemistry::NeuroChemicalState: neurochemical state
//     (processing parameter)
//   - crate::stimulus::Stimulus: sensory input (processing parameter)
//
// Role in the architecture:
//   This module groups the 3 biologically-inspired "brains" based on Paul
//   MacLean's triune brain model (1960s):
//     - reptilian.rs: the reptilian complex (R-complex) — survival instincts,
//       threat detection, reflexive reactions
//     - limbic.rs: the limbic system — emotions, reward circuitry, social
//       bonding
//     - neocortex.rs: the neocortex — rational analysis, cost/benefit
//       evaluation, executive function
//   Each module implements the `BrainModule` trait and emits a `ModuleSignal`.
//   The 3 signals are then combined by consensus.rs to produce the final
//   decision through weighted averaging.
// =============================================================================

/// Sub-module implementing the reptilian complex (R-complex): survival
/// reactions, threat detection, and fight-or-flight reflexes. Based on
/// MacLean's concept of the oldest evolutionary brain layer.
pub mod reptilian;

/// Sub-module implementing the limbic system: emotional processing, reward
/// circuitry (dopamine-mediated), and social bonding (oxytocin-mediated).
/// Corresponds to MacLean's paleomammalian brain.
pub mod limbic;

/// Sub-module implementing the neocortex: rational analysis, cost/benefit
/// evaluation, and executive function. Corresponds to MacLean's
/// neomammalian brain — the most recently evolved layer.
pub mod neocortex;

use serde::{Deserialize, Serialize};
use crate::neurochemistry::NeuroChemicalState;
use crate::stimulus::Stimulus;

/// Signal emitted by a cerebral module — the result of processing a stimulus.
///
/// Each module produces a unique signal that will be combined with the others
/// through weighted consensus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSignal {
    /// Name of the emitting module ("Reptilien", "Limbique", or "Neocortex").
    pub module: String,
    /// Signal value in [-1.0, +1.0]: the module's opinion on the stimulus.
    /// Negative values indicate rejection / perceived danger; positive values
    /// indicate approval / attraction.
    pub signal: f64,
    /// Confidence in the signal in [0.0, 1.0]: the module's certainty in its
    /// response. 1.0 = fully certain, 0.0 = no certainty whatsoever.
    pub confidence: f64,
    /// Textual reasoning: a human-readable explanation of how the module
    /// arrived at its signal value.
    pub reasoning: String,
}

/// Common trait for all 3 cerebral modules — the interface that each module
/// must implement to participate in the tri-cerebral consensus.
pub trait BrainModule {
    /// Returns the name of the module (used for display and logging).
    fn name(&self) -> &str;

    /// Processes a stimulus taking into account the current neurochemical
    /// state.
    ///
    /// # Parameters
    /// - `stimulus`: the sensory input to process, containing the 5
    ///   dimensional scores (danger, reward, urgency, social, novelty).
    /// - `chemistry`: the current neurochemical state, which modulates the
    ///   module's processing (e.g., cortisol amplifies threat perception,
    ///   dopamine amplifies reward sensitivity).
    ///
    /// # Returns
    /// A `ModuleSignal` containing the computed signal value, confidence
    /// level, and textual reasoning.
    fn process(&self, stimulus: &Stimulus, chemistry: &NeuroChemicalState) -> ModuleSignal;
}
