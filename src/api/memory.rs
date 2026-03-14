// =============================================================================
// api/memory.rs — Handlers memoire (working, episodique, LTM, fondateurs, stats)
//
// Role: Endpoints for consulter les 3 niveaux de memoire de l'agent :
// memoire de travail, memories episodic, memoire a long terme,
// memories fondateurs et statistiques memorielles.
// =============================================================================

use std::collections::HashMap;
use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/memory — Returns the data de la memoire (memories recents, etc.).
pub async fn api_get_memory(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.memory_data())
}

/// GET /api/memory/working — Returns le contenu de la working memory.
pub async fn api_get_working_memory(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.memory_data())
}

/// GET /api/memory/episodic — Liste the memories episodic.
pub async fn api_list_episodic(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    if let Some(ref db) = agent.db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
        let offset: i64 = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);
        match db.list_episodic(limit, offset).await {
            Ok(items) => axum::Json(serde_json::json!({"episodic": items})),
            Err(e) => { tracing::error!("API memory: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}

/// GET /api/memory/episodic/:id — Recupere un memory episodic.
pub async fn api_get_episodic_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    if let Some(ref db) = agent.db {
        match db.get_episodic_by_id(id).await {
            Ok(Some(item)) => axum::Json(item),
            Ok(None) => axum::Json(serde_json::json!({"error": "not found"})),
            Err(e) => { tracing::error!("API memory: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}

/// GET /api/memory/ltm — Liste the memories a long terme.
pub async fn api_list_ltm(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    if let Some(ref db) = agent.db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
        let offset: i64 = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);
        match db.list_memories(limit, offset).await {
            Ok(items) => axum::Json(serde_json::json!({"ltm": items})),
            Err(e) => { tracing::error!("API memory: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}

/// GET /api/memory/ltm/:id — Recupere un memory LTM par ID.
pub async fn api_get_ltm_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    if let Some(ref db) = agent.db {
        match db.get_memory_by_id(id).await {
            Ok(Some(item)) => axum::Json(item),
            Ok(None) => axum::Json(serde_json::json!({"error": "not found"})),
            Err(e) => { tracing::error!("API memory: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}

/// GET /api/memory/founding — Liste the memories fondateurs.
pub async fn api_list_founding(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    if let Some(ref db) = agent.db {
        match db.list_founding_memories().await {
            Ok(items) => axum::Json(serde_json::json!({"founding": items})),
            Err(e) => { tracing::error!("API memory: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}

/// GET /api/memory/stats — Statistiques de memoire.
pub async fn api_memory_stats(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    if let Some(ref db) = agent.db {
        match db.memory_stats().await {
            Ok(stats) => axum::Json(stats),
            Err(e) => { tracing::error!("API memory: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}

/// GET /api/memory/archives — Liste les archives memoire.
pub async fn api_list_archives(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    if let Some(ref db) = agent.db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
        let offset: i64 = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);
        match db.list_archives(limit, offset).await {
            Ok(items) => axum::Json(serde_json::json!({"archives": items})),
            Err(e) => { tracing::error!("API memory: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}

/// GET /api/memory/archives/stats — Statistiques des archives.
pub async fn api_archive_stats(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    if let Some(ref db) = agent.db {
        match db.archive_stats().await {
            Ok(stats) => axum::Json(stats),
            Err(e) => { tracing::error!("API memory: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}
