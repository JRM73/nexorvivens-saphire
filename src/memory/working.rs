// working.rs — Working memory (RAM-only, 7-item default capacity, Miller's Law)
//
// This module implements Saphire's working memory, inspired by the human
// cognitive model of short-term / working memory (Baddeley & Hitch, 1974).
// Working memory is a limited-capacity buffer (default 7 items, per Miller's
// Law: 7 +/- 2 chunks) that holds information immediately relevant to the
// current cognitive cycle.
//
// Characteristics:
//   - Volatile: stored exclusively in RAM, never persisted directly to disk or DB.
//   - Decay: each item's relevance score decreases at every cognitive cycle,
//     modeling attentional fading and the displacement of older traces.
//   - Eviction: when an item's relevance reaches 0 or the capacity limit is
//     exceeded, it is evicted and transferred to episodic memory for potential
//     later consolidation into long-term storage.
//
// Dependencies:
//   - VecDeque (std): double-ended queue for efficient front/back access.
//   - chrono: UTC timestamps for item creation.
//   - serde: serialization for WebSocket transmission to the dashboard.

use std::collections::VecDeque;
use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::neurochemistry::ChemicalSignature;

/// Source origin of a working memory item.
///
/// Each variant identifies where the information came from and holds either
/// the textual content (String) or the database identifier (i64) of a
/// recalled memory.
#[derive(Debug, Clone, Serialize)]
pub enum WorkingItemSource {
    /// Message received from the human user.
    UserMessage(String),
    /// Thought generated internally by Saphire's cognitive system.
    OwnThought(String),
    /// Response produced by the LLM (Large Language Model).
    LlmResponse(String),
    /// Knowledge acquired from web search or browsing.
    WebKnowledge(String),
    /// Memory recalled from the episodic store (contains the database row ID).
    EpisodicRecall(i64),
    /// Memory recalled from long-term storage (contains the database row ID).
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
    /// Used in the context summary and the real-time WebSocket dashboard.
    ///
    /// # Returns
    /// A static string containing a single Unicode emoji character.
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

/// A single item stored in working memory.
///
/// Each item represents one unit of information currently active in Saphire's
/// "consciousness" — the attentional focus. Its relevance decays over
/// successive cognitive cycles, modeling the fading of unattended traces.
#[derive(Debug, Clone, Serialize)]
pub struct WorkingItem {
    /// Unique sequential identifier within the working memory instance.
    pub id: u64,
    /// Textual content of the item (message, thought, knowledge, etc.).
    pub content: String,
    /// Origin of this item (user message, internal thought, recall, etc.).
    pub source: WorkingItemSource,
    /// Current relevance score, ranging from 0.0 (forgotten) to 1.0 (maximally
    /// relevant). Decreases by `decay_rate` at each cognitive cycle.
    pub relevance: f64,
    /// UTC timestamp of when this item was created.
    pub created_at: DateTime<Utc>,
    /// Saphire's dominant emotional state at the time this item was created.
    pub emotion_at_creation: String,
    /// Neurochemical signature at the time of encoding, used for
    /// state-dependent memory retrieval.
    pub chemical_signature: ChemicalSignature,
}

/// Working memory — a limited-capacity consciousness buffer.
///
/// Operates as a relevance-based priority queue. When the maximum capacity
/// is reached, the least relevant item is evicted and can be recovered for
/// transfer to episodic memory (hippocampal encoding).
pub struct WorkingMemory {
    /// Double-ended queue holding the currently active items.
    items: VecDeque<WorkingItem>,
    /// Maximum capacity (number of items, typically 7 per Miller's Law).
    max_capacity: usize,
    /// Sequential counter for assigning unique item identifiers.
    next_id: u64,
    /// Per-cycle relevance decay rate subtracted from each item's relevance.
    decay_rate: f64,
}

impl WorkingMemory {
    /// Creates a new, empty working memory instance.
    ///
    /// # Parameters
    /// - `capacity`: maximum number of simultaneous items (typically 7,
    ///   per Miller's Law on the span of immediate memory).
    /// - `decay_rate`: amount subtracted from each item's relevance at every
    ///   cognitive cycle (e.g., 0.05 = 5% loss per cycle).
    ///
    /// # Returns
    /// An empty WorkingMemory ready to receive items.
    pub fn new(capacity: usize, decay_rate: f64) -> Self {
        Self {
            items: VecDeque::with_capacity(capacity),
            max_capacity: capacity,
            next_id: 0,
            decay_rate,
        }
    }

