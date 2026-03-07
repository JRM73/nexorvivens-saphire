// =============================================================================
// api/factory.rs — Factory defaults handlers
//
// This module exposes HTTP endpoints for managing factory default values:
// - Retrieving all factory defaults as JSON.
// - Computing the diff between current agent values and factory defaults.
// - Applying a factory reset at various levels (chemistry only, parameters
//   only, senses, intuition, personal ethics, psychology, or full reset).
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/factory/defaults -- Returns all factory default values as JSON.
///
/// Loads the factory defaults from the embedded configuration and serializes them.
/// No agent state is required; this is a stateless endpoint.
pub async fn api_factory_defaults() -> axum::Json<serde_json::Value> {
    match crate::factory::FactoryDefaults::load() {
        Ok(factory) => axum::Json(factory.as_json()),
        Err(e) => axum::Json(serde_json::json!({"error": e})),
    }
}

/// GET /api/factory/diff -- Returns the differences between current values and factory defaults.
///
/// Compares the agent's current configuration and neurochemistry against the
/// factory defaults and returns only the fields that differ.
pub async fn api_factory_diff(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.factory_diff())
}

/// POST /api/factory/reset -- Applies a factory reset at the specified level.
///
/// # Request body
/// JSON object with a `"level"` field specifying the reset scope:
/// - `"chemistry_only"` / `"ChemistryOnly"`: reset neurochemistry only.
/// - `"parameters_only"` / `"ParametersOnly"`: reset general parameters only.
/// - `"senses_only"` / `"SensesOnly"`: reset sensory configuration only.
/// - `"intuition_only"` / `"IntuitionOnly"`: reset intuition engine only.
/// - `"personal_ethics_only"` / `"PersonalEthicsOnly"`: reset personal ethics only.
/// - `"psychology_only"` / `"PsychologyOnly"`: reset psychology module only.
/// - `"full_reset"` / `"FullReset"`: reset everything to factory defaults.
///
/// # Returns
/// JSON result from `agent.apply_factory_reset()`, or an error for unknown levels.
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
        "full_reset" | "FullReset" => crate::factory::ResetLevel::FullReset,
        _ => return axum::Json(serde_json::json!({"error": format!("Unknown level: {}", level_str)})),
    };
    let mut agent = state.agent.lock().await;
    axum::Json(agent.apply_factory_reset(level).await)
}
