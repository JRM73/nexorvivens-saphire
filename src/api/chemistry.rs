// =============================================================================
// api/chemistry.rs — Handler neurochimie
//
// Role : Endpoint GET pour l'etat neurochimique actuel de l'agent.
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/chemistry — Retourne l'etat neurochimique actuel en JSON.
pub async fn api_get_chemistry(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.chemistry_json())
}
