// =============================================================================
// api/logs.rs — Handlers de logs, traces et historique LLM
//
// Role : Endpoints pour les logs (liste, detail, export), les traces
// cognitives (par cycle, par session) et l'historique des requetes LLM.
// =============================================================================

use std::collections::HashMap;
use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/logs — Liste les logs avec filtrage optionnel.
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

/// GET /api/logs/:id — Recupere un log par ID.
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

/// GET /api/logs/export — Exporte les logs en JSON.
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

/// GET /api/trace/:cycle — Recupere la trace cognitive d'un cycle specifique.
///
/// Parametres :
///   - :cycle (path) : numero du cycle cognitif a recuperer
///   - ?session_id=N (query, optionnel) : filtre par session
///
/// Comportement :
///   - Si session_id est fourni : utilise get_trace_by_cycle_and_session()
///     pour cibler exactement la bonne trace (evite les collisions de cycles
///     entre sessions differentes)
///   - Si session_id est absent : utilise get_trace_by_cycle() qui retourne
///     la trace la plus recente pour ce cycle (toutes sessions confondues)
///
/// Retourne : JSON de la trace complete (19 champs JSONB) ou {"error": ...}
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

/// GET /api/traces — Liste les traces cognitives avec filtres optionnels.
///
/// Parametres (query string, tous optionnels) :
///   - session_id=N : filtrer par session (recommande)
///   - source_type=Human|Autonomous : filtrer par type de source
///     * "Human" : traces issues d'un message utilisateur (contient NLP du message)
///     * "Autonomous" : traces issues de la pensee autonome (contient NLP de la pensee)
///   - limit=50 : nombre max de traces retournees (defaut 50)
///
/// Comportement :
///   - Si session_id est fourni : utilise traces_by_session() avec filtre source_type
///   - Si session_id est absent : utilise recent_traces() (sans filtre source_type)
///
/// Retourne : {"data": [traces...]} ou {"error": ...}
/// Utilise par le dashboard pour le listing cliquable des traces
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

/// GET /api/llm/history — Historique LLM avec pagination.
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

/// GET /api/llm/history/:id — Detail d'une requete LLM.
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
