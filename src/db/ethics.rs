// =============================================================================
// db/ethics.rs — Ethique personnelle (couche 2)
// =============================================================================

use chrono::{DateTime, Utc};
use super::{SaphireDb, DbError};

impl SaphireDb {
    /// Charge tous les principes ethiques personnels depuis la DB.
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

    /// Sauvegarde un nouveau principe ethique personnel. Retourne l'ID.
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

    /// Enregistre une entree dans l'historique des modifications ethiques.
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

    /// Incremente le compteur d'invocations d'un principe ethique.
    pub async fn update_principle_invocation(&self, id: i64) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE personal_ethics SET times_invoked = times_invoked + 1, last_invoked_at = NOW() WHERE id = $1",
            &[&id],
        ).await?;
        Ok(())
    }

    /// Met a jour le statut actif/inactif d'un principe ethique.
    pub async fn update_principle_status(&self, id: i64, is_active: bool) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE personal_ethics SET is_active = $2, modified_at = NOW() WHERE id = $1",
            &[&id, &is_active],
        ).await?;
        Ok(())
    }
}
