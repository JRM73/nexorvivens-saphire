// =============================================================================
// conditions/mod.rs — Conditions, afflictions et etats physiologiques
// =============================================================================
//
// Role : Regroupe les modules de conditions qui affectent Saphire :
//        phobies, cinetose, troubles alimentaires, etc.
//        Chaque condition peut impacter la chimie, le corps, la cognition.
//
// Place dans l'architecture :
//   Les conditions sont verifiees dans le pipeline cognitif et peuvent
//   modifier les baselines chimiques, declencher des reactions somatiques,
//   ou degrader la cognition.
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
