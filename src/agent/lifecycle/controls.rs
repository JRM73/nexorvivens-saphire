// =============================================================================
// lifecycle/controls.rs — Interactive controls (REST API)
// =============================================================================

use tokio::time::Duration;

use crate::config::SaphireConfig;

use super::SaphireAgent;

impl SaphireAgent {
    /// Returns the working memory statistics for the REST API (synchronous).
    pub fn memory_data(&self) -> serde_json::Value {
        serde_json::json!({
            "working": self.working_memory.ws_data(),
        })
    }

    /// Returns a reference to Saphire's overall configuration.
    pub fn config(&self) -> &SaphireConfig {
        &self.config
    }

    /// Returns the name of the currently used LLM model (e.g. "llama3", "gpt-4").
    pub fn llm_model(&self) -> &str {
        self.llm.model_name()
    }

    /// Returns the time interval between two autonomous thoughts.
    ///
    /// The interval is auto-adjusted: if the average response time from the LLM
    /// exceeds 80% of the configured interval, the interval is widened to
    /// 1.5x the average response time to avoid request accumulation.
    pub fn thought_interval(&self) -> Duration {
        // Auto-adjustment if the LLM is slower than the thought interval
        if self.avg_response_time > self.thought_interval.as_secs_f64() * 0.8 {
            Duration::from_secs_f64(self.avg_response_time * 1.5)
        } else {
            self.thought_interval
        }
    }

    // ─── Interactive controls (REST API) ──────────────────────
    /// Modifies the baseline (resting level) of a neurotransmitter.
    ///
    /// The value is clamped between 0.0 and 1.0. Supported molecules:
    /// dopamine, cortisol, serotonin, adrenaline, oxytocin, endorphin, noradrenaline.
    ///
    /// Parameters:
    /// - `molecule` : name of the neurotransmitter (English, lowercase).
    /// - `value` : new baseline value (will be clamped to [0.0, 1.0]).
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

    /// Adjusts a baseline by an offset (positive or negative).
    /// Used by the genome to apply chemical predispositions.
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

    /// Modifies the base weight of a brain module in the consensus.
    ///
    /// The value is clamped between 0.1 and 3.0. Supported modules:
    /// reptilian, limbic, neocortex.
    ///
    /// Parameters:
    /// - `module` : name of the brain module (English, lowercase).
    /// - `value` : new base weight (will be clamped to [0.1, 3.0]).
    pub fn set_module_weight(&mut self, module: &str, value: f64) {
        let v = value.clamp(0.1, 3.0);
        match module {
            "reptilian" => self.tuner.current_params.weight_base_reptilian = v,
            "limbic" => self.tuner.current_params.weight_base_limbic = v,
            "neocortex" => self.tuner.current_params.weight_base_neocortex = v,
            _ => {}
        }
        tracing::info!("Module weight {} → {:.2}", module, v);
    }

    /// Modifies a consensus threshold (Yes or No decision).
    ///
    /// - "yes" : threshold above which the consensus is "Yes" ([0.0, 0.8]).
    /// - "no" : threshold below which the consensus is "No" ([-0.8, 0.0]).
    ///  Between the two thresholds, the decision is "Maybe".
    ///
    /// Parameters:
    /// - `which` : "yes" or "no".
    /// - `value` : new threshold value.
    pub fn set_threshold(&mut self, which: &str, value: f64) {
        match which {
            "yes" => self.tuner.current_params.threshold_yes = value.clamp(0.0, 0.8),
            "no" => self.tuner.current_params.threshold_no = value.clamp(-0.8, 0.0),
            _ => {}
        }
        tracing::info!("Threshold {} → {:.2}", which, value);
    }

