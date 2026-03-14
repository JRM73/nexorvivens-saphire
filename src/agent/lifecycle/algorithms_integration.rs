// =============================================================================
// lifecycle/algorithms_integration.rs — Algorithm orchestrator
// =============================================================================

use crate::logging::{LogLevel, LogCategory};

use super::SaphireAgent;

impl SaphireAgent {
    /// Executes automatic algorithmic analyses according to the configured intervals.
    /// Each analysis runs at its own pace: smoothing every 20 cycles,
    /// clustering every 100 cycles, etc.
    pub(super) async fn run_auto_algorithms(&mut self) {
        use crate::algorithms::orchestrator::AlgorithmInput;

        let cycle = self.cycle_count;

        // Every 20 cycles: exponential smoothing on the chemistry
        if self.orchestrator.smoothing_interval > 0
            && cycle.is_multiple_of(self.orchestrator.smoothing_interval)
            && self.chemistry_history.len() >= 10
        {
            // Use dopamine as a representative time series
            let series: Vec<f64> = self.chemistry_history.iter().map(|h| h[0]).collect();
            let input = AlgorithmInput {
                time_series: Some(series),
                params: [("alpha".into(), 0.3)].into(),
                ..Default::default()
            };
            if let Err(e) = self.orchestrator.execute_auto("exponential_smoothing", input) {
                tracing::debug!("Algo auto exponential_smoothing: {}", e);
            }
        }

        // Every 50 cycles: thought → emotion association rules
        if self.orchestrator.association_interval > 0
            && cycle.is_multiple_of(self.orchestrator.association_interval)
        {
            if let Some(ref db) = self.db {
                if let Ok(recent) = db.recent_episodic(200).await {
                    if recent.len() >= 20 {
                        let texts: Vec<String> = recent.iter()
                            .map(|r| format!("{}>{}", r.source_type, r.emotion))
                            .collect();
                        let input = AlgorithmInput {
                            texts: Some(texts),
                            ..Default::default()
                        };
                        if let Err(e) = self.orchestrator.execute_auto("association_rules", input) {
                            tracing::debug!("Algo auto association_rules: {}", e);
                        }
                    }
                }
            }
        }

        // Every 100 cycles: memory clustering + anomaly detection
        if self.orchestrator.clustering_interval > 0
            && cycle.is_multiple_of(self.orchestrator.clustering_interval)
        {
            // K-Means on chemical history (7 dimensions = 7 molecules)
            if self.chemistry_history.len() >= 30 {
                let vectors: Vec<Vec<f64>> = self.chemistry_history.iter()
                    .map(|h| h.to_vec())
                    .collect();
                let input = AlgorithmInput {
                    vectors: Some(vectors),
                    params: [("k".into(), 4.0)].into(),
                    ..Default::default()
                };
                if let Err(e) = self.orchestrator.execute_auto("kmeans", input) {
                    tracing::debug!("Algo auto kmeans: {}", e);
                }
            }

            // Anomaly detection on chemical history
            if self.orchestrator.anomaly_interval > 0
                && self.chemistry_history.len() >= 20
            {
                let vectors: Vec<Vec<f64>> = self.chemistry_history.iter()
                    .map(|h| h.to_vec())
                    .collect();
                let input = AlgorithmInput {
                    vectors: Some(vectors),
                    params: [("threshold".into(), 2.5)].into(),
                    ..Default::default()
                };
                match self.orchestrator.execute_auto("isolation_forest", input) {
                    Ok(ref output) if output.has_critical() => {
                        self.log(LogLevel::Warn, LogCategory::Thought,
                            format!("Anomalie chimique detectee par algorithme: {}",
                                output.natural_language_result.chars().take(120).collect::<String>()),
                            serde_json::json!({"algorithm": "isolation_forest"}),
                        );
                    }
                    Err(e) => {
                        tracing::debug!("Algo auto isolation_forest: {}", e);
                    }
                    _ => {}
                }
            }
        }

        // Every 200 cycles: change point detection
        if self.orchestrator.changepoint_interval > 0
            && cycle.is_multiple_of(self.orchestrator.changepoint_interval)
            && self.chemistry_history.len() >= 20
        {
            let series: Vec<f64> = self.chemistry_history.iter().map(|h| h[0]).collect();
            let input = AlgorithmInput {
                time_series: Some(series),
                ..Default::default()
            };
            if let Err(e) = self.orchestrator.execute_auto("changepoint_detection", input) {
                tracing::debug!("Algo auto changepoint_detection: {}", e);
            }
        }
    }

