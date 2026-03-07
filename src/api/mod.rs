// =============================================================================
// api/mod.rs — API module: router, shared state, handlers, security
//
// This is the top-level module for the Saphire-Lite HTTP/WebSocket API.
// It declares all sub-modules, re-exports key types, and builds the Axum
// router with both public and protected routes.
//
// Lite version: only essential endpoints are exposed (chemistry, emotions,
// memory, body, vital signs, ethics, consciousness, factory defaults,
// logs, and WebSocket).
// =============================================================================

pub mod state;
pub mod middleware;
pub mod static_files;
pub mod system;
pub mod chemistry;
pub mod memory;
pub mod brain;
pub mod body;
pub mod vital;
pub mod ethics;
pub mod metrics;
pub mod logs;
pub mod factory;
pub mod websocket;

// Re-exports for direct access from main.rs
pub use state::{AppState, ControlMessage};

use axum::{Router, routing::{get, post}};

/// Builds the main Axum router with all essential routes.
///
/// # Arguments
/// * `state` - The shared application state, cloned into every handler.
///
/// # Returns
/// A fully configured `Router` with:
/// - Public routes (static files, health check, WebSocket endpoints)
/// - Protected API routes (guarded by auth and rate-limit middleware)
/// - CORS layer built from the allowed origins in `state`
pub fn build_router(state: AppState) -> Router {
    let cors = middleware::build_cors_layer(&state.allowed_origins);

    let protected_api = Router::new()
        .route("/api/config", get(system::api_get_config).post(system::api_post_config))
        .route("/api/chemistry", get(chemistry::api_get_chemistry))
        .route("/api/stabilize", post(system::api_stabilize))
        .route("/api/world", get(brain::api_get_world))
        .route("/api/memory", get(memory::api_get_memory))
        // ─── Logs ─────────────────────────────────────────────────────
        .route("/api/logs", get(logs::api_get_logs))
        .route("/api/logs/export", get(logs::api_export_logs))
        .route("/api/logs/:id", get(logs::api_get_log_by_id))
        // ─── Memory ──────────────────────────────────────────────────
        .route("/api/memory/working", get(memory::api_get_working_memory))
        .route("/api/memory/episodic", get(memory::api_list_episodic))
        .route("/api/memory/episodic/:id", get(memory::api_get_episodic_by_id))
        .route("/api/memory/ltm", get(memory::api_list_ltm))
        .route("/api/memory/ltm/:id", get(memory::api_get_ltm_by_id))
        .route("/api/memory/founding", get(memory::api_list_founding))
        .route("/api/memory/stats", get(memory::api_memory_stats))
        .route("/api/memory/archives", get(memory::api_list_archives))
        .route("/api/memory/archives/stats", get(memory::api_archive_stats))
        // ─── Cognitive traces ────────────────────────────────────────
        .route("/api/trace/:cycle", get(logs::api_get_trace))
        .route("/api/traces", get(logs::api_list_traces))
        // ─── Metrics ─────────────────────────────────────────────────
        .route("/api/metrics/chemistry", get(metrics::api_metrics_chemistry))
        .route("/api/metrics/emotions", get(metrics::api_metrics_emotions))
        .route("/api/metrics/decisions", get(metrics::api_metrics_decisions))
        .route("/api/metrics/satisfaction", get(metrics::api_metrics_satisfaction))
        .route("/api/metrics/llm", get(metrics::api_metrics_llm))
        .route("/api/metrics/thought_types", get(metrics::api_metrics_thought_types))
        // ─── LLM History ───
        .route("/api/llm/history", get(logs::api_llm_history))
        .route("/api/llm/history/:id", get(logs::api_llm_history_by_id))
        // ─── Body & Heart ────────────────────────────────────────────
        .route("/api/body/status", get(body::api_body_status))
        .route("/api/body/heart", get(body::api_body_heart))
        .route("/api/body/heart/history", get(body::api_body_heart_history))
        .route("/api/body/history", get(body::api_body_history))
        .route("/api/body/vitals", get(body::api_body_vitals))
        .route("/api/body/milestones", get(body::api_body_milestones))
        .route("/api/metrics/heart", get(metrics::api_metrics_heart))
        .route("/api/metrics/body", get(metrics::api_metrics_body))
        // ─── System ──────────────────────────────────────────────────
        .route("/api/system/status", get(system::api_system_status))
        .route("/api/identity", get(system::api_identity))
        .route("/api/system/db/tables", get(system::api_db_tables))
        .route("/api/system/backup", post(system::api_backup))
        .route("/api/system/consolidate", post(system::api_consolidate))
        .route("/api/system/purge_logs", post(system::api_purge_logs))
        // ─── Vital / Intuition / Premonition ────────────────────────
        .route("/api/vital/status", get(vital::api_vital_status))
        .route("/api/vital/threats", get(vital::api_vital_threats))
        .route("/api/intuition/status", get(vital::api_intuition_status))
        .route("/api/intuition/history", get(vital::api_intuition_history))
        .route("/api/premonition/active", get(vital::api_premonition_active))
        .route("/api/premonition/history", get(vital::api_premonition_history))
        // ─── Ethics ──────────────────────────────────────────────────
        .route("/api/ethics/layers", get(ethics::api_ethics_layers))
        .route("/api/ethics/personal", get(ethics::api_ethics_personal))
        .route("/api/ethics/personal/:id", get(ethics::api_ethics_personal_by_id))
        .route("/api/ethics/readiness", get(ethics::api_ethics_readiness))
        // ─── Extended metrics ───────────────────────────────────────
        .route("/api/metrics/vital", get(metrics::api_metrics_vital))
        .route("/api/metrics/intuition", get(metrics::api_metrics_intuition))
        .route("/api/metrics/premonition", get(metrics::api_metrics_premonition))
        .route("/api/metrics/ethics", get(metrics::api_metrics_ethics))
        // ─── Factory defaults ───────────────────────────────────────
        .route("/api/factory/defaults", get(factory::api_factory_defaults))
        .route("/api/factory/diff", get(factory::api_factory_diff))
        .route("/api/factory/reset", post(factory::api_factory_reset))
        // ─── Security middleware layers ─────────────────────────────
        // Auth middleware checks the Bearer token; rate limiter caps requests per IP.
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(), middleware::auth_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(), middleware::rate_limit_middleware,
        ));

    Router::new()
        // Public routes (no authentication required)
        .route("/", get(static_files::index_handler))
        .route("/style.css", get(static_files::css_handler))
        .route("/app.js", get(static_files::js_handler))
        .route("/favicon.svg", get(static_files::favicon_handler))
        .route("/i18n.js", get(static_files::i18n_js_handler))
        .route("/i18n/:lang", get(static_files::i18n_handler))
        // Health check
        .route("/api/health", get(system::health_handler))
        // WebSocket
        .route("/ws", get(websocket::ws_handler))
        .route("/ws/dashboard", get(websocket::ws_dashboard_handler))
        // Merge protected API routes into the public router
        .merge(protected_api)
        .layer(cors)
        .with_state(state)
}
