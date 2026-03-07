// =============================================================================
// api/memory.rs — Memory handlers (working, episodic, LTM, founding, stats)
//
// This module provides HTTP endpoints for querying the agent's three-tier
// memory system:
// - Working memory: the agent's current short-term memory contents.
// - Episodic memory: time-stamped experiential records (with pagination).
// - Long-term memory (LTM): consolidated, persistent memories (with pagination).
// - Founding memories: immutable, identity-defining memories.
// - Memory statistics and archive management.
// =============================================================================

use std::collections::HashMap;
use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/memory -- Returns general memory data (recent memories, etc.).
pub async fn api_get_memory(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.memory_data())
}

/// GET /api/memory/working -- Returns the contents of working memory.
///
/// Working memory holds the agent's current short-term context used during
/// active cognitive processing.
pub async fn api_get_working_memory(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.memory_data())
}

/// GET /api/memory/episodic -- Lists episodic memories with pagination.
///
/// # Query parameters
/// * `limit` (optional, default 50): maximum number of entries.
/// * `offset` (optional, default 0): pagination offset.
///
/// # Returns
/// JSON `{"episodic": [...]}` on success, or `{"error": ...}` if the DB is unavailable.
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

/// GET /api/memory/episodic/:id -- Retrieves a single episodic memory by ID.
///
/// # Path parameters
/// * `id` - The unique identifier of the episodic memory.
///
/// # Returns
/// The memory entry as JSON, `{"error": "not found"}` if absent, or an internal error.
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

/// GET /api/memory/ltm -- Lists long-term memories with pagination.
///
/// # Query parameters
/// * `limit` (optional, default 50): maximum number of entries.
/// * `offset` (optional, default 0): pagination offset.
///
/// # Returns
/// JSON `{"ltm": [...]}` on success, or `{"error": ...}` if the DB is unavailable.
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

/// GET /api/memory/ltm/:id -- Retrieves a single long-term memory by ID.
///
/// # Path parameters
/// * `id` - The unique identifier of the LTM entry.
///
/// # Returns
/// The memory entry as JSON, `{"error": "not found"}` if absent, or an internal error.
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

/// GET /api/memory/founding -- Lists all founding memories.
///
/// Founding memories are immutable, identity-defining records that cannot be
/// consolidated or archived. They represent the agent's core experiential anchors.
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

/// GET /api/memory/stats -- Returns memory statistics (counts, sizes, etc.).
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

/// GET /api/memory/archives -- Lists archived memories with pagination.
///
/// # Query parameters
/// * `limit` (optional, default 50): maximum number of entries.
/// * `offset` (optional, default 0): pagination offset.
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

/// GET /api/memory/archives/stats -- Returns archive statistics (counts, sizes, etc.).
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
