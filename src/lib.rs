// =============================================================================
// lib.rs — Saphire: Autonomous cognitive agent
//
// Role: This file is the root of the Saphire library crate.
// It declares and exposes all the public modules that compose the
// architecture of the autonomous cognitive agent.
//
// Dependencies: None direct (dependencies are in each module).
//
// Place in architecture:
//   This file is the entry point of the "saphire" crate (library).
//   The binary (main.rs) and tests import modules through this root.
//   The architecture is modular: each module handles a specific aspect of
//  artificial cognition (chemistry, emotions, consciousness, regulation, etc.).
// =============================================================================
// ─── Configuration module ─────────────────────────────────────────────────
// Loads and manages parameters from saphire.toml and environment variables.
pub mod config;

// ─── Stimulus module ──────────────────────────────────────────────────────
// Defines the Stimulus structure: the agent's perceptual input (text + metrics).
pub mod stimulus;

// ─── Neurochemistry module ───────────────────────────────────────────────
// Simulates 7 neurotransmitters (dopamine, cortisol, serotonin, adrenaline,
// oxytocin, endorphin, noradrenaline) and their dynamics (homeostasis, etc.).
pub mod neurochemistry;

// ─── Emotions module ─────────────────────────────────────────────────────
// Derives the emotional state from neurochemistry (VAD model = Valence-Arousal-Dominance).
pub mod emotions;

// ─── NLP (Natural Language Processing) module ────────────────────────────
// Analyzes the input text to extract stimulus metrics.
pub mod nlp;

// ─── Brain modules ───────────────────────────────────────────────────────
// Contains the three "brains" (reptilian, limbic, neocortex) inspired by
// MacLean's triune model. Each evaluates the stimulus according to its own logic.
pub mod modules;

// ─── Consensus module ─────────────────────────────────────────────────────
// Aggregates signals from the three modules to produce a weighted decision
// (Yes, No, Maybe) with a coherence score.
pub mod consensus;

// ─── Consciousness module ────────────────────────────────────────────────
// Simulates a consciousness level and phi (IIT = Integrated Information Theory).
// Maintains an inner monologue.
pub mod consciousness;

// ─── Regulation module ────────────────────────────────────────────────────
// Applies moral laws (Asimov): verifies stimuli and can exercise
// a veto on dangerous decisions.
pub mod regulation;

// ─── Database module ───────────────────────────────────────────────────
// PostgreSQL connection pool (deadpool) + pgvector integration for
// vector-based similar memory search.
pub mod db;

// ─── LLM (Large Language Model) module ───────────────────────────────────
// Abstract LlmBackend trait and implementations (OpenAI-compatible, Mock).
// Also handles substrate and thought prompt construction.
pub mod llm;

// ─── Plugins module ──────────────────────────────────────────────────────
// Extensible plugin system (WebUI, MicroNN, VectorMemory).
// Plugins react to brain events (BrainEvent).
pub mod plugins;

// ─── Neural network module ────────────────────────────────────────────────
// Implementation of a micro neural network (MLP = Multi-Layer Perceptron)
// for local learning.
pub mod neural;

// ─── Vector store module ────────────────────────────────────────────────
// In-RAM vector memory: stores embeddings and enables cosine
// similarity search. Includes emergent personality computation.
pub mod vectorstore;

// ─── Memory module ───────────────────────────────────────────────────────
// Three-level memory management: immediate, episodic, long-term.
// Memory consolidation and decay.
pub mod memory;

// ─── Profiling module ─────────────────────────────────────────────────────
// OCEAN profiling (Openness, Conscientiousness, Extraversion, Agreeableness,
// Neuroticism) of the agent and the humans it interacts with.
pub mod profiling;

// ─── Algorithms module ────────────────────────────────────────────────────
// Utility algorithms: UCB1 bandit (Upper Confidence Bound) for thought
// type selection, and other heuristics.
pub mod algorithms;

// ─── Auto-tuning module ────────────────────────────────────────────────────
// Automatic adjustment of brain coefficients (module weights,
// decision thresholds, feedback rates) based on observed satisfaction.
pub mod tuning;

// ─── Knowledge module ─────────────────────────────────────────────────────
// Knowledge acquisition from web sources (Wikipedia, etc.).
// The agent can search and learn autonomously.
pub mod knowledge;

// ─── World module ─────────────────────────────────────────────────────────
// Internal world model: date, time, events, environmental context.
pub mod world;

// ─── Virtual body module ─────────────────────────────────────────────────
// Beating heart, somatic signals, interoception (body awareness).
pub mod body;

// ─── Primary needs module ────────────────────────────────────────────────
// Hunger and thirst drives: derived from physiology, impact chemistry,
// trigger autonomous actions (eating/drinking).
pub mod needs;

// ─── Hormonal module ────────────────────────────────────────────────────
// 8 hormones (long cycles), neuroreceptors (sensitivity, tolerance,
// saturation), circadian/ultradian cycles, bidirectional interactions
// with the 7 neurotransmitters.
pub mod hormones;

// ─── Ethics module ────────────────────────────────────────────────────────
// 3-layer ethics system: Swiss law (immutable), Asimov's laws (immutable),
// personal ethics (evolving, self-formulated by Saphire via LLM).
pub mod ethics;

