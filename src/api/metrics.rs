// =============================================================================
// api/metrics.rs — Metrics handlers (all /api/metrics/* endpoints)
//
// This module exposes time-series metric endpoints for every monitored
// subsystem of the agent. Each handler queries the LogsDb for historical
// data points with a configurable `limit` parameter.
//
// Covered metric domains: neurochemistry, emotions, decisions, satisfaction,
// LLM response times, OCEAN personality history, thought type distribution,
// heart, body, vital instinct, intuition, premonition, ethics, senses,
// sensory acuity, emergent senses, knowledge sources, attention, desires,
// learning, healing, dreams, psyche (Freud), Maslow hierarchy, emotional
// quotient (Goleman), flow state, shadow (Jung), neural network learnings,
// and chemical health indicators.
// =============================================================================

use std::collections::HashMap;
use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/metrics/chemistry -- Neurochemistry time-series metrics.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_chemistry(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_chemistry_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/emotions -- Emotional state time-series metrics.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_emotions(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_emotion_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/decisions -- Decision-making time-series metrics.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_decisions(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_decision_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/satisfaction -- Satisfaction time-series metrics.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_satisfaction(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_satisfaction_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/llm -- LLM performance time-series metrics (response times, token counts).
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_llm(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_llm_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/ocean_history -- OCEAN (Big Five) personality trait history.
///
/// Queries the main agent DB (not LogsDb) for historical OCEAN personality scores.
pub async fn api_metrics_ocean_history(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    if let Some(ref db) = agent.db {
        match db.get_ocean_history().await {
            Ok(data) => axum::Json(data),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}

/// GET /api/metrics/thought_types -- Distribution of thought types over recent cycles.
///
/// Returns the last 20 data points showing the breakdown of thought categories.
pub async fn api_metrics_thought_types(State(state): State<AppState>) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.get_thought_type_distribution(20).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/heart -- Heart time-series metrics (BPM, HRV, beat count).
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_heart(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_heart_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/body -- Body simulation time-series metrics.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_body(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_body_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/vital -- Vital instinct time-series metrics (survival drive, fear, etc.).
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_vital(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_vital_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/intuition -- Intuition engine time-series metrics (acuity, accuracy).
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_intuition(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_intuition_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/premonition -- Premonition (prediction) time-series metrics.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_premonition(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_premonition_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/ethics -- Personal ethics time-series metrics.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_ethics(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_ethics_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/senses -- Sensory system time-series metrics.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_senses(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_senses_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/senses_acuity -- Sensory acuity time-series metrics.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_senses_acuity(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_senses_acuity_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/emergent -- Emergent senses time-series metrics.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_emergent(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_emergent_senses_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/knowledge -- Knowledge source distribution metrics.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_knowledge(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_knowledge_distribution(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/attention -- Attention time-series metrics over a period.
///
/// # Query parameters
/// * `limit` (optional, default 100): maximum number of data points.
pub async fn api_metrics_attention(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(100i64);
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.get_attention_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"data": []}))
    }
}

/// GET /api/metrics/desires -- Desire tracking time-series metrics over a period.
///
/// # Query parameters
/// * `limit` (optional, default 100): maximum number of data points.
pub async fn api_metrics_desires(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(100i64);
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.get_desires_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"data": []}))
    }
}

/// GET /api/metrics/learning -- Learning progress time-series metrics over a period.
///
/// # Query parameters
/// * `limit` (optional, default 100): maximum number of data points.
pub async fn api_metrics_learning(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(100i64);
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.get_learning_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"data": []}))
    }
}

/// GET /api/metrics/healing -- Healing/recovery time-series metrics over a period.
///
/// # Query parameters
/// * `limit` (optional, default 100): maximum number of data points.
pub async fn api_metrics_healing(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(100i64);
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.get_healing_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"data": []}))
    }
}

/// GET /api/metrics/dreams -- Dream activity time-series metrics over a period.
///
/// # Query parameters
/// * `limit` (optional, default 100): maximum number of data points.
pub async fn api_metrics_dreams(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(100i64);
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.get_dreams_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"data": []}))
    }
}

/// GET /api/metrics/psyche -- Psyche (Freudian model: id/ego/superego) time-series metrics.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_psyche(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(200i64);
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.get_psyche_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"data": []}))
    }
}

/// GET /api/metrics/maslow -- Maslow hierarchy of needs time-series metrics.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_maslow(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(200i64);
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.get_maslow_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"data": []}))
    }
}

/// GET /api/metrics/eq -- Emotional Quotient (Goleman model) time-series metrics.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_eq(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(200i64);
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.get_eq_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"data": []}))
    }
}

/// GET /api/metrics/flow -- Flow state time-series metrics.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_flow(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(200i64);
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.get_flow_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"data": []}))
    }
}

/// GET /api/metrics/shadow -- Shadow (Jungian model) time-series metrics.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_shadow(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(200i64);
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.get_shadow_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"data": []}))
    }
}

/// GET /api/metrics/nn_learnings -- Neural network / vector learning time-series metrics.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_nn_learnings(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(200i64);
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.get_nn_learnings_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"data": []}))
    }
}

/// GET /api/metrics/chemical_health -- Chemical health indicators over time.
///
/// Returns aggregated health metrics derived from neurotransmitter balance,
/// stability, and drift from baselines.
///
/// # Query parameters
/// * `limit` (optional, default 200): maximum number of data points.
pub async fn api_metrics_chemical_health(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(200i64);
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.get_chemical_health(limit).await {
            Ok(data) => axum::Json(data),
            Err(e) => { tracing::error!("API chemical_health: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}
