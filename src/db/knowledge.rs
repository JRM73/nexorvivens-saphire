// =============================================================================
// db/knowledge.rs — Acquired knowledge (WebKnowledge)
// =============================================================================

use super::{SaphireDb, DbError};

impl SaphireDb {
    /// Records an acquired piece of knowledge (typically from a web search).
    #[allow(clippy::too_many_arguments)]
    pub async fn log_knowledge(
        &self,
        source: &str,
        query: &str,
        title: &str,
        url: &str,
        extract: &str,
        llm_reflection: Option<&str>,
        emotion: Option<&str>,
        satisfaction: Option<f32>,
    ) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "INSERT INTO knowledge_log (source, query, title, url, extract, llm_reflection, emotion, satisfaction)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
            &[&source, &query, &title, &url, &extract, &llm_reflection, &emotion, &satisfaction],
        ).await?;
        Ok(row.get(0))
    }

    /// Counts the total number of acquired knowledge entries.
    pub async fn count_knowledge(&self) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "SELECT COUNT(*) FROM knowledge_log",
            &[],
        ).await?;
        Ok(row.get(0))
    }

    /// Retrieves the N most recent acquired knowledge entries.
    pub async fn recent_knowledge(&self, limit: i64) -> Result<Vec<(String, String, String)>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT source, title, created_at::text FROM knowledge_log ORDER BY created_at DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            results.push((row.get(0), row.get(1), row.get(2)));
        }
        Ok(results)
    }

    /// Loads knowledge statistics for restoration at boot.
    /// Returns (unique titles by date DESC, total searches, article_read_count).
    pub async fn load_knowledge_stats(&self) -> Result<(Vec<String>, u64, std::collections::HashMap<String, u32>), DbError> {
        let client = self.pool.get().await?;
        // Unique titles sorted by last appearance
        let rows = client.query(
            "SELECT title, COUNT(*)::bigint as cnt FROM knowledge_log GROUP BY title ORDER BY MAX(created_at) DESC",
            &[],
        ).await?;
        let mut titles = Vec::new();
        let mut read_counts = std::collections::HashMap::new();
        let mut total: u64 = 0;
        for row in &rows {
            let title: String = row.get(0);
            let cnt: i64 = row.get(1);
            titles.push(title.clone());
            read_counts.insert(title, cnt as u32);
            total += cnt as u64;
        }
        Ok((titles, total, read_counts))
    }

    /// Lists recent knowledge titles (for anti-repetition).
    pub async fn recent_knowledge_titles(&self, limit: i64) -> Result<Vec<String>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT title FROM knowledge_log ORDER BY created_at DESC LIMIT $1",
            &[&limit],
        ).await?;
        Ok(rows.iter().map(|r| r.get(0)).collect())
    }
}