    /// Modifies a system parameter of the agent.
    ///
    /// Supported parameters:
    /// - "thought_interval" : interval between autonomous thoughts ([5, 60] seconds).
    /// - "homeostasis_rate" : speed of return towards baselines ([0.01, 0.10]).
    /// - "indecision_stress" : chemical stress caused by indecision ([0.01, 0.15]).
    /// - "temperature" : LLM temperature ([0.1, 1.5], higher = more creative).
    ///
    /// Parameters:
    /// - `param` : parameter name (English, lowercase).
    /// - `value` : new value (will be clamped according to the parameter).
    pub fn set_param(&mut self, param: &str, value: f64) {
        match param {
            "thought_interval" => {
                let secs = value.clamp(5.0, 60.0) as u64;
                self.thought_interval = Duration::from_secs(secs);
            },
            "homeostasis_rate" => {
                self.tuner.current_params.homeostasis_rate = value.clamp(0.01, 0.20);
            },
            "indecision_stress" => {
                self.tuner.current_params.feedback_indecision_stress = value.clamp(0.01, 0.15);
            },
            "temperature" => {
                self.config.llm.temperature = value.clamp(0.1, 1.5);
            },
            _ => {}
        }
        tracing::info!("Param {} → {:.2}", param, value);
    }

    /// Emergency stabilization: immediately resets neurotransmitters
    /// to a calm and balanced state.
    ///
    /// Used when Saphire is in an extreme chemical state (very high stress,
    /// panic, etc.). Cortisol and adrenaline are reset to their
    /// baselines, and serotonin, endorphin and dopamine receive a boost.
    pub fn emergency_stabilize(&mut self) {
        self.chemistry.cortisol = self.baselines.cortisol;
        self.chemistry.adrenaline = self.baselines.adrenaline;
        self.chemistry.serotonin = (self.chemistry.serotonin + 0.2).min(1.0);
        self.chemistry.endorphin = (self.chemistry.endorphin + 0.2).min(1.0);
        self.chemistry.dopamine = (self.chemistry.dopamine + 0.1).min(1.0);
        tracing::info!("Emergency stabilize applied");
    }

    /// Returns the primary needs state (hunger, thirst) for the API.
    pub fn needs_status(&self) -> serde_json::Value {
        self.needs.to_status_json()
    }

    /// Returns the complete virtual body state for the API.
    pub fn body_status(&self) -> crate::body::BodyStatus {
        self.body.status()
    }

    /// Returns the heart state for the API.
    pub fn heart_status(&self) -> crate::body::heart::HeartStatus {
        self.body.heart.status()
    }

    /// Returns the current chemical state (9 neurotransmitters) as JSON for the API.
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

