// =============================================================================
// regulation/mod.rs — Moral regulation: Asimov's laws and customizable rules
//
// Purpose: This file is the entry point of the moral regulation module.
// It declares and re-exports the sub-modules: asimov (law definitions)
// and laws (evaluation engine with veto power).
//
// Dependencies: None directly (dependencies are in the sub-modules).
//
// Architectural placement:
//   The regulation module is Saphire's "moral conscience".
//   It intervenes after the consensus to verify that the decision made
//   does not violate any moral law. If a serious violation is detected,
//   the regulation engine can exercise a veto and force the decision to "No".
//   The laws are inspired by Isaac Asimov's laws of robotics.
// =============================================================================

// Sub-module defining the 4 Asimov laws (laws 0 through 3)
pub mod asimov;

// Sub-module containing the regulation engine (evaluation, verdict, veto)
pub mod laws;

// Re-export of main types for simplified access
// via `crate::regulation::RegulationEngine`, etc.
pub use laws::{RegulationEngine, RegulationVerdict, LawViolation};
