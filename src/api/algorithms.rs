// =============================================================================
// api/algorithms.rs — Handlers algorithmes (statut, catalogue, historique)
//
// Role: Endpoints for the orchestrateur d'algorithmes : statut current,
// catalogue complete et history of executions recentes.
// =============================================================================

use std::collections::HashMap;
use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/algorithms/status — Etat de l'orchestrateur d'algorithmes.
pub async fn api_algorithms_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.orchestrator.to_status_json())
}

/// GET /api/algorithms/catalog — Catalogue complete of the algorithms.
pub async fn api_algorithms_catalog(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.orchestrator.catalog_json())
}

/// GET /api/algorithms/history — Historique des executions recentes.
pub async fn api_algorithms_history(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let limit: usize = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20);
    let history: Vec<serde_json::Value> = agent.orchestrator.usage_history.iter()
        .rev()
        .take(limit)
        .map(|u| serde_json::json!({
            "algorithm_id": u.algorithm_id,
            "situation": u.situation,
            "output_summary": u.output_summary,
            "satisfaction": u.satisfaction,
            "used_at": u.used_at.to_rfc3339(),
        }))
        .collect();
    axum::Json(serde_json::json!({"history": history}))
}
