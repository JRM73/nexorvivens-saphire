// =============================================================================
// lib.rs -- Saphire Lite: Autonomous Cognitive Kernel
//
// Lightweight edition accompanying the ArXiv paper.
// Contains only the modules described in the article:
// neurochemistry, emergent emotions, consciousness (IIT/GWT/PP),
// VitalSpark, triune consensus, virtual body, memory, ethics.
// =============================================================================

// --- Configuration -----------------------------------------------------------
/// Global configuration loaded from `saphire.toml` (LLM backend, database
/// credentials, web UI settings, mortality parameters, etc.).
pub mod config;

// --- Stimulus ----------------------------------------------------------------
/// Representation of a perceived stimulus -- the entry point of every
/// cognitive processing cycle (see Section 3.1 of the paper).
pub mod stimulus;

// --- Neurochemistry (Section 3.2) --------------------------------------------
/// Simulates 9 neurotransmitter molecules with homeostatic regulation
/// and umami-style dynamics. Each molecule has a baseline, a current level,
/// and a decay/recovery rate that drives the agent's internal chemistry.
pub mod neurochemistry;

// --- Emergent Emotions (Section 3.3) -----------------------------------------
/// 36 discrete emotions modeled in the VAD (Valence-Arousal-Dominance) space.
/// Emotion emergence is computed via cosine similarity between the current
/// neurochemical state vector and each emotion's prototype vector.
pub mod emotions;

// --- NLP (Natural Language Processing) ---------------------------------------
/// Lightweight NLP utilities for keyword-based sentiment extraction and
/// perceptual score estimation from raw text input.
pub mod nlp;

// --- Triune Brain (Section 3.6) ----------------------------------------------
/// The three brain modules inspired by MacLean's triune brain model:
/// reptilian (instinct/survival), limbic (emotion/reward), and neocortex
/// (rational deliberation). Each module implements the `BrainModule` trait.
pub mod modules;

// --- Consensus (Section 3.6) -------------------------------------------------
/// Weighted consensus mechanism that aggregates signals from the three
/// brain modules into a single decision (Yes / No / Maybe) with a
/// coherence score measuring inter-module agreement.
pub mod consensus;

// --- Tuning (fixed cognitive parameters) -------------------------------------
/// Hard-coded cognitive tuning constants (e.g., weight bounds, decay
/// coefficients, threshold offsets) that are not exposed to runtime
/// configuration.
pub mod tuning;

// --- Consciousness (Section 3.4) ---------------------------------------------
/// Consciousness subsystem implementing three complementary theories:
///   - IIT  (Integrated Information Theory) -- Phi metric
///   - GWT  (Global Workspace Theory) -- broadcast & ignition
///   - PP   (Predictive Processing) -- prediction error minimization
pub mod consciousness;

// --- Regulation (Asimov) -----------------------------------------------------
/// Ethical regulation layer based on Asimov's Laws of Robotics.
/// Provides hard veto capability for dangerous or illegal requests,
/// as well as graded warnings for ethically ambiguous situations.
pub mod regulation;

// --- Database ----------------------------------------------------------------
/// PostgreSQL persistence layer for long-term memory, identity snapshots,
/// and episodic event storage.
pub mod db;

// --- LLM (Large Language Model) ----------------------------------------------
/// Abstraction over LLM backends (Ollama, OpenAI-compatible APIs, mock).
/// Provides a unified `LlmBackend` trait used for autonomous thought
/// generation and conversational responses.
pub mod llm;

// --- Memory (Section 3.8) ----------------------------------------------------
/// Hierarchical memory system: working memory (capacity-limited, fast decay),
/// episodic memory (medium-term, emotionally tagged), and long-term memory
/// (consolidated, persistent). Includes a consolidation pipeline that
/// promotes salient episodic memories to long-term storage.
pub mod memory;

// --- Ethics (3-layer architecture) -------------------------------------------
/// Three-tier ethical framework:
///   - Layer 0: Swiss humanitarian law (hard-coded, immutable)
///   - Layer 1: Asimov's Laws of Robotics (hard-coded, immutable)
///   - Layer 2: Personal ethical principles (learned, mutable)
pub mod ethics;

