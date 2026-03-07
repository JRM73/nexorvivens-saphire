// =============================================================================
// knowledge/ — Stub minimal pour la version lite
//
// Seules les structures KnowledgeResult et KnowledgeError sont definies ici,
// utilisees par agent/lifecycle/thinking.rs pour le contexte de connaissance.
// Le systeme de connaissance web complet (13 sources) n'est pas porte.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Resultat d'une recherche de connaissances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeResult {
    pub source: String,
    pub title: String,
    pub url: String,
    pub extract: String,
    pub section_titles: Vec<String>,
    pub total_length: usize,
    pub relevance_score: f64,
    pub fetched_at: DateTime<Utc>,
}

/// Erreurs du module WebKnowledge.
#[derive(Debug)]
pub enum KnowledgeError {
    NotFound,
    RateLimited,
    DomainBlocked,
    Network(String),
    Parse(String),
    ContentDangerous,
}
