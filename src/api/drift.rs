// =============================================================================
// api/drift.rs — Endpoint API pour le moniteur de derive de persona
//
// Role : Expose l'etat du drift monitor de Saphire (similarite au centroide
// d'identite, tendance, alertes) via un endpoint HTTP GET protege.
// =============================================================================

use axum::extract::State;
use axum::response::Json;
use serde_json::Value;

use super::state::AppState;

/// GET /api/drift/status — Etat du moniteur de derive de persona.
pub async fn api_drift_status(
    State(state): State<AppState>,
) -> Json<Value> {
    let agent = state.agent.lock().await;
    Json(agent.drift_monitor.to_snapshot_json())
}
