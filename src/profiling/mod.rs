// =============================================================================
// profiling/mod.rs — Profilage psychologique dynamique base sur le modele
//                    Big Five OCEAN (Openness / Conscientiousness / Extraversion /
//                    Agreeableness / Neuroticism)
//
// Role : Point d'entree du module de profilage. Expose les sous-modules et les
//        structures de configuration pour le systeme de profilage psychologique
//        bidirectionnel de Saphire :
//          - Auto-profilage (self_profiler) : Saphire observe ses propres cycles
//            cognitifs pour construire son profil OCEAN
//          - Profilage humain (human_profiler) : Saphire analyse les messages de
//            l'interlocuteur pour estimer son profil OCEAN
//          - Adaptation (adaptation) : genere des instructions de style basees sur
//            le profil de l'humain pour adapter les reponses
//          - Narratif (narrative) : genere une description textuelle du profil OCEAN
//
// Dependances :
//   - serde : serialisation/deserialisation de la configuration
//   - Sous-modules : ocean, self_profiler, human_profiler, adaptation, narrative
//
// Place dans l'architecture :
//   Le profilage est un composant transversal du systeme cognitif de Saphire.
//   Il est alimente par les resultats NLP (pour le profilage humain) et par les
//   observations des cycles cognitifs (pour l'auto-profilage). Les profils produits
//   influencent la generation de reponses via le module d'adaptation.
// =============================================================================

pub mod ocean;
pub mod self_profiler;
pub mod human_profiler;
pub mod adaptation;
pub mod narrative;

use serde::{Deserialize, Serialize};

pub use ocean::OceanProfile;
pub use self_profiler::{SelfProfiler, BehaviorObservation};
pub use human_profiler::{HumanProfiler, HumanProfile, CommunicationStyle};

/// Configuration du systeme de profilage psychologique.
///
/// Controle les parametres de fonctionnement du profilage pour l'auto-profil
/// de Saphire et le profil des interlocuteurs humains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingConfig {
    /// Active ou desactive l'ensemble du systeme de profilage
    pub enabled: bool,
    /// Active l'auto-profilage (Saphire observe ses propres comportements)
    pub self_profiling: bool,
    /// Active le profilage des interlocuteurs humains
    pub human_profiling: bool,
    /// Nombre de cycles cognitifs entre chaque recalcul du profil OCEAN.
    /// Plus la valeur est basse, plus le profil est reactif aux changements.
    pub recompute_interval_cycles: u64,
    /// Taille maximale du tampon d'observations comportementales.
    /// Quand le tampon est plein, un recalcul est automatiquement declenche.
    pub observation_buffer_size: usize,
    /// Taux de melange entre l'ancien et le nouveau profil lors du recalcul.
    /// 0.3 signifie 30% nouveau + 70% ancien, ce qui lisse les fluctuations.
    pub profile_blend_rate: f64,
    /// Nombre maximal d'instantanes historiques du profil a conserver.
    /// Permet de suivre l'evolution du profil dans le temps.
    pub history_snapshots: usize,
}

impl Default for ProfilingConfig {
    /// Configuration par defaut du profilage.
    ///
    /// Retour : une configuration avec le profilage actif, un recalcul tous les
    ///          50 cycles, un tampon de 100 observations, un taux de melange de 30%
    ///          et 30 instantanes historiques.
    fn default() -> Self {
        Self {
            enabled: true,
            self_profiling: true,
            human_profiling: true,
            recompute_interval_cycles: 50,
            observation_buffer_size: 100,
            profile_blend_rate: 0.3,
            history_snapshots: 30,
        }
    }
}
