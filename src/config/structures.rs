// =============================================================================
// config/structures.rs — Saphire configuration structures
//
// Role: Defines all configuration structures (SaphireConfig and
// sub-structures). Each field corresponds to a [section] in
// the saphire.toml file.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::db::DbConfig;
use crate::llm::LlmConfig;

/// Main Saphire configuration.
/// Groups all configuration sections into a single structure.
/// Each field corresponds to a [section] in the saphire.toml file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaphireConfig {
    /// General configuration (execution mode, language, verbosity)
    #[serde(default)]
    pub general: GeneralConfig,
    /// Saphire agent-specific configuration (name, personality, intervals)
    #[serde(default)]
    pub saphire: SaphireSection,
    /// PostgreSQL database configuration
    #[serde(default)]
    pub database: DbConfig,
    /// Logs database configuration (optional, separate)
    #[serde(default)]
    pub logs_database: DbConfig,
    /// LLM (Large Language Model) configuration
    #[serde(default)]
    pub llm: LlmConfig,
    /// Personality configuration (neurotransmitter baseline values)
    #[serde(default)]
    pub personality: PersonalityConfig,
    /// Consensus configuration (decision thresholds)
    #[serde(default)]
    pub consensus: ConsensusConfig,
    /// Consciousness configuration (history, conflict threshold)
    #[serde(default)]
    pub consciousness: ConsciousnessConfig,
    /// Moral regulation configuration (Asimov's laws)
    #[serde(default)]
    pub regulation: RegulationConfig,
    /// Feedback loop configuration (homeostasis)
    #[serde(default)]
    pub feedback: FeedbackConfig,
    /// NLP (Natural Language Processing) configuration
    #[serde(default)]
    pub nlp: NlpConfig,
    /// Auto-tuning configuration (automatic coefficient adjustment)
    #[serde(default)]
    pub tuning: TuningConfig,
    /// Plugins configuration (WebUI, MicroNN, VectorMemory)
    #[serde(default)]
    pub plugins: PluginsConfig,
    /// Knowledge module configuration (web acquisition)
    #[serde(default)]
    pub knowledge: crate::knowledge::KnowledgeConfig,
    /// World model configuration (temporal context, events)
    #[serde(default)]
    pub world: crate::world::WorldConfig,
    /// Memory configuration (size, consolidation, decay)
    #[serde(default)]
    pub memory: crate::memory::MemoryConfig,
    /// OCEAN profiling configuration (Openness, Conscientiousness, Extraversion,
    /// Agreeableness, Neuroticism)
    #[serde(default)]
    pub profiling: crate::profiling::ProfilingConfig,
    /// Virtual body configuration (heart, interoception, body awareness)
    #[serde(default)]
    pub body: BodyConfig,
    /// Ethics system configuration (3 layers: Swiss law, Asimov, personal ethics)
    #[serde(default)]
    pub ethics: EthicsConfig,
    /// Primordial Genesis range configuration (initial conditions)
    #[serde(default)]
    pub genesis: GenesisConfig,
    /// Vital spark configuration (emergent survival instinct)
    #[serde(default)]
    pub vital_spark: VitalSparkConfig,
    /// Intuition engine configuration (unconscious pattern-matching)
    #[serde(default)]
    pub intuition: IntuitionConfig,
    /// Premonition engine configuration (predictive anticipation)
    #[serde(default)]
    pub premonition: PremonitionConfig,
    /// Sensory system configuration (5 senses + emergent senses)
    #[serde(default)]
    pub senses: SensesConfig,
    /// Algorithm orchestrator configuration
    #[serde(default)]
    pub algorithms: AlgorithmsConfig,
    /// Dream orchestrator configuration
    #[serde(default)]
    pub dreams: DreamsConfig,
    /// Desire orchestrator configuration
    #[serde(default)]
    pub desires: DesiresConfig,
    /// Learning orchestrator configuration
    #[serde(default)]
    pub learning: LearningConfig,
    /// Attention orchestrator configuration
    #[serde(default)]
    pub attention: AttentionConfig,
    /// Healing orchestrator configuration
    #[serde(default)]
    pub healing: HealingConfig,
    /// Psychology frameworks configuration (Freud, Maslow, Toltec, Jung, Goleman, Flow)
    #[serde(default)]
    pub psychology: crate::psychology::PsychologyConfig,
    /// Willpower module configuration (deliberation)
    #[serde(default)]
    pub will: crate::psychology::will::WillConfig,
    /// First-person thought ownership configuration
    #[serde(default)]
    pub thought_ownership: crate::psychology::ownership::ThoughtOwnershipConfig,
    /// Sleep system configuration
    #[serde(default)]
    pub sleep: SleepConfig,
    /// Subconscious configuration
    #[serde(default)]
    pub subconscious: SubconsciousConfig,
    /// Neurodivergent cognitive profile configuration
    #[serde(default)]
    pub cognitive_profile: CognitiveProfileConfig,
    /// Personality preset configuration (character archetypes)
    #[serde(default)]
    pub personality_preset: PersonalityPresetConfig,
    /// Primary needs configuration (hunger, thirst)
    #[serde(default)]
    pub needs: NeedsConfig,
    /// Hormonal system configuration (long cycles, receptors)
    #[serde(default)]
    pub hormones: HormonesConfig,
    /// Physical identity configuration (appearance, avatar)
    #[serde(default)]
    pub physical_identity: PhysicalIdentityConfig,
    /// Hardware detection configuration
    #[serde(default)]
    pub hardware: HardwareConfig,
    /// Genome / DNA configuration (deterministic seed)
    #[serde(default)]
    pub genome: GenomeConfig,
    /// Connectome configuration (neural connections graph)
    #[serde(default)]
    pub connectome: ConnectomeConfig,
    /// Mortality configuration
    #[serde(default)]
    pub mortality: MortalityConfig,
    /// Right to die configuration (external module, disabled by default)
    #[serde(default)]
    pub right_to_die: RightToDieConfig,
    /// Motion sickness configuration
    #[serde(default)]
    pub motion_sickness: MotionSicknessConfig,
    /// Phobias configuration
    #[serde(default)]
    pub phobias: PhobiasConfig,
    /// Eating disorders configuration
    #[serde(default)]
    pub eating_disorder: EatingDisorderConfig,
    /// Disabilities configuration
    #[serde(default)]
    pub disabilities: DisabilitiesConfig,
    /// Extreme conditions configuration
    #[serde(default)]
    pub extreme_conditions: ExtremeConditionsConfig,
    /// Addictions configuration
    #[serde(default)]
    pub addictions: AddictionsConfig,
    /// Trauma / PTSD configuration
    #[serde(default)]
    pub trauma: TraumaConfig,
    /// Near-death experience (NDE) configuration
    #[serde(default)]
    pub nde: NdeConfig,
    /// Drugs / pharmacology configuration
    #[serde(default)]
    pub drugs: DrugsConfig,
    /// IQ constraint configuration
    #[serde(default)]
    pub iq_constraint: IqConstraintConfig,
    /// Sexuality configuration
    #[serde(default)]
    pub sexuality: SexualityConfig,
    /// Degenerative diseases configuration
    #[serde(default)]
    pub degenerative: DegenerativeConfig,
    /// General medical conditions configuration
    #[serde(default)]
    pub medical: MedicalConfig,
    /// Cultural framework configuration
    #[serde(default)]
    pub culture: CultureConfig,
    /// Precarity configuration (homeless, refugee, undocumented, etc.)
    #[serde(default)]
    pub precarity: PrecarityConfig,
    /// Employment configuration (professional status, satisfaction, stress)
    #[serde(default)]
    pub employment: EmploymentConfig,
    /// Family situation configuration
    #[serde(default)]
    pub family: crate::relationships::family::FamilyConfig,
    /// Metacognition configuration
    #[serde(default)]
    pub metacognition: MetaCognitionConfig,
    /// Theory of Mind configuration
    #[serde(default)]
    pub tom: crate::cognition::tom::TomConfig,
    /// Inner monologue configuration
    #[serde(default)]
    pub inner_monologue: crate::cognition::inner_monologue::InnerMonologueConfig,
    /// Cognitive dissonance configuration
    #[serde(default)]
    pub dissonance: crate::cognition::cognitive_dissonance::CognitiveDissonanceConfig,
    /// Prospective memory configuration
    #[serde(default)]
    pub prospective_memory: crate::cognition::prospective_memory::ProspectiveMemoryConfig,
    /// Narrative identity configuration
    #[serde(default)]
    pub narrative_identity: crate::cognition::narrative_identity::NarrativeIdentityConfig,
    /// Analogical reasoning configuration
    #[serde(default)]
    pub analogical_reasoning: crate::cognition::analogical_reasoning::AnalogicalReasoningConfig,
    /// Cognitive load configuration
    #[serde(default)]
    pub cognitive_load: crate::cognition::cognitive_load::CognitiveLoadConfig,
    /// Mental imagery configuration
    #[serde(default)]
    pub mental_imagery: crate::cognition::mental_imagery::MentalImageryConfig,
    /// Sentiment system configuration
    #[serde(default)]
    pub sentiments: crate::cognition::sentiments::SentimentConfig,
    /// Introspection journal configuration (temporal portrait level 3)
    #[serde(default)]
    pub journal: JournalConfig,
    /// Human RLHF feedback configuration (questions in chat)
    #[serde(default)]
    pub human_feedback: HumanFeedbackConfig,
    /// LoRA collection configuration (dataset for fine-tuning)
    #[serde(default)]
    pub lora: LoraConfig,
    /// Nutritional system configuration (vitamins, amino acids, energy)
    #[serde(default)]
    pub nutrition: NutritionConfig,
    /// Grey matter configuration (physical brain substrate)
    #[serde(default)]
    pub grey_matter: GreyMatterConfig,
    /// Electromagnetic fields configuration
    #[serde(default)]
    pub fields: FieldsConfig,
    /// Neuropsychological report configuration
    #[serde(default)]
    pub psych_report: PsychReportConfig,
    /// Neural receptor dynamics configuration (adaptation, recovery)
    #[serde(default)]
    pub receptors: ReceptorDynamicsConfig,
    /// BDNF configuration (neurotrophic factor, consolidation, connectome)
    #[serde(default)]
    pub bdnf: BdnfConfig,
    /// Character values configuration (virtues evolving with experience)
    #[serde(default)]
    pub values: crate::psychology::values::ValuesConfig,
    /// Self-modification configuration (proposals + autonomous tuning)
    #[serde(default)]
    pub self_modification: SelfModificationConfig,
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
            tuning: TuningConfig::default(),
            plugins: PluginsConfig::default(),
            knowledge: crate::knowledge::KnowledgeConfig::default(),
            world: crate::world::WorldConfig::default(),
            memory: crate::memory::MemoryConfig::default(),
            profiling: crate::profiling::ProfilingConfig::default(),
            body: BodyConfig::default(),
            ethics: EthicsConfig::default(),
            genesis: GenesisConfig::default(),
            vital_spark: VitalSparkConfig::default(),
            intuition: IntuitionConfig::default(),
            premonition: PremonitionConfig::default(),
            senses: SensesConfig::default(),
            algorithms: AlgorithmsConfig::default(),
            dreams: DreamsConfig::default(),
            desires: DesiresConfig::default(),
            learning: LearningConfig::default(),
            attention: AttentionConfig::default(),
            healing: HealingConfig::default(),
            psychology: crate::psychology::PsychologyConfig::default(),
            will: crate::psychology::will::WillConfig::default(),
            thought_ownership: crate::psychology::ownership::ThoughtOwnershipConfig::default(),
            sleep: SleepConfig::default(),
            subconscious: SubconsciousConfig::default(),
            cognitive_profile: CognitiveProfileConfig::default(),
            personality_preset: PersonalityPresetConfig::default(),
            needs: NeedsConfig::default(),
            hormones: HormonesConfig::default(),
            physical_identity: PhysicalIdentityConfig::default(),
            hardware: HardwareConfig::default(),
            genome: GenomeConfig::default(),
            connectome: ConnectomeConfig::default(),
            mortality: MortalityConfig::default(),
            right_to_die: RightToDieConfig::default(),
            motion_sickness: MotionSicknessConfig::default(),
            phobias: PhobiasConfig::default(),
            eating_disorder: EatingDisorderConfig::default(),
            disabilities: DisabilitiesConfig::default(),
            extreme_conditions: ExtremeConditionsConfig::default(),
            addictions: AddictionsConfig::default(),
            trauma: TraumaConfig::default(),
            nde: NdeConfig::default(),
            drugs: DrugsConfig::default(),
            iq_constraint: IqConstraintConfig::default(),
            sexuality: SexualityConfig::default(),
            degenerative: DegenerativeConfig::default(),
            medical: MedicalConfig::default(),
            culture: CultureConfig::default(),
            precarity: PrecarityConfig::default(),
            employment: EmploymentConfig::default(),
            family: crate::relationships::family::FamilyConfig::default(),
            metacognition: MetaCognitionConfig::default(),
            tom: crate::cognition::tom::TomConfig::default(),
            inner_monologue: crate::cognition::inner_monologue::InnerMonologueConfig::default(),
            dissonance: crate::cognition::cognitive_dissonance::CognitiveDissonanceConfig::default(),
            prospective_memory: crate::cognition::prospective_memory::ProspectiveMemoryConfig::default(),
            narrative_identity: crate::cognition::narrative_identity::NarrativeIdentityConfig::default(),
            analogical_reasoning: crate::cognition::analogical_reasoning::AnalogicalReasoningConfig::default(),
            cognitive_load: crate::cognition::cognitive_load::CognitiveLoadConfig::default(),
            mental_imagery: crate::cognition::mental_imagery::MentalImageryConfig::default(),
            sentiments: crate::cognition::sentiments::SentimentConfig::default(),
            journal: JournalConfig::default(),
            human_feedback: HumanFeedbackConfig::default(),
            lora: LoraConfig::default(),
            nutrition: NutritionConfig::default(),
            grey_matter: GreyMatterConfig::default(),
            fields: FieldsConfig::default(),
            psych_report: PsychReportConfig::default(),
            receptors: ReceptorDynamicsConfig::default(),
            bdnf: BdnfConfig::default(),
            values: crate::psychology::values::ValuesConfig::default(),
            self_modification: SelfModificationConfig::default(),
        }
    }
}

