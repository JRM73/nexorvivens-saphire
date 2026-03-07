// =============================================================================
// db/orchestrators.rs — Orchestrator persistence (dreams, desires, lessons, wounds)
//
// Purpose: CRUD operations for the agent's higher-level orchestration data:
//   - Dreams: narratives generated during REM sleep, stored in dream_journal
//   - Desires: goals and aspirations with priority, progress, and milestones
//   - Lessons: extracted wisdom from experience, with confidence tracking
//   - Wounds: emotional injuries with severity, healing progress, and strategy
//
// These entities represent the agent's experiential learning loop:
// experiences create wounds and lessons, dreams process them, and desires
// drive future behavior.
// =============================================================================

use chrono::{DateTime, Utc};
use super::{SaphireDb, DbError};

impl SaphireDb {
    // ─── Dreams ─────────────────────────────────────────────────────────────────

    /// Saves a dream to the journal.
    ///
    /// # Parameters
    /// - `dream_type`: type of dream (e.g., "recombination", "processing", "creative")
    /// - `narrative`: textual narrative of the dream
    /// - `dominant_emotion`: optional dominant emotion during the dream
    /// - `insight`: optional insight extracted from the dream
    /// - `source_memory_ids`: IDs of memories that inspired this dream
    /// - `surreal_connections`: JSON object of surreal/creative associations made
    /// - `remembered`: whether the dream was remembered upon waking
    /// - `sleep_phase`: optional sleep phase (e.g., "REM", "deep")
    ///
    /// # Returns
    /// The ID of the inserted dream
    #[allow(clippy::too_many_arguments)]
    pub async fn save_dream(
        &self,
        dream_type: &str,
        narrative: &str,
        dominant_emotion: Option<&str>,
        insight: Option<&str>,
        source_memory_ids: &[i64],
        surreal_connections: &serde_json::Value,
        remembered: bool,
        sleep_phase: Option<&str>,
    ) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "INSERT INTO dream_journal (dream_type, narrative, dominant_emotion, insight, source_memory_ids, surreal_connections, remembered, sleep_phase)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
            &[&dream_type, &narrative, &dominant_emotion, &insight, &source_memory_ids, &surreal_connections, &remembered, &sleep_phase],
        ).await?;
        Ok(row.get(0))
    }

    /// Loads the N most recent remembered dreams.
    pub async fn load_recent_dreams(&self, limit: i64) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, dream_type, narrative, dominant_emotion, insight, source_memory_ids, surreal_connections, remembered, sleep_phase, created_at
             FROM dream_journal ORDER BY created_at DESC LIMIT $1",
            &[&limit],
        ).await?;

        let mut results = Vec::new();
        for row in &rows {
            results.push(serde_json::json!({
                "id": row.get::<_, i64>(0),
                "dream_type": row.get::<_, Option<String>>(1),
                "narrative": row.get::<_, String>(2),
                "dominant_emotion": row.get::<_, Option<String>>(3),
                "insight": row.get::<_, Option<String>>(4),
                "source_memory_ids": row.get::<_, Option<Vec<i64>>>(5).unwrap_or_default(),
                "surreal_connections": row.get::<_, Option<serde_json::Value>>(6),
                "remembered": row.get::<_, bool>(7),
                "sleep_phase": row.get::<_, Option<String>>(8),
                "created_at": row.get::<_, DateTime<Utc>>(9).to_rfc3339(),
            }));
        }
        Ok(results)
    }

    // ─── Desirs ─────────────────────────────────────────────────────────────────

    /// Sauvegarde un nouveau desir.
    #[allow(clippy::too_many_arguments)]
    pub async fn save_desire(
        &self,
        title: &str,
        description: &str,
        desire_type: &str,
        priority: f32,
        milestones: &serde_json::Value,
        born_from: Option<&str>,
        emotion_at_birth: Option<&str>,
        chemistry_at_birth: &[f32],
    ) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "INSERT INTO desires (title, description, desire_type, priority, milestones, born_from, emotion_at_birth, chemistry_at_birth)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
            &[&title, &description, &desire_type, &priority, &milestones, &born_from, &emotion_at_birth, &chemistry_at_birth],
        ).await?;
        Ok(row.get(0))
    }

    /// Met a jour la progression et la priorite d'un desir.
    pub async fn update_desire_progress(
        &self,
        id: i64,
        progress: f32,
        priority: f32,
        milestones: &serde_json::Value,
        cycles_invested: i64,
    ) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE desires SET progress = $2, priority = $3, milestones = $4, cycles_invested = $5, last_pursued_at = NOW() WHERE id = $1",
            &[&id, &progress, &priority, &milestones, &cycles_invested],
        ).await?;
        Ok(())
    }

    /// Change le statut d'un desir (active, fulfilled, abandoned).
    pub async fn update_desire_status(&self, id: i64, status: &str) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let completed_at: Option<DateTime<Utc>> = if status == "fulfilled" || status == "abandoned" {
            Some(Utc::now())
        } else {
            None
        };
        client.execute(
            "UPDATE desires SET status = $2, completed_at = $3 WHERE id = $1",
            &[&id, &status, &completed_at],
        ).await?;
        Ok(())
    }

    /// Charge tous les desirs actifs.
    pub async fn load_active_desires(&self) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, title, description, desire_type, priority, progress, milestones, born_from, emotion_at_birth, chemistry_at_birth, cycles_invested, created_at, last_pursued_at
             FROM desires WHERE status = 'active' ORDER BY priority DESC",
            &[],
        ).await?;

        let mut results = Vec::new();
        for row in &rows {
            results.push(serde_json::json!({
                "id": row.get::<_, i64>(0),
                "title": row.get::<_, String>(1),
                "description": row.get::<_, String>(2),
                "desire_type": row.get::<_, String>(3),
                "priority": row.get::<_, f32>(4),
                "progress": row.get::<_, f32>(5),
                "milestones": row.get::<_, serde_json::Value>(6),
                "born_from": row.get::<_, Option<String>>(7),
                "emotion_at_birth": row.get::<_, Option<String>>(8),
                "chemistry_at_birth": row.get::<_, Option<Vec<f32>>>(9).unwrap_or_default(),
                "cycles_invested": row.get::<_, i64>(10),
                "created_at": row.get::<_, DateTime<Utc>>(11).to_rfc3339(),
                "last_pursued_at": row.get::<_, Option<DateTime<Utc>>>(12).map(|d| d.to_rfc3339()),
            }));
        }
        Ok(results)
    }

    // ─── Lecons ─────────────────────────────────────────────────────────────────

    /// Sauvegarde une nouvelle lecon.
    pub async fn save_lesson(
        &self,
        title: &str,
        content: &str,
        source_experience: Option<&str>,
        category: &str,
        confidence: f32,
    ) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "INSERT INTO lessons (title, content, source_experience, category, confidence)
             VALUES ($1, $2, $3, $4, $5) RETURNING id",
            &[&title, &content, &source_experience, &category, &confidence],
        ).await?;
        Ok(row.get(0))
    }

    /// Met a jour la confiance et les compteurs d'une lecon.
    pub async fn update_lesson_confidence(
        &self,
        id: i64,
        confidence: f32,
        times_applied: i32,
        times_contradicted: i32,
    ) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE lessons SET confidence = $2, times_applied = $3, times_contradicted = $4 WHERE id = $1",
            &[&id, &confidence, &times_applied, &times_contradicted],
        ).await?;
        Ok(())
    }

    /// Charge toutes les lecons.
    pub async fn load_all_lessons(&self) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, title, content, source_experience, category, times_applied, times_contradicted, confidence, learned_at
             FROM lessons ORDER BY confidence DESC",
            &[],
        ).await?;

        let mut results = Vec::new();
        for row in &rows {
            results.push(serde_json::json!({
                "id": row.get::<_, i64>(0),
                "title": row.get::<_, String>(1),
                "content": row.get::<_, String>(2),
                "source_experience": row.get::<_, Option<String>>(3),
                "category": row.get::<_, String>(4),
                "times_applied": row.get::<_, i32>(5),
                "times_contradicted": row.get::<_, i32>(6),
                "confidence": row.get::<_, f32>(7),
                "learned_at": row.get::<_, DateTime<Utc>>(8).to_rfc3339(),
            }));
        }
        Ok(results)
    }

    // ─── Blessures ──────────────────────────────────────────────────────────────

    /// Sauvegarde une nouvelle blessure.
    pub async fn save_wound(
        &self,
        wound_type: &str,
        description: &str,
        severity: f32,
    ) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "INSERT INTO wounds (wound_type, description, severity)
             VALUES ($1, $2, $3) RETURNING id",
            &[&wound_type, &description, &severity],
        ).await?;
        Ok(row.get(0))
    }

    /// Met a jour la progression de guerison d'une blessure.
    pub async fn update_wound_healing(
        &self,
        id: i64,
        healing_progress: f32,
        healing_strategy: Option<&str>,
        healed_at: Option<DateTime<Utc>>,
    ) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE wounds SET healing_progress = $2, healing_strategy = $3, healed_at = $4 WHERE id = $1",
            &[&id, &healing_progress, &healing_strategy, &healed_at],
        ).await?;
        Ok(())
    }

    /// Charge les blessures actives (non gueries).
    pub async fn load_active_wounds(&self) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, wound_type, description, severity, healing_progress, healing_strategy, created_at
             FROM wounds WHERE healed_at IS NULL ORDER BY created_at ASC",
            &[],
        ).await?;

        let mut results = Vec::new();
        for row in &rows {
            results.push(serde_json::json!({
                "id": row.get::<_, i64>(0),
                "wound_type": row.get::<_, String>(1),
                "description": row.get::<_, String>(2),
                "severity": row.get::<_, f32>(3),
                "healing_progress": row.get::<_, f32>(4),
                "healing_strategy": row.get::<_, Option<String>>(5),
                "created_at": row.get::<_, DateTime<Utc>>(6).to_rfc3339(),
            }));
        }
        Ok(results)
    }

    /// Compte les blessures gueries (pour calculer la resilience).
    pub async fn count_healed_wounds(&self) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "SELECT COUNT(*) FROM wounds WHERE healed_at IS NOT NULL",
            &[],
        ).await?;
        Ok(row.get(0))
    }
}
