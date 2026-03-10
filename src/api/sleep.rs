// =============================================================================
// api/sleep.rs — Endpoints REST pour le systeme de sommeil
//
// Role : Expose 5 endpoints pour consulter et controler le sommeil de Saphire.
// Pattern identique a api/nn_learnings.rs (State<AppState>, agent.lock().await).
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/sleep/status — Etat complet du sommeil.
pub async fn api_sleep_status(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.sleep.to_status_json())
}

/// GET /api/sleep/history — Historique des sommeils.
pub async fn api_sleep_history(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.sleep.to_history_json())
}

/// GET /api/sleep/drive — Etat de la pression de sommeil.
pub async fn api_sleep_drive(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.sleep.to_drive_json())
}

/// POST /api/sleep/force — Forcer l'endormissement.
/// Initie le sommeil sans passer par le broadcast async (sera fait au prochain tick).
pub async fn api_sleep_force(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    if agent.sleep.is_sleeping {
        return axum::Json(serde_json::json!({
            "success": false,
            "reason": "Saphire dort deja",
        }));
    }
    // Initier le sommeil de maniere synchrone (le broadcast sera fait au prochain tick)
    agent.sleep.initiate();
    agent.dream_orch.current_phase = crate::orchestrators::dreams::SleepPhase::Hypnagogic;
    axum::Json(serde_json::json!({
        "success": true,
        "message": "Endormissement force initie",
        "sleep_pressure": agent.sleep.drive.sleep_pressure,
    }))
}

/// POST /api/sleep/wake — Forcer le reveil.
pub async fn api_sleep_wake(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    if !agent.sleep.is_sleeping {
        return axum::Json(serde_json::json!({
            "success": false,
            "reason": "Saphire est deja eveillee",
        }));
    }
    agent.sleep.interrupt("Reveil force via API".into());
    agent.dream_orch.wake_up();
    axum::Json(serde_json::json!({
        "success": true,
        "message": "Reveil force",
    }))
}
