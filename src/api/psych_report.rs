// =============================================================================
// api/psych_report.rs — Neuropsychological report handlers
//
// Role: 4 endpoints to capture psychological snapshots and generate
// clinical reports via the LLM.
//
// Endpoints:
//  GET /api/psych/snapshot  — Take a snapshot, store it (max 5), return it
//  POST /api/psych/report   — Snapshot + LLM call → clinical report
//  GET /api/psych/snapshots — List stored snapshots (light summary)
//  GET /api/psych/reports   — List generated reports
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/psych/snapshot — Capture a complete psychological snapshot.
/// Stores it in the agent's VecDeque (max 5) and returns it as JSON.
pub async fn api_psych_snapshot(State(state): State<AppState>) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;

    if !agent.config().psych_report.enabled {
        return axum::Json(serde_json::json!({
            "error": "Module psych_report desactive"
        }));
    }

    let snapshot = agent.collect_psych_snapshot();

    // Store (max 5)
    if agent.psych_snapshots.len() >= 5 {
        agent.psych_snapshots.pop_front();
    }
    agent.psych_snapshots.push_back(snapshot.clone());

    axum::Json(serde_json::to_value(&snapshot).unwrap_or_default())
}

/// POST /api/psych/report — Generate a neuropsychological report via the LLM.
/// Takes a snapshot, builds prompts, calls the LLM, stores the report.
pub async fn api_psych_report(State(state): State<AppState>) -> impl IntoResponse {
    // Phase 1: collect data under the lock
    let (snapshot, system_prompt, user_prompt, llm_config, max_tokens, temperature, language) = {
        let mut agent = state.agent.lock().await;

        if !agent.config().psych_report.enabled {
            return axum::Json(serde_json::json!({
                "error": "Module psych_report desactive"
            }));
        }

        let snapshot = agent.collect_psych_snapshot();
        let system_prompt = agent.build_psych_report_system_prompt();
        let user_prompt = agent.build_psych_report_user_prompt(&snapshot);
        let llm_config = agent.config().llm.clone();
        let max_tokens = agent.config().psych_report.max_tokens;
        let temperature = agent.config().psych_report.temperature;
        let language = agent.config().general.language.clone();

        // Store the snapshot
        if agent.psych_snapshots.len() >= 5 {
            agent.psych_snapshots.pop_front();
        }
        agent.psych_snapshots.push_back(snapshot.clone());

        (snapshot, system_prompt, user_prompt, llm_config, max_tokens, temperature, language)
    };
    // Lock released here — the LLM can run 30-60s without blocking the agent
    // Phase 2: LLM call (blocking, in a dedicated thread)
    let backend = crate::llm::create_backend(&llm_config);
    let result = tokio::task::spawn_blocking(move || {
        backend.chat(&system_prompt, &user_prompt, temperature, max_tokens)
    }).await;

    let report_text = match result {
        Ok(Ok(text)) => text,
        Ok(Err(e)) => {
            return axum::Json(serde_json::json!({
                "error": format!("Erreur LLM : {}", e),
            }));
        }
        Err(e) => {
            return axum::Json(serde_json::json!({
                "error": format!("Erreur spawn : {}", e),
            }));
        }
    };

    // Phase 3: store the report under the lock
    let report = crate::agent::lifecycle::psych_report::PsychReport {
        timestamp: snapshot.timestamp.clone(),
        cycle: snapshot.cycle,
        report_text: report_text.clone(),
        language: language.clone(),
        token_count_approx: report_text.split_whitespace().count(),
    };

    {
        let mut agent = state.agent.lock().await;
        if agent.psych_reports.len() >= 5 {
            agent.psych_reports.pop_front();
        }
        agent.psych_reports.push_back(report.clone());
    }

    axum::Json(serde_json::to_value(&report).unwrap_or_default())
}

/// GET /api/psych/snapshots — List stored snapshots (light summary).
pub async fn api_psych_snapshots(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let summaries: Vec<_> = agent.psych_snapshots
        .iter()
        .map(|s| s.summary())
        .collect();
    axum::Json(serde_json::json!({
        "count": summaries.len(),
        "snapshots": summaries,
    }))
}

/// GET /api/psych/reports — List generated reports (light summary).
pub async fn api_psych_reports(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let summaries: Vec<_> = agent.psych_reports
        .iter()
        .map(|r| r.summary())
        .collect();
    axum::Json(serde_json::json!({
        "count": summaries.len(),
        "reports": summaries,
    }))
}
