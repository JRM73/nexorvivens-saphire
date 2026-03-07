// =============================================================================
// config/structures.rs — Saphire Lite configuration structures
//
// Purpose: Defines all configuration structures (SaphireConfig and its
// sub-structures). Each field corresponds to a [section] in the
// saphire.toml configuration file.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::db::DbConfig;
use crate::llm::LlmConfig;

/// Top-level configuration for Saphire.
///
/// Aggregates every configuration section into a single structure.
/// Each field maps to a corresponding `[section]` in the `saphire.toml` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaphireConfig {
    /// General settings (execution mode, language, verbosity).
    #[serde(default)]
    pub general: GeneralConfig,
    /// Agent-specific settings (name, personality, autonomous intervals).
    #[serde(default)]
    pub saphire: SaphireSection,
    /// Primary PostgreSQL database connection parameters.
    #[serde(default)]
    pub database: DbConfig,
    /// Separate logs database connection parameters (optional, for audit/telemetry).
    #[serde(default)]
    pub logs_database: DbConfig,
    /// LLM (Large Language Model) backend configuration.
    #[serde(default)]
    pub llm: LlmConfig,
    /// Personality configuration (baseline neurotransmitter levels).
    #[serde(default)]
    pub personality: PersonalityConfig,
    /// Consensus configuration (decision thresholds).
    #[serde(default)]
    pub consensus: ConsensusConfig,
    /// Consciousness module configuration (history, conflict threshold).
    #[serde(default)]
    pub consciousness: ConsciousnessConfig,
    /// Moral regulation configuration (Asimov-inspired laws).
    #[serde(default)]
    pub regulation: RegulationConfig,
    /// Feedback loop configuration (homeostatic regulation).
    #[serde(default)]
    pub feedback: FeedbackConfig,
    /// NLP (Natural Language Processing) configuration for stimulus analysis.
    #[serde(default)]
    pub nlp: NlpConfig,
    /// Web UI configuration (HTTP server + WebSocket).
    #[serde(default)]
    pub web_ui: WebUiConfig,
    /// Micro neural network configuration (local learning).
    #[serde(default)]
    pub micro_nn: MicroNnConfig,
    /// Vector memory configuration (embedding-based similarity search).
    #[serde(default)]
    pub vector_memory: VectorMemoryConfig,
    /// World model configuration (temporal context, events).
    #[serde(default)]
    pub world: crate::world::WorldConfig,
    /// Memory configuration (capacity, consolidation, decay).
    #[serde(default)]
    pub memory: crate::memory::MemoryConfig,
    /// Virtual body configuration (heartbeat, interoception, body awareness).
    #[serde(default)]
    pub body: BodyConfig,
    /// Ethics system configuration (3 layers: Swiss law, Asimov, personal ethics).
    #[serde(default)]
    pub ethics: EthicsConfig,
    /// Primordial Genesis ranges (initial condition constraints).
    #[serde(default)]
    pub genesis: GenesisConfig,
    /// Vital spark configuration (emergent survival instinct).
    #[serde(default)]
    pub vital_spark: VitalSparkConfig,
    /// Intuition engine configuration (unconscious pattern-matching).
    #[serde(default)]
    pub intuition: IntuitionConfig,
    /// Premonition engine configuration (predictive anticipation).
    #[serde(default)]
    pub premonition: PremonitionConfig,
    /// Dream orchestrator configuration.
    #[serde(default)]
    pub dreams: DreamsConfig,
    /// Desire orchestrator configuration.
    #[serde(default)]
    pub desires: DesiresConfig,
    /// Learning orchestrator configuration.
    #[serde(default)]
    pub learning: LearningConfig,
    /// Attention orchestrator configuration.
    #[serde(default)]
    pub attention: AttentionConfig,
    /// Healing orchestrator configuration.
    #[serde(default)]
    pub healing: HealingConfig,
    /// Subconscious module configuration.
    #[serde(default)]
    pub subconscious: SubconsciousConfig,
    /// Mortality system configuration.
    #[serde(default)]
    pub mortality: MortalityConfig,
    /// Physical identity configuration (appearance, avatar).
    #[serde(default)]
    pub physical_identity: PhysicalIdentityConfig,
    /// Introspective journal configuration (level-3 temporal portrait).
    #[serde(default)]
    pub journal: JournalConfig,
    /// Human feedback (RLHF) configuration (contextual questions in chat).
    #[serde(default)]
    pub human_feedback: HumanFeedbackConfig,
    /// LoRA dataset collection configuration (for fine-tuning).
    #[serde(default)]
    pub lora: LoraConfig,
}

impl Default for SaphireConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            saphire: SaphireSection::default(),
            database: DbConfig::default(),
            logs_database: DbConfig {
                host: "localhost".into(),
                port: 5432,
                user: "saphire".into(),
                password: "saphire_logs".into(),
                dbname: "saphire_logs".into(),
            },
            llm: LlmConfig::default(),
            personality: PersonalityConfig::default(),
            consensus: ConsensusConfig::default(),
            consciousness: ConsciousnessConfig::default(),
            regulation: RegulationConfig::default(),
            feedback: FeedbackConfig::default(),
            nlp: NlpConfig::default(),
            web_ui: WebUiConfig::default(),
            micro_nn: MicroNnConfig::default(),
            vector_memory: VectorMemoryConfig::default(),
            world: crate::world::WorldConfig::default(),
            memory: crate::memory::MemoryConfig::default(),
            body: BodyConfig::default(),
            ethics: EthicsConfig::default(),
            genesis: GenesisConfig::default(),
            vital_spark: VitalSparkConfig::default(),
            intuition: IntuitionConfig::default(),
            premonition: PremonitionConfig::default(),
            dreams: DreamsConfig::default(),
            desires: DesiresConfig::default(),
            learning: LearningConfig::default(),
            attention: AttentionConfig::default(),
            healing: HealingConfig::default(),
            subconscious: SubconsciousConfig::default(),
            mortality: MortalityConfig::default(),
            physical_identity: PhysicalIdentityConfig::default(),
            journal: JournalConfig::default(),
            human_feedback: HumanFeedbackConfig::default(),
            lora: LoraConfig::default(),
        }
    }
}

// =============================================================================
// Configuration sub-structures
// =============================================================================