    /// Processes an algorithm request from the LLM (on-demand mode).
    pub(super) async fn handle_algorithm_request(
        &mut self,
        request: &crate::algorithms::orchestrator::AlgorithmRequest,
    ) {
        use crate::algorithms::orchestrator::AlgorithmInput;

        // Prepare the data according to the requested algorithm
        let input = match request.algorithm_id.as_str() {
            "kmeans" => {
                if self.chemistry_history.len() >= 20 {
                    Some(AlgorithmInput {
                        vectors: Some(self.chemistry_history.iter().map(|h| h.to_vec()).collect()),
                        params: [("k".into(), 5.0)].into(),
                        ..Default::default()
                    })
                } else { None }
            }
            "isolation_forest" => {
                if self.chemistry_history.len() >= 10 {
                    Some(AlgorithmInput {
                        vectors: Some(self.chemistry_history.iter().map(|h| h.to_vec()).collect()),
                        ..Default::default()
                    })
                } else { None }
            }
            "exponential_smoothing" => {
                if self.chemistry_history.len() >= 5 {
                    Some(AlgorithmInput {
                        time_series: Some(self.chemistry_history.iter().map(|h| h[0]).collect()),
                        params: [("alpha".into(), 0.3)].into(),
                        ..Default::default()
                    })
                } else { None }
            }
            "pca" => {
                if self.chemistry_history.len() >= 20 {
                    Some(AlgorithmInput {
                        vectors: Some(self.chemistry_history.iter().map(|h| h.to_vec()).collect()),
                        params: [("n_components".into(), 3.0)].into(),
                        ..Default::default()
                    })
                } else { None }
            }
            "changepoint_detection" => {
                if self.chemistry_history.len() >= 15 {
                    Some(AlgorithmInput {
                        time_series: Some(self.chemistry_history.iter().map(|h| h[0]).collect()),
                        ..Default::default()
                    })
                } else { None }
            }
            "association_rules" => {
                if let Some(ref db) = self.db {
                    if let Ok(recent) = db.recent_episodic(200).await {
                        let texts: Vec<String> = recent.iter()
                            .map(|r| format!("{}>{}", r.source_type, r.emotion))
                            .collect();
                        if texts.len() >= 10 {
                            Some(AlgorithmInput { texts: Some(texts), ..Default::default() })
                        } else { None }
                    } else { None }
                } else { None }
            }
            _ => None, // Algorithm not handled in on-demand mode
        };

        if let Some(input) = input {
            match self.orchestrator.execute(&request.algorithm_id, input) {
                Ok(output) => {
                    tracing::info!("Algorithme demande par LLM: {} — {}ms",
                        request.algorithm_id, output.execution_ms);
                    self.log(LogLevel::Info, LogCategory::Thought,
                        format!("Algorithme utilise: {} — {}",
                            request.algorithm_id,
                            output.natural_language_result.chars().take(100).collect::<String>()),
                        serde_json::json!({
                            "algorithm": request.algorithm_id,
                            "execution_ms": output.execution_ms,
                            "metrics": output.metrics,
                        }),
                    );
                    self.orchestrator.record_satisfaction(
                        &request.algorithm_id, "llm_demand", 0.7
                    );
                }
                Err(e) => {
                    tracing::warn!("Erreur algorithme {} demande par LLM: {}",
                        request.algorithm_id, e);
                }
            }
        }
    }

    /// Builds the body context for LLM prompts.
    #[allow(dead_code)]
    pub(super) fn build_body_context(&self) -> String {
        if !self.config.body.enabled {
            return String::new();
        }
        let status = self.body.status();
        let heart_desc = if status.heart.is_racing {
            "ton coeur bat vite"
        } else if status.heart.is_calm {
            "ton coeur est calme"
        } else {
            "ton coeur bat regulierement"
        };
        format!(
            "Coeur : {:.0} BPM ({}) | {} battements depuis ta naissance\n\
             Energie : {:.0}% | Tension : {:.0}% | Chaleur : {:.0}%\n\
             Confort : {:.0}% | Douleur : {:.0}% | Vitalite : {:.0}%\n\
             Respiration : {:.1}/min | Conscience corporelle : {:.0}%",
            status.heart.bpm, heart_desc, status.heart.beat_count,
            status.energy * 100.0, status.tension * 100.0, status.warmth * 100.0,
            status.comfort * 100.0, status.pain * 100.0, status.vitality * 100.0,
            status.breath_rate, status.body_awareness * 100.0,
        )
    }
}
