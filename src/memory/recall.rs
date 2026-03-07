// recall.rs — Types for unified retrieval across the 3 memory tiers
//
// This module defines the types used during memory recall (retrieval) across
// the different tiers of the Saphire memory system.
//
// During a recall operation, the system can retrieve memories from any tier:
//   - Working:  working memory (items currently active in the RAM buffer).
//   - Episodic: episodic memory (recent experiences stored in PostgreSQL).
//   - LongTerm: long-term memory (permanently consolidated memories with
//               vector indexing for semantic retrieval).
//   - Founding: founding memories (pre-programmed identity-defining memories).
//   - Archive:  archived batches of pruned LTM memories, still accessible
//               via vector similarity search.
//
// The MemoryLevel enum allows identification of a retrieved memory's origin,
// which is useful for display, debugging, and differential weighting of
// recall results.
//
// Dependencies:
//   - serde: serialization for WebSocket transmission and API responses.

use serde::Serialize;
use crate::neurochemistry::{ChemicalSignature, NeuroChemicalState};
use crate::db::MemoryRecord;

/// Memory tier from which a recalled memory originates.
///
/// Used to tag retrieval results with their provenance across the unified
/// multi-tier memory search.
#[derive(Debug, Clone, Serialize)]
pub enum MemoryLevel {
    /// Working memory: item currently active in the limited-capacity
    /// consciousness buffer (analogous to the phonological loop).
    Working,
    /// Episodic memory: recent experience persisted in the database,
    /// subject to strength decay (analogous to hippocampal traces).
    Episodic,
    /// Long-term memory: permanently consolidated memory indexed by
    /// vector embedding for semantic cosine-similarity retrieval
    /// (analogous to neocortical storage).
    LongTerm,
    /// Founding memory: pre-programmed initial memory that defines
    /// Saphire's core identity and values. These are never decayed
    /// or pruned.
    Founding,
    /// Archive: a compressed batch of pruned LTM memories. Archives
    /// remain accessible via vector similarity search but at reduced
    /// granularity compared to individual LTM entries.
    Archive,
}

impl MemoryLevel {
    /// Returns a textual label identifying the memory tier.
    ///
    /// # Returns
    /// A static string naming the tier (e.g., "working", "episodic", "long_term").
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

/// Re-ranks LTM memories by combining textual similarity with neurochemical
/// similarity (state-dependent memory).
///
/// A neurochemical state similar to the one present at encoding time
/// facilitates recall, implementing the state-dependent memory effect
/// observed in humans (Godden & Baddeley, 1975; Eich, 1980). Memories
/// encoded under a particular neurochemical profile (e.g., high dopamine)
/// are more easily retrieved when the current state matches.
///
/// The final similarity score is a weighted combination:
///   `final_score = text_similarity * text_weight + chemical_similarity * chem_weight`
///
/// # Parameters
/// - `candidates`: LTM memories already sorted by textual cosine similarity.
///   Their `similarity` field is overwritten with the blended score.
/// - `current_chemistry`: Saphire's current neurochemical state.
/// - `text_weight`: weight assigned to textual similarity (typically 0.8).
/// - `chem_weight`: weight assigned to chemical similarity (typically 0.2).
pub fn recall_with_chemical_context(
    candidates: &mut [MemoryRecord],
    current_chemistry: &NeuroChemicalState,
    text_weight: f64,
    chem_weight: f64,
) {
    let current_sig = ChemicalSignature::from(current_chemistry);
    // Recompute the similarity score as a weighted blend of text and chemistry.
    for mem in candidates.iter_mut() {
        let chem_sim = mem.chemical_signature
            .as_ref()
            .map(|sig| sig.similarity(&current_sig))
            // Default to 0.5 (neutral) for legacy memories without a chemical signature.
            .unwrap_or(0.5);
        // Overwrite the similarity field with the blended score.
        mem.similarity = mem.similarity * text_weight + chem_sim * chem_weight;
    }
    // Re-sort by the blended score in descending order.
    candidates.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
}
