// =============================================================================
// api/grey_matter.rs — Handlers du substrat cerebral physique
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/grey-matter/status — Etat complet de la matiere grise.
pub async fn api_grey_matter_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.grey_matter.to_json())
}

/// GET /api/grey-matter/bdnf — Detail du BDNF et de la neuroplasticite.
pub async fn api_grey_matter_bdnf(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let gm = &agent.grey_matter;
    axum::Json(serde_json::json!({
        "bdnf_level": gm.bdnf_level,
        "neuroplasticity": gm.neuroplasticity,
        "neurogenesis_rate": gm.neurogenesis_rate,
        "synaptic_density": gm.synaptic_density,
        "grey_matter_volume": gm.grey_matter_volume,
        "myelination": gm.myelination,
    }))
}
