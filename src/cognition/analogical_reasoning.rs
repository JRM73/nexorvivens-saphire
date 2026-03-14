// =============================================================================
// analogical_reasoning.rs — Analogical reasoning
//
// Role: Detects analogies between the current situation and memories
// in long-term memory (LTM). When a situation structurally resembles
// a past experience, the module transfers the insight: "last time
// something similar happened, here is what occurred / what worked".
//
// Mechanism:
//   - Comparison by lexical overlap (common words) between the current
//     context and LTM memory summaries.
//   - Similarity bonus if the current emotion matches the memory's emotion.
//   - Configurable threshold (default 0.65) to filter out weak analogies.
//   - Relevant analogies boost dopamine (cognitive reward).
//
// Place in architecture:
//   Standalone module called during the cognitive pipeline, after LTM
//   memory retrieval and before LLM generation. Found analogies
//   enrich the substrate prompt with experiential references.
// =============================================================================

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------
/// Configuration for the analogical reasoning module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalogicalReasoningConfig {
    /// Enables or disables analogical reasoning
    pub enabled: bool,
    /// Minimum structural similarity threshold (0.0 to 1.0)
    pub similarity_threshold: f64,
    /// Maximum number of recent analogies kept
    pub max_recent: usize,
}

impl Default for AnalogicalReasoningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            similarity_threshold: 0.65,
            max_recent: 20,
        }
    }
}

// ---------------------------------------------------------------------------
// Simplified memory record
// ---------------------------------------------------------------------------
/// Simplified memory record for analogical reasoning.
/// Real MemoryRecords are in crate::db; we use a local type
/// to decouple the module from the database layer.
#[derive(Debug, Clone)]
pub struct MemoryRecordRef {
    /// Textual summary of the memory
    pub text_summary: String,
    /// Dominant emotion associated with the memory
    pub emotion: String,
    /// Vector similarity score (pre-computed by pgvector)
    pub similarity: f64,
}

// ---------------------------------------------------------------------------
// Analogy
// ---------------------------------------------------------------------------
/// A detected analogy between a memory and the current context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analogy {
    /// Unique identifier for the analogy
    pub id: u64,
    /// Summary of the source memory (LTM memory)
    pub source_memory_summary: String,
    /// Target context (current situation)
    pub target_context: String,
    /// Structural similarity score (0.0 to 1.0)
    pub structural_similarity: f64,
    /// Insight transferred from the past experience
    pub transferred_insight: String,
    /// Analogy domain: "resolution", "emotion", "comportement"
    pub domain: String,
    /// Cognitive cycle during which the analogy was formed
    pub cycle: u64,
    /// True if the analogy was confirmed as relevant in hindsight
    pub confirmed: bool,
}

// ---------------------------------------------------------------------------
// Analogical reasoning engine
// ---------------------------------------------------------------------------
/// Analogical reasoning engine — detects parallels between
/// the present and past experiences to transfer insights.
pub struct AnalogicalReasoning {
    /// Module enabled or not
    pub enabled: bool,
    /// Circular buffer of recent analogies
    pub recent_analogies: VecDeque<Analogy>,
    /// Historical success rate of analogies (EMA)
    pub success_rate: f64,
    /// Minimum similarity threshold to form an analogy
    pub similarity_threshold: f64,
    /// Total analogy counter since startup
    pub total_analogies: u64,
    /// Maximum size of the recent analogies buffer
    max_recent: usize,
    /// Next unique identifier
    next_id: u64,
}

impl Default for AnalogicalReasoning {
    fn default() -> Self {
        Self::new(&AnalogicalReasoningConfig::default())
    }
}

impl AnalogicalReasoning {
    /// Creates a new analogical reasoning engine.
    pub fn new(config: &AnalogicalReasoningConfig) -> Self {
        Self {
            enabled: config.enabled,
            recent_analogies: VecDeque::with_capacity(config.max_recent),
            success_rate: 0.5,
            similarity_threshold: config.similarity_threshold,
            total_analogies: 0,
            max_recent: config.max_recent,
            next_id: 1,
        }
    }

