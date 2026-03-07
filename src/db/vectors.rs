// =============================================================================
// db/vectors.rs — Stockage et recherche de vecteurs multi-sources
//
// Role : Gere la table memory_vectors qui stocke les embeddings provenant
// de differentes sources (reves, connexions neuronales, insights subconscients,
// consolidation, eureka). Permet la recherche par similarite cosinus sur
// l'ensemble des productions cognitives nocturnes et subconscientes.
// =============================================================================

use super::{SaphireDb, DbError};

/// Resultat d'une recherche de souvenirs subconscients par similarite.
pub struct SubconsciousVectorRecord {
    pub id: i64,
    pub source_type: String,
    pub text_content: String,
    pub emotion: String,
    pub strength: f32,
    pub similarity: f64,
}

/// Source d'un vecteur memoire.
pub enum VectorSource {
    /// Souvenir conscient (pensee, conversation)
    Conscious,
    /// Reve genere pendant le sommeil REM
    Dream,
    /// Connexion neuronale decouverte pendant le sommeil profond
    NeuralConnection,
    /// Insight emerge du subconscient
    SubconsciousInsight,
    /// Consolidation memoire (tier 2 → tier 3)
    Consolidation,
    /// Eureka (insight spontane)
    Eureka,
    /// Image mentale vivace persistee
    MentalImagery,
}

impl VectorSource {
    pub fn as_str(&self) -> &str {
        match self {
            VectorSource::Conscious => "conscious",
            VectorSource::Dream => "dream",
            VectorSource::NeuralConnection => "neural_connection",
            VectorSource::SubconsciousInsight => "subconscious_insight",
            VectorSource::Consolidation => "consolidation",
            VectorSource::Eureka => "eureka",
            VectorSource::MentalImagery => "mental_imagery",
        }
    }
}

/// Nouveau vecteur memoire a inserer.
pub struct NewMemoryVector {
    /// Embedding vectoriel (64 dimensions)
    pub embedding: Vec<f32>,
    /// Source du vecteur
    pub source_type: VectorSource,
    /// Contenu textuel associe
    pub text_content: String,
    /// Emotion dominante au moment de la creation
    pub emotion: String,
    /// Force du vecteur (0.0 a 1.0)
    pub strength: f32,
    /// True si cree pendant le sommeil
    pub created_during_sleep: bool,
    /// Phase de sommeil (si applicable)
    pub sleep_phase: Option<String>,
    /// Identifiant de reference vers la source (dream_journal.id, etc.)
    pub source_ref_id: Option<i64>,
    /// Metadonnees supplementaires en JSON
    pub metadata_json: serde_json::Value,
}

impl SaphireDb {
    /// Stocke un vecteur memoire dans la table memory_vectors.
    pub async fn store_memory_vector(&self, vec: &NewMemoryVector) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let embedding_vec = pgvector::Vector::from(vec.embedding.clone());
        let source_str = vec.source_type.as_str();
        let row = client.query_one(
            "INSERT INTO memory_vectors
             (embedding, source_type, text_content, emotion, strength,
              created_during_sleep, sleep_phase, source_ref_id, metadata_json)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id",
            &[&embedding_vec, &source_str, &vec.text_content, &vec.emotion,
              &vec.strength, &vec.created_during_sleep, &vec.sleep_phase,
              &vec.source_ref_id, &vec.metadata_json],
        ).await?;
        Ok(row.get(0))
    }

    /// Recherche combinee dans memories (conscious) et memory_vectors (toutes sources).
    /// Retourne les vecteurs les plus proches par similarite cosinus.
    pub async fn search_all_vectors(
        &self,
        embedding: &[f32],
        limit: i64,
        threshold: f64,
    ) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let embedding_vec = pgvector::Vector::from(embedding.to_vec());
        let threshold_f32 = threshold as f32;
        let rows = client.query(
            "SELECT * FROM (
                SELECT id, 'conscious' as source_type, text_summary as text_content,
                       emotion, emotional_weight as strength,
                       1 - (embedding <=> $1) as similarity
                FROM memories
                WHERE 1 - (embedding <=> $1) > $3::real
                UNION ALL
                SELECT id, source_type, text_content,
                       emotion, strength,
                       1 - (embedding <=> $1) as similarity
                FROM memory_vectors
                WHERE 1 - (embedding <=> $1) > $3::real
                UNION ALL
                SELECT id, 'archive' as source_type, summary as text_content,
                       COALESCE(emotions[1], '') as emotion,
                       avg_emotional_weight as strength,
                       1 - (embedding <=> $1) as similarity
                FROM memory_archives
                WHERE 1 - (embedding <=> $1) > $3::real
            ) combined
            ORDER BY similarity DESC
            LIMIT $2",
            &[&embedding_vec, &limit, &threshold_f32],
        ).await?;

        let mut results = Vec::new();
        for row in &rows {
            results.push(serde_json::json!({
                "id": row.get::<_, i64>(0),
                "source_type": row.get::<_, String>(1),
                "text_content": row.get::<_, String>(2),
                "emotion": row.get::<_, String>(3),
                "strength": row.get::<_, f32>(4),
                "similarity": row.get::<_, Option<f64>>(5).unwrap_or(0.0),
            }));
        }
        Ok(results)
    }

    /// Statistiques des vecteurs memoire par source.
    pub async fn memory_vectors_stats(&self) -> Result<serde_json::Value, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT source_type, count(*), avg(strength)::real as avg_str
             FROM memory_vectors
             GROUP BY source_type
             ORDER BY count(*) DESC",
            &[],
        ).await?;

        let mut by_source = Vec::new();
        let mut total: i64 = 0;
        for row in &rows {
            let count: i64 = row.get(1);
            total += count;
            by_source.push(serde_json::json!({
                "source_type": row.get::<_, String>(0),
                "count": count,
                "avg_strength": row.try_get::<_, f32>(2).unwrap_or(0.0),
            }));
        }

        Ok(serde_json::json!({
            "total": total,
            "by_source": by_source,
        }))
    }

    /// Recherche de souvenirs subconscients par similarite cosinus.
    /// Interroge uniquement memory_vectors (reves, insights, connexions, eureka, images mentales).
    pub async fn search_subconscious_vectors(
        &self,
        embedding: &[f32],
        limit: i64,
        threshold: f64,
    ) -> Result<Vec<SubconsciousVectorRecord>, DbError> {
        let client = self.pool.get().await?;
        let embedding_vec = pgvector::Vector::from(embedding.to_vec());
        let threshold_f32 = threshold as f32;
        let rows = client.query(
            "SELECT id, source_type, text_content, emotion, strength,
                    1 - (embedding <=> $1) as similarity
             FROM memory_vectors
             WHERE 1 - (embedding <=> $1) > $3::real
             ORDER BY embedding <=> $1
             LIMIT $2",
            &[&embedding_vec, &limit, &threshold_f32],
        ).await?;

        let mut results = Vec::new();
        for row in &rows {
            results.push(SubconsciousVectorRecord {
                id: row.get(0),
                source_type: row.get(1),
                text_content: row.get(2),
                emotion: row.get(3),
                strength: row.get(4),
                similarity: row.get::<_, Option<f64>>(5).unwrap_or(0.0),
            });
        }
        Ok(results)
    }

    /// Nombre total de vecteurs memoire.
    pub async fn memory_vectors_count(&self) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one("SELECT COUNT(*) FROM memory_vectors", &[]).await?;
        Ok(row.get(0))
    }
}