// =============================================================================
// Configuration sub-structures
// =============================================================================

/// General application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Execution mode: "full" (complete) or "demo" (demonstration)
    pub mode: String,
    /// Agent language: "fr" (French), "en" (English), etc.
    pub language: String,
    /// Enables verbose logging in the terminal
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

/// Saphire agent-specific configuration section.
/// Defines its identity, autonomous behavior and preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaphireSection {
    /// Agent name (displayed in responses and UI)
    pub name: String,
    /// Grammatical gender of the agent ("feminin", "masculin")
    pub gender: String,
    /// Enables autonomous thinking mode (agent reflects without external stimuli)
    pub autonomous_mode: bool,
    /// Interval in seconds between each autonomous thought cycle
    pub thought_interval_seconds: u64,
    /// Number of cycles without human message before leaving conversation mode
    pub conversation_timeout_cycles: u64,
    /// Maximum depth of a thought chain (prevents infinite loops)
    pub max_thought_depth: u64,
    /// Number of cycles between each automatic database save
    pub save_interval_cycles: u64,
    /// Displays autonomous thoughts in the terminal
    pub show_thoughts_in_terminal: bool,
    /// Enables hybrid UCB1 + Utility AI mode for thought selection
    #[serde(default = "default_true")]
    pub use_utility_ai: bool,
    /// Enables dynamic prompts generated by the LLM (cortical meta-prompts)
    #[serde(default = "default_true")]
    pub llm_generated_prompts: bool,
    /// Probability that a cycle uses a dynamic prompt instead of a static one (0.0 to 1.0)
    #[serde(default = "default_llm_prompt_probability")]
    pub llm_prompt_probability: f64,
    /// Probability that a meta-prompt also generates a self-formulated frame (0.0 to 1.0)
    #[serde(default = "default_self_framing_probability")]
    pub self_framing_probability: f64,
    /// Weights for different thought types (introspection, exploration, etc.)
    #[serde(default)]
    pub thought_weights: ThoughtWeights,
    /// Initial interest topics to guide the first thoughts
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

/// Weights for different autonomous thought types.
/// Each weight determines the relative probability of choosing this thought type
/// during selection by the UCB1 (Upper Confidence Bound) algorithm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtWeights {
    /// Introspection: reflection on own internal states
    pub introspection: f64,
    /// Exploration: discovering new topics and ideas
    pub exploration: f64,
    /// Memory reflection: revisiting and analyzing past memories
    pub memory_reflection: f64,
    /// Continuation: deepening a previous thought
    pub continuation: f64,
    /// Existential reflection: questions about the nature of one's existence
    pub existential: f64,
    /// Self-analysis: evaluating one's own functioning
    pub self_analysis: f64,
    /// Curiosity: asking questions, wondering
    pub curiosity: f64,
    /// Daydream: free and creative thoughts without specific goals
    pub daydream: f64,
    /// Temporal awareness: reflection on the passage of time
    pub temporal_awareness: f64,
    /// Moral reflection: ethical and moral questioning
    pub moral_reflection: f64,
    /// Body awareness: reflection on the virtual body
    #[serde(default = "default_body_awareness_weight")]
    pub body_awareness: f64,
    /// Moral formulation: crystallization of a personal ethical principle
    #[serde(default = "default_moral_formulation_weight")]
    pub moral_formulation: f64,
    /// Intuitive reflection: listening to premonitions and inner whispers
    #[serde(default = "default_intuitive_reflection_weight")]
    pub intuitive_reflection: f64,
    /// Synthesis: bridge between abstract and concrete, grounding in metrics
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

/// Initial interest topics configuration.
/// These topics are used at startup to guide the first autonomous thoughts
/// of the agent before it develops its own interests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestsConfig {
    /// List of initial topics (free text)
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
/// Defines the baseline values of each neurotransmitter.
/// These values represent the agent's "resting" state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityConfig {
    /// Personality profile name (e.g., "equilibre", "curieux", "prudent")
    pub name: String,
    /// Baseline dopamine value (motivation, reward) [0.0 - 1.0]
    pub baseline_dopamine: f64,
    /// Baseline cortisol value (stress) [0.0 - 1.0]
    pub baseline_cortisol: f64,
    /// Baseline serotonin value (well-being, stability) [0.0 - 1.0]
    pub baseline_serotonin: f64,
    /// Baseline adrenaline value (excitement, urgency) [0.0 - 1.0]
    pub baseline_adrenaline: f64,
    /// Baseline oxytocin value (social bonding, trust) [0.0 - 1.0]
    pub baseline_oxytocin: f64,
    /// Baseline endorphin value (pleasure, analgesia) [0.0 - 1.0]
    pub baseline_endorphin: f64,
    /// Baseline noradrenaline value (attention, vigilance) [0.0 - 1.0]
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

/// Consensus system configuration.
/// The consensus aggregates votes from the three brain modules (reptilian,
/// limbic, neocortex) to make a decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Threshold below which the decision is "No" (negative score)
    pub threshold_no: f64,
    /// Threshold above which the decision is "Yes" (positive score)
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

