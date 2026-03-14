// =============================================================================
// api/psychology.rs — Endpoints API for the cadres psychologiques
//
// 22 handlers au total :
//   7 endpoints /api/psychology/* — vues d'ensemble (existants)
//   3 endpoints /api/psyche/*    — detail Freud (Ca/Moi/Surmoi)
//  3 endpoints /api/maslow/* — pyramide of the needs
//   2 endpoints /api/toltec/*    — 4 accords tolteques
//   2 endpoints /api/shadow/*    — ombre jungienne
//   2 endpoints /api/eq/*        — intelligence emotionnelle
//  2 endpoints /api/flow/* — etat de flow
//   1 endpoint  /api/model/info  — info model LLM
// =============================================================================

use std::collections::HashMap;
use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

// ─── /api/psychology/* — Vues d'ensemble ────────────────────────────────────
/// GET /api/psychology/status — Vue d'ensemble des 6 cadres.
pub async fn api_psychology_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let p = &agent.psychology;
    axum::Json(serde_json::json!({
        "enabled": p.enabled,
        "freudian": {
            "dominant_axis": p.freudian.balance.dominant_axis,
            "psychic_health": p.freudian.balance.psychic_health,
            "internal_conflict": p.freudian.balance.internal_conflict,
        },
        "maslow": {
            "current_level": p.maslow.current_active_level,
            "level_name": p.maslow.levels[p.maslow.current_active_level].name,
            "satisfaction": p.maslow.levels[p.maslow.current_active_level].satisfaction,
        },
        "toltec": {
            "overall_alignment": p.toltec.overall_alignment,
        },
        "jung": {
            "dominant_archetype": format!("{:?}", p.jung.dominant_archetype),
            "integration": p.jung.integration,
            "shadow_leaking": p.jung.shadow_traits.iter().any(|t| t.leaking),
        },
        "eq": {
            "overall_eq": p.eq.overall_eq,
            "growth_experiences": p.eq.growth_experiences,
        },
        "flow": {
            "in_flow": p.flow.in_flow,
            "flow_intensity": p.flow.flow_intensity,
            "total_flow_cycles": p.flow.total_flow_cycles,
        },
    }))
}

/// GET /api/psychology/freudian — Detail du model freudien.
pub async fn api_psychology_freudian(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let f = &agent.psychology.freudian;
    axum::Json(serde_json::json!({
        "id": {
            "drive_strength": f.id.drive_strength,
            "active_drives": f.id.active_drives,
            "frustration": f.id.frustration,
        },
        "ego": {
            "strength": f.ego.strength,
            "anxiety": f.ego.anxiety,
            "strategy": format!("{:?}", f.ego.strategy),
        },
        "superego": {
            "strength": f.superego.strength,
            "guilt": f.superego.guilt,
            "pride": f.superego.pride,
        },
        "balance": {
            "dominant_axis": f.balance.dominant_axis,
            "ego_effectiveness": f.balance.ego_effectiveness,
            "internal_conflict": f.balance.internal_conflict,
            "psychic_health": f.balance.psychic_health,
        },
        "active_defenses": f.active_defenses,
    }))
}

/// GET /api/psychology/maslow — Pyramide de Maslow.
pub async fn api_psychology_maslow(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let m = &agent.psychology.maslow;
    axum::Json(serde_json::json!({
        "levels": m.levels,
        "current_active_level": m.current_active_level,
    }))
}

/// GET /api/psychology/toltec — 4 Accords Tolteques.
pub async fn api_psychology_toltec(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let t = &agent.psychology.toltec;
    axum::Json(serde_json::json!({
        "agreements": t.agreements,
        "overall_alignment": t.overall_alignment,
    }))
}

/// GET /api/psychology/shadow — Ombre jungienne.
pub async fn api_psychology_shadow(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let j = &agent.psychology.jung;
    axum::Json(serde_json::json!({
        "shadow_traits": j.shadow_traits,
        "integration": j.integration,
        "dominant_archetype": format!("{:?}", j.dominant_archetype),
    }))
}

/// GET /api/psychology/eq — Intelligence emotionnelle.
pub async fn api_psychology_eq(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let e = &agent.psychology.eq;
    axum::Json(serde_json::json!({
        "self_awareness": e.self_awareness,
        "self_regulation": e.self_regulation,
        "motivation": e.motivation,
        "empathy": e.empathy,
        "social_skills": e.social_skills,
        "overall_eq": e.overall_eq,
        "growth_experiences": e.growth_experiences,
    }))
}

