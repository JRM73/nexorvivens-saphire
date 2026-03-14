// =============================================================================
// api/personalities.rs — Personality preset handlers
//
// Role: Endpoints to list, load, compare and reset personality presets
// (philosopher, artist, scientist, empathetic, etc.).
// =============================================================================

use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use super::state::AppState;

/// GET /api/personalities — List available presets.
pub async fn api_list_personalities(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let presets = agent.personality_preset_orch.list_presets();
    Json(serde_json::json!({
        "presets": presets,
        "total": presets.len(),
    }))
}

/// GET /api/personalities/current — Active preset + transition state.
pub async fn api_current_personality(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    Json(agent.personality_preset_orch.to_status_json())
}

/// Request body for POST /api/personalities/load
#[derive(Deserialize)]
pub struct LoadPersonalityRequest {
    pub name: String,
}

/// POST /api/personalities/load — Load and apply a preset.
pub async fn api_load_personality(
    State(state): State<AppState>,
    Json(body): Json<LoadPersonalityRequest>,
) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    Json(agent.load_and_apply_personality(&body.name))
}

/// POST /api/personalities/reset — Revert to saphire preset.
pub async fn api_reset_personality(State(state): State<AppState>) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    Json(agent.load_and_apply_personality("saphire"))
}

/// Query parameters for GET /api/personalities/compare
#[derive(Deserialize)]
pub struct ComparePersonalityQuery {
    pub a: String,
    pub b: String,
}

/// GET /api/personalities/compare?a=saphire&b=artiste — Compare two presets.
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
