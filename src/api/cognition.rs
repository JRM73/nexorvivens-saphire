// =============================================================================
// api/cognition.rs — Handlers des 8 modules cognitifs avances
//
// Role: Endpoints for the theorie de l'esprit (ToM), le inner monologue,
// la dissonance cognitive, la memoire prospective, l'identite narrative,
// le raisonnement analogique, la charge cognitive, l'imagerie mentale,
// et les extensions metacognitives (source monitoring, confirmation bias).
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

// =============================================================================
// 1. Theory of Mind (ToM)
// =============================================================================
/// GET /api/tom/status — Etat du moteur de theorie de l'esprit.
pub async fn api_tom_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.tom.to_json())
}

// =============================================================================
// 2. Monologue interieur
// =============================================================================
/// GET /api/monologue/chain — Chaine du inner monologue.
pub async fn api_monologue_chain(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.inner_monologue.to_json())
}

// =============================================================================
// 3-4. Dissonance cognitive
// =============================================================================
/// GET /api/dissonance/status — Etat du moteur de dissonance cognitive.
pub async fn api_dissonance_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.dissonance.to_json())
}

/// GET /api/dissonance/beliefs — Registre des croyances actives.
pub async fn api_dissonance_beliefs(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(serde_json::json!({
        "beliefs": agent.dissonance.beliefs.iter().map(|b| serde_json::json!({
            "id": b.id,
            "content": b.content,
            "domain": b.domain,
            "strength": b.strength,
            "formed_at_cycle": b.formed_at_cycle,
            "confirmed_count": b.confirmed_count,
            "contradicted_count": b.contradicted_count,
        })).collect::<Vec<_>>(),
        "count": agent.dissonance.beliefs.len(),
    }))
}

// =============================================================================
// 5. Memoire prospective
// =============================================================================
/// GET /api/prospective/intentions — Intentions differees en attente.
pub async fn api_prospective_intentions(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.prospective_mem.to_json())
}

// =============================================================================
// 6-7. Identite narrative
// =============================================================================
/// GET /api/narrative/identity — Resume de l'identite narrative.
pub async fn api_narrative_identity(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.narrative_identity.to_json())
}

/// GET /api/narrative/chapters — Chapitres de l'histoire de vie.
pub async fn api_narrative_chapters(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(serde_json::json!({
        "chapters": agent.narrative_identity.chapters.iter().map(|c| serde_json::json!({
            "id": c.id,
            "title": c.title,
            "summary": c.summary,
            "themes": c.themes,
            "dominant_emotion": c.dominant_emotion,
            "growth_score": c.growth_score,
            "is_turning_point": c.is_turning_point,
            "started_at_cycle": c.started_at_cycle,
            "ended_at_cycle": c.ended_at_cycle,
        })).collect::<Vec<_>>(),
        "current_narrative": agent.narrative_identity.current_narrative,
        "narrative_cohesion": agent.narrative_identity.narrative_cohesion,
    }))
}

// =============================================================================
// 8. Raisonnement analogique
// =============================================================================
/// GET /api/analogies/recent — Analogies recentes generees.
pub async fn api_analogies_recent(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.analogical.to_json())
}

// =============================================================================
// 9. Charge cognitive
// =============================================================================
/// GET /api/cognitive-load/status — Etat de la charge cognitive.
pub async fn api_cognitive_load_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.cognitive_load.to_json())
}

// =============================================================================
// 10. Imagerie mentale
// =============================================================================
/// GET /api/imagery/active — Images mentales actives.
pub async fn api_imagery_active(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.imagery.to_json())
}

// =============================================================================
// 11-12. Extensions metacognitives
// =============================================================================
/// GET /api/source-monitor/traced — Suivi de l'origine des connaissances.
pub async fn api_source_monitor_traced(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.metacognition.source_monitor.to_json())
}

/// GET /api/metacognition/biases — Detection des confirmation bias.
pub async fn api_metacognition_biases(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.metacognition.bias_detector.to_json())
}

// =============================================================================
// Sentiments (etats affectifs durables)
// =============================================================================
/// GET /api/sentiments/status — Etat current des sentiments active.
pub async fn api_sentiments_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.sentiments.to_json())
}

/// GET /api/sentiments/history — Historique et statistiques des sentiments.
pub async fn api_sentiments_history(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.sentiments.history_json())
}

// =============================================================================
// Algorithmes Game AI (influence map, FSM cognitive, steering)
// =============================================================================
/// GET /api/influence-map/status — Etat de la carte d'influence attentionnelle.
pub async fn api_influence_map_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.influence_map.to_json())
}

/// GET /api/cognitive-fsm/status — Etat de la machine a etats cognitive.
pub async fn api_cognitive_fsm_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.cognitive_fsm.to_json())
}

/// GET /api/steering/status — Etat du moteur de steering emotionnel.
pub async fn api_steering_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(serde_json::json!({
        "equilibrium": {
            "valence": agent.steering_engine.params.equilibrium.valence,
            "arousal": agent.steering_engine.params.equilibrium.arousal,
        },
        "flee_radius": agent.steering_engine.params.flee_radius,
        "arrive_radius": agent.steering_engine.params.arrive_radius,
        "wander_strength": agent.steering_engine.params.wander_strength,
        "max_force": agent.steering_engine.params.max_force,
    }))
}

// =============================================================================
// MAP Sync (Modulateur Adaptatif de Proprioception)
// =============================================================================
/// GET /api/map-sync/status — Etat du MAP (tension perception/reaction).
pub async fn api_map_sync_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.map_sync.to_broadcast_json())
}
