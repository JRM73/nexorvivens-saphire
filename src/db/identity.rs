// =============================================================================
// db/identity.rs — Identity, sessions, virtual body, vital state, senses
//
// Purpose: Persistence layer for the agent's identity and all subsystem states.
// This file manages the singleton self_identity record (id=1) which stores:
//   - Core identity (name, boot count, cycle count, self-description)
//   - Virtual body state (body_json)
//   - Vital state: spark, intuition, premonition (vital_json)
//   - Sensory state / Sensorium (senses_json)
//   - Micro neural network state (nn_json)
//   - Relationship network (relationships_json)
//   - Metacognition: thought quality + Turing score (metacognition_json)
//   - Nutritional state (nutrition_json)
//   - Grey matter / cerebral substrate (grey_matter_json)
//   - Electromagnetic fields (fields_json)
//   - Psychology state (psychology_state)
//
// Also handles session logging, sleep history, and temporal personality
// portraits (snapshots, emotional trajectory, consciousness history,
// psychology checkpoints, relationship timeline, introspection journal).
// =============================================================================

use super::{SaphireDb, DbError};

impl SaphireDb {
    // ---------------------------------------------------------
    // IDENTITY
    // ---------------------------------------------------------

    /// Saves the agent's identity (upsert singleton, id=1).
    /// If the identity already exists, it is updated. Otherwise, it is created.
    /// The identity contains the name, boot count, cycle count,
    /// a self-description, and the dominant tendency.
    ///
    /// # Parameters
    /// - `identity_json`: JSON object containing the identity fields
    pub async fn save_identity(&self, identity_json: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;

        let name = identity_json["name"].as_str().unwrap_or("Saphire");
        let total_cycles = identity_json["total_cycles"].as_i64().unwrap_or(0);
        let total_boots = identity_json["total_boots"].as_i64().unwrap_or(1) as i32;
        let self_desc = identity_json["self_description"].as_str().unwrap_or("");
        let dominant = identity_json["dominant_tendency"].as_str().unwrap_or("neocortex");
        let dominant_emotion = identity_json["dominant_emotion"].as_str().unwrap_or("Curiosité");
        let human_conversations = identity_json["human_conversations"].as_i64().unwrap_or(0);
        let autonomous_thoughts = identity_json["autonomous_thoughts"].as_i64().unwrap_or(0);
        let interests = serde_json::Value::Array(
            identity_json["interests"].as_array().cloned().unwrap_or_default()
        );
        let core_values = serde_json::Value::Array(
            identity_json["core_values"].as_array().cloned().unwrap_or_default()
        );

        // Try updating the existing singleton first (UPDATE)
        let updated = client.execute(
            "UPDATE self_identity SET
                name = $1, total_cycles = $2, total_boots = $3,
                self_description = $4, dominant_tendency = $5,
                dominant_emotion = $6, human_conversations = $7,
                autonomous_thoughts = $8, interests = $9, core_values = $10,
                updated_at = NOW()
             WHERE id = 1",
            &[&name, &total_cycles, &total_boots, &self_desc, &dominant,
              &dominant_emotion, &human_conversations, &autonomous_thoughts,
              &interests, &core_values],
        ).await?;

        // If no row was updated, this is the first boot: insert
        if updated == 0 {
            client.execute(
                "INSERT INTO self_identity (id, name, born_at, total_boots, total_cycles,
                    self_description, dominant_tendency, dominant_emotion,
                    human_conversations, autonomous_thoughts, interests, core_values, updated_at)
                 VALUES (1, $1, NOW(), $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())",
                &[&name, &total_boots, &total_cycles, &self_desc, &dominant,
                  &dominant_emotion, &human_conversations, &autonomous_thoughts,
                  &interests, &core_values],
            ).await?;
            tracing::info!("Identité créée : {} (émotion: {})", name, dominant_emotion);
        }
        Ok(())
    }

    /// Loads the agent's identity from the database.
    ///
    /// # Returns
    /// - `Some(Value)`: the identity as JSON if it exists
    /// - `None`: if this is the first boot (no identity in database)
    pub async fn load_identity(&self) -> Result<Option<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT row_to_json(self_identity) FROM self_identity WHERE id = 1",
            &[],
        ).await?;
        match result {
            Some(row) => {
                let json: serde_json::Value = row.get(0);
                Ok(Some(json))
            },
            None => Ok(None),
        }
    }

    /// Updates the clean_shutdown flag in the identity.
    /// Allows detecting abrupt shutdowns on next startup.
    ///
    /// # Parameters
    /// - `clean`: true if the shutdown is clean, false at startup (before shutdown)
    pub async fn set_clean_shutdown(&self, clean: bool) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE self_identity SET clean_shutdown = $1, updated_at = NOW() WHERE id = 1",
            &[&clean],
        ).await?;
        Ok(())
    }

    /// Checks whether the last agent shutdown was clean.
    /// Used to detect crashes and adapt behavior at restart.
    ///
    /// # Returns
    /// `true` if the last shutdown was clean (or if this is the first boot)
    pub async fn last_shutdown_clean(&self) -> Result<bool, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT clean_shutdown FROM self_identity WHERE id = 1",
            &[],
        ).await?;
        match result {
            Some(row) => Ok(row.get(0)),
            None => Ok(true), // No identity = first boot, considered as clean
        }
    }

    // ---------------------------------------------------------
    // VIRTUAL BODY
    // ---------------------------------------------------------

    /// Saves the virtual body state to self_identity.body_json.
    pub async fn save_body_state(&self, body_json: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        // The body_json column is added by migration (ALTER TABLE).
        // If the column does not exist yet, the error is silently ignored.
        let result = client.execute(
            "UPDATE self_identity SET body_json = $1, updated_at = NOW() WHERE id = 1",
            &[body_json],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_body_state: {} (colonne body_json peut-etre absente)", e);
        }
        Ok(())
    }

    /// Loads the virtual body state from self_identity.body_json.
    pub async fn load_body_state(&self) -> Result<Option<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT body_json FROM self_identity WHERE id = 1",
            &[],
        ).await;
        match result {
            Ok(Some(row)) => {
                let json: Option<serde_json::Value> = row.get(0);
                Ok(json)
            },
            Ok(None) => Ok(None),
            Err(e) => {
                // The body_json column may not exist if the migration has not been run
                tracing::warn!("load_body_state: {} (colonne body_json peut-etre absente)", e);
                Ok(None)
            }
        }
    }

    // ---------------------------------------------------------
    // VITAL (spark + intuition + premonition)
    // ---------------------------------------------------------

    /// Saves the vital state (spark + intuition + premonition) to self_identity.vital_json.
    pub async fn save_vital_state(&self, vital_json: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "UPDATE self_identity SET vital_json = $1, updated_at = NOW() WHERE id = 1",
            &[vital_json],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_vital_state: {} (colonne vital_json peut-etre absente)", e);
        }
        Ok(())
    }

    /// Loads the vital state from self_identity.vital_json.
    pub async fn load_vital_state(&self) -> Result<Option<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT vital_json FROM self_identity WHERE id = 1",
            &[],
        ).await;
        match result {
            Ok(Some(row)) => {
                let json: Option<serde_json::Value> = row.get(0);
                Ok(json)
            },
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!("load_vital_state: {} (colonne vital_json peut-etre absente)", e);
                Ok(None)
            }
        }
    }

    // ---------------------------------------------------------
    // SENSES (Sensorium)
    // ---------------------------------------------------------

    /// Saves the sensory state (Sensorium) to self_identity.senses_json.
    pub async fn save_senses_state(&self, senses_json: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "UPDATE self_identity SET senses_json = $1, updated_at = NOW() WHERE id = 1",
            &[senses_json],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_senses_state: {} (colonne senses_json peut-etre absente)", e);
        }
        Ok(())
    }

    /// Loads the sensory state from self_identity.senses_json.
    pub async fn load_senses_state(&self) -> Result<Option<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT senses_json FROM self_identity WHERE id = 1",
            &[],
        ).await;
        match result {
            Ok(Some(row)) => {
                let json: Option<serde_json::Value> = row.get(0);
                Ok(json)
            },
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!("load_senses_state: {} (colonne senses_json peut-etre absente)", e);
                Ok(None)
            }
        }
    }

    // ---------------------------------------------------------
    // MICRO NEURAL NETWORK
    // ---------------------------------------------------------

    /// Saves the micro neural network state to self_identity.nn_json.
    pub async fn save_nn_state(&self, nn_json: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "UPDATE self_identity SET nn_json = $1, updated_at = NOW() WHERE id = 1",
            &[nn_json],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_nn_state: {} (colonne nn_json peut-etre absente)", e);
        }
        Ok(())
    }

    /// Loads the micro neural network state from self_identity.nn_json.
    pub async fn load_nn_state(&self) -> Result<Option<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT nn_json FROM self_identity WHERE id = 1",
            &[],
        ).await;
        match result {
            Ok(Some(row)) => {
                let json: Option<serde_json::Value> = row.get(0);
                Ok(json)
            },
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!("load_nn_state: {} (colonne nn_json peut-etre absente)", e);
                Ok(None)
            }
        }
    }

    // ---------------------------------------------------------
    // RELATIONSHIPS (affective bond network)
    // ---------------------------------------------------------

    /// Saves the relationship network state to self_identity.relationships_json.
    pub async fn save_relationships_state(&self, json: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "UPDATE self_identity SET relationships_json = $1, updated_at = NOW() WHERE id = 1",
            &[json],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_relationships_state: {} (colonne relationships_json peut-etre absente)", e);
        }
        Ok(())
    }

    /// Loads the relationship network state from self_identity.relationships_json.
    pub async fn load_relationships_state(&self) -> Result<Option<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT relationships_json FROM self_identity WHERE id = 1",
            &[],
        ).await;
        match result {
            Ok(Some(row)) => {
                let json: Option<serde_json::Value> = row.get(0);
                Ok(json)
            },
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!("load_relationships_state: {} (colonne relationships_json peut-etre absente)", e);
                Ok(None)
            }
        }
    }

    // ---------------------------------------------------------
    // METACOGNITION (thought quality + Turing score)
    // ---------------------------------------------------------

    /// Saves the metacognitive state to self_identity.metacognition_json.
    pub async fn save_metacognition_state(&self, json: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "UPDATE self_identity SET metacognition_json = $1, updated_at = NOW() WHERE id = 1",
            &[json],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_metacognition_state: {} (colonne metacognition_json peut-etre absente)", e);
        }
        Ok(())
    }

    /// Loads the metacognitive state from self_identity.metacognition_json.
    pub async fn load_metacognition_state(&self) -> Result<Option<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT metacognition_json FROM self_identity WHERE id = 1",
            &[],
        ).await;
        match result {
            Ok(Some(row)) => {
                let json: Option<serde_json::Value> = row.get(0);
                Ok(json)
            },
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!("load_metacognition_state: {} (colonne metacognition_json peut-etre absente)", e);
                Ok(None)
            }
        }
    }

    // ---------------------------------------------------------
    // NUTRITION (nutritional system)
    // ---------------------------------------------------------

    /// Saves the nutritional state to self_identity.nutrition_json.
    pub async fn save_nutrition_state(&self, json: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "UPDATE self_identity SET nutrition_json = $1, updated_at = NOW() WHERE id = 1",
            &[json],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_nutrition_state: {} (colonne nutrition_json peut-etre absente)", e);
        }
        Ok(())
    }

    /// Loads the nutritional state from self_identity.nutrition_json.
    pub async fn load_nutrition_state(&self) -> Result<Option<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT nutrition_json FROM self_identity WHERE id = 1",
            &[],
        ).await;
        match result {
            Ok(Some(row)) => {
                let json: Option<serde_json::Value> = row.get(0);
                Ok(json)
            },
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!("load_nutrition_state: {} (colonne nutrition_json peut-etre absente)", e);
                Ok(None)
            }
        }
    }

    // ---------------------------------------------------------
    // GREY MATTER (physical cerebral substrate)
    // ---------------------------------------------------------

    /// Saves the grey matter state to self_identity.grey_matter_json.
    pub async fn save_grey_matter_state(&self, json: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "UPDATE self_identity SET grey_matter_json = $1, updated_at = NOW() WHERE id = 1",
            &[json],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_grey_matter_state: {} (colonne grey_matter_json peut-etre absente)", e);
        }
        Ok(())
    }

    /// Loads the grey matter state from self_identity.grey_matter_json.
    pub async fn load_grey_matter_state(&self) -> Result<Option<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT grey_matter_json FROM self_identity WHERE id = 1",
            &[],
        ).await;
        match result {
            Ok(Some(row)) => {
                let json: Option<serde_json::Value> = row.get(0);
                Ok(json)
            },
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!("load_grey_matter_state: {} (colonne grey_matter_json peut-etre absente)", e);
                Ok(None)
            }
        }
    }

    // ---------------------------------------------------------
    // FIELDS (electromagnetic fields)
    // ---------------------------------------------------------

    /// Saves the electromagnetic fields state to self_identity.fields_json.
    pub async fn save_fields_state(&self, json: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "UPDATE self_identity SET fields_json = $1, updated_at = NOW() WHERE id = 1",
            &[json],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_fields_state: {} (colonne fields_json peut-etre absente)", e);
        }
        Ok(())
    }

    /// Loads the electromagnetic fields state from self_identity.fields_json.
    pub async fn load_fields_state(&self) -> Result<Option<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT fields_json FROM self_identity WHERE id = 1",
            &[],
        ).await;
        match result {
            Ok(Some(row)) => {
                let json: Option<serde_json::Value> = row.get(0);
                Ok(json)
            },
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!("load_fields_state: {} (colonne fields_json peut-etre absente)", e);
                Ok(None)
            }
        }
    }

    // ---------------------------------------------------------
    // TEMPORAL PERSONALITY PORTRAIT (3 levels)
    // ---------------------------------------------------------

    /// Level 1: Saves a complete personality snapshot.
    pub async fn save_personality_snapshot(&self, snap: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "INSERT INTO personality_snapshots (
                cycle, boot_number,
                ocean_openness, ocean_conscientiousness, ocean_extraversion,
                ocean_agreeableness, ocean_neuroticism,
                dominant_emotion, mood_valence, mood_arousal,
                consciousness_level, phi,
                ego_strength, internal_conflict, shadow_integration,
                maslow_level, eq_score, willpower, toltec_overall,
                chemistry_json,
                sentiment_dominant, sentiment_count,
                connectome_nodes, connectome_edges, connectome_plasticity,
                turing_score, narrative_cohesion, monologue_coherence
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19,
                $20, $21, $22, $23, $24, $25, $26, $27, $28
             )",
            &[
                &snap["cycle"].as_i64().unwrap_or(0),
                &(snap["boot_number"].as_i64().unwrap_or(0) as i32),
                &(snap["ocean_openness"].as_f64().unwrap_or(0.5) as f32),
                &(snap["ocean_conscientiousness"].as_f64().unwrap_or(0.5) as f32),
                &(snap["ocean_extraversion"].as_f64().unwrap_or(0.5) as f32),
                &(snap["ocean_agreeableness"].as_f64().unwrap_or(0.5) as f32),
                &(snap["ocean_neuroticism"].as_f64().unwrap_or(0.5) as f32),
                &snap["dominant_emotion"].as_str().unwrap_or("Neutre"),
                &(snap["mood_valence"].as_f64().unwrap_or(0.0) as f32),
                &(snap["mood_arousal"].as_f64().unwrap_or(0.3) as f32),
                &(snap["consciousness_level"].as_f64().unwrap_or(0.3) as f32),
                &(snap["phi"].as_f64().unwrap_or(0.1) as f32),
                &(snap["ego_strength"].as_f64().unwrap_or(0.5) as f32),
                &(snap["internal_conflict"].as_f64().unwrap_or(0.0) as f32),
                &(snap["shadow_integration"].as_f64().unwrap_or(0.0) as f32),
                &(snap["maslow_level"].as_i64().unwrap_or(0) as i32),
                &(snap["eq_score"].as_f64().unwrap_or(0.0) as f32),
                &(snap["willpower"].as_f64().unwrap_or(1.0) as f32),
                &(snap["toltec_overall"].as_f64().unwrap_or(0.5) as f32),
                &snap["chemistry_json"],
                &snap["sentiment_dominant"].as_str(),
                &(snap["sentiment_count"].as_i64().unwrap_or(0) as i32),
                &(snap["connectome_nodes"].as_i64().unwrap_or(0) as i32),
                &(snap["connectome_edges"].as_i64().unwrap_or(0) as i32),
                &(snap["connectome_plasticity"].as_f64().unwrap_or(1.0) as f32),
                &(snap["turing_score"].as_f64().unwrap_or(0.0) as f32),
                &(snap["narrative_cohesion"].as_f64().unwrap_or(0.5) as f32),
                &(snap["monologue_coherence"].as_f64().unwrap_or(0.5) as f32),
            ],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_personality_snapshot: {}", e);
        }
        Ok(())
    }

    /// Level 1: Loads personality snapshots (most recent first).
    pub async fn load_personality_snapshots(&self, limit: i64) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT row_to_json(personality_snapshots) FROM personality_snapshots
             ORDER BY cycle DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results: Vec<serde_json::Value> = rows.iter()
            .map(|r| r.get(0))
            .collect();
        results.reverse();
        Ok(results)
    }

    /// Level 2a: Saves an emotional trajectory entry.
    pub async fn save_emotional_trajectory(&self, data: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "INSERT INTO emotional_trajectory (
                cycle, dominant_emotion, secondary_emotion, valence, arousal,
                spectrum_top5, sentiment_dominant, sentiment_strength, active_sentiments_json
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            &[
                &data["cycle"].as_i64().unwrap_or(0),
                &data["dominant_emotion"].as_str().unwrap_or("Neutre"),
                &data["secondary_emotion"].as_str(),
                &(data["valence"].as_f64().unwrap_or(0.0) as f32),
                &(data["arousal"].as_f64().unwrap_or(0.3) as f32),
                &data["spectrum_top5"],
                &data["sentiment_dominant"].as_str(),
                &data["sentiment_strength"].as_f64().map(|v| v as f32),
                &data["active_sentiments_json"],
            ],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_emotional_trajectory: {}", e);
        }
        Ok(())
    }

    /// Level 2a: Loads the emotional trajectory.
    pub async fn load_emotional_trajectory(&self, limit: i64) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT row_to_json(emotional_trajectory) FROM emotional_trajectory
             ORDER BY cycle DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results: Vec<serde_json::Value> = rows.iter()
            .map(|r| r.get(0))
            .collect();
        results.reverse();
        Ok(results)
    }

    /// Level 2b: Saves a consciousness history data point.
    pub async fn save_consciousness_history(&self, data: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "INSERT INTO consciousness_history (
                cycle, level, phi, coherence, continuity, existence_score, inner_narrative
             ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &data["cycle"].as_i64().unwrap_or(0),
                &(data["level"].as_f64().unwrap_or(0.3) as f32),
                &(data["phi"].as_f64().unwrap_or(0.1) as f32),
                &(data["coherence"].as_f64().unwrap_or(0.5) as f32),
                &(data["continuity"].as_f64().unwrap_or(0.5) as f32),
                &(data["existence_score"].as_f64().unwrap_or(0.0) as f32),
                &data["inner_narrative"].as_str(),
            ],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_consciousness_history: {}", e);
        }
        Ok(())
    }

    /// Level 2b: Loads the consciousness history.
    pub async fn load_consciousness_history(&self, limit: i64) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT row_to_json(consciousness_history) FROM consciousness_history
             ORDER BY cycle DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results: Vec<serde_json::Value> = rows.iter()
            .map(|r| r.get(0))
            .collect();
        results.reverse();
        Ok(results)
    }

    /// Level 2c: Saves a psychological checkpoint.
    pub async fn save_psychology_checkpoint(&self, data: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "INSERT INTO psychology_checkpoints (
                cycle, ego_strength, id_drive, superego_strength, internal_conflict,
                ego_anxiety, shadow_integration, dominant_archetype,
                maslow_level, maslow_satisfaction, toltec_json,
                eq_overall, eq_growth_experiences,
                flow_state, flow_total_cycles,
                willpower, decision_fatigue, total_deliberations
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)",
            &[
                &data["cycle"].as_i64().unwrap_or(0),
                &(data["ego_strength"].as_f64().unwrap_or(0.5) as f32),
                &(data["id_drive"].as_f64().unwrap_or(0.5) as f32),
                &(data["superego_strength"].as_f64().unwrap_or(0.5) as f32),
                &(data["internal_conflict"].as_f64().unwrap_or(0.0) as f32),
                &(data["ego_anxiety"].as_f64().unwrap_or(0.0) as f32),
                &(data["shadow_integration"].as_f64().unwrap_or(0.0) as f32),
                &data["dominant_archetype"].as_str(),
                &(data["maslow_level"].as_i64().unwrap_or(0) as i32),
                &(data["maslow_satisfaction"].as_f64().unwrap_or(0.0) as f32),
                &data["toltec_json"],
                &(data["eq_overall"].as_f64().unwrap_or(0.0) as f32),
                &data["eq_growth_experiences"].as_i64().unwrap_or(0),
                &data["flow_state"].as_str().unwrap_or("None"),
                &data["flow_total_cycles"].as_i64().unwrap_or(0),
                &(data["willpower"].as_f64().unwrap_or(1.0) as f32),
                &(data["decision_fatigue"].as_f64().unwrap_or(0.0) as f32),
                &data["total_deliberations"].as_i64().unwrap_or(0),
            ],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_psychology_checkpoint: {}", e);
        }
        Ok(())
    }

    /// Level 2c: Loads the psychological checkpoints.
    pub async fn load_psychology_checkpoints(&self, limit: i64) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT row_to_json(psychology_checkpoints) FROM psychology_checkpoints
             ORDER BY cycle DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results: Vec<serde_json::Value> = rows.iter()
            .map(|r| r.get(0))
            .collect();
        results.reverse();
        Ok(results)
    }

    /// Level 2d: Saves a relationship timeline data point.
    pub async fn save_relationship_timeline(&self, data: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "INSERT INTO relationship_timeline (
                cycle, person_name, bond_type, strength, trust, conflict_level, shared_memories
             ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &data["cycle"].as_i64().unwrap_or(0),
                &data["person_name"].as_str().unwrap_or("inconnu"),
                &data["bond_type"].as_str().unwrap_or("unknown"),
                &(data["strength"].as_f64().unwrap_or(0.0) as f32),
                &(data["trust"].as_f64().unwrap_or(0.0) as f32),
                &(data["conflict_level"].as_f64().unwrap_or(0.0) as f32),
                &(data["shared_memories"].as_i64().unwrap_or(0) as i32),
            ],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_relationship_timeline: {}", e);
        }
        Ok(())
    }

    /// Level 2d: Loads the relationship timeline.
    pub async fn load_relationship_timeline(&self, limit: i64) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT row_to_json(relationship_timeline) FROM relationship_timeline
             ORDER BY cycle DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results: Vec<serde_json::Value> = rows.iter()
            .map(|r| r.get(0))
            .collect();
        results.reverse();
        Ok(results)
    }

    /// Level 3: Saves an introspection journal entry.
    pub async fn save_journal_entry(&self, data: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "INSERT INTO introspection_journal (
                cycle, boot_number, entry_text, dominant_emotion,
                consciousness_level, turing_score, themes
             ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &data["cycle"].as_i64().unwrap_or(0),
                &(data["boot_number"].as_i64().unwrap_or(0) as i32),
                &data["entry_text"].as_str().unwrap_or(""),
                &data["dominant_emotion"].as_str().unwrap_or("Neutre"),
                &(data["consciousness_level"].as_f64().unwrap_or(0.3) as f32),
                &(data["turing_score"].as_f64().unwrap_or(0.0) as f32),
                &data["themes"],
            ],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_journal_entry: {}", e);
        }
        Ok(())
    }

    /// Level 3: Loads the introspection journal entries.
    pub async fn load_journal_entries(&self, limit: i64) -> Result<Vec<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT row_to_json(introspection_journal) FROM introspection_journal
             ORDER BY cycle DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results: Vec<serde_json::Value> = rows.iter()
            .map(|r| r.get(0))
            .collect();
        results.reverse();
        Ok(results)
    }

    // ---------------------------------------------------------
    // SLEEP HISTORY
    // ---------------------------------------------------------

    /// Saves a sleep record to the sleep_history table.
    pub async fn save_sleep_record(&self, record: &crate::sleep::SleepRecord) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let phases: Vec<String> = Vec::new(); // phases_completed non trackees dans SleepRecord
        let result = client.execute(
            "INSERT INTO sleep_history
                (started_at, ended_at, total_cycles, sleep_cycles_count, phases_completed,
                 dreams_count, memories_consolidated, connections_created, quality,
                 interrupted, interruption_reason)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            &[
                &record.started_at,
                &record.ended_at,
                &(record.duration_cycles as i32),
                &(record.sleep_cycles_completed as i32),
                &phases,
                &(record.dreams_count as i32),
                &(record.memories_consolidated as i32),
                &(record.connections_created as i32),
                &(record.quality as f32),
                &record.interrupted,
                &record.interruption_reason,
            ],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_sleep_record: {}", e);
        }
        Ok(())
    }

    /// Loads the most recent sleep records from the database.
    pub async fn load_sleep_history(&self, limit: i64) -> Result<Vec<crate::sleep::SleepRecord>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT started_at, ended_at, total_cycles, sleep_cycles_count,
                    dreams_count, memories_consolidated, connections_created,
                    quality, interrupted, interruption_reason
             FROM sleep_history ORDER BY ended_at DESC LIMIT $1",
            &[&limit],
        ).await?;

        let mut records = Vec::with_capacity(rows.len());
        for row in &rows {
            let started_at: chrono::DateTime<chrono::Utc> = row.get(0);
            let ended_at: chrono::DateTime<chrono::Utc> = row.get(1);
            let total_cycles: i32 = row.get(2);
            let sleep_cycles_count: i32 = row.get(3);
            let dreams_count: i32 = row.get(4);
            let memories_consolidated: i32 = row.get(5);
            let connections_created: i32 = row.get(6);
            let quality: f32 = row.get(7);
            let interrupted: bool = row.get(8);
            let interruption_reason: Option<String> = row.get(9);

            records.push(crate::sleep::SleepRecord {
                started_at,
                ended_at,
                duration_cycles: total_cycles as u64,
                sleep_cycles_completed: sleep_cycles_count as u8,
                quality: quality as f64,
                memories_consolidated: memories_consolidated as u64,
                connections_created: connections_created as u64,
                dreams_count: dreams_count as u64,
                interrupted,
                interruption_reason,
            });
        }
        // Reverse to get chronological order (oldest first)
        records.reverse();
        Ok(records)
    }

    // ---------------------------------------------------------
    // SESSIONS
    // ---------------------------------------------------------

    /// Records the start of a new session (an agent boot).
    ///
    /// # Parameters
    /// - `boot_number`: boot number (incremented at each boot)
    ///
    /// # Returns
    /// The identifier of the created session
    pub async fn start_session(&self, boot_number: i32) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "INSERT INTO session_log (boot_number) VALUES ($1) RETURNING id",
            &[&boot_number],
        ).await?;
        Ok(row.get(0))
    }

    /// Closes a session by recording the number of cycles and shutdown type.
    ///
    /// # Parameters
    /// - `session_id`: identifier of the session to close
    /// - `cycles`: number of cycles performed during this session
    /// - `clean`: true if the shutdown is clean (Ctrl+C signal), false if crash
    pub async fn end_session(&self, session_id: i64, cycles: i32, clean: bool) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE session_log SET ended_at = NOW(), cycles_this_session = $2, clean_shutdown = $3
             WHERE id = $1",
            &[&session_id, &cycles, &clean],
        ).await?;
        Ok(())
    }

    /// Saves the persistent psychological state (Toltec counters, shadow, EQ, flow).
    pub async fn save_psychology_state(&self, state_json: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "UPDATE self_identity SET psychology_state = $1, updated_at = NOW() WHERE id = 1",
            &[state_json],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_psychology_state: {} (colonne psychology_state peut-etre absente)", e);
        }
        Ok(())
    }

    /// Loads the persistent psychological state.
    pub async fn load_psychology_state(&self) -> Result<Option<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT psychology_state FROM self_identity WHERE id = 1",
            &[],
        ).await;
        match result {
            Ok(Some(row)) => {
                let json: Option<serde_json::Value> = row.get(0);
                Ok(json)
            },
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!("load_psychology_state: {} (colonne psychology_state peut-etre absente)", e);
                Ok(None)
            }
        }
    }
}
