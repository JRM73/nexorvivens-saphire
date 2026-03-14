// encoder.rs — Vector encoders for semantic memory
//
// This module provides two interchangeable encoders via the TextEncoder trait:
//
// 1. LocalEncoder (fallback):
//    - Simplified TF-IDF with FNV-1a hash on n-grams
//    - Fast, deterministic, no external dependencies
//    - Limited semantic quality (hash collisions)
//
// 2. OllamaEncoder (primary):
//    - Calls Ollama's /api/embeddings endpoint
//    - Uses a dedicated embedding model (nomic-embed-text by default, 768 dimensions)
//    - True semantic encoding ("content" ≈ "happy" if contextually close)
//    - Automatic fallback to LocalEncoder if Ollama is unavailable
//
// The TextEncoder trait unifies both approaches and allows the rest of the code
// to work independently of the chosen encoding backend.

use std::time::Duration;

/// Unified trait for text-to-embedding encoders.
///
/// Any structure implementing this trait can transform text into
/// a fixed-dimension numeric vector, usable for cosine similarity search.
pub trait TextEncoder: Send + Sync {
    /// Encodes text into a fixed-dimension, L2-normalized vector.
    fn encode(&self, text: &str) -> Vec<f64>;

    /// Returns the dimension of vectors produced by this encoder.
    fn dim(&self) -> usize;

    /// Name of the encoder (for logging).
    fn name(&self) -> &str;
}

// ═══════════════════════════════════════════════════════════════════
//  LocalEncoder — Lightweight fallback based on FNV-1a
// ═══════════════════════════════════════════════════════════════════

/// Local encoder: transforms text into a fixed-dimension vector.
///
/// Uses n-grams (uni, bi, tri) hashed by FNV-1a and projected
/// into a vector space of dimension `dim`. The resulting vector
/// is L2-normalized.
pub struct LocalEncoder {
    /// Dimension of the vector space (size of produced vectors).
    dim: usize,
}

impl LocalEncoder {
    /// Creates a new encoder with the specified vector dimension.
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
}

impl TextEncoder for LocalEncoder {
    fn encode(&self, text: &str) -> Vec<f64> {
        let lower = text.to_lowercase();
        let tokens: Vec<&str> = lower.split_whitespace().collect();

        let mut vector = vec![0.0; self.dim];

        // Unigrams (weight 1.0)
        for token in &tokens {
            let hash = fnv1a(token.as_bytes());
            let idx = (hash as usize) % self.dim;
            vector[idx] += 1.0;
        }

        // Bigrams (weight 0.5)
        for window in tokens.windows(2) {
            let bigram = format!("{} {}", window[0], window[1]);
            let hash = fnv1a(bigram.as_bytes());
            let idx = (hash as usize) % self.dim;
            vector[idx] += 0.5;
        }

        // Trigrams (weight 0.25)
        for window in tokens.windows(3) {
            let trigram = format!("{} {} {}", window[0], window[1], window[2]);
            let hash = fnv1a(trigram.as_bytes());
            let idx = (hash as usize) % self.dim;
            vector[idx] += 0.25;
        }

        // L2 normalization
        let norm: f64 = vector.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 1e-10 {
            for v in &mut vector {
                *v /= norm;
            }
        }

        vector
    }

    fn dim(&self) -> usize {
        self.dim
    }

    fn name(&self) -> &str {
        "local-fnv1a"
    }
}

// ═══════════════════════════════════════════════════════════════════
//  OllamaEncoder — Semantic encoder via Ollama
// ═══════════════════════════════════════════════════════════════════

/// Semantic encoder that calls Ollama's /api/embeddings endpoint.
///
/// Produces high-dimension vectors (768 for nomic-embed-text)
/// with true semantic understanding of text. Automatic fallback
/// to LocalEncoder if Ollama is unavailable.
pub struct OllamaEncoder {
    /// Ollama base URL (e.g.: "http://localhost:11434")
    base_url: String,
    /// Embedding model name (e.g.: "nomic-embed-text")
    model: String,
    /// Dimension of produced vectors (detected on first call)
    dim: usize,
    /// Timeout for HTTP requests
    timeout: Duration,
    /// Fallback local encoder (same dimension)
    fallback: LocalEncoder,
}

