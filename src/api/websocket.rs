// =============================================================================
// api/websocket.rs — Handlers WebSocket (principal et dashboard)
//
// Role : Gestion des connexions WebSocket pour le chat temps reel
// (principal) et le monitoring dashboard (dedie).
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

/// Verifie l'authentification WebSocket via query param `?token=` ou header Bearer.
fn check_ws_auth(
    headers: &axum::http::HeaderMap,
    params: &HashMap<String, String>,
    api_key: &Option<String>,
) -> bool {
    match api_key {
        None => true, // Pas de cle configuree = tout autorise
        Some(expected) => {
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

/// GET /ws — Handler de mise a niveau WebSocket.
/// Quand un client se connecte sur /ws, la connexion HTTP est promue en WebSocket.
/// Verifie l'origine et l'authentification avant d'accepter la connexion.
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

/// GET /ws/dashboard — WebSocket dedie au dashboard de monitoring.
/// Verifie l'origine et l'authentification avant d'accepter la connexion.
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

/// Gestion d'une connexion WebSocket individuelle.
/// Deux taches paralleles sont lancees :
///   1. Tache d'envoi : relaie les messages du broadcast vers le client
///   2. Tache de reception : traite les messages du client (chat ou controle)
///
/// # Parametres
/// - `socket` : la connexion WebSocket etablie
/// - `state` : etat partage de l'application
async fn handle_ws(socket: axum::extract::ws::WebSocket, state: AppState) {
    // S'abonner au canal broadcast pour recevoir les messages de l'agent
    let mut rx = state.ws_tx.subscribe();
    // Separer le socket en emetteur et recepteur
    let (mut sender, mut receiver) = socket.split();
    let user_tx = state.user_tx.clone();
    let ctrl_tx = state.ctrl_tx.clone();
    // Tache d'envoi : broadcaster -> client WebSocket
    // Chaque message diffuse par l'agent est transmis au client connecte.
    let send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    let ws_msg = axum::extract::ws::Message::Text(msg);
                    if sender.send(ws_msg).await.is_err() {
                        break; // Client deconnecte
                    }
                },
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::debug!("WS broadcast: {} messages sautes (client lent)", n);
                    continue; // Ignorer le lag, ne pas deconnecter
                },
                Err(_) => break, // Canal ferme
            }
        }
    });

    // Tache de reception : client WebSocket -> agent
    // Les messages JSON avec un champ "type" sont interpretes comme des commandes de controle.
    // Les autres messages sont traites comme du texte de chat.
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let axum::extract::ws::Message::Text(text) = msg {
                let text_str: String = text;

                // Tenter de parser le message comme un JSON de controle
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text_str) {
                    if let Some(msg_type) = json.get("type").and_then(|t| t.as_str()) {
                        match msg_type {
                            // Commande : modifier la valeur de base d'une molecule
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
                            // Commande : modifier le poids d'un module cerebral
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
                            // Commande : ajuster un seuil de decision
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
                            // Commande : modifier un parametre general
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
                            // Commande : stabilisation d'urgence
                            "emergency_stabilize" => {
                                let _ = ctrl_tx.send(ControlMessage::EmergencyStabilize).await;
                                continue;
                            },
                            // Commande : suggerer un sujet de reflexion
                            "suggest_topic" => {
                                if let Some(topic) = json.get("topic").and_then(|t| t.as_str()) {
                                    let _ = ctrl_tx.send(ControlMessage::SuggestTopic {
                                        topic: topic.to_string(),
                                    }).await;
                                }
                                continue;
                            },
                            // Commande : reset aux valeurs d'usine
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
                            // Message de chat avec identification de l'interlocuteur
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
                            _ => {} // Type inconnu : traiter comme du texte de chat
                        }
                    }
                }

                // Ce n'est pas un JSON de controle ni un chat JSON :
                // traiter comme du texte brut (retrocompatibilite)
                tracing::info!("WS chat recu: '{}'", &text_str);
                let _ = user_tx.send(UserMessage {
                    text: text_str,
                    username: "Inconnu".to_string(),
                }).await;
            }
        }
    });

    // Attendre que l'une des deux taches se termine (deconnexion du client ou du broadcast)
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

/// Gestion du WebSocket dashboard.
async fn handle_ws_dashboard(socket: axum::extract::ws::WebSocket, state: AppState) {
    let mut rx = state.dashboard_tx.subscribe();
    let (mut sender, mut _receiver) = socket.split();

    // Envoyer les logs en temps reel au dashboard
    while let Ok(msg) = rx.recv().await {
        let ws_msg = axum::extract::ws::Message::Text(msg);
        if sender.send(ws_msg).await.is_err() {
            break;
        }
    }
}
