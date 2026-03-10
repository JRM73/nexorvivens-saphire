// memory/mod.rs — Système de mémoire à 3 niveaux
//
// Ce module est le point d'entrée du sous-système de mémoire de Saphire.
// Il orchestre trois niveaux inspirés de la psychologie cognitive humaine :
//   1. Mémoire de travail (working) : tampon volatile en RAM, capacité limitée.
//   2. Mémoire épisodique (episodic) : souvenirs récents stockés en PostgreSQL
//      avec décroissance progressive.
//   3. Mémoire à long terme (long_term) : souvenirs consolidés permanents,
//      indexés par vecteurs (pgvector) pour la recherche sémantique.
//
// Dépendances principales :
//   - serde : sérialisation / désérialisation de la configuration.
//   - crate::db : couche d'accès à la base de données PostgreSQL.
//
// Ce fichier expose la structure de configuration (MemoryConfig), les types
// réexportés des sous-modules et la fonction utilitaire build_memory_context()
// qui assemble le contexte mémoire injecté dans le prompt LLM
// (Large Language Model = Grand Modèle de Langage).

pub mod working;
pub mod episodic;
pub mod long_term;
pub mod consolidation;
pub mod recall;
pub mod reconsolidation;

pub use working::{WorkingMemory, WorkingItem, WorkingItemSource};
pub use episodic::{EpisodicItem, EpisodicRecord};
pub use consolidation::{ConsolidationReport, ConsolidationParams};
pub use recall::MemoryLevel;

use serde::{Deserialize, Serialize};

/// Configuration complète du système de mémoire.
///
/// Chaque champ possède une valeur par défaut raisonnable via les fonctions
/// `default_*`, ce qui permet de ne spécifier que les valeurs à personnaliser
/// dans le fichier de configuration JSON/TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Capacité maximale de la mémoire de travail (nombre d'items).
    /// Vaut 7 par défaut, d'après la loi de Miller (7 ± 2 éléments).
    #[serde(default = "default_working_capacity")]
    pub working_capacity: usize,

    /// Taux de décroissance de la pertinence en mémoire de travail par cycle.
    /// À chaque cycle cognitif, la pertinence de chaque item diminue de cette valeur.
    #[serde(default = "default_working_decay")]
    pub working_decay_rate: f64,

    /// Nombre maximal de souvenirs épisodiques en base de données.
    /// Au-delà, un élagage (pruning) est déclenché.
    #[serde(default = "default_episodic_max")]
    pub episodic_max: usize,

    /// Taux de décroissance de la force des souvenirs épisodiques par cycle
    /// de consolidation. Les souvenirs non consolidés perdent progressivement
    /// de la force jusqu'à être élagués.
    #[serde(default = "default_episodic_decay")]
    pub episodic_decay_rate: f64,

    /// Nombre cible de souvenirs épisodiques après élagage.
    /// Quand episodic_max est dépassé, on élague jusqu'à atteindre cette valeur.
    #[serde(default = "default_episodic_prune")]
    pub episodic_prune_target: usize,

    /// Intervalle (en cycles cognitifs) entre deux processus de consolidation.
    /// Tous les N cycles, les souvenirs épisodiques sont évalués pour
    /// transfert vers la mémoire à long terme.
    #[serde(default = "default_consol_interval")]
    pub consolidation_interval_cycles: u64,

    /// Score minimum de consolidation pour qu'un souvenir épisodique soit
    /// transféré vers la mémoire à long terme (LTM = Long-Term Memory).
    /// Valeur entre 0.0 et 1.0.
    #[serde(default = "default_consol_threshold")]
    pub consolidation_threshold: f64,

    /// Si vrai, la consolidation est aussi déclenchée lors du « sommeil »
    /// (période d'inactivité de Saphire), imitant le rôle du sommeil
    /// dans la consolidation mnésique chez l'humain.
    #[serde(default = "default_consol_sleep")]
    pub consolidation_on_sleep: bool,

    /// Nombre maximal de souvenirs en mémoire à long terme.
    #[serde(default = "default_ltm_max")]
    pub ltm_max: usize,

    /// Seuil de similarité cosinus pour considérer un souvenir LTM comme
    /// pertinent lors d'un rappel. Valeur entre 0.0 et 1.0.
    #[serde(default = "default_ltm_threshold")]
    pub ltm_similarity_threshold: f64,

    /// Nombre cible de souvenirs LTM apres elagage.
    /// Quand ltm_max est depasse, on elague jusqu'a atteindre cette valeur.
    #[serde(default = "default_ltm_prune_target")]
    pub ltm_prune_target: usize,

    /// Nombre minimum d'acces pour qu'un souvenir LTM soit protege de l'elagage.
    #[serde(default = "default_ltm_protection_access_count")]
    pub ltm_protection_access_count: i32,

    /// Poids emotionnel minimum pour qu'un souvenir LTM soit protege de l'elagage.
    #[serde(default = "default_ltm_protection_emotional_weight")]
    pub ltm_protection_emotional_weight: f32,

    /// Taille des lots lors de l'archivage des souvenirs LTM elagués.
    #[serde(default = "default_archive_batch_size")]
    pub archive_batch_size: usize,

    /// Nombre de souvenirs episodiques rappeles pour le contexte.
    #[serde(default = "default_recall_episodic_limit")]
    pub recall_episodic_limit: usize,

    /// Nombre de souvenirs LTM rappeles par similarite.
    #[serde(default = "default_recall_ltm_limit")]
    pub recall_ltm_limit: usize,

    /// Seuil de similarite minimum pour le rappel LTM.
    #[serde(default = "default_recall_ltm_threshold")]
    pub recall_ltm_threshold: f64,

    /// Nombre d'archives profondes rappelees.
    #[serde(default = "default_recall_archive_limit")]
    pub recall_archive_limit: usize,

    /// Seuil de similarite minimum pour le rappel d'archives.
    #[serde(default = "default_recall_archive_threshold")]
    pub recall_archive_threshold: f64,

    /// Nombre de souvenirs subconscients (vecteurs) rappeles par similarite.
    #[serde(default = "default_recall_vectors_limit")]
    pub recall_vectors_limit: usize,

    /// Seuil de similarite minimum pour le rappel de vecteurs subconscients.
    #[serde(default = "default_recall_vectors_threshold")]
    pub recall_vectors_threshold: f64,
}

