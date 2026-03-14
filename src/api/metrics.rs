// =============================================================================
// api/metrics.rs — Handlers de metriques (tous les endpoints /api/metrics/*)
//
// Role: Endpoints for toutes the metrics temporelles : chemistry, emotions,
// decisions, satisfaction, LLM, OCEAN, types de pensees, coeur, corps,
// vital, intuition, premonition, ethique, sens, acuite, emergents,
// connaissances, attention, desires, learning, guerison, dreams.
// =============================================================================

use std::collections::HashMap;
use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/metrics/chemistry — Metriques neurochimiques.
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

/// GET /api/metrics/emotions — Metriques emotionnelles.
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

/// GET /api/metrics/decisions — Metriques de decisions.
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

/// GET /api/metrics/satisfaction — Metriques de satisfaction.
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

/// GET /api/metrics/llm — Metriques LLM (temps de response).
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

/// GET /api/metrics/ocean_history — Historique OCEAN.
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

/// GET /api/metrics/thought_types — Distribution des types de pensees.
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

/// GET /api/metrics/heart — Metriques cardiaques.
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

/// GET /api/metrics/body — Metriques corporelles.
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

/// GET /api/metrics/vital — Metriques d'instinct vital.
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

/// GET /api/metrics/intuition — Metriques d'intuition.
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

/// GET /api/metrics/premonition — Metriques de premonition.
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

/// GET /api/metrics/ethics — Metriques d'ethique personnelle.
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

/// GET /api/metrics/senses — Metriques sensorielles.
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

/// GET /api/metrics/senses_acuity — Metriques d'acuite sensorielle.
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

/// GET /api/metrics/emergent — Metriques de sens emergents.
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

/// GET /api/metrics/knowledge — Distribution des sources de connaissance.
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

/// GET /api/metrics/attention — Metriques d'attention sur une periode.
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

/// GET /api/metrics/desires — Metriques de desires sur une periode.
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

/// GET /api/metrics/learning — Metriques d'learning sur une periode.
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

/// GET /api/metrics/healing — Metriques de guerison sur une periode.
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

/// GET /api/metrics/dreams — Metriques de dreams sur une periode.
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

/// GET /api/metrics/psyche — Metriques psyche (Freud) sur une periode.
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

/// GET /api/metrics/maslow — Metriques Maslow sur une periode.
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

/// GET /api/metrics/eq — Metriques EQ (Goleman) sur une periode.
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

/// GET /api/metrics/flow — Metriques Flow sur une periode.
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

/// GET /api/metrics/shadow — Metriques Ombre (Jung) sur une periode.
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

/// GET /api/metrics/nn_learnings — Metriques apprentissages vectoriels.
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

/// GET /api/metrics/chemical_health — Indicateurs de sante chemical.
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

/// GET /api/metrics/receptors — Snapshot des sensibilites of the receptors.
pub async fn api_metrics_receptors(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let receptors = &agent.hormonal_system.receptors;
    axum::Json(serde_json::json!({
        "data": {
            "cycle": agent.cycle_count,
            "sensitivity": receptors.to_snapshot_json(),
            "factors": {
                "dopamine": receptors.factor_for(crate::neurochemistry::Molecule::Dopamine),
                "cortisol": receptors.factor_for(crate::neurochemistry::Molecule::Cortisol),
                "serotonin": receptors.factor_for(crate::neurochemistry::Molecule::Serotonin),
                "adrenaline": receptors.factor_for(crate::neurochemistry::Molecule::Adrenaline),
                "oxytocin": receptors.factor_for(crate::neurochemistry::Molecule::Oxytocin),
                "endorphin": receptors.factor_for(crate::neurochemistry::Molecule::Endorphin),
                "noradrenaline": receptors.factor_for(crate::neurochemistry::Molecule::Noradrenaline),
                "gaba": receptors.factor_for(crate::neurochemistry::Molecule::Gaba),
                "glutamate": receptors.factor_for(crate::neurochemistry::Molecule::Glutamate),
            },
            "desensitized": receptors.describe_desensitized(),
        },
    }))
}

/// GET /api/metrics/spine — Metriques de la spinal cord.
pub async fn api_metrics_spine(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_spine_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/curiosity — Metriques de curiosite.
pub async fn api_metrics_curiosity(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_curiosity_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API metrics: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// GET /api/metrics/bdnf — Snapshot BDNF et neuroplasticite.
pub async fn api_metrics_bdnf(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let gm = &agent.grey_matter;
    axum::Json(serde_json::json!({
        "data": {
            "cycle": agent.cycle_count,
            "bdnf_level": gm.bdnf_level,
            "neuroplasticity": gm.neuroplasticity,
            "neurogenesis_rate": gm.neurogenesis_rate,
            "synaptic_density": gm.synaptic_density,
            "grey_matter_volume": gm.grey_matter_volume,
            "myelination": gm.myelination,
        },
    }))
}
