// =============================================================================
// api/chemistry.rs — Neurochemistry handler
//
// This module exposes a single GET endpoint that returns the agent's current
// neurochemical state as JSON. The neurochemistry model simulates
// neurotransmitter levels (dopamine, cortisol, serotonin, adrenaline,
// oxytocin, endorphin, noradrenaline) that influence the agent's emotions,
// decisions, and behavior.
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/chemistry -- Returns the current neurochemical state as JSON.
///
/// Acquires the agent lock and serializes all neurotransmitter levels,
/// their baselines, and derived emotional indicators.
pub async fn api_get_chemistry(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.chemistry_json())
}
