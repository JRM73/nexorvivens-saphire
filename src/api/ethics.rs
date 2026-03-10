// =============================================================================
// api/ethics.rs — Handlers ethique personnelle
//
// Role : Endpoints pour les 3 couches ethiques (vue d'ensemble),
// la liste des principes personnels, le detail par ID,
// et les conditions de formulation (readiness).
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/ethics/layers — Vue d'ensemble des 3 couches ethiques.
pub async fn api_ethics_layers(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.ethics.to_broadcast_json())
}

/// GET /api/ethics/personal — Liste des principes ethiques personnels.
pub async fn api_ethics_personal(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let principles: Vec<serde_json::Value> = agent.ethics.personal_principles().iter().map(|p| {
        serde_json::json!({
            "id": p.id,
            "title": p.title,
            "content": p.content,
            "reasoning": p.reasoning,
            "born_from": p.born_from,
            "born_at_cycle": p.born_at_cycle,
            "emotion_at_creation": p.emotion_at_creation,
            "times_invoked": p.times_invoked,
            "times_questioned": p.times_questioned,
            "is_active": p.is_active,
            "created_at": p.created_at.to_rfc3339(),
        })
    }).collect();
    axum::Json(serde_json::json!({
        "active_count": agent.ethics.active_personal_count(),
        "total_count": agent.ethics.total_personal_count(),
        "principles": principles,
    }))
}

/// GET /api/ethics/personal/:id — Detail d'un principe ethique personnel.
pub async fn api_ethics_personal_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    if let Some(p) = agent.ethics.personal_principles().iter().find(|p| p.id == id) {
        axum::Json(serde_json::json!({
            "id": p.id,
            "title": p.title,
            "content": p.content,
            "reasoning": p.reasoning,
            "born_from": p.born_from,
            "born_at_cycle": p.born_at_cycle,
            "emotion_at_creation": p.emotion_at_creation,
            "times_invoked": p.times_invoked,
            "times_questioned": p.times_questioned,
            "is_active": p.is_active,
            "supersedes": p.supersedes,
            "created_at": p.created_at.to_rfc3339(),
            "modified_at": p.modified_at.map(|t| t.to_rfc3339()),
            "last_invoked_at": p.last_invoked_at.map(|t| t.to_rfc3339()),
        }))
    } else {
        axum::Json(serde_json::json!({"error": "not found"}))
    }
}

/// GET /api/ethics/readiness — Conditions de formulation ethique.
/// Expose l'etat de chaque condition requise pour formuler un nouveau principe.
pub async fn api_ethics_readiness(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let config = &agent.config().ethics;
    let enabled = config.enabled && config.personal_ethics_enabled;

    let c_min_cycles = agent.identity.total_cycles >= 50;
    let c_moral = agent.moral_reflection_count >= config.min_moral_reflections_before as u64;
    let c_consciousness = agent.last_consciousness >= config.min_consciousness_for_formulation;
    let c_cortisol = agent.chemistry.cortisol < 0.5;
    let c_serotonin = agent.chemistry.serotonin >= 0.4;
    let c_cooldown = agent.cycles_since_last_formulation >= config.formulation_cooldown_cycles;
    let c_capacity = agent.ethics.active_personal_count() < config.max_personal_principles;

    let met_count = [c_min_cycles, c_moral, c_consciousness, c_cortisol, c_serotonin, c_cooldown, c_capacity]
        .iter().filter(|&&v| v).count();

    axum::Json(serde_json::json!({
        "enabled": enabled,
        "conditions": {
            "min_cycles": { "required": 50, "current": agent.identity.total_cycles, "met": c_min_cycles },
            "moral_reflections": { "required": config.min_moral_reflections_before, "current": agent.moral_reflection_count, "met": c_moral },
            "consciousness": { "required": config.min_consciousness_for_formulation, "current": agent.last_consciousness, "met": c_consciousness },
            "cortisol": { "max": 0.5, "current": agent.chemistry.cortisol, "met": c_cortisol },
            "serotonin": { "min": 0.4, "current": agent.chemistry.serotonin, "met": c_serotonin },
            "cooldown": { "required": config.formulation_cooldown_cycles, "elapsed": agent.cycles_since_last_formulation, "met": c_cooldown },
            "capacity": { "max": config.max_personal_principles, "current": agent.ethics.active_personal_count(), "met": c_capacity }
        },
        "all_met": enabled && met_count == 7,
        "met_count": met_count,
        "total_conditions": 7
    }))
}
