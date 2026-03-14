// working.rs — Working memory (RAM only, 7 items max, Miller's law)
//
// This module implements Saphire's working memory, inspired by the human
// cognitive model. Working memory is a limited-capacity buffer (default
// 7 elements, per Miller's law: 7 +/- 2) that stores information
// immediately relevant to the current cognitive cycle.
//
// Characteristics:
//   - Volatile: stored only in RAM, never directly persisted.
//   - Decay: each item loses relevance at each cycle.
//   - Eviction: when an item reaches 0 relevance or capacity is exceeded,
//     it is evicted and transferred to episodic memory.
//
// Dependencies:
//   - VecDeque (std): double-ended queue for efficient access.
//   - chrono: timestamping of item creation.
//   - serde: serialization for WebSocket dashboard output.

use std::collections::VecDeque;
use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::neurochemistry::ChemicalSignature;

/// Source of a working memory element.
///
/// Each variant identifies the origin of the information and contains either
/// the textual content (String) or the database identifier (i64) of the
/// recalled memory.
#[derive(Debug, Clone, Serialize)]
pub enum WorkingItemSource {
    /// Message received from the human user.
    UserMessage(String),
    /// Thought generated internally by Saphire's cognitive system.
    OwnThought(String),
    /// Response produced by the LLM.
    LlmResponse(String),
    /// Knowledge acquired from the web.
    WebKnowledge(String),
    /// Memory recalled from episodic memory (contains the DB ID).
    EpisodicRecall(i64),
    /// Memory recalled from long-term memory (contains the DB ID).
    LongTermRecall(i64),
}

impl WorkingItemSource {
    /// Returns a textual label identifying the source type.
    /// Used for JSON serialization and dashboard display.
    ///
    /// # Returns
    /// A static string describing the source type (e.g., "user_message").
    pub fn label(&self) -> &str {
        match self {
            WorkingItemSource::UserMessage(_) => "user_message",
            WorkingItemSource::OwnThought(_) => "thought",
            WorkingItemSource::LlmResponse(_) => "llm_response",
            WorkingItemSource::WebKnowledge(_) => "knowledge",
            WorkingItemSource::EpisodicRecall(_) => "episodic_recall",
            WorkingItemSource::LongTermRecall(_) => "long_term_recall",
        }
    }

    /// Returns a Unicode icon corresponding to the source type.
    /// Used in the context summary and the WebSocket dashboard.
    ///
    /// # Returns
    /// A static string containing a Unicode emoji.
    pub fn icon(&self) -> &str {
        match self {
            WorkingItemSource::UserMessage(_) => "\u{1f4ac}",   // speech bubble
            WorkingItemSource::OwnThought(_) => "\u{1f4ad}",    // thought bubble
            WorkingItemSource::LlmResponse(_) => "\u{1f9e0}",   // brain
            WorkingItemSource::WebKnowledge(_) => "\u{1f4da}",   // books
            WorkingItemSource::EpisodicRecall(_) => "\u{1f4dd}", // memo
            WorkingItemSource::LongTermRecall(_) => "\u{1f48e}", // gem
        }
    }
}

/// An element stored in working memory.
///
/// Each item represents a unit of information active in Saphire's
/// "consciousness." Its relevance decays over cognitive cycles.
#[derive(Debug, Clone, Serialize)]
pub struct WorkingItem {
    /// Unique sequential identifier within working memory.
    pub id: u64,
    /// Textual content of the item (message, thought, knowledge, etc.).
    pub content: String,
    /// Origin of this item (user message, internal thought, recall, etc.).
    pub source: WorkingItemSource,
    /// Current relevance score, between 0.0 (forgotten) and 1.0 (highly relevant).
    /// Decays by `decay_rate` at each cognitive cycle.
    pub relevance: f64,
    /// UTC timestamp of this item's creation.
    pub created_at: DateTime<Utc>,
    /// Saphire's emotional state at the time this item was created.
    pub emotion_at_creation: String,
    /// Chemical signature at the time this item was created.
    pub chemical_signature: ChemicalSignature,
}

/// Working memory — limited-capacity consciousness buffer.
///
/// Functions as a priority queue based on relevance. When maximum
/// capacity is reached, the least relevant item is evicted and can
/// be recovered for transfer to episodic memory.
pub struct WorkingMemory {
    /// Double-ended queue containing active items.
    items: VecDeque<WorkingItem>,
    /// Maximum capacity (number of items, typically 7).
    max_capacity: usize,
    /// Sequential counter for item identifier assignment.
    next_id: u64,
    /// Relevance decay rate applied at each cycle.
    decay_rate: f64,
}

