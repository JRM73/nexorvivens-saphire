// vectorstore/mod.rs — Vector memory (local TF-IDF + brute-force cosine)
//
// This module implements an in-memory vector store (in RAM) for semantic
// search of memories by cosine similarity.
//
// It serves as a complementary search layer to pgvector (in the database).
// Memories are encoded into fixed-dimension vectors via a local encoder
// based on TF-IDF (Term Frequency-Inverse Document Frequency) with
// n-gram hashing via FNV-1a (Fowler-Noll-Vo 1a).
//
// Search uses a brute-force approach: each query is compared to all
// stored memories. This is acceptable because the number of in-RAM
// memories is limited by `max_memories`.
//
// Architecture:
//   - VectorMemory: data structure for a vectorized memory.
//   - VectorStore: container with encoding, storage, search, and pruning.
//   - cosine_similarity: utility function for similarity computation.
//
// Sub-modules:
//   - encoder: local TF-IDF encoder (LocalEncoder).
//   - personality: emergent personality computed from memories.
//
// Dependencies:
//   - serde: serialization/deserialization of vector memories.

pub mod encoder;
pub mod personality;

use serde::{Deserialize, Serialize};
use self::encoder::LocalEncoder;

/// A memory encoded as a vector.
///
/// Associates raw text with its embedding (a fixed-dimension numeric vector),
/// an emotion, and an importance score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMemory {
    /// Textual content of the memory.
    pub text: String,
    /// Fixed-dimension embedding vector, produced by the LocalEncoder.
    /// Represents the semantic content of the text in vector space.
    pub embedding: Vec<f64>,
    /// Emotion associated with this memory (e.g., "Joie", "Curiosity").
    pub emotion: String,
    /// Importance score of the memory, between 0.0 (insignificant) and 1.0 (crucial).
    /// Used for pruning: the least important memories are removed first when
    /// the maximum capacity is reached.
    pub importance: f64,
}

/// In-memory vector store with cosine similarity search.
///
/// Stores vector memories in RAM and provides methods for semantic search
/// by brute force (comparing each memory with the query). Automatic pruning
/// removes the least important memories when the maximum capacity is exceeded.
pub struct VectorStore {
    /// List of stored vector memories.
    memories: Vec<VectorMemory>,
    /// Local encoder for transforming text into embedding vectors.
    encoder: LocalEncoder,
    /// Maximum number of allowed memories. Beyond this, pruning by importance.
    max_memories: usize,
}

impl VectorStore {
    /// Creates a new empty vector store.
    ///
    /// # Parameters
    /// - `embedding_dim`: dimension of embedding vectors (e.g., 256, 512).
    ///   Determines the size of the vector space used by the LocalEncoder.
    /// - `max_memories`: maximum capacity in number of memories.
    ///
    /// # Returns
    /// An empty VectorStore with a configured encoder.
    pub fn new(embedding_dim: usize, max_memories: usize) -> Self {
        Self {
            memories: Vec::new(),
            encoder: LocalEncoder::new(embedding_dim),
            max_memories,
        }
    }

