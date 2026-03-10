// =============================================================================
// agent/mod.rs — Module racine de l'agent Saphire
// =============================================================================
//
// Ce module est le point d'entree du sous-systeme "agent" de Saphire.
// Il regroupe les quatre sous-modules qui constituent le coeur de l'agent :
//
// - `thought_engine` : moteur de pensee autonome, utilisant un algorithme
//   UCB1 (Upper Confidence Bound 1) pour selectionner le type de pensee.
// - `boot` : sequence de demarrage (Genesis / Awakening / Crash Recovery).
// - `identity` : identite persistante de Saphire (nom, statistiques, valeurs).
// - `lifecycle` : boucle de vie principale, pipeline de traitement des stimuli,
//   gestion de la memoire, appels au LLM (Large Language Model), et shutdown.
//
// Dependances directes : tous les sous-modules ci-dessus.
// Place dans l'architecture : c'est le module importe par `main.rs` et par
// le serveur web pour instancier et piloter l'agent Saphire.
// =============================================================================

/// Moteur de pensee autonome (DMN = Default Mode Network) avec bandit UCB1
pub mod thought_engine;

/// Sequence de demarrage : Genesis, Awakening, Crash Recovery
pub mod boot;

/// Identite persistante de Saphire (serialisee en JSON / PostgreSQL)
pub mod identity;

/// Boucle de vie principale, pipeline de stimuli, gestion memoire et shutdown
pub mod lifecycle;

// Re-export de la structure principale pour un acces direct via `crate::agent::SaphireAgent`
pub use lifecycle::SaphireAgent;
