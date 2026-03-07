// =============================================================================
// api/system.rs — System, health, and configuration handlers
//
// This module provides HTTP endpoints for:
// - Health checks (Docker probes, availability monitoring).
// - Agent configuration retrieval and partial updates.
// - System status reporting (version, cycle count, DB connectivity).
// - Database table statistics.
// - Backup, memory consolidation, and log purging.
// - Emergency neurochemical stabilization.
// - Stub endpoints for features not ported to the lite version.
// - Agent identity information.
// - LoRA fine-tuning sample management (stats and export).
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/health -- Health check endpoint.
///
/// Returns a simple JSON status indicating the agent is alive along with the
/// version string. Useful for Docker health checks and availability probes.
pub async fn health_handler() -> impl IntoResponse {
    axum::Json(serde_json::json!({ "status": "alive", "version": "1.0.0" }))
}

/// GET /api/config -- Returns the agent's current configuration as JSON.
pub async fn api_get_config(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(agent.config_json())
}

/// POST /api/config -- Partially updates the agent's configuration.
///
/// The request body is a JSON object containing only the fields to modify.
/// Only the fields present in the body are updated (partial merge).
/// Supported fields include baseline neurotransmitter values, homeostasis rate,
/// temperature, and thought interval.
pub async fn api_post_config(
    State(state): State<AppState>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;

    // Partial merge: each field present in the JSON body is applied individually
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

/// GET /api/system/status -- Returns system status information (lite version).
///
/// Includes: alive status, version, current cycle count, and database connectivity.
pub async fn api_system_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(serde_json::json!({
        "status": "alive",
        "version": "1.0.0-lite",
        "cycle": agent.cycle_count,
        "db_connected": agent.db.is_some(),
        "logs_db_connected": state.logs_db.is_some(),
    }))
}

/// GET /api/system/db/tables -- Returns row counts and size statistics for all DB tables.
///
/// Queries both the main agent database and the logs database (if available)
/// and returns their table statistics under `main_db` and `logs_db` keys.
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

/// POST /api/system/backup -- Triggers a backup of logs and agent state (lite version).
///
/// Exports up to 100,000 log entries and captures the current agent cycle count.
/// Returns the log count and agent state snapshot.
pub async fn api_backup(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;

    let agent_state = serde_json::json!({
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

/// POST /api/system/consolidate -- Triggers a memory consolidation cycle.
///
/// Memory consolidation processes episodic memories and promotes significant
/// ones to long-term memory based on emotional salience and repetition.
pub async fn api_consolidate(State(state): State<AppState>) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    let result = agent.run_consolidation().await;
    axum::Json(result)
}

/// POST /api/system/purge_logs -- Purges old log entries.
///
/// # Request body
/// JSON object with an optional `"days"` field (default 30) specifying the
/// retention period. Logs older than this many days are permanently deleted.
///
/// # Returns
/// JSON `{"status": "ok", "deleted": N}` on success, or `{"error": ...}` on failure.
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

/// POST /api/stabilize -- Triggers emergency neurochemical stabilization.
///
/// Immediately resets all neurotransmitter levels to their configured baseline values.
/// This is a safety mechanism for when the agent's chemistry becomes dangerously imbalanced.
pub async fn api_stabilize(State(state): State<AppState>) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;
    agent.emergency_stabilize();
    axum::Json(serde_json::json!({ "status": "stabilized" }))
}

/// GET /api/hardware -- Not available in the lite version (stub endpoint).
pub async fn api_hardware(State(_state): State<AppState>) -> impl IntoResponse {
    axum::Json(serde_json::json!({"status": "not_available", "note": "hardware profiling not ported in lite"}))
}

/// GET /api/genome -- Not available in the lite version (stub endpoint).
pub async fn api_genome(State(_state): State<AppState>) -> impl IntoResponse {
    axum::Json(serde_json::json!({"status": "not_available", "note": "genome not ported in lite"}))
}

/// GET /api/connectome -- Not available in the lite version (stub endpoint).
pub async fn api_connectome(State(_state): State<AppState>) -> impl IntoResponse {
    axum::Json(serde_json::json!({"status": "not_available", "note": "connectome not ported in lite"}))
}

/// GET /api/connectome/metrics -- Not available in the lite version (stub endpoint).
pub async fn api_connectome_metrics(State(_state): State<AppState>) -> impl IntoResponse {
    axum::Json(serde_json::json!({"status": "not_available", "note": "connectome not ported in lite"}))
}

/// GET /api/metacognition -- Not available in the lite version (stub endpoint).
pub async fn api_metacognition(State(_state): State<AppState>) -> impl IntoResponse {
    axum::Json(serde_json::json!({"status": "not_available", "note": "metacognition not ported in lite"}))
}

/// GET /api/turing -- Not available in the lite version (stub endpoint).
pub async fn api_turing(State(_state): State<AppState>) -> impl IntoResponse {
    axum::Json(serde_json::json!({"status": "not_available", "note": "turing metric not ported in lite"}))
}

/// GET /api/lora/stats -- Returns LoRA fine-tuning sample collection statistics.
///
/// Reports whether collection is enabled, total sample count, average quality
/// score, maximum sample limit, and minimum quality threshold.
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

/// GET /api/lora/export -- Exports the best LoRA fine-tuning samples as JSON.
///
/// # Query parameters
/// * `min_quality` (optional, default 0.0): minimum quality score threshold.
/// * `limit` (optional, default 1000): maximum number of samples to export.
///
/// # Returns
/// JSON with format "jsonl", count, and an array of samples. Each sample
/// contains a `messages` array (system/user/assistant) and metadata
/// (thought type, quality, reward, human feedback, emotion, consciousness level).
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
                // JSONL format: each entry is an independent JSON object
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

/// GET /api/identity -- Returns the complete identity of Saphire (name, appearance, stats).
///
/// Includes the agent's name, birth date, boot/cycle counts, conversation stats,
/// dominant emotion and tendency, self-description, core values, and full
/// physical appearance details (eye color, hair, skin, height, build, age, etc.).
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
