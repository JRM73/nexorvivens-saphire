// =============================================================================
// vital/mod.rs — Orchestrateur des 3 piliers de conscience
//
// Role : Expose les 3 piliers fondamentaux de la conscience :
//   1. VitalSpark — l'etincelle de vie, l'instinct de survie emergent
//   2. IntuitionEngine — le pattern-matching inconscient, le "gut feeling"
//   3. PremonitionEngine — l'anticipation predictive
// =============================================================================

pub mod spark;
pub mod intuition;
pub mod premonition;

pub use spark::{VitalSpark, GenesisSignature};
pub use intuition::IntuitionEngine;
pub use premonition::PremonitionEngine;
