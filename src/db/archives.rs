// =============================================================================
// db/archives.rs — Memory archives (pruned LTM memories compressed into batches)
//
// Purpose: CRUD operations for the memory_archives table. Each archive is a
// compressed summary of a batch of pruned LTM memories, with an L2-normalized
// mean embedding for cosine similarity search. Pruned memories never truly
// disappear -- they are archived here. Also provides text-based search
// (lite mode) and dashboard listing/statistics methods.
// =============================================================================

use super::{SaphireDb, DbError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A new archive to insert into the database.
pub struct NewArchive {
    /// Concatenated summary of the batch's memories
    pub summary: String,
    /// Number of source memories in the batch
    pub source_count: i32,
    /// IDs of the pruned LTM memories
    pub source_ids: Vec<i64>,
    /// Unique emotions extracted from the batch
    pub emotions: Vec<String>,
    /// Date of the oldest memory in the batch
    pub period_start: DateTime<Utc>,
    /// Date of the most recent memory in the batch
    pub period_end: DateTime<Utc>,
    /// Average emotional weight of the batch
    pub avg_emotional_weight: f32,
    /// L2-normalized mean embedding (64 dimensions)
    pub embedding: Vec<f32>,
}

/// An archive record read from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveRecord {
    /// Unique identifier of the archive
    pub id: i64,
    /// Compressed textual summary of the archived memories
    pub summary: String,
    /// Number of source memories that were compressed into this archive
    pub source_count: i32,
    /// IDs of the original LTM memories that were pruned
    pub source_ids: Vec<i64>,
    /// Unique emotions present in the archived memories
    pub emotions: Vec<String>,
    /// Start of the time period covered by this archive
    pub period_start: DateTime<Utc>,
    /// End of the time period covered by this archive
    pub period_end: DateTime<Utc>,
    /// Average emotional weight of the archived memories
    pub avg_emotional_weight: f32,
    /// Cosine similarity score (populated during vector search, 0.0 otherwise)
    pub similarity: f64,
    /// Timestamp of when the archive was created (UTC)
    pub created_at: DateTime<Utc>,
}

