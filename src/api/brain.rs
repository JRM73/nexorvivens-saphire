// =============================================================================
// api/brain.rs — Handlers monde et connaissances (basiques)
//
// Role: Endpoints GET for the donnees du monde et les statistiques
// de connaissances acquises par l'agent.
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/world — Returns the data du model du monde.
pub async fn api_get_world(State(state): State<AppState>) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    axum::Json(agent.world_data())
}

/// GET /api/knowledge — Returns les statistiques de connaissances acquises.
pub async fn api_get_knowledge(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.knowledge_stats())
}
