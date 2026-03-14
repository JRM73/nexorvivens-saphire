// =============================================================================
// api/nn_learnings.rs — Endpoints for the learnings vector
//
// Role: Fournit 2 endpoints REST for consulter les nn_learnings
// from the dashboard (sous-onglet memoire + panneau systeme).
// =============================================================================

use std::collections::HashMap;
use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/nn-learnings/recent?limit=20 — Apprentissages recents.
pub async fn api_nn_learnings_recent(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20);
    if let Some(ref db) = agent.db {
        match db.load_recent_learnings(limit).await {
            Ok(learnings) => {
                let items: Vec<serde_json::Value> = learnings.iter().map(|l| {
                    serde_json::json!({
                        "id": l.id,
                        "domain": l.domain,
                        "scope": l.scope,
                        "summary": l.summary,
                        "keywords": l.keywords,
                        "confidence": l.confidence,
                        "satisfaction": l.satisfaction,
                        "emotion": l.emotion,
                        "strength": l.strength,
                        "access_count": l.access_count,
                        "created_at": l.created_at.to_rfc3339(),
                    })
                }).collect();
                axum::Json(serde_json::json!({"learnings": items, "total": items.len()}))
            }
            Err(e) => { tracing::error!("API nn_learnings: {}", e); axum::Json(serde_json::json!({"error": "internal_error", "learnings": []})) },
        }
    } else {
        axum::Json(serde_json::json!({"error": "DB not available", "learnings": []}))
    }
}

/// GET /api/nn-learnings/stats — Statistiques des learnings vector.
pub async fn api_nn_learnings_stats(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let nn_cfg = &agent.config().plugins.micro_nn;
    let cooldown = nn_cfg.learning_cooldown_cycles;
    let cycles_since = agent.cycles_since_last_nn_learning;

    if let Some(ref db) = agent.db {
        let count = db.count_learnings().await.unwrap_or(0);
        // Domaines uniques via les learnings recents
        let recent = db.load_recent_learnings(100).await.unwrap_or_default();
        let mut domains: Vec<String> = recent.iter().map(|l| l.domain.clone()).collect();
        domains.sort();
        domains.dedup();
        let recent_5: Vec<serde_json::Value> = recent.iter().take(5).map(|l| {
            serde_json::json!({
                "id": l.id, "domain": l.domain, "summary": l.summary,
                "confidence": l.confidence, "created_at": l.created_at.to_rfc3339(),
            })
        }).collect();

        axum::Json(serde_json::json!({
            "total": count,
            "unique_domains": domains.len(),
            "domains": domains,
            "cooldown_cycles": cooldown,
            "cycles_since_last": cycles_since,
            "enabled": nn_cfg.learning_enabled,
            "max_learnings": nn_cfg.max_learnings,
            "recent": recent_5,
        }))
    } else {
        axum::Json(serde_json::json!({
            "total": 0, "unique_domains": 0, "domains": [],
            "cooldown_cycles": cooldown, "cycles_since_last": cycles_since,
            "enabled": nn_cfg.learning_enabled, "max_learnings": nn_cfg.max_learnings,
            "recent": [],
        }))
    }
}
