// =============================================================================
// factory.rs -- Factory-default values and multi-level reset mechanism
// =============================================================================
//
// Purpose:
//   Loads Saphire's original design-time parameter values from
//   factory_defaults.toml (embedded in the binary at compile time).
//   Provides 3+ levels of reset granularity: chemistry-only, parameters-only,
//   senses-only, intuition-only, personal-ethics-only, psychology-only,
//   sleep-only, and full reset.
//
// Dependencies:
//   - toml: TOML file parsing
//   - serde / serde_json: serialization and deserialization
//
// Role in the architecture:
//   Used by the REST API and WebSocket interface to restore factory-default
//   parameters when manual adjustments have destabilized the system.
//   The factory defaults serve as the "ground truth" configuration that
//   the agent can always return to.
// =============================================================================

use serde::{Deserialize, Serialize};

/// The factory_defaults.toml file is embedded into the binary at compile time
/// via `include_str!`, ensuring the defaults are always available regardless
/// of the runtime filesystem.
const FACTORY_DEFAULTS_TOML: &str = include_str!("../factory_defaults.toml");

/// Available reset levels, each targeting a specific subsystem.
///
/// Reset levels are ordered from least disruptive (chemistry-only) to most
/// disruptive (full reset). The choice of level depends on the severity of
/// the destabilization and the amount of learned state the operator wishes
/// to preserve.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResetLevel {
    /// Chemistry-only reset: restores the 7 neurotransmitter molecules to
    /// their homeostatic baselines. Does NOT affect memories, personality,
    /// or identity. This is the least disruptive reset level.
    ChemistryOnly,

    /// Parameters-only reset: restores all operational parameters (thresholds,
    /// intervals, LLM settings, etc.) to factory values. Does NOT affect
    /// memories or identity.
    ParametersOnly,

    /// Senses-only reset: restores the acuity levels of all 5 senses
    /// (reading, listening, contact, taste, ambiance) to their initial values
    /// and resets emergent seed stimulation. Already-germinated senses are
    /// PRESERVED (germination is an irreversible developmental milestone).
    SensesOnly,

    /// Intuition + premonition reset: restores acuity and accuracy to initial
    /// values. Clears active predictions but preserves learned patterns
    /// (the agent retains its accumulated pattern-recognition experience).
    IntuitionOnly,

    /// Personal ethics reset: deactivates all personally formulated ethical
    /// principles (Layer 2). Does NOT affect Layer 0 (Swiss humanitarian law)
    /// or Layer 1 (Asimov's Laws of Robotics), which are immutable.
    PersonalEthicsOnly,

    /// Psychology reset: reinitializes all psychological framework states
    /// (Freud id/ego/superego balance, Maslow hierarchy position, Toltec
    /// agreements, Jung archetypes, EQ emotional quotient, Flow state) to
    /// their initial values. Does NOT affect memories, chemistry, or identity.
    PsychologyOnly,

    /// Sleep reset: resets sleep pressure, fatigue counters, and sleep-wake
    /// cycle state. Does NOT affect sleep history, neural connections formed
    /// during sleep, or subconscious processing state.
    SleepOnly,

    /// Full reset: restores ALL subsystems to factory defaults.
    /// WARNING: erases episodic memories and reinitializes the identity profile.
    /// The following are PRESERVED even during a full reset:
    ///   - Founding memories (core identity anchors, immutable)
    ///   - Long-term memory (consolidated knowledge)
    ///   - The VitalSpark (the agent's intrinsic will to exist)
    ///   - The heart's beat_count (lifetime heartbeat counter, never reset)
    FullReset,
}

/// Operational parameters extracted from factory-default values.
///
/// This struct aggregates all tunable parameters across every subsystem,
/// providing a single snapshot of the factory configuration. Used by
/// `FactoryDefaults::reset_parameters()` to restore the agent to its
/// design-time operating point.
#[derive(Debug, Clone, Serialize)]
pub struct FactoryParams {
    // --- Neurochemistry ---
    /// Rate at which neurotransmitter levels return to baseline per cycle
    /// (homeostatic pull strength). Higher values = faster stabilization.
    pub homeostasis_rate: f64,
    /// Factory baseline levels for the 7 neurotransmitters, in order:
    /// [dopamine, cortisol, serotonin, adrenaline, oxytocin, endorphin, noradrenaline].
    pub baselines: [f64; 7],

    // --- Brain consensus ---
    /// Consensus score threshold above which the decision is YES.
    pub threshold_yes: f64,
    /// Consensus score threshold below which the decision is NO.
    pub threshold_no: f64,

    // --- Autonomous thought ---
    /// Interval in seconds between autonomous thought cycles.
    pub thought_interval: u64,

