// =============================================================================
// micro_nn.rs — Micro neural network plugin (compatibility shell)
//
// Role: Compatibility plugin for the plugin system. The neural network
// is now a direct field of SaphireAgent (micro_nn), wired into the
// cognitive pipeline. This plugin remains registered to receive
// events but no longer carries the NN.
// =============================================================================

use super::{Plugin, BrainEvent, PluginAction};

/// Micro neural network plugin (compatibility shell).
/// The NN is now in SaphireAgent.micro_nn.
pub struct MicroNNPlugin;

impl MicroNNPlugin {
    /// Creates the plugin (empty shell for compatibility).
    pub fn new(_learning_rate: f64) -> Self {
        Self
    }
}

impl Plugin for MicroNNPlugin {
    fn name(&self) -> &str {
        "MicroNN"
    }

    fn on_event(&mut self, _event: &BrainEvent) -> Vec<PluginAction> {
        // The NN is now wired directly in pipeline.rs
        vec![]
    }
}
