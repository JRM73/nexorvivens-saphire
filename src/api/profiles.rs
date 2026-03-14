// =============================================================================
// api/profiles.rs — Neurodivergent cognitive profile handlers
//
// Role: Endpoints to list, load, compare and reset cognitive profiles
// (ADHD, autism, GAD, gifted, bipolar, OCD).
// =============================================================================

use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use super::state::AppState;

/// GET /api/profiles — List available profiles.
pub async fn api_list_profiles(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let profiles = agent.cognitive_profile_orch.list_profiles();
    Json(serde_json::json!({
        "profiles": profiles,
        "total": profiles.len(),
    }))
}

/// GET /api/profiles/current — Active profile + transition state + bipolar phase.
pub async fn api_current_profile(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    Json(agent.cognitive_profile_orch.to_status_json())
}

/// Request body for POST /api/profiles/load
#[derive(Deserialize)]
pub struct LoadProfileRequest {
    pub name: String,
}

/// POST /api/profiles/load — Load and apply a profile.
pub async fn api_load_profile(
    State(state): State<AppState>,
    Json(body): Json<LoadProfileRequest>,
) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    Json(agent.load_and_apply_profile(&body.name))
}

/// POST /api/profiles/reset — Revert to neurotypical profile.
pub async fn api_reset_profile(State(state): State<AppState>) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    Json(agent.load_and_apply_profile("neurotypique"))
}

/// Query parameters for GET /api/profiles/compare
#[derive(Deserialize)]
pub struct CompareQuery {
    pub a: String,
    pub b: String,
}

/// GET /api/profiles/compare?a=neurotypique&b=tdah — Compare two profiles.
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