// --- Fonctions de valeurs par défaut pour la désérialisation serde ---
// Chaque fonction retourne la valeur par défaut d'un champ de MemoryConfig.

fn default_working_capacity() -> usize { 7 }
fn default_working_decay() -> f64 { 0.05 }
fn default_episodic_max() -> usize { 500 }
fn default_episodic_decay() -> f64 { 0.02 }
fn default_episodic_prune() -> usize { 400 }
fn default_consol_interval() -> u64 { 50 }
fn default_consol_threshold() -> f64 { 0.6 }
fn default_consol_sleep() -> bool { true }
fn default_ltm_max() -> usize { 200000 }
fn default_ltm_threshold() -> f64 { 0.7 }
fn default_ltm_prune_target() -> usize { 190000 }
fn default_ltm_protection_access_count() -> i32 { 5 }
fn default_ltm_protection_emotional_weight() -> f32 { 0.7 }
fn default_archive_batch_size() -> usize { 50 }
fn default_recall_episodic_limit() -> usize { 5 }
fn default_recall_ltm_limit() -> usize { 5 }
fn default_recall_ltm_threshold() -> f64 { 0.25 }
fn default_recall_archive_limit() -> usize { 3 }
fn default_recall_archive_threshold() -> f64 { 0.25 }
fn default_recall_vectors_limit() -> usize { 3 }
fn default_recall_vectors_threshold() -> f64 { 0.30 }

impl Default for MemoryConfig {
    /// Retourne une configuration mémoire avec toutes les valeurs par défaut.
    fn default() -> Self {
        Self {
            working_capacity: 7,
            working_decay_rate: 0.05,
            episodic_max: 500,
            episodic_decay_rate: 0.02,
            episodic_prune_target: 400,
            consolidation_interval_cycles: 50,
            consolidation_threshold: 0.6,
            consolidation_on_sleep: true,
            ltm_max: 200000,
            ltm_similarity_threshold: 0.7,
            ltm_prune_target: 190000,
            ltm_protection_access_count: 5,
            ltm_protection_emotional_weight: 0.7,
            archive_batch_size: 50,
            recall_episodic_limit: 5,
            recall_ltm_limit: 5,
            recall_ltm_threshold: 0.25,
            recall_archive_limit: 3,
            recall_archive_threshold: 0.25,
            recall_vectors_limit: 3,
            recall_vectors_threshold: 0.30,
        }
    }
}

