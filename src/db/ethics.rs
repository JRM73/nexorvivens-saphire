// =============================================================================
// db/ethics.rs — Personal ethics (layer 2)
//
// Purpose: CRUD operations for the agent's personal ethical principles.
// Unlike hardcoded safety rules, these principles are emergent -- they are
// discovered, refined, and sometimes deactivated through lived experience.
// Each principle tracks its origin, invocation count, questioning count,
// and active status. A full history of modifications is maintained in
// the personal_ethics_history table for auditability.
// =============================================================================

use chrono::{DateTime, Utc};
use super::{SaphireDb, DbError};

impl SaphireDb {
    /// Loads all personal ethical principles from the database.
    ///
    /// # Returns
    /// A vector of tuples containing: (id, title, content, reasoning, born_from,
    /// born_at_cycle, emotion_at_creation, times_invoked, times_questioned,
    /// is_active, created_at)
    pub async fn load_personal_ethics(&self) -> Result<Vec<(i64, String, String, String, String, i64, String, i64, i64, bool, DateTime<Utc>)>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, title, content, reasoning, born_from, born_at_cycle,
                    emotion_at_creation, times_invoked, times_questioned, is_active, created_at
             FROM personal_ethics ORDER BY created_at ASC",
            &[],
        ).await?;

        let mut results = Vec::new();
        for row in &rows {
            results.push((
                row.get::<_, i64>(0),
                row.get::<_, String>(1),
                row.get::<_, String>(2),
                row.get::<_, String>(3),
                row.get::<_, String>(4),
                row.get::<_, i64>(5),
                row.get::<_, String>(6),
                row.get::<_, i64>(7),
                row.get::<_, i64>(8),
                row.get::<_, bool>(9),
                row.get::<_, DateTime<Utc>>(10),
            ));
        }
        Ok(results)
    }

    /// Saves a new personal ethical principle. Returns the ID.
    ///
    /// # Parameters
    /// - `title`: short title of the principle
    /// - `content`: full content of the ethical principle
    /// - `reasoning`: justification for adopting this principle
    /// - `born_from`: the experience that gave rise to this principle
    /// - `born_at_cycle`: the cycle number at which this principle was created
    /// - `emotion_at_creation`: the dominant emotion when the principle was formed
    pub async fn save_personal_principle(
        &self,
        title: &str,
        content: &str,
        reasoning: &str,
        born_from: &str,
        born_at_cycle: i64,
        emotion_at_creation: &str,
    ) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "INSERT INTO personal_ethics (title, content, reasoning, born_from, born_at_cycle, emotion_at_creation)
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
            &[&title, &content, &reasoning, &born_from, &born_at_cycle, &emotion_at_creation],
        ).await?;
        Ok(row.get(0))
    }

    /// Records an entry in the ethical modifications history.
    ///
    /// # Parameters
    /// - `principle_id`: ID of the principle being modified
    /// - `action`: type of modification (e.g., "created", "updated", "deactivated")
    /// - `old_content`: previous content (None for creation)
    /// - `new_content`: new content (None for deactivation)
    /// - `reason_for_change`: explanation for why the change was made
    /// - `emotion_at_change`: dominant emotion during the change
    /// - `cycle`: cycle number at which the change occurred
    #[allow(clippy::too_many_arguments)]
    pub async fn save_ethics_history(
        &self,
        principle_id: i64,
        action: &str,
        old_content: Option<&str>,
        new_content: Option<&str>,
        reason_for_change: Option<&str>,
        emotion_at_change: Option<&str>,
        cycle: i64,
    ) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "INSERT INTO personal_ethics_history (principle_id, action, old_content, new_content, reason_for_change, emotion_at_change, cycle)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[&principle_id, &action, &old_content, &new_content, &reason_for_change, &emotion_at_change, &cycle],
        ).await?;
        Ok(())
    }

    /// Increments the invocation counter of an ethical principle.
    /// Also updates the last_invoked_at timestamp.
    pub async fn update_principle_invocation(&self, id: i64) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE personal_ethics SET times_invoked = times_invoked + 1, last_invoked_at = NOW() WHERE id = $1",
            &[&id],
        ).await?;
        Ok(())
    }

    /// Updates the active/inactive status of an ethical principle.
    pub async fn update_principle_status(&self, id: i64, is_active: bool) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE personal_ethics SET is_active = $2, modified_at = NOW() WHERE id = $1",
            &[&id, &is_active],
        ).await?;
        Ok(())
    }
}
