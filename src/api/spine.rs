// =============================================================================
// api/spine.rs — Endpoint API for the spinal cord (reflexes, routage)
//
// Role: Expose l'state of the spinal cord de Saphire (reflexes,
// signaux traites, routage) via un endpoint HTTP GET protege.
// =============================================================================

use axum::extract::State;
use axum::response::Json;
use serde_json::Value;

use super::state::AppState;

/// GET /api/spine/status — Etat de la spinal cord.
pub async fn api_spine_status(
    State(state): State<AppState>,
) -> Json<Value> {
    let agent = state.agent.lock().await;
    Json(agent.spine.to_snapshot_json())
}
