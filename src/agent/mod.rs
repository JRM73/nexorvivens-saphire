// =============================================================================
// agent/mod.rs — Root module of the Saphire agent
// =============================================================================
//
// This module is the entry point of the "agent" subsystem of Saphire.
// It groups the four sub-modules that constitute the heart of the agent:
//
// - `thought_engine` : autonomous thought engine, using a UCB1
//   (Upper Confidence Bound 1) algorithm to select the thought type.
// - `boot` : boot sequence (Genesis / Awakening / Crash Recovery).
// - `identity` : persistent identity of Saphire (name, statistics, values).
// - `lifecycle` : main life loop, stimulus processing pipeline,
//   memory management, LLM (Large Language Model) calls, and shutdown.
//
// Direct dependencies: all sub-modules listed above.
// Place in architecture: this is the module imported by `main.rs` and by
// the web server to instantiate and drive the Saphire agent.
// =============================================================================
/// Autonomous thought engine (DMN = Default Mode Network) with UCB1 bandit
pub mod thought_engine;

/// Boot sequence: Genesis, Awakening, Crash Recovery
pub mod boot;

/// Persistent identity of Saphire (serialized in JSON / PostgreSQL)
pub mod identity;

/// Main life loop, stimulus pipeline, memory management and shutdown
pub mod lifecycle;

// Re-export of the main structure for direct access via `crate::agent::SaphireAgent`
pub use lifecycle::SaphireAgent;