impl WorkingMemory {
    /// Creates a new working memory.
    ///
    /// # Parameters
    /// - `capacity`: maximum number of simultaneous items (typically 7).
    /// - `decay_rate`: amount subtracted from each item's relevance
    ///   at each cognitive cycle (e.g., 0.05 = 5% loss per cycle).
    ///
    /// # Returns
    /// An empty WorkingMemory instance, ready to receive items.
    pub fn new(capacity: usize, decay_rate: f64) -> Self {
        Self {
            items: VecDeque::with_capacity(capacity),
            max_capacity: capacity,
            next_id: 0,
            decay_rate,
        }
    }

    /// Adds a new element to working memory.
    ///
    /// If memory is full, the item with the lowest relevance is evicted
    /// to make room. The evicted item is returned so that the caller
    /// can transfer it to episodic memory.
    ///
    /// The new item starts with a relevance of 1.0 (maximum).
    ///
    /// # Parameters
    /// - `content`: textual content of the item.
    /// - `source`: origin of the information (message, thought, recall, etc.).
    /// - `emotion`: Saphire's current emotional state.
    ///
    /// # Returns
    /// `Some(WorkingItem)` if an item was evicted, `None` otherwise.
    pub fn push(
        &mut self,
        content: String,
        source: WorkingItemSource,
        emotion: String,
        chemical_signature: ChemicalSignature,
    ) -> Option<WorkingItem> {
        let item = WorkingItem {
            id: self.next_id,
            content,
            source,
            relevance: 1.0,
            created_at: Utc::now(),
            emotion_at_creation: emotion,
            chemical_signature,
        };
        self.next_id += 1;

        let ejected = if self.items.len() >= self.max_capacity {
            // Evict the least relevant element to respect capacity.
            // We search for the index of the item with the minimum relevance score
            // via a linear scan (the queue is small, max ~7 elements).
            if let Some(min_idx) = self.items.iter()
                .enumerate()
                .min_by(|a, b| a.1.relevance.partial_cmp(&b.1.relevance).unwrap())
                .map(|(i, _)| i)
            {
                self.items.remove(min_idx)
            } else {
                None
            }
        } else {
            None
        };

        self.items.push_back(item);
        ejected
    }

    /// Applies relevance decay to all items.
    ///
    /// Called at each cognitive cycle. Each item loses `decay_rate` of
    /// relevance. Items whose relevance drops to 0 or below are
    /// automatically removed and returned for episodic transfer.
    ///
    /// # Returns
    /// List of evicted items (relevance dropped to 0).
    pub fn decay(&mut self) -> Vec<WorkingItem> {
        let mut ejected = Vec::new();
        // Apply decay to each item, without going below 0.
        for item in &mut self.items {
            item.relevance = (item.relevance - self.decay_rate).max(0.0);
        }
        // Remove items whose relevance has fallen to zero.
        // We use a while loop because indices change after each removal.
        while let Some(pos) = self.items.iter().position(|item| item.relevance <= 0.0) {
            if let Some(item) = self.items.remove(pos) {
                ejected.push(item);
            }
        }
        ejected
    }

