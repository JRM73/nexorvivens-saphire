// =============================================================================
// vector_memory.rs — Vector memory plugin
//
// Role: This plugin manages a vector memory in RAM and computes the
// agent's emergent personality from the emotion history.
// Memories are stored as embedding vectors enabling
// cosine similarity search.
//
// Dependencies:
//   - super: Plugin trait, BrainEvent, PluginAction (plugin system)
//   - crate::vectorstore: VectorStore (vector storage and search in RAM)
//   - crate::vectorstore::personality: EmergentPersonality (personality traits)
//
// Place in architecture:
//   This plugin is registered in the PluginManager. It reacts to
//   CycleCompleted events (to periodically recompute personality) and
//   ThoughtEmitted events (to store thoughts as vector memories).
//   The emergent personality is used by the agent to enrich
//   its self-description and adapt its behavior.
// =============================================================================

use super::{Plugin, BrainEvent, PluginAction};
use crate::vectorstore::VectorStore;
use crate::vectorstore::personality::EmergentPersonality;

/// Vector memory plugin with emergent personality.
/// Stores memories as embedding vectors and periodically computes
/// personality traits from the emotional history.
pub struct VectorMemoryPlugin {
    /// The vector store in RAM (storage + similarity search)
    store: VectorStore,
    /// Emergent personality computed from memory emotions.
    /// Updates periodically as the agent accumulates experiences.
    personality: EmergentPersonality,
    /// Number of cycles between each personality recomputation
    update_interval: u64,
    /// Cycle counter since plugin creation
    cycle_count: u64,
}

impl VectorMemoryPlugin {
    /// Creates a new vector memory plugin.
    ///
    /// # Parameters
    /// - `embedding_dim`: number of dimensions for embedding vectors
    /// - `max_memories`: maximum number of memories stored in RAM
    pub fn new(embedding_dim: usize, max_memories: usize) -> Self {
        Self {
            store: VectorStore::new(embedding_dim, max_memories),
            personality: EmergentPersonality {
                traits: std::collections::HashMap::new(),
                description: "Personnalité en formation...".into(),
                memory_count: 0,
            },
            update_interval: 20, // Recompute every 20 cycles
            cycle_count: 0,
        }
    }

    /// Returns a read-only reference to the vector store.
    /// Used by the agent to perform similarity searches.
    pub fn store(&self) -> &VectorStore {
        &self.store
    }

    /// Returns a mutable reference to the vector store.
    /// Used by the agent to add memories.
    pub fn store_mut(&mut self) -> &mut VectorStore {
        &mut self.store
    }

    /// Returns a reference to the emergent personality.
    /// The personality is periodically recomputed and reflects
    /// the dominant emotions in the memory history.
    pub fn personality(&self) -> &EmergentPersonality {
        &self.personality
    }

    /// Recomputes the emergent personality from all memory emotions.
    /// Extracts the list of emotions, then uses EmergentPersonality::compute()
    /// to derive personality traits (e.g., "curious", "empathetic").
    fn update_personality(&mut self) {
        let emotions: Vec<String> = self.store.memories().iter()
            .map(|m| m.emotion.clone())
            .collect();
        self.personality = EmergentPersonality::compute(&emotions);
    }
}

impl Plugin for VectorMemoryPlugin {
    /// Returns the plugin name.
    fn name(&self) -> &str {
        "VectorMemory"
    }

    /// Reacts to brain events:
    ///
    /// - CycleCompleted: increments the cycle counter and recomputes the
    ///   personality if the interval is reached. Periodic recomputation
    ///   (rather than every cycle) avoids excessive CPU load.
    ///
    /// - ThoughtEmitted: each autonomous thought is stored as a vector
    ///   memory via a StoreMemory action, with medium importance (0.5).
    ///   This allows the agent to remember its own reflections.
    ///
    /// # Parameters
    /// - `event`: the brain event
    ///
    /// # Returns
    /// List of actions (StoreMemory for thoughts, empty otherwise)
    fn on_event(&mut self, event: &BrainEvent) -> Vec<PluginAction> {
        match event {
            BrainEvent::CycleCompleted { emotion: _, .. } => {
                self.cycle_count += 1;
                // Periodically update the emergent personality
                if self.cycle_count.is_multiple_of(self.update_interval) {
                    self.update_personality();
                }
                vec![]
            },
            BrainEvent::ThoughtEmitted { content, .. } => {
                // Store autonomous thoughts as vector memories
                vec![PluginAction::StoreMemory {
                    text: content.clone(),
                    emotion: String::new(), // The emotion will be determined by the agent
                    importance: 0.5,        // Medium importance by default
                }]
            },
            _ => vec![],
        }
    }
}
