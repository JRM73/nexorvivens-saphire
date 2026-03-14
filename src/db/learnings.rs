// =============================================================================
// db/learnings.rs — CRUD for NN vectorial learnings
// =============================================================================
//
// Learning traces formulated by the LLM and stored with vectorial embeddings
// in pgvector. Complementary to the NN (implicit): here this is explicit
// episodic learning, queryable by similarity.
// =============================================================================

use super::{SaphireDb, DbError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A learning retrieved from the database (with optional similarity score).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NnLearningRecord {
    pub id: i64,
    pub domain: String,
    pub scope: String,
    pub summary: String,
    pub keywords: serde_json::Value,
    pub confidence: f32,
    pub satisfaction: f32,
    pub emotion: String,
    pub strength: f32,
    pub access_count: i32,
    pub created_at: DateTime<Utc>,
    pub similarity: f64,
}

impl SaphireDb {
    /// Stores a learning with its vectorial embedding.
    pub async fn store_nn_learning(
        &self,
        embedding: &[f32],
        domain: &str,
        scope: &str,
        summary: &str,
        keywords: &serde_json::Value,
        confidence: f32,
        satisfaction: f32,
        emotion: &str,
        cycle: i64,
    ) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let embedding_vec = pgvector::Vector::from(embedding.to_vec());
        let row = client.query_one(
            "INSERT INTO nn_learnings
             (embedding, domain, scope, summary, keywords, confidence, satisfaction, emotion, cycle_created)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id",
            &[&embedding_vec, &domain, &scope, &summary, keywords,
              &confidence, &satisfaction, &emotion, &cycle],
        ).await?;
        Ok(row.get(0))
    }

    /// Searches for similar learnings by cosine distance.
    pub async fn search_similar_learnings(
        &self,
        embedding: &[f32],
        limit: i64,
        threshold: f64,
    ) -> Result<Vec<NnLearningRecord>, DbError> {
        let client = self.pool.get().await?;
        let embedding_vec = pgvector::Vector::from(embedding.to_vec());
        let threshold_f32 = threshold as f32;
        let rows = client.query(
            "SELECT id, domain, scope, summary, keywords, confidence,
                    satisfaction, emotion, strength, access_count, created_at,
                    1 - (embedding <=> $1) as similarity
             FROM nn_learnings
             WHERE strength > 0.1
               AND 1 - (embedding <=> $1) > $3::real
             ORDER BY embedding <=> $1
             LIMIT $2",
            &[&embedding_vec, &limit, &threshold_f32],
        ).await?;

        let mut results = Vec::new();
        for row in &rows {
            results.push(NnLearningRecord {
                id: row.get(0),
                domain: row.get(1),
                scope: row.get(2),
                summary: row.get(3),
                keywords: row.get(4),
                confidence: row.get(5),
                satisfaction: row.get(6),
                emotion: row.get(7),
                strength: row.get(8),
                access_count: row.get(9),
                created_at: row.get(10),
                similarity: {
                    let sim: Option<f64> = row.get(11);
                    sim.unwrap_or(0.0)
                },
            });
        }
        Ok(results)
    }

    /// Increments the access counter and updates last_accessed_at.
    pub async fn boost_learning_access(&self, id: i64) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE nn_learnings
             SET access_count = access_count + 1,
                 last_accessed_at = NOW()
             WHERE id = $1",
            &[&id],
        ).await?;
        Ok(())
    }

    /// Reinforces a learning (confidence +0.05, strength reset to 1.0).
    pub async fn reinforce_learning(&self, id: i64) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE nn_learnings
             SET confidence = LEAST(1.0, confidence + 0.05),
                 strength = 1.0,
                 access_count = access_count + 1,
                 last_accessed_at = NOW()
             WHERE id = $1",
            &[&id],
        ).await?;
        Ok(())
    }

    /// Natural decay of learning strength.
    /// Frequently accessed learnings resist forgetting better.
    pub async fn decay_learnings(&self, rate: f64) -> Result<u64, DbError> {
        let client = self.pool.get().await?;
        let affected = client.execute(
            "UPDATE nn_learnings
             SET strength = GREATEST(0.0::real,
                strength - ($1::double precision
                    * (1.0 / (1.0 + access_count::double precision * 0.2)))::real
             )
             WHERE strength > 0.0",
            &[&rate],
        ).await?;
        Ok(affected)
    }

    /// Removes the weakest learnings if the quota is exceeded.
    pub async fn prune_learnings(&self, max_count: i64) -> Result<u64, DbError> {
        let client = self.pool.get().await?;
        let count: i64 = client.query_one(
            "SELECT COUNT(*) FROM nn_learnings",
            &[],
        ).await?.get(0);

        if count <= max_count {
            return Ok(0);
        }

        let to_remove = count - max_count;
        let affected = client.execute(
            "DELETE FROM nn_learnings
             WHERE id IN (
                 SELECT id FROM nn_learnings
                 WHERE strength < 0.2
                 ORDER BY strength ASC, created_at ASC
                 LIMIT $1
             )",
            &[&to_remove],
        ).await?;
        Ok(affected)
    }

    /// Counts the total number of learnings.
    pub async fn count_learnings(&self) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "SELECT COUNT(*) FROM nn_learnings",
            &[],
        ).await?;
        Ok(row.get(0))
    }

    /// Clears all vectorial learnings (used by FullReset).
    pub async fn clear_nn_learnings(&self) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "WITH d AS (DELETE FROM nn_learnings RETURNING id) SELECT COUNT(*) FROM d", &[],
        ).await?;
        Ok(row.get(0))
    }

    /// Loads the N most recent learnings.
    pub async fn load_recent_learnings(&self, limit: i64) -> Result<Vec<NnLearningRecord>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, domain, scope, summary, keywords, confidence,
                    satisfaction, emotion, strength, access_count, created_at
             FROM nn_learnings
             WHERE strength > 0.1
             ORDER BY created_at DESC
             LIMIT $1",
            &[&limit],
        ).await?;

        let mut results = Vec::new();
        for row in &rows {
            results.push(NnLearningRecord {
                id: row.get(0),
                domain: row.get(1),
                scope: row.get(2),
                summary: row.get(3),
                keywords: row.get(4),
                confidence: row.get(5),
                satisfaction: row.get(6),
                emotion: row.get(7),
                strength: row.get(8),
                access_count: row.get(9),
                created_at: row.get(10),
                similarity: 0.0,
            });
        }
        Ok(results)
    }
}