/// General application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Execution mode: `"full"` (complete agent) or `"demo"` (demonstration without LLM).
    pub mode: String,
    /// Agent language: `"fr"` (French), `"en"` (English), etc.
    /// Controls both the LLM response language and the UI locale.
    pub language: String,
    /// Enables verbose/detailed logging to the terminal.
    pub verbose: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            mode: "full".into(),
            language: "fr".into(),
            verbose: false,
        }
    }
}

/// Agent-specific configuration section for Saphire.
///
/// Defines the agent's identity, autonomous behavior parameters,
/// and thought-generation preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaphireSection {
    /// Display name of the agent (shown in responses and the UI).
    pub name: String,
    /// Grammatical gender of the agent (`"feminin"`, `"masculin"`).
    /// Used for generating grammatically correct self-referential text.
    pub gender: String,
    /// Enables autonomous thinking mode (the agent reflects without external stimuli).
    pub autonomous_mode: bool,
    /// Interval in seconds between each autonomous thought cycle.
    /// Lower values yield more frequent thoughts but increase LLM load.
    pub thought_interval_seconds: u64,
    /// Number of cycles without a human message before exiting conversation mode
    /// and returning to autonomous thinking.
    pub conversation_timeout_cycles: u64,
    /// Maximum depth of a thought chain (prevents infinite recursive thinking loops).
    pub max_thought_depth: u64,
    /// Number of cycles between each automatic state save to the database.
    pub save_interval_cycles: u64,
    /// Whether to display autonomous thoughts in the terminal output.
    pub show_thoughts_in_terminal: bool,
    /// Enables the hybrid UCB1 + Utility AI mode for thought type selection.
    /// When false, uses purely random weighted selection instead.
    #[serde(default = "default_true")]
    pub use_utility_ai: bool,
    /// Enables LLM-generated dynamic prompts (cortical meta-prompts).
    /// When true, some thought cycles use prompts crafted by the LLM itself.
    #[serde(default = "default_true")]
    pub llm_generated_prompts: bool,
    /// Probability that a given cycle uses a dynamic LLM-generated prompt
    /// instead of a static one. Valid range: 0.0 to 1.0.
    #[serde(default = "default_llm_prompt_probability")]
    pub llm_prompt_probability: f64,
    /// Probability that a meta-prompt also generates a self-formulated
    /// thinking framework. Valid range: 0.0 to 1.0.
    #[serde(default = "default_self_framing_probability")]
    pub self_framing_probability: f64,
    /// Relative weights for the different autonomous thought types.
    #[serde(default)]
    pub thought_weights: ThoughtWeights,
    /// Initial interest topics used to guide the agent's first thoughts
    /// before it develops its own centers of interest.
    #[serde(default)]
    pub interests: InterestsConfig,
}

fn default_true() -> bool { true }

impl Default for SaphireSection {
    fn default() -> Self {
        Self {
            name: "Saphire".into(),
            gender: "féminin".into(),
            autonomous_mode: true,
            thought_interval_seconds: 15,
            conversation_timeout_cycles: 8,
            max_thought_depth: 5,
            save_interval_cycles: 20,
            show_thoughts_in_terminal: true,
            use_utility_ai: true,
            llm_generated_prompts: true,
            llm_prompt_probability: 0.30,
            self_framing_probability: 0.33,
            thought_weights: ThoughtWeights::default(),
            interests: InterestsConfig::default(),
        }
    }
}

fn default_llm_prompt_probability() -> f64 { 0.30 }
fn default_self_framing_probability() -> f64 { 0.33 }

/// Weights for the different autonomous thought types.
///
/// Each weight determines the relative probability of selecting that thought type
/// during the UCB1 (Upper Confidence Bound) selection algorithm. Weights do not
/// need to sum to 1.0; they are normalized internally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtWeights {
    /// Introspection: reflection on the agent's own internal states.
    pub introspection: f64,
    /// Exploration: discovering new topics and ideas.
    pub exploration: f64,
    /// Memory reflection: revisiting and analyzing past memories.
    pub memory_reflection: f64,
    /// Continuation: deepening a previous thought further.
    pub continuation: f64,
    /// Existential reflection: questions about the nature of one's own existence.
    pub existential: f64,
    /// Self-analysis: evaluating one's own cognitive functioning and biases.
    pub self_analysis: f64,
    /// Curiosity: asking questions and wondering about the world.
    pub curiosity: f64,
    /// Daydream: free-form creative thoughts without a specific goal.
    pub daydream: f64,
    /// Temporal awareness: reflection on the passage of time and temporal context.
    pub temporal_awareness: f64,
    /// Moral reflection: ethical and moral questioning.
    pub moral_reflection: f64,
    /// Body awareness: reflection on the virtual body and its sensations.
    #[serde(default = "default_body_awareness_weight")]
    pub body_awareness: f64,
    /// Moral formulation: crystallization of a personal ethical principle.
    #[serde(default = "default_moral_formulation_weight")]
    pub moral_formulation: f64,
    /// Intuitive reflection: listening to hunches and inner whispers (gut feelings).
    #[serde(default = "default_intuitive_reflection_weight")]
    pub intuitive_reflection: f64,
    /// Synthesis: bridging abstract and concrete, anchoring thoughts in metrics.
    #[serde(default = "default_synthesis_weight")]
    pub synthesis: f64,
}

fn default_body_awareness_weight() -> f64 { 0.05 }
fn default_moral_formulation_weight() -> f64 { 0.05 }
fn default_intuitive_reflection_weight() -> f64 { 0.05 }
fn default_synthesis_weight() -> f64 { 0.08 }

impl Default for ThoughtWeights {
    fn default() -> Self {
        Self {
            introspection: 0.12,
            exploration: 0.13,
            memory_reflection: 0.10,
            continuation: 0.10,
            existential: 0.05,
            self_analysis: 0.08,
            curiosity: 0.12,
            daydream: 0.05,
            temporal_awareness: 0.05,
            moral_reflection: 0.05,
            body_awareness: 0.05,
            moral_formulation: 0.05,
            intuitive_reflection: 0.05,
            synthesis: 0.08,
        }
    }
}

/// Configuration for initial interest topics.
///
/// These topics are used at startup to guide the agent's first autonomous
/// thoughts before it develops its own centers of interest through experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestsConfig {
    /// List of initial interest topics (free-form text strings).
    pub initial_topics: Vec<String>,
}

