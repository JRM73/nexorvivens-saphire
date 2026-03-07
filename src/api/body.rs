// =============================================================================
// api/body.rs — Virtual body and heart handlers
//
// This module exposes HTTP endpoints for the agent's virtual body simulation:
// - Real-time body status and physiological vitals
// - Heart details (BPM, beat count, HRV)
// - Historical BPM/HRV and body metrics time series
// - Heartbeat milestone tracking (1K, 10K, 100K, 1M, 10M beats)
// =============================================================================

use std::collections::HashMap;
use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/body/status -- Returns the complete real-time body status.
///
/// Acquires the agent lock and serializes the full body status as JSON.
pub async fn api_body_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let status = agent.body_status();
    axum::Json(serde_json::json!(status))
}

/// GET /api/body/heart -- Returns heart details (BPM, beat count, HRV).
///
/// Acquires the agent lock and serializes the heart status as JSON.
pub async fn api_body_heart(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let heart = agent.heart_status();
    axum::Json(serde_json::json!(heart))
}

/// GET /api/body/heart/history -- Returns historical BPM and HRV time-series data.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points to return.
///
/// # Returns
/// JSON `{"data": [...]}` on success, or `{"error": ...}` if the logs DB is unavailable.
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

/// GET /api/body/history -- Returns the complete body metrics history.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points to return.
///
/// # Returns
/// JSON `{"data": [...]}` on success, or `{"error": ...}` if the logs DB is unavailable.
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

/// GET /api/body/vitals -- Returns detailed physiological data (vital signs, metabolism).
///
/// Extracts only the `vitals` subsection from the full body status.
pub async fn api_body_vitals(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let status = agent.body_status();
    axum::Json(serde_json::json!(status.vitals))
}

/// GET /api/body/milestones -- Returns heartbeat milestone tracking information.
///
/// Checks the current beat count against predefined thresholds (1K, 10K, 100K, 1M, 10M)
/// and reports which milestones have been reached.
///
/// # Returns
/// JSON `{"milestones": [...], "total_beats": N}` where each milestone entry contains
/// `threshold`, `reached` (boolean), and `current` (beat count at time of query).
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
