// =============================================================================
// api/state.rs — Shared application state and control messages
//
// This module defines:
// - `AppState`: the shared state accessible by both HTTP/WebSocket handlers
//   and the agent's main lifecycle loop.
// - `ControlMessage`: an enum of steering commands sent from the web UI
//   to the agent's lifecycle loop (e.g. configuration changes, emergency
//   stabilization, factory reset).
// =============================================================================

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::{mpsc, broadcast, Mutex};

use crate::agent::SaphireAgent;
use crate::agent::lifecycle::{SharedState, UserMessage};
use crate::logging::SaphireLogger;
use crate::logging::db::LogsDb;

/// Control messages sent from the web interface to the agent's main loop.
///
/// Each variant maps to a configuration or steering action that the operator
/// can trigger via the web UI. These messages travel through an `mpsc` channel
/// and are consumed by the lifecycle loop.
#[derive(Debug)]
pub enum ControlMessage {
    /// Set the baseline value for a neurotransmitter molecule.
    /// - `molecule`: name of the neurotransmitter (e.g. "dopamine", "cortisol").
    /// - `value`: new baseline reference value (typically in 0.0..1.0).
    SetBaseline { molecule: String, value: f64 },
    /// Modify the weight of a brain module (reptilian, limbic, neocortex).
    /// - `module`: name of the brain module.
    /// - `value`: new weight factor.
    SetModuleWeight { module: String, value: f64 },
    /// Adjust a decision threshold (yes/no boundary).
    /// - `which`: identifier of the threshold to adjust.
    /// - `value`: new threshold value.
    SetThreshold { which: String, value: f64 },
    /// Modify a general parameter (temperature, thought interval, etc.).
    /// - `param`: parameter name.
    /// - `value`: new parameter value.
    SetParam { param: String, value: f64 },
    /// Emergency stabilization: resets all neurochemistry to baseline values.
    EmergencyStabilize,
    /// Suggest a topic for the agent to reflect on.
    /// - `topic`: the proposed subject for autonomous reflection.
    SuggestTopic { topic: String },
    /// Reset to factory defaults.
    /// - `level`: scope of the reset (ChemistryOnly, ParametersOnly, FullReset, etc.).
    FactoryReset { level: crate::factory::ResetLevel },
    /// Request the current configuration. The response is sent back via a oneshot channel.
    /// - `response_tx`: one-shot channel to send the configuration JSON back to the caller.
    GetConfig { response_tx: tokio::sync::oneshot::Sender<serde_json::Value> },
    /// Request the current neurochemical state. The response is sent back via a oneshot channel.
    /// - `response_tx`: one-shot channel to send the chemistry JSON back to the caller.
    GetChemistry { response_tx: tokio::sync::oneshot::Sender<serde_json::Value> },
}

/// Shared application state, accessible by the web server and the main lifecycle loop.
///
/// This struct is cheaply cloneable (all fields are `Arc`-wrapped or `Clone`) and is
/// injected into every Axum handler via `State<AppState>`. It bridges the HTTP/WebSocket
/// layer with the core agent engine.
#[derive(Clone)]
pub struct AppState {
    /// Broadcast sender for pushing JSON messages to all connected WebSocket clients.
    pub ws_tx: Arc<broadcast::Sender<String>>,
    /// Channel sender for forwarding user chat messages to the main lifecycle loop.
    pub user_tx: mpsc::Sender<UserMessage>,
    /// Channel sender for forwarding control messages (configuration changes) to the main loop.
    pub ctrl_tx: mpsc::Sender<ControlMessage>,
    /// Atomic shutdown flag: when set to `true`, the lifecycle loop terminates gracefully.
    pub shutdown: Arc<AtomicBool>,
    /// Shared reference to the agent, protected by an async mutex.
    pub agent: Arc<Mutex<SaphireAgent>>,
    /// Dedicated broadcast channel for the monitoring dashboard (separate from the main WS).
    pub dashboard_tx: Arc<broadcast::Sender<String>>,
    /// Optional shared centralized logger.
    pub logger: Option<Arc<Mutex<SaphireLogger>>>,
    /// Optional logs database handle for structured log storage and querying.
    pub logs_db: Option<Arc<LogsDb>>,
    /// API key for Bearer token authentication (`None` disables authentication).
    pub api_key: Option<String>,
    /// Allowed origins for CORS headers and WebSocket origin checks.
    pub allowed_origins: Vec<String>,
    /// Per-IP rate limiter shared across all requests.
    pub rate_limiter: Arc<crate::api::middleware::RateLimiter>,
}

// Conversion from `AppState` to `SharedState` for compatibility with the lifecycle module.
// `SharedState` is a reduced view containing only the essential channels needed by the
// lifecycle loop (WebSocket broadcast, user message sender, and shutdown flag).
impl From<AppState> for SharedState {
    fn from(s: AppState) -> Self {
        SharedState {
            ws_tx: s.ws_tx,
            user_tx: s.user_tx,
            shutdown: s.shutdown,
        }
    }
}
