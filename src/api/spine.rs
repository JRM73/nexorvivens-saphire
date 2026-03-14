// =============================================================================
// api/spine.rs — Endpoint API pour la colonne vertebrale (reflexes, routage)
//
// Role : Expose l'etat de la colonne vertebrale de Saphire (reflexes,
// signaux traites, routage) via un endpoint HTTP GET protege.
// =============================================================================

use axum::extract::State;
use axum::response::Json;
use serde_json::Value;

use super::state::AppState;

/// GET /api/spine/status — Etat de la colonne vertebrale.
pub async fn api_spine_status(
    State(state): State<AppState>,
) -> Json<Value> {
    let agent = state.agent.lock().await;
    Json(agent.spine.to_snapshot_json())
}
