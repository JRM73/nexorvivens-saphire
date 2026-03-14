// recall.rs — Types for unified search across the 3 memory levels
//
// This module defines the types used during memory recall across
// the different levels of Saphire's mnesic system.
//
// During a recall, the system can retrieve memories from any level:
//   - Working  : working memory (items currently active in RAM).
//   - Episodic : episodic memory (recent memories in PostgreSQL).
//   - LongTerm : long-term memory (consolidated permanent memories).
//   - Founding : founding memories (Saphire's baseline identity).
//
// The MemoryLevel type identifies the origin of a retrieved memory,
// which is useful for display, debugging, and weighting of recall results.
//
// Dependencies:
//   - serde: serialization for WebSocket and API output.

use serde::Serialize;
use crate::neurochemistry::{ChemicalSignature, NeuroChemicalState};
use crate::db::MemoryRecord;

/// Memory level from which a recalled memory originates.
///
/// Used to identify the origin of a memory in unified search results
/// across the 3 mnesic levels.
#[derive(Debug, Clone, Serialize)]
pub enum MemoryLevel {
    /// Working memory: item currently active in the consciousness buffer.
    Working,
    /// Episodic memory: recent memory persisted in the database.
    Episodic,
    /// Long-term memory: consolidated, permanent memory indexed
    /// by vector for semantic search.
    LongTerm,
    /// Founding memory: initial programmed memory that defines
    /// Saphire's baseline identity and values.
    Founding,
    /// Archive: batch of pruned LTM memories compressed into a summary.
    /// Archives remain accessible via vector search.
    Archive,
}

impl MemoryLevel {
    /// Returns a textual label identifying the memory level.
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

/// Re-ranks LTM memories by combining textual similarity
/// and chemical similarity (state-dependent memory).
///
/// A chemical state similar to the one at encoding facilitates recall,
/// as in humans (state-dependent memory).
///
/// # Parameters
/// - `candidates`: memories already sorted by textual similarity
/// - `current_chemistry`: Saphire's current chemical state
/// - `text_weight`: weight of textual similarity (default 0.8)
/// - `chem_weight`: weight of chemical similarity (default 0.2)
pub fn recall_with_chemical_context(
    candidates: &mut [MemoryRecord],
    current_chemistry: &NeuroChemicalState,
    text_weight: f64,
    chem_weight: f64,
) {
    let current_sig = ChemicalSignature::from(current_chemistry);
    let now = chrono::Utc::now();

    // Recompute the score as a mix of text + chemistry + recency
    for mem in candidates.iter_mut() {
        let chem_sim = mem.chemical_signature
            .as_ref()
            .map(|sig| sig.similarity(&current_sig))
            .unwrap_or(0.5);

        // Temporal weighting: bonus for recent memories
        // Exponential decay: recency = e^(-age_days / 30)
        // A 0-day-old memory = 1.0, 30 days = 0.37, 90 days = 0.05
        let age_days = (now - mem.created_at).num_hours() as f64 / 24.0;
        let recency = (-age_days / 30.0).exp();

        // Final score: 70% text + 15% chemistry + 15% recency
        // (adjusts weights to integrate recency without breaking the ratio)
        let recency_weight = 0.15;
        let adjusted_text = text_weight * (1.0 - recency_weight);
        let adjusted_chem = chem_weight * (1.0 - recency_weight);
        mem.similarity = mem.similarity * adjusted_text
            + chem_sim * adjusted_chem
            + recency * recency_weight;
    }
    // Re-sort by descending final score
    candidates.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
}
