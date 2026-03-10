// =============================================================================
// api/sensoria.rs — Pont entre Saphire et Sensoria (service sensoriel)
//
// Sensoria tourne sur une machine separee (i7, 192.168.1.129) et fournit
// les oreilles (STT), la bouche (TTS) et les yeux (vision) de Saphire.
//
// Endpoints :
//   POST /api/hear     — Sensoria envoie une transcription (ce que Saphire entend)
//   POST /api/speak    — Saphire demande a Sensoria de parler
//   GET  /api/sensoria — Statut de la connexion Sensoria
// =============================================================================

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use super::state::AppState;
use crate::agent::lifecycle::UserMessage;

const SENSORIA_HOST: &str = "192.168.1.129:9090";

// ─── Transcription entrante (Sensoria → Saphire) ─────────────────────────

#[derive(Deserialize)]
pub struct HearRequest {
    /// Texte transcrit par Whisper via Sensoria
    pub text: String,
    /// Source de la transcription
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "sensoria".into()
}

/// Recoit une transcription vocale depuis Sensoria.
/// Le texte est injecte dans le pipeline cognitif de Saphire
/// comme un message utilisateur (via le canal user_tx).
pub async fn api_hear(
    State(state): State<AppState>,
    Json(req): Json<HearRequest>,
) -> impl IntoResponse {
    let text = req.text.trim().to_string();

    if text.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Texte vide"
        })));
    }

    tracing::info!("[SENSORIA] Transcription recue : \"{}\"", text);

    // Injecter dans le pipeline cognitif comme un message de JRM
    let msg = UserMessage {
        text: text.clone(),
        username: "JRM".to_string(),
    };

    match state.user_tx.send(msg).await {
        Ok(_) => {
            (StatusCode::OK, Json(serde_json::json!({
                "status": "ok",
                "received": text,
            })))
        }
        Err(e) => {
            tracing::error!("[SENSORIA] Erreur envoi au pipeline : {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": format!("Pipeline inaccessible : {}", e)
            })))
        }
    }
}

// ─── Demande de parole (Saphire → Sensoria) ──────────────────────────────

#[derive(Deserialize)]
pub struct SpeakRequest {
    /// Texte que Saphire veut prononcer
    pub text: String,
}

/// Envoie du texte a Sensoria pour synthese vocale (TTS).
pub async fn api_speak(
    State(_state): State<AppState>,
    Json(req): Json<SpeakRequest>,
) -> impl IntoResponse {
    let text = req.text.trim().to_string();

    if text.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Texte vide"
        })));
    }

    tracing::info!("[SENSORIA] Envoi TTS vers Sensoria : \"{}\"", text);

    // Appel synchrone via ureq dans un spawn_blocking (pour ne pas bloquer le runtime)
    let text_clone = text.clone();
    let result = tokio::task::spawn_blocking(move || {
        let url = format!("http://{}/api/speak", SENSORIA_HOST);
        ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_string(&serde_json::json!({ "text": text_clone }).to_string())
    })
    .await;

    match result {
        Ok(Ok(resp)) => {
            if resp.status() == 200 {
                (StatusCode::OK, Json(serde_json::json!({
                    "status": "ok",
                    "sent": text,
                })))
            } else {
                (StatusCode::BAD_GATEWAY, Json(serde_json::json!({
                    "error": format!("Sensoria a repondu {}", resp.status()),
                })))
            }
        }
        Ok(Err(e)) => {
            tracing::error!("[SENSORIA] Impossible de joindre Sensoria : {}", e);
            (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "error": format!("Sensoria injoignable : {}", e),
            })))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": format!("Erreur interne : {}", e),
            })))
        }
    }
}

// ─── Statut de la connexion Sensoria ─────────────────────────────────────

/// Retourne le statut de la connexion avec Sensoria.
pub async fn api_sensoria_status(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let sensoria_online = tokio::task::spawn_blocking(|| {
        let url = format!("http://{}/api/status", SENSORIA_HOST);
        matches!(ureq::get(&url).call(), Ok(resp) if resp.status() == 200)
    })
    .await
    .unwrap_or(false);

    Json(serde_json::json!({
        "sensoria_host": SENSORIA_HOST,
        "online": sensoria_online,
    }))
}
