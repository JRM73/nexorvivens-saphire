// =============================================================================
// api/orchestrators.rs — Handlers des orchestrateurs de haut niveau
//
// Role: Endpoints for the orchestrateurs : dreams, desires, learning,
// attention, guerison — statuts et details.
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/dreams/status — Etat de l'orchestrateur de reves.
pub async fn api_dreams_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.dream_orch.to_status_json())
}

/// GET /api/dreams/journal — Journal complete des dreams.
pub async fn api_dreams_journal(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let journal: Vec<serde_json::Value> = agent.dream_orch.dream_journal.iter().map(|entry| {
        serde_json::json!({
            "dream_type": entry.dream.dream_type.as_str(),
            "narrative": entry.dream.narrative,
            "dominant_emotion": entry.dream.dominant_emotion,
            "insight": entry.dream.insight,
            "remembered": entry.remembered,
            "timestamp": entry.dream.started_at.to_rfc3339(),
        })
    }).collect();
    axum::Json(serde_json::json!({"journal": journal, "total": journal.len()}))
}

/// GET /api/desires/status — Etat de l'orchestrateur de desirs.
pub async fn api_desires_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.desire_orch.to_status_json())
}

/// GET /api/desires/active — Desirs active with milestones.
pub async fn api_desires_active(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let active: Vec<serde_json::Value> = agent.desire_orch.active_desires.iter().map(|d| {
        serde_json::json!({
            "id": d.id, "title": d.title, "description": d.description,
            "type": d.desire_type.as_str(), "priority": d.priority, "progress": d.progress,
            "milestones": d.milestones.iter().map(|m| serde_json::json!({
                "description": m.description, "completed": m.completed,
            })).collect::<Vec<_>>(),
            "cycles_invested": d.cycles_invested,
        })
    }).collect();
    let fulfilled: Vec<serde_json::Value> = agent.desire_orch.fulfilled_desires.iter().map(|d| {
        serde_json::json!({"id": d.id, "title": d.title, "type": d.desire_type.as_str()})
    }).collect();
    axum::Json(serde_json::json!({"active": active, "fulfilled": fulfilled}))
}

/// GET /api/desires/needs — Besoins fondamentaux.
pub async fn api_desires_needs(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let needs: Vec<serde_json::Value> = agent.desire_orch.fundamental_needs.iter().map(|n| {
        serde_json::json!({"name": n.name, "description": n.description, "satisfaction": n.satisfaction})
    }).collect();
    axum::Json(serde_json::json!({"needs": needs}))
}

/// GET /api/learning/status — Etat de l'orchestrateur d'learning.
pub async fn api_learning_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.learning_orch.to_status_json())
}

/// GET /api/learning/lessons — Toutes les lessons.
pub async fn api_learning_lessons(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let lessons: Vec<serde_json::Value> = agent.learning_orch.lessons.iter().map(|l| {
        serde_json::json!({
            "id": l.id, "title": l.title, "content": l.content,
            "category": l.category.as_str(), "confidence": l.confidence,
            "times_applied": l.times_applied, "times_contradicted": l.times_contradicted,
            "behavior_change": l.behavior_change.as_ref().map(|bc| serde_json::json!({
                "parameter": bc.parameter, "old_value": bc.old_value,
                "new_value": bc.new_value, "reason": bc.reason,
            })),
        })
    }).collect();
    axum::Json(serde_json::json!({"lessons": lessons, "total": lessons.len()}))
}

/// GET /api/learning/stats — Statistiques d'learning.
pub async fn api_learning_stats(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let total = agent.learning_orch.lessons.len();
    let confirmed = agent.learning_orch.lessons.iter().filter(|l| l.confidence > 0.6).count();
    let contradicted = agent.learning_orch.lessons.iter().filter(|l| l.times_contradicted > 0).count();
    let behavior_changes = agent.learning_orch.lessons.iter().filter(|l| l.behavior_change.is_some()).count();
    axum::Json(serde_json::json!({
        "total": total, "confirmed": confirmed, "contradicted": contradicted,
        "behavior_changes": behavior_changes,
    }))
}

/// GET /api/attention/status — Etat de l'orchestrateur d'attention.
pub async fn api_attention_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.attention_orch.to_status_json())
}

/// GET /api/healing/status — Etat de l'orchestrateur de guerison.
pub async fn api_healing_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.healing_orch.to_status_json())
}

/// GET /api/healing/wounds — Toutes les wounds (actives + gueries).
pub async fn api_healing_wounds(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let active: Vec<serde_json::Value> = agent.healing_orch.active_wounds.iter().map(|w| {
        serde_json::json!({
            "id": w.id, "wound_type": w.wound_type.as_str(), "description": w.description,
            "severity": w.severity, "healing_progress": w.healing_progress,
            "healing_strategy": w.healing_strategy, "status": "active",
        })
    }).collect();
    let healed: Vec<serde_json::Value> = agent.healing_orch.healed_wounds.iter().map(|w| {
        serde_json::json!({
            "id": w.id, "wound_type": w.wound_type.as_str(), "description": w.description,
            "severity": w.severity, "healing_strategy": w.healing_strategy, "status": "healed",
        })
    }).collect();
    axum::Json(serde_json::json!({"active": active, "healed": healed, "resilience": agent.healing_orch.resilience}))
}

/// GET /api/healing/strategies — Strategies de coping with efficacite.
pub async fn api_healing_strategies(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let strategies: Vec<serde_json::Value> = agent.healing_orch.coping_strategies.iter().map(|s| {
        serde_json::json!({
            "name": s.name, "description": s.description,
            "success_rate": s.success_rate, "times_used": s.times_used,
        })
    }).collect();
    axum::Json(serde_json::json!({"strategies": strategies}))
}
