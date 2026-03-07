// =============================================================================
// api/ethics.rs — Personal ethics handlers
//
// This module exposes HTTP endpoints for the agent's three-layer ethical system:
// 1. Overview of all three ethical layers (hardcoded, configured, personal).
// 2. Listing and detail views for personal ethical principles.
// 3. Readiness check: whether all conditions are met for the agent to
//    formulate a new personal ethical principle.
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/ethics/layers -- Overview of all three ethical layers.
///
/// Returns a JSON representation of the hardcoded, configured, and personal
/// ethical layers including their principles and activation status.
pub async fn api_ethics_layers(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.ethics.to_broadcast_json())
}

/// GET /api/ethics/personal -- Lists all personal ethical principles.
///
/// Returns a JSON object containing the active count, total count, and an array
/// of all personal principles with their metadata (title, content, reasoning,
/// origin, creation cycle, emotional context, invocation/questioning counts, etc.).
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

/// GET /api/ethics/personal/:id -- Detail view of a single personal ethical principle.
///
/// # Path parameters
/// * `id` - The unique identifier of the principle.
///
/// # Returns
/// Full JSON representation of the principle including supersession info and timestamps,
/// or `{"error": "not found"}` if no principle matches the given ID.
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

/// GET /api/ethics/readiness -- Ethical formulation readiness check.
///
/// Exposes the status of every precondition required for the agent to formulate
/// a new personal ethical principle. There are 7 conditions that must all be met:
/// 1. Minimum cycle count (>= 50 cycles of operation).
/// 2. Sufficient moral reflections accumulated.
/// 3. Consciousness level above the formulation threshold.
/// 4. Cortisol below 0.5 (not stressed).
/// 5. Serotonin at or above 0.4 (stable mood).
/// 6. Cooldown period since last formulation elapsed.
/// 7. Capacity available (below max personal principles limit).
///
/// # Returns
/// JSON with `enabled`, per-condition status (required/current/met), `all_met`,
/// `met_count`, and `total_conditions`.
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
