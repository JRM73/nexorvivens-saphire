// episodic.rs — Mémoire épisodique (PostgreSQL, avec decay)
//
// Ce module gère la mémoire épisodique de Saphire, le deuxième niveau
// du système mnésique. La mémoire épisodique stocke les souvenirs récents
// sous forme d'épisodes contextualisés (contenu, émotion, satisfaction, etc.)
// dans la base de données PostgreSQL.
//
// Contrairement à la mémoire de travail (volatile en RAM), les souvenirs
// épisodiques sont persistés mais subissent une décroissance progressive
// de leur force. Les souvenirs suffisamment importants seront consolidés
// vers la mémoire à long terme (LTM = Long-Term Memory = Mémoire à Long Terme).
//
// Dépendances :
//   - serde : sérialisation / désérialisation des enregistrements.
//   - chrono : horodatage des souvenirs.
//   - serde_json : stockage des données structurées (stimulus, chimie émotionnelle).

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::neurochemistry::ChemicalSignature;

/// Structure représentant un nouvel épisode à insérer en base de données.
///
/// Utilisée comme DTO (Data Transfer Object = Objet de Transfert de Données)
/// entre le cycle cognitif et la couche de persistance.
pub struct EpisodicItem {
    /// Contenu textuel du souvenir (résumé de l'épisode vécu).
    pub content: String,
    /// Type de source ayant déclenché cet épisode (ex : "user_message",
    /// "thought", "web_knowledge").
    pub source_type: String,
    /// Données JSON du stimulus ayant déclenché le cycle cognitif
    /// (message, événement, etc.).
    pub stimulus_json: serde_json::Value,
    /// Identifiant numérique de la décision prise lors de ce cycle
    /// (action choisie par le module décisionnel).
    pub decision: i16,
    /// État de la chimie émotionnelle au moment de l'épisode, sérialisé en JSON.
    /// Contient les niveaux de neurotransmetteurs simulés (dopamine, sérotonine, etc.).
    pub chemistry_json: serde_json::Value,
    /// Émotion dominante ressentie pendant cet épisode (ex : "Joie", "Curiosité").
    pub emotion: String,
    /// Niveau de satisfaction résultant de l'épisode, entre 0.0 (insatisfait)
    /// et 1.0 (pleinement satisfait).
    pub satisfaction: f32,
    /// Intensité émotionnelle de l'épisode, entre 0.0 (neutre) et 1.0 (très intense).
    /// Facteur clé pour la consolidation : les souvenirs intenses sont mieux retenus.
    pub emotional_intensity: f32,
    /// Identifiant optionnel de la conversation durant laquelle cet épisode
    /// a eu lieu. Permet de regrouper les souvenirs par conversation.
    pub conversation_id: Option<String>,
    /// Signature chimique au moment de l'encodage
    pub chemical_signature: Option<ChemicalSignature>,
}

/// Enregistrement épisodique lu depuis la base de données.
///
/// Contient tous les champs d'un EpisodicItem plus les métadonnées ajoutées
/// par la DB (id, force résiduelle, compteur d'accès, statut de consolidation, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicRecord {
    /// Identifiant unique en base de données (clé primaire).
    pub id: i64,
    /// Contenu textuel du souvenir.
    pub content: String,
    /// Type de source ayant déclenché cet épisode.
    pub source_type: String,
    /// Données JSON du stimulus (optionnel si non disponible à la lecture).
    pub stimulus_json: Option<serde_json::Value>,
    /// Décision prise lors de ce cycle (optionnel si non disponible).
    pub decision: Option<i16>,
    /// État de la chimie émotionnelle en JSON (optionnel).
    pub chemistry_json: Option<serde_json::Value>,
    /// Émotion dominante de l'épisode.
    pub emotion: String,
    /// Niveau de satisfaction de l'épisode (0.0 à 1.0).
    pub satisfaction: f32,
    /// Intensité émotionnelle de l'épisode (0.0 à 1.0).
    pub emotional_intensity: f32,
    /// Force résiduelle du souvenir (0.0 à 1.0). Décroît à chaque cycle de
    /// consolidation. Quand elle tombe trop bas, le souvenir est élagué.
    pub strength: f32,
    /// Nombre de fois que ce souvenir a été rappelé (accédé lors d'un recall).
    /// Un souvenir souvent rappelé est considéré comme plus important.
    pub access_count: i32,
    /// Horodatage du dernier accès à ce souvenir (None si jamais rappelé).
    pub last_accessed_at: Option<DateTime<Utc>>,
    /// Indique si ce souvenir a déjà été consolidé en mémoire à long terme.
    /// Un souvenir consolidé n'est plus candidat à la consolidation.
    pub consolidated: bool,
    /// Identifiant optionnel de la conversation associée.
    pub conversation_id: Option<String>,
    /// Horodatage de la création de ce souvenir en base de données.
    pub created_at: DateTime<Utc>,
    /// Signature chimique au moment de l'encodage (None pour les anciens souvenirs)
    pub chemical_signature: Option<ChemicalSignature>,
}

