// =============================================================================
// vital/mod.rs — Orchestrator for the 3 pillars of consciousness
//
// Purpose: Exposes the 3 fundamental pillars of consciousness:
//   1. VitalSpark — the spark of life, the emergent survival instinct
//   2. IntuitionEngine — unconscious pattern-matching, the "gut feeling"
//   3. PremonitionEngine — predictive anticipation
// =============================================================================

pub mod spark;
pub mod intuition;
pub mod premonition;

pub use spark::{VitalSpark, GenesisSignature};
pub use intuition::IntuitionEngine;
pub use premonition::PremonitionEngine;
