// =============================================================================
// api/personality.rs — Portrait de personnalite temporel (3 niveaux)
//
// Endpoints pour visualiser l'evolution de la personnalite au fil du temps :
//   Niveau 1 : Snapshots periodiques (toutes les 50 cycles)
//   Niveau 2 : Archives par domaine (emotions, conscience, psychologie, relations)
//   Niveau 3 : Journal introspectif (toutes les 200 cycles, genere par LLM)
// =============================================================================

use std::collections::HashMap;
use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/personality/timeline?limit=200 — Snapshots complets de personnalite.
pub async fn api_personality_timeline(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
    if let Some(ref db) = agent.db {
        match db.load_personality_snapshots(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data, "count": data.len()})),
            Err(e) => { tracing::error!("API personality timeline: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}

/// GET /api/personality/emotions?limit=200 — Trajectoire emotionnelle.
pub async fn api_personality_emotions(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
    if let Some(ref db) = agent.db {
        match db.load_emotional_trajectory(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data, "count": data.len()})),
            Err(e) => { tracing::error!("API personality emotions: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}

/// GET /api/personality/consciousness?limit=200 — Evolution conscience/phi.
pub async fn api_personality_consciousness(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
    if let Some(ref db) = agent.db {
        match db.load_consciousness_history(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data, "count": data.len()})),
            Err(e) => { tracing::error!("API personality consciousness: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}

/// GET /api/personality/psychology?limit=200 — Checkpoints psychologiques.
pub async fn api_personality_psychology(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
    if let Some(ref db) = agent.db {
        match db.load_psychology_checkpoints(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data, "count": data.len()})),
            Err(e) => { tracing::error!("API personality psychology: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}

/// GET /api/personality/relationships?limit=200 — Evolution des liens affectifs.
pub async fn api_personality_relationships(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
    if let Some(ref db) = agent.db {
        match db.load_relationship_timeline(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data, "count": data.len()})),
            Err(e) => { tracing::error!("API personality relationships: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}

/// GET /api/personality/journal?limit=50 — Journal introspectif.
pub async fn api_personality_journal(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
    if let Some(ref db) = agent.db {
        match db.load_journal_entries(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data, "count": data.len()})),
            Err(e) => { tracing::error!("API personality journal: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}
