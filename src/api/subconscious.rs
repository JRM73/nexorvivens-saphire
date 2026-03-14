// =============================================================================
// api/subconscious.rs — Endpoints REST pour le subconscient et connexions
//
// Role : Expose les endpoints pour consulter le subconscient, les associations,
// les insights, et les connexions neuronales.
// =============================================================================

use std::collections::HashMap;
use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/subconscious/status — Etat complet du subconscient.
pub async fn api_subconscious_status(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.subconscious.to_status_json())
}

/// GET /api/subconscious/associations — Associations en gestation.
pub async fn api_subconscious_associations(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let associations: Vec<serde_json::Value> = agent.subconscious.pending_associations.iter()
        .map(|a| serde_json::json!({
            "memory_a_id": a.memory_a_id,
            "memory_a_summary": a.memory_a_summary,
            "memory_b_id": a.memory_b_id,
            "memory_b_summary": a.memory_b_summary,
            "strength": a.strength,
            "link_type": a.link_type,
            "maturation_remaining": a.maturation_remaining,
        }))
        .collect();
    axum::Json(serde_json::json!({
        "count": associations.len(),
        "total_created": agent.subconscious.total_associations_created,
        "associations": associations,
    }))
}

/// GET /api/subconscious/insights — Insights prets ou recents.
pub async fn api_subconscious_insights(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let insights: Vec<serde_json::Value> = agent.subconscious.ready_insights.iter()
        .map(|i| serde_json::json!({
            "content": i.content,
            "source_type": i.source_type,
            "strength": i.strength,
            "emotional_charge": i.emotional_charge,
        }))
        .collect();
    axum::Json(serde_json::json!({
        "ready_count": insights.len(),
        "total_surfaced": agent.subconscious.total_insights_surfaced,
        "insights": insights,
    }))
}

/// GET /api/connections/list — Connexions neuronales (pagine depuis la DB).
pub async fn api_connections_list(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);
    let agent = state.agent.lock().await;
    if let Some(ref db) = agent.db {
        match db.get_neural_connections(limit, offset).await {
            Ok(data) => axum::Json(serde_json::json!({
                "connections": data,
                "limit": limit,
                "offset": offset,
            })),
            Err(e) => { tracing::error!("API subconscious: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}

/// GET /api/connections/stats — Stats des connexions neuronales.
pub async fn api_connections_stats(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    if let Some(ref db) = agent.db {
        match db.get_neural_connections_stats().await {
            Ok(data) => axum::Json(data),
            Err(e) => { tracing::error!("API subconscious: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}

/// GET /api/metrics/sleep — Pression, phases, qualite sur le temps.
pub async fn api_metrics_sleep(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_sleep_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API subconscious: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/subconscious — Activation, associations, insights sur le temps.
pub async fn api_metrics_subconscious(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_subconscious_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API subconscious: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}
