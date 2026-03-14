// =============================================================================
// api/psych_report.rs — Handlers du rapport neuropsychologique
//
// Role : 4 endpoints pour capturer des snapshots psychologiques et generer
// des rapports cliniques via le LLM.
//
// Endpoints :
//   GET  /api/psych/snapshot   — Prend un snapshot, le stocke (max 5), le retourne
//   POST /api/psych/report     — Snapshot + appel LLM → rapport clinique
//   GET  /api/psych/snapshots  — Liste les snapshots (resume leger)
//   GET  /api/psych/reports    — Liste les rapports generes
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/psych/snapshot — Capture un snapshot psychologique complet.
/// Le stocke dans le VecDeque de l'agent (max 5) et le retourne en JSON.
pub async fn api_psych_snapshot(State(state): State<AppState>) -> impl IntoResponse {
    let mut agent = state.agent.lock().await;

    if !agent.config().psych_report.enabled {
        return axum::Json(serde_json::json!({
            "error": "Module psych_report desactive"
        }));
    }

    let snapshot = agent.collect_psych_snapshot();

    // Stocker (max 5)
    if agent.psych_snapshots.len() >= 5 {
        agent.psych_snapshots.pop_front();
    }
    agent.psych_snapshots.push_back(snapshot.clone());

    axum::Json(serde_json::to_value(&snapshot).unwrap_or_default())
}

/// POST /api/psych/report — Genere un rapport neuropsychologique via le LLM.
/// Prend un snapshot, construit les prompts, appelle le LLM, stocke le rapport.
pub async fn api_psych_report(State(state): State<AppState>) -> impl IntoResponse {
    // Phase 1 : collecter les donnees sous le lock
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

        // Stocker le snapshot
        if agent.psych_snapshots.len() >= 5 {
            agent.psych_snapshots.pop_front();
        }
        agent.psych_snapshots.push_back(snapshot.clone());

        (snapshot, system_prompt, user_prompt, llm_config, max_tokens, temperature, language)
    };
    // Lock relache ici — le LLM peut tourner 30-60s sans bloquer l'agent

    // Phase 2 : appel LLM (bloquant, dans un thread dedie)
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

    // Phase 3 : stocker le rapport sous le lock
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

/// GET /api/psych/snapshots — Liste les snapshots stockes (resume leger).
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

/// GET /api/psych/reports — Liste les rapports generes (resume leger).
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
