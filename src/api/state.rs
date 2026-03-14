// =============================================================================
// api/state.rs — Shared application state and control messages
//
// Role: Defines AppState (shared state between HTTP/WebSocket handlers and the
// main loop) and ControlMessage (control commands sent from the web
// interface to the life loop).
// =============================================================================

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::{mpsc, broadcast, Mutex};

use crate::agent::SaphireAgent;
use crate::agent::lifecycle::{SharedState, UserMessage};
use crate::logging::SaphireLogger;
use crate::logging::db::LogsDb;

/// Control messages sent from the web interface to the main loop.
/// Each variant corresponds to a configuration or control action
/// that the operator can trigger via the web UI (User Interface).
#[derive(Debug)]
pub enum ControlMessage {
    /// Set the baseline value of a molecule (e.g.: dopamine, cortisol).
    /// `molecule`: neurotransmitter name, `value`: new reference value.
    SetBaseline { molecule: String, value: f64 },
    /// Modify the weight of a brain module (reptilian, limbic, neocortex).
    /// `module`: module name, `value`: new weight.
    SetModuleWeight { module: String, value: f64 },
    /// Adjust a decision threshold (yes/no).
    /// `which`: threshold identifier, `value`: new value.
    SetThreshold { which: String, value: f64 },
    /// Modify a general parameter (temperature, thought interval, etc.).
    /// `param`: parameter name, `value`: new value.
    SetParam { param: String, value: f64 },
    /// Emergency stabilization: resets all neurochemistry to baseline values.
    EmergencyStabilize,
    /// Suggest a topic for the agent to reflect on.
    /// `topic`: the proposed topic.
    SuggestTopic { topic: String },
    /// Reset to factory defaults.
    /// `level`: reset level (ChemistryOnly, ParametersOnly, FullReset)
    FactoryReset { level: crate::factory::ResetLevel },
    /// Request the current configuration. The response is sent via a oneshot channel.
    /// `response_tx`: one-time response channel to send back the configuration JSON.
    GetConfig { response_tx: tokio::sync::oneshot::Sender<serde_json::Value> },
    /// Request the current neurochemical state. The response is sent via a oneshot channel.
    /// `response_tx`: one-time response channel to send back the chemistry JSON.
    GetChemistry { response_tx: tokio::sync::oneshot::Sender<serde_json::Value> },
}

/// Shared application state, accessible by the web server and the main loop.
/// This struct is cloned and shared between HTTP/WebSocket handlers and the agent's core.
#[derive(Clone)]
pub struct AppState {
    /// Broadcast sender to distribute JSON messages to WebSocket clients.
    pub ws_tx: Arc<broadcast::Sender<String>>,
    /// Channel for sending user messages (chat) to the main loop.
    pub user_tx: mpsc::Sender<UserMessage>,
    /// Channel for sending control messages (configuration) to the main loop.
    pub ctrl_tx: mpsc::Sender<ControlMessage>,
    /// Atomic shutdown flag: when set to true, the loop stops.
    pub shutdown: Arc<AtomicBool>,
    /// Shared reference to the agent (protected by an async Mutex).
    pub agent: Arc<Mutex<SaphireAgent>>,
    /// Dedicated broadcast channel for the dashboard (separate from main WS).
    pub dashboard_tx: Arc<broadcast::Sender<String>>,
    /// Shared centralized logger.
    pub logger: Option<Arc<Mutex<SaphireLogger>>>,
    /// Logs database (optional).
    pub logs_db: Option<Arc<LogsDb>>,
    /// API key for authentication (None = no auth)
    pub api_key: Option<String>,
    /// Allowed origins for CORS and WebSocket
    pub allowed_origins: Vec<String>,
    /// Per-IP rate limiter
    pub rate_limiter: Arc<crate::api::middleware::RateLimiter>,
}

// Conversion from AppState to SharedState for compatibility with the lifecycle module.
// SharedState is a reduced version containing only the essential channels.
impl From<AppState> for SharedState {
    fn from(s: AppState) -> Self {
        SharedState {
            ws_tx: s.ws_tx,
            user_tx: s.user_tx,
            shutdown: s.shutdown,
        }
    }
}
