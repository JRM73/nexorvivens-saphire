// =============================================================================
// db/connections.rs — Requetes DB pour les connexions neuronales
//
// Role : CRUD sur la table neural_connections (liens decouverts entre souvenirs).
// Les connexions sont creees pendant le sommeil profond (cosine similarity) ou
// par les algorithmes subconscients (DBSCAN). Chaque connexion relie 2 souvenirs
// LTM avec un type de lien et une force de connexion.
// =============================================================================

use super::{SaphireDb, DbError};

impl SaphireDb {
    /// Recupere les connexions neuronales paginées.
    pub async fn get_neural_connections(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, memory_a_id, memory_b_id, strength, link_type,
                    link_detail, created_during_sleep, discovered_by, created_at
             FROM neural_connections
             ORDER BY created_at DESC
             LIMIT $1 OFFSET $2",
            &[&limit, &offset],
        ).await?;

        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(8);
            results.push(serde_json::json!({
                "id": row.get::<_, i64>(0),
                "memory_a_id": row.get::<_, i64>(1),
                "memory_b_id": row.get::<_, i64>(2),
                "strength": row.get::<_, f32>(3),
                "link_type": row.get::<_, String>(4),
                "link_detail": row.get::<_, Option<String>>(5),
                "created_during_sleep": row.get::<_, Option<bool>>(6).unwrap_or(true),
                "discovered_by": row.get::<_, Option<String>>(7).unwrap_or_default(),
                "created_at": ts.to_rfc3339(),
            }));
        }
        Ok(results)
    }

    /// Stats des connexions neuronales.
    pub async fn get_neural_connections_stats(&self) -> Result<serde_json::Value, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "SELECT count(*) as total,
                    count(*) FILTER (WHERE created_during_sleep) as from_sleep,
                    count(DISTINCT link_type) as link_types,
                    COALESCE(avg(strength)::real, 0) as avg_strength
             FROM neural_connections",
            &[],
        ).await?;

        let total: i64 = row.get(0);
        let from_sleep: i64 = row.get(1);
        let link_types: i64 = row.get(2);
        let avg_strength: f32 = row.try_get::<_, f32>(3).unwrap_or(0.0);

        // Types de liens repartition
        let type_rows = client.query(
            "SELECT link_type, count(*) as cnt
             FROM neural_connections
             GROUP BY link_type
             ORDER BY cnt DESC",
            &[],
        ).await?;

        let types: Vec<serde_json::Value> = type_rows.iter().map(|r| {
            serde_json::json!({
                "type": r.get::<_, String>(0),
                "count": r.get::<_, i64>(1),
            })
        }).collect();

        Ok(serde_json::json!({
            "total": total,
            "from_sleep": from_sleep,
            "from_awake": total - from_sleep,
            "link_types": link_types,
            "avg_strength": avg_strength,
            "by_type": types,
        }))
    }

    /// Compte le nombre total de connexions neuronales.
    pub async fn get_neural_connections_count(&self) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "SELECT count(*) FROM neural_connections", &[],
        ).await?;
        Ok(row.get(0))
    }

    /// Insere une connexion neuronale.
    pub async fn insert_neural_connection(
        &self,
        memory_a_id: i64,
        memory_b_id: i64,
        strength: f32,
        link_type: &str,
        link_detail: Option<&str>,
        created_during_sleep: bool,
        discovered_by: &str,
    ) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "INSERT INTO neural_connections
             (memory_a_id, memory_b_id, strength, link_type, link_detail,
              created_during_sleep, discovered_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id",
            &[&memory_a_id, &memory_b_id, &strength, &link_type,
              &link_detail, &created_during_sleep, &discovered_by],
        ).await?;
        Ok(row.get(0))
    }
}
