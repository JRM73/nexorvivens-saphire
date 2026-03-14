// =============================================================================
// api/fields.rs — Handlers des champs electromagnetiques
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/fields/status — Etat complet des champs EM.
pub async fn api_fields_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.em_fields.to_json())
}

/// GET /api/fields/biofield — Etat du biochamp individuel.
pub async fn api_fields_biofield(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let bf = &agent.em_fields.biofield;
    axum::Json(serde_json::json!({
        "cardiac_em": bf.cardiac_em,
        "brainwave_coherence": bf.brainwave_coherence,
        "biofield_integrity": bf.biofield_integrity,
        "aura_luminosity": bf.aura_luminosity,
    }))
}
