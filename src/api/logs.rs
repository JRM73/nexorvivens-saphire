// =============================================================================
// api/logs.rs — Log, cognitive trace, and LLM history handlers
//
// This module provides HTTP endpoints for:
// - Logs: listing with filtering, detail by ID, bulk export.
// - Cognitive traces: per-cycle retrieval (with optional session filtering),
//   and paginated listing with source type filtering.
// - LLM history: paginated listing of LLM requests and detail by ID.
//
// All endpoints require the LogsDb to be available; they return a JSON error
// if it is not configured.
// =============================================================================

use std::collections::HashMap;
use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/logs -- Lists logs with optional filtering.
///
/// # Query parameters
/// * `level` (optional): filter by log level (e.g. "info", "warn", "error").
/// * `category` (optional): filter by log category.
/// * `limit` (optional, default 100): maximum number of log entries to return.
/// * `offset` (optional, default 0): pagination offset.
///
/// # Returns
/// JSON `{"logs": [...]}` on success, or `{"error": ...}` on failure.
pub async fn api_get_logs(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let level = params.get("level").map(|s| s.as_str());
        let category = params.get("category").map(|s| s.as_str());
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(100);
        let offset: i64 = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);
        match logs_db.get_logs(level, category, limit, offset).await {
            Ok(logs) => axum::Json(serde_json::json!({"logs": logs})),
            Err(e) => { tracing::error!("API logs: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/logs/:id -- Retrieves a single log entry by its ID.
///
/// # Path parameters
/// * `id` - The unique identifier of the log entry.
///
/// # Returns
/// The log entry as JSON, `{"error": "not found"}` if absent, or an internal error.
pub async fn api_get_log_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.get_log_by_id(id).await {
            Ok(Some(log)) => axum::Json(log),
            Ok(None) => axum::Json(serde_json::json!({"error": "not found"})),
            Err(e) => { tracing::error!("API logs: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/logs/export -- Exports up to 10,000 log entries as a JSON array.
///
/// This is a bulk export endpoint intended for backup or offline analysis.
pub async fn api_export_logs(State(state): State<AppState>) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.export_logs(10000).await {
            Ok(logs) => axum::Json(serde_json::json!({"logs": logs})),
            Err(e) => { tracing::error!("API logs: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/trace/:cycle -- Retrieves the cognitive trace for a specific cycle.
///
/// # Path parameters
/// * `cycle` - The cognitive cycle number to retrieve.
///
/// # Query parameters
/// * `session_id` (optional): when provided, targets the exact trace for this
///   cycle within the given session (avoids cycle number collisions across sessions).
///   When absent, returns the most recent trace for this cycle across all sessions.
///
/// # Returns
/// JSON of the complete trace (19 JSONB fields) on success, or `{"error": ...}`.
pub async fn api_get_trace(
    State(state): State<AppState>,
    axum::extract::Path(cycle): axum::extract::Path<i64>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let result = if let Some(sid) = params.get("session_id").and_then(|s| s.parse::<i64>().ok()) {
            logs_db.get_trace_by_cycle_and_session(cycle, sid).await
        } else {
            logs_db.get_trace_by_cycle(cycle).await
        };
        match result {
            Ok(Some(trace)) => axum::Json(trace),
            Ok(None) => axum::Json(serde_json::json!({"error": "not found"})),
            Err(e) => { tracing::error!("API logs: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/traces -- Lists cognitive traces with optional filters.
///
/// # Query parameters (all optional)
/// * `session_id` (recommended): filter traces by session ID.
/// * `source_type`: filter by trace source type:
///   - `"Human"`: traces originating from a user message (contains NLP of the message).
///   - `"Autonomous"`: traces from autonomous thought (contains NLP of the thought).
/// * `limit` (default 50): maximum number of traces to return.
///
/// # Behavior
/// - If `session_id` is provided: uses `traces_by_session()` with `source_type` filter.
/// - If `session_id` is absent: uses `recent_traces()` (no source_type filter).
///
/// # Returns
/// JSON `{"data": [traces...]}` on success, or `{"error": ...}` on failure.
/// Used by the dashboard for the clickable trace listing.
pub async fn api_list_traces(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
        let source_type = params.get("source_type").map(|s| s.as_str());

        let result = if let Some(sid) = params.get("session_id").and_then(|s| s.parse::<i64>().ok()) {
            logs_db.traces_by_session(sid, source_type, limit).await
        } else {
            logs_db.recent_traces(limit).await
        };
        match result {
            Ok(traces) => axum::Json(serde_json::json!({"data": traces})),
            Err(e) => { tracing::error!("API logs: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/llm/history -- Paginated LLM request history.
///
/// # Query parameters
/// * `limit` (optional, default 50): maximum number of entries to return.
/// * `offset` (optional, default 0): pagination offset.
///
/// # Returns
/// JSON `{"history": [...]}` on success, or `{"error": ...}` on failure.
pub async fn api_llm_history(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
        let offset: i64 = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);
        match logs_db.get_llm_history(limit, offset).await {
            Ok(data) => axum::Json(serde_json::json!({"history": data})),
            Err(e) => { tracing::error!("API logs: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/llm/history/:id -- Detail view of a single LLM request.
///
/// # Path parameters
/// * `id` - The unique identifier of the LLM request entry.
///
/// # Returns
/// Full LLM request data as JSON, `{"error": "not found"}` if absent, or an internal error.
pub async fn api_llm_history_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.get_llm_by_id(id).await {
            Ok(Some(data)) => axum::Json(data),
            Ok(None) => axum::Json(serde_json::json!({"error": "not found"})),
            Err(e) => { tracing::error!("API logs: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}