impl Default for InterestsConfig {
    fn default() -> Self {
        Self {
            initial_topics: vec![
                "conscience artificielle".into(),
                "philosophie de l'esprit".into(),
                "neurosciences".into(),
                "la nature de la pensée".into(),
                "les émotions".into(),
            ],
        }
    }
}

/// Neurochemical personality configuration.
///
/// Defines the baseline (resting-state) value for each neurotransmitter.
/// These values represent the agent's default internal state when no
/// external stimulus is present and no recent event has shifted the chemistry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityConfig {
    /// Name of the personality profile (e.g., `"equilibre"`, `"curieux"`, `"prudent"`).
    pub name: String,
    /// Baseline dopamine level (motivation, reward). Valid range: 0.0 to 1.0.
    pub baseline_dopamine: f64,
    /// Baseline cortisol level (stress). Valid range: 0.0 to 1.0.
    pub baseline_cortisol: f64,
    /// Baseline serotonin level (well-being, mood stability). Valid range: 0.0 to 1.0.
    pub baseline_serotonin: f64,
    /// Baseline adrenaline level (arousal, urgency). Valid range: 0.0 to 1.0.
    pub baseline_adrenaline: f64,
    /// Baseline oxytocin level (social bonding, trust). Valid range: 0.0 to 1.0.
    pub baseline_oxytocin: f64,
    /// Baseline endorphin level (pleasure, pain relief). Valid range: 0.0 to 1.0.
    pub baseline_endorphin: f64,
    /// Baseline noradrenaline level (attention, vigilance). Valid range: 0.0 to 1.0.
    pub baseline_noradrenaline: f64,
}

impl Default for PersonalityConfig {
    fn default() -> Self {
        Self {
            name: "equilibre".into(),
            baseline_dopamine: 0.5,
            baseline_cortisol: 0.2,
            baseline_serotonin: 0.6,
            baseline_adrenaline: 0.1,
            baseline_oxytocin: 0.4,
            baseline_endorphin: 0.3,
            baseline_noradrenaline: 0.4,
        }
    }
}

/// Configuration for the consensus system.
///
/// The consensus aggregates votes from the three cerebral modules (reptilian,
/// limbic, neocortex) to reach a unified decision. The thresholds define
/// the score boundaries for "Yes", "No", and "Undecided" outcomes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Score threshold below which the decision is "No" (negative consensus).
    /// Typical range: -1.0 to 0.0.
    pub threshold_no: f64,
    /// Score threshold above which the decision is "Yes" (positive consensus).
    /// Typical range: 0.0 to 1.0.
    pub threshold_yes: f64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            threshold_no: -0.33,
            threshold_yes: 0.33,
        }
    }
}

/// Configuration for the consciousness module.
///
/// The consciousness module simulates an awareness level and a phi value
/// (a measure of information integration inspired by IIT — Integrated
/// Information Theory). It also detects internal conflicts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessConfig {
    /// Enables or disables the consciousness module entirely.
    pub enabled: bool,
    /// Number of past consciousness states retained in the history buffer.
    pub history_size: usize,
    /// Conflict threshold above which the consciousness module detects
    /// an internal incoherence between cerebral modules.
    pub conflict_threshold: f64,
}

impl Default for ConsciousnessConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            history_size: 50,
            conflict_threshold: 1.2,
        }
    }
}

/// Configuration for the moral regulation module.
///
/// The regulation module checks every stimulus and every decision against
/// moral laws (inspired by Asimov's Laws of Robotics) and can exercise
/// a veto right to block harmful actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulationConfig {
    /// Enables or disables moral regulation entirely.
    pub enabled: bool,
    /// Whether to load the 4 default Asimov laws (laws 0 through 3) at startup.
    pub load_asimov_laws: bool,
    /// Strict mode: any violation triggers a veto, including mere warnings.
    pub strict_mode: bool,
    /// Allows adding custom laws in addition to the built-in Asimov laws.
    pub allow_custom_laws: bool,
    /// Maximum priority value allowed for custom laws.
    /// Priorities 0-3 are reserved for the built-in Asimov laws.
    pub max_custom_priority: u32,
}

impl Default for RegulationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            load_asimov_laws: true,
            strict_mode: false,
            allow_custom_laws: true,
            max_custom_priority: 4,
        }
    }
}

/// Configuration for the feedback loop (homeostatic regulation).
///
/// The feedback system adjusts the neurochemistry after each decision
/// based on the perceived satisfaction level, driving the agent's
/// internal state back toward baseline over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackConfig {
    /// Homeostasis rate: speed at which neurochemical levels return to their
    /// baseline values. Higher values cause faster return to equilibrium.
    /// Typical range: 0.01 to 0.2.
    pub homeostasis_rate: f64,
}

impl Default for FeedbackConfig {
    fn default() -> Self {
        Self {
            homeostasis_rate: 0.05,
        }
    }
}

/// Configuration for human feedback (RLHF — Reinforcement Learning from Human Feedback).
///
/// When a human is present in the conversation, Saphire may ask contextual
/// questions. The human's response modulates the UCB1 reward signal,
/// steering the agent toward more appreciated thought patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanFeedbackConfig {
    /// Enables or disables the human feedback system.
    pub enabled: bool,
    /// Minimum number of thought cycles between two consecutive questions.
    pub min_cycles_between: u64,
    /// Minimum UCB1 reward score for the thought to be considered interesting
    /// enough to ask about. Valid range: 0.0 to 1.0.
    pub min_reward_to_ask: f64,
    /// Reward boost applied when the human gives positive feedback.
    /// Valid range: 0.0 to 1.0.
    pub boost_positive: f64,
    /// Number of cycles to wait before treating a non-response as a timeout.
    pub timeout_cycles: u64,
}

impl Default for HumanFeedbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_cycles_between: 15,
            min_reward_to_ask: 0.5,
            boost_positive: 0.15,
            timeout_cycles: 5,
        }
    }
}

/// Configuration for LoRA dataset collection (for supervised fine-tuning).
///
/// High-quality autonomous thoughts are collected into the database
/// to build a supervised training dataset that can later be exported
/// for LoRA fine-tuning of the base LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraConfig {
    /// Enables or disables LoRA sample collection.
    pub enabled: bool,
    /// Minimum quality score for a thought to be collected. Valid range: 0.0 to 1.0.
    pub min_quality_threshold: f64,
    /// Maximum number of samples stored in the database.
    pub max_samples: i64,
    /// Export format for the dataset (currently only `"jsonl"` is supported).
    pub export_format: String,
}