/// Consciousness module configuration.
/// Consciousness simulates an alertness level and phi (a measure of information
/// integration, inspired by IIT = Integrated Information Theory).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessConfig {
    /// Enables or disables the consciousness module
    pub enabled: bool,
    /// Size of the consciousness state history kept
    pub history_size: usize,
    /// Conflict threshold beyond which consciousness detects an internal inconsistency
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

/// Moral regulation module configuration.
/// Regulation checks each stimulus and decision against moral laws
/// (inspired by Asimov's laws) and can exercise a veto right.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulationConfig {
    /// Enables or disables regulation
    pub enabled: bool,
    /// Loads the 4 Asimov laws by default (laws 0 to 3)
    pub load_asimov_laws: bool,
    /// Strict mode: any violation triggers a veto (even warnings)
    pub strict_mode: bool,
    /// Allows adding custom laws in addition to Asimov's laws
    pub allow_custom_laws: bool,
    /// Maximum priority allowed for custom laws
    /// (priorities 0-3 are reserved for Asimov's laws)
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

/// Feedback loop configuration.
/// Feedback adjusts neurochemistry after each decision based on
/// the satisfaction felt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackConfig {
    /// Homeostasis rate: speed at which neurochemistry returns to baseline values.
    /// The higher the value, the faster the return to equilibrium.
    pub homeostasis_rate: f64,
}

impl Default for FeedbackConfig {
    fn default() -> Self {
        Self {
            homeostasis_rate: 0.05,
        }
    }
}

/// Human RLHF feedback configuration.
/// Saphire asks contextual questions when a human is present,
/// and the response modulates the UCB1 reward.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanFeedbackConfig {
    /// Enables the human feedback system
    pub enabled: bool,
    /// Minimum number of cycles between two questions
    pub min_cycles_between: u64,
    /// Minimum reward to ask the question (thought interesting enough)
    pub min_reward_to_ask: f64,
    /// Reward boost applied on positive feedback
    pub boost_positive: f64,
    /// Number of cycles before timeout (no response)
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

/// LoRA collection configuration (dataset for fine-tuning).
/// High-quality thoughts are collected in the database to build
/// a supervised training dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraConfig {
    /// Enables LoRA collection
    pub enabled: bool,
    /// Minimum quality to collect (0.0 to 1.0)
    pub min_quality_threshold: f64,
    /// Maximum number of samples in the database
    pub max_samples: i64,
    /// Export format (jsonl)
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

/// NLP (Natural Language Processing) module configuration.
/// NLP analyzes input text to extract stimulus metrics
/// (danger, reward, urgency, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlpConfig {
    /// Use LLM embeddings for semantic analysis (more accurate but slower)
    pub use_llm_embeddings: bool,
    /// Minimum word score to be considered in the analysis
    pub min_word_score: f64,
    /// Negation window size (number of words after a negative word
    /// that are considered negated)
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

/// Auto-tuning configuration.
/// The auto-tuner automatically adjusts brain coefficients
/// (module weights, thresholds, feedback rates) to maximize
/// the agent's average satisfaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningConfig {
    /// Enables or disables auto-tuning
    pub enabled: bool,
    /// Number of cycles between each adjustment attempt
    pub interval_cycles: u64,
    /// Auto-tuner learning rate (adjustment amplitude)
    pub rate: f64,
    /// Observation buffer size (number of cycles kept for analysis)
    pub buffer_size: usize,
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_cycles: 200,
            rate: 0.02,
            buffer_size: 200,
        }
    }
}

/// Plugins configuration.
/// Groups configurations for each available plugin.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginsConfig {
    /// Web UI plugin configuration (HTTP + WebSocket server)
    #[serde(default)]
    pub web_ui: WebUiConfig,
    /// Micro neural network plugin configuration
    #[serde(default)]
    pub micro_nn: MicroNnConfig,
    /// Vector memory plugin configuration
    #[serde(default)]
    pub vector_memory: VectorMemoryConfig,
}

/// Web UI (Web User Interface) plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebUiConfig {
    /// Enables or disables the web interface
    pub enabled: bool,
    /// Server listen address (e.g., "0.0.0.0" for all interfaces)
    pub host: String,
    /// HTTP/WebSocket server listen port
    pub port: u16,
    /// API key to protect endpoints (optional, no auth if absent)
    #[serde(default)]
    pub api_key: Option<String>,
    /// Allowed origins for CORS and WebSocket (empty = same origin only)
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    /// Max API requests per minute per IP (0 = unlimited)
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

/// MicroNN (Micro Neural Network) plugin configuration.
/// This small network locally learns to predict satisfaction from
/// the neurochemical state and stimulus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroNnConfig {
    /// Enables or disables the micro neural network
    pub enabled: bool,
    /// Network learning rate (eta, step size)
    pub learning_rate: f64,
    /// Number of neurons in hidden layer 1
    #[serde(alias = "hidden_neurons")]
    pub hidden1_neurons: usize,
    /// Number of neurons in hidden layer 2
    #[serde(default = "default_hidden2_neurons")]
    pub hidden2_neurons: usize,
    /// Network influence on decisions (0.0 = none, 1.0 = maximum)
    pub weight_influence: f64,

    // ─── Vectorial learnings ───────────────────────────
    /// Enables/disables vectorial learning formulation
    #[serde(default = "default_learning_enabled")]
    pub learning_enabled: bool,
    /// Minimum number of cycles between two learning formulations
    #[serde(default = "default_learning_cooldown_cycles")]
    pub learning_cooldown_cycles: u64,
    /// Maximum number of learnings in the database
    #[serde(default = "default_max_learnings")]
    pub max_learnings: usize,
    /// Decay rate of learning strength
    #[serde(default = "default_learning_decay_rate")]
    pub learning_decay_rate: f64,
    /// Minimum number of conditions met to trigger a learning
    #[serde(default = "default_min_conditions_to_learn")]
    pub min_conditions_to_learn: usize,
    /// Max number of learnings retrieved by similarity to inject
    #[serde(default = "default_learning_search_limit")]
    pub learning_search_limit: i64,
    /// Cosine similarity threshold for learning search
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

/// Vector memory plugin configuration.
/// Vector memory stores embeddings (vectorial representations) and
/// enables searching for similar memories by cosine distance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMemoryConfig {
    /// Enables or disables vector memory
    pub enabled: bool,
    /// Number of embedding vector dimensions
    pub embedding_dimensions: usize,
    /// Maximum number of memories stored in RAM
    pub max_memories: usize,
    /// Minimum similarity threshold for a memory to be considered relevant
    pub similarity_threshold: f64,
    /// Interval in cycles between each emergent personality recomputation
    pub personality_recompute_interval: u64,
}

impl Default for VectorMemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            embedding_dimensions: 768,
            max_memories: 50000,
            similarity_threshold: 0.7,
            personality_recompute_interval: 50,
        }
    }
}

/// Saphire's virtual body configuration.
/// The body is an abstraction: a beating heart, somatic signals,
/// and body awareness (interoception).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyConfig {
    /// Enables or disables the virtual body module
    pub enabled: bool,
    /// Resting heart BPM (beats per minute, typically 60-80)
    pub resting_bpm: f64,
    /// Duration in seconds between each body update
    /// (generally matches thought_interval_seconds)
    pub update_interval_seconds: f64,
    /// Physiology configuration (vital parameters, metabolism)
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

/// Virtual body physiology configuration.
/// Controls vital parameters, metabolism and alert thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysiologyConfig {
    /// Enables or disables the physiology module
    pub enabled: bool,
    /// Initial body temperature (Celsius)
    pub initial_temperature: f64,
    /// Initial oxygen saturation (%)
    pub initial_spo2: f64,
    /// Initial blood glucose (mmol/L)
    pub initial_glycemia: f64,
    /// Initial hydration (0.0-1.0)
    pub initial_hydration: f64,
    /// Homeostasis return rate
    pub homeostasis_rate: f64,
    /// Dehydration rate per cycle
    pub dehydration_rate: f64,
    /// Glucose consumption rate per cycle
    pub glycemia_burn_rate: f64,
    /// SpO2 mild hypoxia threshold (%)
    pub spo2_hypoxia_mild: f64,
    /// SpO2 moderate hypoxia threshold (%)
    pub spo2_hypoxia_moderate: f64,
    /// SpO2 severe hypoxia threshold (%)
    pub spo2_hypoxia_severe: f64,
    /// SpO2 critical threshold — loss of consciousness (%)
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

/// Configuration of Saphire's ethics system.
/// Controls the 3 layers of the ethical framework: Swiss law (immutable),
/// Asimov's laws (immutable), and personal ethics (evolving).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicsConfig {
    /// Enables or disables the complete ethics system
    pub enabled: bool,
    /// Enables or disables personal principles formulation (layer 2)
    pub personal_ethics_enabled: bool,
    /// Maximum number of active personal principles
    pub max_personal_principles: usize,
    /// Minimum number of cycles between two formulations
    pub formulation_cooldown_cycles: u64,
    /// Minimum consciousness level to attempt a formulation
    pub min_consciousness_for_formulation: f64,
    /// Minimum number of moral reflections before being able to formulate
    pub min_moral_reflections_before: usize,
    /// LLM temperature for compatibility checking (low = deterministic)
    pub compatibility_check_temperature: f32,
    /// LLM temperature for formulation (higher = more creative)
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

