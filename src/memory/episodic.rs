// episodic.rs — Episodic memory (PostgreSQL, with decay)
//
// This module manages Saphire's episodic memory, the second level
// of the mnesic system. Episodic memory stores recent memories
// as contextualized episodes (content, emotion, satisfaction, etc.)
// in the PostgreSQL database.
//
// Unlike working memory (volatile in RAM), episodic memories are
// persisted but undergo progressive strength decay. Sufficiently
// important memories will be consolidated to long-term memory (LTM).
//
// Dependencies:
//   - serde: serialization / deserialization of records.
//   - chrono: timestamping of memories.
//   - serde_json: storage of structured data (stimulus, emotional chemistry).

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::neurochemistry::ChemicalSignature;

/// Structure representing a new episode to insert into the database.
///
/// Used as a DTO (Data Transfer Object) between the cognitive cycle
/// and the persistence layer.
pub struct EpisodicItem {
    /// Textual content of the memory (summary of the experienced episode).
    pub content: String,
    /// Source type that triggered this episode (e.g., "user_message",
    /// "thought", "web_knowledge").
    pub source_type: String,
    /// JSON data of the stimulus that triggered the cognitive cycle
    /// (message, event, etc.).
    pub stimulus_json: serde_json::Value,
    /// Numeric identifier of the decision made during this cycle
    /// (action chosen by the decision module).
    pub decision: i16,
    /// Emotional chemistry state at the time of the episode, serialized as JSON.
    /// Contains simulated neurotransmitter levels (dopamine, serotonin, etc.).
    pub chemistry_json: serde_json::Value,
    /// Dominant emotion felt during this episode (e.g., "Joie", "Curiosité").
    pub emotion: String,
    /// Satisfaction level resulting from the episode, between 0.0 (unsatisfied)
    /// and 1.0 (fully satisfied).
    pub satisfaction: f32,
    /// Emotional intensity of the episode, between 0.0 (neutral) and 1.0 (very intense).
    /// Key factor for consolidation: intense memories are better retained.
    pub emotional_intensity: f32,
    /// Optional identifier of the conversation during which this episode
    /// occurred. Allows grouping memories by conversation.
    pub conversation_id: Option<String>,
    /// Chemical signature at the time of encoding
    pub chemical_signature: Option<ChemicalSignature>,
}

/// Episodic record read from the database.
///
/// Contains all fields of an EpisodicItem plus metadata added
/// by the DB (id, residual strength, access counter, consolidation status, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicRecord {
    /// Unique identifier in the database (primary key).
    pub id: i64,
    /// Textual content of the memory.
    pub content: String,
    /// Source type that triggered this episode.
    pub source_type: String,
    /// Stimulus JSON data (optional if not available at read time).
    pub stimulus_json: Option<serde_json::Value>,
    /// Decision made during this cycle (optional if not available).
    pub decision: Option<i16>,
    /// Emotional chemistry state as JSON (optional).
    pub chemistry_json: Option<serde_json::Value>,
    /// Dominant emotion of the episode.
    pub emotion: String,
    /// Satisfaction level of the episode (0.0 to 1.0).
    pub satisfaction: f32,
    /// Emotional intensity of the episode (0.0 to 1.0).
    pub emotional_intensity: f32,
    /// Residual strength of the memory (0.0 to 1.0). Decays at each
    /// consolidation cycle. When it drops too low, the memory is pruned.
    pub strength: f32,
    /// Number of times this memory has been recalled (accessed during a recall).
    /// A frequently recalled memory is considered more important.
    pub access_count: i32,
    /// Timestamp of the last access to this memory (None if never recalled).
    pub last_accessed_at: Option<DateTime<Utc>>,
    /// Indicates whether this memory has already been consolidated to long-term memory.
    /// A consolidated memory is no longer a candidate for consolidation.
    pub consolidated: bool,
    /// Optional identifier of the associated conversation.
    pub conversation_id: Option<String>,
    /// Timestamp of this memory's creation in the database.
    pub created_at: DateTime<Utc>,
    /// Chemical signature at the time of encoding (None for old memories)
    pub chemical_signature: Option<ChemicalSignature>,
}

/// Computes the consolidation score of an episodic memory.
///
/// This score determines whether a memory deserves to be transferred from
/// episodic memory to long-term memory (LTM). It is based on a weighted
/// model of 5 factors, inspired by cognitive psychology:
///
/// | Factor                  | Weight | Justification                            |
/// |-------------------------|--------|------------------------------------------|
/// | Emotional intensity     | 0.35   | Strong emotions anchor memories          |
/// | Satisfaction impact     | 0.20   | Notable successes and failures matter    |
/// | Recall frequency        | 0.15   | A frequently recalled memory is important|
/// | Residual strength       | 0.15   | A still-strong memory deserves retention |
/// | Human interaction       | 0.15   | Exchanges with a human are privileged    |
///
/// The final score is modulated by the BDNF level:
/// - BDNF = 0.0 -> multiplier 0.8 (weakened consolidation)
/// - BDNF = 0.5 -> multiplier 1.0 (normal consolidation)
/// - BDNF = 1.0 -> multiplier 1.2 (enhanced consolidation)
///
/// # Parameters
/// - `record`: episodic record to evaluate.
/// - `bdnf_level`: current BDNF level (0.0 - 1.0).
///
/// # Returns
/// Score between 0.0 and 1.0. Compared against the `consolidation_threshold`
/// configuration value to decide on transfer to LTM.
pub fn consolidation_score(record: &EpisodicRecord, bdnf_level: f64) -> f64 {
    let mut score = 0.0;

    // Factor 1: Emotional intensity is the primary factor (weight 0.35).
    // Strongly emotional memories are better retained, as in humans.
    score += record.emotional_intensity as f64 * 0.35;

    // Factor 2: Satisfaction impact (weight 0.20).
    // We measure the deviation from neutral (0.5): extremes (very
    // satisfying or very unsatisfying) are more memorable than neutral.
    let satisfaction_impact = (record.satisfaction as f64 - 0.5).abs() * 2.0;
    score += satisfaction_impact * 0.20;

    // Factor 3: Recall frequency (weight 0.15).
    // A frequently recalled memory is manifestly important to Saphire.
    // We cap at 10 accesses to normalize between 0 and 1.
    let access_factor = (record.access_count as f64).min(10.0) / 10.0;
    score += access_factor * 0.15;

    // Factor 4: Residual strength of the memory (weight 0.15).
    // A memory that has resisted decay well is a good candidate.
    score += record.strength as f64 * 0.15;

    // Factor 5: Human interaction bonus (weight 0.15).
    // Memories linked to exchanges with a human are considered
    // more significant and receive a fixed bonus.
    let is_human = record.source_type == "user_message"
                 || record.source_type == "conversation";
    if is_human {
        score += 0.15;
    }

    // BDNF modulation: the neurotrophic factor modulates consolidation.
    // Low BDNF (0.0) -> 0.8x, normal BDNF (0.5) -> 1.0x, high BDNF (1.0) -> 1.2x
    let bdnf_mod = 0.8 + bdnf_level * 0.4;
    score *= bdnf_mod;

    // Final clamping to ensure the score is in the [0.0, 1.0] range.
    score.clamp(0.0, 1.0)
}