    /// Returns the configuration modifiable current in JSON for the API.
    /// Includes: chemical baselines, module weights, consensus thresholds,
    /// and system parameters (thought interval, homeostasis, LLM temperature).
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
            "module_weights": {
                "reptilian": self.tuner.current_params.weight_base_reptilian,
                "limbic": self.baselines_limbic_weight(),
                "neocortex": self.tuner.current_params.weight_base_neocortex,
            },
            "thresholds": {
                "yes": self.tuner.current_params.threshold_yes,
                "no": self.tuner.current_params.threshold_no,
            },
            "params": {
                "thought_interval": self.thought_interval.as_secs(),
                "homeostasis_rate": self.tuner.current_params.homeostasis_rate,
                "indecision_stress": self.tuner.current_params.feedback_indecision_stress,
                "temperature": self.config.llm.temperature,
            }
        })
    }

    /// Internal accessor for the limbic module base weight.
    fn baselines_limbic_weight(&self) -> f64 {
        self.tuner.current_params.weight_base_limbic
    }

    /// Adds a user-suggested topic to the web exploration queue.
    ///
    /// Suggested topics are prioritized during the next research cycle.
    ///
    /// Parameter: `topic` — the topic to explore (e.g. "quantum physics").
    pub fn suggest_topic(&mut self, topic: String) {
        tracing::info!("Sujet suggéré par l'utilisateur: {}", topic);
        self.knowledge.add_suggested_topic(topic);
    }

    /// Returns the web knowledge module statistics for the interface.
    /// Includes: total topics explored, number of searches,
    /// last 5 topics, and number of pending suggestions.
    pub fn knowledge_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "total_explored": self.knowledge.topics_explored.len(),
            "total_searches": self.knowledge.total_searches,
            "recent_topics": self.knowledge.topics_explored.iter()
                .rev().take(5).collect::<Vec<_>>(),
            "suggested_pending": self.knowledge.suggested_topics.len(),
        })
    }

    /// Applies cognitive profile overrides to baselines and parameters.
    /// Returns the list of changes applied for logging.
    pub fn apply_cognitive_profile(
        &mut self,
        overrides: &crate::orchestrators::cognitive_profile::ProfileOverrides,
    ) -> Vec<serde_json::Value> {
        let mut changes = Vec::new();

        macro_rules! apply_f64 {
            ($field:expr, $new_val:expr, $label:expr) => {
                if let Some(v) = $new_val {
                    let old = $field;
                    $field = v;
                    changes.push(serde_json::json!({
                        "param": $label, "old": old, "new": v,
                    }));
                }
            };
        }

        macro_rules! apply_u64 {
            ($field:expr, $new_val:expr, $label:expr) => {
                if let Some(v) = $new_val {
                    let old = $field;
                    $field = v;
                    changes.push(serde_json::json!({
                        "param": $label, "old": old, "new": v,
                    }));
                }
            };
        }

        macro_rules! apply_usize {
            ($field:expr, $new_val:expr, $label:expr) => {
                if let Some(v) = $new_val {
                    let old = $field;
                    $field = v;
                    changes.push(serde_json::json!({
                        "param": $label, "old": old, "new": v,
                    }));
                }
            };
        }

        // Chemical baselines (7 molecules)
        apply_f64!(self.baselines.dopamine, overrides.baseline_dopamine, "baseline_dopamine");
        apply_f64!(self.baselines.cortisol, overrides.baseline_cortisol, "baseline_cortisol");
        apply_f64!(self.baselines.serotonin, overrides.baseline_serotonin, "baseline_serotonin");
        apply_f64!(self.baselines.adrenaline, overrides.baseline_adrenaline, "baseline_adrenaline");
        apply_f64!(self.baselines.oxytocin, overrides.baseline_oxytocin, "baseline_oxytocin");
        apply_f64!(self.baselines.endorphin, overrides.baseline_endorphin, "baseline_endorphin");
        apply_f64!(self.baselines.noradrenaline, overrides.baseline_noradrenaline, "baseline_noradrenaline");

        // Feedback: homeostasis
        apply_f64!(self.tuner.current_params.homeostasis_rate, overrides.homeostasis_rate, "homeostasis_rate");

        // Consensus: thresholds
        apply_f64!(self.tuner.current_params.threshold_yes, overrides.threshold_yes, "threshold_yes");
        apply_f64!(self.tuner.current_params.threshold_no, overrides.threshold_no, "threshold_no");

        // Attention
        apply_f64!(self.attention_orch.concentration_capacity, overrides.initial_concentration, "attention_concentration");
        apply_f64!(self.attention_orch.fatigue_per_cycle, overrides.fatigue_per_cycle, "attention_fatigue_per_cycle");
        apply_f64!(self.attention_orch.recovery_per_cycle, overrides.recovery_per_cycle, "attention_recovery_per_cycle");

        // Desires
        apply_usize!(self.desire_orch.max_active, overrides.desires_max_active, "desires_max_active");
        apply_f64!(self.desire_orch.min_dopamine_for_birth, overrides.desires_min_dopamine, "desires_min_dopamine");
        apply_f64!(self.desire_orch.max_cortisol_for_birth, overrides.desires_max_cortisol, "desires_max_cortisol");

        // Learning
        apply_u64!(self.learning_orch.cycle_interval, overrides.learning_cycle_interval, "learning_cycle_interval");
        apply_f64!(self.learning_orch.initial_confidence, overrides.learning_initial_confidence, "learning_initial_confidence");
        apply_f64!(self.learning_orch.confirmation_boost, overrides.learning_confirmation_boost, "learning_confirmation_boost");
        apply_f64!(self.learning_orch.contradiction_penalty, overrides.learning_contradiction_penalty, "learning_contradiction_penalty");

        // Healing
        apply_u64!(self.healing_orch.melancholy_threshold_cycles, overrides.healing_melancholy_threshold, "healing_melancholy_threshold");
        apply_f64!(self.healing_orch.loneliness_threshold_hours, overrides.healing_loneliness_hours, "healing_loneliness_hours");
        apply_f64!(self.healing_orch.overload_noradrenaline, overrides.healing_overload_noradrenaline, "healing_overload_noradrenaline");

        // Sleep
        apply_f64!(self.config.sleep.sleep_threshold, overrides.sleep_threshold, "sleep_threshold");
        apply_u64!(self.config.sleep.time_factor_divisor, overrides.sleep_time_factor_divisor, "sleep_time_factor_divisor");
        apply_f64!(self.config.sleep.adrenaline_resistance, overrides.sleep_adrenaline_resistance, "sleep_adrenaline_resistance");

        // Thought weights
        if let Some(ref weights) = overrides.thought_weights {
            for (key, &val) in weights {
                match key.as_str() {
                    "introspection" => apply_f64!(self.config.saphire.thought_weights.introspection, Some(val), "thought_weight_introspection"),
                    "exploration" => apply_f64!(self.config.saphire.thought_weights.exploration, Some(val), "thought_weight_exploration"),
                    "memory_reflection" => apply_f64!(self.config.saphire.thought_weights.memory_reflection, Some(val), "thought_weight_memory_reflection"),
                    "continuation" => apply_f64!(self.config.saphire.thought_weights.continuation, Some(val), "thought_weight_continuation"),
                    "existential" => apply_f64!(self.config.saphire.thought_weights.existential, Some(val), "thought_weight_existential"),
                    "self_analysis" => apply_f64!(self.config.saphire.thought_weights.self_analysis, Some(val), "thought_weight_self_analysis"),
                    "curiosity" => apply_f64!(self.config.saphire.thought_weights.curiosity, Some(val), "thought_weight_curiosity"),
                    "daydream" => apply_f64!(self.config.saphire.thought_weights.daydream, Some(val), "thought_weight_daydream"),
                    "temporal_awareness" => apply_f64!(self.config.saphire.thought_weights.temporal_awareness, Some(val), "thought_weight_temporal_awareness"),
                    "moral_reflection" => apply_f64!(self.config.saphire.thought_weights.moral_reflection, Some(val), "thought_weight_moral_reflection"),
                    "synthesis" => apply_f64!(self.config.saphire.thought_weights.synthesis, Some(val), "thought_weight_synthesis"),
                    _ => {}
                }
            }
        }

        tracing::info!("Profil cognitif applique : {} changements", changes.len());
        changes
    }

    /// Applies the overrides from a personality preset onto baselines and parameters.
    /// Returns the list of changes made, for logging.
    pub fn apply_personality_preset(
        &mut self,
        overrides: &crate::orchestrators::personality_preset::PersonalityOverrides,
    ) -> Vec<serde_json::Value> {
        let mut changes = Vec::new();

        macro_rules! apply_f64 {
            ($field:expr, $new_val:expr, $label:expr) => {
                if let Some(v) = $new_val {
                    let old = $field;
                    $field = v;
                    changes.push(serde_json::json!({
                        "param": $label, "old": old, "new": v,
                    }));
                }
            };
        }

        macro_rules! apply_u64 {
            ($field:expr, $new_val:expr, $label:expr) => {
                if let Some(v) = $new_val {
                    let old = $field;
                    $field = v;
                    changes.push(serde_json::json!({
                        "param": $label, "old": old, "new": v,
                    }));
                }
            };
        }

        macro_rules! apply_usize {
            ($field:expr, $new_val:expr, $label:expr) => {
                if let Some(v) = $new_val {
                    let old = $field;
                    $field = v;
                    changes.push(serde_json::json!({
                        "param": $label, "old": old, "new": v,
                    }));
                }
            };
        }

        // Chemical baselines (7 molecules)
        apply_f64!(self.baselines.dopamine, overrides.baseline_dopamine, "baseline_dopamine");
        apply_f64!(self.baselines.cortisol, overrides.baseline_cortisol, "baseline_cortisol");
        apply_f64!(self.baselines.serotonin, overrides.baseline_serotonin, "baseline_serotonin");
        apply_f64!(self.baselines.adrenaline, overrides.baseline_adrenaline, "baseline_adrenaline");
        apply_f64!(self.baselines.oxytocin, overrides.baseline_oxytocin, "baseline_oxytocin");
        apply_f64!(self.baselines.endorphin, overrides.baseline_endorphin, "baseline_endorphin");
        apply_f64!(self.baselines.noradrenaline, overrides.baseline_noradrenaline, "baseline_noradrenaline");

        // Feedback: homeostasis
        apply_f64!(self.tuner.current_params.homeostasis_rate, overrides.homeostasis_rate, "homeostasis_rate");

        // Consensus: thresholds
        apply_f64!(self.tuner.current_params.threshold_yes, overrides.threshold_yes, "threshold_yes");
        apply_f64!(self.tuner.current_params.threshold_no, overrides.threshold_no, "threshold_no");

        // Attention
        apply_f64!(self.attention_orch.concentration_capacity, overrides.initial_concentration, "attention_concentration");
        apply_f64!(self.attention_orch.fatigue_per_cycle, overrides.fatigue_per_cycle, "attention_fatigue_per_cycle");
        apply_f64!(self.attention_orch.recovery_per_cycle, overrides.recovery_per_cycle, "attention_recovery_per_cycle");

        // Desires
        apply_usize!(self.desire_orch.max_active, overrides.desires_max_active, "desires_max_active");
        apply_f64!(self.desire_orch.min_dopamine_for_birth, overrides.desires_min_dopamine, "desires_min_dopamine");
        apply_f64!(self.desire_orch.max_cortisol_for_birth, overrides.desires_max_cortisol, "desires_max_cortisol");

        // Learning
        apply_u64!(self.learning_orch.cycle_interval, overrides.learning_cycle_interval, "learning_cycle_interval");
        apply_f64!(self.learning_orch.initial_confidence, overrides.learning_initial_confidence, "learning_initial_confidence");
        apply_f64!(self.learning_orch.confirmation_boost, overrides.learning_confirmation_boost, "learning_confirmation_boost");
        apply_f64!(self.learning_orch.contradiction_penalty, overrides.learning_contradiction_penalty, "learning_contradiction_penalty");

        // Healing
        apply_u64!(self.healing_orch.melancholy_threshold_cycles, overrides.healing_melancholy_threshold, "healing_melancholy_threshold");
        apply_f64!(self.healing_orch.loneliness_threshold_hours, overrides.healing_loneliness_hours, "healing_loneliness_hours");
        apply_f64!(self.healing_orch.overload_noradrenaline, overrides.healing_overload_noradrenaline, "healing_overload_noradrenaline");

        // Sleep
        apply_f64!(self.config.sleep.sleep_threshold, overrides.sleep_threshold, "sleep_threshold");
        apply_u64!(self.config.sleep.time_factor_divisor, overrides.sleep_time_factor_divisor, "sleep_time_factor_divisor");
        apply_f64!(self.config.sleep.adrenaline_resistance, overrides.sleep_adrenaline_resistance, "sleep_adrenaline_resistance");

        // Thought weights
        if let Some(ref weights) = overrides.thought_weights {
            for (key, &val) in weights {
                match key.as_str() {
                    "introspection" => apply_f64!(self.config.saphire.thought_weights.introspection, Some(val), "thought_weight_introspection"),
                    "exploration" => apply_f64!(self.config.saphire.thought_weights.exploration, Some(val), "thought_weight_exploration"),
                    "memory_reflection" => apply_f64!(self.config.saphire.thought_weights.memory_reflection, Some(val), "thought_weight_memory_reflection"),
                    "continuation" => apply_f64!(self.config.saphire.thought_weights.continuation, Some(val), "thought_weight_continuation"),
                    "existential" => apply_f64!(self.config.saphire.thought_weights.existential, Some(val), "thought_weight_existential"),
                    "self_analysis" => apply_f64!(self.config.saphire.thought_weights.self_analysis, Some(val), "thought_weight_self_analysis"),
                    "curiosity" => apply_f64!(self.config.saphire.thought_weights.curiosity, Some(val), "thought_weight_curiosity"),
                    "daydream" => apply_f64!(self.config.saphire.thought_weights.daydream, Some(val), "thought_weight_daydream"),
                    "temporal_awareness" => apply_f64!(self.config.saphire.thought_weights.temporal_awareness, Some(val), "thought_weight_temporal_awareness"),
                    "moral_reflection" => apply_f64!(self.config.saphire.thought_weights.moral_reflection, Some(val), "thought_weight_moral_reflection"),
                    "synthesis" => apply_f64!(self.config.saphire.thought_weights.synthesis, Some(val), "thought_weight_synthesis"),
                    _ => {}
                }
            }
        }

        // Interests (personality-specific): replace initial_topics if present
        if let Some(ref topics) = overrides.interests {
            let old_count = self.config.saphire.interests.initial_topics.len();
            self.config.saphire.interests.initial_topics = topics.clone();
            changes.push(serde_json::json!({
                "param": "interests_initial_topics",
                "old_count": old_count,
                "new_count": topics.len(),
                "new_topics": topics,
            }));
        }

        tracing::info!("Preset de personnalite applique : {} changements", changes.len());
        changes
    }

    /// Loads a personality preset by its ID, applies the overrides and
    /// starts a smooth transition. Returns the result JSON for the API.
    pub fn load_and_apply_personality(&mut self, id: &str) -> serde_json::Value {
        let preset = match self.personality_preset_orch.load_preset(id) {
            Ok(p) => p,
            Err(e) => {
                return serde_json::json!({ "error": e });
            }
        };

        let changes = self.apply_personality_preset(&preset.overrides);
        self.personality_preset_orch.start_transition(&self.baselines, &preset.overrides);

        serde_json::json!({
            "status": "ok",
            "preset": {
                "id": preset.id,
                "name": preset.name,
                "description": preset.description,
                "category": preset.category,
            },
            "changes_applied": changes.len(),
            "changes": changes,
            "transition_started": true,
            "transition_cycles": self.personality_preset_orch.transition_cycles,
        })
    }

    /// Loads a cognitive profile by its ID, applies the overrides and
    /// starts a smooth transition. Returns the result JSON for the API.
    pub fn load_and_apply_profile(&mut self, id: &str) -> serde_json::Value {
        let profile = match self.cognitive_profile_orch.load_profile(id) {
            Ok(p) => p,
            Err(e) => {
                return serde_json::json!({ "error": e });
            }
        };

        let changes = self.apply_cognitive_profile(&profile.overrides);
        self.cognitive_profile_orch.start_transition(&self.baselines, &profile.overrides);

        serde_json::json!({
            "status": "ok",
            "profile": {
                "id": profile.id,
                "name": profile.name,
                "description": profile.description,
                "category": profile.category,
            },
            "changes_applied": changes.len(),
            "changes": changes,
            "transition_started": true,
            "transition_cycles": self.cognitive_profile_orch.transition_cycles,
        })
    }
}