/// GET /api/psychology/flow — Etat de flow.
pub async fn api_psychology_flow(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let f = &agent.psychology.flow;
    axum::Json(serde_json::json!({
        "in_flow": f.in_flow,
        "flow_intensity": f.flow_intensity,
        "perceived_challenge": f.perceived_challenge,
        "perceived_skill": f.perceived_skill,
        "duration_cycles": f.duration_cycles,
        "total_flow_cycles": f.total_flow_cycles,
    }))
}

// ─── /api/psyche/* — Detail Freud (Ca/Moi/Surmoi) ──────────────────────────
/// GET /api/psyche/status — Etat psychique complet (forces, equilibre, sante).
pub async fn api_psyche_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let f = &agent.psychology.freudian;
    axum::Json(serde_json::json!({
        "id": {
            "drive_strength": f.id.drive_strength,
            "frustration": f.id.frustration,
            "active_drives": f.id.active_drives,
        },
        "ego": {
            "strength": f.ego.strength,
            "anxiety": f.ego.anxiety,
            "strategy": format!("{:?}", f.ego.strategy),
        },
        "superego": {
            "strength": f.superego.strength,
            "guilt": f.superego.guilt,
            "pride": f.superego.pride,
        },
        "balance": {
            "dominant_axis": f.balance.dominant_axis,
            "ego_effectiveness": f.balance.ego_effectiveness,
            "internal_conflict": f.balance.internal_conflict,
            "psychic_health": f.balance.psychic_health,
        },
        "active_defenses": f.active_defenses,
    }))
}

/// GET /api/psyche/defenses — Mecanismes de defense active.
pub async fn api_psyche_defenses(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let f = &agent.psychology.freudian;
    axum::Json(serde_json::json!({
        "active_defenses": f.active_defenses,
        "ego_anxiety": f.ego.anxiety,
        "internal_conflict": f.balance.internal_conflict,
        "strategy": format!("{:?}", f.ego.strategy),
    }))
}

/// GET /api/psyche/drives — Pulsions actives du Ca.
pub async fn api_psyche_drives(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let f = &agent.psychology.freudian;
    axum::Json(serde_json::json!({
        "active_drives": f.id.active_drives,
        "drive_strength": f.id.drive_strength,
        "frustration": f.id.frustration,
    }))
}

// ─── /api/maslow/* — Pyramide of the needs ───────────────────────────────────
/// GET /api/maslow/pyramid — Pyramide complete with tous the levels.
pub async fn api_maslow_pyramid(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let m = &agent.psychology.maslow;
    axum::Json(serde_json::json!({
        "levels": m.levels,
        "current_active_level": m.current_active_level,
        "current_level_name": m.levels[m.current_active_level].name,
    }))
}

/// GET /api/maslow/ceiling — Plafond current de la pyramide.
pub async fn api_maslow_ceiling(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let m = &agent.psychology.maslow;
    let level = &m.levels[m.current_active_level];
    axum::Json(serde_json::json!({
        "ceiling": m.current_active_level,
        "ceiling_name": level.name,
        "satisfaction": level.satisfaction,
        "threshold": level.threshold,
        "all_satisfactions": m.levels.iter().map(|l| {
            serde_json::json!({
                "name": l.name,
                "satisfaction": l.satisfaction,
                "satisfied": l.satisfaction >= l.threshold
            })
        }).collect::<Vec<_>>(),
    }))
}

/// GET /api/maslow/needs — Besoins prioritaires (niveaux non satisfaits).
pub async fn api_maslow_needs(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let m = &agent.psychology.maslow;
    let needs: Vec<serde_json::Value> = m.levels.iter().enumerate()
        .filter(|(_, l)| l.satisfaction < l.threshold)
        .map(|(i, l)| serde_json::json!({
            "level": i,
            "name": l.name,
            "satisfaction": l.satisfaction,
            "threshold": l.threshold,
            "deficit": l.threshold - l.satisfaction,
        }))
        .collect();
    axum::Json(serde_json::json!({
        "priority_needs": needs,
        "current_active_level": m.current_active_level,
    }))
}

// ─── /api/toltec/* — Accords tolteques ──────────────────────────────────────
/// GET /api/toltec/agreements — Detail des 4 accords.
pub async fn api_toltec_agreements(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let t = &agent.psychology.toltec;
    axum::Json(serde_json::json!({
        "agreements": t.agreements,
        "overall_alignment": t.overall_alignment,
    }))
}

