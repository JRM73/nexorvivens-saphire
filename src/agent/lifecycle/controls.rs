// =============================================================================
// lifecycle/controls.rs — Interactive controls (REST API) (lite version)
// =============================================================================
//
// Streamlined version for the ArXiv paper.
// Removed: needs_status, knowledge_stats, suggest_topic,
//          apply_cognitive_profile, apply_personality_preset,
//          load_and_apply_personality, load_and_apply_profile.
// Simplified: set_module_weight, set_threshold, set_param, config_json
//             use config.consensus and config.feedback instead of tuner.
// =============================================================================

use tokio::time::Duration;

use crate::config::SaphireConfig;

use super::SaphireAgent;

impl SaphireAgent {
    /// Returns working memory statistics for the REST API.
    pub fn memory_data(&self) -> serde_json::Value {
        serde_json::json!({
            "working": self.working_memory.ws_data(),
        })
    }

    /// Returns a reference to the global Saphire configuration.
    pub fn config(&self) -> &SaphireConfig {
        &self.config
    }

    /// Returns the name of the currently used LLM model.
    pub fn llm_model(&self) -> &str {
        self.llm.model_name()
    }

    /// Returns the time interval between two autonomous thoughts.
    /// Auto-adjusts if the LLM is slower than the configured interval
    /// (uses 1.5x the average response time in that case).
    pub fn thought_interval(&self) -> Duration {
        if self.avg_response_time > self.thought_interval.as_secs_f64() * 0.8 {
            Duration::from_secs_f64(self.avg_response_time * 1.5)
        } else {
            self.thought_interval
        }
    }

    // ─── Interactive controls (REST API) ──────────────────────

    /// Sets the baseline of a neurotransmitter to an absolute value.
    /// The value is clamped to [0.0, 1.0].
    ///
    /// # Parameters
    /// - `molecule` — name of the neurotransmitter (e.g., "dopamine", "cortisol").
    /// - `value` — the new baseline value.
    pub fn set_baseline(&mut self, molecule: &str, value: f64) {
        let v = value.clamp(0.0, 1.0);
        match molecule {
            "dopamine" => self.baselines.dopamine = v,
            "cortisol" => self.baselines.cortisol = v,
            "serotonin" => self.baselines.serotonin = v,
            "adrenaline" => self.baselines.adrenaline = v,
            "oxytocin" => self.baselines.oxytocin = v,
            "endorphin" => self.baselines.endorphin = v,
            "noradrenaline" => self.baselines.noradrenaline = v,
            _ => {}
        }
        tracing::info!("Baseline {} → {:.2}", molecule, v);
    }

    /// Adjusts a baseline by a relative offset.
    ///
    /// # Parameters
    /// - `molecule` — name of the neurotransmitter.
    /// - `offset` — the delta to add (can be negative).
    pub fn adjust_baseline(&mut self, molecule: &str, offset: f64) {
        let current = match molecule {
            "dopamine" => self.baselines.dopamine,
            "cortisol" => self.baselines.cortisol,
            "serotonin" => self.baselines.serotonin,
            "adrenaline" => self.baselines.adrenaline,
            "oxytocin" => self.baselines.oxytocin,
            "endorphin" => self.baselines.endorphin,
            "noradrenaline" => self.baselines.noradrenaline,
            _ => return,
        };
        self.set_baseline(molecule, current + offset);
    }

    /// Sets a consensus threshold.
    /// - "yes": "Yes" threshold (clamped to [0.0, 0.8])
    /// - "no": "No" threshold (clamped to [-0.8, 0.0])
    pub fn set_threshold(&mut self, which: &str, value: f64) {
        match which {
            "yes" => self.config.consensus.threshold_yes = value.clamp(0.0, 0.8),
            "no" => self.config.consensus.threshold_no = value.clamp(-0.8, 0.0),
            _ => {}
        }
        tracing::info!("Threshold {} → {:.2}", which, value);
    }

    /// Sets a system parameter of the agent.
    /// - "thought_interval": [5, 60] seconds
    /// - "homeostasis_rate": [0.01, 0.20]
    /// - "temperature": [0.1, 1.5]
    pub fn set_param(&mut self, param: &str, value: f64) {
        match param {
            "thought_interval" => {
                let secs = value.clamp(5.0, 60.0) as u64;
                self.thought_interval = Duration::from_secs(secs);
            },
            "homeostasis_rate" => {
                self.config.feedback.homeostasis_rate = value.clamp(0.01, 0.20);
            },
            "temperature" => {
                self.config.llm.temperature = value.clamp(0.1, 1.5);
            },
            _ => {}
        }
        tracing::info!("Param {} → {:.2}", param, value);
    }

    /// Emergency stabilization: resets the chemistry to a calm state.
    /// Resets cortisol and adrenaline to baselines, boosts serotonin,
    /// endorphin, and dopamine slightly.
    pub fn emergency_stabilize(&mut self) {
        self.chemistry.cortisol = self.baselines.cortisol;
        self.chemistry.adrenaline = self.baselines.adrenaline;
        self.chemistry.serotonin = (self.chemistry.serotonin + 0.2).min(1.0);
        self.chemistry.endorphin = (self.chemistry.endorphin + 0.2).min(1.0);
        self.chemistry.dopamine = (self.chemistry.dopamine + 0.1).min(1.0);
        tracing::info!("Emergency stabilize applied");
    }

    /// Returns the virtual body status for the API.
    pub fn body_status(&self) -> crate::body::BodyStatus {
        self.body.status()
    }

    /// Returns the heart status for the API.
    pub fn heart_status(&self) -> crate::body::heart::HeartStatus {
        self.body.heart.status()
    }

    /// Returns the current chemical state as JSON for the API.
    pub fn chemistry_json(&self) -> serde_json::Value {
        serde_json::json!({
            "dopamine": self.chemistry.dopamine,
            "cortisol": self.chemistry.cortisol,
            "serotonin": self.chemistry.serotonin,
            "adrenaline": self.chemistry.adrenaline,
            "oxytocin": self.chemistry.oxytocin,
            "endorphin": self.chemistry.endorphin,
            "noradrenaline": self.chemistry.noradrenaline,
            "gaba": self.chemistry.gaba,
            "glutamate": self.chemistry.glutamate,
        })
    }

    /// Returns the current modifiable configuration as JSON for the API.
    /// Includes baselines, thresholds, and system parameters.
    pub fn config_json(&self) -> serde_json::Value {
        serde_json::json!({
            "baselines": {
                "dopamine": self.baselines.dopamine,
                "cortisol": self.baselines.cortisol,
                "serotonin": self.baselines.serotonin,
                "adrenaline": self.baselines.adrenaline,
                "oxytocin": self.baselines.oxytocin,
                "endorphin": self.baselines.endorphin,
                "noradrenaline": self.baselines.noradrenaline,
            },
            "thresholds": {
                "yes": self.config.consensus.threshold_yes,
                "no": self.config.consensus.threshold_no,
            },
            "params": {
                "thought_interval": self.thought_interval.as_secs(),
                "homeostasis_rate": self.config.feedback.homeostasis_rate,
                "temperature": self.config.llm.temperature,
            }
        })
    }
}
