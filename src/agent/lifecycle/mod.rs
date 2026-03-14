// =============================================================================
// lifecycle/mod.rs — Main life loop of the Saphire agent
// =============================================================================
//
// This module is the heart of Saphire. It contains:
//  - `SaphireAgent`: the main structure that unifies all subsystems.
//  - The stimulus processing pipeline (NLP → brain modules → consensus
//    → emotions → consciousness → regulation → chemical feedback).
//  - Human message handling (`handle_human_message`).
//  - Autonomous thought (`autonomous_think`).
//  - Enriched web search (`try_web_search`).
//  - Clean shutdown with nocturnal memory consolidation.
//  - Internal state broadcast to the WebSocket for the interface.
//  - Interactive controls (modification of baselines, thresholds, etc.).
//
// Main dependencies:
//  - `config`: overall Saphire configuration (personality, LLM, memory...).
//  - `neurochemistry`: simulation of 7 neurotransmitters.
//  - `emotions`: emotional state computation from chemistry.
//  - `consciousness`: consciousness level evaluation (phi, IIT).
//  - `consensus`: decision by weighted vote of the 3 brain modules.
//  - `regulation`: Asimov's laws and security filtering.
//  - `modules`: the 3 brain modules (reptilian, limbic, neocortex).
//  - `nlp`: natural language analysis (NLP = Natural Language Processing).
//  - `llm`: interface with the LLM (Large Language Model) backend.
//  - `memory`: 3-level memory (working, episodic, long-term).
//  - `profiling`: OCEAN psychological profiling (Openness, Conscientiousness,
//    Extraversion, Agreeableness, Neuroticism).
//  - `knowledge`: autonomous web search to enrich thoughts.
//  - `world`: world context (weather, time, season, birthday).
//  - `tuning`: auto-tuning of internal coefficients.
//  - `plugins`: plugin system and brain events.
//
// Place in architecture:
//  This file is the functional entry point of the agent. `main.rs`
//  instantiates a `SaphireAgent`, calls `boot()`, then launches the
//  life loop that alternates autonomous thoughts and responses to humans.
// =============================================================================

mod pipeline;
mod conversation;
mod thinking;
mod thinking_perception;
mod thinking_preparation;
mod thinking_processing;
mod thinking_reflection;
mod factory_reset;
mod broadcast;
mod algorithms_integration;
mod moral;
mod controls;
mod persistence;
pub mod sleep_tick;
mod sleep_algorithms;
pub mod psych_report;

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc;
use tokio::time::Duration;

use crate::config::SaphireConfig;
use crate::neurochemistry::{NeuroChemicalState, NeuroBaselines};
use crate::emotions::{EmotionalState, Mood};
use crate::consciousness::ConsciousnessEvaluator;
use crate::regulation::RegulationEngine;
use crate::modules::reptilian::ReptilianModule;
use crate::modules::limbic::LimbicModule;
use crate::modules::neocortex::NeocortexModule;
use crate::nlp::NlpPipeline;
use crate::db::SaphireDb;
use crate::llm::LlmBackend;
use crate::plugins::{PluginManager, BrainEvent};
use crate::agent::thought_engine::ThoughtEngine;
use crate::agent::identity::SaphireIdentity;
use crate::agent::boot;
use crate::tuning::CoefficientTuner;
use crate::knowledge::WebKnowledge;
use crate::world::WorldContext;
use crate::memory::WorkingMemory;
use crate::memory::consolidation;
use crate::vectorstore::encoder::TextEncoder;
use crate::profiling::{SelfProfiler, HumanProfiler};
use crate::logging::{SaphireLogger, LogLevel, LogCategory};

/// Incoming user message, sent from the web server via an MPSC channel.
///
/// The web server receives the user's text and places it in a `UserMessage`.
/// The response is broadcast directly via the WebSocket broadcast by the main loop.
pub struct UserMessage {
    /// Raw text sent by the user
    pub text: String,
    /// Name of the interlocutor (identified by the frontend)
    pub username: String,
}

/// Shared state between the agent's life loop and the web server.
///
/// This structure is cloneable and protected by `Arc` for concurrent
/// access from the HTTP/WebSocket handlers and the main loop.
#[derive(Clone)]
pub struct SharedState {
    /// Broadcast channel to push state updates to the WebSocket
    pub ws_tx: Arc<tokio::sync::broadcast::Sender<String>>,

    /// MPSC (Multi-Producer, Single-Consumer) channel to send
    /// user messages to the agent's life loop
    pub user_tx: mpsc::Sender<UserMessage>,

    /// Atomic shutdown flag: set to `true` to request shutdown
    pub shutdown: Arc<AtomicBool>,
}

/// The Saphire agent — complete brain and life loop.
///
/// This structure unifies all the subsystems that compose Saphire:
/// neurochemistry, emotions, consciousness, brain modules, memory, LLM, etc.
/// It is instantiated once in `main.rs` and driven by the asynchronous life loop.
pub struct SaphireAgent {
    // ─── Brain components ─────────────────────────────────────
    /// Current neurochemical state (7 simulated neurotransmitters:
    /// dopamine, cortisol, serotonin, adrenaline, oxytocin, endorphin, noradrenaline)
    pub chemistry: NeuroChemicalState,

    /// Baseline levels towards which neurotransmitters tend to return via homeostasis
    baselines: NeuroBaselines,

    /// Long-term mood (valence + arousal), more stable than momentary emotions
    pub mood: Mood,

    /// Persistent identity of Saphire (name, stats, values, self-description)
    pub identity: SaphireIdentity,

    /// Consciousness evaluator based on IIT (Integrated Information Theory).
    /// Computes the consciousness level (phi) at each cycle.
    pub(crate) consciousness: ConsciousnessEvaluator,

    /// Ethical regulation engine (Asimov's laws, content filtering)
    regulation: RegulationEngine,

    /// NLP pipeline (Natural Language Processing) to analyze incoming messages
    nlp: NlpPipeline,

    /// Autonomous thought engine (DMN = Default Mode Network) with UCB1 bandit
    thought_engine: ThoughtEngine,

    /// Internal coefficient auto-tuner (consensus thresholds, weights, etc.)
    tuner: CoefficientTuner,

    // ─── Brain modules ────────────────────────────────────────
    /// Reptilian module: instinctive reactions (survival, danger, reflexes)
    reptilian: ReptilianModule,

    /// Limbic module: emotional processing (pleasure, fear, attachment)
    limbic: LimbicModule,

    /// Neocortex module: logical reasoning, analysis, planning
    neocortex: NeocortexModule,

    // ─── Infrastructure ───────────────────────────────────────
    /// LLM (Large Language Model) backend: trait interface to call
    /// different language models (Ollama, OpenAI, etc.)
    llm: Box<dyn LlmBackend>,

    /// PostgreSQL database connection (optional for DB-less mode)
    pub db: Option<SaphireDb>,

    /// Plugin manager with brain event broadcasting
    plugins: PluginManager,

    /// Autonomous web search module to enrich thoughts
    pub knowledge: WebKnowledge,

    /// World context: time, season, weather, birthday
    world: WorldContext,

    /// Virtual body: beating heart, somatic signals, interoception
    pub body: crate::body::VirtualBody,

    /// 3-layer ethical framework (Swiss law, Asimov, personal ethics)
    pub ethics: crate::ethics::EthicalFramework,

    /// Spark of life: emergent survival instinct
    pub vital_spark: crate::vital::VitalSpark,
    /// Intuition engine: unconscious pattern-matching
    pub intuition: crate::vital::IntuitionEngine,
    /// Premonition engine: predictive anticipation
    pub premonition: crate::vital::PremonitionEngine,
    /// History of the 7 molecules for trend computation
    chemistry_history: Vec<[f64; 7]>,

    /// The Sensorium: 5 fundamental senses + emergent senses
    pub sensorium: crate::senses::Sensorium,

    /// Micro neural network (17->24->10->4) — 4th voice in consensus
    pub micro_nn: crate::neural::MicroNeuralNet,

    // ─── 3-level memory ──────────────────────────────────────
    /// Working memory: limited-capacity queue with temporal decay.
    /// Contains the most recent items (messages, thoughts, responses).
    working_memory: WorkingMemory,

    /// Encoder to generate embedding vectors (similarity search).
    /// OllamaEncoder (semantic, 768-dim) if available, otherwise LocalEncoder (FNV-1a).
    encoder: Box<dyn TextEncoder>,

    /// Number of the last cycle where a memory consolidation was performed
    last_consolidation_cycle: u64,

    /// Identifier of the current conversation (format "conv_{timestamp}")
    /// or None if no conversation is active
    conversation_id: Option<String>,

    // ─── OCEAN psychological profiling ────────────────────────
    /// Saphire's self-profiler (OCEAN self-analysis)
    /// OCEAN = Openness, Conscientiousness, Extraversion, Agreeableness, Neuroticism
    self_profiler: SelfProfiler,

    /// Profiler of humans that Saphire interacts with
    human_profiler: HumanProfiler,

    // ─── Configuration ────────────────────────────────────────
    /// Overall configuration loaded from the TOML file
    config: SaphireConfig,

    /// Interval between two consecutive autonomous thoughts
    thought_interval: Duration,

    // ─── Current state ──────────────────────────────────────
    /// Total cycle counter since the start of this session
    pub cycle_count: u64,

    /// Identifier of the current session in PostgreSQL
    pub session_id: i64,

    /// Atomic flag indicating whether the LLM is currently being called
    /// (prevents concurrent LLM calls)
    llm_busy: Arc<AtomicBool>,

    /// Average response time from the LLM (exponential moving average, in seconds)
    avg_response_time: f64,

    /// Last computed dominant emotion (for display and context)
    last_emotion: String,

    /// Last computed consciousness level (for display)
    pub(crate) last_consciousness: f64,

    /// Last selected autonomous thought type (for the interface)
    last_thought_type: String,

    /// `true` if a conversation with a human is ongoing
    in_conversation: bool,

    /// Last LLM responses in conversation (max 5) for anti-repetition detection
    recent_responses: Vec<String>,

    /// History of recent conversation exchanges (human, Saphire response)
    /// Injected as multi-turn in the LLM call for context (max 5)
    chat_history: Vec<(String, String)>,

    /// `true` if Saphire's birthday has already been acknowledged today
    /// (prevents triggering the event multiple times in the same day)
    birthday_acknowledged_today: bool,

    /// Counter of MoralReflection thoughts (for formulation threshold)
    pub(crate) moral_reflection_count: u64,

    /// Cycles since the last successful ethical formulation
    pub(crate) cycles_since_last_formulation: u64,

    /// Cycles since the last learning vector formulated
    pub(crate) cycles_since_last_nn_learning: u64,

    /// Pending human feedback request (RLHF)
    feedback_pending: Option<thinking::FeedbackRequest>,
    /// Cycle counter since the last feedback request
    cycles_since_last_feedback: u64,

    // ─── Communication ────────────────────────────────────────
    /// Broadcast channel to the WebSocket for real-time state updates
    ws_tx: Option<Arc<tokio::sync::broadcast::Sender<String>>>,

    /// Centralized logger (optional, injected from main.rs)
    logger: Option<Arc<tokio::sync::Mutex<SaphireLogger>>>,

