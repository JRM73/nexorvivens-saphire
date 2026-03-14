// =============================================================================
// db/profiling.rs — Personality, OCEAN profiles and human profiles
// =============================================================================

use super::{SaphireDb, DbError};

impl SaphireDb {
    /// Saves the agent's emergent personality traits.
    /// Deletes the old traits and inserts the new ones (full replacement).
    pub async fn save_personality_traits(&self, traits: &[(String, f32, f32)]) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute("DELETE FROM personality_traits", &[]).await?;
        for (name, value, confidence) in traits {
            client.execute(
                "INSERT INTO personality_traits (trait_name, trait_value, confidence) VALUES ($1, $2, $3)",
                &[name, value, confidence],
            ).await?;
        }
        Ok(())
    }

    /// Loads personality traits from the database.
    pub async fn load_personality_traits(&self) -> Result<Vec<(String, f32, f32)>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT trait_name, trait_value, confidence FROM personality_traits ORDER BY computed_at DESC",
            &[],
        ).await?;

        let mut traits = Vec::new();
        for row in &rows {
            traits.push((row.get(0), row.get(1), row.get(2)));
        }
        Ok(traits)
    }

    /// Saves Saphire's OCEAN profile (upsert singleton, id=1).
    pub async fn save_ocean_profile(
        &self,
        ocean_json: &serde_json::Value,
        data_points: i64,
        confidence: f32,
        history_json: &serde_json::Value,
    ) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let updated = client.execute(
            "UPDATE ocean_self_profile SET
                ocean_json = $1, data_points = $2, confidence = $3,
                history_json = $4, updated_at = NOW()
             WHERE id = 1",
            &[ocean_json, &data_points, &confidence, history_json],
        ).await?;

        if updated == 0 {
            client.execute(
                "INSERT INTO ocean_self_profile (id, ocean_json, data_points, confidence, history_json)
                 VALUES (1, $1, $2, $3, $4)",
                &[ocean_json, &data_points, &confidence, history_json],
            ).await?;
        }
        Ok(())
    }

    /// Loads Saphire's OCEAN profile from the database.
    pub async fn load_ocean_profile(&self) -> Result<Option<(serde_json::Value, i64, f32)>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT ocean_json, data_points, confidence FROM ocean_self_profile WHERE id = 1",
            &[],
        ).await?;
        match result {
            Some(row) => Ok(Some((row.get(0), row.get(1), row.get(2)))),
            None => Ok(None),
        }
    }

    /// Saves a human profile (upsert by identifier).
    #[allow(clippy::too_many_arguments)]
    pub async fn save_human_profile(
        &self,
        id: &str,
        name: &str,
        ocean_json: &serde_json::Value,
        style_json: &serde_json::Value,
        interaction_count: i64,
        topics_json: &serde_json::Value,
        patterns_json: &serde_json::Value,
        rapport_score: f32,
    ) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "INSERT INTO human_profiles (id, name, ocean_json, communication_style_json,
                interaction_count, preferred_topics, emotional_patterns, rapport_score, last_seen)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
             ON CONFLICT (id) DO UPDATE SET
                name = $2, ocean_json = $3, communication_style_json = $4,
                interaction_count = $5, preferred_topics = $6,
                emotional_patterns = $7, rapport_score = $8, last_seen = NOW()",
            &[&id, &name, ocean_json, style_json, &interaction_count,
              topics_json, patterns_json, &rapport_score],
        ).await?;
        Ok(())
    }

    /// Loads all human profiles from the database.
    /// Returns the 50 most recently seen profiles.
    pub async fn load_human_profiles(&self) -> Result<Vec<(String, serde_json::Value)>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, row_to_json(human_profiles) FROM human_profiles ORDER BY last_seen DESC LIMIT 50",
            &[],
        ).await?;
        let mut profiles = Vec::new();
        for row in &rows {
            let id: String = row.get(0);
            let json: serde_json::Value = row.get(1);
            profiles.push((id, json));
        }
        Ok(profiles)
    }

    /// Retrieves the OCEAN history.
    pub async fn get_ocean_history(&self) -> Result<serde_json::Value, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT history_json FROM ocean_self_profile WHERE id = 1",
            &[],
        ).await?;
        match result {
            Some(row) => Ok(row.get(0)),
            None => Ok(serde_json::json!([])),
        }
    }
}