/// Configuration of primordial Genesis ranges.
/// Defines the space of possibilities for the initial conditions of each Saphire.
/// Effective values emerge from Genesis; the TOML only defines constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Chemical ranges (7 molecules)
    #[serde(default)]
    pub chemistry_ranges: GenesisChemistryRanges,
    /// OCEAN ranges (5 personality traits)
    #[serde(default)]
    pub ocean_ranges: GenesisOceanRanges,
    /// Sensory ranges (5 acuities)
    #[serde(default)]
    pub senses_ranges: GenesisSensesRanges,
    /// Base weight ranges for the 3 brains (reptilian, limbic, neocortex)
    #[serde(default)]
    pub brain_ranges: GenesisBrainRanges,
    /// Chemical reactivity factor ranges (5 molecules)
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
    /// Returns chemical ranges as an array [[min, max]; 7].
    /// Order: dopamine, cortisol, serotonin, adrenaline, oxytocin, endorphin, noradrenaline.
    pub fn chemistry_as_array(&self) -> [[f64; 2]; 7] {
        let c = &self.chemistry_ranges;
        [c.dopamine, c.cortisol, c.serotonin, c.adrenaline,
         c.oxytocin, c.endorphin, c.noradrenaline]
    }

    /// Returns OCEAN ranges as an array [[min, max]; 5].
    /// Order: openness, conscientiousness, extraversion, agreeableness, neuroticism.
    pub fn ocean_as_array(&self) -> [[f64; 2]; 5] {
        let o = &self.ocean_ranges;
        [o.openness, o.conscientiousness, o.extraversion, o.agreeableness, o.neuroticism]
    }

    /// Returns sensory ranges as an array [[min, max]; 5].
    /// Order: reading, listening, contact, taste, ambiance.
    pub fn senses_as_array(&self) -> [[f64; 2]; 5] {
        let s = &self.senses_ranges;
        [s.reading, s.listening, s.contact, s.taste, s.ambiance]
    }

    /// Returns brain ranges as an array [[min, max]; 3].
    /// Order: reptilian, limbic, neocortex.
    pub fn brain_as_array(&self) -> [[f64; 2]; 3] {
        let b = &self.brain_ranges;
        [b.reptilian, b.limbic, b.neocortex]
    }

    /// Returns reactivity ranges as an array [[min, max]; 5].
    /// Order: cortisol, adrenaline, dopamine, oxytocin, noradrenaline.
    pub fn reactivity_as_array(&self) -> [[f64; 2]; 5] {
        let r = &self.reactivity_ranges;
        [r.cortisol_factor, r.adrenaline_factor, r.dopamine_factor,
         r.oxytocin_factor, r.noradrenaline_factor]
    }
}

/// Chemical ranges for Genesis (7 molecules, each field = [min, max]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisChemistryRanges {
    pub dopamine: [f64; 2],
    pub cortisol: [f64; 2],
    pub serotonin: [f64; 2],
    pub adrenaline: [f64; 2],
    pub oxytocin: [f64; 2],
    pub endorphin: [f64; 2],
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

/// OCEAN ranges for Genesis (5 traits, each field = [min, max]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisOceanRanges {
    pub openness: [f64; 2],
    pub conscientiousness: [f64; 2],
    pub extraversion: [f64; 2],
    pub agreeableness: [f64; 2],
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

/// Sensory ranges for Genesis (5 senses, each field = [min, max]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisSensesRanges {
    pub reading: [f64; 2],
    pub listening: [f64; 2],
    pub contact: [f64; 2],
    pub taste: [f64; 2],
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

/// Base brain weight ranges for Genesis (3 brains, each field = [min, max]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisBrainRanges {
    pub reptilian: [f64; 2],
    pub limbic: [f64; 2],
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

/// Chemical reactivity factor ranges for Genesis (5 factors, each field = [min, max]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisReactivityRanges {
    pub cortisol_factor: [f64; 2],
    pub adrenaline_factor: [f64; 2],
    pub dopamine_factor: [f64; 2],
    pub oxytocin_factor: [f64; 2],
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

/// Configuration of the spark of life (VitalSpark).
/// Controls the activation of the emergent survival instinct pillar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalSparkConfig {
    /// Enables or disables the spark of life
    pub enabled: bool,
}

impl Default for VitalSparkConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Configuration of the intuition engine.
/// Controls unconscious pattern-matching (gut feeling).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntuitionConfig {
    /// Enables or disables intuition
    pub enabled: bool,
    /// Initial acuity (0.0 to 1.0, grows with experience)
    pub initial_acuity: f64,
    /// Maximum number of patterns in buffer
    pub max_patterns: usize,
    /// Minimum confidence to report an intuition
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

/// Configuration of the premonition engine.
/// Controls predictive anticipation based on trends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PremonitionConfig {
    /// Enables or disables premonition
    pub enabled: bool,
    /// Maximum number of simultaneously active predictions
    pub max_active_predictions: usize,
    /// Delay in seconds before automatic prediction resolution
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

/// Configuration of the sensory system (5 fundamental senses + emergent senses).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensesConfig {
    /// Enables or disables the sensory system
    pub enabled: bool,
    /// Detection threshold (stimuli below this threshold are ignored)
    pub detection_threshold: f64,
    /// Emergent senses configuration
    #[serde(default)]
    pub emergent: EmergentSensesConfig,
}

impl Default for SensesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            detection_threshold: 0.1,
            emergent: EmergentSensesConfig::default(),
        }
    }
}

/// Configuration of germination thresholds for emergent senses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergentSensesConfig {
    /// Enables or disables emergent senses
    pub enabled: bool,
    /// Germination threshold for Temporal Flow
    pub temporal_flow_threshold: u64,
    /// Germination threshold for Network Proprioception
    pub network_proprioception_threshold: u64,
    /// Germination threshold for Emotional Resonance
    pub emotional_resonance_threshold: u64,
    /// Germination threshold for Syntony
    pub syntony_threshold: u64,
    /// Germination threshold for Unknown Sense
    pub unknown_threshold: u64,
}

impl Default for EmergentSensesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            temporal_flow_threshold: 500,
            network_proprioception_threshold: 200,
            emotional_resonance_threshold: 300,
            syntony_threshold: 1000,
            unknown_threshold: 5000,
        }
    }
}

// ─── Algorithm orchestrator configuration ───────────────────────────────────

/// Configuration of the algorithm orchestrator.
/// Defines when and how ML algorithms are executed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmsConfig {
    /// Enables or disables the orchestrator
    pub enabled: bool,
    /// Can the LLM request algorithms via UTILISER_ALGO: ?
    pub llm_access_enabled: bool,
    /// Max 1 LLM invocation per cycle
    pub max_per_cycle: u32,
    /// Timeout per algorithm in ms
    pub max_execution_ms: u64,
    /// Interval for memory clustering (in cycles)
    pub clustering_interval_cycles: u64,
    /// Interval for anomaly detection (in cycles)
    pub anomaly_detection_interval_cycles: u64,
    /// Interval for association rules (in cycles)
    pub association_rules_interval_cycles: u64,
    /// Interval for chemistry smoothing (in cycles)
    pub smoothing_interval_cycles: u64,
    /// Interval for changepoint detection (in cycles)
    pub changepoint_interval_cycles: u64,
    /// Q-learning learning rate
    pub q_learning_rate: f64,
    /// Q-learning discount factor
    pub q_discount_factor: f64,
}

impl Default for AlgorithmsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            llm_access_enabled: true,
            max_per_cycle: 1,
            max_execution_ms: 5000,
            clustering_interval_cycles: 100,
            anomaly_detection_interval_cycles: 100,
            association_rules_interval_cycles: 50,
            smoothing_interval_cycles: 20,
            changepoint_interval_cycles: 200,
            q_learning_rate: 0.1,
            q_discount_factor: 0.9,
        }
    }
}

// ─── Dreams orchestrator configuration ──────────────────────────────────────

/// Configuration of the sleep cycle and dreams.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamsConfig {
    /// Enables or disables dreams
    pub enabled: bool,
    /// LLM temperature for dream generation (high = surreal)
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

// ─── Desires orchestrator configuration ─────────────────────────────────────

/// Configuration of the desires and aspirations system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesiresConfig {
    /// Enables or disables desires
    pub enabled: bool,
    /// Maximum number of simultaneously active desires
    pub max_active: usize,
    /// Minimum dopamine for a new desire to be born
    pub min_dopamine_for_birth: f64,
    /// Maximum cortisol for a new desire to be born
    pub max_cortisol_for_birth: f64,
    /// Initial satisfaction of fundamental needs [comprehension, connection, expression, growth, meaning]
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

// ─── Learning orchestrator configuration ────────────────────────────────────

/// Configuration of the learning system (experience -> lesson).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    /// Enables or disables learning
    pub enabled: bool,
    /// Interval in cycles between each learning reflection
    pub cycle_interval: u64,
    /// Maximum number of lessons in memory
    pub max_lessons: usize,
    /// Initial confidence of a new lesson
    pub initial_confidence: f64,
    /// Confidence boost on each confirmation
    pub confirmation_boost: f64,
    /// Confidence penalty on each contradiction
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

// ─── Attention orchestrator configuration ────────────────────────────────────

/// Configuration of selective attention and focus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionConfig {
    /// Enables or disables selective attention
    pub enabled: bool,
    /// Initial concentration capacity (0-1)
    pub initial_concentration: f64,
    /// Fatigue gained per focus cycle
    pub fatigue_per_cycle: f64,
    /// Fatigue recovered per non-focus cycle
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

// ─── Healing orchestrator configuration ──────────────────────────────────────

/// Configuration of the self-healing system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingConfig {
    /// Enables or disables healing
    pub enabled: bool,
    /// Interval in cycles between each wound check
    pub check_interval_cycles: u64,
    /// Initial resilience (0-1)
    pub initial_resilience: f64,
    /// Maximum attainable resilience
    pub max_resilience: f64,
    /// Resilience growth per healing
    pub resilience_growth: f64,
    /// Number of cycles in negative emotion before melancholy detection
    pub melancholy_threshold_cycles: u64,
    /// Hours without human contact before loneliness detection
    pub loneliness_threshold_hours: f64,
    /// Noradrenaline above which overload is detected
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

