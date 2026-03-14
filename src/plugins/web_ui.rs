// =============================================================================
// web_ui.rs — Plugin interface web (axum + WebSocket)
//
// Role : Ce plugin gere la communication avec l'interface web via WebSocket.
// A chaque evenement du cerveau, il serialise l'evenement en JSON et
// demande sa diffusion a tous les clients WebSocket connectes.
//
// Dependances :
//   - super : trait Plugin, BrainEvent, PluginAction (systeme de plugins)
//   - serde_json : serialisation des evenements en JSON
//
// Place dans l'architecture :
//   Ce plugin est enregistre dans le PluginManager au demarrage.
//   Il fait le pont entre le coeur cognitif (evenements internes) et
//   l'interface utilisateur web (affichage en temps reel).
//   La diffusion WebSocket est geree par le serveur axum dans main.rs.
// =============================================================================

use super::{Plugin, BrainEvent, PluginAction};

/// Plugin Web UI (Interface Utilisateur Web).
/// Reagit a chaque evenement du cerveau en le serialisant en JSON
/// et en demandant sa diffusion via WebSocket.
pub struct WebUiPlugin {
    /// Dernier etat serialise en JSON, conserve pour pouvoir le renvoyer
    /// aux nouveaux clients qui se connectent (rattrapage d'etat).
    last_state_json: String,
}

impl Default for WebUiPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl WebUiPlugin {
    /// Cree un nouveau plugin WebUI avec un etat initial vide.
    pub fn new() -> Self {
        Self {
            last_state_json: "{}".into(),
        }
    }

    /// Retourne le dernier etat serialise en JSON.
    /// Utilise pour envoyer l'etat actuel aux clients qui viennent de se connecter.
    ///
    /// # Retour
    /// Reference vers la chaine JSON du dernier etat
    pub fn last_state(&self) -> &str {
        &self.last_state_json
    }
}

impl Plugin for WebUiPlugin {
    /// Retourne le nom du plugin.
    fn name(&self) -> &str {
        "WebUI"
    }

    /// Reagit a un evenement du cerveau :
    ///   1. Serialise l'evenement en JSON
    ///   2. Conserve le JSON comme dernier etat
    ///   3. Retourne une action WebSocketBroadcast pour diffuser le JSON
    ///
    /// Si la serialisation echoue, aucune action n'est renvoyee.
    ///
    /// # Parametres
    /// - `event` : l'evenement du cerveau a diffuser
    ///
    /// # Retour
    /// Un vecteur contenant une action WebSocketBroadcast, ou vide en cas d'erreur
    fn on_event(&mut self, event: &BrainEvent) -> Vec<PluginAction> {
        // Serialiser l'evenement en JSON pour le transport WebSocket
        if let Ok(json) = serde_json::to_string(event) {
            self.last_state_json = json.clone();
            vec![PluginAction::WebSocketBroadcast { data: json }]
        } else {
            vec![]
        }
    }
}