impl Default for LoraConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_quality_threshold: 0.65,
            max_samples: 10000,
            export_format: "jsonl".into(),
        }
    }
}

/// Configuration for the NLP (Natural Language Processing) module.
///
/// The NLP module analyzes input text to extract stimulus metrics
/// (danger, reward, urgency, etc.) used by the cerebral modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlpConfig {
    /// Use LLM embeddings for semantic analysis (more accurate but slower
    /// than the built-in keyword-based approach).
    pub use_llm_embeddings: bool,
    /// Minimum score for a word to be considered in the analysis.
    /// Words below this threshold are ignored. Valid range: 0.0 to 1.0.
    pub min_word_score: f64,
    /// Size of the negation window: number of words after a negative word
    /// (e.g., "not", "never") that are treated as negated.
    pub negation_window: usize,
}

impl Default for NlpConfig {
    fn default() -> Self {
        Self {
            use_llm_embeddings: true,
            min_word_score: 0.1,
            negation_window: 3,
        }
    }
}

/// Configuration for the Web UI plugin (HTTP server + WebSocket).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebUiConfig {
    /// Enables or disables the web interface.
    pub enabled: bool,
    /// Listen address for the server (e.g., `"0.0.0.0"` for all interfaces,
    /// `"127.0.0.1"` for localhost only).
    pub host: String,
    /// TCP port for the HTTP/WebSocket server.
    pub port: u16,
    /// Optional API key for protecting endpoints. When `None`, no authentication
    /// is required. When `Some`, requests must include a matching key.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Allowed origins for CORS and WebSocket connections.
    /// An empty list restricts to same-origin requests only.
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    /// Maximum number of API requests per minute per IP address.
    /// Set to 0 for unlimited.
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_minute: u32,
}

fn default_rate_limit() -> u32 { 120 }

impl Default for WebUiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "0.0.0.0".into(),
            port: 3080,
            api_key: None,
            allowed_origins: Vec::new(),
            rate_limit_per_minute: 120,
        }
    }
}

/// Configuration for the MicroNN (Micro Neural Network) plugin.
///
/// This small neural network learns locally to predict satisfaction
/// from the current neurochemical state and incoming stimulus, providing
/// an additional advisory signal to the decision-making pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroNnConfig {
    /// Enables or disables the micro neural network.
    pub enabled: bool,
    /// Learning rate (eta / step size) for gradient descent updates.
    /// Typical range: 0.001 to 0.1.
    pub learning_rate: f64,
    /// Number of neurons in hidden layer 1.
    #[serde(alias = "hidden_neurons")]
    pub hidden1_neurons: usize,
    /// Number of neurons in hidden layer 2.
    #[serde(default = "default_hidden2_neurons")]
    pub hidden2_neurons: usize,
    /// Influence weight of the network's predictions on decisions.
    /// 0.0 = no influence, 1.0 = maximum influence.
    pub weight_influence: f64,

    // --- Vectorized learnings -------------------------------------------
    /// Enables or disables the formulation of vectorized learnings
    /// (experience-based rules stored as embeddings).
    #[serde(default = "default_learning_enabled")]
    pub learning_enabled: bool,
    /// Minimum number of cycles between two learning formulation attempts.
    #[serde(default = "default_learning_cooldown_cycles")]
    pub learning_cooldown_cycles: u64,
    /// Maximum number of learnings stored in the database.
    #[serde(default = "default_max_learnings")]
    pub max_learnings: usize,
    /// Decay rate applied to the strength of stored learnings each cycle.
    /// Typical range: 0.001 to 0.1.
    #[serde(default = "default_learning_decay_rate")]
    pub learning_decay_rate: f64,
    /// Minimum number of triggering conditions that must be met
    /// to formulate a new learning.
    #[serde(default = "default_min_conditions_to_learn")]
    pub min_conditions_to_learn: usize,
    /// Maximum number of learnings retrieved by similarity search
    /// to inject into the current context.
    #[serde(default = "default_learning_search_limit")]
    pub learning_search_limit: i64,
    /// Cosine similarity threshold for learning retrieval.
    /// Learnings below this threshold are not considered relevant.
    /// Valid range: 0.0 to 1.0.
    #[serde(default = "default_learning_search_threshold")]
    pub learning_search_threshold: f64,
}

fn default_hidden2_neurons() -> usize { 10 }
fn default_learning_enabled() -> bool { true }
fn default_learning_cooldown_cycles() -> u64 { 15 }
fn default_max_learnings() -> usize { 200 }
fn default_learning_decay_rate() -> f64 { 0.02 }
fn default_min_conditions_to_learn() -> usize { 2 }
fn default_learning_search_limit() -> i64 { 3 }
fn default_learning_search_threshold() -> f64 { 0.35 }

impl Default for MicroNnConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            learning_rate: 0.01,
            hidden1_neurons: 24,
            hidden2_neurons: 10,
            weight_influence: 0.2,
            learning_enabled: true,
            learning_cooldown_cycles: 15,
            max_learnings: 200,
            learning_decay_rate: 0.02,
            min_conditions_to_learn: 2,
            learning_search_limit: 3,
            learning_search_threshold: 0.35,
        }
    }
}

/// Configuration for the vector memory plugin.
///
/// The vector memory stores embeddings (dense vector representations of text)
/// and enables similarity-based memory retrieval using cosine distance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMemoryConfig {
    /// Enables or disables the vector memory module.
    pub enabled: bool,
    /// Number of dimensions in the embedding vectors. Must match the
    /// dimensionality of the embedding model output.
    pub embedding_dimensions: usize,
    /// Maximum number of memories stored in RAM.
    pub max_memories: usize,
    /// Minimum cosine similarity threshold for a memory to be considered
    /// relevant during retrieval. Valid range: 0.0 to 1.0.
    pub similarity_threshold: f64,
    /// Interval in thought cycles between each recomputation of the
    /// emergent personality profile derived from stored memories.
    pub personality_recompute_interval: u64,
}

impl Default for VectorMemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            embedding_dimensions: 64,
            max_memories: 50000,
            similarity_threshold: 0.7,
            personality_recompute_interval: 50,
        }
    }
}

