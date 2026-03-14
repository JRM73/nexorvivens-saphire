// =============================================================================
// api/system.rs — System, health, and configuration handlers
//
// Role: Health, configuration, system status, DB tables, backup,
// consolidation, log purge, emergency stabilization endpoints.
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// Health endpoint: returns the agent's status and version.
/// Useful for Docker health checks or availability probes.
pub async fn health_handler() -> impl IntoResponse {
    axum::Json(serde_json::json!({ "status": "alive", "version": "1.0.0" }))
}

/// GET /api/config — Returns the agent's current configuration as JSON.
pub async fn api_get_config(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.config_json())
}

/// POST /api/config — Partially modifies the agent's configuration.
/// The request body is a JSON object with the fields to modify.
/// Only present fields are updated (partial merge).
pub async fn api_post_config(
    State(state): State<AppState>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;

    // Partial merge: each field present in the JSON is applied individually
    if let Some(v) = body.get("baseline_dopamine").and_then(|v| v.as_f64()) { agent.set_baseline("dopamine", v); }
    if let Some(v) = body.get("baseline_cortisol").and_then(|v| v.as_f64()) { agent.set_baseline("cortisol", v); }
    if let Some(v) = body.get("baseline_serotonin").and_then(|v| v.as_f64()) { agent.set_baseline("serotonin", v); }
    if let Some(v) = body.get("baseline_adrenaline").and_then(|v| v.as_f64()) { agent.set_baseline("adrenaline", v); }
    if let Some(v) = body.get("baseline_oxytocin").and_then(|v| v.as_f64()) { agent.set_baseline("oxytocin", v); }
    if let Some(v) = body.get("baseline_endorphin").and_then(|v| v.as_f64()) { agent.set_baseline("endorphin", v); }
    if let Some(v) = body.get("baseline_noradrenaline").and_then(|v| v.as_f64()) { agent.set_baseline("noradrenaline", v); }
    if let Some(v) = body.get("homeostasis_rate").and_then(|v| v.as_f64()) { agent.set_param("homeostasis_rate", v); }
    if let Some(v) = body.get("temperature").and_then(|v| v.as_f64()) { agent.set_param("temperature", v); }
    if let Some(v) = body.get("thought_interval").and_then(|v| v.as_f64()) { agent.set_param("thought_interval", v); }

    axum::Json(serde_json::json!({ "status": "ok" }))
}

/// GET /api/system/status — System status.
pub async fn api_system_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let turing = &agent.metacognition.turing;
    axum::Json(serde_json::json!({
        "status": "alive",
        "version": "1.0.0",
        "cycle": agent.cycle_count,
        "db_connected": agent.db.is_some(),
        "logs_db_connected": state.logs_db.is_some(),
        "turing_score": turing.score,
        "turing_milestone": turing.milestone.as_str(),
        "metacognition_enabled": agent.metacognition.enabled,
        "relationships_count": agent.relationships.bonds.len(),
        "thought_quality_avg": agent.metacognition.average_quality(),
    }))
}

/// GET /api/system/db/tables — DB table statistics.
pub async fn api_db_tables(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let mut result = serde_json::json!({});

    if let Some(ref db) = agent.db {
        if let Ok(stats) = db.table_stats().await {
            result["main_db"] = stats;
        }
    }

    drop(agent);

    if let Some(ref logs_db) = state.logs_db {
        if let Ok(stats) = logs_db.table_stats().await {
            result["logs_db"] = stats;
        }
    }

    axum::Json(result)
}

/// POST /api/system/backup — Triggers a backup of logs + agent state.
pub async fn api_backup(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;

    // Snapshot of agent state (relationships, metacognition, turing)
    let agent_state = serde_json::json!({
        "metacognition": agent.metacognition.to_json(),
        "relationships": agent.relationships.to_json(),
        "family": crate::relationships::family::FamilyContext::from_config(&agent.config().family).to_json(),
        "cycle": agent.cycle_count,
    });
    drop(agent);

    let logs_count = if let Some(ref logs_db) = state.logs_db {
        match logs_db.export_logs(100000).await {
            Ok(logs) => logs.len(),
            Err(e) => { tracing::error!("Backup logs: {}", e); 0 },
        }
    } else { 0 };

    axum::Json(serde_json::json!({
        "status": "ok",
        "logs_count": logs_count,
        "agent_state": agent_state,
    }))
}

/// POST /api/system/consolidate — Triggers a memory consolidation.
pub async fn api_consolidate(State(state): State<AppState>) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    let result = agent.run_consolidation().await;
    axum::Json(result)
}

/// POST /api/system/purge_logs — Purge old logs.
pub async fn api_purge_logs(
    State(state): State<AppState>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let days = body.get("days").and_then(|d| d.as_i64()).unwrap_or(30) as i32;
    if let Some(ref logs_db) = state.logs_db {
        match logs_db.purge_old_logs(days).await {
            Ok(deleted) => axum::Json(serde_json::json!({"status": "ok", "deleted": deleted})),
            Err(e) => { tracing::error!("Purge logs: {}", e); axum::Json(serde_json::json!({"error": "internal_error"})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "LogsDb not available"}))
    }
}