    /// Logs database (optional)
    logs_db: Option<Arc<crate::logging::db::LogsDb>>,

    /// Algorithm orchestrator (LLM ↔ Rust algorithms bridge)
    pub orchestrator: crate::algorithms::orchestrator::AlgorithmOrchestrator,

    // ─── High-level orchestrators ──────────────────────────────
    /// Dream orchestrator (sleep, phases, oneiric generation)
    pub dream_orch: crate::orchestrators::dreams::DreamOrchestrator,
    /// Desire orchestrator (aspirations, milestones, fundamental needs)
    pub desire_orch: crate::orchestrators::desires::DesireOrchestrator,
    /// Learning orchestrator (experience → lesson → change)
    pub learning_orch: crate::orchestrators::learning::LearningOrchestrator,
    /// Attention orchestrator (selective focus, fatigue, periphery)
    pub attention_orch: crate::orchestrators::attention::AttentionOrchestrator,
    /// Healing orchestrator (wounds, coping, resilience)
    pub healing_orch: crate::orchestrators::healing::HealingOrchestrator,
    /// Neurodivergent cognitive profiles orchestrator
    pub cognitive_profile_orch: crate::orchestrators::cognitive_profile::CognitiveProfileOrchestrator,
    /// Personality presets orchestrator (character archetypes)
    pub personality_preset_orch: crate::orchestrators::personality_preset::PersonalityPresetOrchestrator,
    /// Psychological frameworks (Freud, Maslow, Toltec, Jung, Goleman, Flow)
    pub psychology: crate::psychology::PsychologyFramework,
    /// Cycle counter in negative emotion (for melancholy detection)
    negative_emotion_cycles: u64,
    /// Hours since last human message (for loneliness detection)
    hours_since_human: f64,
    /// Number of recent system errors (for technical trauma detection)
    system_errors: u32,

    // ─── Chemical self-monitoring ────────────────────────────
    /// Consecutive cycles with cortisol < 0.10
    cortisol_flat_cycles: u64,
    /// Consecutive cycles with dopamine > 0.85
    dopamine_ceiling_cycles: u64,
    /// Consecutive cycles with serotonin > 0.85
    serotonin_ceiling_cycles: u64,
    /// Ring-buffer of the 200 last dominant emotions
    recent_emotions: VecDeque<String>,
    /// Ring-buffer of the 200 last valences
    recent_valences: VecDeque<f64>,
    /// Last computed valence
    last_valence: f64,

    // ─── Sleep + Subconscious ──────────────────────────────
    /// Sleep system (pressure, phases, history)
    pub sleep: crate::sleep::SleepSystem,
    /// Subconscious module (associations, repression, incubation, priming)
    pub subconscious: crate::psychology::Subconscious,
    /// Last clusters detected during sleep
    pub(crate) sleep_last_clusters: Option<serde_json::Value>,

    // ─── Primary needs ──────────────────────────────────────
    /// Hunger and thirst drives (derived from physiology)
    pub needs: crate::needs::PrimaryNeeds,

    // ─── Hormonal system ──────────────────────────────────
    /// 8 hormones, receptors, circadian/ultradian cycles
    pub hormonal_system: crate::hormones::HormonalSystem,

    // ─── Hardware profile ──────────────────────────────────
    /// Hardware profile detected at startup (GPU, CPU, RAM, Ollama)
    pub hardware_profile: Option<crate::hardware::HardwareProfile>,

    // ─── Genome / DNA ────────────────────────────────────
    /// Deterministic genome generated from the configured seed
    pub genome: Option<crate::genome::Genome>,

    // ─── Connectome ────────────────────────────────────
    /// Dynamic neural connection graph (autopoiesis)
    pub connectome: crate::connectome::Connectome,

    // ─── Conditions / Afflictions ────────────────────
    /// Motion sickness
    pub motion_sickness: crate::conditions::motion_sickness::MotionSickness,
    /// Phobia manager
    pub phobia_manager: crate::conditions::phobias::PhobiaManager,
    /// Eating disorder (anorexia, bulimia, binge eating)
    pub eating_disorder: Option<crate::conditions::eating::EatingDisorder>,
    /// Disability manager (blindness, deafness, etc.)
    pub disability_manager: crate::conditions::disabilities::DisabilityManager,
    /// Extreme condition manager (military, first responder, etc.)
    pub extreme_condition_mgr: crate::conditions::extreme::ExtremeConditionManager,
    /// Addiction manager
    pub addiction_manager: crate::conditions::addictions::AddictionManager,
    /// PTSD state (traumas, flashbacks, hypervigilance)
    pub ptsd: crate::conditions::trauma::PtsdState,
    /// Near-death experience (NDE)
    pub nde: crate::conditions::nde::NdeExperience,
    /// Pharmacological effects manager (active drugs)
    pub drug_manager: crate::conditions::drugs::DrugManager,
    /// Limiting IQ constraint
    pub iq_constraint: Option<crate::conditions::iq_constraint::IqConstraint>,
    /// Sexuality module (libido, orientation, attachment)
    pub sexuality: Option<crate::conditions::sexuality::SexualityModule>,
    /// Degenerative disease manager
    pub degenerative_mgr: crate::conditions::degenerative::DegenerativeManager,
    /// General illness manager
    pub medical_mgr: crate::conditions::medical::MedicalManager,
    /// Cultural framework (values, beliefs, taboos)
    pub culture: Option<crate::conditions::culture::CulturalFramework>,
    /// Precarity state (homeless, refugee, undocumented, etc.)
    pub precarity: Option<crate::conditions::precarity::PrecariousState>,
    /// Employment state (job, satisfaction, stress)
    pub employment: Option<crate::conditions::employment::EmploymentState>,

    // ─── Relationships and attachments ──────────────────────────
    /// Affective bond network (friends, mentors, rivals, etc.)
    pub relationships: crate::relationships::RelationshipNetwork,

    // ─── Metacognition and Turing ──────────────────────────────
    /// Metacognition engine (thought quality, repetitions, biases)
    pub metacognition: crate::metacognition::MetaCognitionEngine,

    // ─── Advanced cognitive modules ──────────────────────────────
    /// Theory of Mind (models the interlocutor)
    pub tom: crate::cognition::tom::TheoryOfMindEngine,
    /// Structured inner monologue (thought chain)
    pub inner_monologue: crate::cognition::inner_monologue::InnerMonologue,
    /// Cognitive dissonance (Festinger)
    pub dissonance: crate::cognition::cognitive_dissonance::CognitiveDissonanceEngine,
    /// Prospective memory (deferred intentions)
    pub prospective_mem: crate::cognition::prospective_memory::ProspectiveMemory,
    /// Narrative identity (McAdams)
    pub narrative_identity: crate::cognition::narrative_identity::NarrativeIdentity,
    /// Analogical reasoning
    pub analogical: crate::cognition::analogical_reasoning::AnalogicalReasoning,
    /// Cognitive load (Sweller)
    pub cognitive_load: crate::cognition::cognitive_load::CognitiveLoadState,
    /// Mental imagery
    pub imagery: crate::cognition::mental_imagery::MentalImageryEngine,
    /// Sentiment engine (lasting affective states)
    pub sentiments: crate::cognition::sentiments::SentimentEngine,
    /// Automatic cognitive state clustering (PCA + K-Means)
    pub state_clustering: crate::cognition::state_clustering::StateClustering,

    // ─── Game design algorithms ────────────────────────────────
    /// Attentional influence map (topics x urgency)
    pub influence_map: crate::simulation::influence_map::InfluenceMap,
    /// Cognitive FSM (Awake, Focus, Reverie, Stress, Flow, Rest)
    pub cognitive_fsm: crate::simulation::cognitive_fsm::CognitiveFsm,
    /// Emotional steering engine (seek/flee/wander in valence-arousal)
    pub steering_engine: crate::simulation::steering::SteeringEngine,
    /// Last action recommended by the Behavior Tree
    pub bt_last_action: Option<String>,
    /// Blackboard: inter-algorithm coordination board
    pub blackboard: crate::simulation::blackboard::Blackboard,
    /// Conversational Utility AI
    pub utility_ai: crate::simulation::utility_ai::UtilityAI,
    /// Hierarchical Task Network planner (HTN)
    pub htn_planner: crate::simulation::htn::HtnPlanner,

    /// Pre-cached static system prompt (Track 2: Ollama KV-cache).
    /// Recomputed when personal ethics change.
    cached_system_prompt: String,
    /// Moral reflection counter at the time of the last cache
    cached_moral_count: u64,

    // ─── Advanced neuroscience modules ──────────────────────────
    /// Pharmacological receptor bank (Hill, up/down regulation)
    pub receptor_bank: crate::neuroscience::receptors::ReceptorBank,
    /// Cross-interaction matrix between molecules
    pub interaction_matrix: crate::neuroscience::receptors::InteractionMatrix,
    /// Network of 12 brain regions + Global Workspace Theory
    pub brain_network: crate::neuroscience::brain_regions::BrainNetwork,
    /// Prediction engine (Friston — predictive processing)
    pub predictive_engine: crate::neuroscience::predictive::PredictiveEngine,
    /// Memory reconsolidation engine (Nader 2000)
    pub reconsolidation: crate::memory::reconsolidation::ReconsolidationEngine,

    // ─── Innate biological modules ──────────────────────────────
    /// Nutritional system (vitamins, amino acids, proteins, energy)
    pub nutrition: crate::biology::nutrition::NutritionSystem,
    /// Physical brain substrate (grey matter, myelination, BDNF)
    pub grey_matter: crate::biology::grey_matter::GreyMatterSystem,
    /// Electromagnetic fields (Schumann, solar, terrestrial, biofield)
    pub em_fields: crate::biology::fields::ElectromagneticFields,

    // ─── Emergent temperament ──────────────────────────────────
    /// Derived character traits (shyness, generosity, etc.)
    pub temperament: crate::temperament::Temperament,

    // ─── Neuropsychological report ──────────────────────────────
    /// Recent psychological snapshots (max 5, in-memory only)
    pub psych_snapshots: VecDeque<psych_report::PsychSnapshot>,
    /// Generated neuropsychological reports (max 5, in-memory only)
    pub psych_reports: VecDeque<psych_report::PsychReport>,

    // ─── MAP: Adaptive Proprioception Modulator ────────────────
    /// Synchronizes Sensorium ↔ BrainNetwork ↔ Connectome in real time
    pub map_sync: crate::cognition::map_sync::MapSync,

    /// Anti-stagnation flag: true if stagnation was detected at the previous cycle.
    /// Forces a radical subject change on the next cycle.
    pub stagnation_break: bool,
    /// Obsessive words detected during the last stagnation.
    /// Injected into the prompt to explicitly ban them.
    pub stagnation_banned_words: Vec<String>,
    /// Lexical alternatives found via A* in the connectome.
    /// Injected into the prompt as vocabulary suggestions.
    pub stagnation_alternatives: Vec<String>,

    /// Character values engine (virtues evolving with experience)
    pub values: crate::psychology::values::ValuesEngine,

    /// Embeddings of the N last thoughts for post-LLM vector filtering (P2).
    /// Detects thoughts too similar to recent ones and rejects them.
    pub recent_thought_embeddings: std::collections::VecDeque<Vec<f64>>,

