// =============================================================================
// api/personalities.rs — Handlers des presets de personnalite
//
// Role : Endpoints pour lister, charger, comparer et reinitialiser les presets
// de personnalite (philosophe, artiste, scientifique, empathique, etc.).
// =============================================================================

use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use super::state::AppState;

/// GET /api/personalities — Liste des presets disponibles.
pub async fn api_list_personalities(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let presets = agent.personality_preset_orch.list_presets();
    Json(serde_json::json!({
        "presets": presets,
        "total": presets.len(),
    }))
}

/// GET /api/personalities/current — Preset actif + etat transition.
pub async fn api_current_personality(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    Json(agent.personality_preset_orch.to_status_json())
}

/// Corps de la requete POST /api/personalities/load
#[derive(Deserialize)]
pub struct LoadPersonalityRequest {
    pub name: String,
}

/// POST /api/personalities/load — Charge et applique un preset.
pub async fn api_load_personality(
    State(state): State<AppState>,
    Json(body): Json<LoadPersonalityRequest>,
) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    Json(agent.load_and_apply_personality(&body.name))
}

/// POST /api/personalities/reset — Revenir au preset saphire.
pub async fn api_reset_personality(State(state): State<AppState>) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    Json(agent.load_and_apply_personality("saphire"))
}

/// Parametres de la requete GET /api/personalities/compare
#[derive(Deserialize)]
pub struct ComparePersonalityQuery {
    pub a: String,
    pub b: String,
}

/// GET /api/personalities/compare?a=saphire&b=artiste — Compare deux presets.
pub async fn api_compare_personalities(
    State(state): State<AppState>,
    Query(query): Query<ComparePersonalityQuery>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    match agent.personality_preset_orch.compare_presets(&query.a, &query.b) {
        Ok(diff) => Json(diff),
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}