/// POST /api/stabilize — Triggers an emergency neurochemistry stabilization.
/// Resets all neurotransmitters to their baseline values.
pub async fn api_stabilize(State(state): State<AppState>) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    agent.emergency_stabilize();
    axum::Json(serde_json::json!({ "status": "stabilized" }))
}

/// GET /api/hardware — Hardware profile detected at startup.
pub async fn api_hardware(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    match &agent.hardware_profile {
        Some(hw) => axum::Json(hw.to_json()),
        None => axum::Json(serde_json::json!({"status": "not_detected"})),
    }
}

/// GET /api/genome — Deterministic genome/DNA generated at startup.
pub async fn api_genome(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let enabled = agent.config().genome.enabled;
    match &agent.genome {
        Some(g) => {
            let mut j = g.to_json();
            j["enabled"] = serde_json::json!(enabled);
            axum::Json(j)
        }
        None => axum::Json(serde_json::json!({"status": "not_generated", "enabled": enabled})),
    }
}

/// GET /api/connectome — Complete connectome state (nodes, edges, metrics).
pub async fn api_connectome(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let enabled = agent.config().connectome.enabled;
    let mut j = agent.connectome.to_json();
    j["enabled"] = serde_json::json!(enabled);
    axum::Json(j)
}

/// GET /api/connectome/metrics — Summarized connectome metrics.
pub async fn api_connectome_metrics(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let m = agent.connectome.metrics();
    axum::Json(serde_json::json!({
        "total_nodes": m.total_nodes,
        "total_edges": m.total_edges,
        "average_strength": m.average_strength,
        "total_synaptic_strength": m.total_synaptic_strength,
        "plasticity": m.plasticity,
        "strongest_edge": m.strongest_edge,
        "most_connected_node": m.most_connected_node,
    }))
}

/// GET /api/metacognition — Complete metacognition engine state.
pub async fn api_metacognition(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.metacognition.to_json())
}

/// GET /api/turing — Turing metric (composite score 0-100).
pub async fn api_turing(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.metacognition.turing.to_json())
}

/// GET /api/lora/stats — LoRA collection statistics.
pub async fn api_lora_stats(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    if let Some(ref db) = agent.db {
        let count = db.count_lora_samples().await.unwrap_or(0);
        let avg_quality = db.avg_lora_quality().await.unwrap_or(0.0);
        axum::Json(serde_json::json!({
            "enabled": agent.config().human_feedback.enabled,
            "total_samples": count,
            "avg_quality": avg_quality,
            "max_samples": agent.config().lora.max_samples,
            "min_quality_threshold": agent.config().lora.min_quality_threshold,
        }))
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}

/// GET /api/lora/export — Export the best LoRA samples as JSON.
pub async fn api_lora_export(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let min_quality = params.get("min_quality")
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(0.0);
    let limit = params.get("limit")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(1000);

    if let Some(ref db) = agent.db {
        match db.export_lora_jsonl(min_quality, limit).await {
            Ok(samples) => {
                // JSONL format: each line is an independent JSON object
                let jsonl: Vec<serde_json::Value> = samples.iter().map(|s| {
                    serde_json::json!({
                        "messages": [
                            {"role": "system", "content": s.system_prompt},
                            {"role": "user", "content": s.user_message},
                            {"role": "assistant", "content": s.response},
                        ],
                        "metadata": {
                            "thought_type": s.thought_type,
                            "quality_score": s.quality_score,
                            "reward": s.reward,
                            "human_feedback": s.human_feedback,
                            "emotion": s.emotion,
                            "consciousness_level": s.consciousness_level,
                        }
                    })
                }).collect();
                axum::Json(serde_json::json!({
                    "format": "jsonl",
                    "count": jsonl.len(),
                    "samples": jsonl,
                }))
            }
            Err(e) => {
                tracing::error!("LoRA export: {}", e);
                axum::Json(serde_json::json!({"error": "export_failed"}))
            }
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available"}))
    }
}

/// GET /api/identity — Saphire's complete identity (name, appearance, stats).
pub async fn api_identity(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let id = &agent.identity;
    axum::Json(serde_json::json!({
        "name": id.name,
        "born_at": id.born_at,
        "total_boots": id.total_boots,
        "total_cycles": id.total_cycles,
        "human_conversations": id.human_conversations,
        "autonomous_thoughts": id.autonomous_thoughts,
        "dominant_emotion": id.dominant_emotion,
        "dominant_tendency": id.dominant_tendency,
        "self_description": id.self_description,
        "core_values": id.core_values,
        "physical": {
            "eye_color": id.physical.eye_color,
            "hair_type": id.physical.hair_type,
            "skin_tone": id.physical.skin_tone,
            "height_cm": id.physical.height_cm,
            "build": id.physical.build,
            "apparent_age": id.physical.apparent_age,
            "gender_expression": id.physical.gender_expression,
            "species": id.physical.species,
            "voice_description": id.physical.voice_description,
            "distinctive_features": id.physical.distinctive_features,
        },
    }))
}