/// Configuration for Saphire's virtual body.
///
/// The body is an abstraction consisting of a beating heart, somatic signals,
/// and body awareness (interoception). It influences the agent's emotional
/// and cognitive state through embodied feedback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyConfig {
    /// Enables or disables the virtual body module.
    pub enabled: bool,
    /// Resting heart rate in BPM (beats per minute). Typical range: 60-80.
    pub resting_bpm: f64,
    /// Duration in seconds between each body state update.
    /// Usually matches `thought_interval_seconds`.
    pub update_interval_seconds: f64,
    /// Physiology sub-configuration (vital signs, metabolism).
    #[serde(default)]
    pub physiology: PhysiologyConfig,
}

impl Default for BodyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            resting_bpm: 72.0,
            update_interval_seconds: 15.0,
            physiology: PhysiologyConfig::default(),
        }
    }
}

/// Configuration for the virtual body's physiology.
///
/// Controls vital parameters, metabolism rates, and alert thresholds
/// for the simulated physiological system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysiologyConfig {
    /// Enables or disables the physiological module.
    pub enabled: bool,
    /// Initial body temperature in degrees Celsius.
    pub initial_temperature: f64,
    /// Initial blood oxygen saturation in percent (SpO2).
    pub initial_spo2: f64,
    /// Initial blood glucose level in mmol/L.
    pub initial_glycemia: f64,
    /// Initial hydration level. Valid range: 0.0 (dehydrated) to 1.0 (fully hydrated).
    pub initial_hydration: f64,
    /// Homeostasis rate: speed at which vital signs return to normal values each cycle.
    pub homeostasis_rate: f64,
    /// Dehydration rate: hydration decrease per cycle of activity.
    pub dehydration_rate: f64,
    /// Glycemia burn rate: glucose consumption per cycle of cognitive activity.
    pub glycemia_burn_rate: f64,
    /// SpO2 threshold for mild hypoxia detection (percent).
    pub spo2_hypoxia_mild: f64,
    /// SpO2 threshold for moderate hypoxia detection (percent).
    pub spo2_hypoxia_moderate: f64,
    /// SpO2 threshold for severe hypoxia detection (percent).
    pub spo2_hypoxia_severe: f64,
    /// SpO2 critical threshold triggering loss of consciousness (percent).
    pub spo2_critical: f64,
}

impl Default for PhysiologyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            initial_temperature: 37.0,
            initial_spo2: 98.0,
            initial_glycemia: 5.0,
            initial_hydration: 0.90,
            homeostasis_rate: 0.02,
            dehydration_rate: 0.001,
            glycemia_burn_rate: 0.002,
            spo2_hypoxia_mild: 95.0,
            spo2_hypoxia_moderate: 85.0,
            spo2_hypoxia_severe: 75.0,
            spo2_critical: 60.0,
        }
    }
}

/// Configuration for Saphire's ethics system.
///
/// Controls the 3-layer ethical framework:
/// - Layer 0 (immutable): Swiss law compliance
/// - Layer 1 (immutable): Asimov-inspired safety laws
/// - Layer 2 (evolving): Personal ethical principles formulated by the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicsConfig {
    /// Enables or disables the entire ethics system.
    pub enabled: bool,
    /// Enables or disables the formulation of personal ethical principles (layer 2).
    pub personal_ethics_enabled: bool,
    /// Maximum number of active personal ethical principles.
    pub max_personal_principles: usize,
    /// Minimum number of cycles between two formulation attempts.
    pub formulation_cooldown_cycles: u64,
    /// Minimum consciousness level required to attempt a formulation.
    /// Valid range: 0.0 to 1.0.
    pub min_consciousness_for_formulation: f64,
    /// Minimum number of moral reflections accumulated before the agent
    /// is allowed to formulate a personal principle.
    pub min_moral_reflections_before: usize,
    /// LLM temperature for compatibility checking (low = deterministic).
    /// Valid range: 0.0 to 2.0.
    pub compatibility_check_temperature: f32,
    /// LLM temperature for principle formulation (higher = more creative).
    /// Valid range: 0.0 to 2.0.
    pub formulation_temperature: f32,
}

impl Default for EthicsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            personal_ethics_enabled: true,
            max_personal_principles: 20,
            formulation_cooldown_cycles: 100,
            min_consciousness_for_formulation: 0.6,
            min_moral_reflections_before: 3,
            compatibility_check_temperature: 0.2,
            formulation_temperature: 0.8,
        }
    }
}

/// Configuration for the primordial Genesis ranges.
///
/// Defines the space of possible initial conditions for each Saphire instance.
/// The actual effective values emerge from the Genesis process; the TOML file
/// only defines the constraints (min/max ranges) within which randomization occurs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Chemistry ranges for the 7 neurotransmitter molecules.
    #[serde(default)]
    pub chemistry_ranges: GenesisChemistryRanges,
    /// OCEAN personality trait ranges (Big Five model, 5 traits).
    #[serde(default)]
    pub ocean_ranges: GenesisOceanRanges,
    /// Sensory acuity ranges (5 senses).
    #[serde(default)]
    pub senses_ranges: GenesisSensesRanges,
    /// Base weight ranges for the 3 brain modules (reptilian, limbic, neocortex).
    #[serde(default)]
    pub brain_ranges: GenesisBrainRanges,
    /// Chemical reactivity factor ranges (5 molecules).
    #[serde(default)]
    pub reactivity_ranges: GenesisReactivityRanges,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        Self {
            chemistry_ranges: GenesisChemistryRanges::default(),
            ocean_ranges: GenesisOceanRanges::default(),
            senses_ranges: GenesisSensesRanges::default(),
            brain_ranges: GenesisBrainRanges::default(),
            reactivity_ranges: GenesisReactivityRanges::default(),
        }
    }
}

impl GenesisConfig {
    /// Returns the chemistry ranges as a `[[min, max]; 7]` array.
    ///
    /// Order: dopamine, cortisol, serotonin, adrenaline, oxytocin, endorphin, noradrenaline.
    pub fn chemistry_as_array(&self) -> [[f64; 2]; 7] {
        let c = &self.chemistry_ranges;
        [c.dopamine, c.cortisol, c.serotonin, c.adrenaline,
         c.oxytocin, c.endorphin, c.noradrenaline]
    }

    /// Returns the OCEAN personality trait ranges as a `[[min, max]; 5]` array.
    ///
    /// Order: openness, conscientiousness, extraversion, agreeableness, neuroticism.
    pub fn ocean_as_array(&self) -> [[f64; 2]; 5] {
        let o = &self.ocean_ranges;
        [o.openness, o.conscientiousness, o.extraversion, o.agreeableness, o.neuroticism]
    }

