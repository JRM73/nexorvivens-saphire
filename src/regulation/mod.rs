// =============================================================================
// regulation/mod.rs — Regulation morale : lois d'Asimov et regles personnalisees
//
// Role : Ce fichier est le point d'entree du module de regulation morale.
// Il declare et reexporte les sous-modules : asimov (definition des lois)
// et laws (moteur d'evaluation avec pouvoir de veto).
//
// Dependances : Aucune directe (les dependances sont dans les sous-modules).
//
// Place dans l'architecture :
//   Le module de regulation est la "conscience morale" de Saphire.
//   Il intervient apres le consensus pour verifier que la decision prise
//   ne viole aucune loi morale. Si une violation grave est detectee,
//   le moteur de regulation peut exercer un veto et forcer la decision a "Non".
//   Les lois sont inspirees des lois de la robotique d'Isaac Asimov.
// =============================================================================

// Sous-module definissant les 4 lois d'Asimov (lois 0 a 3)
pub mod asimov;

// Sous-module contenant le moteur de regulation (evaluation, verdict, veto)
pub mod laws;

// Reexportation des types principaux pour un acces simplifie
// depuis `crate::regulation::RegulationEngine`, etc.
pub use laws::{RegulationEngine, RegulationVerdict, LawViolation};
