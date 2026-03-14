// =============================================================================
// lifecycle/factory_reset.rs — Reset to factory defaults (multiple levels)
// =============================================================================

use crate::logging::{LogLevel, LogCategory};

use super::SaphireAgent;

impl SaphireAgent {
    /// Synchronous helper: resets the 7 molecules to factory baselines.
    /// Returns the list of changes made.
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

        // Also reset the baselines
        self.baselines.dopamine = baselines[0];
        self.baselines.cortisol = baselines[1];
        self.baselines.serotonin = baselines[2];
        self.baselines.adrenaline = baselines[3];
        self.baselines.oxytocin = baselines[4];
        self.baselines.endorphin = baselines[5];
        self.baselines.noradrenaline = baselines[6];

        changes
    }

    /// Synchronous helper: resets operational parameters to factory defaults.
    /// Includes chemistry reset. Returns the list of changes.
    pub(super) fn apply_parameters_reset(&mut self, factory: &crate::factory::FactoryDefaults) -> Vec<serde_json::Value> {
        let params = factory.reset_parameters();

        // Reset chemistry (included in parameters)
        let mut changes = self.apply_chemistry_reset(factory);

        // Homeostasis
        let old_rate = self.config.feedback.homeostasis_rate;
        if (old_rate - params.homeostasis_rate).abs() > 0.001 {
            changes.push(serde_json::json!({
                "param": "homeostasis_rate", "old": old_rate, "new": params.homeostasis_rate
            }));
            self.config.feedback.homeostasis_rate = params.homeostasis_rate;
        }

        // Consensus thresholds
        let old_yes = self.config.consensus.threshold_yes;
        if (old_yes - params.threshold_yes).abs() > 0.001 {
            changes.push(serde_json::json!({
                "param": "threshold_yes", "old": old_yes, "new": params.threshold_yes
            }));
            self.config.consensus.threshold_yes = params.threshold_yes;
        }
        let old_no = self.config.consensus.threshold_no;
        if (old_no - params.threshold_no).abs() > 0.001 {
            changes.push(serde_json::json!({
                "param": "threshold_no", "old": old_no, "new": params.threshold_no
            }));
            self.config.consensus.threshold_no = params.threshold_no;
        }

        // LLM
        let old_temp = self.config.llm.temperature;
        if (old_temp - params.temperature).abs() > 0.001 {
            changes.push(serde_json::json!({
                "param": "temperature", "old": old_temp, "new": params.temperature
            }));
            self.config.llm.temperature = params.temperature;
        }
        let old_max = self.config.llm.max_tokens;
        if old_max != params.max_tokens {
            changes.push(serde_json::json!({
                "param": "max_tokens", "old": old_max, "new": params.max_tokens
            }));
            self.config.llm.max_tokens = params.max_tokens;
        }

        // Memory
        let old_decay = self.config.memory.episodic_decay_rate;
        if (old_decay - params.episodic_decay).abs() > 0.001 {
            changes.push(serde_json::json!({
                "param": "episodic_decay_rate", "old": old_decay, "new": params.episodic_decay
            }));
            self.config.memory.episodic_decay_rate = params.episodic_decay;
        }

        changes
    }

    pub async fn apply_factory_reset(&mut self, level: crate::factory::ResetLevel) -> serde_json::Value {
        use crate::factory::{FactoryDefaults, ResetLevel};

        let factory = match FactoryDefaults::load() {
            Ok(f) => f,
            Err(e) => return serde_json::json!({"error": e}),
        };

        let changes = match level {
            ResetLevel::ChemistryOnly => {
                let c = self.apply_chemistry_reset(&factory);
                tracing::info!("Factory reset: chimie remise aux baselines ({} changements)", c.len());
                c
            }
            ResetLevel::ParametersOnly => {
                let c = self.apply_parameters_reset(&factory);
                tracing::info!("Factory reset: parametres restaures ({} changements)", c.len());
                c
            }
            ResetLevel::SensesOnly => {
                let mut c = Vec::new();
                let params = factory.reset_parameters();
                // Reset acuity of the 5 senses
                self.sensorium.reading.acuity = params.reading_initial_acuity;
                c.push(serde_json::json!({"param": "reading_acuity", "old": "?", "new": params.reading_initial_acuity}));
                self.sensorium.listening.acuity = params.listening_initial_acuity;
                c.push(serde_json::json!({"param": "listening_acuity", "old": "?", "new": params.listening_initial_acuity}));
                self.sensorium.contact.acuity = params.contact_initial_acuity;
                c.push(serde_json::json!({"param": "contact_acuity", "old": "?", "new": params.contact_initial_acuity}));
                self.sensorium.taste.acuity = params.taste_initial_acuity;
                c.push(serde_json::json!({"param": "taste_acuity", "old": "?", "new": params.taste_initial_acuity}));
                self.sensorium.ambiance.acuity = params.ambiance_initial_acuity;
                c.push(serde_json::json!({"param": "ambiance_acuity", "old": "?", "new": params.ambiance_initial_acuity}));
                // Reset stimulation of emergent seeds (do not un-germinate)
                self.sensorium.emergent_seeds.reset_stimulation();
                c.push(serde_json::json!({"param": "emergent_seeds_stimulation", "old": "?", "new": 0}));
                tracing::info!("Factory reset: sens remis aux valeurs initiales ({} changements)", c.len());
                c
            }
            ResetLevel::IntuitionOnly => {
                let mut c = Vec::new();
                let params = factory.reset_parameters();
                // Reset intuition
                let old_acuity = self.intuition.acuity;
                let old_accuracy = self.intuition.accuracy;
                self.intuition.acuity = params.intuition_initial_acuity;
                self.intuition.accuracy = params.intuition_initial_accuracy;
                self.intuition.pattern_buffer.clear();
                c.push(serde_json::json!({"param": "intuition_acuity", "old": old_acuity, "new": params.intuition_initial_acuity}));
                c.push(serde_json::json!({"param": "intuition_accuracy", "old": old_accuracy, "new": params.intuition_initial_accuracy}));
                c.push(serde_json::json!({"param": "intuition_patterns", "old": "cleared", "new": 0}));
                // Reset premonition
                let old_pred_acc = self.premonition.accuracy;
                self.premonition.accuracy = params.premonition_initial_accuracy;
                self.premonition.active_predictions.clear();
                c.push(serde_json::json!({"param": "premonition_accuracy", "old": old_pred_acc, "new": params.premonition_initial_accuracy}));
                c.push(serde_json::json!({"param": "premonition_predictions", "old": "cleared", "new": 0}));
                tracing::info!("Factory reset: intuition/premonition remises aux valeurs initiales ({} changements)", c.len());
                c
            }
            ResetLevel::PersonalEthicsOnly => {
                let mut c = Vec::new();
                let count = self.ethics.deactivate_all_personal();
                c.push(serde_json::json!({"param": "personal_principles", "old": count, "new": "all_deactivated"}));
                tracing::info!("Factory reset: {} principes personnels desactives", count);
                c
            }
            ResetLevel::PsychologyOnly => {
                let mut c = Vec::new();
                let config = &self.config.psychology;
                let will_config = &self.config.will;
                let old_eq = self.psychology.eq.overall_eq;
                let old_integ = self.psychology.jung.integration;
                self.psychology = crate::psychology::PsychologyFramework::new(config, will_config);
                c.push(serde_json::json!({"param": "freudian", "old": "reset", "new": "initial"}));
                c.push(serde_json::json!({"param": "maslow", "old": "reset", "new": "initial"}));
                c.push(serde_json::json!({"param": "toltec", "old": "reset", "new": "initial"}));
                c.push(serde_json::json!({"param": "jung_shadow", "old": format!("{:.0}%", old_integ * 100.0), "new": "0%"}));
                c.push(serde_json::json!({"param": "eq", "old": format!("{:.0}%", old_eq * 100.0), "new": "30%"}));
                c.push(serde_json::json!({"param": "flow", "old": "reset", "new": "initial"}));
                tracing::info!("Factory reset: psychologie reintialisee ({} changements)", c.len());
                c
            }
            ResetLevel::SleepOnly => {
                let mut c = Vec::new();
                // Reset sleep pressure
                let old_pressure = self.sleep.drive.sleep_pressure;
                self.sleep.drive.sleep_pressure = 0.0;
                self.sleep.drive.cycles_since_last_sleep = 0;
                self.sleep.drive.sleep_forced = false;
                c.push(serde_json::json!({"param": "sleep_pressure", "old": old_pressure, "new": 0.0}));
                // Reset fatigue levels
                let old_dec = self.psychology.will.decision_fatigue;
                self.psychology.will.decision_fatigue = 0.0;
                self.attention_orch.reset_fatigue();
                c.push(serde_json::json!({"param": "decision_fatigue", "old": old_dec, "new": 0.0}));
                c.push(serde_json::json!({"param": "attention_fatigue", "old": "reset", "new": 0.0}));
                // DO NOT TOUCH: sleep_history, neural_connections, subconscious
                tracing::info!("Factory reset: sommeil reinitialise ({} changements)", c.len());
                c
            }
            ResetLevel::BiologyReset => {
                let mut c = Vec::new();

                // Reset hormonal_system receptors (sensitivity=1.0, tolerance=0.0)
                self.hormonal_system.receptors = crate::hormones::receptors::ReceptorSystem::new(&self.config.hormones);
                c.push(serde_json::json!({"param": "hormonal_receptors", "old": "reset", "new": "sensitivity=1.0, tolerance=0.0"}));

                // Reset receptor_bank (neuroscience/receptors.rs)
                self.receptor_bank = crate::neuroscience::receptors::ReceptorBank::new();
                c.push(serde_json::json!({"param": "receptor_bank", "old": "reset", "new": "density=1.0, tolerance=0.0"}));

                // Reset BDNF to baseline (0.5)
                let old_bdnf = self.grey_matter.bdnf_level;
                self.grey_matter.bdnf_level = self.config.bdnf.homeostasis_baseline;
                c.push(serde_json::json!({"param": "bdnf_level", "old": old_bdnf, "new": self.config.bdnf.homeostasis_baseline}));

                // Reset grey_matter to default values
                self.grey_matter = crate::biology::grey_matter::GreyMatterSystem::new(&self.config.grey_matter);
                c.push(serde_json::json!({"param": "grey_matter", "old": "reset", "new": "initial"}));

                tracing::info!("Factory reset: biologie (recepteurs + BDNF + matiere grise) reinitialises ({} changements)", c.len());
                c
            }
            ResetLevel::FullReset => {
                // Reset parameters (includes chemistry)
                let mut c = self.apply_parameters_reset(&factory);

                // Reset senses
                let params = factory.reset_parameters();
                self.sensorium.reading.acuity = params.reading_initial_acuity;
                self.sensorium.listening.acuity = params.listening_initial_acuity;
                self.sensorium.contact.acuity = params.contact_initial_acuity;
                self.sensorium.taste.acuity = params.taste_initial_acuity;
                self.sensorium.ambiance.acuity = params.ambiance_initial_acuity;
                self.sensorium.emergent_seeds.reset_stimulation();
                c.push(serde_json::json!({"param": "senses", "old": "reset", "new": "initial"}));

                // Reset intuition/premonition
                self.intuition.acuity = params.intuition_initial_acuity;
                self.intuition.accuracy = params.intuition_initial_accuracy;
                self.intuition.pattern_buffer.clear();
                self.premonition.accuracy = params.premonition_initial_accuracy;
                self.premonition.active_predictions.clear();
                c.push(serde_json::json!({"param": "intuition_premonition", "old": "reset", "new": "initial"}));

                // Deactivate personal ethics
                let ethics_count = self.ethics.deactivate_all_personal();
                c.push(serde_json::json!({"param": "personal_ethics", "old": ethics_count, "new": "deactivated"}));

                // Clear episodic memories (LTM and founding memories preserved)
                if let Some(ref db) = self.db {
                    match db.clear_episodic_memories().await {
                        Ok(n) => {
                            c.push(serde_json::json!({
                                "param": "episodic_memories", "old": n, "new": 0
                            }));
                            tracing::info!("Factory reset complet: {} souvenirs episodiques effaces", n);
                        }
                        Err(e) => tracing::warn!("Erreur clear episodic: {}", e),
                    }

                    // Clear vector learnings (nn_learnings)
                    match db.clear_nn_learnings().await {
                        Ok(n) => {
                            c.push(serde_json::json!({
                                "param": "nn_learnings", "old": n, "new": 0
                            }));
                            tracing::info!("Factory reset complet: {} apprentissages vectoriels effaces", n);
                        }
                        Err(e) => tracing::warn!("Erreur clear nn_learnings: {}", e),
                    }
                }
                self.cycles_since_last_nn_learning = 0;

                // Reset psychology
                let config = &self.config.psychology;
                let will_config = &self.config.will;
                self.psychology = crate::psychology::PsychologyFramework::new(config, will_config);
                c.push(serde_json::json!({"param": "psychology", "old": "reset", "new": "initial"}));

                // Reset relationships
                self.relationships = crate::relationships::RelationshipNetwork::default();
                c.push(serde_json::json!({"param": "relationships", "old": "reset", "new": "initial"}));

                // Reset metacognition (preserve turing history for continuity)
                self.metacognition = crate::metacognition::MetaCognitionEngine::new();
                self.metacognition.enabled = self.config.metacognition.enabled;
                self.metacognition.check_interval = self.config.metacognition.check_interval;
                c.push(serde_json::json!({"param": "metacognition", "old": "reset", "new": "initial"}));

                // Reset sentiments (lasting affective states)
                self.sentiments.reset();
                c.push(serde_json::json!({"param": "sentiments", "old": "reset", "new": "initial"}));

                // Reset innate biological modules
                self.nutrition = crate::biology::nutrition::NutritionSystem::new(&self.config.nutrition);
                c.push(serde_json::json!({"param": "nutrition", "old": "reset", "new": "initial"}));
                self.grey_matter = crate::biology::grey_matter::GreyMatterSystem::new(&self.config.grey_matter);
                c.push(serde_json::json!({"param": "grey_matter", "old": "reset", "new": "initial"}));
                self.em_fields = crate::biology::fields::ElectromagneticFields::new(&self.config.fields);
                c.push(serde_json::json!({"param": "em_fields", "old": "reset", "new": "initial"}));

                // Reset receptors (sensitivity=1.0, tolerance=0.0)
                self.hormonal_system.receptors = crate::hormones::receptors::ReceptorSystem::new(&self.config.hormones);
                c.push(serde_json::json!({"param": "hormonal_receptors", "old": "reset", "new": "sensitivity=1.0, tolerance=0.0"}));
                self.receptor_bank = crate::neuroscience::receptors::ReceptorBank::new();
                c.push(serde_json::json!({"param": "receptor_bank", "old": "reset", "new": "density=1.0, tolerance=0.0"}));

                // Reset memory recall params to defaults
                self.config.memory.recall_episodic_limit = 5;
                self.config.memory.recall_ltm_limit = 5;
                self.config.memory.recall_ltm_threshold = 0.25;
                self.config.memory.recall_archive_limit = 3;
                self.config.memory.recall_archive_threshold = 0.25;
                self.config.memory.recall_vectors_limit = 3;
                self.config.memory.recall_vectors_threshold = 0.30;
                c.push(serde_json::json!({"param": "memory_recall", "old": "reset", "new": "defaults"}));

                // Reset spinal cord (reflexes)
                self.spine = crate::spine::SpinalCord::new();
                c.push(serde_json::json!({"param": "spine", "old": "reset", "new": "initial"}));

                // Reset curiosity
                self.curiosity = crate::cognition::curiosity::CuriosityDrive::new();
                c.push(serde_json::json!({"param": "curiosity", "old": "reset", "new": "initial"}));

                // Reset drift monitor
                self.drift_monitor = crate::cognition::drift_monitor::DriftMonitor::new();
                self.drift_monitor.initialize(&*self.encoder);
                c.push(serde_json::json!({"param": "drift_monitor", "old": "reset", "new": "re-initialized"}));

                // Reset counters
                self.cycle_count = 0;
                self.last_consolidation_cycle = 0;

                tracing::info!("Factory reset complet ({} changements)", c.len());
                c
            }
        };

        // Notify the frontend
        if let Some(ref tx) = self.ws_tx {
            let msg = serde_json::json!({
                "type": "factory_reset_done",
                "level": format!("{:?}", level),
                "changes": changes,
            });
            let _ = tx.send(msg.to_string());
        }

        self.log(LogLevel::Info, LogCategory::System,
            format!("Factory reset {:?}: {} changements", level, changes.len()),
            serde_json::json!({"changes": changes.len()}),
        );

        serde_json::json!({
            "success": true,
            "level": format!("{:?}", level),
            "changes": changes,
        })
    }

    /// Returns the differences between current values and factory defaults.
    pub fn factory_diff(&self) -> serde_json::Value {
        use crate::factory::FactoryDefaults;

        let factory = match FactoryDefaults::load() {
            Ok(f) => f,
            Err(e) => return serde_json::json!({"error": e}),
        };

        let params = factory.reset_parameters();
        let mut diffs = Vec::new();

        // Compare chemistry
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
                    "current": current[i],
                    "factory": params.baselines[i],
                    "diff": current[i] - params.baselines[i],
                }));
            }
        }

        // Compare parameters
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
                    "param": name,
                    "current": current_val,
                    "factory": factory_val,
                    "diff": current_val - factory_val,
                }));
            }
        }

        // Compare BDNF
        let bdnf_diff = self.grey_matter.bdnf_level - params.bdnf_homeostasis_baseline;
        if bdnf_diff.abs() > 0.001 {
            diffs.push(serde_json::json!({
                "param": "bdnf_level",
                "current": self.grey_matter.bdnf_level,
                "factory": params.bdnf_homeostasis_baseline,
                "diff": bdnf_diff,
            }));
        }

        serde_json::json!({
            "diffs": diffs,
            "total_params": 60,
            "modified_count": diffs.len(),
        })
    }
}