    /// Inserts a new item into working memory.
    ///
    /// If the memory is at capacity, the item with the lowest relevance score
    /// is evicted to make room. The evicted item is returned so the caller
    /// can transfer it to episodic memory (modeling hippocampal encoding of
    /// displaced working memory traces).
    ///
    /// The new item starts with a relevance of 1.0 (maximum).
    ///
    /// # Parameters
    /// - `content`: textual content of the item.
    /// - `source`: origin of the information (message, thought, recall, etc.).
    /// - `emotion`: Saphire's current dominant emotional state.
    /// - `chemical_signature`: current neurochemical state snapshot for
    ///   state-dependent encoding.
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
            // Evict the least relevant item to respect capacity constraints.
            // Linear scan to find the index of the minimum-relevance item.
            // This is efficient because the queue is small (max ~7 items).
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

    /// Applies relevance decay to all items in working memory.
    ///
    /// Called at each cognitive cycle. Every item loses `decay_rate` relevance.
    /// Items whose relevance drops to 0 or below are automatically removed
    /// and returned for transfer to episodic memory.
    ///
    /// This models the natural fading of unattended memory traces and the
    /// displacement mechanism described in Baddeley's working memory model.
    ///
    /// # Returns
    /// A vector of items evicted due to their relevance reaching zero.
    pub fn decay(&mut self) -> Vec<WorkingItem> {
        let mut ejected = Vec::new();
        // Apply decay to each item, clamping at 0.0 to prevent negative values.
        for item in &mut self.items {
            item.relevance = (item.relevance - self.decay_rate).max(0.0);
        }
        // Remove items whose relevance has dropped to zero.
        // Uses a while-loop because indices shift after each removal.
        while let Some(pos) = self.items.iter().position(|item| item.relevance <= 0.0) {
            if let Some(item) = self.items.remove(pos) {
                ejected.push(item);
            }
        }
        ejected
    }

    /// Reinforces a specific item's relevance by +0.3 (capped at 1.0).
    ///
    /// Used when an item becomes relevant again in the current context
    /// (e.g., when the user revisits a previously mentioned topic). This
    /// models attentional re-focusing and the recency boost in working
    /// memory.
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
    /// Items are sorted by descending relevance so that the LLM sees the
    /// most pertinent information first. This ordering ensures that the
    /// most salient context occupies the top positions in the prompt.
    ///
    /// # Returns
    /// A formatted string with each item prefixed by its source icon,
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

    /// Drains all items from working memory and returns them.
    ///
    /// Used for bulk transfer to episodic memory, for example during
    /// Saphire's "sleep" phase, analogous to the hippocampal replay
    /// of the day's experiences during slow-wave sleep.
    ///
    /// # Returns
    /// A vector containing all removed items.
    pub fn drain_all(&mut self) -> Vec<WorkingItem> {
        self.items.drain(..).collect()
    }

    /// Flushes only conversation-related items (user messages and LLM responses)
    /// while retaining internal thoughts, web knowledge, and memory recalls.
    ///
    /// Called at the end of a conversation to clear the conversational context
    /// without losing background reflections and recalled knowledge.
    ///
    /// # Returns
    /// A vector of the ejected conversational items (for episodic transfer).
    pub fn flush_conversation(&mut self) -> Vec<WorkingItem> {
        let mut ejected = Vec::new();
        let mut keep = VecDeque::new();
        // Separate conversational items (to eject) from internal items (to keep).
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

    /// Returns the number of items currently stored in working memory.
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

    /// Returns a read-only reference to the underlying item queue.
    pub fn items(&self) -> &VecDeque<WorkingItem> {
        &self.items
    }

    /// Produces a JSON structure intended for real-time WebSocket transmission
    /// to the visualization dashboard.
    ///
    /// Each item is serialized with a content preview truncated to 100
    /// characters, its source type label, icon, relevance score, and
    /// associated emotion. The JSON also includes the total capacity and
    /// current usage count.
    ///
    /// # Returns
    /// A `serde_json::Value` object ready for JSON serialization.
    pub fn ws_data(&self) -> serde_json::Value {
        let items: Vec<serde_json::Value> = self.items.iter().map(|item| {
            // Truncate content to 100 characters for compact dashboard display.
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
