// =============================================================================
// api/needs.rs — Handlers besoins primaires (faim, soif)
//
// Role: Endpoints for consulter l'state ofs besoins primaires et declencher
// manually the eat/drink actions.// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/needs — Etat complete of the needs primaires.
pub async fn api_needs_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.needs.to_status_json())
}

/// POST /api/needs/eat — Declencher un repas manuellement.
pub async fn api_needs_eat(State(state): State<AppState>) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    if !agent.needs.enabled {
        return axum::Json(serde_json::json!({"error": "needs module disabled"}));
    }
    let cycle = agent.cycle_count;
    let config = agent.config().needs.clone();
    let result = agent.needs.eat(cycle, &config);
    // Apply the boost to the physiology    agent.body.physiology.glycemia = result.glycemia_target;
    agent.chemistry.dopamine = (agent.chemistry.dopamine + result.dopamine_boost).min(1.0);
    axum::Json(serde_json::json!({
        "action": "eat",
        "glycemia_target": result.glycemia_target,
        "dopamine_boost": result.dopamine_boost,
        "meals_count": agent.needs.hunger.meals_count,
        "hunger_level": agent.needs.hunger.level,
    }))
}

/// POST /api/needs/drink — Declencher une boisson manuellement.
pub async fn api_needs_drink(State(state): State<AppState>) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    if !agent.needs.enabled {
        return axum::Json(serde_json::json!({"error": "needs module disabled"}));
    }
    let cycle = agent.cycle_count;
    let config = agent.config().needs.clone();
    let result = agent.needs.drink(cycle, &config);
    // Apply the boost to the physiology    agent.body.physiology.hydration = result.hydration_target;
    agent.chemistry.dopamine = (agent.chemistry.dopamine + result.dopamine_boost).min(1.0);
    axum::Json(serde_json::json!({
        "action": "drink",
        "hydration_target": result.hydration_target,
        "dopamine_boost": result.dopamine_boost,
        "drinks_count": agent.needs.thirst.drinks_count,
        "thirst_level": agent.needs.thirst.level,
    }))
}
