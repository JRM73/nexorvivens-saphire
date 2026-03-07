// =============================================================================
// api/brain.rs — World model and knowledge handlers (basic)
//
// This module provides GET endpoints for:
// - The agent's world model data (environmental awareness, contextual state).
// - Knowledge statistics (not available in the lite version).
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/world -- Returns the agent's world model data.
///
/// The world model contains the agent's current understanding of its environment,
/// temporal context, and external state. Requires a mutable lock because
/// `world_data()` may perform lazy initialization.
pub async fn api_get_world(State(state): State<AppState>) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    axum::Json(agent.world_data())
}

/// GET /api/knowledge -- Not available in the lite version.
///
/// Returns a stub JSON response indicating that the knowledge module
/// has not been ported to the lite build.
pub async fn api_get_knowledge(State(_state): State<AppState>) -> impl IntoResponse {
    axum::Json(serde_json::json!({
        "status": "not_available",
        "note": "knowledge module not ported in lite",
    }))
}
