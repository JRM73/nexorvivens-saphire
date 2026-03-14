// memory/mod.rs — 3-level memory system
//
// This module is the entry point for Saphire's memory subsystem.
// It orchestrates three levels inspired by human cognitive psychology:
//   1. Working memory (working): volatile RAM buffer, limited capacity.
//   2. Episodic memory (episodic): recent memories stored in PostgreSQL
//      with progressive decay.
//   3. Long-term memory (long_term): consolidated permanent memories,
//      indexed by vectors (pgvector) for semantic search.
//
// Main dependencies:
//   - serde: serialization / deserialization of configuration.
//   - crate::db: PostgreSQL database access layer.
//
// This file exposes the configuration structure (MemoryConfig), re-exported
// types from submodules, and the utility function build_memory_context()
// which assembles the memory context injected into the LLM prompt.

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

/// Complete configuration for the memory system.
///
/// Each field has a reasonable default value via the `default_*` functions,
/// allowing only the values to customize to be specified in the JSON/TOML
/// configuration file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Maximum capacity of working memory (number of items).
    /// Defaults to 7, based on Miller's law (7 +/- 2 elements).
    #[serde(default = "default_working_capacity")]
    pub working_capacity: usize,

    /// Relevance decay rate in working memory per cycle.
    /// At each cognitive cycle, the relevance of each item decreases by this value.
    #[serde(default = "default_working_decay")]
    pub working_decay_rate: f64,

    /// Maximum number of episodic memories in the database.
    /// Beyond this, pruning is triggered.
    #[serde(default = "default_episodic_max")]
    pub episodic_max: usize,

    /// Strength decay rate for episodic memories per consolidation cycle.
    /// Unconsolidated memories progressively lose strength until pruned.
    #[serde(default = "default_episodic_decay")]
    pub episodic_decay_rate: f64,

    /// Target number of episodic memories after pruning.
    /// When episodic_max is exceeded, prune down to this value.
    #[serde(default = "default_episodic_prune")]
    pub episodic_prune_target: usize,

    /// Interval (in cognitive cycles) between two consolidation processes.
    /// Every N cycles, episodic memories are evaluated for transfer
    /// to long-term memory.
    #[serde(default = "default_consol_interval")]
    pub consolidation_interval_cycles: u64,

    /// Minimum consolidation score for an episodic memory to be
    /// transferred to long-term memory (LTM).
    /// Value between 0.0 and 1.0.
    #[serde(default = "default_consol_threshold")]
    pub consolidation_threshold: f64,

    /// If true, consolidation is also triggered during "sleep"
    /// (Saphire's inactivity period), mimicking the role of sleep
    /// in human mnesic consolidation.
    #[serde(default = "default_consol_sleep")]
    pub consolidation_on_sleep: bool,

    /// Maximum number of long-term memories.
    #[serde(default = "default_ltm_max")]
    pub ltm_max: usize,

    /// Cosine similarity threshold to consider an LTM memory as
    /// relevant during recall. Value between 0.0 and 1.0.
    #[serde(default = "default_ltm_threshold")]
    pub ltm_similarity_threshold: f64,

    /// Target number of LTM memories after pruning.
    /// When ltm_max is exceeded, prune down to this value.
    #[serde(default = "default_ltm_prune_target")]
    pub ltm_prune_target: usize,

    /// Minimum access count for an LTM memory to be protected from pruning.
    #[serde(default = "default_ltm_protection_access_count")]
    pub ltm_protection_access_count: i32,

    /// Minimum emotional weight for an LTM memory to be protected from pruning.
    #[serde(default = "default_ltm_protection_emotional_weight")]
    pub ltm_protection_emotional_weight: f32,

    /// Batch size for archiving pruned LTM memories.
    #[serde(default = "default_archive_batch_size")]
    pub archive_batch_size: usize,

    /// Number of episodic memories recalled for context.
    #[serde(default = "default_recall_episodic_limit")]
    pub recall_episodic_limit: usize,

    /// Number of LTM memories recalled by similarity.
    #[serde(default = "default_recall_ltm_limit")]
    pub recall_ltm_limit: usize,

    /// Minimum similarity threshold for LTM recall.
    #[serde(default = "default_recall_ltm_threshold")]
    pub recall_ltm_threshold: f64,

    /// Number of deep archives recalled.
    #[serde(default = "default_recall_archive_limit")]
    pub recall_archive_limit: usize,

    /// Minimum similarity threshold for archive recall.
    #[serde(default = "default_recall_archive_threshold")]
    pub recall_archive_threshold: f64,

    /// Number of subconscious memories (vectors) recalled by similarity.
    #[serde(default = "default_recall_vectors_limit")]
    pub recall_vectors_limit: usize,

    /// Minimum similarity threshold for subconscious vector recall.
    #[serde(default = "default_recall_vectors_threshold")]
    pub recall_vectors_threshold: f64,
}

// --- Default value functions for serde deserialization ---
// Each function returns the default value of a MemoryConfig field.

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
    /// Returns a memory configuration with all default values.
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

/// Builds the complete memory context intended to be injected into the
/// prompt sent to the LLM.
///
/// This function merges three sources of memories into a single structured
/// text block, so that the LLM is aware of past context.
///
/// # Parameters
/// - `wm_summary`: textual summary of working memory (active items).
/// - `episodic_recent`: recent episodic memories retrieved from the DB.
/// - `ltm_similar`: long-term memories found by vector similarity (cosine)
///   with the current query.
///
/// # Returns
/// A formatted string containing the three context sections.
pub fn build_memory_context(
    wm_summary: &str,
    episodic_recent: &[EpisodicRecord],
    ltm_similar: &[crate::db::MemoryRecord],
    archive_similar: &[crate::db::archives::ArchiveRecord],
    subconscious_vectors: &[crate::db::vectors::SubconsciousVectorRecord],
) -> String {
    let mut ctx = String::new();

    // Section 1: Working memory (immediate context)
    if !wm_summary.is_empty() {
        ctx.push_str(wm_summary);
        ctx.push('\n');
    }

    // Section 2: Recent episodic memories
    if !episodic_recent.is_empty() {
        ctx.push_str("SOUVENIRS RECENTS :\n");
        for ep in episodic_recent {
            let preview: String = ep.content.chars().take(200).collect();
            ctx.push_str(&format!("  - {} ({})\n", preview, ep.emotion));
        }
        ctx.push('\n');
    }

    // Section 3: Relevant long-term memories
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

    // Section 4: Deep archives (pruned LTM memories compressed into summaries)
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

    // Section 5: Subconscious memories (dreams, insights, connections, eureka, mental images)
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

/// Builds the past learnings context for injection into the LLM prompt.
///
/// Each learning is displayed with its domain, summary, and confidence.
/// This context allows the LLM to draw on its previous learnings.
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