/// Construit le contexte mémoire complet destiné à être injecté dans le
/// prompt envoyé au LLM (Large Language Model = Grand Modèle de Langage).
///
/// Cette fonction fusionne trois sources de souvenirs en un seul bloc de
/// texte structuré, pour que le LLM ait conscience du contexte passé.
///
/// # Paramètres
/// - `wm_summary` : résumé textuel de la mémoire de travail (items actifs).
/// - `episodic_recent` : souvenirs épisodiques récents récupérés depuis la DB.
/// - `ltm_similar` : souvenirs de la mémoire à long terme trouvés par
///   similarité vectorielle (cosinus) avec la requête courante.
///
/// # Retour
/// Une chaîne de caractères formatée contenant les trois sections de contexte.
pub fn build_memory_context(
    wm_summary: &str,
    episodic_recent: &[EpisodicRecord],
    ltm_similar: &[crate::db::MemoryRecord],
    archive_similar: &[crate::db::archives::ArchiveRecord],
    subconscious_vectors: &[crate::db::vectors::SubconsciousVectorRecord],
) -> String {
    let mut ctx = String::new();

    // Section 1 : Memoire de travail (contexte immediat)
    if !wm_summary.is_empty() {
        ctx.push_str(wm_summary);
        ctx.push('\n');
    }

    // Section 2 : Souvenirs episodiques recents
    if !episodic_recent.is_empty() {
        ctx.push_str("SOUVENIRS RECENTS :\n");
        for ep in episodic_recent {
            let preview: String = ep.content.chars().take(200).collect();
            ctx.push_str(&format!("  - {} ({})\n", preview, ep.emotion));
        }
        ctx.push('\n');
    }

    // Section 3 : Souvenirs de la memoire a long terme pertinents
    if !ltm_similar.is_empty() {
        ctx.push_str("MEMOIRE PROFONDE :\n");
        for mem in ltm_similar {
            let preview: String = mem.text_summary.chars().take(200).collect();
            ctx.push_str(&format!("  - {} (similarite: {:.0}%)\n",
                preview,
                mem.similarity * 100.0
            ));
        }
        ctx.push('\n');
    }

    // Section 4 : Archives profondes (souvenirs LTM elagués puis compresses)
    if !archive_similar.is_empty() {
        ctx.push_str("ARCHIVES PROFONDES :\n");
        for arc in archive_similar {
            let preview: String = arc.summary.chars().take(150).collect();
            ctx.push_str(&format!("  - {} ({} souvenirs, {})\n",
                preview,
                arc.source_count,
                arc.emotions.join("/"),
            ));
        }
        ctx.push('\n');
    }

    // Section 5 : Souvenirs subconscients (reves, insights, connexions, eureka, images mentales)
    if !subconscious_vectors.is_empty() {
        ctx.push_str("SOUVENIRS SUBCONSCIENTS :\n");
        for sv in subconscious_vectors {
            let label = match sv.source_type.as_str() {
                "dream" => "reve",
                "mental_imagery" => "image mentale",
                "subconscious_insight" => "insight",
                "neural_connection" => "connexion",
                "eureka" => "eureka",
                other => other,
            };
            let preview: String = sv.text_content.chars().take(150).collect();
            ctx.push_str(&format!("  - [{}] {} (similarite: {:.0}%)\n",
                label, preview, sv.similarity * 100.0,
            ));
        }
    }

    ctx
}

/// Construit le contexte des apprentissages passes pour injection dans le prompt LLM.
///
/// Chaque apprentissage est affiche avec son domaine, son resume et sa confiance.
/// Ce contexte permet au LLM de s'appuyer sur ses apprentissages anterieurs.
pub fn build_learning_context(
    learnings: &[crate::db::learnings::NnLearningRecord],
) -> String {
    if learnings.is_empty() {
        return String::new();
    }
    let mut ctx = String::from("APPRENTISSAGES PASSES :\n");
    for l in learnings {
        ctx.push_str(&format!(
            "  - [{}] {} (confiance: {:.0}%)\n",
            l.domain,
            l.summary,
            l.confidence * 100.0,
        ));
    }
    ctx
}
