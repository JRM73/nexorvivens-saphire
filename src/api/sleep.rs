// =============================================================================
// api/sleep.rs — REST endpoints for the sleep system
//
// Role: Exposes 5 endpoints to query and control Saphire's sleep.
// Same pattern as api/nn_learnings.rs (State<AppState>, agent.lock().await).
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/sleep/status — Complete sleep state.
pub async fn api_sleep_status(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.sleep.to_status_json())
}

/// GET /api/sleep/history — Sleep history.
pub async fn api_sleep_history(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.sleep.to_history_json())
}

/// GET /api/sleep/drive — Sleep pressure state.
pub async fn api_sleep_drive(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.sleep.to_drive_json())
}

/// POST /api/sleep/force — Force sleep onset.
/// Initiates sleep without going through the async broadcast (will be done on next tick).
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
    // Initiate sleep synchronously (the broadcast will be done on next tick)
    agent.sleep.initiate();
    agent.dream_orch.current_phase = crate::orchestrators::dreams::SleepPhase::Hypnagogic;
    axum::Json(serde_json::json!({
        "success": true,
        "message": "Endormissement force initie",
        "sleep_pressure": agent.sleep.drive.sleep_pressure,
    }))
}

/// POST /api/sleep/wake — Force waking.
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
