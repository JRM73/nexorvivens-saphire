// =============================================================================
// api/websocket.rs — WebSocket handlers (main and dashboard)
//
// Role: WebSocket connection management for real-time chat (main)
// and the dedicated monitoring dashboard.
// =============================================================================

use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use futures::{StreamExt, SinkExt};
use tokio::sync::broadcast;

use super::state::{AppState, ControlMessage};
use super::middleware::check_ws_origin;
use crate::agent::lifecycle::UserMessage;

/// Checks WebSocket authentication via query param `?token=` or Bearer header.
fn check_ws_auth(
    headers: &axum::http::HeaderMap,
    params: &HashMap<String, String>,
    api_key: &Option<String>,
) -> bool {
    match api_key {
        None => true, // No key configured = all allowed        Some(expected) => {
            // 1. Query param ?token=xxx
            if let Some(token) = params.get("token") {
                if token == expected {
                    return true;
                }
            }
            // 2. Header Authorization: Bearer xxx
            if let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
                if let Some(token) = auth.strip_prefix("Bearer ") {
                    if token == expected {
                        return true;
                    }
                }
            }
            false
        }
    }
}

/// GET /ws — WebSocket upgrade handler.
/// When a client connects to /ws, the HTTP connection is upgraded to WebSocket.
/// Checks origin and authentication before accepting the connection.
pub async fn ws_handler(
    headers: axum::http::HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<AppState>,
) -> axum::response::Response {
    if !check_ws_origin(&headers, &state.allowed_origins) {
        return (StatusCode::FORBIDDEN, "Origin not allowed").into_response();
    }
    if !check_ws_auth(&headers, &params, &state.api_key) {
        return (StatusCode::UNAUTHORIZED, "Token invalide").into_response();
    }
    ws.on_upgrade(move |socket| handle_ws(socket, state)).into_response()
}

/// GET /ws/dashboard — Dedicated WebSocket for the monitoring dashboard.
/// Checks origin and authentication before accepting the connection.
pub async fn ws_dashboard_handler(
    headers: axum::http::HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<AppState>,
) -> axum::response::Response {
    if !check_ws_origin(&headers, &state.allowed_origins) {
        return (StatusCode::FORBIDDEN, "Origin not allowed").into_response();
    }
    if !check_ws_auth(&headers, &params, &state.api_key) {
        return (StatusCode::UNAUTHORIZED, "Token invalide").into_response();
    }
    ws.on_upgrade(move |socket| handle_ws_dashboard(socket, state)).into_response()
}

/// Handles an individual WebSocket connection.
/// Two parallel tasks are spawned:
///  1. Send task: relays broadcast messages to the client
///  2. Receive task: processes client messages (chat or control)
///
/// # Parameters
/// - `socket`: the established WebSocket connection
/// - `state`: shared application state
async fn handle_ws(socket: axum::extract::ws::WebSocket, state: AppState) {
    // Subscribe to the broadcast channel to receive agent messages
    let mut rx = state.ws_tx.subscribe();
    // Split the socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();
    let user_tx = state.user_tx.clone();
    let ctrl_tx = state.ctrl_tx.clone();
    // Send task: broadcaster -> WebSocket client
    // Each message broadcast by the agent is forwarded to the connected client.
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
                    continue; // Ignore lag, do not disconnect                },
                Err(_) => break, // Channel closed
            }
        }
    });

    // Receive task: WebSocket client -> agent
    // JSON messages with a "type" field are interpreted as control commands.
    // Other messages are treated as chat text.
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let axum::extract::ws::Message::Text(text) = msg {
                let text_str: String = text;

                // Try to parse the message as a control JSON
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text_str) {
                    if let Some(msg_type) = json.get("type").and_then(|t| t.as_str()) {
                        match msg_type {
                            // Command: modify the baseline value of a molecule
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
                            // Command: emergency stabilization
                            "emergency_stabilize" => {
                                let _ = ctrl_tx.send(ControlMessage::EmergencyStabilize).await;
                                continue;
                            },
                            // Command: suggest a topic for reflection
                            "suggest_topic" => {
                                if let Some(topic) = json.get("topic").and_then(|t| t.as_str()) {
                                    let _ = ctrl_tx.send(ControlMessage::SuggestTopic {
                                        topic: topic.to_string(),
                                    }).await;
                                }
                                continue;
                            },
                            // Command: reset to factory defaults
                            "factory_reset" => {
                                let level_str = json.get("level").and_then(|l| l.as_str()).unwrap_or("chemistry_only");
                                let level = match level_str {
                                    "chemistry_only" | "ChemistryOnly" => crate::factory::ResetLevel::ChemistryOnly,
                                    "parameters_only" | "ParametersOnly" => crate::factory::ResetLevel::ParametersOnly,
                                    "senses_only" | "SensesOnly" => crate::factory::ResetLevel::SensesOnly,
                                    "intuition_only" | "IntuitionOnly" => crate::factory::ResetLevel::IntuitionOnly,
                                    "personal_ethics_only" | "PersonalEthicsOnly" => crate::factory::ResetLevel::PersonalEthicsOnly,
                                    "psychology_only" | "PsychologyOnly" => crate::factory::ResetLevel::PsychologyOnly,
                                    "sleep_only" | "SleepOnly" => crate::factory::ResetLevel::SleepOnly,
                                    "biology_reset" | "BiologyReset" => crate::factory::ResetLevel::BiologyReset,
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
                            _ => {} // Unknown type: treat as chat text                        }
                    }
                }

                // Not a control JSON nor a chat JSON:
                // treat as raw text (backward compatibility)
                tracing::info!("WS chat recu: '{}'", &text_str);
                let _ = user_tx.send(UserMessage {
                    text: text_str,
                    username: "Inconnu".to_string(),
                }).await;
            }
        }
    });

    // Wait for either task to finish (client or broadcast disconnection)
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

/// Dashboard WebSocket handler.
async fn handle_ws_dashboard(socket: axum::extract::ws::WebSocket, state: AppState) {
    let mut rx = state.dashboard_tx.subscribe();
    let (mut sender, mut _receiver) = socket.split();

    // Send logs in real-time to the dashboard
    while let Ok(msg) = rx.recv().await {
        let ws_msg = axum::extract::ws::Message::Text(msg);
        if sender.send(ws_msg).await.is_err() {
            break;
        }
    }
}
