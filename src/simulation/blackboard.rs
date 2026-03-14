// =============================================================================
// blackboard.rs — Blackboard architecture for algorithm coordination
//
// Role: Central board where each algorithm (BT, FSM, InfluenceMap, Steering,
//       Utility AI) writes its recommendations. The Blackboard resolves
//       conflicts by priority and generates a compact summary for the LLM prompt.
//
// Place in the architecture:
//   Replaces separate injections from each algorithm with a single
//   coordination point. Each algo writes to the blackboard, and the prompt
//   reads a coherent synthesis.
// =============================================================================

use std::collections::HashMap;

/// Entry in the blackboard.
#[derive(Debug, Clone)]
pub struct BlackboardEntry {
    /// Textual value of the recommendation
    pub value: String,
    /// Source of the recommendation (algorithm name)
    pub source: &'static str,
    /// Priority (higher = more important)
    pub priority: u8,
    /// Cycle when this entry was written
    pub cycle: u64,
}

/// Central coordination board for inter-algorithm communication.
#[derive(Debug, Clone)]
pub struct Blackboard {
    /// Slots: key -> list of entries (ordered by priority)
    entries: HashMap<String, Vec<BlackboardEntry>>,
}

impl Default for Blackboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Blackboard {
    /// Creates an empty blackboard.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Writes an entry to a blackboard slot.
    /// If the slot already exists, adds the entry to the list.
    pub fn write(&mut self, slot: &str, value: String, source: &'static str, priority: u8, cycle: u64) {
        let entry = BlackboardEntry { value, source, priority, cycle };
        self.entries.entry(slot.to_string()).or_default().push(entry);
    }

    /// Reads the best entry of a slot (highest priority).
    pub fn read_best(&self, slot: &str) -> Option<&BlackboardEntry> {
        self.entries.get(slot)
            .and_then(|entries| entries.iter().max_by_key(|e| e.priority))
    }

    /// Reads all entries of a slot.
    pub fn read_all(&self, slot: &str) -> Vec<&BlackboardEntry> {
        self.entries.get(slot)
            .map(|entries| entries.iter().collect())
            .unwrap_or_default()
    }

    /// Removes stale entries (older than `max_age` cycles).
    pub fn clear_stale(&mut self, current_cycle: u64, max_age: u64) {
        for entries in self.entries.values_mut() {
            entries.retain(|e| current_cycle - e.cycle <= max_age);
        }
        // Remove empty slots
        self.entries.retain(|_, v| !v.is_empty());
    }

    /// Generates a compact summary for the LLM prompt.
    /// Takes the best entry per slot, max 3 lines.
    pub fn describe_for_prompt(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for (slot, entries) in &self.entries {
            if let Some(best) = entries.iter().max_by_key(|e| e.priority) {
                lines.push(format!("{}: {} ({})", slot, best.value, best.source));
            }
        }
        lines.sort();
        lines.truncate(3);
        if lines.is_empty() {
            String::new()
        } else {
            lines.join(" | ")
        }
    }

    /// Resets the blackboard.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Number of active slots.
    pub fn slot_count(&self) -> usize {
        self.entries.len()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_and_read() {
        let mut bb = Blackboard::new();
        bb.write("mode", "explore".into(), "BT", 150, 10);
        bb.write("mode", "focus".into(), "FSM", 120, 10);
        let best = bb.read_best("mode").unwrap();
        assert_eq!(best.value, "explore");
        assert_eq!(best.source, "BT");
        assert_eq!(best.priority, 150);
    }

    #[test]
    fn test_clear_stale() {
        let mut bb = Blackboard::new();
        bb.write("old", "data".into(), "test", 100, 5);
        bb.write("new", "data".into(), "test", 100, 15);
        bb.clear_stale(20, 10);
        assert!(bb.read_best("old").is_none());
        assert!(bb.read_best("new").is_some());
    }

    #[test]
    fn test_describe_for_prompt() {
        let mut bb = Blackboard::new();
        bb.write("mode", "explore".into(), "BT", 150, 10);
        bb.write("focus", "philosophie".into(), "IM", 100, 10);
        let desc = bb.describe_for_prompt();
        assert!(!desc.is_empty());
        assert!(desc.contains("explore"));
    }
}