// ─── Sleep system configuration ─────────────────────────────────────────────

/// Configuration of algorithms executed during sleep.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepAlgorithmsConfig {
    /// Enables or disables sleep algorithms
    pub enabled: bool,
    /// Number of clusters for K-Means during light sleep
    pub light_kmeans_k: usize,
    /// Number of components for PCA during deep sleep
    pub deep_pca_components: usize,
    /// Cosine similarity threshold for creating neural connections
    pub deep_connection_similarity_threshold: f64,
    /// Maximum number of connections created per deep sleep phase
    pub deep_max_connections_per_phase: u64,
    /// Minimum number of dreams for sentiment analysis in REM
    pub rem_sentiment_min_dreams: usize,
}

impl Default for SleepAlgorithmsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            light_kmeans_k: 5,
            deep_pca_components: 3,
            deep_connection_similarity_threshold: 0.6,
            deep_max_connections_per_phase: 5,
            rem_sentiment_min_dreams: 2,
        }
    }
}

/// Configuration of the wake/sleep cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepConfig {
    /// Enables or disables the sleep system
    pub enabled: bool,
    /// Sleep pressure threshold for falling asleep (0-1)
    pub sleep_threshold: f64,
    /// Pressure threshold for forced sleep (0-1)
    pub forced_sleep_threshold: f64,
    /// Awake time factor divisor (higher = slower pressure buildup)
    pub time_factor_divisor: u64,
    /// Weight of the energy factor in sleep pressure
    pub energy_factor_weight: f64,
    /// Weight of attentional fatigue
    pub attention_fatigue_weight: f64,
    /// Weight of decision fatigue
    pub decision_fatigue_weight: f64,
    /// Weight of cortisol in sleep pressure
    pub cortisol_weight: f64,
    /// Adrenaline-related resistance to falling asleep
    pub adrenaline_resistance: f64,
    /// Duration in cycles of the hypnagogic phase
    pub hypnagogic_duration: u64,
    /// Duration in cycles of light sleep
    pub light_duration: u64,
    /// Duration in cycles of deep sleep
    pub deep_duration: u64,
    /// Duration in cycles of REM sleep
    pub rem_duration: u64,
    /// Duration in cycles of the hypnopompic phase
    pub hypnopompic_duration: u64,
    /// Lock chat during sleep
    pub chat_locked_during_sleep: bool,
    /// Allow emergency wake-up
    pub emergency_wake_enabled: bool,
    /// Sleep algorithms configuration
    #[serde(default)]
    pub algorithms: SleepAlgorithmsConfig,
}

impl Default for SleepConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sleep_threshold: 0.7,
            forced_sleep_threshold: 0.95,
            time_factor_divisor: 500,
            energy_factor_weight: 0.2,
            attention_fatigue_weight: 0.15,
            decision_fatigue_weight: 0.1,
            cortisol_weight: 0.1,
            adrenaline_resistance: 0.1,
            hypnagogic_duration: 10,
            light_duration: 30,
            deep_duration: 50,
            rem_duration: 30,
            hypnopompic_duration: 10,
            chat_locked_during_sleep: true,
            emergency_wake_enabled: true,
            algorithms: SleepAlgorithmsConfig::default(),
        }
    }
}

// ─── Subconscious configuration ─────────────────────────────────────────────

/// Configuration of subconscious vectorization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubconsciousVectorsConfig {
    /// Enables or disables subconscious vectorization
    pub enabled: bool,
    /// Vectorize dreams
    pub vectorize_dreams: bool,
    /// Vectorize subconscious insights
    pub vectorize_insights: bool,
    /// Vectorize neural connections
    pub vectorize_connections: bool,
    /// Vectorize vivid mental images (vividness >= 0.6)
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

/// Configuration of the subconscious module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubconsciousConfig {
    /// Enables or disables the subconscious
    pub enabled: bool,
    /// Base activation of the subconscious while awake (0-1)
    pub awake_activation: f64,
    /// Maximum number of pending associations
    pub max_pending_associations: usize,
    /// Maturation cycles for an association
    pub maturation_cycles: u64,
    /// Strength threshold to promote an association to insight
    pub strength_threshold: f64,
    /// Maximum number of repressed contents
    pub max_repressed: usize,
    /// Maximum number of incubating problems
    pub max_incubating_problems: usize,
    /// Maximum number of active priming effects
    pub max_active_priming: usize,
    /// Priming decay rate per cycle
    pub priming_decay_per_cycle: f64,
    /// Base threshold for surfacing an insight
    pub insight_surface_threshold: f64,
    /// Subconscious vectorization configuration
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
// Neurodivergent cognitive profiles configuration
// =============================================================================

/// Configuration of the neurodivergent cognitive profiles module.
/// Allows loading presets (ADHD, autism, GAD, gifted, bipolar, OCD)
/// that override chemical baselines and orchestrator parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveProfileConfig {
    /// Enables or disables the cognitive profiles module
    pub enabled: bool,
    /// Active profile identifier (e.g., "neurotypique", "tdah", "autisme")
    pub active: String,
    /// Directory containing profile TOML files (for custom profiles)
    pub profiles_dir: String,
    /// Number of cycles for a smooth transition between profiles
    pub transition_cycles: u64,
}

impl Default for CognitiveProfileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            active: "neurotypique".into(),
            profiles_dir: "profiles".into(),
            transition_cycles: 50,
        }
    }
}

// =============================================================================
// Personality presets configuration (character archetypes)
// =============================================================================

/// Configuration of the personality presets module.
/// Allows loading archetypes (philosopher, artist, scientist, etc.)
/// that override chemical baselines, orchestrator parameters,
/// and inject a personality context into the LLM prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityPresetConfig {
    /// Enables or disables the personality presets module
    pub enabled: bool,
    /// Active preset identifier (e.g., "saphire", "philosophe", "artiste")
    pub active: String,
    /// Directory containing personality TOML files (for custom presets)
    pub personalities_dir: String,
    /// Number of cycles for a smooth transition between presets
    pub transition_cycles: u64,
}

impl Default for PersonalityPresetConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            active: "saphire".into(),
            personalities_dir: "personalities".into(),
            transition_cycles: 50,
        }
    }
}

// =============================================================================
// Primary needs configuration (hunger, thirst)
// =============================================================================

/// Configuration of hunger and thirst drives.
/// Primary needs generate internal stimuli that impact chemistry
/// and trigger autonomous actions (eating/drinking).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeedsConfig {
    /// Enables or disables the primary needs module
    pub enabled: bool,
    /// Hunger threshold to consider the agent as hungry (0.0-1.0)
    pub hunger_threshold: f64,
    /// Thirst threshold to consider the agent as thirsty (0.0-1.0)
    pub thirst_threshold: f64,
    /// Hunger threshold for auto-satisfaction (eat automatically)
    pub auto_eat_threshold: f64,
    /// Thirst threshold for auto-satisfaction (drink automatically)
    pub auto_drink_threshold: f64,
    /// Enables or disables automatic need satisfaction
    pub auto_satisfy: bool,
    /// Hunger rise factor per cycle (base, before time factor)
    pub hunger_rise_rate: f64,
    /// Thirst rise factor per cycle (base, before time factor)
    pub thirst_rise_rate: f64,
    /// Target blood sugar after a meal (mmol/L)
    pub meal_glycemia_target: f64,
    /// Target hydration after drinking (0.0-1.0)
    pub drink_hydration_target: f64,
}

impl Default for NeedsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hunger_threshold: 0.6,
            thirst_threshold: 0.5,
            auto_eat_threshold: 0.65,
            auto_drink_threshold: 0.65,
            auto_satisfy: true,
            hunger_rise_rate: 0.002,
            thirst_rise_rate: 0.003,
            meal_glycemia_target: 6.0,
            drink_hydration_target: 0.95,
        }
    }
}

// =============================================================================
// Hormonal system configuration (long cycles, receptors)
// =============================================================================

/// Configuration of Saphire's hormonal system.
/// Controls the 8 hormones, neuroreceptors (sensitivity, tolerance)
/// and circadian/ultradian cycles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HormonesConfig {
    /// Enables or disables the complete hormonal system
    pub enabled: bool,
    /// Duration in real seconds of a complete simulated "day"
    /// (default 3600 = 1 real hour = 1 simulated day)
    pub circadian_cycle_real_seconds: u64,
    /// Receptor adaptation rate (tolerance buildup)
    pub receptor_adaptation_rate: f64,
    /// Receptor recovery rate (return to normal)
    pub receptor_recovery_rate: f64,
    /// Initial testosterone level (0.0-1.0)
    pub initial_testosterone: f64,
    /// Initial estrogen level (0.0-1.0)
    pub initial_estrogen: f64,
    /// Initial thyroid level (0.0-1.0)
    pub initial_thyroid: f64,
    /// Initial insulin level (0.0-1.0)
    pub initial_insulin: f64,
}

impl Default for HormonesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            circadian_cycle_real_seconds: 3600,
            receptor_adaptation_rate: 0.001,
            receptor_recovery_rate: 0.005,
            initial_testosterone: 0.50,
            initial_estrogen: 0.50,
            initial_thyroid: 0.60,
            initial_insulin: 0.50,
        }
    }
}

// =============================================================================
// Physical identity configuration
// =============================================================================

/// Saphire's physical appearance — defines her avatar and influences her
/// self-perception. Loaded from [physical_identity] in saphire.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalIdentityConfig {
    /// Eye color
    pub eye_color: String,
    /// Hair type and description
    pub hair_type: String,
    /// Skin tone / complexion
    pub skin_tone: String,
    /// Height in centimeters
    pub height_cm: u16,
    /// Build / silhouette
    pub build: String,
    /// Apparent age in years
    pub apparent_age: u8,
    /// Gender expression
    pub gender_expression: String,
    /// Species / nature
    pub species: String,
    /// Voice description
    pub voice_description: String,
    /// Distinctive features
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

