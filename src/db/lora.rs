// =============================================================================
// db/lora.rs — CRUD for the lora_training_data table
//
// Role: Store high-quality thoughts to build a LoRA fine-tuning dataset.
// JSONL export for supervised training.
// =============================================================================

use serde::{Deserialize, Serialize};
use super::{SaphireDb, DbError};

/// LoRA sample retrieved from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraSample {
    pub id: i64,
    pub system_prompt: String,
    pub user_message: String,
    pub response: String,
    pub thought_type: String,
    pub quality_score: f32,
    pub reward: f32,
    pub human_feedback: Option<bool>,
    pub emotion: Option<String>,
    pub consciousness_level: Option<f32>,
    pub cycle: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl SaphireDb {
    /// Inserts a new LoRA sample into the database.
    pub async fn store_lora_sample(
        &self,
        system_prompt: &str,
        user_message: &str,
        response: &str,
        thought_type: &str,
        quality_score: f32,
        reward: f32,
        human_feedback: Option<bool>,
        emotion: Option<&str>,
        consciousness_level: Option<f32>,
        cycle: i64,
    ) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "INSERT INTO lora_training_data
             (system_prompt, user_message, response, thought_type, quality_score,
              reward, human_feedback, emotion, consciousness_level, cycle)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             RETURNING id",
            &[
                &system_prompt, &user_message, &response, &thought_type,
                &quality_score, &reward, &human_feedback,
                &emotion, &consciousness_level, &cycle,
            ],
        ).await?;
        Ok(row.get(0))
    }

    /// Total number of LoRA samples.
    pub async fn count_lora_samples(&self) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one("SELECT COUNT(*) FROM lora_training_data", &[]).await?;
        Ok(row.get(0))
    }

    /// Average quality of LoRA samples.
    pub async fn avg_lora_quality(&self) -> Result<f64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "SELECT COALESCE(AVG(quality_score), 0.0)::DOUBLE PRECISION FROM lora_training_data",
            &[],
        ).await?;
        Ok(row.get(0))
    }

    /// Exports the best LoRA samples (by descending quality).
    pub async fn export_lora_jsonl(
        &self,
        min_quality: f32,
        limit: i64,
    ) -> Result<Vec<LoraSample>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, system_prompt, user_message, response, thought_type,
                    quality_score, reward, human_feedback, emotion,
                    consciousness_level, cycle, created_at
             FROM lora_training_data
             WHERE quality_score >= $1
             ORDER BY quality_score DESC
             LIMIT $2",
            &[&min_quality, &limit],
        ).await?;

        Ok(rows.iter().map(|row| LoraSample {
            id: row.get(0),
            system_prompt: row.get(1),
            user_message: row.get(2),
            response: row.get(3),
            thought_type: row.get(4),
            quality_score: row.get(5),
            reward: row.get(6),
            human_feedback: row.get(7),
            emotion: row.get(8),
            consciousness_level: row.get(9),
            cycle: row.get(10),
            created_at: row.get(11),
        }).collect())
    }

    /// Prunes the oldest/weakest LoRA samples when max_count is exceeded.
    pub async fn prune_lora_samples(&self, max_count: i64) -> Result<u64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one("SELECT COUNT(*) FROM lora_training_data", &[]).await?;
        let count: i64 = row.get(0);

        if count <= max_count {
            return Ok(0);
        }

        let to_delete = count - max_count;
        let result = client.execute(
            "DELETE FROM lora_training_data WHERE id IN (
                SELECT id FROM lora_training_data
                ORDER BY quality_score ASC, created_at ASC
                LIMIT $1
            )",
            &[&to_delete],
        ).await?;
        Ok(result)
    }
}