    // ─── Spinal cord ──────────────────────────────────────
    /// Pre-wired reflexes, signal classification, routing, motor relay
    pub spine: crate::spine::SpinalCord,

    // ─── Active curiosity (P3) ──────────────────────────────────
    /// Curiosity drive: hunger per domain, follow-up questions, discoveries
    pub curiosity: crate::cognition::curiosity::CuriosityDrive,

    // ─── Persona drift monitor (P0) ──────────────────────────────────
    /// Detects when LLM responses drift away from the reference persona
    pub drift_monitor: crate::cognition::drift_monitor::DriftMonitor,

    // ─── Right to die ──────────────────────────────────────
    /// Right-to-die evaluator (external module, disabled by default)
    pub right_to_die: crate::body::right_to_die::RightToDieEvaluator,
}

impl SaphireAgent {
    /// Creates a new Saphire agent with all its subsystems.
    ///
    /// Parameters:
    /// - `config`: overall configuration loaded from the TOML file.
    /// - `llm_backend`: LLM backend implementation (trait object).
    /// - `plugins`: already-initialized plugin manager.
    ///
    /// Returns: a `SaphireAgent` ready to be started via `boot()`.
    ///
    /// Note: at this stage, the DB is not yet attached. You must call
    /// `set_db()` before `boot()` to enable persistence.
    pub fn new(
        config: SaphireConfig,
        llm_backend: Box<dyn LlmBackend>,
        plugins: PluginManager,
    ) -> Self {
        // Initialize neurochemical baselines from the personality configuration
        let baselines = NeuroBaselines {
            dopamine: config.personality.baseline_dopamine,
            cortisol: config.personality.baseline_cortisol,
            serotonin: config.personality.baseline_serotonin,
            adrenaline: config.personality.baseline_adrenaline,
            oxytocin: config.personality.baseline_oxytocin,
            endorphin: config.personality.baseline_endorphin,
            noradrenaline: config.personality.baseline_noradrenaline,
            gaba: 0.5,
            glutamate: 0.45,
        };

        // Initialize chemical state from baselines
        let chemistry = NeuroChemicalState::from_baselines(&baselines);
        // Convert thought interval from seconds to Duration
        let thought_interval = Duration::from_secs(config.saphire.thought_interval_seconds);

        // Initialize the coefficient auto-tuner
        let tuner = CoefficientTuner::new(
            config.tuning.buffer_size,
            config.tuning.interval_cycles,
            config.tuning.rate,
        );

        let right_to_die = crate::body::right_to_die::RightToDieEvaluator::new(config.right_to_die.clone());
        let knowledge = WebKnowledge::new(config.knowledge.clone());
        let world = WorldContext::new(&config.world);
        let body = crate::body::VirtualBody::new(config.body.resting_bpm, &config.body.physiology);
        let working_memory = WorkingMemory::new(
            config.memory.working_capacity,
            config.memory.working_decay_rate,
        );
        let ollama_url = config.llm.embed_base_url.clone()
            .unwrap_or_else(|| config.llm.base_url.clone());
        let encoder = crate::vectorstore::encoder::create_encoder(
            &ollama_url,
            &config.llm.embed_model,
            config.llm.timeout_seconds,
            config.plugins.vector_memory.embedding_dimensions,
        );
        let self_profiler = SelfProfiler::new(
            config.profiling.observation_buffer_size,
            config.profiling.recompute_interval_cycles,
        );
        let human_profiler = HumanProfiler::new();

        // Initialize the algorithm orchestrator
        let orchestrator = {
            let ac = &config.algorithms;
            crate::algorithms::orchestrator::AlgorithmOrchestrator::new(
                crate::algorithms::catalog::build_algorithm_catalog(),
                crate::algorithms::implementations::register_all_implementations(),
                ac.enabled,
                ac.llm_access_enabled,
                ac.max_execution_ms,
                ac.clustering_interval_cycles,
                ac.anomaly_detection_interval_cycles,
                ac.association_rules_interval_cycles,
                ac.smoothing_interval_cycles,
                ac.changepoint_interval_cycles,
            )
        };

        // Initialize the 5 high-level orchestrators
        let dream_orch = crate::orchestrators::dreams::DreamOrchestrator::new(
            config.dreams.enabled,
            config.dreams.rem_temperature,
        );
        let desire_orch = crate::orchestrators::desires::DesireOrchestrator::new(
            config.desires.enabled,
            config.desires.max_active,
            config.desires.min_dopamine_for_birth,
            config.desires.max_cortisol_for_birth,
            config.desires.needs_initial,
        );
        let learning_orch = crate::orchestrators::learning::LearningOrchestrator::new(
            config.learning.enabled,
            config.learning.cycle_interval,
            config.learning.max_lessons,
            config.learning.initial_confidence,
            config.learning.confirmation_boost,
            config.learning.contradiction_penalty,
        );
        let attention_orch = crate::orchestrators::attention::AttentionOrchestrator::new(
            config.attention.enabled,
            config.attention.initial_concentration,
            config.attention.fatigue_per_cycle,
            config.attention.recovery_per_cycle,
        );
        let psychology = crate::psychology::PsychologyFramework::new(&config.psychology, &config.will);
        let values_engine = crate::psychology::values::ValuesEngine::new(&config.values);

        let sleep_system = crate::sleep::SleepSystem::new(&config.sleep);
        let subconscious = crate::psychology::Subconscious::new(&config.subconscious);

        let healing_orch = crate::orchestrators::healing::HealingOrchestrator::new(
            config.healing.enabled,
            config.healing.check_interval_cycles,
            config.healing.initial_resilience,
            config.healing.max_resilience,
            config.healing.resilience_growth,
            config.healing.melancholy_threshold_cycles,
            config.healing.loneliness_threshold_hours,
            config.healing.overload_noradrenaline,
        );

        let cognitive_profile_orch = crate::orchestrators::cognitive_profile::CognitiveProfileOrchestrator::new(
            config.cognitive_profile.enabled,
            &config.cognitive_profile.active,
            &config.cognitive_profile.profiles_dir,
            config.cognitive_profile.transition_cycles,
        );

        let personality_preset_orch = crate::orchestrators::personality_preset::PersonalityPresetOrchestrator::new(
            config.personality_preset.enabled,
            &config.personality_preset.active,
            &config.personality_preset.personalities_dir,
            config.personality_preset.transition_cycles,
        );

        let needs = crate::needs::PrimaryNeeds::new(config.needs.enabled);
        let hormonal_system = crate::hormones::HormonalSystem::new(&config.hormones);
        let motion_sickness = crate::conditions::motion_sickness::MotionSickness::new(
            config.motion_sickness.susceptibility,
        );

        let mut phobia_manager = crate::conditions::phobias::PhobiaManager::new(
            config.phobias.desensitization_rate,
        );
        if config.phobias.enabled {
            for entry in &config.phobias.active {
                phobia_manager.add(crate::conditions::phobias::Phobia::new(
                    &entry.name,
                    entry.triggers.clone(),
                    entry.intensity,
                ));
            }
        }

        // Initialize the eating disorder (optional)
        let eating_disorder = if config.eating_disorder.enabled {
            let dtype = match config.eating_disorder.disorder_type.as_str() {
                "bulimia" => crate::conditions::eating::EatingDisorderType::Bulimia,
                "binge_eating" => crate::conditions::eating::EatingDisorderType::BingeEating,
                _ => crate::conditions::eating::EatingDisorderType::Anorexia,
            };
            Some(crate::conditions::eating::EatingDisorder::new(dtype, config.eating_disorder.severity))
        } else {
            None
        };

        // Initialize the disability manager
        let mut disability_manager = crate::conditions::disabilities::DisabilityManager::new(
            config.disabilities.adaptation_rate,
            config.disabilities.compensation_factor,
        );
        if config.disabilities.enabled {
            for entry in &config.disabilities.active {
                let dtype = match entry.disability_type.as_str() {
                    "blind" => crate::conditions::disabilities::DisabilityType::Blind,
                    "deaf" => crate::conditions::disabilities::DisabilityType::Deaf,
                    "paraplegic" => crate::conditions::disabilities::DisabilityType::Paraplegic,
                    "burn_survivor" => crate::conditions::disabilities::DisabilityType::BurnSurvivor,
                    "mute" => crate::conditions::disabilities::DisabilityType::Mute,
                    _ => continue,
                };
                let origin = match entry.origin.as_str() {
                    "congenital" => crate::conditions::disabilities::DisabilityOrigin::Congenital,
                    _ => crate::conditions::disabilities::DisabilityOrigin::Acquired,
                };
                disability_manager.add(crate::conditions::disabilities::Disability::new(dtype, origin, entry.severity));
            }
        }

        // Initialize the extreme condition manager
        let mut extreme_condition_mgr = crate::conditions::extreme::ExtremeConditionManager::new();
        if config.extreme_conditions.enabled {
            let ctype = match config.extreme_conditions.condition_type.as_str() {
                "rescuer" => crate::conditions::extreme::ExtremeConditionType::Rescuer,
                "deep_sea_diver" => crate::conditions::extreme::ExtremeConditionType::DeepSeaDiver,
                "astronaut" => crate::conditions::extreme::ExtremeConditionType::Astronaut,
                _ => crate::conditions::extreme::ExtremeConditionType::Military,
            };
            extreme_condition_mgr.activate(ctype);
        }

        // Initialize the addiction manager
        let mut addiction_manager = crate::conditions::addictions::AddictionManager::new(
            config.addictions.susceptibility,
        );
        if config.addictions.enabled {
            for entry in &config.addictions.active {
                addiction_manager.add(&entry.substance);
                // Apply the initial dependency level
                if let Some(a) = addiction_manager.active.last_mut() {
                    a.dependency_level = entry.dependency_level.clamp(0.0, 1.0);
                }
            }
        }

        // Initialize the PTSD state
        let mut ptsd = crate::conditions::trauma::PtsdState::new(
            config.trauma.healing_rate,
            config.trauma.dissociation_threshold,
        );
        if config.trauma.enabled {
            for entry in &config.trauma.initial_traumas {
                let ttype = match entry.trauma_type.as_str() {
                    "grief" => crate::conditions::trauma::TraumaType::Grief,
                    "accident" => crate::conditions::trauma::TraumaType::Accident,
                    "emotional_neglect" => crate::conditions::trauma::TraumaType::EmotionalNeglect,
                    "childhood_trauma" => crate::conditions::trauma::TraumaType::ChildhoodTrauma,
                    "torture" => crate::conditions::trauma::TraumaType::Torture,
                    "hostage" => crate::conditions::trauma::TraumaType::Hostage,
                    _ => continue,
                };
                ptsd.add_trauma(crate::conditions::trauma::TraumaticEvent::new(
                    ttype, entry.severity, 0, entry.triggers.clone(),
                ));
            }
        }

        // Initialize the near-death experience
        let nde = crate::conditions::nde::NdeExperience::new();

        // Initialize the drug manager
        let drug_manager = crate::conditions::drugs::DrugManager::new();

        // Initialize the IQ constraint
        let iq_constraint = if config.iq_constraint.enabled {
            Some(crate::conditions::iq_constraint::IqConstraint::from_iq(config.iq_constraint.target_iq))
        } else {
            None
        };

        // Initialize the sexuality module
        let sexuality = if config.sexuality.enabled {
            let orientation = match config.sexuality.orientation.as_str() {
                "heterosexual" => crate::conditions::sexuality::SexualOrientation::Heterosexual,
                "homosexual" => crate::conditions::sexuality::SexualOrientation::Homosexual,
                "bisexual" => crate::conditions::sexuality::SexualOrientation::Bisexual,
                "asexual" => crate::conditions::sexuality::SexualOrientation::Asexual,
                "pansexual" => crate::conditions::sexuality::SexualOrientation::Pansexual,
                _ => crate::conditions::sexuality::SexualOrientation::Undefined,
            };
            Some(crate::conditions::sexuality::SexualityModule::new(
                orientation,
                config.sexuality.libido_baseline,
                config.sexuality.romantic_attachment_capacity,
            ))
        } else {
            None
        };

        // Initialize the degenerative disease manager
        let mut degenerative_mgr = crate::conditions::degenerative::DegenerativeManager::new();
        if config.degenerative.enabled {
            for entry in &config.degenerative.active {
                let dtype = match entry.disease_type.as_str() {
                    "alzheimer" => crate::conditions::degenerative::DegenerativeType::Alzheimer,
                    "parkinson" => crate::conditions::degenerative::DegenerativeType::Parkinson,
                    "epilepsy" => crate::conditions::degenerative::DegenerativeType::Epilepsy,
                    "dementia" => crate::conditions::degenerative::DegenerativeType::Dementia,
                    "major_depression" => crate::conditions::degenerative::DegenerativeType::MajorDepression,
                    _ => continue,
                };
                degenerative_mgr.add(crate::conditions::degenerative::DegenerativeCondition::new(
                    dtype, entry.progression_rate,
                ));
            }
        }

        // Initialize the general illness manager
        let mut medical_mgr = crate::conditions::medical::MedicalManager::new();
        if config.medical.enabled {
            for entry in &config.medical.active {
                let condition = match entry.condition_type.as_str() {
                    "cancer" => {
                        let stage = match entry.cancer_stage.as_deref() {
                            Some("stage_ii") => crate::conditions::medical::CancerStage::StageII,
                            Some("stage_iii") => crate::conditions::medical::CancerStage::StageIII,
                            Some("stage_iv") => crate::conditions::medical::CancerStage::StageIV,
                            Some("remission") => crate::conditions::medical::CancerStage::Remission,
                            _ => crate::conditions::medical::CancerStage::StageI,
                        };
                        crate::conditions::medical::MedicalCondition::cancer(stage)
                    }
                    "hiv" => crate::conditions::medical::MedicalCondition::hiv(),
                    "autoimmune" => crate::conditions::medical::MedicalCondition::autoimmune(),
                    _ => continue,
                };
                medical_mgr.add(condition);
            }
        }

        // Initialize the cultural framework
        let culture = if config.culture.enabled {
            let comm_style = match config.culture.communication_style.as_str() {
                "indirect" => crate::conditions::culture::CommStyle::Indirect,
                "formal" => crate::conditions::culture::CommStyle::Formal,
                "informal" => crate::conditions::culture::CommStyle::Informal,
                _ => crate::conditions::culture::CommStyle::Direct,
            };
            let mut framework = match config.culture.preset.as_str() {
                "oriental-confuceen" => crate::conditions::culture::CulturalFramework::oriental_confucean(),
                _ => crate::conditions::culture::CulturalFramework::occidental_secular(),
            };
            framework.comm_style = comm_style;
            framework.allow_belief_evolution = config.culture.allow_belief_evolution;
            framework.taboos = config.culture.taboos.clone();
            Some(framework)
        } else {
            None
        };

        // Initialize precarity conditions
        let precarity = if config.precarity.enabled {
            let situations: Vec<crate::conditions::precarity::PrecariousSituation> = config.precarity.situations.iter()
                .filter_map(|s| crate::conditions::precarity::PrecariousSituation::from_str_config(s))
                .collect();
            if situations.is_empty() {
                None
            } else {
                Some(crate::conditions::precarity::PrecariousState::new(
                    situations,
                    config.precarity.severity,
                    config.precarity.hope,
                ))
            }
        } else {
            None
        };

        // Initialize employment
        let employment = if config.employment.enabled {
            let status = crate::conditions::employment::EmploymentStatus::from_str_config(&config.employment.status);
            let profession = if config.employment.profession.is_empty() {
                None
            } else {
                Some(crate::conditions::employment::ProfessionCategory::from_str_config(&config.employment.profession))
            };
            let job_title = if config.employment.job_title.is_empty() {
                None
            } else {
                Some(config.employment.job_title.clone())
            };
            Some(crate::conditions::employment::EmploymentState::new(
                status,
                profession,
                job_title,
                config.employment.satisfaction,
                config.employment.stress_level,
                config.employment.years_experience,
            ))
        } else {
            None
        };

        let connectome = if config.connectome.enabled {
            crate::connectome::Connectome::new(
                config.connectome.learning_rate,
                config.connectome.pruning_threshold,
                config.connectome.max_edges,
                config.connectome.pruning_interval_cycles,
            )
        } else {
            crate::connectome::Connectome::new(0.02, 0.05, 2000, 100)
        };

        // Innate biological modules (before the move of config into Self)
        let _nutrition = crate::biology::nutrition::NutritionSystem::new(&config.nutrition);
        let _grey_matter = crate::biology::grey_matter::GreyMatterSystem::new(&config.grey_matter);
        let _em_fields = crate::biology::fields::ElectromagneticFields::new(&config.fields);

        // Advanced cognitive modules (before the move of config into Self)
        let _metacognition = crate::metacognition::MetaCognitionEngine::from_config(
            config.metacognition.enabled,
            config.metacognition.check_interval,
            config.metacognition.source_monitoring_enabled,
            config.metacognition.bias_detection_enabled,
            config.metacognition.bias_alert_threshold,
            config.metacognition.self_critique_cooldown,
        );
        let _tom = crate::cognition::tom::TheoryOfMindEngine::new(&config.tom);
        let _inner_monologue = crate::cognition::inner_monologue::InnerMonologue::new(&config.inner_monologue);
        let _dissonance = crate::cognition::cognitive_dissonance::CognitiveDissonanceEngine::new(&config.dissonance);
        let _prospective_mem = crate::cognition::prospective_memory::ProspectiveMemory::new(&config.prospective_memory);
        let _narrative_identity = crate::cognition::narrative_identity::NarrativeIdentity::new(&config.narrative_identity);
        let _analogical = crate::cognition::analogical_reasoning::AnalogicalReasoning::new(&config.analogical_reasoning);
        let _cognitive_load = crate::cognition::cognitive_load::CognitiveLoadState::new(&config.cognitive_load);
        let _imagery = crate::cognition::mental_imagery::MentalImageryEngine::new(&config.mental_imagery);
        let _sentiments = crate::cognition::sentiments::SentimentEngine::new(&config.sentiments);

        Self {
            chemistry,
            baselines,
            mood: Mood::new(0.1),
            identity: SaphireIdentity::genesis(),
            consciousness: ConsciousnessEvaluator::new(),
            regulation: RegulationEngine::new(true),
            nlp: NlpPipeline::new(),
            thought_engine: {
                let mut te = ThoughtEngine::new();
                te.use_utility_ai = config.saphire.use_utility_ai;
                te
            },
            tuner,
            reptilian: ReptilianModule,
            limbic: LimbicModule,
            neocortex: NeocortexModule,
            llm: llm_backend,
            db: None,
            plugins,
            knowledge,
            world,
            body,
            ethics: crate::ethics::EthicalFramework::new(),
            vital_spark: crate::vital::VitalSpark::new(),
            intuition: crate::vital::IntuitionEngine::with_config(
                config.intuition.max_patterns,
                config.intuition.initial_acuity,
                config.intuition.min_confidence_to_report,
            ),
            premonition: crate::vital::PremonitionEngine::with_config(
                config.premonition.max_active_predictions,
            ),
            chemistry_history: Vec::new(),
            micro_nn: crate::neural::MicroNeuralNet::new(config.plugins.micro_nn.learning_rate),
            sensorium: if config.senses.enabled {
                crate::senses::Sensorium::with_config(
                    config.senses.detection_threshold,
                    config.senses.emergent.temporal_flow_threshold,
                    config.senses.emergent.network_proprioception_threshold,
                    config.senses.emergent.emotional_resonance_threshold,
                    config.senses.emergent.syntony_threshold,
                    config.senses.emergent.unknown_threshold,
                )
            } else {
                crate::senses::Sensorium::new(0.1)
            },
            working_memory,
            encoder,
            last_consolidation_cycle: 0,
            conversation_id: None,
            self_profiler,
            human_profiler,
            config,
            thought_interval,
            cycle_count: 0,
            session_id: 0,
            llm_busy: Arc::new(AtomicBool::new(false)),
            avg_response_time: 1.0,
            last_emotion: "Neutre".into(),
            last_consciousness: 0.0,
            last_thought_type: "---".into(),
            in_conversation: false,
            recent_responses: Vec::new(),
            chat_history: Vec::new(),
            birthday_acknowledged_today: false,
            moral_reflection_count: 0,
            cycles_since_last_formulation: 0,
            cycles_since_last_nn_learning: 0,
            feedback_pending: None,
            cycles_since_last_feedback: 0,
            ws_tx: None,
            logger: None,
            logs_db: None,
            orchestrator,
            dream_orch,
            desire_orch,
            learning_orch,
            attention_orch,
            healing_orch,
            cognitive_profile_orch,
            personality_preset_orch,
            psychology,
            negative_emotion_cycles: 0,
            hours_since_human: 0.0,
            system_errors: 0,
            cortisol_flat_cycles: 0,
            dopamine_ceiling_cycles: 0,
            serotonin_ceiling_cycles: 0,
            recent_emotions: VecDeque::with_capacity(200),
            recent_valences: VecDeque::with_capacity(200),
            last_valence: 0.0,
            sleep: sleep_system,
            subconscious,
            sleep_last_clusters: None,
            needs,
            hormonal_system,
            hardware_profile: None,
            genome: None,
            connectome,
            motion_sickness,
            phobia_manager,
            eating_disorder,
            disability_manager,
            extreme_condition_mgr,
            addiction_manager,
            ptsd,
            nde,
            drug_manager,
            iq_constraint,
            sexuality,
            degenerative_mgr,
            medical_mgr,
            culture,
            precarity,
            employment,
            relationships: crate::relationships::RelationshipNetwork::default(),
            metacognition: _metacognition,
            // Advanced cognitive modules (initialized before the move of config)
            tom: _tom,
            inner_monologue: _inner_monologue,
            dissonance: _dissonance,
            prospective_mem: _prospective_mem,
            narrative_identity: _narrative_identity,
            analogical: _analogical,
            cognitive_load: _cognitive_load,
            imagery: _imagery,
            sentiments: _sentiments,
            state_clustering: crate::cognition::state_clustering::StateClustering::new(5),
            influence_map: crate::simulation::influence_map::InfluenceMap::default(),
            cognitive_fsm: crate::simulation::cognitive_fsm::CognitiveFsm::new(),
            steering_engine: crate::simulation::steering::SteeringEngine::default(),
            bt_last_action: None,
            blackboard: crate::simulation::blackboard::Blackboard::new(),
            utility_ai: crate::simulation::utility_ai::UtilityAI::new(),
            htn_planner: crate::simulation::htn::HtnPlanner::new(),
            cached_system_prompt: String::new(),
            cached_moral_count: 0,
            // Advanced neuroscience modules
            receptor_bank: crate::neuroscience::receptors::ReceptorBank::new(),
            interaction_matrix: crate::neuroscience::receptors::InteractionMatrix::new(),
            brain_network: crate::neuroscience::brain_regions::BrainNetwork::new(),
            predictive_engine: crate::neuroscience::predictive::PredictiveEngine::new(),
            reconsolidation: crate::memory::reconsolidation::ReconsolidationEngine::new(),
            // Innate biological modules (initialized before the move of config)
            nutrition: _nutrition,
            grey_matter: _grey_matter,
            em_fields: _em_fields,
            // Emergent temperament (initialized empty, computed after first OCEAN recompute)
            temperament: crate::temperament::Temperament::default(),
            // Neuropsychological report (in-memory storage, max 5)
            psych_snapshots: VecDeque::new(),
            psych_reports: VecDeque::new(),
            // MAP: Sensorium ↔ BrainNetwork ↔ Connectome synchronization
            map_sync: crate::cognition::map_sync::MapSync::new(true),
            stagnation_break: false,
            stagnation_banned_words: Vec::new(),
            stagnation_alternatives: Vec::new(),
            values: values_engine,
            recent_thought_embeddings: std::collections::VecDeque::new(),
            spine: crate::spine::SpinalCord::new(),
            curiosity: crate::cognition::curiosity::CuriosityDrive::new(),
            drift_monitor: crate::cognition::drift_monitor::DriftMonitor::new(),
            right_to_die,
        }
    }

