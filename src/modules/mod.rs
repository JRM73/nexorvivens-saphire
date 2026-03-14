// =============================================================================
// modules/mod.rs — Saphire's 3 brain modules
// =============================================================================
//
// Role: This file declares the brain sub-modules and defines the common types
// shared by all 3 modules: `ModuleSignal` (module output) and the
// `BrainModule` trait (common interface).
//
// Dependencies:
//   - serde: serialization / deserialization
//   - crate::neurochemistry::NeuroChemicalState: chemical state (processing parameter)
//   - crate::stimulus::Stimulus: sensory input (processing parameter)
//
// Place in the architecture:
//   This module groups the 3 biologically inspired "brains":
//     - reptilian.rs: reptilian brain (survival, danger, reflexes)
//     - limbic.rs: limbic system (emotions, reward, social bonds)
//     - neocortex.rs: neocortex (rational analysis, cost/benefit)
//   Each module implements the `BrainModule` trait and emits a `ModuleSignal`.
//   The 3 signals are then combined by consensus.rs to produce
//   the final decision.
// =============================================================================

/// Reptilian sub-module: survival reactions and danger detection
pub mod reptilian;
/// Limbic sub-module: emotional processing and reward
pub mod limbic;
/// Neocortex sub-module: rational analysis and mental clarity
pub mod neocortex;

use serde::{Deserialize, Serialize};
use crate::neurochemistry::NeuroChemicalState;
use crate::stimulus::Stimulus;

/// Signal emitted by a brain module — result of processing a stimulus.
///
/// Each module produces a unique signal that will be combined with the others
/// in the weighted consensus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSignal {
    /// Name of the emitting module ("Reptilien", "Limbique" or "Neocortex")
    pub module: String,
    /// Signal [-1, +1]: the module's opinion on the stimulus.
    /// Negative = rejection / danger, positive = approval / attraction.
    pub signal: f64,
    /// Confidence in the signal [0, 1]: the module's certainty in its response.
    /// 1.0 = completely certain, 0.0 = no certainty.
    pub confidence: f64,
    /// Textual reasoning: explanation of the module's response.
    pub reasoning: String,
}

/// Common trait for the 3 brain modules — interface that each module
/// must implement to participate in the consensus.
pub trait BrainModule {
    /// Returns the module name (used for display and logging).
    fn name(&self) -> &str;

    /// Processes a stimulus taking into account the current neurochemical state.
    ///
    /// # Parameters
    /// - `stimulus`: sensory input to process.
    /// - `chemistry`: current chemical state (influences processing).
    ///
    /// # Returns
    /// A `ModuleSignal` containing the signal, confidence and reasoning.
    fn process(&self, stimulus: &Stimulus, chemistry: &NeuroChemicalState) -> ModuleSignal;
}