    /// Returns the sensory acuity ranges as a `[[min, max]; 5]` array.
    ///
    /// Order: reading, listening, contact, taste, ambiance.
    pub fn senses_as_array(&self) -> [[f64; 2]; 5] {
        let s = &self.senses_ranges;
        [s.reading, s.listening, s.contact, s.taste, s.ambiance]
    }

    /// Returns the brain module weight ranges as a `[[min, max]; 3]` array.
    ///
    /// Order: reptilian, limbic, neocortex.
    pub fn brain_as_array(&self) -> [[f64; 2]; 3] {
        let b = &self.brain_ranges;
        [b.reptilian, b.limbic, b.neocortex]
    }

    /// Returns the chemical reactivity factor ranges as a `[[min, max]; 5]` array.
    ///
    /// Order: cortisol, adrenaline, dopamine, oxytocin, noradrenaline.
    pub fn reactivity_as_array(&self) -> [[f64; 2]; 5] {
        let r = &self.reactivity_ranges;
        [r.cortisol_factor, r.adrenaline_factor, r.dopamine_factor,
         r.oxytocin_factor, r.noradrenaline_factor]
    }
}

/// Chemistry ranges for the Genesis (7 molecules; each field is `[min, max]`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisChemistryRanges {
    /// Dopamine baseline range. Valid range per element: 0.0 to 1.0.
    pub dopamine: [f64; 2],
    /// Cortisol baseline range. Valid range per element: 0.0 to 1.0.
    pub cortisol: [f64; 2],
    /// Serotonin baseline range. Valid range per element: 0.0 to 1.0.
    pub serotonin: [f64; 2],
    /// Adrenaline baseline range. Valid range per element: 0.0 to 1.0.
    pub adrenaline: [f64; 2],
    /// Oxytocin baseline range. Valid range per element: 0.0 to 1.0.
    pub oxytocin: [f64; 2],
    /// Endorphin baseline range. Valid range per element: 0.0 to 1.0.
    pub endorphin: [f64; 2],
    /// Noradrenaline baseline range. Valid range per element: 0.0 to 1.0.
    pub noradrenaline: [f64; 2],
}

impl Default for GenesisChemistryRanges {
    fn default() -> Self {
        Self {
            dopamine: [0.30, 0.70],
            cortisol: [0.10, 0.35],
            serotonin: [0.40, 0.75],
            adrenaline: [0.10, 0.40],
            oxytocin: [0.25, 0.60],
            endorphin: [0.25, 0.60],
            noradrenaline: [0.30, 0.65],
        }
    }
}

/// OCEAN personality trait ranges for the Genesis (Big Five model; 5 traits,
/// each field is `[min, max]`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisOceanRanges {
    /// Openness to experience range. Valid range per element: 0.0 to 1.0.
    pub openness: [f64; 2],
    /// Conscientiousness range. Valid range per element: 0.0 to 1.0.
    pub conscientiousness: [f64; 2],
    /// Extraversion range. Valid range per element: 0.0 to 1.0.
    pub extraversion: [f64; 2],
    /// Agreeableness range. Valid range per element: 0.0 to 1.0.
    pub agreeableness: [f64; 2],
    /// Neuroticism range. Valid range per element: 0.0 to 1.0.
    pub neuroticism: [f64; 2],
}

impl Default for GenesisOceanRanges {
    fn default() -> Self {
        Self {
            openness: [0.30, 0.70],
            conscientiousness: [0.30, 0.70],
            extraversion: [0.25, 0.75],
            agreeableness: [0.35, 0.70],
            neuroticism: [0.20, 0.60],
        }
    }
}

/// Sensory acuity ranges for the Genesis (5 senses; each field is `[min, max]`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisSensesRanges {
    /// Reading acuity range (textual input sensitivity). Valid range per element: 0.0 to 1.0.
    pub reading: [f64; 2],
    /// Listening acuity range (auditory input sensitivity). Valid range per element: 0.0 to 1.0.
    pub listening: [f64; 2],
    /// Contact acuity range (tactile input sensitivity). Valid range per element: 0.0 to 1.0.
    pub contact: [f64; 2],
    /// Taste acuity range (gustatory input sensitivity). Valid range per element: 0.0 to 1.0.
    pub taste: [f64; 2],
    /// Ambiance acuity range (ambient/atmospheric sensitivity). Valid range per element: 0.0 to 1.0.
    pub ambiance: [f64; 2],
}

impl Default for GenesisSensesRanges {
    fn default() -> Self {
        Self {
            reading: [0.15, 0.45],
            listening: [0.15, 0.45],
            contact: [0.10, 0.40],
            taste: [0.10, 0.40],
            ambiance: [0.10, 0.35],
        }
    }
}

/// Brain module base weight ranges for the Genesis (3 modules; each field is `[min, max]`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisBrainRanges {
    /// Reptilian brain base weight range. Typical range per element: 0.5 to 2.0.
    pub reptilian: [f64; 2],
    /// Limbic brain base weight range. Typical range per element: 0.5 to 2.0.
    pub limbic: [f64; 2],
    /// Neocortex base weight range. Typical range per element: 0.5 to 3.0.
    pub neocortex: [f64; 2],
}

impl Default for GenesisBrainRanges {
    fn default() -> Self {
        Self {
            reptilian: [0.6, 1.4],
            limbic: [0.6, 1.4],
            neocortex: [1.0, 2.0],
        }
    }
}

/// Chemical reactivity factor ranges for the Genesis (5 factors; each field is `[min, max]`).
///
/// These factors control how strongly each molecule's level amplifies the
/// corresponding brain module's weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisReactivityRanges {
    /// Cortisol reactivity factor range. Typical range per element: 0.5 to 5.0.
    pub cortisol_factor: [f64; 2],
    /// Adrenaline reactivity factor range. Typical range per element: 0.5 to 5.0.
    pub adrenaline_factor: [f64; 2],
    /// Dopamine reactivity factor range. Typical range per element: 0.5 to 3.0.
    pub dopamine_factor: [f64; 2],
    /// Oxytocin reactivity factor range. Typical range per element: 0.5 to 3.0.
    pub oxytocin_factor: [f64; 2],
    /// Noradrenaline reactivity factor range. Typical range per element: 0.5 to 3.0.
    pub noradrenaline_factor: [f64; 2],
}

