// =============================================================================
// api/profiles.rs — Handlers des profils cognitifs neurodivergents
//
// Role : Endpoints pour lister, charger, comparer et reinitialiser les profils
// cognitifs (TDAH, autisme, TAG, HPI, bipolaire, TOC).
// =============================================================================

use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use super::state::AppState;

/// GET /api/profiles — Liste des profils disponibles.
pub async fn api_list_profiles(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let profiles = agent.cognitive_profile_orch.list_profiles();
    Json(serde_json::json!({
        "profiles": profiles,
        "total": profiles.len(),
    }))
}

/// GET /api/profiles/current — Profil actif + etat transition + phase bipolaire.
pub async fn api_current_profile(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    Json(agent.cognitive_profile_orch.to_status_json())
}

/// Corps de la requete POST /api/profiles/load
#[derive(Deserialize)]
pub struct LoadProfileRequest {
    pub name: String,
}

/// POST /api/profiles/load — Charge et applique un profil.
pub async fn api_load_profile(
    State(state): State<AppState>,
    Json(body): Json<LoadProfileRequest>,
) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    Json(agent.load_and_apply_profile(&body.name))
}

/// POST /api/profiles/reset — Revenir au profil neurotypique.
pub async fn api_reset_profile(State(state): State<AppState>) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    Json(agent.load_and_apply_profile("neurotypique"))
}

/// Parametres de la requete GET /api/profiles/compare
#[derive(Deserialize)]
pub struct CompareQuery {
    pub a: String,
    pub b: String,
}

/// GET /api/profiles/compare?a=neurotypique&b=tdah — Compare deux profils.
pub async fn api_compare_profiles(
    State(state): State<AppState>,
    Query(query): Query<CompareQuery>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    match agent.cognitive_profile_orch.compare_profiles(&query.a, &query.b) {
        Ok(diff) => Json(diff),
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}
