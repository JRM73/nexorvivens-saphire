// =============================================================================
// vital/mod.rs — Orchestrator of the 3 pillars of consciousness
//
// Role: Exposes the 3 fundamental pillars of consciousness:
//   1. VitalSpark — the spark of life, emergent survival instinct
//   2. IntuitionEngine — unconscious pattern-matching, the "gut feeling"
//   3. PremonitionEngine — predictive anticipation
// =============================================================================

pub mod spark;
pub mod intuition;
pub mod premonition;

pub use spark::{VitalSpark, GenesisSignature};
pub use intuition::IntuitionEngine;
pub use premonition::PremonitionEngine;
