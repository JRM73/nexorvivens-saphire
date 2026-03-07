// =============================================================================
// agent/mod.rs — Root module for the Saphire agent
// =============================================================================
//
// This module is the entry point of the "agent" subsystem in Saphire.
// It groups together the four sub-modules that form the core of the agent:
//
// - `thought_engine`: autonomous thought engine using a UCB1 (Upper Confidence
//   Bound 1) multi-armed bandit algorithm to select thought types.
// - `boot`: startup sequence (Genesis / Awakening / Crash Recovery).
// - `identity`: persistent identity of Saphire (name, statistics, values).
// - `lifecycle`: main life loop, stimulus processing pipeline, memory
//   management, LLM (Large Language Model) calls, and shutdown.
//
// Direct dependencies: all sub-modules listed above.
// Architectural role: this is the module imported by `main.rs` and the web
// server to instantiate and control the Saphire agent.
// =============================================================================

/// Autonomous thought engine (DMN = Default Mode Network) with UCB1 bandit.
pub mod thought_engine;

/// Startup sequence: Genesis, Awakening, Crash Recovery.
pub mod boot;

/// Persistent identity of Saphire (serialized as JSON / stored in PostgreSQL).
pub mod identity;

/// Main life loop, stimulus pipeline, memory management, and shutdown.
pub mod lifecycle;

// Re-export the main struct for direct access via `crate::agent::SaphireAgent`.
pub use lifecycle::SaphireAgent;