    /// Forms analogies between the current context and LTM memories.
    ///
    /// For each memory record, computes structural similarity
    /// by lexical overlap (common words) and emotional correspondence.
    /// If the score exceeds the threshold, an analogy is created and added to the buffer.
    ///
    /// Returns the number of new analogies formed.
    pub fn form_analogies(
        &mut self,
        current_context: &str,
        ltm_records: &[MemoryRecordRef],
        current_emotion: &str,
        cycle: u64,
    ) -> usize {
        if !self.enabled || current_context.is_empty() {
            return 0;
        }

        let context_words = Self::extract_words(current_context);
        if context_words.is_empty() {
            return 0;
        }

        let mut count = 0;

        for record in ltm_records {
            let memory_words = Self::extract_words(&record.text_summary);
            if memory_words.is_empty() {
                continue;
            }

            // -- Lexical similarity: proportion of common words --
            let common = context_words.iter()
                .filter(|w| memory_words.contains(w))
                .count();
            let union_size = context_words.len().max(memory_words.len());
            let lexical_sim = common as f64 / union_size as f64;

            // -- Emotion bonus: +0.15 if emotions match --
            let emotion_bonus = if !current_emotion.is_empty()
                && !record.emotion.is_empty()
                && current_emotion.to_lowercase() == record.emotion.to_lowercase()
            {
                0.15
            } else {
                0.0
            };

            // -- Combined score, capped at 1.0 --
            let structural_similarity = (lexical_sim + emotion_bonus).min(1.0);

            if structural_similarity < self.similarity_threshold {
                continue;
            }

            // -- Determine the analogy domain --
            let domain = Self::detect_domain(current_context, &record.emotion);

            // -- Build the transferred insight --
            let source_short: String = record.text_summary.chars().take(80).collect();
            let context_short: String = current_context.chars().take(60).collect();
            let transferred_insight = format!(
                "Comme [{}], je pourrais [appliquer cette experience a : {}]",
                source_short, context_short
            );

            let analogy = Analogy {
                id: self.next_id,
                source_memory_summary: record.text_summary.clone(),
                target_context: current_context.to_string(),
                structural_similarity,
                transferred_insight,
                domain,
                cycle,
                confirmed: false,
            };

            // Add to circular buffer
            if self.recent_analogies.len() >= self.max_recent {
                self.recent_analogies.pop_front();
            }
            self.recent_analogies.push_back(analogy);

            self.next_id += 1;
            self.total_analogies += 1;
            count += 1;
        }

        count
    }

    /// Description for the LLM substrate prompt.
    /// Produces readable text describing the active analogies.
    pub fn describe_for_prompt(&self) -> String {
        if !self.enabled {
            return String::new();
        }

        let recent: Vec<&Analogy> = self.recent_analogies.iter()
            .rev()
            .take(3)
            .collect();

        if recent.is_empty() {
            return "ANALOGIES : Aucune analogie active — situation inedite.".into();
        }

        let mut lines = vec!["ANALOGIES — Cette situation me rappelle :".to_string()];

        for (i, a) in recent.iter().enumerate() {
            let source_short: String = a.source_memory_summary.chars().take(60).collect();
            lines.push(format!(
                "  {}. [{}] sim={:.0}% — \"{}\"",
                i + 1,
                a.domain,
                a.structural_similarity * 100.0,
                source_short,
            ));
        }

        lines.push(format!(
            "  Taux de reussite historique : {:.0}%. Total : {}.",
            self.success_rate * 100.0,
            self.total_analogies,
        ));

        lines.join("\n")
    }

    /// Chemistry influence from analogical reasoning.
    /// A relevant recent analogy boosts dopamine (cognitive reward).
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        if !self.enabled {
            return ChemistryAdjustment::default();
        }

        // Check if there are recent relevant analogies
        let has_strong_analogy = self.recent_analogies.iter()
            .rev()
            .take(3)
            .any(|a| a.structural_similarity > self.similarity_threshold);

