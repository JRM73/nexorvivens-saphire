// =============================================================================
// api/factory.rs — Handlers valeurs d'usine (factory defaults)
//
// Role : Endpoints pour consulter les valeurs d'usine, comparer les
// differences avec les valeurs actuelles, et appliquer un reset.
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/factory/defaults — Retourne toutes les valeurs d'usine en JSON.
pub async fn api_factory_defaults() -> axum::Json<serde_json::Value> {
    match crate::factory::FactoryDefaults::load() {
        Ok(factory) => axum::Json(factory.as_json()),
        Err(e) => axum::Json(serde_json::json!({"error": e})),
    }
}

/// GET /api/factory/diff — Differences entre valeurs actuelles et usine.
pub async fn api_factory_diff(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.factory_diff())
}

/// POST /api/factory/reset — Applique un reset aux valeurs d'usine.
/// Body: { "level": "ChemistryOnly" | "ParametersOnly" | "SensesOnly" |
///         "IntuitionOnly" | "PersonalEthicsOnly" | "PsychologyOnly" |
///         "SleepOnly" | "BiologyReset" | "FullReset" }
pub async fn api_factory_reset(
    State(state): State<AppState>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let level_str = body.get("level").and_then(|l| l.as_str()).unwrap_or("chemistry_only");
    let level = match level_str {
        "chemistry_only" | "ChemistryOnly" => crate::factory::ResetLevel::ChemistryOnly,
        "parameters_only" | "ParametersOnly" => crate::factory::ResetLevel::ParametersOnly,
        "senses_only" | "SensesOnly" => crate::factory::ResetLevel::SensesOnly,
        "intuition_only" | "IntuitionOnly" => crate::factory::ResetLevel::IntuitionOnly,
        "personal_ethics_only" | "PersonalEthicsOnly" => crate::factory::ResetLevel::PersonalEthicsOnly,
        "psychology_only" | "PsychologyOnly" => crate::factory::ResetLevel::PsychologyOnly,
        "sleep_only" | "SleepOnly" => crate::factory::ResetLevel::SleepOnly,
        "biology_reset" | "BiologyReset" => crate::factory::ResetLevel::BiologyReset,
        "full_reset" | "FullReset" => crate::factory::ResetLevel::FullReset,
        _ => return axum::Json(serde_json::json!({"error": format!("Unknown level: {}", level_str)})),
    };
    let mut agent = state.agent.lock().await;
    axum::Json(agent.apply_factory_reset(level).await)
}
