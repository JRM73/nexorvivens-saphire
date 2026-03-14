// =============================================================================
// api/logs.rs — Log, trace and LLM history handlers
//
// Role: Endpoints for logs (list, detail, export), cognitive traces
// (by cycle, by session) and LLM request history.
// =============================================================================

use std::collections::HashMap;
use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/logs — List logs with optional filtering.
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

/// GET /api/logs/:id — Retrieve a log by ID.
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

/// GET /api/logs/export — Export logs as JSON.
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

/// GET /api/trace/:cycle — Retrieve the cognitive trace of a specific cycle.
///
/// Parameters:
///  - :cycle (path): cognitive cycle number to retrieve
///  - ?session_id=N (query, optional): filter by session
///
/// Behavior:
///  - If session_id is provided: uses get_trace_by_cycle_and_session()
///    to target the exact trace (avoids cycle collisions between
///    different sessions)
///  - If session_id is absent: uses get_trace_by_cycle() which returns
///    the most recent trace for that cycle (across all sessions)
///
/// Returns: JSON of the complete trace (19 JSONB fields) or {"error": ...}
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

/// GET /api/traces — List cognitive traces with optional filters.
///
/// Parameters (query string, all optional):
///  - session_id=N: filter by session (recommended)
///  - source_type=Human|Autonomous: filter by source type
///  * "Human": traces from a user message (contains NLP of the message)
///  * "Autonomous": traces from autonomous thought (contains NLP of the thought)
///  - limit=50: max number of traces returned (default 50)
///
/// Behavior:
///  - If session_id is provided: uses traces_by_session() with source_type filter
///  - If session_id is absent: uses recent_traces() (without source_type filter)
///
/// Returns: {"data": [traces...]} or {"error": ...}
/// Used by the dashboard for the clickable trace listing
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

/// GET /api/trace/last — Complete cognitive trace of the last cycle.
///
/// Returns the most recent trace (all data from the 24 pipeline stages).
/// Optional parameters: ?source_type=Autonomous|Human
pub async fn api_get_trace_last(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let source_type = params.get("source_type").map(|s| s.as_str());
        let result = if let Some(st) = source_type {
            let agent = state.agent.lock().await;
            let sid = agent.session_id;
            drop(agent);
            logs_db.traces_by_session(sid, Some(st), 1).await
        } else {
            logs_db.recent_traces(1).await
        };
        match result {
            Ok(traces) if !traces.is_empty() => axum::Json(traces[0].clone()),
            Ok(_) => axum::Json(serde_json::json!({"error": "no traces found"})),
            Err(e) => { tracing::error!("API trace/last: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/llm/history — LLM history with pagination.
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

/// GET /api/llm/history/:id — Detail of an LLM request.
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
