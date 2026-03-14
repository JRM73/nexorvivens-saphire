// =============================================================================
// regulation/mod.rs — Moral regulation: Asimov laws and custom rules
//
// Role: This file is the entry point of the moral regulation module.
// It declares and re-exports the sub-modules: asimov (law definitions)
// and laws (evaluation engine with veto power).
//
// Dependencies: None direct (dependencies are in the sub-modules).
//
// Place in the architecture:
//   The regulation module is Saphire's "moral conscience".
//   It intervenes after consensus to verify that the decision taken
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
