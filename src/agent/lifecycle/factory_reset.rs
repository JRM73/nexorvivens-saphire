// =============================================================================
// lifecycle/factory_reset.rs — Factory reset to default values (lite version)
// =============================================================================
//
// This file implements factory reset functionality at various levels:
// - ChemistryOnly: reset neurotransmitter levels and baselines
// - ParametersOnly: reset all tunable parameters (chemistry + thresholds + LLM)
// - IntuitionOnly: reset intuition and premonition modules
// - PersonalEthicsOnly: deactivate all personal ethical principles
// - FullReset: all of the above + clear episodic memories + reset cycle count
//
// Also provides `factory_diff()` to compare current state vs factory defaults.
// =============================================================================

use crate::logging::{LogLevel, LogCategory};
use super::SaphireAgent;

impl SaphireAgent {
    /// Resets all neurotransmitter levels and baselines to factory defaults.
    ///
    /// # Parameters
    /// - `factory` — the factory defaults loaded from configuration.
    ///
    /// # Returns
    /// A vector of JSON values describing each parameter change (old -> new).
    pub(super) fn apply_chemistry_reset(&mut self, factory: &crate::factory::FactoryDefaults) -> Vec<serde_json::Value> {
        let baselines = factory.reset_chemistry();
        let labels = ["dopamine", "cortisol", "serotonin", "adrenaline",
                      "oxytocin", "endorphin", "noradrenaline"];
        let current = [
            self.chemistry.dopamine, self.chemistry.cortisol,
            self.chemistry.serotonin, self.chemistry.adrenaline,
            self.chemistry.oxytocin, self.chemistry.endorphin,
            self.chemistry.noradrenaline,
        ];
        let mut changes = Vec::new();
        for (i, &label) in labels.iter().enumerate() {
            if (current[i] - baselines[i]).abs() > 0.001 {
                changes.push(serde_json::json!({
                    "param": label, "old": current[i], "new": baselines[i]
                }));
            }
        }
        self.chemistry.dopamine = baselines[0];
        self.chemistry.cortisol = baselines[1];
        self.chemistry.serotonin = baselines[2];
        self.chemistry.adrenaline = baselines[3];
        self.chemistry.oxytocin = baselines[4];
        self.chemistry.endorphin = baselines[5];
        self.chemistry.noradrenaline = baselines[6];
        self.baselines.dopamine = baselines[0];
        self.baselines.cortisol = baselines[1];
        self.baselines.serotonin = baselines[2];
        self.baselines.adrenaline = baselines[3];
        self.baselines.oxytocin = baselines[4];
        self.baselines.endorphin = baselines[5];
        self.baselines.noradrenaline = baselines[6];
        changes
    }

    /// Resets all tunable parameters to factory defaults, including chemistry,
    /// homeostasis rate, consensus thresholds, temperature, max tokens, and
    /// episodic decay rate.
    ///
    /// # Returns
    /// A vector of JSON values describing each parameter change.
    pub(super) fn apply_parameters_reset(&mut self, factory: &crate::factory::FactoryDefaults) -> Vec<serde_json::Value> {
        let params = factory.reset_parameters();
        let mut changes = self.apply_chemistry_reset(factory);

        let old_rate = self.config.feedback.homeostasis_rate;
        if (old_rate - params.homeostasis_rate).abs() > 0.001 {
            changes.push(serde_json::json!({"param": "homeostasis_rate", "old": old_rate, "new": params.homeostasis_rate}));
            self.config.feedback.homeostasis_rate = params.homeostasis_rate;
        }
        let old_yes = self.config.consensus.threshold_yes;
        if (old_yes - params.threshold_yes).abs() > 0.001 {
            changes.push(serde_json::json!({"param": "threshold_yes", "old": old_yes, "new": params.threshold_yes}));
            self.config.consensus.threshold_yes = params.threshold_yes;
        }
        let old_no = self.config.consensus.threshold_no;
        if (old_no - params.threshold_no).abs() > 0.001 {
            changes.push(serde_json::json!({"param": "threshold_no", "old": old_no, "new": params.threshold_no}));
            self.config.consensus.threshold_no = params.threshold_no;
        }
        let old_temp = self.config.llm.temperature;
        if (old_temp - params.temperature).abs() > 0.001 {
            changes.push(serde_json::json!({"param": "temperature", "old": old_temp, "new": params.temperature}));
            self.config.llm.temperature = params.temperature;
        }
        let old_max = self.config.llm.max_tokens;
        if old_max != params.max_tokens {
            changes.push(serde_json::json!({"param": "max_tokens", "old": old_max, "new": params.max_tokens}));
            self.config.llm.max_tokens = params.max_tokens;
        }
        let old_decay = self.config.memory.episodic_decay_rate;
        if (old_decay - params.episodic_decay).abs() > 0.001 {
            changes.push(serde_json::json!({"param": "episodic_decay_rate", "old": old_decay, "new": params.episodic_decay}));
            self.config.memory.episodic_decay_rate = params.episodic_decay;
        }
        changes
    }

