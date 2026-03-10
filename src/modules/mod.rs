// =============================================================================
// modules/mod.rs — Les 3 modules cérébraux de Saphire
// =============================================================================
//
// Rôle : Ce fichier déclare les sous-modules cérébraux et définit les types
// communs partagés par les 3 modules : le `ModuleSignal` (sortie d'un module)
// et le trait `BrainModule` (interface commune).
//
// Dépendances :
//   - serde : sérialisation / désérialisation
//   - crate::neurochemistry::NeuroChemicalState : état chimique (paramètre de traitement)
//   - crate::stimulus::Stimulus : entrée sensorielle (paramètre de traitement)
//
// Place dans l'architecture :
//   Ce module regroupe les 3 « cerveaux » biologiquement inspirés :
//     - reptilian.rs : cerveau reptilien (survie, danger, réflexes)
//     - limbic.rs : système limbique (émotions, récompense, liens sociaux)
//     - neocortex.rs : néocortex (analyse rationnelle, coût/bénéfice)
//   Chaque module implémente le trait `BrainModule` et émet un `ModuleSignal`.
//   Les 3 signaux sont ensuite combinés par consensus.rs pour produire
//   la décision finale.
// =============================================================================

/// Sous-module reptilien : réactions de survie et détection du danger
pub mod reptilian;
/// Sous-module limbique : traitement émotionnel et récompense
pub mod limbic;
/// Sous-module néocortex : analyse rationnelle et clarté mentale
pub mod neocortex;

use serde::{Deserialize, Serialize};
use crate::neurochemistry::NeuroChemicalState;
use crate::stimulus::Stimulus;

/// Signal émis par un module cérébral — résultat du traitement d'un stimulus.
///
/// Chaque module produit un signal unique qui sera combiné avec les autres
/// dans le consensus pondéré.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSignal {
    /// Nom du module émetteur ("Reptilien", "Limbique" ou "Néocortex")
    pub module: String,
    /// Signal [-1, +1] : opinion du module sur le stimulus.
    /// Négatif = rejet / danger, positif = approbation / attrait.
    pub signal: f64,
    /// Confiance dans le signal [0, 1] : certitude du module dans sa réponse.
    /// 1.0 = totalement certain, 0.0 = aucune certitude.
    pub confidence: f64,
    /// Raisonnement textuel en français : explication de la réponse du module.
    pub reasoning: String,
}

/// Trait commun aux 3 modules cérébraux — interface que chaque module
/// doit implémenter pour participer au consensus.
pub trait BrainModule {
    /// Retourne le nom du module (utilisé pour l'affichage et le logging).
    fn name(&self) -> &str;

    /// Traite un stimulus en tenant compte de l'état neurochimique actuel.
    ///
    /// # Paramètres
    /// - `stimulus` : entrée sensorielle à traiter.
    /// - `chemistry` : état chimique actuel (influence le traitement).
    ///
    /// # Retour
    /// Un `ModuleSignal` contenant le signal, la confiance et le raisonnement.
    fn process(&self, stimulus: &Stimulus, chemistry: &NeuroChemicalState) -> ModuleSignal;
}
