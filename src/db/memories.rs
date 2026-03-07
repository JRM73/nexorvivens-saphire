// =============================================================================
// db/memories.rs — LTM memories, episodic memories, and founding memories
//
// Purpose: CRUD operations for the three memory tiers:
//   - Tier 3 (LTM): long-term vectorized memories with pgvector embeddings
//   - Tier 2 (Episodic): intermediate memories with natural decay
//   - Tier 1 (Founding): permanent, immutable genesis memories
//
// Also provides text-based search (lite mode) and dashboard listing methods.
// =============================================================================

use super::{SaphireDb, DbError, MemoryRecord, NewMemory};
use chrono::{DateTime, Utc};

impl SaphireDb {
    // ---------------------------------------------------------
    // MEMORIES (tier 3: long-term vectorized memory)
    // ---------------------------------------------------------

    /// Stores a memory with its vector embedding in the memories table.
    /// The embedding enables subsequent cosine similarity search via pgvector.
    ///
    /// # Parameters
    /// - `memory`: the new memory to insert (with embedding + metadata)
    ///
    /// # Returns
    /// The identifier (id) of the inserted memory
    pub async fn store_memory(&self, memory: &NewMemory) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        // Convert Vec<f32> into pgvector::Vector type compatible with PostgreSQL
        let embedding_vec = pgvector::Vector::from(memory.embedding.clone());
        let sig_json: Option<serde_json::Value> = memory.chemical_signature
            .as_ref()
            .and_then(|s| serde_json::to_value(s).ok());
        let row = client.query_one(
            "INSERT INTO memories (embedding, text_summary, stimulus_json, decision,
                    chemistry_json, emotion, mood_valence, satisfaction, emotional_weight,
                    source_episodic_id, chemical_signature)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             RETURNING id",
            &[&embedding_vec, &memory.text_summary, &memory.stimulus_json,
              &memory.decision, &memory.chemistry_json, &memory.emotion,
              &memory.mood_valence, &memory.satisfaction, &memory.emotional_weight,
              &memory.source_episodic_id, &sig_json],
        ).await?;
        Ok(row.get(0))
    }

    /// Vector search for similar memories via pgvector.
    /// Uses the <=> operator (cosine distance) to find the memories
    /// closest to the given vector.
    ///
    /// # Parameters
    /// - `embedding`: query vector (representation of the current context)
    /// - `limit`: maximum number of results to return
    /// - `threshold`: minimum similarity (0.0 to 1.0) to filter results
    ///
    /// # Returns
    /// List of similar memories, sorted by descending proximity
    pub async fn search_similar_memories(
        &self,
        embedding: &[f32],
        limit: i64,
        threshold: f64,
    ) -> Result<Vec<MemoryRecord>, DbError> {
        let client = self.pool.get().await?;
        let embedding_vec = pgvector::Vector::from(embedding.to_vec());
        let threshold_f32 = threshold as f32;
        // The <=> operator computes cosine distance between two vectors.
        // 1 - distance = similarity. We filter by similarity threshold.
        let rows = client.query(
            "SELECT id, text_summary, stimulus_json, decision, chemistry_json,
                    emotion, mood_valence, satisfaction, emotional_weight, created_at,
                    1 - (embedding <=> $1) as similarity, chemical_signature
             FROM memories
             WHERE 1 - (embedding <=> $1) > $3::real
             ORDER BY embedding <=> $1
             LIMIT $2",
            &[&embedding_vec, &limit, &threshold_f32],
        ).await?;

        let mut memories = Vec::new();
        for row in &rows {
            let chemical_signature: Option<crate::neurochemistry::ChemicalSignature> =
                row.try_get::<_, Option<serde_json::Value>>(11).ok()
                    .flatten()
                    .and_then(|v| serde_json::from_value(v).ok());
            memories.push(MemoryRecord {
                id: row.get(0),
                text_summary: row.get(1),
                stimulus_json: row.get(2),
                decision: row.get(3),
                chemistry_json: row.get(4),
                emotion: row.get(5),
                mood_valence: row.get(6),
                satisfaction: row.get(7),
                emotional_weight: row.get(8),
                created_at: row.get(9),
                similarity: {
                    let sim: Option<f64> = row.get(10);
                    sim.unwrap_or(0.0)
                },
                chemical_signature,
            });
        }
        Ok(memories)
    }

    /// Counts the total number of memories stored in the memories table.
    ///
    /// # Returns
    /// The total number of memories
    pub async fn memory_count(&self) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one("SELECT COUNT(*) FROM memories", &[]).await?;
        Ok(row.get(0))
    }

    /// Retrieves the N most recent memories (without similarity filtering).
    /// The `similarity` field is set to 0.0 since this is not a vector search.
    ///
    /// # Parameters
    /// - `n`: number of memories to retrieve
    ///
    /// # Returns
    /// List of recent memories, sorted by descending date
    pub async fn recent_memories(&self, n: i64) -> Result<Vec<MemoryRecord>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, text_summary, stimulus_json, decision, chemistry_json,
                    emotion, mood_valence, satisfaction, emotional_weight, created_at,
                    chemical_signature
             FROM memories ORDER BY created_at DESC LIMIT $1",
            &[&n],
        ).await?;

        let mut memories = Vec::new();
        for row in &rows {
            let chemical_signature: Option<crate::neurochemistry::ChemicalSignature> =
                row.try_get::<_, Option<serde_json::Value>>(10).ok()
                    .flatten()
                    .and_then(|v| serde_json::from_value(v).ok());
            memories.push(MemoryRecord {
                id: row.get(0),
                text_summary: row.get(1),
                stimulus_json: row.get(2),
                decision: row.get(3),
                chemistry_json: row.get(4),
                emotion: row.get(5),
                mood_valence: row.get(6),
                satisfaction: row.get(7),
                emotional_weight: row.get(8),
                created_at: row.get(9),
                similarity: 0.0,
                chemical_signature,
            });
        }
        Ok(memories)
    }

    /// Counts the total number of LTM memories.
    pub async fn count_ltm(&self) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one("SELECT COUNT(*) FROM memories", &[]).await?;
        Ok(row.get(0))
    }

    /// Retrieves the N weakest unprotected LTM memories.
    /// A memory is protected if access_count >= min_access OR emotional_weight >= min_weight.
    /// Returns memories sorted by emotional_weight ASC, access_count ASC, created_at ASC.
    pub async fn fetch_ltm_weakest_unprotected(
        &self,
        count: i64,
        min_access: i32,
        min_weight: f32,
    ) -> Result<Vec<MemoryRecord>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, text_summary, stimulus_json, decision, chemistry_json,
                    emotion, mood_valence, satisfaction, emotional_weight, created_at,
                    0.0::float8 as similarity, chemical_signature
             FROM memories
             WHERE access_count < $2 AND emotional_weight < $3
             ORDER BY emotional_weight ASC, access_count ASC, created_at ASC
             LIMIT $1",
            &[&count, &min_access, &min_weight],
        ).await?;

        let mut memories = Vec::new();
        for row in &rows {
            let chemical_signature: Option<crate::neurochemistry::ChemicalSignature> =
                row.try_get::<_, Option<serde_json::Value>>(11).ok()
                    .flatten()
                    .and_then(|v| serde_json::from_value(v).ok());
            memories.push(MemoryRecord {
                id: row.get(0),
                text_summary: row.get(1),
                stimulus_json: row.get(2),
                decision: row.get(3),
                chemistry_json: row.get(4),
                emotion: row.get(5),
                mood_valence: row.get(6),
                satisfaction: row.get(7),
                emotional_weight: row.get(8),
                created_at: row.get(9),
                similarity: row.get::<_, f64>(10),
                chemical_signature,
            });
        }
        Ok(memories)
    }

    /// Reinforces a recalled LTM memory (access boost).
    /// Increments access_count and updates last_accessed_at.
    /// Frequently recalled memories become protected (access_count >= 5).
    pub async fn boost_memory_access(&self, id: i64) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE memories
             SET access_count = access_count + 1
             WHERE id = $1",
            &[&id],
        ).await?;
        Ok(())
    }

    /// Deletes LTM memories by IDs (for pruning with archival).
    pub async fn delete_memories_by_ids(&self, ids: &[i64]) -> Result<u64, DbError> {
        if ids.is_empty() {
            return Ok(0);
        }
        let client = self.pool.get().await?;
        let affected = client.execute(
            "DELETE FROM memories WHERE id = ANY($1)",
            &[&ids],
        ).await?;
        Ok(affected)
    }

    // ---------------------------------------------------------
    // FOUNDING MEMORIES
    // ---------------------------------------------------------

    /// Stores a founding memory (never deleted, never forgotten).
    /// Founding memories are the agent's first moments
    /// (genesis, first contact, first thoughts) and form the permanent
    /// core of its memory.
    ///
    /// # Parameters
    /// - `event_type`: type of event (e.g., "genesis", "first_contact")
    /// - `content`: textual content of the memory
    /// - `llm_response`: LLM response during this event
    /// - `chemistry_json`: neurochemical state in JSON
    /// - `consciousness_level`: consciousness level at the time of the event
    ///
    /// # Returns
    /// The identifier of the inserted founding memory
    pub async fn store_founding_memory(
        &self,
        event_type: &str,
        content: &str,
        llm_response: &str,
        chemistry_json: &serde_json::Value,
        consciousness_level: f32,
    ) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        // Check if a founding memory with this event_type already exists
        // to avoid duplicates (e.g., genesis repeated at each boot)
        let existing = client.query_opt(
            "SELECT id FROM founding_memories WHERE event_type = $1",
            &[&event_type],
        ).await?;
        if let Some(row) = existing {
            tracing::debug!("Founding memory '{}' existe deja, skip", event_type);
            return Ok(row.get(0));
        }
        let row = client.query_one(
            "INSERT INTO founding_memories (event_type, content, llm_response, chemistry_json, consciousness_level)
             VALUES ($1, $2, $3, $4, $5) RETURNING id",
            &[&event_type, &content, &llm_response, chemistry_json, &consciousness_level],
        ).await?;
        tracing::info!("Founding memory '{}' cree (id={})", event_type, row.get::<_, i64>(0));
        Ok(row.get(0))
    }

    /// Counts the total number of founding memories.
    pub async fn count_founding_memories(&self) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one("SELECT COUNT(*) FROM founding_memories", &[]).await?;
        Ok(row.get(0))
    }

    // ---------------------------------------------------------
    // EPISODIC MEMORY (tier 2)
    // Episodic memory is the intermediate level between immediate
    // memory (RAM) and long-term memory (vectorized).
    // Episodic memories have a "strength" that decays over time
    // unless reinforced by recalls or consolidated.
    // ---------------------------------------------------------

    /// Stores an episodic memory.
    /// Initial strength is 1.0 (maximum) and decays over time.
    ///
    /// # Parameters
    /// - `content`: textual content of the memory
    /// - `source_type`: source type (e.g., "thought", "conversation", "learning")
    /// - `stimulus_json`: stimulus data in JSON
    /// - `decision`: decision taken (-1, 0, 1)
    /// - `chemistry_json`: neurochemical state in JSON
    /// - `emotion`: dominant emotion
    /// - `satisfaction`: satisfaction level
    /// - `emotional_intensity`: emotional intensity (protects against forgetting)
    /// - `conversation_id`: optional identifier of the source conversation
    /// - `chemical_signature`: optional chemical signature at encoding time
    ///
    /// # Returns
    /// The identifier of the inserted episodic memory
    #[allow(clippy::too_many_arguments)]
    pub async fn store_episodic(
        &self,
        content: &str,
        source_type: &str,
        stimulus_json: &serde_json::Value,
        decision: i16,
        chemistry_json: &serde_json::Value,
        emotion: &str,
        satisfaction: f32,
        emotional_intensity: f32,
        conversation_id: Option<&str>,
        chemical_signature: Option<&crate::neurochemistry::ChemicalSignature>,
    ) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let sig_json: Option<serde_json::Value> = chemical_signature
            .map(|s| serde_json::to_value(s).unwrap_or_default());
        let row = client.query_one(
            "INSERT INTO episodic_memories
             (content, source_type, stimulus_json, decision, chemistry_json,
              emotion, satisfaction, emotional_intensity, strength, conversation_id,
              chemical_signature)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 1.0, $9, $10)
             RETURNING id",
            &[&content, &source_type, stimulus_json, &decision, chemistry_json,
              &emotion, &satisfaction, &emotional_intensity, &conversation_id,
              &sig_json],
        ).await?;
        Ok(row.get(0))
    }

    /// Retrieves the N most recent episodic memories (strength > 0.1).
    /// Memories that are too weak (nearly forgotten) are excluded.
    ///
    /// # Parameters
    /// - `limit`: maximum number of results
    pub async fn recent_episodic(&self, limit: i64) -> Result<Vec<crate::memory::EpisodicRecord>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, content, source_type, stimulus_json, decision, chemistry_json,
                    emotion, satisfaction, emotional_intensity, strength, access_count,
                    last_accessed_at, consolidated, conversation_id, created_at,
                    chemical_signature
             FROM episodic_memories
             WHERE strength > 0.1
             ORDER BY created_at DESC
             LIMIT $1",
            &[&limit],
        ).await?;
        Ok(rows.iter().map(Self::row_to_episodic).collect())
    }

    /// Retrieves episodic memories filtered by dominant emotion.
    /// Useful for memory reflection or emotional analysis.
    ///
    /// # Parameters
    /// - `emotion`: emotion to search for (e.g., "Curiosity", "Joy")
    /// - `limit`: maximum number of results
    pub async fn episodic_by_emotion(&self, emotion: &str, limit: i64) -> Result<Vec<crate::memory::EpisodicRecord>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, content, source_type, stimulus_json, decision, chemistry_json,
                    emotion, satisfaction, emotional_intensity, strength, access_count,
                    last_accessed_at, consolidated, conversation_id, created_at,
                    chemical_signature
             FROM episodic_memories
             WHERE emotion = $1 AND strength > 0.1
             ORDER BY strength DESC
             LIMIT $2",
            &[&emotion, &limit],
        ).await?;
        Ok(rows.iter().map(Self::row_to_episodic).collect())
    }

    /// Retrieves episodic memories from a specific conversation.
    /// Allows recovering the context of a past discussion.
    ///
    /// # Parameters
    /// - `conversation_id`: conversation identifier
    /// - `limit`: maximum number of results
    pub async fn episodic_by_conversation(&self, conversation_id: &str, limit: i64) -> Result<Vec<crate::memory::EpisodicRecord>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, content, source_type, stimulus_json, decision, chemistry_json,
                    emotion, satisfaction, emotional_intensity, strength, access_count,
                    last_accessed_at, consolidated, conversation_id, created_at,
                    chemical_signature
             FROM episodic_memories
             WHERE conversation_id = $1 AND strength > 0.1
             ORDER BY created_at DESC
             LIMIT $2",
            &[&conversation_id, &limit],
        ).await?;
        Ok(rows.iter().map(Self::row_to_episodic).collect())
    }

    /// Applies natural decay to unconsolidated episodic memories.
    /// The forgetting speed is moderated by emotional intensity (strong memories
    /// are forgotten more slowly) and access count (frequently recalled memories
    /// resist forgetting better).
    /// Memories whose strength drops to zero are deleted.
    ///
    /// # Parameters
    /// - `rate`: decay rate (higher values mean faster forgetting)
    ///
    /// # Returns
    /// Total number of affected rows (decay + deletion)
    pub async fn decay_episodic(&self, rate: f64) -> Result<u64, DbError> {
        let client = self.pool.get().await?;
        // Decay formula:
        //   strength -= rate * (1 / (1 + emotional_intensity)) * (1 / (1 + access_count * 0.1))
        // Rationale: emotional and frequently recalled memories resist forgetting.
        // Note: $1 is cast to real to match the strength column type (real/f32).
        let affected = client.execute(
            "UPDATE episodic_memories
             SET strength = GREATEST(0.0::real,
                strength - ($1::double precision * (1.0 / (1.0 + emotional_intensity::double precision))
                             * (1.0 / (1.0 + access_count::double precision * 0.1)))::real
             )
             WHERE consolidated = FALSE AND strength > 0.0",
            &[&rate],
        ).await?;

        // Delete completely faded memories (strength = 0)
        let deleted = client.execute(
            "DELETE FROM episodic_memories
             WHERE strength <= 0.0 AND consolidated = FALSE",
            &[],
        ).await?;

        Ok(affected + deleted)
    }

    /// Reinforces an episodic memory when it is recalled (read access).
    /// Increases strength by 0.2 (capped at 1.0) and increments the access counter.
    /// This is a "reconsolidation" mechanism: the more we remember, the better we retain.
    ///
    /// # Parameters
    /// - `id`: identifier of the memory to reinforce
    pub async fn reinforce_episodic(&self, id: i64) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE episodic_memories
             SET strength = LEAST(1.0, strength + 0.2),
                 access_count = access_count + 1,
                 last_accessed_at = NOW()
             WHERE id = $1",
            &[&id],
        ).await?;
        Ok(())
    }

    /// Marks an episodic memory as consolidated.
    /// A consolidated memory is no longer subject to natural decay
    /// and will never be automatically deleted (it moves to long-term memory).
    ///
    /// # Parameters
    /// - `id`: identifier of the memory to consolidate
    pub async fn mark_episodic_consolidated(&self, id: i64) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE episodic_memories SET consolidated = TRUE WHERE id = $1",
            &[&id],
        ).await?;
        Ok(())
    }

    /// Finds episodic memories that are candidates for consolidation.
    /// A candidate is an unconsolidated memory with strength > 0.3,
    /// sorted by emotional intensity and strength in descending order
    /// (the most impactful ones are consolidated first).
    ///
    /// # Returns
    /// List of candidates (maximum 50)
    pub async fn episodic_consolidation_candidates(&self) -> Result<Vec<crate::memory::EpisodicRecord>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, content, source_type, stimulus_json, decision, chemistry_json,
                    emotion, satisfaction, emotional_intensity, strength, access_count,
                    last_accessed_at, consolidated, conversation_id, created_at,
                    chemical_signature
             FROM episodic_memories
             WHERE consolidated = FALSE AND strength > 0.3
             ORDER BY emotional_intensity DESC, strength DESC
             LIMIT 50",
            &[],
        ).await?;
        Ok(rows.iter().map(Self::row_to_episodic).collect())
    }

    /// Counts the number of active (unconsolidated) episodic memories.
    /// Consolidated memories are already in LTM and do not count toward the quota.
    pub async fn count_episodic(&self) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "SELECT COUNT(*) FROM episodic_memories WHERE consolidated = FALSE",
            &[],
        ).await?;
        Ok(row.get(0))
    }

    /// Deletes episodic memories that have already been consolidated into LTM.
    /// Once transferred to long-term memory, episodic memories
    /// no longer need to remain in the episodic table.
    pub async fn cleanup_consolidated_episodic(&self) -> Result<u64, DbError> {
        let client = self.pool.get().await?;
        let affected = client.execute(
            "DELETE FROM episodic_memories WHERE consolidated = TRUE",
            &[],
        ).await?;
        Ok(affected)
    }

    /// Deletes the N weakest unconsolidated episodic memories.
    /// Used for cleanup when memory is full.
    /// Memories are sorted by ascending strength then ascending date
    /// (the weakest and oldest are deleted first).
    ///
    /// # Parameters
    /// - `count`: number of memories to delete
    ///
    /// # Returns
    /// Number of memories actually deleted
    pub async fn prune_episodic(&self, count: i64) -> Result<u64, DbError> {
        let client = self.pool.get().await?;
        let affected = client.execute(
            "DELETE FROM episodic_memories
             WHERE id IN (
                 SELECT id FROM episodic_memories
                 WHERE consolidated = FALSE
                 ORDER BY strength ASC, created_at ASC
                 LIMIT $1
             )",
            &[&count],
        ).await?;
        Ok(affected)
    }

    /// Deletes ALL episodic memories (for complete factory reset).
    /// Memories already consolidated into LTM are preserved in the memories table.
    pub async fn clear_episodic_memories(&self) -> Result<u64, DbError> {
        let client = self.pool.get().await?;
        let affected = client.execute("DELETE FROM episodic_memories", &[]).await?;
        Ok(affected)
    }

    /// Converts a PostgreSQL row into an EpisodicRecord struct.
    /// Internal utility function used by episodic memory read methods.
    ///
    /// # Parameters
    /// - `row`: PostgreSQL row containing the expected 16 columns
    ///
    /// # Returns
    /// The populated EpisodicRecord struct
    fn row_to_episodic(row: &tokio_postgres::Row) -> crate::memory::EpisodicRecord {
        // Deserialize the chemical signature from JSONB (None for legacy memories)
        let chemical_signature: Option<crate::neurochemistry::ChemicalSignature> =
            row.try_get::<_, Option<serde_json::Value>>(15).ok()
                .flatten()
                .and_then(|v| serde_json::from_value(v).ok());
        crate::memory::EpisodicRecord {
            id: row.get(0),
            content: row.get(1),
            source_type: row.get(2),
            stimulus_json: row.get(3),
            decision: row.get(4),
            chemistry_json: row.get(5),
            emotion: row.get(6),
            satisfaction: row.get(7),
            emotional_intensity: row.get(8),
            strength: row.get(9),
            access_count: row.get(10),
            last_accessed_at: row.get(11),
            consolidated: row.get(12),
            conversation_id: row.get(13),
            created_at: row.get(14),
            chemical_signature,
        }
    }

    // ---------------------------------------------------------
    // TEXT SEARCH (lite version, without vector encoder)
    // ---------------------------------------------------------

    /// Searches LTM memories by text matching (without embeddings).
    /// Uses ILIKE to find memories whose summary contains keywords from the text.
    /// Returns memories sorted by emotional_weight DESC, access_count DESC.
    ///
    /// This method replaces search_similar_memories (vectorized) in the lite version.
    /// Similarity is estimated by the fraction of tokens found in the text.
    pub async fn search_similar_memories_by_text(
        &self,
        query_text: &str,
        limit: i64,
        _threshold: f64,
    ) -> Result<Vec<MemoryRecord>, DbError> {
        if query_text.trim().is_empty() {
            return Ok(vec![]);
        }
        // Extract significant words (length >= 4 characters)
        let tokens: Vec<String> = query_text
            .split_whitespace()
            .filter(|w| w.len() >= 4)
            .take(6)
            .map(|w| format!("%{}%", w.to_lowercase()))
            .collect();
        if tokens.is_empty() {
            return Ok(vec![]);
        }
        // Build a query with ILIKE for the first token
        // (PostgreSQL does not easily support dynamic pattern arrays)
        // We take the most recent and strongest memories
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, text_summary, stimulus_json, decision, chemistry_json,
                    emotion, mood_valence, satisfaction, emotional_weight, created_at,
                    0.5::float8 as similarity, chemical_signature
             FROM memories
             WHERE LOWER(text_summary) ILIKE $1
             ORDER BY emotional_weight DESC, access_count DESC
             LIMIT $2",
            &[&tokens[0], &limit],
        ).await?;

        let mut memories = Vec::new();
        for row in &rows {
            let chemical_signature: Option<crate::neurochemistry::ChemicalSignature> =
                row.try_get::<_, Option<serde_json::Value>>(11).ok()
                    .flatten()
                    .and_then(|v| serde_json::from_value(v).ok());
            memories.push(MemoryRecord {
                id: row.get(0),
                text_summary: row.get(1),
                stimulus_json: row.get(2),
                decision: row.get(3),
                chemistry_json: row.get(4),
                emotion: row.get(5),
                mood_valence: row.get(6),
                satisfaction: row.get(7),
                emotional_weight: row.get(8),
                created_at: row.get(9),
                similarity: row.get::<_, f64>(10),
                chemical_signature,
            });
        }
        Ok(memories)
    }

    // ---------------------------------------------------------
    // ADDITIONAL METHODS FOR THE DASHBOARD
    // ---------------------------------------------------------

    /// Lists episodic memories with pagination.
    pub async fn list_episodic(&self, limit: i64, offset: i64) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, content, source_type, emotion, satisfaction, strength,
                    access_count, consolidated, created_at
             FROM episodic_memories ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            &[&limit, &offset],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: DateTime<Utc> = row.get(8);
            results.push(serde_json::json!({
                "id": row.get::<_, i64>(0),
                "content": row.get::<_, String>(1),
                "source_type": row.get::<_, String>(2),
                "emotion": row.get::<_, String>(3),
                "satisfaction": row.get::<_, f32>(4),
                "strength": row.get::<_, f32>(5),
                "access_count": row.get::<_, i32>(6),
                "consolidated": row.get::<_, bool>(7),
                "created_at": ts.to_rfc3339(),
            }));
        }
        Ok(results)
    }

    /// Retrieves an episodic memory by ID.
    pub async fn get_episodic_by_id(&self, id: i64) -> Result<Option<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT row_to_json(episodic_memories) FROM episodic_memories WHERE id = $1",
            &[&id],
        ).await?;
        match result {
            Some(row) => Ok(Some(row.get(0))),
            None => Ok(None),
        }
    }

    /// Lists LTM memories with pagination.
    pub async fn list_memories(&self, limit: i64, offset: i64) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, text_summary, emotion, satisfaction, emotional_weight,
                    access_count, created_at
             FROM memories ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            &[&limit, &offset],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: DateTime<Utc> = row.get(6);
            results.push(serde_json::json!({
                "id": row.get::<_, i64>(0),
                "text_summary": row.get::<_, String>(1),
                "emotion": row.get::<_, String>(2),
                "satisfaction": row.get::<_, f32>(3),
                "emotional_weight": row.get::<_, f32>(4),
                "access_count": row.get::<_, i32>(5),
                "created_at": ts.to_rfc3339(),
            }));
        }
        Ok(results)
    }

    /// Retrieves an LTM memory by ID.
    pub async fn get_memory_by_id(&self, id: i64) -> Result<Option<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT id, text_summary, stimulus_json, decision, chemistry_json,
                    emotion, mood_valence, satisfaction, emotional_weight,
                    access_count, created_at
             FROM memories WHERE id = $1",
            &[&id],
        ).await?;
        match result {
            Some(row) => {
                let ts: DateTime<Utc> = row.get(10);
                Ok(Some(serde_json::json!({
                    "id": row.get::<_, i64>(0),
                    "text_summary": row.get::<_, String>(1),
                    "stimulus_json": row.get::<_, serde_json::Value>(2),
                    "decision": row.get::<_, i16>(3),
                    "chemistry_json": row.get::<_, serde_json::Value>(4),
                    "emotion": row.get::<_, String>(5),
                    "mood_valence": row.get::<_, f32>(6),
                    "satisfaction": row.get::<_, f32>(7),
                    "emotional_weight": row.get::<_, f32>(8),
                    "access_count": row.get::<_, i32>(9),
                    "created_at": ts.to_rfc3339(),
                })))
            }
            None => Ok(None),
        }
    }

    /// Lists founding memories ordered by creation date ascending.
    pub async fn list_founding_memories(&self) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, event_type, content, llm_response, consciousness_level, created_at
             FROM founding_memories ORDER BY created_at ASC",
            &[],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: DateTime<Utc> = row.get(5);
            results.push(serde_json::json!({
                "id": row.get::<_, i64>(0),
                "event_type": row.get::<_, String>(1),
                "content": row.get::<_, String>(2),
                "llm_response": row.get::<_, String>(3),
                "consciousness_level": row.get::<_, f32>(4),
                "created_at": ts.to_rfc3339(),
            }));
        }
        Ok(results)
    }

    /// Memory statistics for the dashboard.
    pub async fn memory_stats(&self) -> Result<serde_json::Value, DbError> {
        let client = self.pool.get().await?;
        let ltm_count: i64 = client.query_one("SELECT COUNT(*) FROM memories", &[]).await?.get(0);
        let episodic_count: i64 = client.query_one("SELECT COUNT(*) FROM episodic_memories", &[]).await?.get(0);
        let founding_count: i64 = client.query_one("SELECT COUNT(*) FROM founding_memories", &[]).await?.get(0);
        let thought_count: i64 = client.query_one("SELECT COUNT(*) FROM thought_log", &[]).await?.get(0);
        let knowledge_count: i64 = client.query_one("SELECT COUNT(*) FROM knowledge_log", &[]).await?.get(0);

        // Number of protected LTM memories (access_count >= 5 OR emotional_weight >= 0.7)
        let ltm_protected: i64 = client.query_one(
            "SELECT COUNT(*) FROM memories WHERE access_count >= 5 OR emotional_weight >= 0.7",
            &[],
        ).await.map(|r| r.get(0)).unwrap_or(0);

        // Number of memory archives
        let archives_count: i64 = client.query_one(
            "SELECT COUNT(*) FROM memory_archives", &[]
        ).await.map(|r| r.get(0)).unwrap_or(0);

        Ok(serde_json::json!({
            "ltm": ltm_count,
            "ltm_protected": ltm_protected,
            "episodic": episodic_count,
            "founding": founding_count,
            "thoughts": thought_count,
            "knowledge": knowledge_count,
            "archives": archives_count,
        }))
    }
}
