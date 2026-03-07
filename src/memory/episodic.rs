// episodic.rs — Episodic memory (PostgreSQL-backed, with strength decay)
//
// This module manages Saphire's episodic memory, the second tier of the
// three-level memory system. Episodic memory stores recent experiences as
// contextualized episodes (content, emotion, satisfaction, etc.) in the
// PostgreSQL database, analogous to the hippocampal encoding of
// autobiographical events in human memory.
//
// Unlike working memory (volatile, RAM-only), episodic memories are persisted
// but undergo progressive strength decay over consolidation cycles. Memories
// that are sufficiently important (high consolidation score) will be
// consolidated into long-term memory (LTM), mirroring hippocampal-to-
// neocortical transfer during sleep replay.
//
// Dependencies:
//   - serde: serialization and deserialization of records.
//   - chrono: UTC timestamps for memory creation and access tracking.
//   - serde_json: structured data storage (stimulus payload, emotional chemistry).

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::neurochemistry::ChemicalSignature;

/// A new episodic memory item to be inserted into the database.
///
/// Serves as a DTO (Data Transfer Object) between the cognitive cycle and
/// the persistence layer. All fields are populated at encoding time.
pub struct EpisodicItem {
    /// Textual content of the memory (summary of the experienced episode).
    pub content: String,
    /// Source type that triggered this episode (e.g., "user_message",
    /// "thought", "web_knowledge").
    pub source_type: String,
    /// JSON payload of the stimulus that triggered the cognitive cycle
    /// (message, event, etc.).
    pub stimulus_json: serde_json::Value,
    /// Numeric identifier of the decision made during this cycle
    /// (action chosen by the decision-making module).
    pub decision: i16,
    /// JSON-serialized emotional chemistry state at the time of the episode.
    /// Contains simulated neurotransmitter levels (dopamine, serotonin, etc.).
    pub chemistry_json: serde_json::Value,
    /// Dominant emotion experienced during this episode (e.g., "Joy", "Curiosity").
    pub emotion: String,
    /// Satisfaction level resulting from the episode, ranging from 0.0 (fully
    /// unsatisfied) to 1.0 (fully satisfied).
    pub satisfaction: f32,
    /// Emotional intensity of the episode, ranging from 0.0 (neutral) to 1.0
    /// (extremely intense). This is a key factor for consolidation: intense
    /// memories are better retained, consistent with amygdala-mediated
    /// modulation of hippocampal encoding.
    pub emotional_intensity: f32,
    /// Optional conversation identifier, used to group memories by conversation
    /// for contextual retrieval.
    pub conversation_id: Option<String>,
    /// Neurochemical signature at encoding time, enabling state-dependent
    /// memory retrieval (memories are more easily recalled when the chemical
    /// state matches the encoding state).
    pub chemical_signature: Option<ChemicalSignature>,
}

/// An episodic memory record read from the database.
///
/// Contains all fields of an EpisodicItem plus database-assigned metadata
/// (primary key, residual strength, access counter, consolidation status, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicRecord {
    /// Unique database identifier (primary key).
    pub id: i64,
    /// Textual content of the memory.
    pub content: String,
    /// Source type that triggered this episode.
    pub source_type: String,
    /// JSON payload of the stimulus (None if unavailable at read time).
    pub stimulus_json: Option<serde_json::Value>,
    /// Decision made during this cycle (None if unavailable).
    pub decision: Option<i16>,
    /// JSON emotional chemistry state (None if unavailable).
    pub chemistry_json: Option<serde_json::Value>,
    /// Dominant emotion of the episode.
    pub emotion: String,
    /// Satisfaction level of the episode (0.0 to 1.0).
    pub satisfaction: f32,
    /// Emotional intensity of the episode (0.0 to 1.0).
    pub emotional_intensity: f32,
    /// Residual strength of the memory (0.0 to 1.0). Decreases at each
    /// consolidation cycle. When it drops too low, the memory is pruned.
    /// Models hippocampal trace decay over time.
    pub strength: f32,
    /// Number of times this memory has been recalled (accessed during a
    /// retrieval operation). A frequently recalled memory is considered more
    /// important — this models the testing effect (retrieval practice
    /// strengthens memory traces).
    pub access_count: i32,
    /// Timestamp of the last access to this memory (None if never recalled).
    pub last_accessed_at: Option<DateTime<Utc>>,
    /// Whether this memory has already been consolidated into long-term memory.
    /// A consolidated memory is no longer a candidate for re-consolidation.
    pub consolidated: bool,
    /// Optional conversation identifier for grouping memories by conversation.
    pub conversation_id: Option<String>,
    /// Timestamp of when this memory was created in the database.
    pub created_at: DateTime<Utc>,
    /// Neurochemical signature at encoding time (None for legacy memories
    /// created before this feature was added).
    pub chemical_signature: Option<ChemicalSignature>,
}

