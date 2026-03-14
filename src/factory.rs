// =============================================================================
// factory.rs — Factory defaults and reset mechanism
// =============================================================================
//
// Role: Loads the original design values of Saphire from
//        factory_defaults.toml (embedded in the binary). Provides 9 reset
//  levels: chemistry, parameters, senses, intuition, ethics, psychology,
//        sleep, biology, and full reset.
//
// Dependencies:
//   - toml : TOML file parsing
//  - serde/serde_json : serialization
//
// Place in architecture:
//   This module is used by the REST API and WebSocket to restore
//  factory defaults when a manual modification unbalances
//   the system.
// =============================================================================

use serde::{Deserialize, Serialize};

/// The factory_defaults.toml file is embedded in the binary
const FACTORY_DEFAULTS_TOML: &str = include_str!("../factory_defaults.toml");

/// Available reset levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResetLevel {
    /// Reset chemistry only — resets the 7 molecules to baselines.
    /// Does NOT touch memories, personality, or identity.
    ChemistryOnly,
    /// Reset parameters — resets all operating parameters
    /// to factory defaults. Does NOT touch memories or identity.
    ParametersOnly,
    /// Reset senses — resets the 5 senses' acuities to initial values
    /// and reinitializes emergent seed stimulation.
    /// Germinated senses are PRESERVED.
    SensesOnly,
    /// Reset intuition + premonition — resets acuity and accuracy to initial values.
    /// Clears active predictions but not learned patterns.
    IntuitionOnly,
    /// Reset personal ethics — deactivates all personal principles.
    /// Does NOT touch layers 0 (Swiss law) and 1 (Asimov).
    PersonalEthicsOnly,
    /// Reset psychology — reinitializes all psychological frameworks
    /// (Freud, Maslow, Toltec, Jung, EQ, Flow) to initial values.
    /// Does NOT touch memories, chemistry, or identity.
    PsychologyOnly,
    /// Reset sleep — resets sleep pressure and fatigue counters.
    /// Does NOT touch: sleep_history, neural_connections, subconscious.
    SleepOnly,
    /// Reset biology — resets receptors (sensitivity=1.0, tolerance=0.0),
    /// BDNF (0.5 baseline), and grey matter to default values.
    /// Does NOT touch memories, current chemistry, or identity.
    BiologyReset,
    /// Full reset — resets EVERYTHING to factory defaults.
    /// WARNING: erases episodic memories and reinitializes the profile.
    /// Founding_memories, LTM, and the spark are PRESERVED.
    /// The heart's beat_count is NEVER reset to zero.
    FullReset,
}

/// Operating parameters extracted from factory defaults
#[derive(Debug, Clone, Serialize)]
pub struct FactoryParams {
    // Chemistry
    pub homeostasis_rate: f64,
    pub baselines: [f64; 7],
    // Brain
    pub threshold_yes: f64,
    pub threshold_no: f64,
    // Thought
    pub thought_interval: u64,
    // LLM
    pub temperature: f64,
    pub max_tokens: u32,
    pub max_tokens_thought: u32,
    // Body
    pub resting_bpm: f64,
    // Memory
    pub wm_capacity: usize,
    pub wm_decay: f64,
    pub episodic_decay: f64,
    pub episodic_max: usize,
    pub consolidation_interval: u64,
    pub consolidation_threshold: f64,
    // Intuition
    pub intuition_initial_acuity: f64,
    pub intuition_initial_accuracy: f64,
    // Premonition
    pub premonition_initial_accuracy: f64,
    // Senses
    pub senses_detection_threshold: f64,
    pub reading_initial_acuity: f64,
    pub listening_initial_acuity: f64,
    pub contact_initial_acuity: f64,
    pub taste_initial_acuity: f64,
    pub ambiance_initial_acuity: f64,
    // Ethics
    pub ethics_max_principles: usize,
    pub ethics_cooldown_cycles: u64,
    // Vector learnings (nn_learning)
    pub nn_learning_enabled: bool,
    pub nn_learning_cooldown: u64,
    pub nn_max_learnings: u64,
    pub nn_learning_decay_rate: f64,
    pub nn_min_conditions: u64,
    pub nn_learning_search_limit: u64,
    pub nn_learning_search_threshold: f64,
    // Sleep
    pub sleep_enabled: bool,
    pub sleep_threshold: f64,
    pub forced_sleep_threshold: f64,
    // Subconscious
    pub subconscious_enabled: bool,
    pub subconscious_awake_activation: f64,
    // Receptors
    pub receptor_adaptation_rate: f64,
    pub receptor_recovery_rate: f64,
    // BDNF
    pub bdnf_dopamine_weight: f64,
    pub bdnf_novelty_bonus: f64,
    pub bdnf_flow_state_bonus: f64,
    pub bdnf_cortisol_penalty_weight: f64,
    pub bdnf_cortisol_penalty_threshold: f64,
    pub bdnf_homeostasis_rate: f64,
    pub bdnf_homeostasis_baseline: f64,
    pub bdnf_consolidation_floor: f64,
    pub bdnf_consolidation_range: f64,
    pub bdnf_connectome_boost_threshold: f64,
    pub bdnf_connectome_boost_factor: f64,
}

