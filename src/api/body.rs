// =============================================================================
// api/body.rs — Handlers corps virtuel et coeur
//
// Role: Endpoints for the virtual body (statut, coeur, historique BPM/HRV,
// historique corporel, milestones de battements cardiaques).
// =============================================================================

use std::collections::HashMap;
use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/body/status — Etat complete du corps (temps reel).
pub async fn api_body_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let status = agent.body_status();
    axum::Json(serde_json::json!(status))
}

/// GET /api/body/heart — Details of the heart (BPM, beat_count, HRV).
pub async fn api_body_heart(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let heart = agent.heart_status();
    axum::Json(serde_json::json!(heart))
}

/// GET /api/body/heart/history — Historique BPM + HRV.
pub async fn api_body_heart_history(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_heart_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API body: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/body/history — Historique corporel complet.
pub async fn api_body_history(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_body_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API body: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/body/vitals — Details physiologiques (parameters vitaux, metabolisme).
pub async fn api_body_vitals(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let status = agent.body_status();
    axum::Json(serde_json::json!(status.vitals))
}

/// GET /api/body/milestones — Milestones de battements cardiaques.
pub async fn api_body_milestones(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let beat_count = agent.heart_status().beat_count;
    let mut milestones = Vec::new();
    let thresholds: [u64; 5] = [1_000, 10_000, 100_000, 1_000_000, 10_000_000];
    for &threshold in &thresholds {
        milestones.push(serde_json::json!({
            "threshold": threshold,
            "reached": beat_count >= threshold,
            "current": beat_count,
        }));
    }
    axum::Json(serde_json::json!({"milestones": milestones, "total_beats": beat_count}))
}
