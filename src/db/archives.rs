// =============================================================================
// db/archives.rs — Archives memoire (souvenirs LTM elagués compresses en lots)
//
// Role : CRUD pour la table memory_archives. Chaque archive est un resume
// compresse d'un lot de souvenirs LTM elagués, avec un embedding moyen
// normalise L2 pour la recherche par similarite cosinus.
// Les souvenirs elagués ne disparaissent jamais — ils sont archivés ici.
// =============================================================================

use super::{SaphireDb, DbError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Nouvelle archive a inserer en base.
pub struct NewArchive {
    /// Resume concatene des souvenirs du lot
    pub summary: String,
    /// Nombre de souvenirs source
    pub source_count: i32,
    /// IDs des souvenirs LTM elagués
    pub source_ids: Vec<i64>,
    /// Emotions uniques extraites du lot
    pub emotions: Vec<String>,
    /// Date du souvenir le plus ancien du lot
    pub period_start: DateTime<Utc>,
    /// Date du souvenir le plus recent du lot
    pub period_end: DateTime<Utc>,
    /// Poids emotionnel moyen du lot
    pub avg_emotional_weight: f32,
    /// Embedding moyen normalise L2 (64 dimensions)
    pub embedding: Vec<f32>,
}

/// Archive lue depuis la base de donnees.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveRecord {
    pub id: i64,
    pub summary: String,
    pub source_count: i32,
    pub source_ids: Vec<i64>,
    pub emotions: Vec<String>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub avg_emotional_weight: f32,
    pub similarity: f64,
    pub created_at: DateTime<Utc>,
}

impl SaphireDb {
    /// Stocke une archive memoire dans la table memory_archives.
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

    /// Recherche les archives les plus similaires par distance cosinus.
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

    /// Compte le nombre total d'archives.
    pub async fn count_archives(&self) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one("SELECT COUNT(*) FROM memory_archives", &[]).await?;
        Ok(row.get(0))
    }

    /// Liste les archives avec pagination (pour le dashboard).
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

    /// Statistiques des archives pour le dashboard.
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