/// Factory defaults manager
pub struct FactoryDefaults {
    config: toml::Value,
}

impl FactoryDefaults {
    /// Loads factory defaults from the embedded TOML
    pub fn load() -> Result<Self, String> {
        let config: toml::Value = toml::from_str(FACTORY_DEFAULTS_TOML)
            .map_err(|e| format!("Erreur parsing factory_defaults.toml: {}", e))?;
        Ok(Self { config })
    }

    /// Returns the 7 factory chemical baselines
    /// Order: [dopamine, cortisol, serotonin, adrenaline, oxytocin, endorphin, noradrenaline]
    pub fn reset_chemistry(&self) -> [f64; 7] {
        let b = &self.config["chemistry"]["baselines"];
        [
            self.val(b, "dopamine", 0.50),
            self.val(b, "cortisol", 0.15),
            self.val(b, "serotonin", 0.60),
            self.val(b, "adrenaline", 0.10),
            self.val(b, "oxytocin", 0.40),
            self.val(b, "endorphin", 0.40),
            self.val(b, "noradrenaline", 0.40),
        ]
    }

    /// Returns all factory parameters
    pub fn reset_parameters(&self) -> FactoryParams {
        let baselines = self.reset_chemistry();
        FactoryParams {
            homeostasis_rate: self.get_f64("chemistry.homeostasis.rate", 0.03),
            baselines,
            threshold_yes: self.get_f64("brain.consensus.threshold_yes", 0.33),
            threshold_no: self.get_f64("brain.consensus.threshold_no", -0.33),
            thought_interval: self.get_u64("thought.interval_seconds", 15),
            temperature: self.get_f64("llm.temperature", 0.7),
            max_tokens: self.get_u64("llm.max_tokens", 800) as u32,
            max_tokens_thought: self.get_u64("llm.max_tokens_thought", 600) as u32,
            resting_bpm: self.get_f64("body.resting_bpm", 72.0),
            wm_capacity: self.get_u64("memory.working.capacity", 7) as usize,
            wm_decay: self.get_f64("memory.working.decay_rate", 0.05),
            episodic_decay: self.get_f64("memory.episodic.decay_rate", 0.05),
            episodic_max: self.get_u64("memory.episodic.max_count", 500) as usize,
            consolidation_interval: self.get_u64("memory.consolidation.interval_cycles", 50),
            consolidation_threshold: self.get_f64("memory.consolidation.threshold", 0.35),
            // Intuition
            intuition_initial_acuity: self.get_f64("intuition.initial_acuity", 0.2),
            intuition_initial_accuracy: self.get_f64("intuition.initial_accuracy", 0.5),
            // Premonition
            premonition_initial_accuracy: self.get_f64("premonition.initial_accuracy", 0.5),
            // Senses
            senses_detection_threshold: self.get_f64("senses.detection_threshold", 0.1),
            reading_initial_acuity: self.get_f64("senses.reading.initial_acuity", 0.3),
            listening_initial_acuity: self.get_f64("senses.listening.initial_acuity", 0.3),
            contact_initial_acuity: self.get_f64("senses.contact.initial_acuity", 0.3),
            taste_initial_acuity: self.get_f64("senses.taste.initial_acuity", 0.3),
            ambiance_initial_acuity: self.get_f64("senses.ambiance.initial_acuity", 0.2),
            // Ethics
            ethics_max_principles: self.get_u64("ethics.personal.max_personal_principles", 20) as usize,
            ethics_cooldown_cycles: self.get_u64("ethics.personal.formulation_cooldown_cycles", 100),
            // Vector learnings
            nn_learning_enabled: self.get_bool("nn_learning.enabled", true),
            nn_learning_cooldown: self.get_u64("nn_learning.cooldown_cycles", 15),
            nn_max_learnings: self.get_u64("nn_learning.max_learnings", 200),
            nn_learning_decay_rate: self.get_f64("nn_learning.decay_rate", 0.02),
            nn_min_conditions: self.get_u64("nn_learning.min_conditions", 2),
            nn_learning_search_limit: self.get_u64("nn_learning.search_limit", 3),
            nn_learning_search_threshold: self.get_f64("nn_learning.search_threshold", 0.35),
            // Sleep
            sleep_enabled: self.get_bool("sleep.enabled", true),
            sleep_threshold: self.get_f64("sleep.sleep_threshold", 0.7),
            forced_sleep_threshold: self.get_f64("sleep.forced_sleep_threshold", 0.95),
            // Subconscious
            subconscious_enabled: self.get_bool("subconscious.enabled", true),
            subconscious_awake_activation: self.get_f64("subconscious.awake_activation", 0.2),
            // Receptors
            receptor_adaptation_rate: self.get_f64("receptors.adaptation_rate", 0.02),
            receptor_recovery_rate: self.get_f64("receptors.recovery_rate", 0.005),
            // BDNF
            bdnf_dopamine_weight: self.get_f64("bdnf.dopamine_weight", 0.15),
            bdnf_novelty_bonus: self.get_f64("bdnf.novelty_bonus", 0.1),
            bdnf_flow_state_bonus: self.get_f64("bdnf.flow_state_bonus", 0.1),
            bdnf_cortisol_penalty_weight: self.get_f64("bdnf.cortisol_penalty_weight", 0.4),
            bdnf_cortisol_penalty_threshold: self.get_f64("bdnf.cortisol_penalty_threshold", 0.6),
            bdnf_homeostasis_rate: self.get_f64("bdnf.homeostasis_rate", 0.01),
            bdnf_homeostasis_baseline: self.get_f64("bdnf.homeostasis_baseline", 0.5),
            bdnf_consolidation_floor: self.get_f64("bdnf.consolidation_floor", 0.8),
            bdnf_consolidation_range: self.get_f64("bdnf.consolidation_range", 0.4),
            bdnf_connectome_boost_threshold: self.get_f64("bdnf.connectome_boost_threshold", 0.4),
            bdnf_connectome_boost_factor: self.get_f64("bdnf.connectome_boost_factor", 0.5),
        }
    }

    /// Returns the entire content of factory_defaults.toml as JSON
    pub fn as_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.config).unwrap_or_default()
    }

    /// Navigate the TOML by dotted path (e.g.: "chemistry.baselines.dopamine")
    pub fn get_f64(&self, path: &str, default: f64) -> f64 {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.config;
        for part in &parts {
            current = match current.get(part) {
                Some(v) => v,
                None => return default,
            };
        }
        current.as_float().unwrap_or(default)
    }

    fn get_bool(&self, path: &str, default: bool) -> bool {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.config;
        for part in &parts {
            current = match current.get(part) {
                Some(v) => v,
                None => return default,
            };
        }
        current.as_bool().unwrap_or(default)
    }

    fn get_u64(&self, path: &str, default: u64) -> u64 {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.config;
        for part in &parts {
            current = match current.get(part) {
                Some(v) => v,
                None => return default,
            };
        }
        current.as_integer().unwrap_or(default as i64) as u64
    }

    fn val(&self, table: &toml::Value, key: &str, default: f64) -> f64 {
        table.get(key)
            .and_then(|v| v.as_float())
            .unwrap_or(default)
    }
}