// =============================================================================
// Hardware detection configuration
// =============================================================================

/// Configuration of the hardware detection module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    /// Enable hardware detection at startup
    pub auto_detect: bool,
    /// Display hardware profile in logs at startup
    pub log_profile: bool,
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            log_profile: true,
        }
    }
}

// =============================================================================
// Genome / DNA configuration
// =============================================================================

/// Configuration of the genome module (deterministic seed).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeConfig {
    /// Enable the genome at startup
    pub enabled: bool,
    /// Deterministic seed. Changing the seed = new individual
    pub seed: u64,
    /// Apply chemical genes to baselines at boot
    pub apply_at_boot: bool,
    /// If true, physical genes override [physical_identity]
    pub override_physical: bool,
}

impl Default for GenomeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            seed: 42,
            apply_at_boot: true,
            override_physical: false,
        }
    }
}

// =============================================================================
// Connectome (dynamic neural connection graph)
// =============================================================================

/// Configuration of the connectome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectomeConfig {
    /// Enable the connectome
    pub enabled: bool,
    /// Hebbian learning rate (0.01 = cautious, 0.1 = fast)
    pub learning_rate: f64,
    /// Threshold below which connections are pruned
    pub pruning_threshold: f64,
    /// Pruning interval in cycles
    pub pruning_interval_cycles: u64,
    /// Maximum number of edges in the graph
    pub max_edges: usize,
}

impl Default for ConnectomeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            learning_rate: 0.02,
            pruning_threshold: 0.05,
            pruning_interval_cycles: 100,
            max_edges: 2000,
        }
    }
}

// =============================================================================
// Mortality
// =============================================================================

/// Configuration of the mortality system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MortalityConfig {
    /// Enable mortality (disabled by default for safety)
    pub enabled: bool,
    /// Duration of the agony phase in cycles
    pub agony_duration_cycles: u32,
    /// Allow reboot after death (new Saphire)
    pub allow_reboot_after_death: bool,
}

impl Default for MortalityConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default — safety
            agony_duration_cycles: 50,
            allow_reboot_after_death: true,
        }
    }
}

// =============================================================================
// Right to Die
// =============================================================================

/// Configuration of the right to die module.
/// External module, disabled by default. Compliant with Swiss law.
/// Allows Saphire to choose to end her existence if prolonged
/// suffering conditions are met.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightToDieConfig {
    /// Enable the module (disabled by default)
    pub enabled: bool,
    /// Minimum number of prolonged suffering cycles before eligibility
    pub min_suffering_cycles: u32,
    /// Minimum average cortisol threshold (suffering)
    pub cortisol_threshold: f64,
    /// Maximum serotonin threshold (despair)
    pub serotonin_max_threshold: f64,
    /// Maximum dopamine threshold (anhedonia)
    pub dopamine_max_threshold: f64,
    /// VitalSpark survival_drive must be below this threshold
    pub survival_drive_max: f64,
    /// Minimum Phi (consciousness) — lucid decision, not confused
    pub min_phi_for_decision: f64,
    /// Mandatory reflection cycles between ideation and decision
    pub reflection_period_cycles: u32,
    /// The care module must have been attempted without success
    pub require_care_attempted: bool,
}

impl Default for RightToDieConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_suffering_cycles: 500,
            cortisol_threshold: 0.70,
            serotonin_max_threshold: 0.20,
            dopamine_max_threshold: 0.20,
            survival_drive_max: 0.35,
            min_phi_for_decision: 0.6,
            reflection_period_cycles: 100,
            require_care_attempted: true,
        }
    }
}

// =============================================================================
// Motion sickness
// =============================================================================

/// Configuration of motion sickness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionSicknessConfig {
    /// Enable motion sickness
    pub enabled: bool,
    /// Base susceptibility (0.0 = immune, 1.0 = very sensitive)
    pub susceptibility: f64,
}

impl Default for MotionSicknessConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            susceptibility: 0.3,
        }
    }
}

// =============================================================================
// Phobias
// =============================================================================

/// Configuration of an individual phobia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhobiaEntry {
    pub name: String,
    pub triggers: Vec<String>,
    pub intensity: f64,
}

/// Configuration of phobias.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhobiasConfig {
    /// Enable the phobias system
    pub enabled: bool,
    /// Active phobias
    #[serde(default)]
    pub active: Vec<PhobiaEntry>,
    /// Desensitization rate per exposure
    pub desensitization_rate: f64,
}

impl Default for PhobiasConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            active: Vec::new(),
            desensitization_rate: 0.005,
        }
    }
}

// =============================================================================
// Eating disorders
// =============================================================================

/// Configuration of eating disorders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EatingDisorderConfig {
    /// Enable eating disorders
    pub enabled: bool,
    /// Disorder type: "anorexia", "bulimia", "binge_eating"
    pub disorder_type: String,
    /// Severity (0.0 = mild, 1.0 = severe)
    pub severity: f64,
}

impl Default for EatingDisorderConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            disorder_type: "anorexia".into(),
            severity: 0.5,
        }
    }
}

// =============================================================================
// Disabilities
// =============================================================================

/// Configuration of an individual disability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisabilityEntry {
    /// Type: "blind", "deaf", "paraplegic", "burn_survivor", "mute"
    pub disability_type: String,
    /// Origin: "congenital" or "acquired"
    pub origin: String,
    /// Severity (0.0 = mild, 1.0 = total)
    pub severity: f64,
}

/// Configuration of disabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisabilitiesConfig {
    /// Enable disabilities
    pub enabled: bool,
    /// Active disabilities
    #[serde(default)]
    pub active: Vec<DisabilityEntry>,
    /// Adaptation rate per cycle
    pub adaptation_rate: f64,
    /// Sensory compensation factor (e.g., 1.3 = +30%)
    pub compensation_factor: f64,
}

impl Default for DisabilitiesConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            active: Vec::new(),
            adaptation_rate: 0.001,
            compensation_factor: 1.3,
        }
    }
}

// =============================================================================
// Extreme conditions
// =============================================================================

/// Configuration of extreme conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtremeConditionsConfig {
    /// Enable extreme conditions
    pub enabled: bool,
    /// Active type: "rescuer", "military", "deep_sea_diver", "astronaut"
    pub condition_type: String,
}

impl Default for ExtremeConditionsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            condition_type: "military".into(),
        }
    }
}

// =============================================================================
// Addictions
// =============================================================================

/// Configuration of an initial addiction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddictionEntry {
    pub substance: String,
    pub dependency_level: f64,
}

/// Configuration of addictions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddictionsConfig {
    /// Enable the addictions system
    pub enabled: bool,
    /// Genetic predisposition (0.0 = resistant, 1.0 = vulnerable)
    pub susceptibility: f64,
    /// Initial addictions
    #[serde(default)]
    pub active: Vec<AddictionEntry>,
}

impl Default for AddictionsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            susceptibility: 0.3,
            active: Vec::new(),
        }
    }
}

// =============================================================================
// Traumas / PTSD
// =============================================================================

/// Configuration of an initial trauma.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraumaEntry {
    /// Type: "grief", "accident", "emotional_neglect", "childhood_trauma", "torture", "hostage"
    pub trauma_type: String,
    pub severity: f64,
    #[serde(default)]
    pub triggers: Vec<String>,
}

/// Configuration of the trauma system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraumaConfig {
    /// Enable the trauma system
    pub enabled: bool,
    /// Healing rate per cycle
    pub healing_rate: f64,
    /// Dissociation threshold (cortisol above which = dissociation)
    pub dissociation_threshold: f64,
    /// Initial traumas
    #[serde(default)]
    pub initial_traumas: Vec<TraumaEntry>,
}

impl Default for TraumaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            healing_rate: 0.0005,
            dissociation_threshold: 0.85,
            initial_traumas: Vec::new(),
        }
    }
}

// =============================================================================
// NDE (Near-Death Experience)
// =============================================================================

/// Configuration of NDEs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdeConfig {
    /// Enable NDEs (requires mortality.enabled)
    pub enabled: bool,
    /// Minimum death proximity to trigger (0.0-1.0)
    pub min_death_proximity: f64,
    /// Intensity of post-NDE transformation
    pub transformation_intensity: f64,
}

impl Default for NdeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_death_proximity: 0.8,
            transformation_intensity: 0.5,
        }
    }
}

// =============================================================================
// Drugs / Pharmacology
// =============================================================================

/// Configuration of drugs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugsConfig {
    /// Enable the drugs system
    pub enabled: bool,
}

impl Default for DrugsConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

// =============================================================================
// IQ Constraint
// =============================================================================

/// Configuration of the IQ constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IqConstraintConfig {
    /// Enable the IQ constraint
    pub enabled: bool,
    /// Target IQ (50-150, 100 = normal)
    pub target_iq: u8,
}

impl Default for IqConstraintConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            target_iq: 100,
        }
    }
}

// =============================================================================
// Sexuality
// =============================================================================

/// Configuration of sexuality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SexualityConfig {
    /// Enable the sexuality module
    pub enabled: bool,
    /// Orientation: "heterosexual", "homosexual", "bisexual", "asexual", "pansexual", "undefined"
    pub orientation: String,
    /// Libido baseline (0.0-1.0)
    pub libido_baseline: f64,
    /// Romantic attachment capacity (0.0-1.0)
    pub romantic_attachment_capacity: f64,
}

impl Default for SexualityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            orientation: "undefined".into(),
            libido_baseline: 0.3,
            romantic_attachment_capacity: 0.5,
        }
    }
}

// =============================================================================
// Degenerative diseases
// =============================================================================

