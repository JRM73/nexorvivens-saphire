// =============================================================================
// db/tuning.rs — Auto-tuning and UCB1 bandit
// =============================================================================

use super::{SaphireDb, DbError};

impl SaphireDb {
    /// Saves auto-tuning parameters (upsert singleton, id=1).
    pub async fn save_tuning_params(
        &self,
        params_json: &serde_json::Value,
        best_params_json: &serde_json::Value,
        best_score: f32,
        tuning_count: i32,
    ) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "INSERT INTO tuning_params (id, params_json, best_params_json, best_score, tuning_count, updated_at)
             VALUES (1, $1, $2, $3, $4, NOW())
             ON CONFLICT (id) DO UPDATE SET params_json = $1, best_params_json = $2,
                best_score = $3, tuning_count = $4, updated_at = NOW()",
            &[params_json, best_params_json, &best_score, &tuning_count],
        ).await?;
        Ok(())
    }

    /// Loads auto-tuning parameters from the database.
    pub async fn load_tuning_params(&self) -> Result<Option<(serde_json::Value, serde_json::Value, f32, i32)>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT params_json, best_params_json, best_score, tuning_count FROM tuning_params WHERE id = 1",
            &[],
        ).await?;
        match result {
            Some(row) => Ok(Some((row.get(0), row.get(1), row.get(2), row.get(3)))),
            None => Ok(None),
        }
    }

    /// Saves the UCB1 bandit arms.
    /// Each arm represents an autonomous thought type.
    pub async fn save_bandit_arms(&self, arms: &[(String, u64, f64)]) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        for (name, pulls, total_reward) in arms {
            let pulls_i64 = *pulls as i64;
            client.execute(
                "INSERT INTO bandit_arms (arm_name, pulls, total_reward, updated_at)
                 VALUES ($1, $2, $3, NOW())
                 ON CONFLICT (arm_name) DO UPDATE SET pulls = $2, total_reward = $3, updated_at = NOW()",
                &[name, &pulls_i64, total_reward],
            ).await?;
        }
        Ok(())
    }

    /// Loads the UCB1 bandit arms from the database.
    pub async fn load_bandit_arms(&self) -> Result<Vec<(String, u64, f64)>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT arm_name, pulls, total_reward FROM bandit_arms ORDER BY arm_name",
            &[],
        ).await?;

        let mut arms = Vec::new();
        for row in &rows {
            let name: String = row.get(0);
            let pulls: i64 = row.get(1);
            let total_reward: f64 = row.get(2);
            arms.push((name, pulls as u64, total_reward));
        }
        Ok(arms)
    }
}
