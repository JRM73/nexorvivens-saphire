// =============================================================================
// api/relationships.rs — Handlers liens affectifs et situation familiale
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/relationships — Reseau de liens affectifs complete.
pub async fn api_relationships(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.relationships.to_json())
}

/// GET /api/relationships/chemistry — Influence chemical des relations.
pub async fn api_relationships_chemistry(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let adj = agent.relationships.chemistry_influence();
    axum::Json(serde_json::json!({
        "oxytocin": adj.oxytocin,
        "cortisol": adj.cortisol,
        "serotonin": adj.serotonin,
        "dopamine": adj.dopamine,
    }))
}

/// GET /api/family — Situation familiale complete.
pub async fn api_family(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let family = crate::relationships::family::FamilyContext::from_config(&agent.config().family);
    axum::Json(family.to_json())
}

/// GET /api/family/chemistry — Influence chemical de la famille.
pub async fn api_family_chemistry(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let family = crate::relationships::family::FamilyContext::from_config(&agent.config().family);
    let adj = family.chemistry_influence();
    axum::Json(serde_json::json!({
        "oxytocin": adj.oxytocin,
        "cortisol": adj.cortisol,
        "serotonin": adj.serotonin,
    }))
}
