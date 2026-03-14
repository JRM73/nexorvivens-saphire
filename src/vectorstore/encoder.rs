// encoder.rs — Encodeurs vectoriels pour la mémoire sémantique
//
// Ce module fournit deux encodeurs interchangeables via le trait TextEncoder :
//
// 1. LocalEncoder (fallback) :
//    - TF-IDF simplifié avec hash FNV-1a sur n-grammes
//    - Rapide, déterministe, aucune dépendance externe
//    - Qualité sémantique limitée (collisions de hachage)
//
// 2. OllamaEncoder (principal) :
//    - Appelle l'endpoint /api/embeddings d'Ollama
//    - Utilise un modèle dédié (nomic-embed-text par défaut, 768 dimensions)
//    - Vrai encodage sémantique ("content" ≈ "heureux" si contextuellement proches)
//    - Fallback automatique sur LocalEncoder si Ollama est indisponible
//
// Le trait TextEncoder unifie les deux approches et permet au reste du code
// de fonctionner indépendamment du backend d'encodage choisi.

use std::time::Duration;

/// Trait unifié pour les encodeurs de texte en vecteurs d'embedding.
///
/// Toute structure implémentant ce trait peut transformer du texte en
/// vecteur numérique de dimension fixe, utilisable pour la recherche
/// par similarité cosinus.
pub trait TextEncoder: Send + Sync {
    /// Encode un texte en vecteur de dimension fixe, normalisé L2.
    fn encode(&self, text: &str) -> Vec<f64>;

    /// Retourne la dimension des vecteurs produits par cet encodeur.
    fn dim(&self) -> usize;

    /// Nom de l'encodeur (pour le logging).
    fn name(&self) -> &str;
}

// ═══════════════════════════════════════════════════════════════════
//  LocalEncoder — Fallback léger basé sur FNV-1a
// ═══════════════════════════════════════════════════════════════════

/// Encodeur local : transforme un texte en vecteur de dimension fixe.
///
/// Utilise des n-grammes (uni, bi, tri) hashés par FNV-1a et projetés
/// dans un espace vectoriel de dimension `dim`. Le vecteur résultant
/// est normalisé L2.
pub struct LocalEncoder {
    /// Dimension de l'espace vectoriel (taille des vecteurs produits).
    dim: usize,
}

impl LocalEncoder {
    /// Crée un nouvel encodeur avec la dimension vectorielle spécifiée.
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
}

impl TextEncoder for LocalEncoder {
    fn encode(&self, text: &str) -> Vec<f64> {
        let lower = text.to_lowercase();
        let tokens: Vec<&str> = lower.split_whitespace().collect();

        let mut vector = vec![0.0; self.dim];

        // Unigrammes (poids 1.0)
        for token in &tokens {
            let hash = fnv1a(token.as_bytes());
            let idx = (hash as usize) % self.dim;
            vector[idx] += 1.0;
        }

        // Bigrammes (poids 0.5)
        for window in tokens.windows(2) {
            let bigram = format!("{} {}", window[0], window[1]);
            let hash = fnv1a(bigram.as_bytes());
            let idx = (hash as usize) % self.dim;
            vector[idx] += 0.5;
        }

        // Trigrammes (poids 0.25)
        for window in tokens.windows(3) {
            let trigram = format!("{} {} {}", window[0], window[1], window[2]);
            let hash = fnv1a(trigram.as_bytes());
            let idx = (hash as usize) % self.dim;
            vector[idx] += 0.25;
        }

        // Normalisation L2
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
//  OllamaEncoder — Encodeur sémantique via Ollama
// ═══════════════════════════════════════════════════════════════════

/// Encodeur sémantique qui appelle l'endpoint /api/embeddings d'Ollama.
///
/// Produit des vecteurs de haute dimension (768 pour nomic-embed-text)
/// avec une vraie compréhension sémantique du texte. Fallback automatique
/// sur LocalEncoder si Ollama est indisponible.
pub struct OllamaEncoder {
    /// URL de base d'Ollama (ex: "http://localhost:11434")
    base_url: String,
    /// Nom du modèle d'embedding (ex: "nomic-embed-text")
    model: String,
    /// Dimension des vecteurs produits (détectée au premier appel)
    dim: usize,
    /// Timeout pour les requêtes HTTP
    timeout: Duration,
    /// Encodeur local de fallback (même dimension)
    fallback: LocalEncoder,
}

impl OllamaEncoder {
    /// Crée un nouvel encodeur Ollama.
    ///
    /// Détecte automatiquement la dimension du modèle en envoyant un texte
    /// de test. Si Ollama est indisponible, retourne None.
    pub fn try_new(base_url: &str, model: &str, timeout_secs: u64) -> Option<Self> {
        let base_url = base_url.trim_end_matches('/').to_string();
        // Supprimer /v1 si présent (Ollama natif n'utilise pas /v1)
        let base_url = base_url.trim_end_matches("/v1").to_string();
        let timeout = Duration::from_secs(timeout_secs);

        // Détecter la dimension avec retry (Ollama peut mettre du temps à charger le modèle)
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

    /// Envoie un texte de test pour détecter la dimension du modèle.
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

    /// Appelle l'API Ollama pour encoder un texte.
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
//  Fonctions utilitaires
// ═══════════════════════════════════════════════════════════════════

/// Crée l'encodeur optimal selon la configuration.
///
/// Tente d'abord OllamaEncoder (encodage sémantique de haute qualité).
/// Si Ollama n'est pas disponible, utilise LocalEncoder en fallback.
pub fn create_encoder(
    ollama_base_url: &str,
    embed_model: &str,
    timeout_secs: u64,
    fallback_dim: usize,
) -> Box<dyn TextEncoder> {
    // Essayer OllamaEncoder en premier
    if let Some(encoder) = OllamaEncoder::try_new(ollama_base_url, embed_model, timeout_secs) {
        return Box::new(encoder);
    }

    // Fallback sur LocalEncoder
    tracing::warn!(
        "Fallback sur LocalEncoder (dim={}). La qualité de recherche sémantique sera dégradée.",
        fallback_dim
    );
    Box::new(LocalEncoder::new(fallback_dim))
}

/// Calcule le hash FNV-1a 64 bits d'une séquence d'octets.
///
/// FNV-1a (Fowler-Noll-Vo 1a) est un algorithme de hachage non
/// cryptographique conçu pour être très rapide tout en ayant une bonne
/// distribution.
fn fnv1a(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
