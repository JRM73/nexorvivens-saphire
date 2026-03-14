// =============================================================================
// api/knowledge.rs — Handlers sources et statistiques de connaissances
//
// Role: Endpoints for the sources de connaissance configurees
// et les statistiques detaillees d'acquisition.
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/knowledge/sources — Liste des sources de connaissance configurees.
pub async fn api_knowledge_sources(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let sources = &agent.knowledge.config.sources;
    axum::Json(serde_json::json!({
        "sources": {
            "wikipedia": sources.wikipedia,
            "arxiv": sources.arxiv,
            "medium": sources.medium,
            "sep": sources.sep,
            "gutenberg": sources.gutenberg,
            "semantic_scholar": sources.semantic_scholar,
            "openlibrary": sources.openlibrary,
            "philosophy_rss": sources.philosophy_rss,
        },
        "recent_sources": agent.knowledge.recent_sources,
    }))
}

/// GET /api/knowledge/stats — Statistiques detaillees des connaissances.
pub async fn api_knowledge_stats(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(serde_json::json!({
        "total_explored": agent.knowledge.topics_explored.len(),
        "total_searches": agent.knowledge.total_searches,
        "recent_topics": agent.knowledge.topics_explored.iter()
            .rev().take(10).collect::<Vec<_>>(),
        "suggested_pending": agent.knowledge.suggested_topics.len(),
        "recent_sources": agent.knowledge.recent_sources,
        "cache_size": agent.knowledge.config.cache_size,
        "cooldown_cycles": agent.knowledge.config.search_cooldown_cycles,
        "cycles_since_last": agent.knowledge.cycles_since_last_search,
    }))
}