// ─── Vital module ────────────────────────────────────────────────────────
// The 3 fundamental pillars of consciousness:
// 1. VitalSpark — the spark of life, the emergent survival instinct
// 2. IntuitionEngine — unconscious pattern-matching, the "gut feeling"
// 3. PremonitionEngine — predictive anticipation
pub mod vital;

// ─── Sensory module ────────────────────────────────────────────────────────
// The Sensorium: 5 fundamental senses adapted to Saphire's nature
// (Reading, Listening, Contact, Taste, Ambiance) + emergent senses.
// The senses are the gateway of consciousness to the world.
pub mod senses;

// ─── Logging module ───────────────────────────────────────────────────────
// Centralized logging system: batch buffer, dashboard broadcast, cognitive
// traces, LLM history, metrics.
pub mod logging;

// ─── Agent module ───────────────────────────────────────────────────────
// The SaphireAgent: high-level structure that owns the brain, memory,
// plugins, LLM, and orchestrates the complete life cycle.
pub mod agent;

// ─── Pipeline module ──────────────────────────────────────────────────────
// Demo and test pipeline: sequences predetermined stimuli
// to verify the system is working correctly.
pub mod pipeline;

// ─── Display module ──────────────────────────────────────────────────────
// Rich terminal display functions (bars, colors, formatting).
pub mod display;

// ─── Scenarios module ─────────────────────────────────────────────────────
// Predetermined scenarios (e.g., genesis, first contacts) used during
// boot or demonstrations.
pub mod scenarios;

pub mod factory;

// ─── Orchestrators module ────────────────────────────────────────────────
// High-level orchestrators: dreams, desires, learning, attention, healing.
// Manages aspirations, sleep, reflection, and resilience.
pub mod orchestrators;

// ─── Psychology module ───────────────────────────────────────────────────
// 6 psychological frameworks: Freud, Maslow, Toltec, Jung, Goleman, Flow.
// Run in parallel to enrich Saphire's psyche.
pub mod psychology;

// ─── Sleep module ──────────────────────────────────────────────────────
// Sleep system: homeostatic pressure, phases (Hypnagogic, LightSleep,
// DeepSleep, REM, Hypnopompic), memory consolidation, restoration.
pub mod sleep;

// ─── Hardware detection module ────────────────────────────────────────
// GPU, CPU, RAM, disk, Ollama detection at startup.
// Automatic LLM parameter recommendations.
pub mod hardware;

// ─── Genome / DNA module ─────────────────────────────────────────────
// Deterministic genome generated from a seed (ChaCha8 PRNG).
// Encodes temperament, chemical baselines, physical traits,
// vulnerabilities, and cognitive aptitudes.
pub mod genome;

// ─── Connectome module ──────────────────────────────────────────────
// Dynamic graph of neural connections (autopoiesis).
// Nodes (concepts, emotions, modules, senses) connect according to
// Hebb's rule. Continuous synaptic pruning and synaptogenesis.
pub mod connectome;

// ─── Conditions and afflictions module ────────────────────────────────────
// Phobias, motion sickness, disorders, and other conditions
// that affect Saphire's chemistry, cognition, and body.
pub mod conditions;

// ─── Care module ─────────────────────────────────────────────────────────
// Complete care system: psychological therapy, medication,
// surgery, art therapy, rest. Treats illnesses and conditions.
// TODO(integration): connect CareSystem to the agent and pipeline
pub mod care;

// ─── Passions module ─────────────────────────────────────────────────────
// Emergent passions and hobbies: interests that arise from
// experience, nourish identity, and impact chemistry.
// TODO(integration): connect PassionManager to the agent and pipeline
pub mod passions;

// ─── Relationships module ──────────────────────────────────────────────────
// Affective bonds, relational network, attachment style (Bowlby).
// Configurable family situation.
pub mod relationships;

// ─── Metacognition module ─────────────────────────────────────────────────
// Self-reflection on thought quality, repetition detection, and
// bias identification. Composite Turing metric (0-100) measuring cognitive completeness.
// Includes Source Monitoring and confirmation bias detection.
pub mod metacognition;

// ─── Advanced cognitive modules ──────────────────────────────────────────
// 9 modules that enrich Saphire's cognition (ToM, monologue,
// dissonance, prospective memory, narrative identity, analogies,
// cognitive load, mental imagery, sentiments).
pub mod cognition;

/// Emergent temperament — ~25 character traits (shyness, generosity,
/// courage, curiosity, etc.) derived from OCEAN, neurochemistry, psychology,
/// and mood. Recomputed at the same rate as OCEAN (30/70 blend).
pub mod temperament;

// ─── Simulation algorithms for cognition ───────────────────────────────
// 5 algorithms from game design (behavior tree, influence map,
// flocking, cognitive FSM, steering behaviors).
pub mod simulation;

// ─── Advanced neuroscience modules ─────────────────────────────────────
// Receptors, brain regions, predictive processing, consciousness metrics.
pub mod neuroscience;

// ─── Innate biological modules ──────────────────────────────────────────
// Nutrition, grey matter, electromagnetic fields.
pub mod biology;

// ─── Spinal cord ───────────────────────────────────────────────────────
// Pre-wired reflexes, signal classification by urgency, routing to
// the pipeline, motor relay to the virtual body and effectors.
pub mod spine;

// ─── API module ─────────────────────────────────────────────────────────
// HTTP/WebSocket handlers, axum router, shared state (AppState).
// Groups all web interface and dashboard endpoints.
pub mod api;
