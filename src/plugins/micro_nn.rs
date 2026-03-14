// =============================================================================
// micro_nn.rs — Plugin micro reseau de neurones (coque de compatibilite)
//
// Role : Plugin de compatibilite pour le systeme de plugins. Le reseau de
// neurones est maintenant un champ direct de SaphireAgent (micro_nn),
// cable dans le pipeline cognitif. Ce plugin reste enregistre pour recevoir
// les evenements mais ne porte plus le NN.
// =============================================================================

use super::{Plugin, BrainEvent, PluginAction};

/// Plugin micro reseau de neurones (coque de compatibilite).
/// Le NN est desormais dans SaphireAgent.micro_nn.
pub struct MicroNNPlugin;

impl MicroNNPlugin {
    /// Cree le plugin (coque vide pour compatibilite).
    pub fn new(_learning_rate: f64) -> Self {
        Self
    }
}

impl Plugin for MicroNNPlugin {
    fn name(&self) -> &str {
        "MicroNN"
    }

    fn on_event(&mut self, _event: &BrainEvent) -> Vec<PluginAction> {
        // Le NN est desormais cable directement dans pipeline.rs
        vec![]
    }
}