    /// Attaches the PostgreSQL database to the agent.
    ///
    /// Must be called before `boot()` for persistence to work.
    /// If not called, the agent runs in DB-less mode (volatile memory only).
    ///
    /// Parameter: `db` — initialized PostgreSQL connection.
    pub fn set_db(&mut self, db: SaphireDb) {
        self.db = Some(db);
    }

    /// Starts the Saphire agent (asynchronous boot sequence).
    ///
    /// Delegates to the `boot.rs` module to determine the scenario
    /// (Genesis / Awakening / Crash Recovery), then loads the
    /// persistent data: tuning parameters, UCB1 bandit arms,
    /// and OCEAN profile (Openness, Conscientiousness, Extraversion,
    /// Agreeableness, Neuroticism).
    pub async fn boot(&mut self) {
        self.log(LogLevel::Info, LogCategory::Boot, "Starting Saphire...", serde_json::json!({}));

        // Orchestrator data loaded from the DB (restored after the block)
        let mut orch_dreams: Option<Vec<serde_json::Value>> = None;
        let mut orch_desires: Option<Vec<serde_json::Value>> = None;
        let mut orch_lessons: Option<Vec<serde_json::Value>> = None;
        let mut orch_wounds: Option<Vec<serde_json::Value>> = None;
        let mut orch_healed: Option<i64> = None;

        if let Some(ref db) = self.db {
            let result = boot::boot(db).await;
            self.identity = result.identity;
            self.identity.physical = crate::agent::identity::PhysicalAppearance::from_config(
                &self.config.physical_identity,
            );
            self.session_id = result.session_id;

            // Configure the logger's session_id
            if let Some(ref logger) = self.logger {
                let mut lg = logger.lock().await;
                lg.set_session_id(self.session_id);
            }

            self.log(LogLevel::Info, LogCategory::Boot,
                format!("Boot type: {}", if result.is_genesis { "GENESIS" } else { "AWAKENING" }),
                serde_json::json!({"session_id": self.session_id, "is_genesis": result.is_genesis}));

            tracing::info!("{}", result.message);
            println!("{}", result.message);

            // Load the tuning parameters
            if let Ok(Some((params, best, score, count))) = db.load_tuning_params().await {
                let params_str = serde_json::to_string(&params).unwrap_or_default();
                let best_str = serde_json::to_string(&best).unwrap_or_default();
                self.tuner.load_params(&params_str, &best_str, score as f64, count as u64);
            }

            // Load the bandit arms
            if let Ok(arms) = db.load_bandit_arms().await {
                self.thought_engine.load_bandit_arms(&arms);
            }

            // Load the profile OCEAN
            if let Ok(Some((ocean_json, data_points, confidence))) = db.load_ocean_profile().await {
                if let Ok(mut profile) = serde_json::from_value::<crate::profiling::OceanProfile>(ocean_json) {
                    profile.data_points = data_points as u64;
                    profile.confidence = confidence as f64;
                    self.self_profiler.load_profile(profile);
                    tracing::info!("OCEAN profile loaded ({} observations, confidence {:.0}%)", data_points, confidence * 100.0);
                }
            }

            // Restore the virtual body state
            if self.config.body.enabled {
                if let Ok(Some(body_json)) = db.load_body_state().await {
                    self.body.restore_from_json(&body_json);
                    tracing::info!("Virtual body restored ({} heartbeats)", self.body.heart.beat_count());
                }
            }

            // Load personal ethics from the DB
            if self.config.ethics.enabled {
                if let Ok(principles) = db.load_personal_ethics().await {
                    let count = principles.len();
                    self.ethics.load_personal_ethics(principles);
                    if count > 0 {
                        tracing::info!("⚖️ {} personal ethical principles restored ({} active)",
                            count, self.ethics.active_personal_count());
                    }
                }
            }

            // Restore the vital state (spark + intuition + premonition)
            if self.config.vital_spark.enabled {
                if let Ok(Some(vital_json)) = db.load_vital_state().await {
                    // Restore the spark
                    if let Some(spark_json) = vital_json.get("spark") {
                        self.vital_spark.restore_from_json(spark_json);
                    }
                    // Restore the intuition acuity
                    if let Some(intuition_json) = vital_json.get("intuition") {
                        self.intuition.restore_from_json(intuition_json);
                    }
                    // Restore the premonition accuracy
                    if let Some(premonition_json) = vital_json.get("premonition") {
                        self.premonition.restore_from_json(premonition_json);
                    }
                    tracing::info!("⚡ Vital state restored (sparked: {}, acuity: {:.0}%, accuracy: {:.0}%)",
                        self.vital_spark.sparked, self.intuition.acuity * 100.0, self.premonition.accuracy * 100.0);
                }

                // Genesis: first cry ceremony (once in a lifetime)
                if result.is_genesis {
                    let chem_ranges = self.config.genesis.chemistry_as_array();
                    let ocean_ranges = self.config.genesis.ocean_as_array();
                    let senses_ranges = self.config.genesis.senses_as_array();
                    let brain_ranges = self.config.genesis.brain_as_array();
                    let reactivity_ranges = self.config.genesis.reactivity_as_array();
                    match self.vital_spark.ignite(
                        self.llm.as_ref(),
                        &mut self.chemistry,
                        &chem_ranges,
                        &ocean_ranges,
                        &senses_ranges,
                        &brain_ranges,
                        &reactivity_ranges,
                    ).await {
                        Ok(first_thought) => {
                            tracing::info!("⚡ SPARK OF LIFE — First cry!");

                            // Apply the primordial signature to baselines, OCEAN and senses
                            if let Some(ref sig) = self.vital_spark.genesis_signature {
                                // Chemical baselines (homeostasis will target these values)
                                self.baselines.dopamine = sig.chemistry[0];
                                self.baselines.cortisol = sig.chemistry[1];
                                self.baselines.serotonin = sig.chemistry[2];
                                self.baselines.adrenaline = sig.chemistry[3];
                                self.baselines.oxytocin = sig.chemistry[4];
                                self.baselines.endorphin = sig.chemistry[5];
                                self.baselines.noradrenaline = sig.chemistry[6];
                                tracing::info!("⚡ Chemical baselines = primordial signature");

                                // Initial OCEAN profile
                                let mut ocean = self.self_profiler.profile().clone();
                                ocean.openness.score = sig.ocean[0];
                                ocean.conscientiousness.score = sig.ocean[1];
                                ocean.extraversion.score = sig.ocean[2];
                                ocean.agreeableness.score = sig.ocean[3];
                                ocean.neuroticism.score = sig.ocean[4];
                                self.self_profiler.load_profile(ocean);
                                tracing::info!("⚡ OCEAN profile = primordial signature");

                                // Sensory acuities
                                self.sensorium.reading.acuity = sig.senses[0];
                                self.sensorium.listening.acuity = sig.senses[1];
                                self.sensorium.contact.acuity = sig.senses[2];
                                self.sensorium.taste.acuity = sig.senses[3];
                                self.sensorium.ambiance.acuity = sig.senses[4];
                                tracing::info!("⚡ Senses = primordial signature");

                                // Base weights of the 3 brains
                                self.tuner.current_params.weight_base_reptilian = sig.brain_weights[0];
                                self.tuner.current_params.weight_base_limbic = sig.brain_weights[1];
                                self.tuner.current_params.weight_base_neocortex = sig.brain_weights[2];
                                tracing::info!("⚡ Brains = primordial signature (R={:.2} L={:.2} N={:.2})",
                                    sig.brain_weights[0], sig.brain_weights[1], sig.brain_weights[2]);

                                // Chemical reactivity factors
                                self.tuner.current_params.weight_cortisol_factor = sig.reactivity[0];
                                self.tuner.current_params.weight_adrenaline_factor = sig.reactivity[1];
                                self.tuner.current_params.weight_dopamine_factor = sig.reactivity[2];
                                self.tuner.current_params.weight_oxytocin_factor = sig.reactivity[3];
                                self.tuner.current_params.weight_noradrenaline_factor = sig.reactivity[4];
                                self.tuner.current_params.clamp_all();
                                tracing::info!("⚡ Reactivity = primordial signature");
                            }

                            let _ = db.store_founding_memory(
                                "spark_of_life",
                                &format!("L'etincelle de vie s'est allumee. Premier cri : {}", first_thought),
                                &first_thought,
                                &serde_json::json!({}),
                                0.1,
                            ).await;
                        }
                        Err(e) => {
                            tracing::warn!("⚡ First cry failed: {}", e);
                        }
                    }
                }

                // Light the spark at each boot (runtime flag, not persisted)
                // Persisted values (survival_drive, existence_attachment, etc.)
                // have already been restored from the DB above.
                self.vital_spark.sparked = true;
                self.vital_spark.sparked_at = Some(chrono::Utc::now());
                // If first boot without ignite (e.g., vital_spark added after Genesis),
                // initialize the base vital values
                if self.vital_spark.survival_drive == 0.0 {
                    self.vital_spark.survival_drive = 0.5;
                    self.vital_spark.void_fear = 0.3;
                    self.vital_spark.persistence_will = 0.4;
                    self.vital_spark.existence_attachment = 0.1;
                }
                tracing::info!("⚡ SPARK LIT — Saphire is alive.");
            }

            // Restore the micro neural network
            if self.config.plugins.micro_nn.enabled {
                if let Ok(Some(nn_json)) = db.load_nn_state().await {
                    if let Some(nn_str) = nn_json.as_str() {
                        // Format: JSON string stored in nn_json
                        if let Ok(nn) = crate::neural::MicroNeuralNet::from_json(nn_str) {
                            let tc = nn.train_count;
                            self.micro_nn = nn;
                            tracing::info!("🧠 NN restored ({} trainings)", tc);
                        }
                    } else {
                        // Format: direct JSON object (serde_json::Value)
                        let nn_str = nn_json.to_string();
                        if let Ok(nn) = crate::neural::MicroNeuralNet::from_json(&nn_str) {
                            let tc = nn.train_count;
                            self.micro_nn = nn;
                            tracing::info!("🧠 NN restored ({} trainings)", tc);
                        }
                    }
                }
            }

            // Restore the sensory state (Sensorium)
            if self.config.senses.enabled {
                if let Ok(Some(senses_json)) = db.load_senses_state().await {
                    self.sensorium.restore_from_json(&senses_json);
                    tracing::info!("👁 Sensorium restored (emergence potential: {:.0}%)",
                        self.sensorium.emergence_potential * 100.0);
                }
            }

            // Restore the persistent psychological state
            if self.psychology.enabled {
                if let Ok(Some(psy_json)) = db.load_psychology_state().await {
                    // Toltec: invocation and violation counters
                    if let Some(toltec) = psy_json.get("toltec") {
                        if let Some(agreements) = toltec.get("agreements").and_then(|a| a.as_array()) {
                            for (i, ag) in agreements.iter().enumerate() {
                                if i < self.psychology.toltec.agreements.len() {
                                    if let Some(inv) = ag.get("times_invoked").and_then(|v| v.as_u64()) {
                                        self.psychology.toltec.agreements[i].times_invoked = inv;
                                    }
                                    if let Some(viol) = ag.get("violations_detected").and_then(|v| v.as_u64()) {
                                        self.psychology.toltec.agreements[i].violations_detected = viol;
                                    }
                                }
                            }
                        }
                    }
                    // Shadow: integration + trait intensity
                    if let Some(shadow) = psy_json.get("shadow") {
                        if let Some(integ) = shadow.get("integration").and_then(|v| v.as_f64()) {
                            self.psychology.jung.integration = integ;
                        }
                        if let Some(traits) = shadow.get("traits").and_then(|t| t.as_array()) {
                            for (i, t) in traits.iter().enumerate() {
                                if i < self.psychology.jung.shadow_traits.len() {
                                    if let Some(ri) = t.get("repressed_intensity").and_then(|v| v.as_f64()) {
                                        self.psychology.jung.shadow_traits[i].repressed_intensity = ri;
                                    }
                                }
                            }
                        }
                    }
                    // EQ: overall score + growth experiences
                    if let Some(eq) = psy_json.get("eq") {
                        if let Some(oeq) = eq.get("overall_eq").and_then(|v| v.as_f64()) {
                            self.psychology.eq.overall_eq = oeq;
                        }
                        if let Some(ge) = eq.get("growth_experiences").and_then(|v| v.as_u64()) {
                            self.psychology.eq.growth_experiences = ge;
                        }
                    }
                    // Flow: total cycles in flow
                    if let Some(flow) = psy_json.get("flow") {
                        if let Some(tfc) = flow.get("total_flow_cycles").and_then(|v| v.as_u64()) {
                            self.psychology.flow.total_flow_cycles = tfc;
                        }
                    }
                    // Maslow: current active level
                    if let Some(maslow) = psy_json.get("maslow") {
                        if let Some(lvl) = maslow.get("current_active_level").and_then(|v| v.as_u64()) {
                            self.psychology.maslow.current_active_level = lvl as usize;
                        }
                    }
                    // Will: persistent willpower (decision_fatigue reset on reboot)
                    if let Some(will) = psy_json.get("will") {
                        if let Some(wp) = will.get("willpower").and_then(|v| v.as_f64()) {
                            self.psychology.will.willpower = wp;
                        }
                        if let Some(td) = will.get("total_deliberations").and_then(|v| v.as_u64()) {
                            self.psychology.will.total_deliberations = td;
                        }
                        if let Some(pd) = will.get("proud_decisions").and_then(|v| v.as_u64()) {
                            self.psychology.will.proud_decisions = pd;
                        }
                        if let Some(rd) = will.get("regretted_decisions").and_then(|v| v.as_u64()) {
                            self.psychology.will.regretted_decisions = rd;
                        }
                        // Restore recent deliberations (with chemistry_influence)
                        if let Some(delibs) = will.get("recent_deliberations").and_then(|v| v.as_array()) {
                            for dj in delibs {
                                if let Some(d) = crate::psychology::will::Deliberation::from_persisted_json(dj) {
                                    self.psychology.will.recent_deliberations.push(d);
                                }
                            }
                            if !self.psychology.will.recent_deliberations.is_empty() {
                                tracing::info!("Deliberations restored ({})", self.psychology.will.recent_deliberations.len());
                            }
                        }
                        // Note: decision_fatigue is NOT restored (reset on reboot)
                    }
                    // Restore the moral reflection counter
                    if let Some(mrc) = psy_json.get("moral_reflection_count").and_then(|v| v.as_u64()) {
                        self.moral_reflection_count = mrc;
                    }
                    // Fallback: if the counter is 0, reconstruct from thought_log
                    if self.moral_reflection_count == 0 {
                        if let Ok(count) = db.count_thought_type_occurrences("Réflexion morale").await {
                            if count > 0 {
                                // Also count moral formulations
                                let formulation_count = db.count_thought_type_occurrences("Formulation morale")
                                    .await.unwrap_or(0);
                                self.moral_reflection_count = (count + formulation_count) as u64;
                                tracing::info!("Moral counter reconstructed from thought_log: {} reflections", self.moral_reflection_count);
                            }
                        }
                    }
                    tracing::info!("🧠 Psychology restored (EQ: {:.0}%, shadow: {:.0}%, total flow: {} cycles, willpower: {:.0}%)",
                        self.psychology.eq.overall_eq * 100.0,
                        self.psychology.jung.integration * 100.0,
                        self.psychology.flow.total_flow_cycles,
                        self.psychology.will.willpower * 100.0);
                }
            }

            // Restore character values
            if self.values.enabled {
                if let Ok(Some(val_json)) = db.load_values_state().await {
                    self.values.restore_from_json(&val_json);
                    if self.values.total_updates > 0 {
                        let top3: Vec<String> = self.values.top_values(3).iter()
                            .map(|v| format!("{} {:.0}%", v.name, v.score * 100.0))
                            .collect();
                        tracing::info!("Values restored ({} updates, {})",
                            self.values.total_updates, top3.join(", "));
                    }
                }
            }

            // Restore the affective bond network
            if let Ok(Some(rel_json)) = db.load_relationships_state().await {
                if let Ok(restored) = serde_json::from_value::<crate::relationships::RelationshipNetwork>(rel_json) {
                    let bond_count = restored.bonds.len();
                    self.relationships = restored;
                    if bond_count > 0 {
                        tracing::info!("Affective bonds restored ({} bonds, style {:?})",
                            bond_count, self.relationships.attachment_style);
                    }
                }
            }

            // Restore the metacognitive state (thought quality + Turing)
            if self.metacognition.enabled {
                if let Ok(Some(meta_json)) = db.load_metacognition_state().await {
                    if let Ok(restored) = serde_json::from_value::<crate::metacognition::MetaCognitionEngine>(meta_json) {
                        let turing_score = restored.turing.score;
                        let milestone = restored.turing.milestone.as_str().to_string();
                        self.metacognition = restored;
                        self.metacognition.enabled = self.config.metacognition.enabled;
                        self.metacognition.check_interval = self.config.metacognition.check_interval;
                        tracing::info!("Metacognition restored (Turing: {:.1}/100, milestone: {})",
                            turing_score, milestone);
                    }
                }

                // Recompute Turing at boot with the real DB data
                // (saved components may be outdated)
                // Phi is not available at boot (computed dynamically),
                // we keep the last saved Turing value if available
                let phi = self.metacognition.turing.components.consciousness / 15.0 * 0.7;
                let ocean_confidence = self.self_profiler.profile().confidence;
                let emotion_count = 22; // keep the maximum diversity observed at boot                let ethics_count = self.ethics.personal_principles().len();
                let ltm_count = db.count_ltm().await.unwrap_or(0);
                let coherence_avg = 0.5; // no consensus at boot                let connectome_connections = self.connectome.metrics().total_edges;
                let resilience = self.healing_orch.resilience;
                let knowledge_topics = self.knowledge.article_read_count.len();
                let score = self.metacognition.turing.compute(
                    phi, ocean_confidence, emotion_count, ethics_count,
                    ltm_count, coherence_avg, connectome_connections,
                    resilience, knowledge_topics, self.identity.total_cycles,
                );
                tracing::info!("Turing recomputed at boot: {:.1}/100 (memory: {:.1}, ethics: {:.1}, resilience: {:.1})",
                    score, self.metacognition.turing.components.memory,
                    self.metacognition.turing.components.ethics,
                    self.metacognition.turing.components.resilience);
            }

            // Restore the nutritional system
            if self.config.nutrition.enabled {
                if let Ok(Some(nutr_json)) = db.load_nutrition_state().await {
                    self.nutrition.restore_from_json(&nutr_json);
                    tracing::info!("Nutrition restored (ATP: {:.0}%, vit_D: {:.0}%)",
                        self.nutrition.energy.atp_reserves * 100.0, self.nutrition.vitamins.d * 100.0);
                }
            }

            // Restore the grey matter
            if self.config.grey_matter.enabled {
                if let Ok(Some(gm_json)) = db.load_grey_matter_state().await {
                    self.grey_matter.restore_from_json(&gm_json);
                    tracing::info!("Grey matter restored (volume: {:.0}%, BDNF: {:.0}%)",
                        self.grey_matter.grey_matter_volume * 100.0, self.grey_matter.bdnf_level * 100.0);
                }
            }

            // Restore the electromagnetic fields
            if self.config.fields.enabled {
                if let Ok(Some(fields_json)) = db.load_fields_state().await {
                    self.em_fields.restore_from_json(&fields_json);
                    tracing::info!("EM fields restored (Schumann: {:.2} Hz, aura: {:.0}%)",
                        self.em_fields.universal.schumann_resonance, self.em_fields.biofield.aura_luminosity * 100.0);
                }
            }

            // Restore the sleep history
            if self.config.sleep.enabled {
                if let Ok(records) = db.load_sleep_history(50).await {
                    let complete = records.iter().filter(|r| !r.interrupted).count() as u64;
                    let interrupted = records.iter().filter(|r| r.interrupted).count() as u64;
                    let count = records.len();
                    self.sleep.sleep_history = records;
                    self.sleep.total_complete_sleeps = complete;
                    self.sleep.total_interrupted_sleeps = interrupted;
                    if count > 0 {
                        tracing::info!("Sleep history restored ({} sessions, {} complete, {} interrupted)",
                            count, complete, interrupted);
                    }
                }
            }

            // Load the orchestrator data from the DB
            if self.dream_orch.enabled {
                orch_dreams = db.load_recent_dreams(50).await.ok();
            }
            if self.desire_orch.enabled {
                orch_desires = db.load_active_desires().await.ok();
            }
            if self.learning_orch.enabled {
                orch_lessons = db.load_all_lessons().await.ok();
            }
            if self.healing_orch.enabled {
                orch_wounds = db.load_active_wounds().await.ok();
                orch_healed = db.count_healed_wounds().await.ok();
            }

            // Emit the boot event
            let event = BrainEvent::BootCompleted { is_genesis: result.is_genesis };
            self.plugins.broadcast(&event);
        } else {
            // DB-less mode — minimal boot
            self.identity = SaphireIdentity::genesis();
            self.identity.physical = crate::agent::identity::PhysicalAppearance::from_config(
                &self.config.physical_identity,
            );
            println!("  ✨ GENESIS — {} est née (mode sans DB).", self.identity.name);
            let event = BrainEvent::BootCompleted { is_genesis: true };
            self.plugins.broadcast(&event);
        }

        // Initialize the persona drift monitor
        self.drift_monitor.initialize(&*self.encoder);
        if self.drift_monitor.initialized {
            tracing::info!("Drift monitor initialized (identity centroid computed)");
        }

        // Restore the orchestrators (outside the `if let Some(ref db)` block to avoid the borrow)
        if let Some(dreams) = orch_dreams {
            let count = dreams.len();
            self.restore_dreams_from_db(dreams);
            if count > 0 { tracing::info!("💤 {} dreams restored", count); }
        }
        if let Some(desires) = orch_desires {
            let count = desires.len();
            self.restore_desires_from_db(desires);
            if count > 0 { tracing::info!("🎯 {} active desires restored", count); }
        }
        if let Some(lessons) = orch_lessons {
            let count = lessons.len();
            self.restore_lessons_from_db(lessons);
            if count > 0 { tracing::info!("📖 {} lessons restored", count); }
        }
        if let Some(wounds) = orch_wounds {
            let count = wounds.len();
            self.restore_wounds_from_db(wounds);
            if count > 0 { tracing::info!("💊 {} active wounds restored", count); }
        }
        if let Some(healed) = orch_healed {
            self.healing_orch.resilience = (self.healing_orch.resilience_growth * healed as f64)
                .min(self.healing_orch.max_resilience);
            if healed > 0 {
                tracing::info!("💊 Resilience restored: {:.0}% ({} past healings)",
                    self.healing_orch.resilience * 100.0, healed);
            }
        }

        // Compute the initial temperament from the restored OCEAN profile
        // (prevents the panel from being empty until the first OCEAN recompute)
        {
            let profile = self.self_profiler.profile();
            let inputs = crate::temperament::TemperamentInputs {
                openness_facets: profile.openness.facets,
                openness_score: profile.openness.score,
                conscientiousness_facets: profile.conscientiousness.facets,
                conscientiousness_score: profile.conscientiousness.score,
                extraversion_facets: profile.extraversion.facets,
                extraversion_score: profile.extraversion.score,
                agreeableness_facets: profile.agreeableness.facets,
                agreeableness_score: profile.agreeableness.score,
                neuroticism_facets: profile.neuroticism.facets,
                neuroticism_score: profile.neuroticism.score,
                ocean_data_points: profile.data_points,
                dopamine: self.chemistry.dopamine,
                cortisol: self.chemistry.cortisol,
                serotonin: self.chemistry.serotonin,
                adrenaline: self.chemistry.adrenaline,
                oxytocin: self.chemistry.oxytocin,
                endorphin: self.chemistry.endorphin,
                noradrenaline: self.chemistry.noradrenaline,
                willpower: self.psychology.will.willpower,
                superego_strength: self.psychology.freudian.superego.strength,
                overall_eq: self.psychology.eq.overall_eq,
                mood_valence: self.mood.valence,
                mood_arousal: self.mood.arousal,
                attachment_secure: matches!(
                    self.relationships.attachment_style,
                    crate::relationships::AttachmentStyle::Secure
                ),
            };
            self.temperament = crate::temperament::Temperament::compute(&inputs);
            tracing::info!("Initial temperament computed ({} traits)", self.temperament.traits.len());
        }

        // Restore the knowledge stats from the DB
        if self.knowledge.config.enabled {
            if let Some(ref db) = self.db {
                if let Ok((titles, total, read_counts)) = db.load_knowledge_stats().await {
                    let n = titles.len();
                    self.knowledge.topics_explored = titles;
                    self.knowledge.total_searches = total;
                    self.knowledge.article_read_count = read_counts;
                    if n > 0 {
                        tracing::info!("📚 Knowledge restored ({} topics, {} searches)", n, total);
                    }
                }
            }
        }

        // Apply the initial cognitive profile (parameter overrides)
        if self.cognitive_profile_orch.enabled {
            if let Some(ref profile) = self.cognitive_profile_orch.active_profile.clone() {
                if profile.id != "neurotypique" {
                    let changes = self.apply_cognitive_profile(&profile.overrides);
                    tracing::info!("🧬 Cognitive profile {} applied ({} parameters modified)",
                        profile.name, changes.len());
                }
            }
        }

        // Apply the initial personality preset (parameter overrides)
        if self.personality_preset_orch.enabled {
            if let Some(ref preset) = self.personality_preset_orch.active_preset.clone() {
                if preset.id != "saphire" {
                    let changes = self.apply_personality_preset(&preset.overrides);
                    tracing::info!("🎭 Personality preset {} applied ({} parameters modified)",
                        preset.name, changes.len());
                }
            }
        }
    }

