// =============================================================================
// api/sensoria.rs — Bridge between Saphire and Sensoria (sensory service)
//
// Sensoria runs on a separate machine (i7, 192.168.1.129) and provides
// ears (STT), mouth (TTS) and eyes (vision) for Saphire.
//
// Endpoints:
//  POST /api/hear — Sensoria sends a transcription (what Saphire hears)
//  POST /api/speak — Saphire asks Sensoria to speak
//  GET /api/sensoria — Sensoria connection status
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

// ─── Incoming transcription (Sensoria → Saphire) ─────────────────────────

#[derive(Deserialize)]
pub struct HearRequest {
    /// Text transcribed by Whisper via Sensoria
    pub text: String,
    /// Transcription source
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "sensoria".into()
}

/// Receives a voice transcription from Sensoria.
/// The text is injected into Saphire's cognitive pipeline
/// as a user message (via the user_tx channel).
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

    // Inject into the cognitive pipeline as a message from JRM
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

// ─── Speech request (Saphire → Sensoria) ──────────────────────────────

#[derive(Deserialize)]
pub struct SpeakRequest {
    /// Text that Saphire wants to speak
    pub text: String,
    /// Dominant emotion (optional — for Qwen3-TTS voice modulation)
    pub emotion: Option<String>,
}

/// Sends text to Sensoria for speech synthesis (TTS).
/// If an emotion is provided, it is forwarded for voice modulation.
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

    let emotion = req.emotion.unwrap_or_else(|| "Neutre".into());

    tracing::info!("[SENSORIA] Envoi TTS vers Sensoria : \"{}\" [{}]", text, emotion);

    // Synchronous call via ureq in a spawn_blocking (to not block the runtime)
    let text_clone = text.clone();
    let emotion_clone = emotion.clone();
    let result = tokio::task::spawn_blocking(move || {
        let url = format!("http://{}/api/speak", SENSORIA_HOST);
        ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_string(&serde_json::json!({
                "text": text_clone,
                "emotion": emotion_clone,
            }).to_string())
    })
    .await;

    match result {
        Ok(Ok(resp)) => {
            if resp.status() == 200 {
                (StatusCode::OK, Json(serde_json::json!({
                    "status": "ok",
                    "sent": text,
                    "emotion": emotion,
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

// ─── Sensoria connection status ─────────────────────────────────────
/// Returns the connection status with Sensoria.
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