/// Computes the consolidation score for an episodic memory.
///
/// This score determines whether a memory qualifies for transfer from episodic
/// memory to long-term memory (LTM). It is based on a weighted multi-factor
/// model inspired by cognitive psychology research on memory consolidation:
///
/// | Factor                  | Weight | Justification                                    |
/// |-------------------------|--------|--------------------------------------------------|
/// | Emotional intensity     | 0.35   | Strong emotions anchor memories (amygdala-mediated enhancement) |
/// | Satisfaction impact     | 0.20   | Extreme outcomes (successes and failures) are more memorable    |
/// | Recall frequency        | 0.15   | Frequently recalled memories are important (testing effect)     |
/// | Residual strength       | 0.15   | Memories that resisted decay are good consolidation candidates  |
/// | Human interaction bonus | 0.15   | Exchanges with a human are given priority (social cognition)    |
///
/// # Parameters
/// - `record`: the episodic record to evaluate.
///
/// # Returns
/// A score between 0.0 and 1.0. This is compared against the
/// `consolidation_threshold` configuration value to decide whether the
/// memory should be transferred to LTM.
pub fn consolidation_score(record: &EpisodicRecord) -> f64 {
    let mut score = 0.0;

    // Factor 1: Emotional intensity is the dominant factor (weight 0.35).
    // Strongly emotional memories are better retained, reflecting the well-
    // established amygdala-hippocampus interaction in emotional memory encoding
    // (McGaugh, 2004).
    score += record.emotional_intensity as f64 * 0.35;

    // Factor 2: Satisfaction impact (weight 0.20).
    // Measures deviation from neutral (0.5): extreme outcomes — whether highly
    // satisfying or highly unsatisfying — are more memorable than neutral ones.
    // This models the Von Restorff isolation effect for emotionally distinctive events.
    let satisfaction_impact = (record.satisfaction as f64 - 0.5).abs() * 2.0;
    score += satisfaction_impact * 0.20;

    // Factor 3: Recall frequency (weight 0.15).
    // A memory that has been recalled often is evidently important to Saphire.
    // Capped at 10 accesses and normalized to [0, 1] to prevent runaway scores.
    // Models the testing effect: each retrieval strengthens the memory trace
    // (Roediger & Karpicke, 2006).
    let access_factor = (record.access_count as f64).min(10.0) / 10.0;
    score += access_factor * 0.15;

    // Factor 4: Residual strength (weight 0.15).
    // A memory that has resisted decay over multiple consolidation cycles is a
    // strong candidate for permanent storage. This reflects the complementary
    // learning systems theory: robust hippocampal traces are preferentially
    // transferred to the neocortex.
    score += record.strength as f64 * 0.15;

    // Factor 5: Human interaction bonus (weight 0.15).
    // Memories linked to human exchanges are considered more significant,
    // reflecting the social dimension of episodic memory and the preferential
    // encoding of socially relevant information.
    let is_human = record.source_type == "user_message"
                 || record.source_type == "conversation";
    if is_human {
        score += 0.15;
    }

    // Final clamping to ensure the score stays within the [0.0, 1.0] interval.
    score.clamp(0.0, 1.0)
}