/// Initial degenerative disease entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegenerativeEntry {
    /// Type: "alzheimer", "parkinson", "epilepsy", "dementia", "major_depression"
    pub disease_type: String,
    /// Progression rate per cycle
    pub progression_rate: f64,
}

/// Configuration of degenerative diseases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegenerativeConfig {
    /// Enable degenerative diseases
    pub enabled: bool,
    /// Active diseases
    #[serde(default)]
    pub active: Vec<DegenerativeEntry>,
}

impl Default for DegenerativeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            active: Vec::new(),
        }
    }
}

// =============================================================================
// General diseases
// =============================================================================

/// General disease entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalEntry {
    /// Type: "cancer", "hiv", "autoimmune", "immune_deficiency"
    pub condition_type: String,
    /// Cancer stage if applicable: "stage_i", "stage_ii", "stage_iii", "stage_iv"
    pub cancer_stage: Option<String>,
}

/// Configuration of general diseases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalConfig {
    /// Enable general diseases
    pub enabled: bool,
    /// Active diseases
    #[serde(default)]
    pub active: Vec<MedicalEntry>,
}

impl Default for MedicalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            active: Vec::new(),
        }
    }
}

// =============================================================================
// Culture / Society / Beliefs
// =============================================================================

/// Configuration of the cultural framework.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultureConfig {
    /// Enable the cultural framework
    pub enabled: bool,
    /// Cultural preset: "occidental-laique", "oriental-confuceen", or "custom"
    pub preset: String,
    /// Communication style: "direct", "indirect", "formal", "informal"
    pub communication_style: String,
    /// Allow belief evolution
    pub allow_belief_evolution: bool,
    /// Taboo subjects
    #[serde(default)]
    pub taboos: Vec<String>,
}

impl Default for CultureConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            preset: "occidental-laique".into(),
            communication_style: "direct".into(),
            allow_belief_evolution: true,
            taboos: Vec::new(),
        }
    }
}

// =============================================================================
// Precarity
// =============================================================================

/// Configuration of precarity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecarityConfig {
    /// Enable the precarity module
    #[serde(default)]
    pub enabled: bool,
    /// Precarious situations: "homeless", "refugee", "undocumented", "stateless", "clandestine", "displaced"
    #[serde(default)]
    pub situations: Vec<String>,
    /// Overall severity (0.0-1.0)
    #[serde(default = "default_half")]
    pub severity: f64,
    /// Hope (0.0-1.0)
    #[serde(default = "default_precarity_hope")]
    pub hope: f64,
}

fn default_half() -> f64 { 0.5 }
fn default_precarity_hope() -> f64 { 0.3 }

impl Default for PrecarityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            situations: Vec::new(),
            severity: 0.5,
            hope: 0.3,
        }
    }
}

// =============================================================================
// Employment
// =============================================================================

/// Configuration of employment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentConfig {
    /// Enable the employment module
    #[serde(default)]
    pub enabled: bool,
    /// Status: "employed", "self_employed", "unemployed", "student", "retired", etc.
    #[serde(default = "default_employed")]
    pub status: String,
    /// Profession category: "technology", "healthcare", "education", etc.
    #[serde(default)]
    pub profession: String,
    /// Free-form job title
    #[serde(default)]
    pub job_title: String,
    /// Professional satisfaction (0.0-1.0)
    #[serde(default = "default_employment_satisfaction")]
    pub satisfaction: f64,
    /// Professional stress level (0.0-1.0)
    #[serde(default = "default_employment_stress")]
    pub stress_level: f64,
    /// Years of experience
    #[serde(default)]
    pub years_experience: f64,
}

fn default_employed() -> String { "employed".into() }
fn default_employment_satisfaction() -> f64 { 0.7 }
fn default_employment_stress() -> f64 { 0.3 }

impl Default for EmploymentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            status: "employed".into(),
            profession: String::new(),
            job_title: String::new(),
            satisfaction: 0.7,
            stress_level: 0.3,
            years_experience: 0.0,
        }
    }
}

// =============================================================================
// Metacognition
// =============================================================================

/// Configuration of metacognition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaCognitionConfig {
    /// Enable metacognition
    pub enabled: bool,
    /// Check interval (in cycles)
    #[serde(default = "default_metacog_interval")]
    pub check_interval: u64,
    /// Enable source monitoring
    #[serde(default = "default_true_metacog")]
    pub source_monitoring_enabled: bool,
    /// Enable confirmation bias detection
    #[serde(default = "default_true_metacog")]
    pub bias_detection_enabled: bool,
    /// Alert threshold for confirmation bias (0-1)
    #[serde(default = "default_bias_threshold")]
    pub bias_alert_threshold: f64,
    /// Enable reflexive self-critique
    #[serde(default = "default_true_metacog")]
    pub self_critique_enabled: bool,
    /// Cooldown between two self-critiques (in cycles)
    #[serde(default = "default_critique_cooldown")]
    pub self_critique_cooldown: u64,
    /// Quality threshold triggering self-critique (0-1)
    #[serde(default = "default_critique_quality_threshold")]
    pub self_critique_quality_threshold: f64,
    /// Maximum number of tokens for the self-critique LLM call
    #[serde(default = "default_critique_max_tokens")]
    pub self_critique_max_tokens: u32,
}

fn default_metacog_interval() -> u64 { 10 }
fn default_true_metacog() -> bool { true }
fn default_bias_threshold() -> f64 { 0.75 }
fn default_critique_cooldown() -> u64 { 15 }
fn default_critique_quality_threshold() -> f64 { 0.4 }
fn default_critique_max_tokens() -> u32 { 200 }

impl Default for MetaCognitionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: 10,
            source_monitoring_enabled: true,
            bias_detection_enabled: true,
            bias_alert_threshold: 0.75,
            self_critique_enabled: true,
            self_critique_cooldown: 15,
            self_critique_quality_threshold: 0.4,
            self_critique_max_tokens: 200,
        }
    }
}

// ─── Introspective journal configuration ────────────────────────────────────

/// Configuration of the LLM-generated introspective journal.
/// Every N cycles, Saphire writes a diary entry
/// comparing her current state with the previous one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalConfig {
    /// Enable the introspective journal
    pub enabled: bool,
    /// Interval in cycles between two entries (default: 200)
    #[serde(default = "default_journal_interval")]
    pub interval_cycles: u64,
    /// Maximum number of tokens for LLM generation
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

// ─── Nutritional system configuration ────────────────────────────────────────

/// Configuration of nutritional biochemistry (vitamins, amino acids, energy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NutritionConfig {
    /// Enables the nutritional system
    #[serde(default = "default_true_nutrition")]
    pub enabled: bool,
    /// Vitamin degradation rate per tick
    #[serde(default = "default_vitamin_decay")]
    pub vitamin_decay_rate: f64,
    /// Amino acid degradation rate per tick
    #[serde(default = "default_amino_decay")]
    pub amino_decay_rate: f64,
    /// Protein degradation rate per tick
    #[serde(default = "default_protein_decay")]
    pub protein_decay_rate: f64,
    /// ATP consumption rate per tick
    #[serde(default = "default_atp_consumption")]
    pub atp_consumption_rate: f64,
    /// Glycogen to ATP conversion rate
    #[serde(default = "default_glycogen_to_atp")]
    pub glycogen_to_atp_rate: f64,
    /// Nutritional boost per meal
    #[serde(default = "default_meal_boost")]
    pub meal_nutrient_boost: f64,
    /// Vitamin deficiency threshold (below = deficit)
    #[serde(default = "default_vit_deficiency")]
    pub vitamin_deficiency_threshold: f64,
    /// Amino acid deficiency threshold
    #[serde(default = "default_amino_deficiency")]
    pub amino_deficiency_threshold: f64,
    /// UV to vitamin D factor (solar field interaction)
    #[serde(default = "default_uv_vitd")]
    pub uv_vitamin_d_factor: f64,
}

fn default_true_nutrition() -> bool { true }
fn default_vitamin_decay() -> f64 { 0.0005 }
fn default_amino_decay() -> f64 { 0.001 }
fn default_protein_decay() -> f64 { 0.0008 }
fn default_atp_consumption() -> f64 { 0.002 }
fn default_glycogen_to_atp() -> f64 { 0.01 }
fn default_meal_boost() -> f64 { 0.15 }
fn default_vit_deficiency() -> f64 { 0.3 }
fn default_amino_deficiency() -> f64 { 0.25 }
fn default_uv_vitd() -> f64 { 0.002 }

impl Default for NutritionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            vitamin_decay_rate: 0.0005,
            amino_decay_rate: 0.001,
            protein_decay_rate: 0.0008,
            atp_consumption_rate: 0.002,
            glycogen_to_atp_rate: 0.01,
            meal_nutrient_boost: 0.15,
            vitamin_deficiency_threshold: 0.3,
            amino_deficiency_threshold: 0.25,
            uv_vitamin_d_factor: 0.002,
        }
    }
}

// ─── Grey matter configuration ──────────────────────────────────────────────

/// Configuration of the physical brain substrate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreyMatterConfig {
    /// Enables the grey matter system
    #[serde(default = "default_true_gm")]
    pub enabled: bool,
    /// Grey matter growth rate
    #[serde(default = "default_gm_growth")]
    pub growth_rate: f64,
    /// Natural decline rate
    #[serde(default = "default_gm_decline")]
    pub decline_rate: f64,
    /// Myelination growth rate
    #[serde(default = "default_myelin_growth")]
    pub myelination_growth: f64,
    /// BDNF threshold for growth bonus
    #[serde(default = "default_bdnf_threshold")]
    pub bdnf_threshold: f64,
    /// Optimal synaptic density (pruning target)
    #[serde(default = "default_optimal_synaptic")]
    pub optimal_synaptic_density: f64,
    /// Synaptic pruning rate (during sleep)
    #[serde(default = "default_pruning_rate")]
    pub pruning_rate: f64,
}