        if has_strong_analogy {
            ChemistryAdjustment {
                dopamine: 0.03,
                ..Default::default()
            }
        } else {
            ChemistryAdjustment::default()
        }
    }

    /// Serializes the analogical engine state to JSON for the API.
    pub fn to_json(&self) -> serde_json::Value {
        let recent_json: Vec<serde_json::Value> = self.recent_analogies.iter()
            .rev()
            .take(5)
            .map(|a| {
                serde_json::json!({
                    "id": a.id,
                    "source_summary": a.source_memory_summary.chars().take(100).collect::<String>(),
                    "target_context": a.target_context.chars().take(100).collect::<String>(),
                    "structural_similarity": (a.structural_similarity * 1000.0).round() / 1000.0,
                    "transferred_insight": a.transferred_insight,
                    "domain": a.domain,
                    "cycle": a.cycle,
                    "confirmed": a.confirmed,
                })
            })
            .collect();

        serde_json::json!({
            "enabled": self.enabled,
            "total_analogies": self.total_analogies,
            "recent_count": self.recent_analogies.len(),
            "success_rate": (self.success_rate * 1000.0).round() / 1000.0,
            "similarity_threshold": self.similarity_threshold,
            "recent_analogies": recent_json,
        })
    }

    /// Confirms an analogy as relevant (positive feedback).
    /// Updates the success rate via exponential moving average.
    pub fn confirm_analogy(&mut self, analogy_id: u64) {
        if let Some(a) = self.recent_analogies.iter_mut().find(|a| a.id == analogy_id) {
            a.confirmed = true;
            // EMA: weight 0.1 for the new data point
            self.success_rate = self.success_rate * 0.9 + 0.1;
        }
    }

    /// Invalidates an analogy (negative feedback).
    /// Reduces the success rate via EMA.
    pub fn invalidate_analogy(&mut self, analogy_id: u64) {
        if let Some(a) = self.recent_analogies.iter_mut().find(|a| a.id == analogy_id) {
            a.confirmed = false;
            // EMA: weight 0.1 for the new data point (failure = 0.0)
            self.success_rate = self.success_rate * 0.9;
        }
    }

    // -----------------------------------------------------------------------
    // Internal utility functions
    // -----------------------------------------------------------------------
    /// Extracts significant words from a text (lowercase, > 2 characters).
    /// Filters common stop words in French and English.
    fn extract_words(text: &str) -> Vec<String> {
        // Stop words to ignore
        const STOP_WORDS: &[&str] = &[
            "le", "la", "les", "de", "du", "des", "un", "une",
            "et", "ou", "en", "est", "sont", "dans", "sur", "pour",
            "par", "avec", "que", "qui", "ne", "pas", "ce", "se",
            "je", "tu", "il", "nous", "vous", "ils", "mon", "ma",
            "the", "is", "are", "was", "and", "for", "with", "this",
            "that", "from", "has", "have", "but", "not", "can",
        ];

        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric() && c != '-')
            .filter(|w| w.len() > 2)
            .filter(|w| !STOP_WORDS.contains(w))
            .map(|w| w.to_string())
            .collect()
    }

    /// Detects the analogy domain from the context and emotion.
    fn detect_domain(context: &str, emotion: &str) -> String {
        let ctx_lower = context.to_lowercase();

        // Problem-solving keywords
        if ctx_lower.contains("probleme")
            || ctx_lower.contains("resoudre")
            || ctx_lower.contains("solution")
            || ctx_lower.contains("comment")
            || ctx_lower.contains("pourquoi")
        {
            return "resolution".into();
        }

        // Behavior / action keywords
        if ctx_lower.contains("faire")
            || ctx_lower.contains("agir")
            || ctx_lower.contains("decide")
            || ctx_lower.contains("choix")
            || ctx_lower.contains("strategie")
        {
            return "comportement".into();
        }

        // If the emotion is non-empty, it's probably an emotional analogy
        if !emotion.is_empty() {
            return "emotion".into();
        }

        // Default: resolution
        "resolution".into()
    }
}
