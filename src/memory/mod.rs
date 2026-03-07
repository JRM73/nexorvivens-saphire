// memory/mod.rs — Three-tier memory system
//
// This module is the entry point of the Saphire memory subsystem.
// It orchestrates three hierarchical memory tiers inspired by human cognitive
// psychology and the Atkinson-Shiffrin memory model:
//   1. Working memory (working): volatile RAM-only buffer with limited capacity,
//      analogous to the phonological loop and visuospatial sketchpad.
//   2. Episodic memory (episodic): recent experiences stored in PostgreSQL
//      with progressive strength decay, modeling the hippocampal encoding
//      of autobiographical events.
//   3. Long-term memory (long_term): permanently consolidated memories indexed
//      by vector embeddings (pgvector) for cosine-similarity semantic retrieval,
//      analogous to neocortical long-term storage after hippocampal replay.
//
// Key dependencies:
//   - serde: serialization and deserialization of configuration structures.
//   - crate::db: PostgreSQL database access layer.
//
// This file exposes the configuration struct (MemoryConfig), re-exported types
// from sub-modules, and the utility function build_memory_context() which
// assembles the memory context block injected into the LLM (Large Language
// Model) prompt.

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

/// Complete configuration for the three-tier memory system.
///
/// Each field has a sensible default provided by the corresponding `default_*`
/// function, allowing partial configuration in JSON/TOML files where only
/// overridden values need to be specified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Maximum capacity of working memory (number of items).
    /// Defaults to 7, based on Miller's Law (7 +/- 2 chunks), which models the
    /// limited capacity of the human phonological loop.
    #[serde(default = "default_working_capacity")]
    pub working_capacity: usize,

    /// Per-cycle relevance decay rate for working memory items.
    /// At each cognitive cycle, every item's relevance score is reduced by this
    /// amount, simulating attentional fading.
    #[serde(default = "default_working_decay")]
    pub working_decay_rate: f64,

    /// Maximum number of episodic memories stored in the database.
    /// When exceeded, a pruning pass is triggered to remove the weakest entries.
    #[serde(default = "default_episodic_max")]
    pub episodic_max: usize,

    /// Per-consolidation-cycle strength decay rate for episodic memories.
    /// Unconsolidated memories progressively lose strength until they are
    /// either consolidated into LTM or pruned, modeling hippocampal trace decay.
    #[serde(default = "default_episodic_decay")]
    pub episodic_decay_rate: f64,

    /// Target number of episodic memories after a pruning pass.
    /// When episodic_max is exceeded, the weakest entries are removed until
    /// this count is reached.
    #[serde(default = "default_episodic_prune")]
    pub episodic_prune_target: usize,

    /// Interval (in cognitive cycles) between two consolidation processes.
    /// Every N cycles, episodic memories are evaluated for transfer to long-term
    /// memory, analogous to hippocampal-neocortical replay during sleep.
    #[serde(default = "default_consol_interval")]
    pub consolidation_interval_cycles: u64,

    /// Minimum consolidation score required for an episodic memory to be
    /// transferred to long-term memory (LTM). Value between 0.0 and 1.0.
    #[serde(default = "default_consol_threshold")]
    pub consolidation_threshold: f64,

    /// When true, consolidation is also triggered during Saphire's "sleep"
    /// (idle period), mimicking the role of slow-wave sleep and REM sleep
    /// in human memory consolidation via hippocampal replay.
    #[serde(default = "default_consol_sleep")]
    pub consolidation_on_sleep: bool,

    /// Maximum number of memories in long-term memory storage.
    #[serde(default = "default_ltm_max")]
    pub ltm_max: usize,

    /// Cosine similarity threshold for considering an LTM memory as relevant
    /// during recall. Value between 0.0 and 1.0.
    #[serde(default = "default_ltm_threshold")]
    pub ltm_similarity_threshold: f64,

    /// Target number of LTM memories after a pruning pass.
    /// When ltm_max is exceeded, the weakest unprotected entries are archived
    /// until this count is reached.
    #[serde(default = "default_ltm_prune_target")]
    pub ltm_prune_target: usize,

    /// Minimum access count for an LTM memory to be protected from pruning.
    /// Frequently recalled memories are considered important and are preserved,
    /// modeling the testing effect (retrieval practice strengthens traces).
    #[serde(default = "default_ltm_protection_access_count")]
    pub ltm_protection_access_count: i32,

    /// Minimum emotional weight for an LTM memory to be protected from pruning.
    /// Emotionally significant memories resist forgetting, consistent with
    /// amygdala-mediated modulation of hippocampal consolidation.
    #[serde(default = "default_ltm_protection_emotional_weight")]
    pub ltm_protection_emotional_weight: f32,

    /// Batch size used when archiving pruned LTM memories.
    /// Pruned memories are compressed into archive summaries in batches of this size.
    #[serde(default = "default_archive_batch_size")]
    pub archive_batch_size: usize,

    /// Number of recent episodic memories retrieved for context injection.
    #[serde(default = "default_recall_episodic_limit")]
    pub recall_episodic_limit: usize,

    /// Number of LTM memories retrieved by similarity for context injection.
    #[serde(default = "default_recall_ltm_limit")]
    pub recall_ltm_limit: usize,

    /// Minimum similarity threshold for LTM recall.
    #[serde(default = "default_recall_ltm_threshold")]
    pub recall_ltm_threshold: f64,

    /// Number of deep archive entries retrieved for context injection.
    #[serde(default = "default_recall_archive_limit")]
    pub recall_archive_limit: usize,

    /// Minimum similarity threshold for archive recall.
    #[serde(default = "default_recall_archive_threshold")]
    pub recall_archive_threshold: f64,

    /// Number of subconscious vector memories (dreams, insights, connections)
    /// retrieved by similarity for context injection.
    #[serde(default = "default_recall_vectors_limit")]
    pub recall_vectors_limit: usize,

    /// Minimum similarity threshold for subconscious vector recall.
    #[serde(default = "default_recall_vectors_threshold")]
    pub recall_vectors_threshold: f64,
}

