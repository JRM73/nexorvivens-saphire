// =============================================================================
// api/websocket.rs — WebSocket handlers (main chat and dashboard)
//
// This module manages WebSocket connections for two distinct channels:
// 1. Main WebSocket (/ws): bidirectional real-time chat and control commands
//    between the web UI and the agent's lifecycle loop.
// 2. Dashboard WebSocket (/ws/dashboard): unidirectional stream of monitoring
//    data (logs, metrics) pushed to the dashboard UI.
//
// Both handlers verify the Origin header before accepting connections.
// =============================================================================

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use futures::{StreamExt, SinkExt};
use tokio::sync::broadcast;

use super::state::{AppState, ControlMessage};
use super::middleware::check_ws_origin;
use crate::agent::lifecycle::UserMessage;

/// GET /ws -- Main WebSocket upgrade handler.
///
/// Promotes the HTTP connection to a WebSocket when a client connects to /ws.
/// The Origin header is verified against the allowed origins list before accepting.
///
/// # Arguments
/// * `headers` - HTTP request headers (used for Origin verification).
/// * `ws` - Axum WebSocket upgrade extractor.
/// * `state` - Shared application state.
///
/// # Returns
/// A WebSocket upgrade response, or HTTP 403 Forbidden if the Origin is not allowed.
pub async fn ws_handler(
    headers: axum::http::HeaderMap,
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<AppState>,
) -> axum::response::Response {
    if !check_ws_origin(&headers, &state.allowed_origins) {
        return (StatusCode::FORBIDDEN, "Origin not allowed").into_response();
    }
    ws.on_upgrade(move |socket| handle_ws(socket, state)).into_response()
}

/// GET /ws/dashboard -- Dedicated dashboard monitoring WebSocket upgrade handler.
///
/// Similar to `ws_handler`, but routes to a separate broadcast channel used exclusively
/// for dashboard monitoring data. Verifies the Origin header before accepting.
///
/// # Arguments
/// * `headers` - HTTP request headers (used for Origin verification).
/// * `ws` - Axum WebSocket upgrade extractor.
/// * `state` - Shared application state.
///
/// # Returns
/// A WebSocket upgrade response, or HTTP 403 Forbidden if the Origin is not allowed.
pub async fn ws_dashboard_handler(
    headers: axum::http::HeaderMap,
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<AppState>,
) -> axum::response::Response {
    if !check_ws_origin(&headers, &state.allowed_origins) {
        return (StatusCode::FORBIDDEN, "Origin not allowed").into_response();
    }
    ws.on_upgrade(move |socket| handle_ws_dashboard(socket, state)).into_response()
}

