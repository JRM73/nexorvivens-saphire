// =============================================================================
// api/nutrition.rs — Handlers du systeme nutritionnel
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/nutrition/status — Etat complet du systeme nutritionnel.
pub async fn api_nutrition_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.nutrition.to_json())
}

/// GET /api/nutrition/deficiencies — Liste des carences detectees.
pub async fn api_nutrition_deficiencies(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let config = agent.config().nutrition.clone();
    let defs = agent.nutrition.deficiencies(&config);
    axum::Json(serde_json::json!({
        "deficiencies": defs.iter().map(|(name, level)| {
            serde_json::json!({ "nutrient": name, "level": level })
        }).collect::<Vec<_>>(),
        "count": defs.len(),
    }))
}