/// GET /api/toltec/history — Statistiques invocations/violations.
pub async fn api_toltec_history(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let t = &agent.psychology.toltec;
    let stats: Vec<serde_json::Value> = t.agreements.iter().map(|a| {
        serde_json::json!({
            "number": a.number,
            "name": a.name,
            "alignment": a.alignment,
            "times_invoked": a.times_invoked,
            "violations_detected": a.violations_detected,
            "violation_rate": if a.times_invoked > 0 {
                a.violations_detected as f64 / a.times_invoked as f64
            } else { 0.0 },
        })
    }).collect();
    axum::Json(serde_json::json!({
        "agreement_stats": stats,
        "overall_alignment": t.overall_alignment,
    }))
}

// ─── /api/shadow/* — Ombre jungienne ────────────────────────────────────────
/// GET /api/shadow/status — Etat de l'ombre et des archetypes.
pub async fn api_shadow_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let j = &agent.psychology.jung;
    axum::Json(serde_json::json!({
        "shadow_traits": j.shadow_traits,
        "integration": j.integration,
        "dominant_archetype": format!("{:?}", j.dominant_archetype),
        "leaking_traits": j.shadow_traits.iter()
            .filter(|t| t.leaking)
            .map(|t| serde_json::json!({
                "name": t.name,
                "intensity": t.repressed_intensity,
            }))
            .collect::<Vec<_>>(),
    }))
}

/// GET /api/shadow/archetype_history — Historique des archetypes (via metriques).
pub async fn api_shadow_archetype_history(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_shadow_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API psychology: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

// ─── /api/eq/* — Intelligence emotionnelle ──────────────────────────────────
/// GET /api/eq/status — Score EQ et 5 composantes.
pub async fn api_eq_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let e = &agent.psychology.eq;
    axum::Json(serde_json::json!({
        "overall_eq": e.overall_eq,
        "self_awareness": e.self_awareness,
        "self_regulation": e.self_regulation,
        "motivation": e.motivation,
        "empathy": e.empathy,
        "social_skills": e.social_skills,
        "growth_experiences": e.growth_experiences,
    }))
}

/// GET /api/eq/history — Historique EQ (via metriques).
pub async fn api_eq_history(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_eq_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API psychology: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

// ─── /api/flow/* — Etat de flow ─────────────────────────────────────────────
/// GET /api/flow/status — Etat de flow actuel.
pub async fn api_flow_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let f = &agent.psychology.flow;
    axum::Json(serde_json::json!({
        "in_flow": f.in_flow,
        "flow_intensity": f.flow_intensity,
        "perceived_challenge": f.perceived_challenge,
        "perceived_skill": f.perceived_skill,
        "duration_cycles": f.duration_cycles,
        "total_flow_cycles": f.total_flow_cycles,
    }))
}