    /// Adds a memory to the vector store.
    ///
    /// The text is automatically encoded into an embedding vector via the
    /// LocalEncoder. If the number of memories exceeds `max_memories`,
    /// the least important memories are pruned (sorted by descending
    /// importance, then truncated).
    ///
    /// # Parameters
    /// - `text`: textual content of the memory.
    /// - `emotion`: emotion associated with the memory.
    /// - `importance`: importance score (0.0 to 1.0).
    pub fn add(&mut self, text: &str, emotion: &str, importance: f64) {
        let embedding = self.encoder.encode(text);
        self.memories.push(VectorMemory {
            text: text.to_string(),
            embedding,
            emotion: emotion.to_string(),
            importance,
        });

        // Prune if the maximum capacity is exceeded.
        // Sort by descending importance and keep only the
        // `max_memories` most important memories.
        if self.memories.len() > self.max_memories {
            self.memories.sort_by(|a, b| b.importance.partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal));
            self.memories.truncate(self.max_memories);
        }
    }

    /// Adds a memory with an externally pre-computed embedding.
    ///
    /// Unlike `add()`, this method does not recompute the embedding
    /// and does not trigger automatic pruning. Useful for loading
    /// memories from the database.
    ///
    /// # Parameters
    /// - `text`: textual content of the memory.
    /// - `embedding`: pre-computed embedding vector.
    /// - `emotion`: emotion associated with the memory.
    /// - `importance`: importance score (0.0 to 1.0).
    pub fn add_with_embedding(&mut self, text: &str, embedding: Vec<f64>, emotion: &str, importance: f64) {
        self.memories.push(VectorMemory {
            text: text.to_string(),
            embedding,
            emotion: emotion.to_string(),
            importance,
        });
    }

    /// Searches for the K most similar memories to a text query.
    ///
    /// The query text is encoded into an embedding vector, then
    /// compared to all memories by cosine similarity.
    ///
    /// # Parameters
    /// - `query`: text of the search query.
    /// - `k`: maximum number of results to return.
    ///
    /// # Returns
    /// Vector of tuples (similarity score, reference to the memory),
    /// sorted by descending similarity.
    pub fn search(&self, query: &str, k: usize) -> Vec<(f64, &VectorMemory)> {
        let query_embedding = self.encoder.encode(query);
        self.search_by_embedding(&query_embedding, k)
    }

    /// Searches for the K most similar memories to a given embedding.
    ///
    /// Version of `search()` that accepts a pre-computed embedding vector
    /// instead of raw text. Useful when the embedding is already available.
    ///
    /// Algorithm: brute force -- compute cosine similarity between the
    /// query vector and each memory, then sort and return the top K.
    ///
    /// # Parameters
    /// - `query`: embedding vector of the query.
    /// - `k`: maximum number of results to return.
    ///
    /// # Returns
    /// Vector of tuples (similarity score, reference to the memory),
    /// sorted by descending similarity.
    pub fn search_by_embedding(&self, query: &[f64], k: usize) -> Vec<(f64, &VectorMemory)> {
        // Compute cosine similarity between the query and each memory.
        let mut scored: Vec<(f64, &VectorMemory)> = self.memories.iter()
            .map(|mem| {
                let sim = cosine_similarity(query, &mem.embedding);
                (sim, mem)
            })
            .collect();

        // Sort by descending similarity and keep only the top K.
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(k).collect()
    }

    /// Returns the number of currently stored memories.
    pub fn len(&self) -> usize {
        self.memories.len()
    }

    /// Checks whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.memories.is_empty()
    }

    /// Returns a read-only slice of all memories.
    pub fn memories(&self) -> &[VectorMemory] {
        &self.memories
    }

    /// Returns a read-only reference to the local encoder.
    pub fn encoder(&self) -> &LocalEncoder {
        &self.encoder
    }
}

/// Computes the cosine similarity between two vectors.
///
/// Cosine similarity measures the angle between two vectors in vector
/// space, regardless of their magnitude. It equals:
///   - 1.0 if the vectors point in the same direction (identical).
///   - 0.0 if the vectors are orthogonal (no relation).
///   - -1.0 if the vectors point in opposite directions.
///
/// Formula: cos(theta) = (A . B) / (||A|| * ||B||)
///
/// # Parameters
/// - `a`: first vector.
/// - `b`: second vector.
///
/// # Returns
/// Similarity score between -1.0 and 1.0. Returns 0.0 if either vector
/// has a near-zero norm (< 1e-10), to avoid division by zero.
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    // Dot product of the two vectors.
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    // L2 norm (Euclidean norm) of each vector.
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    // Guard against division by zero: if a vector is near-zero,
    // similarity is undefined, so return 0.
    if norm_a < 1e-10 || norm_b < 1e-10 {
        return 0.0;
    }
    // Clamp to [-1.0, 1.0] to compensate for potential floating-point
    // rounding errors.
    (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
}
