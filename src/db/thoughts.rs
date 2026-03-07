// =============================================================================
// db/thoughts.rs — Autonomous thought journal
//
// Purpose: Persistence for the agent's autonomous thoughts. Each thought
// is logged with its type, content, dominant emotion, and consciousness
// metrics (level, phi, mood valence, neurochemistry). This provides a
// complete audit trail of the agent's inner cognitive activity.
// =============================================================================

use super::{SaphireDb, DbError};

impl SaphireDb {
    /// Records a thought in the journal.
    /// Every autonomous thought of the agent is logged with its type,
    /// content, the emotion felt, and consciousness metrics.
    ///
    /// # Parameters
    /// - `thought_type`: category of thought (e.g., "reflection", "exploration", "dream")
    /// - `content`: textual content of the thought
    /// - `emotion`: dominant emotion during the thought
    /// - `consciousness_level`: consciousness level [0.0 - 1.0]
    /// - `phi`: integrated information (phi) metric
    /// - `mood_valence`: mood valence [-1.0 to +1.0]
    /// - `chemistry_json`: neurochemical state in JSON
    ///
    /// # Returns
    /// The ID of the inserted thought log entry
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

    /// Counts the occurrences of a thought type in the history.
    ///
    /// # Parameters
    /// - `thought_type`: the thought type to count
    ///
    /// # Returns
    /// The number of thoughts of the given type
    pub async fn count_thought_type_occurrences(&self, thought_type: &str) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "SELECT COUNT(*) FROM thought_log WHERE thought_type = $1",
            &[&thought_type],
        ).await?;
        Ok(row.get(0))
    }
}
