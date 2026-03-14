// =============================================================================
// api/temperament.rs — Endpoint temperament emergent
//
// Role : Expose les traits de temperament deduits (timidite, courage, etc.)
// via GET /api/temperament pour le dashboard.
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;
use serde_json::json;

use super::AppState;

/// GET /api/temperament — Traits de temperament emergent
pub async fn api_temperament_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let temp = &agent.temperament;
    if temp.traits.is_empty() {
        return axum::Json(json!({
            "status": "not_computed",
            "traits": []
        }));
    }
    axum::Json(json!({
        "status": "active",
        "traits": temp.traits.iter().map(|t| json!({
            "name": t.name,
            "score": t.score,
            "category": t.category.as_str(),
        })).collect::<Vec<_>>(),
        "computed_at": temp.computed_at.to_rfc3339(),
        "data_points": temp.data_points,
    }))
}