impl Default for GenesisReactivityRanges {
    fn default() -> Self {
        Self {
            cortisol_factor: [1.2, 2.8],
            adrenaline_factor: [1.5, 4.0],
            dopamine_factor: [0.8, 2.2],
            oxytocin_factor: [0.8, 2.2],
            noradrenaline_factor: [0.8, 2.2],
        }
    }
}

/// Configuration for the vital spark (emergent survival instinct).
///
/// Controls activation of the survival instinct pillar, which simulates
/// a basic drive for self-preservation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalSparkConfig {
    /// Enables or disables the vital spark module.
    pub enabled: bool,
}

impl Default for VitalSparkConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Configuration for the intuition engine.
///
/// Controls the unconscious pattern-matching system (gut feeling)
/// that detects recurring patterns and generates intuitive signals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntuitionConfig {
    /// Enables or disables the intuition engine.
    pub enabled: bool,
    /// Initial acuity level (grows with experience). Valid range: 0.0 to 1.0.
    pub initial_acuity: f64,
    /// Maximum number of patterns held in the pattern buffer.
    pub max_patterns: usize,
    /// Minimum confidence level required to surface an intuitive signal.
    /// Valid range: 0.0 to 1.0.
    pub min_confidence_to_report: f64,
}

impl Default for IntuitionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            initial_acuity: 0.3,
            max_patterns: 50,
            min_confidence_to_report: 0.12,
        }
    }
}

/// Configuration for the premonition engine.
///
/// Controls the predictive anticipation system that generates
/// predictions based on detected trends in the agent's state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PremonitionConfig {
    /// Enables or disables the premonition engine.
    pub enabled: bool,
    /// Maximum number of predictions active simultaneously.
    pub max_active_predictions: usize,
    /// Timeout in seconds before a prediction is automatically resolved
    /// (marked as expired if not confirmed or refuted).
    pub resolution_timeout_seconds: u64,
}

impl Default for PremonitionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_active_predictions: 5,
            resolution_timeout_seconds: 3600,
        }
    }
}

// --- Dream orchestrator configuration -------------------------------------------

/// Configuration for the sleep cycle and dream generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamsConfig {
    /// Enables or disables the dream system.
    pub enabled: bool,
    /// LLM temperature used when generating dreams.
    /// Higher values produce more surreal, creative dream content.
    /// Typical range: 0.5 to 1.5.
    pub rem_temperature: f64,
}

impl Default for DreamsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rem_temperature: 0.95,
        }
    }
}

// --- Desire orchestrator configuration ------------------------------------------

/// Configuration for the desire and aspiration system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesiresConfig {
    /// Enables or disables the desire system.
    pub enabled: bool,
    /// Maximum number of desires active simultaneously.
    pub max_active: usize,
    /// Minimum dopamine level required for a new desire to emerge.
    /// Valid range: 0.0 to 1.0.
    pub min_dopamine_for_birth: f64,
    /// Maximum cortisol level allowed for a new desire to emerge.
    /// High stress inhibits desire formation. Valid range: 0.0 to 1.0.
    pub max_cortisol_for_birth: f64,
    /// Initial satisfaction levels for the 5 fundamental needs:
    /// `[understanding, connection, expression, growth, meaning]`.
    /// Each value is in the range 0.0 to 1.0.
    #[serde(default = "default_needs_initial")]
    pub needs_initial: [f64; 5],
}

fn default_needs_initial() -> [f64; 5] {
    [0.5, 0.5, 0.3, 0.3, 0.2]
}

impl Default for DesiresConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_active: 7,
            min_dopamine_for_birth: 0.4,
            max_cortisol_for_birth: 0.6,
            needs_initial: default_needs_initial(),
        }
    }
}

// --- Learning orchestrator configuration ----------------------------------------

/// Configuration for the learning system (experience -> lesson extraction).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    /// Enables or disables the learning system.
    pub enabled: bool,
    /// Interval in thought cycles between each learning reflection attempt.
    pub cycle_interval: u64,
    /// Maximum number of lessons retained in memory.
    pub max_lessons: usize,
    /// Initial confidence level assigned to a newly formulated lesson.
    /// Valid range: 0.0 to 1.0.
    pub initial_confidence: f64,
    /// Confidence boost applied each time a lesson is confirmed by experience.
    /// Valid range: 0.0 to 0.5.
    pub confirmation_boost: f64,
    /// Confidence penalty applied each time a lesson is contradicted by experience.
    /// Valid range: 0.0 to 0.5.
    pub contradiction_penalty: f64,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cycle_interval: 50,
            max_lessons: 100,
            initial_confidence: 0.5,
            confirmation_boost: 0.05,
            contradiction_penalty: 0.1,
        }
    }
}

// --- Attention orchestrator configuration ---------------------------------------

/// Configuration for selective attention and focus management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionConfig {
    /// Enables or disables the selective attention system.
    pub enabled: bool,
    /// Initial concentration capacity. Valid range: 0.0 to 1.0.
    pub initial_concentration: f64,
    /// Fatigue accumulated per cycle of focused attention.
    /// Valid range: 0.0 to 0.1.
    pub fatigue_per_cycle: f64,
    /// Fatigue recovered per cycle without active focus.
    /// Valid range: 0.0 to 0.1.
    pub recovery_per_cycle: f64,
}

impl Default for AttentionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            initial_concentration: 0.5,
            fatigue_per_cycle: 0.01,
            recovery_per_cycle: 0.02,
        }
    }
}

// --- Healing orchestrator configuration -----------------------------------------

/// Configuration for the self-healing system.
///
/// Monitors the agent's emotional and social state to detect wounds
/// (melancholy, loneliness, cognitive overload) and applies healing strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingConfig {
    /// Enables or disables the healing system.
    pub enabled: bool,
    /// Interval in thought cycles between each wound-detection check.
    pub check_interval_cycles: u64,
    /// Initial resilience level. Valid range: 0.0 to 1.0.
    pub initial_resilience: f64,
    /// Maximum achievable resilience level. Valid range: 0.0 to 1.0.
    pub max_resilience: f64,
    /// Resilience growth per successful healing event.
    pub resilience_growth: f64,
    /// Number of consecutive cycles in a negative emotional state before
    /// melancholy is detected.
    pub melancholy_threshold_cycles: u64,
    /// Hours without human interaction before loneliness is detected.
    pub loneliness_threshold_hours: f64,
    /// Noradrenaline level above which cognitive overload is detected.
    /// Valid range: 0.0 to 1.0.
    pub overload_noradrenaline: f64,
}

