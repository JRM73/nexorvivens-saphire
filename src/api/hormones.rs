// =============================================================================
// api/hormones.rs — Endpoints API for the systeme hormonal
//
// Role: Expose l'etat hormonal, les recepteurs et the phase circadienne
// via des endpoints HTTP GET proteges par auth + rate limit.
// =============================================================================

use axum::extract::State;
use axum::response::Json;
use serde_json::Value;

use super::state::AppState;

/// GET /api/hormones — Etat hormonal complet + recepteurs + phase circadienne.
pub async fn api_hormones_status(
    State(state): State<AppState>,
) -> Json<Value> {
    let agent = state.agent.lock().await;
    let system = &agent.hormonal_system;

    Json(serde_json::json!({
        "enabled": system.enabled,
        "state": {
            "cortisol_h": system.state.cortisol_h,
            "melatonin": system.state.melatonin,
            "epinephrine": system.state.epinephrine,
            "testosterone": system.state.testosterone,
            "estrogen": system.state.estrogen,
            "oxytocin_h": system.state.oxytocin_h,
            "insulin": system.state.insulin,
            "thyroid": system.state.thyroid,
        },
        "circadian_phase": system.circadian_phase,
        "circadian_time": crate::hormones::circadian_time_label(system.circadian_phase),
        "cycle_counter": system.cycle_counter,
        "receptors": system.receptors.to_snapshot_json(),
    }))
}

/// GET /api/hormones/receptors — Detail des neuroreceptors.
pub async fn api_hormones_receptors(
    State(state): State<AppState>,
) -> Json<Value> {
    let agent = state.agent.lock().await;
    let receptors = &agent.hormonal_system.receptors;

    Json(serde_json::json!({
        "receptors": receptors.to_snapshot_json(),
        "desensitized": receptors.describe_desensitized(),
    }))
}

/// GET /api/hormones/cycle — Phase circadienne et rythmes.
pub async fn api_hormones_cycle(
    State(state): State<AppState>,
) -> Json<Value> {
    let agent = state.agent.lock().await;
    let system = &agent.hormonal_system;

    Json(serde_json::json!({
        "circadian_phase": system.circadian_phase,
        "circadian_time": crate::hormones::circadian_time_label(system.circadian_phase),
        "cycle_counter": system.cycle_counter,
        "melatonin": system.state.melatonin,
        "cortisol_h": system.state.cortisol_h,
        "insulin": system.state.insulin,
    }))
}