/// Handles an individual main WebSocket connection.
///
/// Two concurrent tasks are spawned:
/// 1. **Send task**: relays broadcast messages from the agent to the connected client.
/// 2. **Receive task**: processes incoming messages from the client, which may be:
///    - JSON control commands (identified by a `"type"` field): routed to the control channel.
///    - JSON chat messages (type `"chat"`): forwarded to the user message channel.
///    - Plain text: treated as chat messages for backward compatibility.
///
/// The connection is maintained until either task completes (client disconnect or
/// broadcast channel closure).
///
/// # Arguments
/// * `socket` - The established WebSocket connection.
/// * `state` - Shared application state containing broadcast and message channels.
async fn handle_ws(socket: axum::extract::ws::WebSocket, state: AppState) {
    // Subscribe to the broadcast channel to receive messages from the agent
    let mut rx = state.ws_tx.subscribe();
    // Split the socket into separate sender and receiver halves for concurrent use
    let (mut sender, mut receiver) = socket.split();
    let user_tx = state.user_tx.clone();
    let ctrl_tx = state.ctrl_tx.clone();
    // Send task: broadcast channel -> WebSocket client
    // Every message published by the agent is forwarded to this connected client.
    let send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    let ws_msg = axum::extract::ws::Message::Text(msg);
                    if sender.send(ws_msg).await.is_err() {
                        break; // Client disconnected
                    }
                },
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::debug!("WS broadcast: {} messages skipped (slow client)", n);
                    continue; // Ignore lag, do not disconnect
                },
                Err(_) => break, // Channel closed
            }
        }
    });

    // Receive task: WebSocket client -> agent
    // JSON messages with a "type" field are interpreted as control commands.
    // All other messages are treated as plain-text chat input.
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let axum::extract::ws::Message::Text(text) = msg {
                let text_str: String = text;

                // Attempt to parse the message as a JSON control command
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text_str) {
                    if let Some(msg_type) = json.get("type").and_then(|t| t.as_str()) {
                        match msg_type {
                            // Command: set baseline value for a neurotransmitter molecule
                            "set_baseline" => {
                                if let (Some(mol), Some(val)) = (
                                    json.get("molecule").and_then(|m| m.as_str()),
                                    json.get("value").and_then(|v| v.as_f64()),
                                ) {
                                    let _ = ctrl_tx.send(ControlMessage::SetBaseline {
                                        molecule: mol.to_string(), value: val,
                                    }).await;
                                }
                                continue;
                            },
                            // Command: modify the weight of a brain module
                            "set_module_weight" => {
                                if let (Some(m), Some(v)) = (
                                    json.get("module").and_then(|m| m.as_str()),
                                    json.get("value").and_then(|v| v.as_f64()),
                                ) {
                                    let _ = ctrl_tx.send(ControlMessage::SetModuleWeight {
                                        module: m.to_string(), value: v,
                                    }).await;
                                }
                                continue;
                            },
                            // Command: adjust a decision threshold
                            "set_threshold" => {
                                if let (Some(w), Some(v)) = (
                                    json.get("which").and_then(|w| w.as_str()),
                                    json.get("value").and_then(|v| v.as_f64()),
                                ) {
                                    let _ = ctrl_tx.send(ControlMessage::SetThreshold {
                                        which: w.to_string(), value: v,
                                    }).await;
                                }
                                continue;
                            },
                            // Command: modify a general parameter
                            "set_param" => {
                                if let (Some(p), Some(v)) = (
                                    json.get("param").and_then(|p| p.as_str()),
                                    json.get("value").and_then(|v| v.as_f64()),
                                ) {
                                    let _ = ctrl_tx.send(ControlMessage::SetParam {
                                        param: p.to_string(), value: v,
                                    }).await;
                                }
                                continue;
                            },
                            // Command: emergency neurochemical stabilization
                            "emergency_stabilize" => {
                                let _ = ctrl_tx.send(ControlMessage::EmergencyStabilize).await;
                                continue;
                            },
                            // Command: suggest a topic for autonomous reflection
                            "suggest_topic" => {
                                if let Some(topic) = json.get("topic").and_then(|t| t.as_str()) {
                                    let _ = ctrl_tx.send(ControlMessage::SuggestTopic {
                                        topic: topic.to_string(),
                                    }).await;
                                }
                                continue;
                            },
                            // Command: factory reset to default values
                            "factory_reset" => {
                                let level_str = json.get("level").and_then(|l| l.as_str()).unwrap_or("chemistry_only");
                                let level = match level_str {
                                    "chemistry_only" | "ChemistryOnly" => crate::factory::ResetLevel::ChemistryOnly,
                                    "parameters_only" | "ParametersOnly" => crate::factory::ResetLevel::ParametersOnly,
                                    "senses_only" | "SensesOnly" => crate::factory::ResetLevel::SensesOnly,
                                    "intuition_only" | "IntuitionOnly" => crate::factory::ResetLevel::IntuitionOnly,
                                    "personal_ethics_only" | "PersonalEthicsOnly" => crate::factory::ResetLevel::PersonalEthicsOnly,
                                    "psychology_only" | "PsychologyOnly" => crate::factory::ResetLevel::PsychologyOnly,
                                    "full_reset" | "FullReset" => crate::factory::ResetLevel::FullReset,
                                    _ => continue,
                                };
                                let _ = ctrl_tx.send(ControlMessage::FactoryReset { level }).await;
                                continue;
                            },
                            // Chat message with speaker identification
                            "chat" => {
                                let chat_text = json.get("text")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let username = json.get("username")
                                    .and_then(|u| u.as_str())
                                    .unwrap_or("Inconnu")
                                    .to_string();
                                if !chat_text.is_empty() {
                                    tracing::info!("WS chat recu de '{}': '{}'", &username, &chat_text);
                                    let _ = user_tx.send(UserMessage {
                                        text: chat_text,
                                        username,
                                    }).await;
                                }
                                continue;
                            },
                            _ => {} // Unknown type: fall through and treat as plain-text chat
                        }
                    }
                }

                // Not a JSON control command or a JSON chat message:
                // treat as raw text for backward compatibility
                tracing::info!("WS chat recu: '{}'", &text_str);
                let _ = user_tx.send(UserMessage {
                    text: text_str,
                    username: "Inconnu".to_string(),
                }).await;
            }
        }
    });

    // Wait for either task to complete (client disconnect or broadcast channel closure)
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

/// Handles a dashboard WebSocket connection.
///
/// This is a unidirectional (server-to-client) connection that streams real-time
/// monitoring data from the dedicated dashboard broadcast channel. The client
/// receiver is kept alive but not actively read.
///
/// # Arguments
/// * `socket` - The established WebSocket connection.
/// * `state` - Shared application state containing the dashboard broadcast channel.
async fn handle_ws_dashboard(socket: axum::extract::ws::WebSocket, state: AppState) {
    let mut rx = state.dashboard_tx.subscribe();
    let (mut sender, mut _receiver) = socket.split();

    // Forward real-time monitoring data to the dashboard client
    while let Ok(msg) = rx.recv().await {
        let ws_msg = axum::extract::ws::Message::Text(msg);
        if sender.send(ws_msg).await.is_err() {
            break;
        }
    }
}