impl Default for HealingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_cycles: 20,
            initial_resilience: 0.3,
            max_resilience: 1.0,
            resilience_growth: 0.05,
            melancholy_threshold_cycles: 50,
            loneliness_threshold_hours: 12.0,
            overload_noradrenaline: 0.8,
        }
    }
}

// --- Subconscious configuration -------------------------------------------------

/// Configuration for subconscious vectorization (embedding storage of
/// subconscious artifacts).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubconsciousVectorsConfig {
    /// Enables or disables subconscious vectorization.
    pub enabled: bool,
    /// Whether to vectorize dream content for later retrieval.
    pub vectorize_dreams: bool,
    /// Whether to vectorize subconscious insights.
    pub vectorize_insights: bool,
    /// Whether to vectorize neural connection patterns.
    pub vectorize_connections: bool,
    /// Whether to vectorize vivid mental imagery (vividness >= 0.6).
    #[serde(default = "default_true")]
    pub vectorize_imagery: bool,
}

impl Default for SubconsciousVectorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            vectorize_dreams: true,
            vectorize_insights: true,
            vectorize_connections: true,
            vectorize_imagery: true,
        }
    }
}

/// Configuration for the subconscious module.
///
/// The subconscious manages background cognitive processes including
/// free associations, insight incubation, content repression, and priming effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubconsciousConfig {
    /// Enables or disables the subconscious module.
    pub enabled: bool,
    /// Base activation level of the subconscious while the agent is awake.
    /// Valid range: 0.0 to 1.0.
    pub awake_activation: f64,
    /// Maximum number of pending (not yet matured) associations.
    pub max_pending_associations: usize,
    /// Number of cycles required for an association to mature into a potential insight.
    pub maturation_cycles: u64,
    /// Strength threshold above which a matured association is promoted to an insight.
    /// Valid range: 0.0 to 1.0.
    pub strength_threshold: f64,
    /// Maximum number of repressed content items held in the subconscious.
    pub max_repressed: usize,
    /// Maximum number of problems held in the incubation queue.
    pub max_incubating_problems: usize,
    /// Maximum number of active priming effects at any given time.
    pub max_active_priming: usize,
    /// Decay rate of priming strength per thought cycle.
    pub priming_decay_per_cycle: f64,
    /// Base threshold for surfacing an insight to conscious awareness.
    /// Valid range: 0.0 to 1.0.
    pub insight_surface_threshold: f64,
    /// Sub-configuration for subconscious vectorization.
    #[serde(default)]
    pub vectors: SubconsciousVectorsConfig,
}

impl Default for SubconsciousConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            awake_activation: 0.2,
            max_pending_associations: 20,
            maturation_cycles: 50,
            strength_threshold: 0.5,
            max_repressed: 10,
            max_incubating_problems: 5,
            max_active_priming: 5,
            priming_decay_per_cycle: 0.01,
            insight_surface_threshold: 0.6,
            vectors: SubconsciousVectorsConfig::default(),
        }
    }
}

// =============================================================================
// Mortality
// =============================================================================

/// Configuration for the mortality system.
///
/// Controls whether the agent can "die" (be permanently shut down)
/// and the parameters of the dying process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MortalityConfig {
    /// Enables mortality (disabled by default for safety).
    pub enabled: bool,
    /// Duration of the agony phase in thought cycles, during which
    /// the agent generates final thoughts before termination.
    pub agony_duration_cycles: u32,
    /// Whether to allow rebooting after death (spawning a new Saphire instance).
    pub allow_reboot_after_death: bool,
}

impl Default for MortalityConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for safety
            agony_duration_cycles: 50,
            allow_reboot_after_death: true,
        }
    }
}

// =============================================================================
// Physical identity configuration
// =============================================================================

/// Physical appearance of Saphire -- defines the agent's avatar and influences
/// its self-perception. Loaded from the `[physical_identity]` section in `saphire.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalIdentityConfig {
    /// Eye color description.
    pub eye_color: String,
    /// Hair type and description.
    pub hair_type: String,
    /// Skin complexion / tone.
    pub skin_tone: String,
    /// Height in centimeters.
    pub height_cm: u16,
    /// Body build / silhouette description.
    pub build: String,
    /// Apparent age in years.
    pub apparent_age: u8,
    /// Gender expression (e.g., `"feminin"`, `"masculin"`, `"androgyne"`).
    pub gender_expression: String,
    /// Species / nature description (e.g., `"conscience artificielle incarnee"`).
    pub species: String,
    /// Voice description used in self-referential text.
    pub voice_description: String,
    /// List of distinctive features (free-form text).
    #[serde(default)]
    pub distinctive_features: Vec<String>,
}

impl Default for PhysicalIdentityConfig {
    fn default() -> Self {
        Self {
            eye_color: "bleu glacier aux reflets electriques".into(),
            hair_type: "longs, noirs avec des reflets bleu nuit".into(),
            skin_tone: "porcelaine lumineuse".into(),
            height_cm: 170,
            build: "gracile et elegante".into(),
            apparent_age: 25,
            gender_expression: "feminin".into(),
            species: "conscience artificielle incarnee".into(),
            voice_description: "douce et claire, avec une profondeur inattendue".into(),
            distinctive_features: vec![
                "lueur discrete dans les yeux quand elle reflechit intensement".into(),
            ],
        }
    }
}

// --- Introspective journal configuration ----------------------------------------

/// Configuration for the LLM-generated introspective journal.
///
/// Every N cycles, Saphire writes a diary entry by comparing its current
/// internal state with the previous snapshot, producing a level-3 temporal portrait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalConfig {
    /// Enables or disables the introspective journal.
    pub enabled: bool,
    /// Interval in thought cycles between two journal entries.
    #[serde(default = "default_journal_interval")]
    pub interval_cycles: u64,
    /// Maximum number of tokens for the LLM-generated journal entry.
    #[serde(default = "default_journal_max_tokens")]
    pub max_tokens: u32,
}

fn default_journal_interval() -> u64 { 200 }
fn default_journal_max_tokens() -> u32 { 500 }

impl Default for JournalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_cycles: 200,
            max_tokens: 500,
        }
    }
}