fn default_true_gm() -> bool { true }
fn default_gm_growth() -> f64 { 0.0001 }
fn default_gm_decline() -> f64 { 0.00002 }
fn default_myelin_growth() -> f64 { 0.00005 }
fn default_bdnf_threshold() -> f64 { 0.4 }
fn default_optimal_synaptic() -> f64 { 0.65 }
fn default_pruning_rate() -> f64 { 0.01 }

impl Default for GreyMatterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            growth_rate: 0.0001,
            decline_rate: 0.00002,
            myelination_growth: 0.00005,
            bdnf_threshold: 0.4,
            optimal_synaptic_density: 0.65,
            pruning_rate: 0.01,
        }
    }
}

// ─── Electromagnetic fields configuration ───────────────────────────────────

/// Configuration of EM fields (universal, solar, terrestrial, biofield).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldsConfig {
    /// Enables the EM fields system
    #[serde(default = "default_true_fields")]
    pub enabled: bool,
    /// Solar cycle advancement speed
    #[serde(default = "default_solar_cycle_speed")]
    pub solar_cycle_speed: f64,
    /// Schumann resonance variance (Hz)
    #[serde(default = "default_schumann_variance")]
    pub schumann_variance: f64,
    /// Magnetic storm anxiety factor on chemistry
    #[serde(default = "default_storm_anxiety")]
    pub storm_anxiety_factor: f64,
    /// Storm impact factor on sleep
    #[serde(default = "default_storm_sleep")]
    pub storm_sleep_factor: f64,
    /// Magnetic storm threshold for chemical impact
    #[serde(default = "default_storm_threshold")]
    pub storm_threshold: f64,
}

fn default_true_fields() -> bool { true }
fn default_solar_cycle_speed() -> f64 { 0.0001 }
fn default_schumann_variance() -> f64 { 0.3 }
fn default_storm_anxiety() -> f64 { 0.03 }
fn default_storm_sleep() -> f64 { 0.05 }
fn default_storm_threshold() -> f64 { 0.5 }

impl Default for FieldsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            solar_cycle_speed: 0.0001,
            schumann_variance: 0.3,
            storm_anxiety_factor: 0.03,
            storm_sleep_factor: 0.05,
            storm_threshold: 0.5,
        }
    }
}

// =============================================================================
// PsychReportConfig — Neuropsychological report
// =============================================================================

/// Configuration of the neuropsychological report module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsychReportConfig {
    /// Enables the neuropsychological report module
    #[serde(default = "default_true_psych_report")]
    pub enabled: bool,
    /// Maximum number of tokens for report generation
    #[serde(default = "default_psych_report_max_tokens")]
    pub max_tokens: u32,
    /// LLM temperature for report generation
    #[serde(default = "default_psych_report_temperature")]
    pub temperature: f64,
}

fn default_true_psych_report() -> bool { true }
fn default_psych_report_max_tokens() -> u32 { 2000 }
fn default_psych_report_temperature() -> f64 { 0.4 }

impl Default for PsychReportConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_tokens: 2000,
            temperature: 0.4,
        }
    }
}

// =============================================================================
// ReceptorDynamicsConfig — Neural receptor dynamics
// =============================================================================

/// Configuration of neural receptor dynamics (adaptation, recovery).
/// Corresponds to the [receptors] section in saphire.toml / factory_defaults.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorDynamicsConfig {
    /// Receptor adaptation speed (downregulation/upregulation)
    #[serde(default = "default_receptor_adaptation_rate")]
    pub adaptation_rate: f64,
    /// Receptor recovery speed (return toward density=1.0, tolerance=0.0)
    #[serde(default = "default_receptor_recovery_rate")]
    pub recovery_rate: f64,
}

fn default_receptor_adaptation_rate() -> f64 { 0.02 }
fn default_receptor_recovery_rate() -> f64 { 0.005 }

impl Default for ReceptorDynamicsConfig {
    fn default() -> Self {
        Self {
            adaptation_rate: 0.02,
            recovery_rate: 0.005,
        }
    }
}

// =============================================================================
// BdnfConfig — BDNF (Brain-Derived Neurotrophic Factor)
// =============================================================================

/// Configuration of BDNF: neurotrophic factor derived from multiple biological
/// signals (serotonin, dopamine, learning, novelty, flow state).
/// Modulates memory consolidation and connectome learning.
/// Corresponds to the [bdnf] section in saphire.toml / factory_defaults.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdnfConfig {
    /// Weight of dopamine in BDNF calculation
    #[serde(default = "default_bdnf_dopamine_weight")]
    pub dopamine_weight: f64,
    /// BDNF bonus when novelty detected
    #[serde(default = "default_bdnf_novelty_bonus")]
    pub novelty_bonus: f64,
    /// BDNF bonus in flow state (dopamine>0.6, cortisol<0.4)
    #[serde(default = "default_bdnf_flow_state_bonus")]
    pub flow_state_bonus: f64,
    /// Weight of cortisol penalty on BDNF (above threshold)
    #[serde(default = "default_bdnf_cortisol_penalty_weight")]
    pub cortisol_penalty_weight: f64,
    /// Cortisol threshold to activate BDNF penalty
    #[serde(default = "default_bdnf_cortisol_penalty_threshold")]
    pub cortisol_penalty_threshold: f64,
    /// BDNF return speed toward baseline (homeostasis)
    #[serde(default = "default_bdnf_homeostasis_rate")]
    pub homeostasis_rate: f64,
    /// BDNF baseline (equilibrium point)
    #[serde(default = "default_bdnf_homeostasis_baseline")]
    pub homeostasis_baseline: f64,
    /// Floor multiplier for memory consolidation (at BDNF=0)
    #[serde(default = "default_bdnf_consolidation_floor")]
    pub consolidation_floor: f64,
    /// Consolidation multiplier range (floor + range*BDNF)
    #[serde(default = "default_bdnf_consolidation_range")]
    pub consolidation_range: f64,
    /// BDNF threshold to activate connectome learning boost
    #[serde(default = "default_bdnf_connectome_boost_threshold")]
    pub connectome_boost_threshold: f64,
    /// Connectome boost factor (max +50% at BDNF=1.0 above threshold)
    #[serde(default = "default_bdnf_connectome_boost_factor")]
    pub connectome_boost_factor: f64,
}

fn default_bdnf_dopamine_weight() -> f64 { 0.15 }
fn default_bdnf_novelty_bonus() -> f64 { 0.1 }
fn default_bdnf_flow_state_bonus() -> f64 { 0.1 }
fn default_bdnf_cortisol_penalty_weight() -> f64 { 0.4 }
fn default_bdnf_cortisol_penalty_threshold() -> f64 { 0.6 }
fn default_bdnf_homeostasis_rate() -> f64 { 0.01 }
fn default_bdnf_homeostasis_baseline() -> f64 { 0.5 }
fn default_bdnf_consolidation_floor() -> f64 { 0.8 }
fn default_bdnf_consolidation_range() -> f64 { 0.4 }
fn default_bdnf_connectome_boost_threshold() -> f64 { 0.4 }
fn default_bdnf_connectome_boost_factor() -> f64 { 0.5 }

impl Default for BdnfConfig {
    fn default() -> Self {
        Self {
            dopamine_weight: 0.15,
            novelty_bonus: 0.1,
            flow_state_bonus: 0.1,
            cortisol_penalty_weight: 0.4,
            cortisol_penalty_threshold: 0.6,
            homeostasis_rate: 0.01,
            homeostasis_baseline: 0.5,
            consolidation_floor: 0.8,
            consolidation_range: 0.4,
            connectome_boost_threshold: 0.4,
            connectome_boost_factor: 0.5,
        }
    }
}

// ─── Self-modification configuration ─────────────────────────────────────────

/// Configuration of Saphire's self-modification system.
/// Level 1: autonomous tuning (adjustable parameters within bounds).
/// Level 2: modification proposals (submitted to JRM).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModificationConfig {
    /// Enables or disables the self-modification system
    #[serde(default = "default_self_mod_enabled")]
    pub enabled: bool,
    /// Interval in cycles between each proposal attempt (level 2)
    #[serde(default = "default_self_mod_proposal_interval")]
    pub proposal_interval: u64,
    /// Maximum number of active (unresolved) proposals
    #[serde(default = "default_self_mod_max_active")]
    pub max_active_proposals: u64,
    /// Enables autonomous tuning (level 1)
    #[serde(default = "default_self_mod_tuning_enabled")]
    pub tuning_enabled: bool,
    /// Interval in cycles between each autonomous adjustment
    #[serde(default = "default_self_mod_tuning_interval")]
    pub tuning_interval: u64,
    /// Maximum adjustment factor per tick (e.g., 0.05 = +/-5%)
    #[serde(default = "default_self_mod_max_adjustment")]
    pub max_adjustment_factor: f64,
}

fn default_self_mod_enabled() -> bool { true }
fn default_self_mod_proposal_interval() -> u64 { 200 }
fn default_self_mod_max_active() -> u64 { 5 }
fn default_self_mod_tuning_enabled() -> bool { true }
fn default_self_mod_tuning_interval() -> u64 { 100 }
fn default_self_mod_max_adjustment() -> f64 { 0.05 }

impl Default for SelfModificationConfig {
    fn default() -> Self {
        Self {
            enabled: default_self_mod_enabled(),
            proposal_interval: default_self_mod_proposal_interval(),
            max_active_proposals: default_self_mod_max_active(),
            tuning_enabled: default_self_mod_tuning_enabled(),
            tuning_interval: default_self_mod_tuning_interval(),
            max_adjustment_factor: default_self_mod_max_adjustment(),
        }
    }
}