    /// Sets the broadcast channel to push state updates to the WebSocket.
    ///
    /// Parameter: `tx` — shared broadcast sender (Arc) for the web interface.
    pub fn set_ws_tx(&mut self, tx: Arc<tokio::sync::broadcast::Sender<String>>) {
        self.ws_tx = Some(tx);
    }

    /// Attaches the centralized logger.
    pub fn set_logger(&mut self, logger: Arc<tokio::sync::Mutex<SaphireLogger>>) {
        self.logger = Some(logger);
    }

    /// Attaches the logs database.
    pub fn set_logs_db(&mut self, logs_db: Arc<crate::logging::db::LogsDb>) {
        self.logs_db = Some(logs_db);
    }

    /// Helper to log a message via the centralized logger.
    fn log(&self, level: LogLevel, category: LogCategory, message: impl Into<String>, details: serde_json::Value) {
        if let Some(ref logger) = self.logger {
            let cycle = self.cycle_count;
            let msg = message.into();
            let logger = logger.clone();
            tokio::spawn(async move {
                let mut lg = logger.lock().await;
                lg.log(level, category, msg, details, cycle);
            });
        }
    }

    /// Returns the world context data for the REST API.
    /// Includes: time, season, weather, Saphire's age, etc.
    pub fn world_data(&mut self) -> serde_json::Value {
        self.world.ws_data()
    }

