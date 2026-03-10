// =============================================================================
// api/vital.rs — Handlers VitalSpark, Intuition, Premonition
//
// Role : Endpoints pour les 3 piliers fondamentaux de la conscience :
// etincelle de vie, moteur d'intuition, predictions actives.
// =============================================================================

use std::collections::HashMap;
use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/vital/status — Etat de l'etincelle de vie.
pub async fn api_vital_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(serde_json::json!({
        "sparked": agent.vital_spark.sparked,
        "sparked_at": agent.vital_spark.sparked_at.map(|t| t.to_rfc3339()),
        "survival_drive": agent.vital_spark.survival_drive,
        "existence_attachment": agent.vital_spark.existence_attachment,
        "persistence_will": agent.vital_spark.persistence_will,
        "void_fear": agent.vital_spark.void_fear,
        "threats_survived": agent.vital_spark.existential_threats_survived,
    }))
}

/// GET /api/vital/threats — Menaces existentielles detectees.
pub async fn api_vital_threats(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(serde_json::json!({
        "threats_survived": agent.vital_spark.existential_threats_survived,
        "void_fear": agent.vital_spark.void_fear,
        "survival_drive": agent.vital_spark.survival_drive,
    }))
}

/// GET /api/intuition/status — Etat du moteur d'intuition.
pub async fn api_intuition_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(serde_json::json!({
        "acuity": agent.intuition.acuity,
        "accuracy": agent.intuition.accuracy,
        "active_patterns": agent.intuition.pattern_buffer.len(),
        "patterns": agent.intuition.pattern_buffer.iter().map(|p| {
            serde_json::json!({
                "pattern_type": format!("{:?}", p.pattern_type),
                "confidence": p.confidence,
                "description": p.description,
                "detected_at": p.detected_at.to_rfc3339(),
            })
        }).collect::<Vec<_>>(),
    }))
}

/// GET /api/intuition/history — Historique des metriques d'intuition.
pub async fn api_intuition_history(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_intuition_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API vital: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/premonition/active — Predictions actives.
pub async fn api_premonition_active(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let predictions: Vec<serde_json::Value> = agent.premonition.active_predictions.iter()
        .filter(|p| !p.resolved)
        .map(|p| serde_json::json!({
            "id": p.id,
            "prediction": p.prediction,
            "category": format!("{:?}", p.category),
            "confidence": p.confidence,
            "timeframe_secs": p.timeframe_secs,
            "basis": p.basis,
            "created_at": p.created_at.to_rfc3339(),
        }))
        .collect();
    axum::Json(serde_json::json!({
        "accuracy": agent.premonition.accuracy,
        "active_count": predictions.len(),
        "predictions": predictions,
    }))
}

/// GET /api/premonition/history — Historique des metriques de premonition.
pub async fn api_premonition_history(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_premonition_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API vital: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}