// --- Default value functions for serde deserialization ---
// Each function returns the default value for one field of MemoryConfig.

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
    /// Returns a MemoryConfig instance with all fields set to their default values.
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

/// Builds the complete memory context string intended for injection into the
/// LLM (Large Language Model) prompt.
///
/// This function merges memories from five sources into a single structured
/// text block, providing the LLM with awareness of past context and enabling
/// continuity across conversations.
///
/// # Parameters
/// - `wm_summary`: textual summary of the working memory (currently active items).
/// - `episodic_recent`: recent episodic memories retrieved from the database.
/// - `ltm_similar`: long-term memories found by cosine vector similarity
///   against the current query embedding.
/// - `archive_similar`: deep archive entries (compressed batches of pruned LTM
///   memories) found by vector similarity.
/// - `subconscious_vectors`: subconscious vector records (dreams, insights,
///   neural connections, eureka moments, mental imagery) found by similarity.
///
/// # Returns
/// A formatted string containing up to five labeled sections of memory context.
pub fn build_memory_context(
    wm_summary: &str,
    episodic_recent: &[EpisodicRecord],
    ltm_similar: &[crate::db::MemoryRecord],
    archive_similar: &[crate::db::archives::ArchiveRecord],
    subconscious_vectors: &[crate::db::vectors::SubconsciousVectorRecord],
) -> String {
    let mut ctx = String::new();

    // Section 1: Working memory (immediate context, currently active items)
    if !wm_summary.is_empty() {
        ctx.push_str(wm_summary);
        ctx.push('\n');
    }

    // Section 2: Recent episodic memories (hippocampal short-term store)
    if !episodic_recent.is_empty() {
        ctx.push_str("SOUVENIRS RECENTS :\n");
        for ep in episodic_recent {
            let preview: String = ep.content.chars().take(200).collect();
            ctx.push_str(&format!("  - {} ({})\n", preview, ep.emotion));
        }
        ctx.push('\n');
    }

    // Section 3: Relevant long-term memories (neocortical consolidated store)
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

    // Section 5: Subconscious memories (dreams, insights, neural connections,
    // eureka moments, mental imagery — generated during idle/sleep processing)
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

/// Builds the context string for past learnings, intended for LLM prompt injection.
///
/// Each learning entry is displayed with its domain, summary, and confidence score.
/// This context allows the LLM to leverage previously acquired knowledge and
/// build upon earlier insights.
///
/// # Parameters
/// - `learnings`: slice of learning records retrieved from the database.
///
/// # Returns
/// A formatted string listing all learnings, or an empty string if none exist.
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