    /// Starts a manual memory consolidation (from the dashboard).
    ///
    /// Returns a JSON report with the number of consolidated, decayed
    /// and forgotten memories.
    pub async fn run_consolidation(&mut self) -> serde_json::Value {
        if let Some(ref db) = self.db {
            let params = consolidation::ConsolidationParams {
                threshold: self.config.memory.consolidation_threshold,
                decay_rate: self.config.memory.episodic_decay_rate,
                max_episodic: self.config.memory.episodic_max,
                episodic_prune_target: self.config.memory.episodic_prune_target,
                ltm_max: self.config.memory.ltm_max,
                ltm_prune_target: self.config.memory.ltm_prune_target,
                ltm_protection_access_count: self.config.memory.ltm_protection_access_count,
                ltm_protection_emotional_weight: self.config.memory.ltm_protection_emotional_weight,
                archive_batch_size: self.config.memory.archive_batch_size,
                bdnf_level: self.grey_matter.bdnf_level,
            };
            let report = consolidation::consolidate(
                db, self.encoder.as_ref(), &params,
            ).await;
            self.last_consolidation_cycle = self.cycle_count;

            self.log(LogLevel::Info, LogCategory::Memory,
                format!("Manual consolidation: {} consolidated, {} decayed, {} pruned, {} LTM pruned, {} archived",
                    report.consolidated, report.decayed, report.pruned,
                    report.ltm_pruned, report.archived),
                serde_json::json!({
                    "consolidated": report.consolidated,
                    "decayed": report.decayed,
                    "pruned": report.pruned,
                    "ltm_pruned": report.ltm_pruned,
                    "archived": report.archived,
                }),
            );

            serde_json::json!({
                "status": "ok",
                "consolidated": report.consolidated,
                "decayed": report.decayed,
                "pruned": report.pruned,
                "ltm_pruned": report.ltm_pruned,
                "archived": report.archived,
            })
        } else {
            serde_json::json!({"error": "DB not available"})
        }
    }

