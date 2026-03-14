// =============================================================================
// web_ui.rs — Web interface plugin (axum + WebSocket)
//
// Role: This plugin manages communication with the web interface via WebSocket.
// At each brain event, it serializes the event to JSON and
// requests its broadcast to all connected WebSocket clients.
//
// Dependencies:
//   - super: Plugin trait, BrainEvent, PluginAction (plugin system)
//   - serde_json: event serialization to JSON
//
// Place in architecture:
//   This plugin is registered in the PluginManager at startup.
//   It bridges the cognitive core (internal events) and
//   the web user interface (real-time display).
//   WebSocket broadcasting is handled by the axum server in main.rs.
// =============================================================================

use super::{Plugin, BrainEvent, PluginAction};

/// Web UI plugin.
/// Reacts to each brain event by serializing it to JSON
/// and requesting its broadcast via WebSocket.
pub struct WebUiPlugin {
    /// Last state serialized to JSON, kept to be able to send it
    /// to newly connecting clients (state catch-up).
    last_state_json: String,
}

impl Default for WebUiPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl WebUiPlugin {
    /// Creates a new WebUI plugin with an empty initial state.
    pub fn new() -> Self {
        Self {
            last_state_json: "{}".into(),
        }
    }

    /// Returns the last state serialized to JSON.
    /// Used to send the current state to clients that just connected.
    ///
    /// # Returns
    /// Reference to the JSON string of the last state
    pub fn last_state(&self) -> &str {
        &self.last_state_json
    }
}

impl Plugin for WebUiPlugin {
    /// Returns the plugin name.
    fn name(&self) -> &str {
        "WebUI"
    }

    /// Reacts to a brain event:
    ///   1. Serializes the event to JSON
    ///   2. Keeps the JSON as the last state
    ///   3. Returns a WebSocketBroadcast action to broadcast the JSON
    ///
    /// If serialization fails, no action is returned.
    ///
    /// # Parameters
    /// - `event`: the brain event to broadcast
    ///
    /// # Returns
    /// A vector containing a WebSocketBroadcast action, or empty on error
    fn on_event(&mut self, event: &BrainEvent) -> Vec<PluginAction> {
        // Serialize the event to JSON for WebSocket transport
        if let Ok(json) = serde_json::to_string(event) {
            self.last_state_json = json.clone();
            vec![PluginAction::WebSocketBroadcast { data: json }]
        } else {
            vec![]
        }
    }
}