    // --- LLM (Large Language Model) ---
    /// Sampling temperature for LLM text generation (higher = more creative).
    pub temperature: f64,
    /// Maximum token count for LLM responses to user messages.
    pub max_tokens: u32,
    /// Maximum token count for LLM-generated autonomous thoughts.
    pub max_tokens_thought: u32,

    // --- Virtual body ---
    /// Resting heart rate in beats per minute (BPM). The virtual heart rate
    /// fluctuates around this baseline based on arousal and stress levels.
    pub resting_bpm: f64,

    // --- Memory ---
    /// Working memory capacity (number of items). Inspired by Miller's Law
    /// (7 +/- 2 items in human short-term memory).
    pub wm_capacity: usize,
    /// Working memory decay rate per cycle (items lose salience over time).
    pub wm_decay: f64,
    /// Episodic memory decay rate per cycle.
    pub episodic_decay: f64,
    /// Maximum number of episodic memories before oldest are evicted.
    pub episodic_max: usize,
    /// Number of cycles between memory consolidation sweeps (episodic -> long-term).
    pub consolidation_interval: u64,
    /// Minimum salience threshold for a memory to be consolidated to long-term storage.
    pub consolidation_threshold: f64,

    // --- Intuition ---
    /// Initial acuity of the intuition subsystem (ability to detect patterns).
    pub intuition_initial_acuity: f64,
    /// Initial accuracy of intuitive judgments (proportion of correct hunches).
    pub intuition_initial_accuracy: f64,

    // --- Premonition ---
    /// Initial accuracy of predictive anticipation (premonition subsystem).
    pub premonition_initial_accuracy: f64,

    // --- Senses ---
    /// Minimum signal strength for a sense to register a detection event.
    pub senses_detection_threshold: f64,
    /// Initial acuity of the reading sense (text comprehension depth).
    pub reading_initial_acuity: f64,
    /// Initial acuity of the listening sense (auditory pattern recognition).
    pub listening_initial_acuity: f64,
    /// Initial acuity of the contact sense (touch/haptic feedback).
    pub contact_initial_acuity: f64,
    /// Initial acuity of the taste sense (aesthetic/qualitative judgment).
    pub taste_initial_acuity: f64,
    /// Initial acuity of the ambiance sense (environmental mood detection).
    pub ambiance_initial_acuity: f64,

    // --- Ethics ---
    /// Maximum number of personal ethical principles the agent can formulate.
    pub ethics_max_principles: usize,
    /// Cooldown in cycles between formulations of new personal principles.
    pub ethics_cooldown_cycles: u64,

    // --- Neural network learning (vectorial micro-NN) ---
    /// Whether the micro-NN learning subsystem is enabled.
    pub nn_learning_enabled: bool,
    /// Minimum cycles between two learning events (prevents over-fitting).
    pub nn_learning_cooldown: u64,
    /// Maximum number of stored learned weight adjustments.
    pub nn_max_learnings: u64,
    /// Decay rate for learned adjustments (older learnings fade over time).
    pub nn_learning_decay_rate: f64,
    /// Minimum number of triggering conditions before a learning is applied.
    pub nn_min_conditions: u64,
    /// Maximum number of candidate learnings to search per cycle.
    pub nn_learning_search_limit: u64,
    /// Cosine similarity threshold for matching a stimulus to a stored learning.
    pub nn_learning_search_threshold: f64,

    // --- Sleep ---
    /// Whether the sleep/wake cycle subsystem is enabled.
    pub sleep_enabled: bool,
    /// Sleep pressure threshold above which the agent enters voluntary sleep.
    pub sleep_threshold: f64,
    /// Sleep pressure threshold above which sleep is forced (cannot be deferred).
    pub forced_sleep_threshold: f64,

    // --- Subconscious ---
    /// Whether the subconscious processing subsystem is enabled.
    pub subconscious_enabled: bool,
    /// Activation level of subconscious processing while the agent is awake
    /// (subconscious runs at a reduced level during wakefulness).
    pub subconscious_awake_activation: f64,
}

/// Manager for factory-default values.
///
/// Wraps the parsed TOML configuration and provides typed accessor methods
/// for extracting parameter values with fallback defaults.
pub struct FactoryDefaults {
    /// The parsed TOML value tree from factory_defaults.toml.
    config: toml::Value,
}

impl FactoryDefaults {
    /// Loads and parses the embedded factory_defaults.toml file.
    ///
    /// # Returns
    /// - `Ok(FactoryDefaults)`: the parsed factory defaults manager.
    /// - `Err(String)`: a human-readable error if TOML parsing fails.
    pub fn load() -> Result<Self, String> {
        let config: toml::Value = toml::from_str(FACTORY_DEFAULTS_TOML)
            .map_err(|e| format!("Error parsing factory_defaults.toml: {}", e))?;
        Ok(Self { config })
    }