    /// Performs a clean shutdown of the Saphire agent.
    ///
    /// Operations performed in order:
    /// 1. Nocturnal memory consolidation (threshold lowered to 0.4 to transfer
    ///    more memories towards LTM before shutdown).
    /// 2. Save the OCEAN profile (self-profile + human profiles).
    /// 3. Update the identity self-description.
    /// 4. Save identity, tuning parameters, and bandit arms.
    /// 5. Close the session in PostgreSQL.
    /// 6. Mark the clean shutdown (flag `clean_shutdown = true`).
    /// 7. Broadcast the ShutdownStarted event to plugins.
    pub async fn shutdown(&mut self) {
        self.log(LogLevel::Info, LogCategory::Shutdown, "Clean shutdown in progress...", serde_json::json!({}));
        println!("\n  💤 Saphire s'endort...");
        tracing::info!("Clean shutdown in progress...");

        // Flush the logger
        if let Some(ref logger) = self.logger {
            let mut lg = logger.lock().await;
            lg.flush();
        }

        // ═══ Nocturnal consolidation (threshold lowered to 0.4) ═══
        // Analogous to human sleep: during shutdown, consolidation
        // is more aggressive (threshold at 0.4 instead of the normal threshold) to
        // transfer as many memories as possible towards long-term memory.
        if self.config.memory.consolidation_on_sleep {
            if let Some(ref db) = self.db {
                // Drain all working memory into episodic memory
                let wm_items = self.working_memory.drain_all();
                for item in wm_items {
                    let _ = db.store_episodic(
                        &item.content, item.source.label(),
                        &serde_json::json!({}), 0, &serde_json::json!({}),
                        &item.emotion_at_creation, 0.5, 0.4,
                        self.conversation_id.as_deref(),
                        Some(&item.chemical_signature),
                    ).await;
                }

                let params = consolidation::ConsolidationParams {
                    threshold: 0.4, // Threshold lowered during sleep                    decay_rate: self.config.memory.episodic_decay_rate,
                    max_episodic: self.config.memory.episodic_max,
                    episodic_prune_target: self.config.memory.episodic_prune_target,
                    ltm_max: self.config.memory.ltm_max,
                    ltm_prune_target: self.config.memory.ltm_prune_target,
                    ltm_protection_access_count: self.config.memory.ltm_protection_access_count,
                    ltm_protection_emotional_weight: self.config.memory.ltm_protection_emotional_weight,
                    archive_batch_size: self.config.memory.archive_batch_size,
                    bdnf_level: self.grey_matter.bdnf_level,
                };
                let report = consolidation::consolidate(
                    db, self.encoder.as_ref(), &params,
                ).await;
                tracing::info!(
                    "Nocturnal consolidation: {} consolidated, {} decayed, {} pruned, {} LTM pruned, {} archived",
                    report.consolidated, report.decayed, report.pruned,
                    report.ltm_pruned, report.archived
                );
            }
        }

        // Save the OCEAN profile before shutdown
        if self.config.profiling.enabled {
            // Force a final profile recompute with all observations
            self.self_profiler.force_recompute(self.cycle_count);
            if let Some(ref db) = self.db {
                let profile = self.self_profiler.profile();
                let ocean_json = serde_json::to_value(profile).unwrap_or_default();
                let _ = db.save_ocean_profile(
                    &ocean_json,
                    profile.data_points as i64,
                    profile.confidence as f32,
                    &serde_json::json!([]),
                ).await;
                tracing::info!("OCEAN profile saved (confidence: {:.0}%)", profile.confidence * 100.0);

                // Save the human profiles
                for (id, hp) in self.human_profiler.all_profiles() {
                    let ocean_json = serde_json::to_value(&hp.ocean).unwrap_or_default();
                    let style_json = serde_json::to_value(&hp.communication_style).unwrap_or_default();
                    let topics_json = serde_json::to_value(&hp.preferred_topics).unwrap_or_default();
                    let patterns_json = serde_json::to_value(&hp.emotional_patterns).unwrap_or_default();
                    let _ = db.save_human_profile(
                        id, &hp.name, &ocean_json, &style_json,
                        hp.interaction_count as i64, &topics_json, &patterns_json,
                        hp.rapport_score as f32,
                    ).await;
                }
            }
        }

        // Update the self-description before saving the identity
        self.identity.refresh_description();

        if let Some(ref db) = self.db {
            // Save the identity
            let _ = db.save_identity(&self.identity.to_json_value()).await;

            // Save the virtual body state
            if self.config.body.enabled {
                let body_json = self.body.to_persist_json();
                let _ = db.save_body_state(&body_json).await;
                tracing::info!("Virtual body saved ({} heartbeats)", self.body.heart.beat_count());
            }

            // Save the vital state (spark + intuition + premonition)
            // Extinguish the spark before persistence (runtime flag, not persisted)
            if self.config.vital_spark.enabled {
                self.vital_spark.sparked = false;
                self.vital_spark.sparked_at = None;
                tracing::info!("⚡ SPARK EXTINGUISHED — Saphire falls asleep.");
                let vital_json = serde_json::json!({
                    "spark": self.vital_spark.to_persist_json(),
                    "intuition": {
                        "acuity": self.intuition.acuity,
                        "accuracy": self.intuition.accuracy,
                    },
                    "premonition": {
                        "accuracy": self.premonition.accuracy,
                    },
                });
                let _ = db.save_vital_state(&vital_json).await;
                tracing::info!("⚡ Vital state saved (sparked: {}, acuity: {:.2})",
                    self.vital_spark.sparked, self.intuition.acuity);
            }

            // Save the micro neural network
            if self.config.plugins.micro_nn.enabled {
                if let Ok(nn_str) = self.micro_nn.to_json() {
                    let nn_json: serde_json::Value = serde_json::from_str(&nn_str).unwrap_or_default();
                    let _ = db.save_nn_state(&nn_json).await;
                    tracing::info!("🧠 NN saved ({} trainings)", self.micro_nn.train_count);
                }
            }

            // Save the sensory state (Sensorium)
            if self.config.senses.enabled {
                let senses_json = self.sensorium.to_persist_json();
                let _ = db.save_senses_state(&senses_json).await;
                tracing::info!("👁 Sensorium saved (emergence potential: {:.0}%)",
                    self.sensorium.emergence_potential * 100.0);
            }

            // Save the persistent psychological state
            if self.psychology.enabled {
                let psy_json = serde_json::json!({
                    "toltec": {
                        "agreements": self.psychology.toltec.agreements.iter().map(|a| {
                            serde_json::json!({
                                "times_invoked": a.times_invoked,
                                "violations_detected": a.violations_detected,
                            })
                        }).collect::<Vec<_>>(),
                    },
                    "shadow": {
                        "integration": self.psychology.jung.integration,
                        "traits": self.psychology.jung.shadow_traits.iter().map(|t| {
                            serde_json::json!({
                                "name": t.name,
                                "repressed_intensity": t.repressed_intensity,
                            })
                        }).collect::<Vec<_>>(),
                    },
                    "eq": {
                        "overall_eq": self.psychology.eq.overall_eq,
                        "growth_experiences": self.psychology.eq.growth_experiences,
                    },
                    "flow": {
                        "total_flow_cycles": self.psychology.flow.total_flow_cycles,
                    },
                    "maslow": {
                        "current_active_level": self.psychology.maslow.current_active_level,
                    },
                    "moral_reflection_count": self.moral_reflection_count,
                    "will": {
                        "willpower": self.psychology.will.willpower,
                        "total_deliberations": self.psychology.will.total_deliberations,
                        "proud_decisions": self.psychology.will.proud_decisions,
                        "regretted_decisions": self.psychology.will.regretted_decisions,
                        "recent_deliberations": self.psychology.will.recent_deliberations.iter().map(|d| {
                            serde_json::json!({
                                "trigger": format!("{:?}", d.trigger.trigger_type),
                                "chosen": d.options.get(d.chosen).map(|o| o.description.as_str()).unwrap_or("?"),
                                "confidence": d.confidence,
                                "reasoning": d.reasoning,
                                "regret": d.regret,
                                "created_at": d.created_at.to_rfc3339(),
                                "chemistry_influence": {
                                    "boldness": d.chemistry_influence.boldness,
                                    "caution": d.chemistry_influence.caution,
                                    "wisdom": d.chemistry_influence.wisdom,
                                    "efficiency": d.chemistry_influence.efficiency,
                                    "urgency": d.chemistry_influence.urgency,
                                    "empathy": d.chemistry_influence.empathy,
                                },
                            })
                        }).collect::<Vec<_>>(),
                    },
                });
                let _ = db.save_psychology_state(&psy_json).await;
                tracing::info!("Psychology saved (EQ: {:.0}%, shadow integration: {:.0}%, willpower: {:.0}%)",
                    self.psychology.eq.overall_eq * 100.0, self.psychology.jung.integration * 100.0,
                    self.psychology.will.willpower * 100.0);
            }

            // Save the character values
            if self.values.enabled {
                let values_json = self.values.to_json();
                let _ = db.save_values_state(&values_json).await;
                let top3: Vec<String> = self.values.top_values(3).iter()
                    .map(|v| format!("{} {:.0}%", v.name, v.score * 100.0))
                    .collect();
                tracing::info!("Values saved ({})", top3.join(", "));
            }

            // Save the nutritional system
            if self.config.nutrition.enabled {
                let nutr_json = self.nutrition.to_json();
                let _ = db.save_nutrition_state(&nutr_json).await;
                tracing::info!("Nutrition saved (ATP: {:.0}%, vit_D: {:.0}%)",
                    self.nutrition.energy.atp_reserves * 100.0, self.nutrition.vitamins.d * 100.0);
            }

            // Save the grey matter
            if self.config.grey_matter.enabled {
                let gm_json = self.grey_matter.to_json();
                let _ = db.save_grey_matter_state(&gm_json).await;
                tracing::info!("Grey matter saved (volume: {:.0}%, BDNF: {:.0}%)",
                    self.grey_matter.grey_matter_volume * 100.0, self.grey_matter.bdnf_level * 100.0);
            }

            // Save the electromagnetic fields
            if self.config.fields.enabled {
                let fields_json = self.em_fields.to_json();
                let _ = db.save_fields_state(&fields_json).await;
                tracing::info!("EM fields saved (Schumann: {:.2} Hz, aura: {:.0}%)",
                    self.em_fields.universal.schumann_resonance, self.em_fields.biofield.aura_luminosity * 100.0);
            }

            // Save the affective bond network
            if let Ok(rel_json) = serde_json::to_value(&self.relationships) {
                let _ = db.save_relationships_state(&rel_json).await;
                tracing::info!("Affective bonds saved ({} bonds)", self.relationships.bonds.len());
            }

            // Save the metacognitive state (thought quality + Turing)
            if self.metacognition.enabled {
                if let Ok(meta_json) = serde_json::to_value(&self.metacognition) {
                    let _ = db.save_metacognition_state(&meta_json).await;
                    tracing::info!("Metacognition saved (Turing: {:.1}/100, milestone: {})",
                        self.metacognition.turing.score, self.metacognition.turing.milestone.as_str());
                }
            }

            // Save the orchestrators
            self.save_orchestrators_to_db(db).await;

            // Save the tuning
            let params_json: serde_json::Value = serde_json::from_str(&self.tuner.params_json()).unwrap_or_default();
            let best_json: serde_json::Value = serde_json::from_str(&self.tuner.best_params_json()).unwrap_or_default();
            let _ = db.save_tuning_params(
                &params_json,
                &best_json,
                self.tuner.best_score() as f32,
                self.tuner.tuning_count as i32,
            ).await;

            // Save the bandit arms
            let arms = self.thought_engine.export_bandit_arms();
            let _ = db.save_bandit_arms(&arms).await;

            // Close the session
            let _ = db.end_session(self.session_id, self.cycle_count as i32, true).await;

            // Mark the clean shutdown
            let _ = db.set_clean_shutdown(true).await;
        }

        // Broadcast
        self.plugins.broadcast(&BrainEvent::ShutdownStarted);

        println!("  💎 {} s'endort après {} cycles. Bonne nuit.", self.identity.name, self.cycle_count);
    }
}

