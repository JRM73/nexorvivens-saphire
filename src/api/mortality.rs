// =============================================================================
// api/mortality.rs — Mortality endpoints
//
// Role: Mortality state, poison injection (test), resuscitation.
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/mortality — Current mortality state.
pub async fn api_mortality_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.body.mortality.to_json())
}

/// POST /api/mortality/poison — Inject a poison (test/scenario).
/// Body: { "amount": 0.5 }
pub async fn api_mortality_poison(
    State(state): State<AppState>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    let amount = body.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.3);
    agent.body.mortality.inject_poison(amount);
    axum::Json(serde_json::json!({
        "status": "ok",
        "toxicity": agent.body.mortality.toxicity,
    }))
}

/// POST /api/mortality/revive — Resuscitation (reset mortality to Alive).
/// Only works if allow_reboot_after_death is active.
pub async fn api_mortality_revive(State(state): State<AppState>) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    if agent.body.mortality.state.is_dead() {
        // Reset the mortality monitor
        let agony_cycles = agent.body.mortality.agony_max_cycles;
        agent.body.mortality = crate::body::mortality::MortalityMonitor::new(agony_cycles);
        axum::Json(serde_json::json!({ "status": "revived" }))
    } else {
        axum::Json(serde_json::json!({ "status": "not_dead", "message": "Saphire n'est pas morte" }))
    }
}