/// GET /api/flow/history — Historique flow (via metriques).
pub async fn api_flow_history(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(ref logs_db) = state.logs_db {
        let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
        match logs_db.get_flow_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API psychology: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

// ─── /api/will/* — Module de volonte ─────────────────────────────────────────
/// GET /api/will/status — Etat du module de volonte (deliberation interne).
pub async fn api_will_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let w = &agent.psychology.will;
    axum::Json(serde_json::json!({
        "enabled": agent.config().will.enabled,
        "willpower": w.willpower,
        "decision_fatigue": w.decision_fatigue,
        "total_deliberations": w.total_deliberations,
        "proud_decisions": w.proud_decisions,
        "regretted_decisions": w.regretted_decisions,
        "recent_deliberations_count": w.recent_deliberations.len(),
        "last_deliberation": w.recent_deliberations.last().map(|d| {
            serde_json::json!({
                "trigger": format!("{:?}", d.trigger.trigger_type),
                "chosen": d.options[d.chosen].description,
                "confidence": d.confidence,
                "reasoning": d.reasoning,
            })
        }),
    }))
}

/// GET /api/will/last — Derniere deliberation complete.
pub async fn api_will_last(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let w = &agent.psychology.will;
    let data = w.recent_deliberations.last().map(|d| {
        serde_json::json!({
            "trigger": format!("{:?}", d.trigger.trigger_type),
            "urgency": d.trigger.urgency,
            "complexity": d.trigger.complexity,
            "stakes": d.trigger.stakes,
            "options": d.options.iter().map(|o| serde_json::json!({
                "description": o.description,
                "id_score": o.id_score,
                "superego_score": o.superego_score,
                "maslow_score": o.maslow_score,
                "toltec_score": o.toltec_score,
                "pragmatic_score": o.pragmatic_score,
                "weighted_score": o.weighted_score,
            })).collect::<Vec<_>>(),
            "chosen_index": d.chosen,
            "chosen": d.options[d.chosen].description,
            "confidence": d.confidence,
            "reasoning": d.reasoning,
            "chemistry_influence": {
                "boldness": d.chemistry_influence.boldness,
                "caution": d.chemistry_influence.caution,
                "wisdom": d.chemistry_influence.wisdom,
                "efficiency": d.chemistry_influence.efficiency,
                "urgency": d.chemistry_influence.urgency,
                "empathy": d.chemistry_influence.empathy,
            },
            "regret": d.regret,
            "created_at": d.created_at.to_rfc3339(),
        })
    });
    axum::Json(serde_json::json!({ "last_deliberation": data }))
}

/// GET /api/will/history — 20 dernieres deliberations.
pub async fn api_will_history(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let w = &agent.psychology.will;
    let history: Vec<_> = w.recent_deliberations.iter().rev().map(|d| {
        serde_json::json!({
            "trigger": format!("{:?}", d.trigger.trigger_type),
            "chosen": d.options[d.chosen].description,
            "confidence": d.confidence,
            "reasoning": d.reasoning,
            "regret": d.regret,
            "created_at": d.created_at.to_rfc3339(),
        })
    }).collect();
    axum::Json(serde_json::json!({ "deliberations": history, "total": w.total_deliberations }))
}

/// GET /api/will/stats — Statistiques de volonte.
pub async fn api_will_stats(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let w = &agent.psychology.will;
    // Influence chemical moyenne des deliberations recentes
    let avg_chem = if w.recent_deliberations.is_empty() {
        serde_json::json!({})
    } else {
        let n = w.recent_deliberations.len() as f64;
        let sum = w.recent_deliberations.iter().fold(
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            |acc, d| (
                acc.0 + d.chemistry_influence.boldness,
                acc.1 + d.chemistry_influence.caution,
                acc.2 + d.chemistry_influence.wisdom,
                acc.3 + d.chemistry_influence.efficiency,
                acc.4 + d.chemistry_influence.urgency,
                acc.5 + d.chemistry_influence.empathy,
            ),
        );
        serde_json::json!({
            "boldness": sum.0 / n,
            "caution": sum.1 / n,
            "wisdom": sum.2 / n,
            "efficiency": sum.3 / n,
            "urgency": sum.4 / n,
            "empathy": sum.5 / n,
        })
    };
    axum::Json(serde_json::json!({
        "total_deliberations": w.total_deliberations,
        "proud_decisions": w.proud_decisions,
        "regretted_decisions": w.regretted_decisions,
        "willpower": w.willpower,
        "decision_fatigue": w.decision_fatigue,
        "avg_chemistry_influence": avg_chem,
    }))
}

/// GET /api/monologue/current — Monologue interieur du cycle current.
pub async fn api_monologue_current(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    // Le monologue est the description du prompt de volonte
    let will_desc = if agent.config().will.enabled {
        agent.psychology.will.describe_for_prompt()
    } else {
        String::new()
    };
    // L'emotion from the chemistry current
    let emotion_state = crate::emotions::EmotionalState::compute(&agent.chemistry);
    let emotion_feeling = crate::psychology::ownership::describe_emotion_as_feeling(
        &emotion_state.dominant
    );
    // Contexte psychologique en vecu
    let psyche_desc = agent.psychology.describe_for_prompt();
    axum::Json(serde_json::json!({
        "emotion": emotion_feeling,
        "will_context": will_desc,
        "psyche_context": psyche_desc,
        "last_emotion": emotion_state.dominant,
    }))
}

/// GET /api/metrics/will — Metriques de volonte over time.
pub async fn api_metrics_will(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(200i64);
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.get_will_metrics(limit).await {
            Ok(data) => axum::Json(serde_json::json!({"data": data})),
            Err(e) => { tracing::error!("API psychology: {}", e); axum::Json(serde_json::json!({"error": "internal_error", "data": []})) },
        }
    } else {
        axum::Json(serde_json::json!({"data": []}))
    }
}

// ─── /api/model/* — Info model LLM ─────────────────────────────────────────
/// GET /api/model/info — Information sur le model LLM utilise.
pub async fn api_model_info(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let cfg = agent.config();
    axum::Json(serde_json::json!({
        "model": cfg.llm.model,
        "base_model": "mistral-nemo:12b",
        "identity_in_model": true,
        "base_url": cfg.llm.base_url,
        "max_tokens": cfg.llm.max_tokens,
        "temperature": cfg.llm.temperature,
        "num_ctx": cfg.llm.num_ctx,
    }))
}