    /// Applies a factory reset at the specified level.
    /// Broadcasts the result to WebSocket and logs the changes.
    ///
    /// # Parameters
    /// - `level` — the reset level (ChemistryOnly, ParametersOnly, IntuitionOnly,
    ///   PersonalEthicsOnly, FullReset, or others not available in lite).
    ///
    /// # Returns
    /// A JSON value with `success`, `level`, and `changes` fields.
    pub async fn apply_factory_reset(&mut self, level: crate::factory::ResetLevel) -> serde_json::Value {
        use crate::factory::{FactoryDefaults, ResetLevel};

        let factory = match FactoryDefaults::load() {
            Ok(f) => f,
            Err(e) => return serde_json::json!({"error": e}),
        };

        let changes = match level {
            ResetLevel::ChemistryOnly => {
                let c = self.apply_chemistry_reset(&factory);
                tracing::info!("Factory reset: chimie ({} changements)", c.len());
                c
            }
            ResetLevel::ParametersOnly => {
                let c = self.apply_parameters_reset(&factory);
                tracing::info!("Factory reset: parametres ({} changements)", c.len());
                c
            }
            ResetLevel::IntuitionOnly => {
                let mut c = Vec::new();
                let params = factory.reset_parameters();
                let old_acuity = self.intuition.acuity;
                let old_accuracy = self.intuition.accuracy;
                self.intuition.acuity = params.intuition_initial_acuity;
                self.intuition.accuracy = params.intuition_initial_accuracy;
                self.intuition.pattern_buffer.clear();
                c.push(serde_json::json!({"param": "intuition_acuity", "old": old_acuity, "new": params.intuition_initial_acuity}));
                c.push(serde_json::json!({"param": "intuition_accuracy", "old": old_accuracy, "new": params.intuition_initial_accuracy}));
                let old_pred_acc = self.premonition.accuracy;
                self.premonition.accuracy = params.premonition_initial_accuracy;
                self.premonition.active_predictions.clear();
                c.push(serde_json::json!({"param": "premonition_accuracy", "old": old_pred_acc, "new": params.premonition_initial_accuracy}));
                c
            }
            ResetLevel::PersonalEthicsOnly => {
                let mut c = Vec::new();
                let count = self.ethics.deactivate_all_personal();
                c.push(serde_json::json!({"param": "personal_principles", "old": count, "new": "all_deactivated"}));
                c
            }
            ResetLevel::FullReset => {
                let mut c = self.apply_parameters_reset(&factory);
                // Reset intuition/premonition
                let params = factory.reset_parameters();
                self.intuition.acuity = params.intuition_initial_acuity;
                self.intuition.accuracy = params.intuition_initial_accuracy;
                self.intuition.pattern_buffer.clear();
                self.premonition.accuracy = params.premonition_initial_accuracy;
                self.premonition.active_predictions.clear();
                c.push(serde_json::json!({"param": "intuition_premonition", "old": "reset", "new": "initial"}));
                // Desactiver ethique personnelle
                let ethics_count = self.ethics.deactivate_all_personal();
                c.push(serde_json::json!({"param": "personal_ethics", "old": ethics_count, "new": "deactivated"}));
                // Effacer episodiques
                if let Some(ref db) = self.db {
                    if let Ok(n) = db.clear_episodic_memories().await {
                        c.push(serde_json::json!({"param": "episodic_memories", "old": n, "new": 0}));
                    }
                }
                self.cycle_count = 0;
                self.last_consolidation_cycle = 0;
                c
            }
            // Levels not available in the lite version
            _ => {
                vec![serde_json::json!({"param": "warning", "old": "N/A", "new": "Level not available in lite version"})]
            }
        };

        if let Some(ref tx) = self.ws_tx {
            let msg = serde_json::json!({"type": "factory_reset_done", "level": format!("{:?}", level), "changes": changes});
            let _ = tx.send(msg.to_string());
        }

        self.log(LogLevel::Info, LogCategory::System,
            format!("Factory reset {:?}: {} changements", level, changes.len()),
            serde_json::json!({"changes": changes.len()}),
        );

        serde_json::json!({"success": true, "level": format!("{:?}", level), "changes": changes})
    }

    /// Computes the differences between the current agent state and factory defaults.
    ///
    /// # Returns
    /// A JSON value with `diffs` (array of parameter differences),
    /// `total_params` (total number of compared parameters), and
    /// `modified_count` (number of parameters that differ).
    pub fn factory_diff(&self) -> serde_json::Value {
        use crate::factory::FactoryDefaults;

        let factory = match FactoryDefaults::load() {
            Ok(f) => f,
            Err(e) => return serde_json::json!({"error": e}),
        };

        let params = factory.reset_parameters();
        let mut diffs = Vec::new();

        let labels = ["dopamine", "cortisol", "serotonin", "adrenaline",
                      "oxytocin", "endorphin", "noradrenaline"];
        let current = [
            self.baselines.dopamine, self.baselines.cortisol,
            self.baselines.serotonin, self.baselines.adrenaline,
            self.baselines.oxytocin, self.baselines.endorphin,
            self.baselines.noradrenaline,
        ];
        for (i, &label) in labels.iter().enumerate() {
            if (current[i] - params.baselines[i]).abs() > 0.001 {
                diffs.push(serde_json::json!({
                    "param": format!("baseline_{}", label),
                    "current": current[i], "factory": params.baselines[i],
                    "diff": current[i] - params.baselines[i],
                }));
            }
        }

        let checks: Vec<(&str, f64, f64)> = vec![
            ("homeostasis_rate", self.config.feedback.homeostasis_rate, params.homeostasis_rate),
            ("threshold_yes", self.config.consensus.threshold_yes, params.threshold_yes),
            ("threshold_no", self.config.consensus.threshold_no, params.threshold_no),
            ("temperature", self.config.llm.temperature, params.temperature),
            ("episodic_decay_rate", self.config.memory.episodic_decay_rate, params.episodic_decay),
            ("consolidation_threshold", self.config.memory.consolidation_threshold, params.consolidation_threshold),
        ];

        for (name, current_val, factory_val) in checks {
            if (current_val - factory_val).abs() > 0.001 {
                diffs.push(serde_json::json!({
                    "param": name, "current": current_val, "factory": factory_val,
                    "diff": current_val - factory_val,
                }));
            }
        }

        serde_json::json!({"diffs": diffs, "total_params": 13, "modified_count": diffs.len()})
    }
}
