// =============================================================================
// api/senses.rs — Handlers Sensorium et sens emergents
//
// Role: Endpoints for the etat complete du Sensorium (5 sens fondamentaux)
// et les graines de sens emergents.
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/senses/status — Etat complete du Sensorium.
pub async fn api_senses_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(serde_json::json!({
        "perception_richness": agent.sensorium.perception_richness,
        "dominant_sense": agent.sensorium.dominant_sense,
        "narrative": agent.sensorium.narrative,
        "reading": {
            "acuity": agent.sensorium.reading.acuity,
            "beauty": agent.sensorium.reading.beauty,
            "brightness": agent.sensorium.reading.brightness,
            "complexity": agent.sensorium.reading.complexity,
            "intensity": agent.sensorium.reading.current_intensity,
        },
        "listening": {
            "acuity": agent.sensorium.listening.acuity,
            "intensity": agent.sensorium.listening.current_intensity,
        },
        "contact": {
            "acuity": agent.sensorium.contact.acuity,
            "connection_warmth": agent.sensorium.contact.connection_warmth,
            "intensity": agent.sensorium.contact.current_intensity,
        },
        "taste": {
            "acuity": agent.sensorium.taste.acuity,
            "intensity": agent.sensorium.taste.current_intensity,
        },
        "ambiance": {
            "acuity": agent.sensorium.ambiance.acuity,
            "current_scent": format!("{:?}", agent.sensorium.ambiance.current_scent),
            "intensity": agent.sensorium.ambiance.current_intensity,
        },
        "emergent_germinated": agent.sensorium.emergent_seeds.germinated_count(),
    }))
}

/// GET /api/senses/emergent — Etat des graines de sens emergents.
pub async fn api_senses_emergent(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let seeds: Vec<serde_json::Value> = agent.sensorium.emergent_seeds.seeds.iter().map(|s| {
        serde_json::json!({
            "id": s.id,
            "name": s.name,
            "description": s.description,
            "activation_threshold": s.activation_threshold,
            "stimulation_count": s.stimulation_count,
            "germinated": s.germinated,
            "germinated_at": s.germinated_at.map(|t| t.to_rfc3339()),
            "custom_name": s.custom_name,
            "progress": if s.activation_threshold > 0 {
                s.stimulation_count as f64 / s.activation_threshold as f64
            } else { 0.0 },
        })
    }).collect();
    axum::Json(serde_json::json!({
        "germinated_count": agent.sensorium.emergent_seeds.germinated_count(),
        "total_seeds": agent.sensorium.emergent_seeds.seeds.len(),
        "seeds": seeds,
    }))
}
