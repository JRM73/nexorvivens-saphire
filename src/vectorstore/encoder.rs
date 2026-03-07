// encoder.rs — Local TF-IDF encoder with FNV-1a hashing
//
// This module implements a local vector encoder that transforms text into
// fixed-dimension vectors, without dependency on an external language model
// (no neural network, no API).
//
// Approach used:
//   - Tokenization by splitting on whitespace.
//   - Extraction of n-grams: unigrams, bigrams, and trigrams.
//   - Hashing via FNV-1a (Fowler-Noll-Vo 1a = a very fast non-cryptographic
//     hash algorithm) to project each n-gram into a fixed-dimension space.
//   - Degressive weighting: unigrams (1.0), bigrams (0.5), trigrams (0.25),
//     simulating a simplified TF-IDF (Term Frequency-Inverse Document
//     Frequency) where longer n-grams are more specific but less frequent.
//   - Final L2 normalization (Euclidean norm) so that all vectors have
//     unit norm, making cosine similarity equivalent to the dot product.
//
// This approach is lightweight, deterministic, and requires no pre-trained
// model. The trade-off is lower semantic quality compared to neural
// embeddings, but sufficient for approximate recall.
//
// Dependencies: none (self-contained module, standard library only).

/// Local encoder: transforms text into a fixed-dimension vector.
///
/// Uses n-grams (uni, bi, tri) hashed by FNV-1a and projected into a
/// vector space of dimension `dim`. The resulting vector is L2-normalized.
pub struct LocalEncoder {
    /// Dimension of the vector space (size of produced vectors).
    /// Typically 256 or 512. A larger dimension reduces hash collisions
    /// but increases memory usage.
    dim: usize,
}

impl LocalEncoder {
    /// Creates a new encoder with the specified vector dimension.
    ///
    /// # Parameters
    /// - `dim`: dimension of the vector space (e.g., 256, 512).
    ///
    /// # Returns
    /// A LocalEncoder instance ready to encode text.
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    /// Encodes text into a fixed-dimension vector.
    ///
    /// The encoding process proceeds in 4 steps:
    ///   1. Lowercase conversion and whitespace tokenization.
    ///   2. Accumulation of hashed n-grams into the vector.
    ///   3. L2 normalization to obtain a unit vector.
    ///
    /// # Parameters
    /// - `text`: text to encode (can be of any length).
    ///
    /// # Returns
    /// A vector of dimension `self.dim`, L2-normalized. If the text is
    /// empty or consists only of whitespace, returns a zero vector.
    pub fn encode(&self, text: &str) -> Vec<f64> {
        // Step 1: Normalize the text to lowercase and split into tokens.
        let lower = text.to_lowercase();
        let tokens: Vec<&str> = lower.split_whitespace().collect();

        let mut vector = vec![0.0; self.dim];

        // Step 2a: Unigrams (individual words), weight 1.0.
        // Each word is hashed by FNV-1a, then projected onto a vector
        // index via modulo. The counter at that index is incremented.
        for token in &tokens {
            let hash = fnv1a(token.as_bytes());
            let idx = (hash as usize) % self.dim;
            vector[idx] += 1.0;
        }

        // Step 2b: Bigrams (pairs of consecutive words), weight 0.5.
        // Bigrams capture local context and two-word expressions.
        // Reduced weight because they are more specific than unigrams.
        for window in tokens.windows(2) {
            let bigram = format!("{} {}", window[0], window[1]);
            let hash = fnv1a(bigram.as_bytes());
            let idx = (hash as usize) % self.dim;
            vector[idx] += 0.5;
        }

        // Step 2c: Trigrams (triplets of consecutive words), weight 0.25.
        // Trigrams capture even more specific patterns.
        // Minimal weight to reflect their high specificity.
        for window in tokens.windows(3) {
            let trigram = format!("{} {} {}", window[0], window[1], window[2]);
            let hash = fnv1a(trigram.as_bytes());
            let idx = (hash as usize) % self.dim;
            vector[idx] += 0.25;
        }

        // Step 3: L2 normalization (Euclidean norm).
        // Divide each component by the vector's norm to obtain a unit
        // vector. This allows comparing vectors by dot product rather
        // than full cosine similarity.
        let norm: f64 = vector.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 1e-10 {
            for v in &mut vector {
                *v /= norm;
            }
        }

        vector
    }

    /// Returns the vector space dimension of this encoder.
    pub fn dim(&self) -> usize {
        self.dim
    }
}

/// Computes the 64-bit FNV-1a hash of a byte sequence.
///
/// FNV-1a (Fowler-Noll-Vo 1a) is a non-cryptographic hash algorithm
/// designed to be very fast while having good distribution. The basic
/// operation is:
///   1. XOR each byte with the current hash.
///   2. Multiply by the FNV prime number (0x100000001b3).
///
/// The initial value (offset basis) is 0xcbf29ce484222325.
///
/// # Parameters
/// - `data`: byte sequence to hash.
///
/// # Returns
/// 64-bit unsigned hash value.
fn fnv1a(data: &[u8]) -> u64 {
    // FNV-1a 64-bit offset basis (standardized starting value).
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        // XOR with the current byte, then multiply by the FNV prime.
        // wrapping_mul handles arithmetic overflow without panicking.
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
