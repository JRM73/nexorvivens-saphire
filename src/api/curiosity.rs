// =============================================================================
// api/curiosity.rs — Endpoint API pour le moteur de curiosite
//
// Role : Expose l'etat du moteur de curiosite de Saphire (faim par domaine,
// decouvertes, questions en attente) via un endpoint HTTP GET protege.
// =============================================================================

use axum::extract::State;
use axum::response::Json;
use serde_json::Value;

use super::state::AppState;

/// GET /api/curiosity/status — Etat du moteur de curiosite.
pub async fn api_curiosity_status(
    State(state): State<AppState>,
) -> Json<Value> {
    let agent = state.agent.lock().await;
    Json(agent.curiosity.to_snapshot_json())
}