impl OllamaEncoder {
    /// Creates a new Ollama encoder.
    ///
    /// Automatically detects the model dimension by sending a test text.
    /// If Ollama is unavailable, returns None.
    pub fn try_new(base_url: &str, model: &str, timeout_secs: u64) -> Option<Self> {
        let base_url = base_url.trim_end_matches('/').to_string();
        // Remove /v1 if present (native Ollama does not use /v1)
        let base_url = base_url.trim_end_matches("/v1").to_string();
        let timeout = Duration::from_secs(timeout_secs);

        // Detect dimension with retry (Ollama may take time to load the model)
        let max_retries = 3;
        let mut dim = None;
        for attempt in 1..=max_retries {
            match Self::probe_dimension(&base_url, model, timeout) {
                Ok(d) => {
                    dim = Some(d);
                    break;
                }
                Err(e) => {
                    if attempt < max_retries {
                        tracing::info!(
                            "OllamaEncoder: tentative {}/{} échouée ({}), retry dans 5s...",
                            attempt, max_retries, e
                        );
                        std::thread::sleep(Duration::from_secs(5));
                    } else {
                        tracing::warn!(
                            "OllamaEncoder: impossible de détecter la dimension du modèle '{}' \
                             après {} tentatives: {}. Fallback sur LocalEncoder.",
                            model, max_retries, e
                        );
                        return None;
                    }
                }
            }
        }
        let dim = dim.unwrap();

        tracing::info!(
            "OllamaEncoder initialisé: modèle={}, dimension={}, url={}",
            model, dim, base_url
        );

        Some(Self {
            base_url,
            model: model.to_string(),
            dim,
            timeout,
            fallback: LocalEncoder::new(dim),
        })
    }

    /// Sends a test text to detect the model dimension.
    fn probe_dimension(base_url: &str, model: &str, timeout: Duration) -> Result<usize, String> {
        let url = format!("{}/api/embeddings", base_url);
        let body = serde_json::json!({
            "model": model,
            "prompt": "dimension test"
        });

        let agent = ureq::AgentBuilder::new()
            .timeout(timeout)
            .build();

        let body_str = serde_json::to_string(&body)
            .map_err(|e| format!("JSON serialize: {}", e))?;

        let response = agent.post(&url)
            .set("Content-Type", "application/json")
            .send_string(&body_str)
            .map_err(|e| format!("HTTP: {}", e))?;

        let resp_str = response.into_string()
            .map_err(|e| format!("Read response: {}", e))?;

        let resp: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| format!("JSON parse: {}", e))?;

        let embedding = resp["embedding"]
            .as_array()
            .ok_or_else(|| "Réponse sans champ 'embedding'".to_string())?;

        if embedding.is_empty() {
            return Err("Embedding vide".into());
        }

        Ok(embedding.len())
    }

    /// Calls the Ollama API to encode a text.
    fn ollama_encode(&self, text: &str) -> Result<Vec<f64>, String> {
        let url = format!("{}/api/embeddings", self.base_url);
        let body = serde_json::json!({
            "model": self.model,
            "prompt": text
        });

        let agent = ureq::AgentBuilder::new()
            .timeout(self.timeout)
            .build();

        let body_str = serde_json::to_string(&body)
            .map_err(|e| format!("JSON serialize: {}", e))?;

        let response = agent.post(&url)
            .set("Content-Type", "application/json")
            .send_string(&body_str)
            .map_err(|e| format!("HTTP: {}", e))?;

        let resp_str = response.into_string()
            .map_err(|e| format!("Read response: {}", e))?;

        let resp: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| format!("JSON parse: {}", e))?;

        let embedding_arr = resp["embedding"]
            .as_array()
            .ok_or_else(|| "Réponse sans champ 'embedding'".to_string())?;

        let embedding: Vec<f64> = embedding_arr.iter()
            .filter_map(|v| v.as_f64())
            .collect();

        if embedding.len() != self.dim {
            return Err(format!(
                "Dimension inattendue: {} (attendu {})",
                embedding.len(), self.dim
            ));
        }

        Ok(embedding)
    }
}

impl TextEncoder for OllamaEncoder {
    fn encode(&self, text: &str) -> Vec<f64> {
        match self.ollama_encode(text) {
            Ok(embedding) => embedding,
            Err(e) => {
                tracing::warn!("OllamaEncoder fallback (LocalEncoder): {}", e);
                self.fallback.encode(text)
            }
        }
    }

    fn dim(&self) -> usize {
        self.dim
    }

    fn name(&self) -> &str {
        "ollama"
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Utility functions
// ═══════════════════════════════════════════════════════════════════

/// Creates the optimal encoder based on configuration.
///
/// First attempts OllamaEncoder (high-quality semantic encoding).
/// If Ollama is not available, uses LocalEncoder as fallback.
pub fn create_encoder(
    ollama_base_url: &str,
    embed_model: &str,
    timeout_secs: u64,
    fallback_dim: usize,
) -> Box<dyn TextEncoder> {
    // Try OllamaEncoder first
    if let Some(encoder) = OllamaEncoder::try_new(ollama_base_url, embed_model, timeout_secs) {
        return Box::new(encoder);
    }

    // Fallback to LocalEncoder
    tracing::warn!(
        "Fallback sur LocalEncoder (dim={}). La qualité de recherche sémantique sera dégradée.",
        fallback_dim
    );
    Box::new(LocalEncoder::new(fallback_dim))
}

/// Computes the 64-bit FNV-1a hash of a byte sequence.
///
/// FNV-1a (Fowler-Noll-Vo 1a) is a non-cryptographic hashing algorithm
/// designed to be very fast while having good distribution.
fn fnv1a(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