    /// Reinforces the relevance of a specific item (increase of 0.3).
    ///
    /// Used when an item becomes relevant again in the current context
    /// (for example, when the user returns to a previously discussed topic).
    /// Relevance is capped at 1.0.
    ///
    /// # Parameters
    /// - `id`: identifier of the item to reinforce.
    pub fn reinforce(&mut self, id: u64) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.relevance = (item.relevance + 0.3).min(1.0);
        }
    }

    /// Generates a textual summary of the current working memory contents.
    ///
    /// Items are sorted by descending relevance so that the LLM sees
    /// the most relevant information first.
    ///
    /// # Returns
    /// A formatted string with each item preceded by its source icon,
    /// or an empty string if working memory is empty.
    pub fn context_summary(&self) -> String {
        if self.items.is_empty() {
            return String::new();
        }

        // Sort by descending relevance to prioritize the most important items.
        let mut sorted: Vec<&WorkingItem> = self.items.iter().collect();
        sorted.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());

        let mut summary = String::from("CONTEXTE IMMEDIAT (memoire de travail) :\n");
        for item in &sorted {
            summary.push_str(&format!("  {} {}\n", item.source.icon(), item.content));
        }
        summary
    }

    /// Completely empties working memory and returns all items.
    ///
    /// Used during a bulk transfer to episodic memory
    /// (for example, during Saphire's "sleep").
    ///
    /// # Returns
    /// Vector containing all removed items.
    pub fn drain_all(&mut self) -> Vec<WorkingItem> {
        self.items.drain(..).collect()
    }

    /// Flushes only conversation-related items (user messages and LLM
    /// responses), while preserving internal thoughts, web knowledge,
    /// and memory recalls.
    ///
    /// Called at the end of a conversation to clean up the conversational
    /// context without losing background reflections.
    ///
    /// # Returns
    /// Vector of evicted conversational items (for episodic transfer).
    pub fn flush_conversation(&mut self) -> Vec<WorkingItem> {
        let mut ejected = Vec::new();
        let mut keep = VecDeque::new();
        // Separate conversational items (to evict) from internal items (to keep).
        for item in self.items.drain(..) {
            match &item.source {
                WorkingItemSource::UserMessage(_) | WorkingItemSource::LlmResponse(_) => {
                    ejected.push(item);
                },
                _ => keep.push_back(item),
            }
        }
        self.items = keep;
        ejected
    }

    /// Returns the number of elements currently in working memory.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if working memory contains no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the configured maximum capacity.
    pub fn capacity(&self) -> usize {
        self.max_capacity
    }

    /// Returns a read-only reference to the current items.
    pub fn items(&self) -> &VecDeque<WorkingItem> {
        &self.items
    }

    /// Produces a JSON structure intended to be sent via WebSocket
    /// to the real-time visualization dashboard.
    ///
    /// Each item is serialized with a content preview truncated to 100
    /// characters, its source type, icon, relevance, and associated emotion.
    /// The JSON also includes the total capacity and number of used items.
    ///
    /// # Returns
    /// A `serde_json::Value` object ready to be serialized to JSON.
    pub fn ws_data(&self) -> serde_json::Value {
        let items: Vec<serde_json::Value> = self.items.iter().map(|item| {
            // Truncate content to 100 characters for dashboard display.
            let content_preview: String = if item.content.len() > 100 {
                let preview: String = item.content.chars().take(100).collect();
                format!("{}...", preview)
            } else {
                item.content.clone()
            };
            serde_json::json!({
                "id": item.id,
                "content": content_preview,
                "source": item.source.label(),
                "icon": item.source.icon(),
                "relevance": item.relevance,
                "emotion": item.emotion_at_creation,
                "chemical_signature": item.chemical_signature,
            })
        }).collect();

        serde_json::json!({
            "items": items,
            "capacity": self.max_capacity,
            "used": self.items.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capacity_limit() {
        let mut wm = WorkingMemory::new(7, 0.05);
        for i in 0..10 {
            wm.push(format!("item_{}", i), WorkingItemSource::OwnThought("test".into()), "Curiosite".into(), ChemicalSignature::default());
        }
        assert_eq!(wm.len(), 7, "Working memory should not exceed capacity");
    }

    #[test]
    fn test_push_returns_evicted_when_full() {
        let mut wm = WorkingMemory::new(3, 0.05);
        wm.push("a".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        wm.push("b".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        wm.push("c".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        let evicted = wm.push("d".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        assert!(evicted.is_some(), "Should evict an item when at capacity");
    }

    #[test]
    fn test_decay_reduces_relevance() {
        let mut wm = WorkingMemory::new(7, 0.1);
        wm.push("test".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        let initial = wm.items().front().unwrap().relevance;
        wm.decay();
        let after = wm.items().front().unwrap().relevance;
        assert!(after < initial, "Decay should reduce relevance");
    }

    #[test]
    fn test_decay_removes_zero_relevance() {
        let mut wm = WorkingMemory::new(7, 1.0); // Very aggressive decay
        wm.push("test".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        let dropped = wm.decay();
        assert!(!dropped.is_empty(), "Extreme decay should remove items");
        assert_eq!(wm.len(), 0, "All items should be gone");
    }

    #[test]
    fn test_reinforce_increases_relevance() {
        let mut wm = WorkingMemory::new(7, 0.1);
        wm.push("test".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        wm.decay(); // Reduce relevance first
        let id = wm.items().front().unwrap().id;
        let before = wm.items().front().unwrap().relevance;
        wm.reinforce(id);
        let after = wm.items().front().unwrap().relevance;
        assert!(after > before, "Reinforce should increase relevance");
    }

    #[test]
    fn test_drain_all() {
        let mut wm = WorkingMemory::new(7, 0.05);
        wm.push("a".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        wm.push("b".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        let drained = wm.drain_all();
        assert_eq!(drained.len(), 2);
        assert_eq!(wm.len(), 0);
    }

    #[test]
    fn test_is_empty() {
        let wm = WorkingMemory::new(7, 0.05);
        assert!(wm.is_empty());
    }
}