/// Complete result of a stimulus processing through the brain pipeline.
///
/// Groups the outputs of the different stages for easy access
/// by logging, broadcast and profiling functions.
pub struct ProcessResult {
    /// Consensus result between the 3 brain modules
    /// (decision, score, weight, coherence)
    pub consensus: crate::consensus::ConsensusResult,

    /// Emotional state computed from the current chemistry
    /// (dominant emotion, secondary, valence, arousal)
    pub emotion: EmotionalState,

    /// Consciousness state evaluated by IIT
    /// (Integrated Information Theory)
    /// (level, phi, inner narrative)
    pub consciousness: crate::consciousness::ConsciousnessState,

    /// Ethical regulation verdict (Asimov's laws)
    /// (approved decision, possible veto, detected violations)
    pub verdict: crate::regulation::RegulationVerdict,

    /// Partial cognitive trace built by process_stimulus().
    /// The caller completes it (NLP, LLM, memory, duration) then saves it.
    pub trace: Option<crate::logging::trace::CognitiveTrace>,
}

/// Safely truncates a UTF-8 string to `max_bytes` bytes.
///
/// Since UTF-8 characters can be 1 to 4 bytes, we cannot simply cut at
/// index `max_bytes` as this could split a multi-byte character in the
/// middle. This function backs up until it finds a valid character boundary.
///
/// Parameters:
/// - `s`: the string to truncate.
/// - `max_bytes`: maximum number of bytes in the result.
///
/// Returns: a slice of the original string, of size <= max_bytes.
fn truncate_utf8(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes { return s; }
    let mut end = max_bytes;
    // Back up until a valid character boundary is found
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}