    /// Returns the 7 factory-default neurochemical baselines.
    ///
    /// Order: [dopamine, cortisol, serotonin, adrenaline, oxytocin, endorphin, noradrenaline].
    /// These represent the "resting state" concentrations that the homeostatic
    /// mechanism pulls toward after each perturbation.
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

    /// Returns all factory-default operational parameters as a `FactoryParams` struct.
    ///
    /// This is the primary method used during a parameters-only or full reset
    /// to restore every tunable value to its design-time setting.
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
            // Intuition subsystem
            intuition_initial_acuity: self.get_f64("intuition.initial_acuity", 0.2),
            intuition_initial_accuracy: self.get_f64("intuition.initial_accuracy", 0.5),
            // Premonition subsystem
            premonition_initial_accuracy: self.get_f64("premonition.initial_accuracy", 0.5),
            // Senses subsystem
            senses_detection_threshold: self.get_f64("senses.detection_threshold", 0.1),
            reading_initial_acuity: self.get_f64("senses.reading.initial_acuity", 0.3),
            listening_initial_acuity: self.get_f64("senses.listening.initial_acuity", 0.3),
            contact_initial_acuity: self.get_f64("senses.contact.initial_acuity", 0.3),
            taste_initial_acuity: self.get_f64("senses.taste.initial_acuity", 0.3),
            ambiance_initial_acuity: self.get_f64("senses.ambiance.initial_acuity", 0.2),
            // Ethics subsystem
            ethics_max_principles: self.get_u64("ethics.personal.max_personal_principles", 20) as usize,
            ethics_cooldown_cycles: self.get_u64("ethics.personal.formulation_cooldown_cycles", 100),
            // Neural network learning (vectorial micro-NN)
            nn_learning_enabled: self.get_bool("nn_learning.enabled", true),
            nn_learning_cooldown: self.get_u64("nn_learning.cooldown_cycles", 15),
            nn_max_learnings: self.get_u64("nn_learning.max_learnings", 200),
            nn_learning_decay_rate: self.get_f64("nn_learning.decay_rate", 0.02),
            nn_min_conditions: self.get_u64("nn_learning.min_conditions", 2),
            nn_learning_search_limit: self.get_u64("nn_learning.search_limit", 3),
            nn_learning_search_threshold: self.get_f64("nn_learning.search_threshold", 0.35),
            // Sleep subsystem
            sleep_enabled: self.get_bool("sleep.enabled", true),
            sleep_threshold: self.get_f64("sleep.sleep_threshold", 0.7),
            forced_sleep_threshold: self.get_f64("sleep.forced_sleep_threshold", 0.95),
            // Subconscious subsystem
            subconscious_enabled: self.get_bool("subconscious.enabled", true),
            subconscious_awake_activation: self.get_f64("subconscious.awake_activation", 0.2),
        }
    }

    /// Returns the entire factory_defaults.toml content as a JSON value.
    ///
    /// Used by the REST API to expose the full factory configuration to
    /// the web dashboard for inspection.
    pub fn as_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.config).unwrap_or_default()
    }

    /// Navigates the TOML value tree using a dot-separated path and returns
    /// the value as an `f64`, or the provided default if the path is missing
    /// or the value cannot be converted.
    ///
    /// # Parameters
    /// - `path`: dot-separated key path (e.g., "chemistry.baselines.dopamine").
    /// - `default`: fallback value if the path does not exist or is not a float.
    ///
    /// # Returns
    /// The floating-point value at the specified path, or `default`.
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

    /// Navigates the TOML value tree using a dot-separated path and returns
    /// the value as a `bool`, or the provided default if the path is missing.
    ///
    /// # Parameters
    /// - `path`: dot-separated key path (e.g., "sleep.enabled").
    /// - `default`: fallback value if the path does not exist or is not a boolean.
    ///
    /// # Returns
    /// The boolean value at the specified path, or `default`.
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

    /// Navigates the TOML value tree using a dot-separated path and returns
    /// the value as a `u64`, or the provided default if the path is missing.
    ///
    /// # Parameters
    /// - `path`: dot-separated key path (e.g., "thought.interval_seconds").
    /// - `default`: fallback value if the path does not exist or is not an integer.
    ///
    /// # Returns
    /// The unsigned integer value at the specified path, or `default`.
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

    /// Extracts a floating-point value from a TOML table by key name.
    ///
    /// # Parameters
    /// - `table`: the TOML table (object) to look up.
    /// - `key`: the key name within the table.
    /// - `default`: fallback value if the key is missing or not a float.
    ///
    /// # Returns
    /// The floating-point value for the given key, or `default`.
    fn val(&self, table: &toml::Value, key: &str, default: f64) -> f64 {
        table.get(key)
            .and_then(|v| v.as_float())
            .unwrap_or(default)
    }
}
