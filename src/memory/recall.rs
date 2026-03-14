// recall.rs — Types pour la recherche unifiée dans les 3 niveaux de mémoire
//
// Ce module définit les types utilisés lors du rappel (recall) de souvenirs
// à travers les différents niveaux du système mnésique de Saphire.
//
// Lors d'un rappel, le système peut retrouver des souvenirs provenant de
// n'importe quel niveau :
//   - Working   : mémoire de travail (items actuellement actifs en RAM).
//   - Episodic  : mémoire épisodique (souvenirs récents en PostgreSQL).
//   - LongTerm  : mémoire à long terme (souvenirs consolidés permanents).
//   - Founding  : souvenirs fondateurs (identité de base de Saphire).
//
// Le type MemoryLevel permet d'identifier la provenance d'un souvenir
// retrouvé, ce qui est utile pour l'affichage, le débogage et la
// pondération des résultats de rappel.
//
// Dépendances :
//   - serde : sérialisation pour l'envoi via WebSocket et les API.

use serde::Serialize;
use crate::neurochemistry::{ChemicalSignature, NeuroChemicalState};
use crate::db::MemoryRecord;

/// Niveau de mémoire d'où provient un souvenir retrouvé lors d'un rappel.
///
/// Utilisé pour identifier l'origine d'un souvenir dans les résultats
/// de recherche unifiée à travers les 3 niveaux mnésiques.
#[derive(Debug, Clone, Serialize)]
pub enum MemoryLevel {
    /// Mémoire de travail : item actuellement actif dans le tampon de conscience.
    Working,
    /// Mémoire épisodique : souvenir récent persisté en base de données.
    Episodic,
    /// Mémoire à long terme : souvenir consolidé, permanent et indexé
    /// par vecteur pour la recherche sémantique.
    LongTerm,
    /// Souvenir fondateur : memoire initiale programmee qui definit
    /// l'identite et les valeurs de base de Saphire.
    Founding,
    /// Archive : lot de souvenirs LTM elagués puis compresses en resume.
    /// Les archives restent accessibles par recherche vectorielle.
    Archive,
}

impl MemoryLevel {
    /// Retourne un libelle textuel identifiant le niveau de memoire.
    pub fn label(&self) -> &str {
        match self {
            MemoryLevel::Working => "working",
            MemoryLevel::Episodic => "episodic",
            MemoryLevel::LongTerm => "long_term",
            MemoryLevel::Founding => "founding",
            MemoryLevel::Archive => "archive",
        }
    }
}

/// Re-classe des souvenirs LTM par combinaison de similarite textuelle
/// et de similarite chimique (state-dependent memory).
///
/// Un etat chimique similaire a celui de l'encodage facilite le rappel,
/// comme chez l'humain (memoire dependante de l'etat).
///
/// # Parametres
/// - `candidates` : souvenirs deja tries par similarite textuelle
/// - `current_chemistry` : etat chimique courant de Saphire
/// - `text_weight` : poids de la similarite textuelle (defaut 0.8)
/// - `chem_weight` : poids de la similarite chimique (defaut 0.2)
pub fn recall_with_chemical_context(
    candidates: &mut [MemoryRecord],
    current_chemistry: &NeuroChemicalState,
    text_weight: f64,
    chem_weight: f64,
) {
    let current_sig = ChemicalSignature::from(current_chemistry);
    let now = chrono::Utc::now();

    // Recalculer le score comme mix texte + chimie + recence
    for mem in candidates.iter_mut() {
        let chem_sim = mem.chemical_signature
            .as_ref()
            .map(|sig| sig.similarity(&current_sig))
            .unwrap_or(0.5);

        // Ponderation temporelle : bonus pour les souvenirs recents
        // Decroissance exponentielle : recency = e^(-age_days / 30)
        // Un souvenir de 0 jours = 1.0, 30 jours = 0.37, 90 jours = 0.05
        let age_days = (now - mem.created_at).num_hours() as f64 / 24.0;
        let recency = (-age_days / 30.0).exp();

        // Score final : 70% texte + 15% chimie + 15% recence
        // (ajuste les poids pour integrer la recence sans casser le ratio)
        let recency_weight = 0.15;
        let adjusted_text = text_weight * (1.0 - recency_weight);
        let adjusted_chem = chem_weight * (1.0 - recency_weight);
        mem.similarity = mem.similarity * adjusted_text
            + chem_sim * adjusted_chem
            + recency * recency_weight;
    }
    // Re-trier par score final decroissant
    candidates.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
}
