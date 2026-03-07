// =============================================================================
// knowledge/ — Minimal stub for the lite edition
// =============================================================================
//
// Purpose: Only the KnowledgeResult and KnowledgeError structures are
//          defined here, used by agent/lifecycle/thinking.rs for the
//          knowledge context. The full web knowledge system (13 sources)
//          is not ported in the lite edition.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Result of a knowledge search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeResult {
    /// Source name (e.g., "Wikipedia", "ArXiv")
    pub source: String,
    /// Title of the retrieved article or resource
    pub title: String,
    /// URL of the source
    pub url: String,
    /// Extracted text excerpt relevant to the query
    pub extract: String,
    /// Section titles found in the document
    pub section_titles: Vec<String>,
    /// Total length of the source document in characters
    pub total_length: usize,
    /// Relevance score [0.0, 1.0] indicating match quality
    pub relevance_score: f64,
    /// UTC timestamp of when the data was fetched
    pub fetched_at: DateTime<Utc>,
}

/// Errors from the WebKnowledge module.
#[derive(Debug)]
pub enum KnowledgeError {
    /// The requested knowledge was not found
    NotFound,
    /// The source API rate-limited the request
    RateLimited,
    /// The target domain is blocked by policy
    DomainBlocked,
    /// Network error (timeout, connection refused, etc.)
    Network(String),
    /// Response parsing error
    Parse(String),
    /// Content flagged as dangerous by safety checks
    ContentDangerous,
}
