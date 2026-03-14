// =============================================================================
// api/conditions.rs — Conditions and afflictions endpoints
//
// Role: Phobias, motion sickness, and future conditions.
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/conditions/phobias — Phobias status.
pub async fn api_phobias_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.phobia_manager.to_json())
}

/// GET /api/conditions/motion_sickness — Motion sickness status.
pub async fn api_motion_sickness_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.motion_sickness.to_json())
}

/// POST /api/conditions/motion_sickness/trigger — Trigger an episode.
/// Body: { "type": "vertigo" } (air, sea, land, vertigo, barotrauma)
pub async fn api_motion_sickness_trigger(
    State(state): State<AppState>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    let motion_type = match body.get("type").and_then(|v| v.as_str()).unwrap_or("vertigo") {
        "air" => crate::conditions::motion_sickness::MotionType::Air,
        "sea" => crate::conditions::motion_sickness::MotionType::Sea,
        "land" => crate::conditions::motion_sickness::MotionType::Land,
        "barotrauma" => crate::conditions::motion_sickness::MotionType::Barotrauma,
        _ => crate::conditions::motion_sickness::MotionType::Vertigo,
    };
    agent.motion_sickness.trigger(motion_type);
    axum::Json(serde_json::json!({ "status": "ok", "nausea": agent.motion_sickness.current_nausea }))
}

/// GET /api/conditions/eating — Eating disorder status.
pub async fn api_eating_disorder_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let json = match &agent.eating_disorder {
        Some(ed) => ed.to_json(),
        None => serde_json::json!({ "active": false }),
    };
    axum::Json(json)
}

/// GET /api/conditions/disabilities — Disabilities status.
pub async fn api_disabilities_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.disability_manager.to_json())
}

/// GET /api/conditions/extreme — Extreme conditions status.
pub async fn api_extreme_condition_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.extreme_condition_mgr.to_json())
}

/// GET /api/conditions/addictions — Addictions status.
pub async fn api_addictions_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.addiction_manager.to_json())
}

/// GET /api/conditions/trauma — PTSD and trauma status.
pub async fn api_trauma_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.ptsd.to_json())
}

/// GET /api/conditions/nde — Near-death experience status.
pub async fn api_nde_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.nde.to_json())
}

/// GET /api/conditions/drugs — Active drugs.
pub async fn api_drugs_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.drug_manager.to_json())
}

/// POST /api/conditions/drugs/administer — Administer a drug.
/// Body: { "substance": "caffeine" }
pub async fn api_drugs_administer(
    State(state): State<AppState>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    let substance = body.get("substance").and_then(|v| v.as_str()).unwrap_or("caffeine");
    match crate::conditions::drugs::drug_catalog(substance) {
        Some(profile) => {
            agent.drug_manager.administer(profile);
            axum::Json(serde_json::json!({ "status": "ok", "substance": substance }))
        }
        None => {
            axum::Json(serde_json::json!({ "status": "error", "message": "Substance inconnue" }))
        }
    }
}

/// GET /api/conditions/iq — IQ constraint.
pub async fn api_iq_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let json = match &agent.iq_constraint {
        Some(iq) => iq.to_json(),
        None => serde_json::json!({ "active": false }),
    };
    axum::Json(json)
}

/// GET /api/conditions/sexuality — Sexuality module.
pub async fn api_sexuality_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let json = match &agent.sexuality {
        Some(s) => s.to_json(),
        None => serde_json::json!({ "active": false }),
    };
    axum::Json(json)
}

/// GET /api/conditions/degenerative — Degenerative diseases.
pub async fn api_degenerative_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.degenerative_mgr.to_json())
}

/// GET /api/conditions/medical — General medical conditions.
pub async fn api_medical_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.medical_mgr.to_json())
}

/// GET /api/conditions/culture — Cultural framework.
pub async fn api_culture_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let json = match &agent.culture {
        Some(c) => c.to_json(),
        None => serde_json::json!({ "active": false }),
    };
    axum::Json(json)
}

/// GET /api/conditions/precarity — Precarity status.
pub async fn api_precarity_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let json = match &agent.precarity {
        Some(p) => p.to_json(),
        None => serde_json::json!({ "active": false }),
    };
    axum::Json(json)
}

/// GET /api/conditions/employment — Employment status.
pub async fn api_employment_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let json = match &agent.employment {
        Some(e) => e.to_json(),
        None => serde_json::json!({ "active": false }),
    };
    axum::Json(json)
}
