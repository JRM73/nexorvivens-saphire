// =============================================================================
// db/identity.rs — Identite, sessions, corps virtuel, vital, senses
// =============================================================================

use super::{SaphireDb, DbError};

impl SaphireDb {
    // ---------------------------------------------------------
    // IDENTITE
    // ---------------------------------------------------------

    /// Sauvegarde l'identite de l'agent (upsert singleton, id=1).
    /// Si l'identite existe deja, elle est mise a jour. Sinon, elle est creee.
    /// L'identite contient le nom, le nombre de boots, le nombre de cycles,
    /// une auto-description et la tendance dominante.
    ///
    /// # Parametres
    /// - `identity_json` : objet JSON contenant les champs de l'identite
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

        // Essayer d'abord une mise a jour (UPDATE) du singleton existant
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

        // Si aucune ligne n'a ete mise a jour, c'est le premier boot : inserer
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

    /// Charge l'identite de l'agent depuis la base de donnees.
    ///
    /// # Retour
    /// - `Some(Value)` : l'identite en JSON si elle existe
    /// - `None` : si c'est le premier boot (pas d'identite en base)
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

    /// Met a jour le drapeau clean_shutdown dans l'identite.
    /// Permet de detecter les arrets brutaux au prochain demarrage.
    ///
    /// # Parametres
    /// - `clean` : true si l'arret est propre, false au demarrage (avant l'arret)
    pub async fn set_clean_shutdown(&self, clean: bool) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE self_identity SET clean_shutdown = $1, updated_at = NOW() WHERE id = 1",
            &[&clean],
        ).await?;
        Ok(())
    }

    /// Verifie si le dernier arret de l'agent etait propre.
    /// Utilise pour detecter les crashs et adapter le comportement au redemarrage.
    ///
    /// # Retour
    /// true si le dernier arret etait propre (ou si c'est le premier boot)
    pub async fn last_shutdown_clean(&self) -> Result<bool, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT clean_shutdown FROM self_identity WHERE id = 1",
            &[],
        ).await?;
        match result {
            Some(row) => Ok(row.get(0)),
            None => Ok(true), // Pas d'identite = premier boot, considere comme propre
        }
    }

    // ---------------------------------------------------------
    // CORPS VIRTUEL
    // ---------------------------------------------------------

    /// Sauvegarde l'etat du corps virtuel dans self_identity.body_json.
    pub async fn save_body_state(&self, body_json: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        // La colonne body_json est ajoutee par migration (ALTER TABLE).
        // Si la colonne n'existe pas encore, on ignore silencieusement l'erreur.
        let result = client.execute(
            "UPDATE self_identity SET body_json = $1, updated_at = NOW() WHERE id = 1",
            &[body_json],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_body_state: {} (colonne body_json peut-etre absente)", e);
        }
        Ok(())
    }

    /// Charge l'etat du corps virtuel depuis self_identity.body_json.
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
                // La colonne body_json peut ne pas exister si la migration n'a pas ete jouee
                tracing::warn!("load_body_state: {} (colonne body_json peut-etre absente)", e);
                Ok(None)
            }
        }
    }

    // ---------------------------------------------------------
    // VITAL (spark + intuition + premonition)
    // ---------------------------------------------------------

    /// Sauvegarde l'etat vital (spark + intuition + premonition) dans self_identity.vital_json.
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

    /// Charge l'etat vital depuis self_identity.vital_json.
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

    /// Sauvegarde l'etat sensoriel (Sensorium) dans self_identity.senses_json.
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

    /// Charge l'etat sensoriel depuis self_identity.senses_json.
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
    // MICRO RESEAU DE NEURONES
    // ---------------------------------------------------------

    /// Sauvegarde l'etat du micro-reseau de neurones dans self_identity.nn_json.
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

    /// Charge l'etat du micro-reseau de neurones depuis self_identity.nn_json.
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
    // RELATIONSHIPS (reseau de liens affectifs)
    // ---------------------------------------------------------

    /// Sauvegarde l'etat du reseau relationnel dans self_identity.relationships_json.
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

    /// Charge l'etat du reseau relationnel depuis self_identity.relationships_json.
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
    // METACOGNITION (qualite de pensee + Turing)
    // ---------------------------------------------------------

    /// Sauvegarde l'etat metacognitif dans self_identity.metacognition_json.
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

    /// Charge l'etat metacognitif depuis self_identity.metacognition_json.
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
    // NUTRITION (systeme nutritionnel)
    // ---------------------------------------------------------

    /// Sauvegarde l'etat nutritionnel dans self_identity.nutrition_json.
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

    /// Charge l'etat nutritionnel depuis self_identity.nutrition_json.
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
    // GREY MATTER (substrat cerebral physique)
    // ---------------------------------------------------------

    /// Sauvegarde l'etat de la matiere grise dans self_identity.grey_matter_json.
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

    /// Charge l'etat de la matiere grise depuis self_identity.grey_matter_json.
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
    // HORMONAL RECEPTORS (sensibilite des recepteurs)
    // ---------------------------------------------------------

    /// Sauvegarde l'etat des recepteurs hormonaux dans self_identity.hormonal_receptors_json.
    pub async fn save_hormonal_receptors_state(&self, json: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "UPDATE self_identity SET hormonal_receptors_json = $1, updated_at = NOW() WHERE id = 1",
            &[json],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_hormonal_receptors_state: {} (colonne hormonal_receptors_json peut-etre absente)", e);
        }
        Ok(())
    }

    /// Charge l'etat des recepteurs hormonaux depuis self_identity.hormonal_receptors_json.
    pub async fn load_hormonal_receptors_state(&self) -> Result<Option<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT hormonal_receptors_json FROM self_identity WHERE id = 1",
            &[],
        ).await;
        match result {
            Ok(Some(row)) => {
                let json: Option<serde_json::Value> = row.get(0);
                Ok(json)
            },
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!("load_hormonal_receptors_state: {} (colonne hormonal_receptors_json peut-etre absente)", e);
                Ok(None)
            }
        }
    }

    // ---------------------------------------------------------
    // FIELDS (champs electromagnetiques)
    // ---------------------------------------------------------

    /// Sauvegarde l'etat des champs EM dans self_identity.fields_json.
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

    /// Charge l'etat des champs EM depuis self_identity.fields_json.
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
    // PORTRAIT DE PERSONNALITE TEMPOREL (3 niveaux)
    // ---------------------------------------------------------

    /// Niveau 1 : Sauvegarde un snapshot de personnalite complet.
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

    /// Niveau 1 : Charge les snapshots de personnalite (plus recents d'abord).
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

    /// Niveau 2a : Sauvegarde une entree de trajectoire emotionnelle.
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

    /// Niveau 2a : Charge la trajectoire emotionnelle.
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

    /// Niveau 2b : Sauvegarde un point d'historique de conscience.
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

    /// Niveau 2b : Charge l'historique de conscience.
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

    /// Niveau 2c : Sauvegarde un checkpoint psychologique.
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

    /// Niveau 2c : Charge les checkpoints psychologiques.
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

    /// Niveau 2d : Sauvegarde un point de timeline relationnelle.
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

    /// Niveau 2d : Charge la timeline relationnelle.
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

    /// Niveau 3 : Sauvegarde une entree de journal introspectif.
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

    /// Niveau 3 : Charge les entrees du journal introspectif.
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

    /// Sauvegarde un enregistrement de sommeil dans la table sleep_history.
    pub async fn save_sleep_record(&self, record: &crate::sleep::SleepRecord) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "INSERT INTO sleep_history
                (started_at, ended_at, total_sleep_cycles,
                 dreams_count, memories_consolidated, connections_created, quality,
                 interrupted, interrupt_reason)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            &[
                &record.started_at,
                &record.ended_at,
                &(record.duration_cycles as i32),
                &(record.dreams_count as i32),
                &(record.memories_consolidated as i64),
                &(record.connections_created as i64),
                &(record.quality as f32),
                &record.interrupted,
                &record.interruption_reason,
            ],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_sleep_record: db error: {}", e);
        }
        Ok(())
    }

    /// Charge les derniers enregistrements de sommeil depuis la DB.
    pub async fn load_sleep_history(&self, limit: i64) -> Result<Vec<crate::sleep::SleepRecord>, DbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT started_at, ended_at, total_sleep_cycles,
                    dreams_count, memories_consolidated, connections_created,
                    quality, interrupted, interrupt_reason
             FROM sleep_history ORDER BY ended_at DESC LIMIT $1",
            &[&limit],
        ).await?;

        let mut records = Vec::with_capacity(rows.len());
        for row in &rows {
            let started_at: chrono::DateTime<chrono::Utc> = row.get(0);
            let ended_at: chrono::DateTime<chrono::Utc> = row.get(1);
            let total_sleep_cycles: i32 = row.get(2);
            let dreams_count: i32 = row.get(3);
            let memories_consolidated: i64 = row.get(4);
            let connections_created: i64 = row.get(5);
            let quality: f32 = row.get(6);
            let interrupted: bool = row.get(7);
            let interruption_reason: Option<String> = row.get(8);

            records.push(crate::sleep::SleepRecord {
                started_at,
                ended_at,
                duration_cycles: total_sleep_cycles as u64,
                sleep_cycles_completed: 0,
                quality: quality as f64,
                memories_consolidated: memories_consolidated as u64,
                connections_created: connections_created as u64,
                dreams_count: dreams_count as u64,
                interrupted,
                interruption_reason,
            });
        }
        // Inverser pour avoir l'ordre chronologique (plus ancien d'abord)
        records.reverse();
        Ok(records)
    }

    // ---------------------------------------------------------
    // SESSIONS
    // ---------------------------------------------------------

    /// Enregistre le debut d'une nouvelle session (un boot de l'agent).
    ///
    /// # Parametres
    /// - `boot_number` : numero de demarrage (incremente a chaque boot)
    ///
    /// # Retour
    /// L'identifiant de la session creee
    pub async fn start_session(&self, boot_number: i32) -> Result<i64, DbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "INSERT INTO session_log (boot_number) VALUES ($1) RETURNING id",
            &[&boot_number],
        ).await?;
        Ok(row.get(0))
    }

    /// Cloture une session en enregistrant le nombre de cycles et le type d'arret.
    ///
    /// # Parametres
    /// - `session_id` : identifiant de la session a cloturer
    /// - `cycles` : nombre de cycles effectues pendant cette session
    /// - `clean` : true si l'arret est propre (signal Ctrl+C), false si crash
    pub async fn end_session(&self, session_id: i64, cycles: i32, clean: bool) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE session_log SET ended_at = NOW(), cycles_this_session = $2, clean_shutdown = $3
             WHERE id = $1",
            &[&session_id, &cycles, &clean],
        ).await?;
        Ok(())
    }

    /// Sauvegarde l'etat psychologique persistant (compteurs Tolteques, ombre, EQ, flow).
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

    /// Charge l'etat psychologique persistant.
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

    // ---------------------------------------------------------
    // VALUES (valeurs de caractere / vertus)
    // ---------------------------------------------------------

    /// Sauvegarde l'etat des valeurs de caractere dans self_identity.values_json.
    pub async fn save_values_state(&self, json: &serde_json::Value) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "UPDATE self_identity SET values_json = $1, updated_at = NOW() WHERE id = 1",
            &[json],
        ).await;
        if let Err(e) = result {
            tracing::warn!("save_values_state: {} (colonne values_json peut-etre absente)", e);
        }
        Ok(())
    }

    /// Charge l'etat des valeurs de caractere depuis self_identity.values_json.
    pub async fn load_values_state(&self) -> Result<Option<serde_json::Value>, DbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT values_json FROM self_identity WHERE id = 1",
            &[],
        ).await;
        match result {
            Ok(Some(row)) => {
                let json: Option<serde_json::Value> = row.get(0);
                Ok(json)
            },
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!("load_values_state: {} (colonne values_json peut-etre absente)", e);
                Ok(None)
            }
        }
    }
}