/// Calcule le score de consolidation d'un souvenir épisodique.
///
/// Ce score détermine si un souvenir mérite d'être transféré de la mémoire
/// épisodique vers la mémoire à long terme (LTM). Il est basé sur un modèle
/// pondéré de 5 facteurs, inspiré de la psychologie cognitive :
///
/// | Facteur                 | Poids | Justification                            |
/// |-------------------------|-------|------------------------------------------|
/// | Intensité émotionnelle  | 0.35  | Les émotions fortes ancrent les souvenirs|
/// | Impact de satisfaction  | 0.20  | Réussites et échecs marquants comptent   |
/// | Fréquence de rappel     | 0.15  | Un souvenir souvent rappelé est important|
/// | Force résiduelle        | 0.15  | Un souvenir encore « fort » mérite d'être conservé |
/// | Interaction humaine     | 0.15  | Les échanges avec un humain sont privilégiés |
///
/// Le score final est module par le niveau de BDNF :
/// - BDNF = 0.0 → multiplicateur 0.8 (consolidation affaiblie)
/// - BDNF = 0.5 → multiplicateur 1.0 (consolidation normale)
/// - BDNF = 1.0 → multiplicateur 1.2 (consolidation renforcee)
///
/// # Paramètres
/// - `record` : enregistrement épisodique à évaluer.
/// - `bdnf_level` : niveau courant de BDNF (0.0 - 1.0).
///
/// # Retour
/// Score entre 0.0 et 1.0. Comparé au seuil `consolidation_threshold`
/// de la configuration pour décider du transfert vers la LTM.
pub fn consolidation_score(record: &EpisodicRecord, bdnf_level: f64) -> f64 {
    let mut score = 0.0;

    // Facteur 1 : L'intensité émotionnelle est le facteur principal (poids 0.35).
    // Les souvenirs fortement émotionnels sont mieux retenus, comme chez l'humain.
    score += record.emotional_intensity as f64 * 0.35;

    // Facteur 2 : Impact de la satisfaction (poids 0.20).
    // On mesure l'écart par rapport au neutre (0.5) : les extrêmes (très
    // satisfaisant ou très insatisfaisant) sont plus mémorables que le neutre.
    let satisfaction_impact = (record.satisfaction as f64 - 0.5).abs() * 2.0;
    score += satisfaction_impact * 0.20;

    // Facteur 3 : Fréquence de rappel (poids 0.15).
    // Un souvenir rappelé souvent est manifestement important pour Saphire.
    // On plafonne à 10 accès pour normaliser entre 0 et 1.
    let access_factor = (record.access_count as f64).min(10.0) / 10.0;
    score += access_factor * 0.15;

    // Facteur 4 : Force résiduelle du souvenir (poids 0.15).
    // Un souvenir qui a bien résisté à la décroissance est un bon candidat.
    score += record.strength as f64 * 0.15;

    // Facteur 5 : Bonus d'interaction humaine (poids 0.15).
    // Les souvenirs liés à des échanges avec un humain sont considérés
    // comme plus significatifs et reçoivent un bonus fixe.
    let is_human = record.source_type == "user_message"
                 || record.source_type == "conversation";
    if is_human {
        score += 0.15;
    }

    // Modulation BDNF : le facteur neurotrophique module la consolidation.
    // BDNF bas (0.0) → 0.8x, BDNF normal (0.5) → 1.0x, BDNF haut (1.0) → 1.2x
    let bdnf_mod = 0.8 + bdnf_level * 0.4;
    score *= bdnf_mod;

    // Clampage final pour garantir un score dans l'intervalle [0.0, 1.0].
    score.clamp(0.0, 1.0)
}