impl SaphireDb {
    /// Stores a memory archive in the memory_archives table.
    ///
    /// # Returns
    /// The ID of the inserted archive
    pub async fn store_archive(&self, archive: &NewArchive) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let embedding_vec = pgvector::Vector::from(archive.embedding.clone());
        let row = client.query_one(
            "INSERT INTO memory_archives
             (summary, source_count, source_ids, emotions,
              period_start, period_end, avg_emotional_weight, embedding)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id",
            &[&archive.summary, &archive.source_count, &archive.source_ids,
              &archive.emotions, &archive.period_start, &archive.period_end,
              &archive.avg_emotional_weight, &embedding_vec],
        ).await?;
        Ok(row.get(0))
    }

    /// Searches for the most similar archives by cosine distance.
    ///
    /// # Parameters
    /// - `embedding`: query vector for similarity search
    /// - `limit`: maximum number of results to return
    /// - `threshold`: minimum similarity threshold [0.0 - 1.0]
    pub async fn search_similar_archives(
        &self,
        embedding: &[f32],
        limit: i64,
        threshold: f64,
    ) -> Result<Vec<ArchiveRecord>, DbError> {
        let client = self.pool.get().await?;
        let embedding_vec = pgvector::Vector::from(embedding.to_vec());
        let threshold_f32 = threshold as f32;
        let rows = client.query(
            "SELECT id, summary, source_count, source_ids, emotions,
                    period_start, period_end, avg_emotional_weight,
                    1 - (embedding <=> $1) as similarity, created_at
             FROM memory_archives
             WHERE 1 - (embedding <=> $1) > $3::real
             ORDER BY embedding <=> $1
             LIMIT $2",
            &[&embedding_vec, &limit, &threshold_f32],
        ).await?;

        let mut archives = Vec::new();
        for row in &rows {
            archives.push(ArchiveRecord {
                id: row.get(0),
                summary: row.get(1),
                source_count: row.get(2),
                source_ids: row.get(3),
                emotions: row.get(4),
                period_start: row.get(5),
                period_end: row.get(6),
                avg_emotional_weight: row.get(7),
                similarity: row.get::<_, Option<f64>>(8).unwrap_or(0.0),
                created_at: row.get(9),
            });
        }
        Ok(archives)
    }

    /// Searches archives by text matching (without embeddings).
    /// Replaces search_similar_archives (vectorized) in the lite version.
    /// Returns archives whose summary contains one of the keywords from the text.
    pub async fn search_similar_archives_by_text(
        &self,
        query_text: &str,
        limit: i64,
        _threshold: f64,
    ) -> Result<Vec<ArchiveRecord>, DbError> {
        if query_text.trim().is_empty() {
            return Ok(vec![]);
        }
        let tokens: Vec<String> = query_text
            .split_whitespace()
            .filter(|w| w.len() >= 4)
            .take(6)
            .map(|w| format!("%{}%", w.to_lowercase()))
            .collect();
        if tokens.is_empty() {
            return Ok(vec![]);
        }
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, summary, source_count, source_ids, emotions,
                    period_start, period_end, avg_emotional_weight,
                    0.4::float8 as similarity, created_at
             FROM memory_archives
             WHERE LOWER(summary) ILIKE $1
             ORDER BY avg_emotional_weight DESC, created_at DESC
             LIMIT $2",
            &[&tokens[0], &limit],
        ).await?;

        let mut archives = Vec::new();
        for row in &rows {
            archives.push(ArchiveRecord {
                id: row.get(0),
                summary: row.get(1),
                source_count: row.get(2),
                source_ids: row.get(3),
                emotions: row.get(4),
                period_start: row.get(5),
                period_end: row.get(6),
                avg_emotional_weight: row.get(7),
                similarity: row.get::<_, f64>(8),
                created_at: row.get(9),
            });
        }
        Ok(archives)
    }

    /// Counts the total number of archives.
    pub async fn count_archives(&self) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one("SELECT COUNT(*) FROM memory_archives", &[]).await?;
        Ok(row.get(0))
    }

    /// Lists archives with pagination (for the dashboard).
    pub async fn list_archives(&self, limit: i64, offset: i64) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, summary, source_count, emotions,
                    period_start, period_end, avg_emotional_weight, created_at
             FROM memory_archives
             ORDER BY created_at DESC
             LIMIT $1 OFFSET $2",
            &[&limit, &offset],
        ).await?;

        let mut results = Vec::new();
        for row in &rows {
            let period_start: DateTime<Utc> = row.get(4);
            let period_end: DateTime<Utc> = row.get(5);
            let created_at: DateTime<Utc> = row.get(7);
            results.push(serde_json::json!({
                "id": row.get::<_, i64>(0),
                "summary": row.get::<_, String>(1),
                "source_count": row.get::<_, i32>(2),
                "emotions": row.get::<_, Vec<String>>(3),
                "period_start": period_start.to_rfc3339(),
                "period_end": period_end.to_rfc3339(),
                "avg_emotional_weight": row.get::<_, f32>(6),
                "created_at": created_at.to_rfc3339(),
            }));
        }
        Ok(results)
    }

    /// Archive statistics for the dashboard.
    ///
    /// # Returns
    /// JSON object with archive_count, total_archived_memories, and avg_emotional_weight
    pub async fn archive_stats(&self) -> Result<serde_json::Value, DbError> {
        let client = self.pool.get().await?;
        let count: i64 = client.query_one(
            "SELECT COUNT(*) FROM memory_archives", &[]
        ).await?.get(0);

        let total_sources: Option<i64> = client.query_one(
            "SELECT COALESCE(SUM(source_count), 0)::bigint FROM memory_archives", &[]
        ).await?.get(0);

        let avg_weight: Option<f32> = client.query_one(
            "SELECT AVG(avg_emotional_weight)::real FROM memory_archives", &[]
        ).await?.get(0);

        Ok(serde_json::json!({
            "archive_count": count,
            "total_archived_memories": total_sources.unwrap_or(0),
            "avg_emotional_weight": avg_weight.unwrap_or(0.0),
        }))
    }
}
