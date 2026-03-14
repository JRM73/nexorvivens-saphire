// =============================================================================
// conditions/mod.rs — Conditions, afflictions, and physiological states
// =============================================================================
//
// Purpose: Groups condition modules that affect Saphire:
//          phobias, motion sickness, eating disorders, etc.
//          Each condition can impact chemistry, body, and cognition.
//
// Architecture:
//   Conditions are checked in the cognitive pipeline and can
//   modify chemical baselines, trigger somatic reactions,
//   or degrade cognition.
// =============================================================================

pub mod motion_sickness;
pub mod phobias;
pub mod eating;
pub mod disabilities;
pub mod extreme;
pub mod addictions;
pub mod trauma;
pub mod nde;
pub mod drugs;
pub mod iq_constraint;
pub mod sexuality;
pub mod degenerative;
pub mod medical;
pub mod culture;
pub mod precarity;
pub mod employment;
