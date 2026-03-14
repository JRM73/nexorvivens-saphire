// =============================================================================
// db/thoughts.rs — Autonomous thought journal
// =============================================================================

use super::{SaphireDb, DbError};

impl SaphireDb {
    /// Records a thought in the journal.
    /// Each autonomous thought of the agent is logged with its type,
    /// content, felt emotion, and consciousness metrics.
    #[allow(clippy::too_many_arguments)]
    pub async fn log_thought(
        &self,
        thought_type: &str,
        content: &str,
        emotion: &str,
        consciousness_level: f32,
        phi: f32,
        mood_valence: f32,
        chemistry_json: &serde_json::Value,
    ) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "INSERT INTO thought_log (thought_type, content, emotion, consciousness_level,
                                      phi, mood_valence, chemistry_json)
             VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
            &[&thought_type, &content, &emotion, &consciousness_level,
              &phi, &mood_valence, chemistry_json],
        ).await?;
        Ok(row.get(0))
    }

    /// Counts occurrences of a thought type in the history.
    pub async fn count_thought_type_occurrences(&self, thought_type: &str) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "SELECT COUNT(*) FROM thought_log WHERE thought_type = $1",
            &[&thought_type],
        ).await?;
        Ok(row.get(0))
    }
}