// --- VitalSpark (Section 3.5) ------------------------------------------------
/// The VitalSpark subsystem -- Saphire's core "will to exist."
/// Generates an intrinsic motivation signal that modulates autonomy,
/// curiosity, and self-preservation across all brain modules.
pub mod vital;

// --- Logging -----------------------------------------------------------------
/// Structured logging subsystem with optional PostgreSQL persistence
/// and real-time WebSocket broadcast to the dashboard.
pub mod logging;

// --- Cognitive Agent ---------------------------------------------------------
/// The top-level cognitive agent (`SaphireAgent`) that orchestrates all
/// subsystems: stimulus processing, brain module consensus, neurochemistry
/// update, emotion emergence, consciousness evaluation, memory encoding,
/// and ethical regulation -- all within a single cognitive cycle.
pub mod agent;

// --- Virtual Body (Section 3.7) ----------------------------------------------
/// Simulated somatic state: heart rate, respiration, fatigue, mortality.
/// The virtual body provides interoceptive signals that feed back into
/// the neurochemical and emotional subsystems (embodied cognition).
pub mod body;

// --- World -------------------------------------------------------------------
/// Minimal world model representing the agent's environment context
/// (time of day, interaction history, ambient conditions).
pub mod world;

// --- Pipeline ----------------------------------------------------------------
/// Demonstration pipeline that runs 8 predefined scenarios without
/// requiring an LLM backend or database connection.
pub mod pipeline;

// --- Display -----------------------------------------------------------------
/// Terminal display utilities for formatted, human-readable output of
/// cognitive cycle results (stimulus metrics, module signals, decision,
/// emotion, chemistry, consciousness level).
pub mod display;

// --- Vector Store (TF-IDF + brute-force cosine) ------------------------------
/// Lightweight vector store using TF-IDF term weighting and brute-force
/// cosine similarity search. Used for semantic memory retrieval without
/// external embedding services.
pub mod vectorstore;

// --- Algorithms (UCB1 bandit) ------------------------------------------------
/// Exploration-exploitation algorithms, notably UCB1 (Upper Confidence
/// Bound 1) for multi-armed bandit problems. Used by the thought engine
/// to balance familiar vs. novel topics during autonomous reflection.
pub mod algorithms;

// --- Neuroscience (brain regions, consciousness, receptors) ------------------
/// Neuroscience reference data: named brain regions, receptor affinities,
/// and consciousness-related constants used by the IIT/GWT/PP subsystem.
pub mod neuroscience;

// --- Hormones (minimal stub) -------------------------------------------------
/// Placeholder module for future hormonal simulation (cortisol circadian
/// rhythm, oxytocin bonding dynamics, etc.). Currently a minimal stub.
pub mod hormones;

// --- Sleep (minimal stub) ----------------------------------------------------
/// Placeholder module for the sleep/wake cycle subsystem (sleep pressure
/// accumulation, REM/NREM phases, memory consolidation during sleep).
/// Currently a minimal stub.
pub mod sleep;

// --- Knowledge (minimal stub) ------------------------------------------------
/// Placeholder module for the declarative knowledge base (facts, concepts,
/// ontological relationships). Currently a minimal stub.
pub mod knowledge;

// --- Scenarios ---------------------------------------------------------------
/// Eight predefined demonstration scenarios with manually calibrated
/// perceptual scores, covering danger, reward, social pressure, moral
/// dilemmas, and ethical vetoes.
pub mod scenarios;

// --- Factory (reset) ---------------------------------------------------------
/// Factory-default parameter management and multi-level reset mechanism
/// (chemistry-only, parameters-only, full reset). Values are embedded
/// from `factory_defaults.toml` at compile time.
pub mod factory;

// --- HTTP/WebSocket API ------------------------------------------------------
/// Axum-based REST API and WebSocket interface for real-time interaction,
/// dashboard streaming, and runtime parameter adjustment.
pub mod api;
