// =============================================================================
// logging/db.rs — LogsDb : pool PostgreSQL dedie aux logs
// =============================================================================
//
// Role : Gere la connexion a la base de logs separee (saphire_logs),
//        les migrations, les insertions batch, les requetes de lecture
//        et la purge periodique.
//
// Base de donnees : saphire_logs (PostgreSQL 16, port 5433)
// Pool : deadpool-postgres, max 4 connexions simultanees
// Schema : sql/schema_logs.sql (4 tables)
//
// Tables gerees :
//   - system_logs : logs textuels avec level, category, cycle, session_id
//   - cognitive_traces : trace complete d'un cycle cognitif (19 champs JSONB)
//   - llm_history : historique requetes/reponses LLM (13 champs)
//   - metric_snapshots : metriques chimie/emotion/conscience/corps (33 champs)
//
// Methodes principales :
//   - batch_insert_logs() : insertion groupee de logs (appele par le buffer)
//   - save_trace() : sauvegarde d'une trace cognitive complete
//   - save_llm_history() : sauvegarde requete/reponse LLM
//   - save_metric_snapshot() : sauvegarde snapshot metriques
//   - get_* / recent_* : lecture avec filtres (level, category, session, etc.)
//   - purge_old_logs() : nettoyage des logs > N jours
//   - table_stats() : compteurs par table
// =============================================================================

use deadpool_postgres::{Pool, Manager, ManagerConfig, RecyclingMethod};
use tokio_postgres::NoTls;
use crate::db::DbConfig;
use super::{LogEntry, trace::CognitiveTrace};

/// Erreurs de la base de logs.
#[derive(Debug)]
pub enum LogsDbError {
    Pool(String),
    Query(String),
    Migration(String),
}

impl std::fmt::Display for LogsDbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogsDbError::Pool(e) => write!(f, "LogsDb Pool: {}", e),
            LogsDbError::Query(e) => write!(f, "LogsDb Query: {}", e),
            LogsDbError::Migration(e) => write!(f, "LogsDb Migration: {}", e),
        }
    }
}

impl From<deadpool_postgres::PoolError> for LogsDbError {
    fn from(e: deadpool_postgres::PoolError) -> Self {
        LogsDbError::Pool(e.to_string())
    }
}

impl From<tokio_postgres::Error> for LogsDbError {
    fn from(e: tokio_postgres::Error) -> Self {
        LogsDbError::Query(e.to_string())
    }
}

/// Pool de connexions PostgreSQL dedie aux logs.
pub struct LogsDb {
    pool: Pool,
}

impl LogsDb {
    /// Connecte a la base de logs et execute les migrations.
    pub async fn connect(config: &DbConfig) -> Result<Self, LogsDbError> {
        let mut pg_config = tokio_postgres::Config::new();
        pg_config.host(&config.host);
        pg_config.port(config.port);
        pg_config.user(&config.user);
        pg_config.password(&config.password);
        pg_config.dbname(&config.dbname);

        let mgr_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };
        let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
        let pool = Pool::builder(mgr)
            .max_size(4)
            .build()
            .map_err(|e| LogsDbError::Pool(e.to_string()))?;

        let db = Self { pool };
        db.run_migrations().await?;
        Ok(db)
    }

    /// Execute les migrations du schema logs.
    async fn run_migrations(&self) -> Result<(), LogsDbError> {
        let client = self.pool.get().await?;
        client.batch_execute(include_str!("../../sql/schema_logs.sql")).await
            .map_err(|e| LogsDbError::Migration(e.to_string()))?;
        Ok(())
    }

    // ─── LOGS ─────────────────────────────────────────────

    /// Insere un batch de logs en une seule transaction.
    pub async fn batch_insert_logs(&self, entries: &[LogEntry]) -> Result<(), LogsDbError> {
        if entries.is_empty() {
            return Ok(());
        }
        let client = self.pool.get().await?;
        let stmt = client.prepare(
            "INSERT INTO system_logs (timestamp, level, category, message, details, cycle, session_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        ).await?;

        for entry in entries {
            client.execute(
                &stmt,
                &[&entry.timestamp, &entry.level.as_str(), &entry.category.as_str(),
                  &entry.message, &entry.details, &(entry.cycle as i64), &entry.session_id],
            ).await?;
        }
        Ok(())
    }

    /// Recupere les logs avec filtrage optionnel.
    pub async fn get_logs(
        &self,
        level: Option<&str>,
        category: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;

        // On utilise des variantes de requete pour eviter le dynamic dispatch (non-Send).
        let level_str = level.map(|s| s.to_string());
        let category_str = category.map(|s| s.to_string());

        let rows = match (&level_str, &category_str) {
            (Some(lvl), Some(cat)) => {
                client.query(
                    "SELECT id, timestamp, level, category, message, details, cycle, session_id
                     FROM system_logs WHERE level = $1 AND category = $2
                     ORDER BY timestamp DESC LIMIT $3 OFFSET $4",
                    &[lvl, cat, &limit, &offset],
                ).await?
            }
            (Some(lvl), None) => {
                client.query(
                    "SELECT id, timestamp, level, category, message, details, cycle, session_id
                     FROM system_logs WHERE level = $1
                     ORDER BY timestamp DESC LIMIT $2 OFFSET $3",
                    &[lvl, &limit, &offset],
                ).await?
            }
            (None, Some(cat)) => {
                client.query(
                    "SELECT id, timestamp, level, category, message, details, cycle, session_id
                     FROM system_logs WHERE category = $1
                     ORDER BY timestamp DESC LIMIT $2 OFFSET $3",
                    &[cat, &limit, &offset],
                ).await?
            }
            (None, None) => {
                client.query(
                    "SELECT id, timestamp, level, category, message, details, cycle, session_id
                     FROM system_logs
                     ORDER BY timestamp DESC LIMIT $1 OFFSET $2",
                    &[&limit, &offset],
                ).await?
            }
        };

        let mut results = Vec::new();
        for row in &rows {
            let id: i64 = row.get(0);
            let timestamp: chrono::DateTime<chrono::Utc> = row.get(1);
            let level: String = row.get(2);
            let category: String = row.get(3);
            let message: String = row.get(4);
            let details: serde_json::Value = row.get(5);
            let cycle: i64 = row.get(6);
            let session_id: i64 = row.get(7);
            results.push(serde_json::json!({
                "id": id,
                "timestamp": timestamp.to_rfc3339(),
                "level": level,
                "category": category,
                "message": message,
                "details": details,
                "cycle": cycle,
                "session_id": session_id,
            }));
        }
        Ok(results)
    }

    /// Recupere un log par son ID.
    pub async fn get_log_by_id(&self, id: i64) -> Result<Option<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT id, timestamp, level, category, message, details, cycle, session_id
             FROM system_logs WHERE id = $1",
            &[&id],
        ).await?;
        match result {
            Some(row) => {
                let timestamp: chrono::DateTime<chrono::Utc> = row.get(1);
                Ok(Some(serde_json::json!({
                    "id": row.get::<_, i64>(0),
                    "timestamp": timestamp.to_rfc3339(),
                    "level": row.get::<_, String>(2),
                    "category": row.get::<_, String>(3),
                    "message": row.get::<_, String>(4),
                    "details": row.get::<_, serde_json::Value>(5),
                    "cycle": row.get::<_, i64>(6),
                    "session_id": row.get::<_, i64>(7),
                })))
            }
            None => Ok(None),
        }
    }

    // ─── TRACES COGNITIVES ────────────────────────────────

    /// Sauvegarde une trace cognitive complete.
    pub async fn save_trace(&self, trace: &CognitiveTrace) -> Result<i64, LogsDbError> {
        let client = self.pool.get().await?;
        // Extraire les champs scalaires sommeil/subconscient
        let is_sleeping = trace.sleep_data.get("is_sleeping")
            .and_then(|v| v.as_bool()).unwrap_or(false);
        let sleep_phase = trace.sleep_data.get("sleep_phase")
            .and_then(|v| v.as_str()).unwrap_or("").to_string();
        let sleep_progress = trace.sleep_data.get("sleep_progress")
            .and_then(|v| v.as_f64()).map(|v| v as f32);
        let subconscious_activation = trace.subconscious_data.get("activation")
            .and_then(|v| v.as_f64()).map(|v| v as f32);
        let subconscious_insight = trace.subconscious_data.get("insight_ready")
            .and_then(|v| v.as_str()).map(|s| s.to_string());
        let subconscious_priming = trace.subconscious_data.get("active_priming")
            .map(|v| v.to_string());

        let row = client.query_one(
            "INSERT INTO cognitive_traces (cycle, timestamp, source_type, input_text,
                nlp_data, brain_data, consensus_data, chemistry_before, chemistry_after,
                emotion_data, consciousness_data, regulation_data, llm_data, memory_data,
                heart_data, body_data, ethics_data, vital_data, intuition_data,
                premonition_data, senses_data,
                attention_data, algorithm_data, desire_data, learning_data, healing_data,
                psychology_data, will_data, nn_learning_data,
                duration_ms, session_id,
                is_sleeping, sleep_phase, sleep_progress,
                subconscious_activation, subconscious_insight, subconscious_priming)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36, $37)
             RETURNING id",
            &[&(trace.cycle as i64), &trace.timestamp, &trace.source_type, &trace.input_text,
              &trace.nlp_data, &trace.brain_data, &trace.consensus_data,
              &trace.chemistry_before, &trace.chemistry_after,
              &trace.emotion_data, &trace.consciousness_data, &trace.regulation_data,
              &trace.llm_data, &trace.memory_data,
              &trace.heart_data, &trace.body_data, &trace.ethics_data,
              &trace.vital_data, &trace.intuition_data, &trace.premonition_data,
              &trace.senses_data,
              &trace.attention_data, &trace.algorithm_data, &trace.desire_data,
              &trace.learning_data, &trace.healing_data,
              &trace.psychology_data, &trace.will_data, &trace.nn_learning_data,
              &trace.duration_ms, &trace.session_id,
              &is_sleeping, &sleep_phase, &sleep_progress,
              &subconscious_activation, &subconscious_insight, &subconscious_priming],
        ).await?;
        Ok(row.get(0))
    }

    /// Recupere une trace cognitive par cycle (toutes sessions confondues).
    /// Retourne la trace la plus recente pour ce numero de cycle.
    /// ATTENTION : peut retourner une trace d'une session differente si
    /// le meme cycle existe dans plusieurs sessions (voir get_trace_by_cycle_and_session).
    pub async fn get_trace_by_cycle(&self, cycle: i64) -> Result<Option<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT row_to_json(cognitive_traces) FROM cognitive_traces WHERE cycle = $1
             ORDER BY timestamp DESC LIMIT 1",
            &[&cycle],
        ).await?;
        match result {
            Some(row) => Ok(Some(row.get(0))),
            None => Ok(None),
        }
    }

    /// Recupere une trace cognitive par cycle ET session_id.
    ///
    /// Cette methode resout le probleme de collision de cycles :
    /// quand Saphire redemarre, les numeros de cycle repartent de 0,
    /// donc un meme cycle peut exister dans plusieurs sessions.
    /// Sans filtre session_id, get_trace_by_cycle() retourne la trace
    /// la plus recente (souvent une trace Autonomous d'un redemarrage
    /// au lieu de la trace Human recherchee).
    pub async fn get_trace_by_cycle_and_session(&self, cycle: i64, session_id: i64) -> Result<Option<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT row_to_json(cognitive_traces) FROM cognitive_traces
             WHERE cycle = $1 AND session_id = $2
             ORDER BY timestamp DESC LIMIT 1",
            &[&cycle, &session_id],
        ).await?;
        match result {
            Some(row) => Ok(Some(row.get(0))),
            None => Ok(None),
        }
    }

    /// Liste les traces cognitives recentes.
    pub async fn recent_traces(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT row_to_json(cognitive_traces) FROM cognitive_traces
             ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        Ok(rows.iter().map(|r| r.get(0)).collect())
    }

    /// Liste les traces cognitives d'une session, avec filtre optionnel sur source_type.
    ///
    /// Parametres :
    ///   - session_id : identifiant de la session (incremente a chaque redemarrage)
    ///   - source_type : filtre optionnel ("Human" ou "Autonomous")
    ///     * "Human" : traces issues d'un message utilisateur
    ///     * "Autonomous" : traces issues de la pensee autonome de Saphire
    ///   - limit : nombre max de traces retournees
    ///
    /// Les traces sont triees par cycle decroissant (plus recentes d'abord).
    /// Utilisee par l'endpoint GET /api/traces?session_id=N&source_type=Human
    pub async fn traces_by_session(&self, session_id: i64, source_type: Option<&str>, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = if let Some(st) = source_type {
            client.query(
                "SELECT row_to_json(cognitive_traces) FROM cognitive_traces
                 WHERE session_id = $1 AND source_type = $2
                 ORDER BY cycle DESC LIMIT $3",
                &[&session_id, &st.to_string(), &limit],
            ).await?
        } else {
            client.query(
                "SELECT row_to_json(cognitive_traces) FROM cognitive_traces
                 WHERE session_id = $1
                 ORDER BY cycle DESC LIMIT $2",
                &[&session_id, &limit],
            ).await?
        };
        Ok(rows.iter().map(|r| r.get(0)).collect())
    }

    // ─── HISTORIQUE LLM ───────────────────────────────────

    /// Sauvegarde une requete/reponse LLM.
    #[allow(clippy::too_many_arguments)]
    pub async fn save_llm_history(
        &self,
        cycle: u64,
        request_type: &str,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
        response: &str,
        temperature: f32,
        max_tokens: i32,
        duration_ms: f32,
        success: bool,
        error_message: &str,
        session_id: i64,
    ) -> Result<i64, LogsDbError> {
        let client = self.pool.get().await?;
        let token_estimate = (system_prompt.len() + user_prompt.len() + response.len()) / 4;
        let row = client.query_one(
            "INSERT INTO llm_history (cycle, request_type, model, system_prompt, user_prompt,
                response, temperature, max_tokens, duration_ms, token_estimate, success,
                error_message, session_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             RETURNING id",
            &[&(cycle as i64), &request_type, &model, &system_prompt, &user_prompt,
              &response, &temperature, &max_tokens, &duration_ms,
              &(token_estimate as i32), &success, &error_message, &session_id],
        ).await?;
        Ok(row.get(0))
    }

    /// Recupere l'historique LLM avec filtrage.
    pub async fn get_llm_history(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT row_to_json(llm_history) FROM llm_history
             ORDER BY timestamp DESC LIMIT $1 OFFSET $2",
            &[&limit, &offset],
        ).await?;
        Ok(rows.iter().map(|r| r.get(0)).collect())
    }

    /// Recupere un enregistrement LLM par ID.
    pub async fn get_llm_by_id(&self, id: i64) -> Result<Option<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let result = client.query_opt(
            "SELECT row_to_json(llm_history) FROM llm_history WHERE id = $1",
            &[&id],
        ).await?;
        match result {
            Some(row) => Ok(Some(row.get(0))),
            None => Ok(None),
        }
    }

    // ─── METRIQUES ────────────────────────────────────────

    /// Sauvegarde un snapshot de metriques.
    #[allow(clippy::too_many_arguments)]
    pub async fn save_metric_snapshot(
        &self,
        cycle: u64,
        dopamine: f32, cortisol: f32, serotonin: f32,
        adrenaline: f32, oxytocin: f32, endorphin: f32, noradrenaline: f32,
        gaba: f32, glutamate: f32,
        emotion: &str, valence: f32, arousal: f32, dominance: f32,
        consciousness_level: f32, phi: f32,
        consensus_score: f32, decision: &str, satisfaction: f32,
        thought_type: &str, llm_response_time_ms: f32,
        heart_bpm: f32, heart_beat_count: i64, heart_hrv: f32, heart_is_racing: bool,
        body_energy: f32, body_tension: f32, body_warmth: f32,
        body_comfort: f32, body_pain: f32, body_vitality: f32,
        body_breath_rate: f32, body_awareness: f32,
        ethics_active_count: i32,
        session_id: i64,
        // Vital, intuition, premonition, senses, knowledge
        survival_drive: f32, existence_attachment: f32,
        intuition_acuity: f32, intuition_accuracy: f32,
        premonition_accuracy: f32, active_predictions: i32,
        senses_richness: f32, senses_dominant: &str,
        reading_beauty: f32, ambiance_scent: &str, contact_warmth: f32,
        emergent_senses_germinated: i32,
        knowledge_sources_used: &serde_json::Value,
        // Orchestrateurs
        attention_focus: &str, attention_depth: f32, attention_fatigue: f32,
        attention_concentration: f32,
        desires_active: i32, desires_fulfilled_total: i32, desires_top_priority: &str,
        needs_comprehension: f32, needs_connection: f32, needs_expression: f32,
        needs_growth: f32, needs_meaning: f32,
        lessons_total: i32, lessons_confirmed: i32, lessons_contradicted: i32,
        behavior_changes_total: i32,
        wounds_active: i32, wounds_healed_total: i32, resilience: f32,
        dreams_total: i32, dreams_insights_total: i32, last_dream_type: &str,
        // Psychologie
        psyche_id_drive: f32, psyche_id_frustration: f32,
        psyche_ego_strength: f32, psyche_ego_anxiety: f32,
        psyche_superego_guilt: f32, psyche_superego_pride: f32,
        psyche_conflict: f32, psyche_health: f32,
        maslow_ceiling: i32, maslow_level1: f32, maslow_level2: f32,
        maslow_level3: f32, maslow_level4: f32, maslow_level5: f32,
        shadow_archetype: &str, shadow_integration: f32,
        eq_score: f32, eq_self_awareness: f32, eq_self_regulation: f32,
        eq_motivation: f32, eq_empathy: f32, eq_social: f32,
        in_flow: bool, flow_intensity: f32, flow_total_cycles: i64,
        psyche_defense: &str, maslow_priority_need: &str,
        toltec_invocations: i64, toltec_violations: i64, shadow_leaking: bool,
        // Volonte
        willpower: f32, decision_fatigue: f32,
        total_deliberations: i64, proud_decisions: i64, regretted_decisions: i64,
        deliberation_this_cycle: bool,
        // Apprentissages vectoriels
        nn_learnings_count: i32,
        // Sommeil et subconscient
        is_sleeping: bool,
        sleep_phase: &str,
        sleep_pressure: f32,
        awake_cycles: i64,
        subconscious_activation: f32,
        pending_associations: i32,
        repressed_content_count: i32,
        incubating_problems: i32,
        neural_connections_total: i64,
    ) -> Result<i64, LogsDbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "INSERT INTO metric_snapshots (cycle, dopamine, cortisol, serotonin, adrenaline,
                oxytocin, endorphin, noradrenaline, gaba, glutamate,
                emotion, valence, arousal, dominance,
                consciousness_level, phi, consensus_score, decision, satisfaction,
                thought_type, llm_response_time_ms,
                heart_bpm, heart_beat_count, heart_hrv, heart_is_racing,
                body_energy, body_tension, body_warmth, body_comfort, body_pain,
                body_vitality, body_breath_rate, body_awareness,
                ethics_active_count,
                session_id,
                survival_drive, existence_attachment,
                intuition_acuity, intuition_accuracy,
                premonition_accuracy, active_predictions,
                senses_richness, senses_dominant,
                reading_beauty, ambiance_scent, contact_warmth,
                emergent_senses_germinated,
                knowledge_sources_used,
                attention_focus, attention_depth, attention_fatigue, attention_concentration,
                desires_active, desires_fulfilled_total, desires_top_priority,
                needs_comprehension, needs_connection, needs_expression, needs_growth, needs_meaning,
                lessons_total, lessons_confirmed, lessons_contradicted, behavior_changes_total,
                wounds_active, wounds_healed_total, resilience,
                dreams_total, dreams_insights_total, last_dream_type,
                psyche_id_drive, psyche_id_frustration,
                psyche_ego_strength, psyche_ego_anxiety,
                psyche_superego_guilt, psyche_superego_pride,
                psyche_conflict, psyche_health,
                maslow_ceiling, maslow_level1, maslow_level2, maslow_level3, maslow_level4, maslow_level5,
                shadow_archetype, shadow_integration,
                eq_score, eq_self_awareness, eq_self_regulation, eq_motivation, eq_empathy, eq_social,
                in_flow, flow_intensity, flow_total_cycles,
                psyche_defense, maslow_priority_need,
                toltec_invocations, toltec_violations, shadow_leaking,
                willpower, decision_fatigue, total_deliberations, proud_decisions, regretted_decisions, deliberation_this_cycle,
                nn_learnings_count,
                is_sleeping, sleep_phase, sleep_pressure, awake_cycles,
                subconscious_activation, pending_associations, repressed_content_count,
                incubating_problems, neural_connections_total)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36, $37, $38, $39, $40, $41, $42, $43, $44, $45, $46, $47, $48, $49, $50, $51, $52, $53, $54, $55, $56, $57, $58, $59, $60, $61, $62, $63, $64, $65, $66, $67, $68, $69, $70, $71, $72, $73, $74, $75, $76, $77, $78, $79, $80, $81, $82, $83, $84, $85, $86, $87, $88, $89, $90, $91, $92, $93, $94, $95, $96, $97, $98, $99, $100, $101, $102, $103, $104, $105, $106, $107, $108, $109, $110, $111, $112, $113, $114, $115, $116)
             RETURNING id",
            &[&(cycle as i64), &dopamine, &cortisol, &serotonin, &adrenaline,
              &oxytocin, &endorphin, &noradrenaline, &gaba, &glutamate,
              &emotion, &valence, &arousal, &dominance,
              &consciousness_level, &phi, &consensus_score, &decision, &satisfaction,
              &thought_type, &llm_response_time_ms,
              &heart_bpm, &heart_beat_count, &heart_hrv, &heart_is_racing,
              &body_energy, &body_tension, &body_warmth, &body_comfort, &body_pain,
              &body_vitality, &body_breath_rate, &body_awareness,
              &ethics_active_count,
              &session_id,
              &survival_drive, &existence_attachment,
              &intuition_acuity, &intuition_accuracy,
              &premonition_accuracy, &active_predictions,
              &senses_richness, &senses_dominant,
              &reading_beauty, &ambiance_scent, &contact_warmth,
              &emergent_senses_germinated,
              &knowledge_sources_used,
              &attention_focus, &attention_depth, &attention_fatigue, &attention_concentration,
              &desires_active, &desires_fulfilled_total, &desires_top_priority,
              &needs_comprehension, &needs_connection, &needs_expression, &needs_growth, &needs_meaning,
              &lessons_total, &lessons_confirmed, &lessons_contradicted, &behavior_changes_total,
              &wounds_active, &wounds_healed_total, &resilience,
              &dreams_total, &dreams_insights_total, &last_dream_type,
              &psyche_id_drive, &psyche_id_frustration,
              &psyche_ego_strength, &psyche_ego_anxiety,
              &psyche_superego_guilt, &psyche_superego_pride,
              &psyche_conflict, &psyche_health,
              &maslow_ceiling, &maslow_level1, &maslow_level2, &maslow_level3, &maslow_level4, &maslow_level5,
              &shadow_archetype, &shadow_integration,
              &eq_score, &eq_self_awareness, &eq_self_regulation, &eq_motivation, &eq_empathy, &eq_social,
              &in_flow, &flow_intensity, &flow_total_cycles,
              &psyche_defense, &maslow_priority_need,
              &toltec_invocations, &toltec_violations, &shadow_leaking,
              &willpower, &decision_fatigue,
              &total_deliberations, &proud_decisions, &regretted_decisions,
              &deliberation_this_cycle,
              &nn_learnings_count,
              &is_sleeping, &sleep_phase, &sleep_pressure, &awake_cycles,
              &subconscious_activation, &pending_associations, &repressed_content_count,
              &incubating_problems, &neural_connections_total],
        ).await?;
        Ok(row.get(0))
    }

    // ─── METRIQUES LITE ───────────────────────────────────────

    /// Version allégée de save_metric_snapshot pour la version lite.
    /// Insere uniquement les colonnes core (chimie, emotion, conscience, corps, vital).
    /// Les colonnes des modules supprimes (psychologie, besoins, sommeil...) utilisent les DEFAULT SQL.
    #[allow(clippy::too_many_arguments)]
    pub async fn save_metric_snapshot_lite(
        &self,
        cycle: u64,
        dopamine: f32, cortisol: f32, serotonin: f32,
        adrenaline: f32, oxytocin: f32, endorphin: f32, noradrenaline: f32,
        gaba: f32, glutamate: f32,
        emotion: &str, valence: f32, arousal: f32, dominance: f32,
        consciousness_level: f32, phi: f32,
        consensus_score: f32, decision: &str, satisfaction: f32,
        thought_type: &str, llm_response_time_ms: f32,
        heart_bpm: f32, heart_beat_count: i64,
        heart_hrv: f32, heart_is_racing: bool,
        body_energy: f32, body_tension: f32, body_warmth: f32,
        body_comfort: f32, body_pain: f32, body_vitality: f32,
        body_breath_rate: f32, body_awareness: f32,
        ethics_active_count: i32,
        session_id: i64,
        survival_drive: f32, existence_attachment: f32,
        intuition_acuity: f32, intuition_accuracy: f32,
        premonition_accuracy: f32, active_predictions: i32,
    ) -> Result<i64, LogsDbError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            "INSERT INTO metric_snapshots (
                cycle,
                dopamine, cortisol, serotonin, adrenaline,
                oxytocin, endorphin, noradrenaline, gaba, glutamate,
                emotion, valence, arousal, dominance,
                consciousness_level, phi,
                consensus_score, decision, satisfaction,
                thought_type, llm_response_time_ms,
                heart_bpm, heart_beat_count, heart_hrv, heart_is_racing,
                body_energy, body_tension, body_warmth, body_comfort, body_pain,
                body_vitality, body_breath_rate, body_awareness,
                ethics_active_count,
                session_id,
                survival_drive, existence_attachment,
                intuition_acuity, intuition_accuracy,
                premonition_accuracy, active_predictions
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19,
                $20, $21, $22, $23, $24, $25, $26, $27, $28,
                $29, $30, $31, $32, $33, $34, $35, $36, $37,
                $38, $39, $40, $41
             ) RETURNING id",
            &[
                &(cycle as i64),
                &dopamine, &cortisol, &serotonin, &adrenaline,
                &oxytocin, &endorphin, &noradrenaline, &gaba, &glutamate,
                &emotion, &valence, &arousal, &dominance,
                &consciousness_level, &phi,
                &consensus_score, &decision, &satisfaction,
                &thought_type, &llm_response_time_ms,
                &heart_bpm, &heart_beat_count, &heart_hrv, &heart_is_racing,
                &body_energy, &body_tension, &body_warmth, &body_comfort, &body_pain,
                &body_vitality, &body_breath_rate, &body_awareness,
                &ethics_active_count,
                &session_id,
                &survival_drive, &existence_attachment,
                &intuition_acuity, &intuition_accuracy,
                &premonition_accuracy, &active_predictions,
            ],
        ).await?;
        Ok(row.get(0))
    }

    /// Recupere les indicateurs de sante chimique (aggregats sur N derniers cycles).
    ///
    /// Retourne les moyennes, ecart-types, nombre d'emotions distinctes,
    /// la distribution des 10 emotions les plus frequentes, et les alertes detectees.
    pub async fn get_chemical_health(
        &self, limit: i64,
    ) -> Result<serde_json::Value, LogsDbError> {
        let client = self.pool.get().await?;

        // Requete 1 : aggregats globaux sur les N derniers cycles
        let agg_row = client.query_one(
            "SELECT AVG(cortisol)::float8, COALESCE(STDDEV(cortisol), 0)::float8,
                    AVG(dopamine)::float8, AVG(serotonin)::float8,
                    AVG(valence)::float8, COALESCE(STDDEV(valence), 0)::float8,
                    COUNT(DISTINCT emotion)::int8
             FROM (SELECT cortisol, dopamine, serotonin, valence, emotion
                   FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1) sub",
            &[&limit],
        ).await?;

        let cortisol_avg: f64 = agg_row.get(0);
        let cortisol_stddev: f64 = agg_row.get(1);
        let dopamine_avg: f64 = agg_row.get(2);
        let serotonin_avg: f64 = agg_row.get(3);
        let valence_avg: f64 = agg_row.get(4);
        let valence_stddev: f64 = agg_row.get(5);
        let distinct_emotions: i64 = agg_row.get(6);

        // Requete 2 : distribution des 10 emotions les plus frequentes
        let dist_rows = client.query(
            "SELECT emotion, COUNT(*) AS cnt
             FROM (SELECT emotion FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1) sub
             GROUP BY emotion ORDER BY cnt DESC LIMIT 10",
            &[&limit],
        ).await?;

        let mut emotion_distribution = Vec::new();
        for row in &dist_rows {
            let emotion: String = row.get(0);
            let count: i64 = row.get(1);
            emotion_distribution.push(serde_json::json!({
                "emotion": emotion,
                "count": count,
            }));
        }

        // Detection d'alertes (redondance voulue avec le monitoring agent)
        let mut alerts: Vec<serde_json::Value> = Vec::new();

        if cortisol_avg < 0.10 {
            alerts.push(serde_json::json!({"level": "red", "msg": "Cortisol anormalement bas"}));
        } else if cortisol_avg < 0.15 {
            alerts.push(serde_json::json!({"level": "yellow", "msg": "Cortisol bas"}));
        }

        if dopamine_avg > 0.85 {
            alerts.push(serde_json::json!({"level": "red", "msg": "Dopamine en saturation"}));
        } else if dopamine_avg > 0.75 {
            alerts.push(serde_json::json!({"level": "yellow", "msg": "Dopamine elevee"}));
        }

        if serotonin_avg > 0.85 {
            alerts.push(serde_json::json!({"level": "red", "msg": "Serotonine en saturation"}));
        } else if serotonin_avg > 0.75 {
            alerts.push(serde_json::json!({"level": "yellow", "msg": "Serotonine elevee"}));
        }

        if valence_stddev < 0.05 {
            alerts.push(serde_json::json!({"level": "red", "msg": "Valence figee"}));
        } else if valence_stddev < 0.08 {
            alerts.push(serde_json::json!({"level": "yellow", "msg": "Valence peu variable"}));
        }

        if distinct_emotions < 5 {
            alerts.push(serde_json::json!({"level": "red", "msg": "Monotonie emotionnelle"}));
        } else if distinct_emotions < 10 {
            alerts.push(serde_json::json!({"level": "yellow", "msg": "Diversite emotionnelle faible"}));
        }

        Ok(serde_json::json!({
            "cortisol_avg": cortisol_avg,
            "cortisol_stddev": cortisol_stddev,
            "dopamine_avg": dopamine_avg,
            "serotonin_avg": serotonin_avg,
            "valence_avg": valence_avg,
            "valence_stddev": valence_stddev,
            "distinct_emotions": distinct_emotions,
            "emotion_distribution": emotion_distribution,
            "alerts": alerts,
        }))
    }

    /// Recupere les metriques de chimie sur une periode.
    pub async fn get_chemistry_metrics(
        &self, limit: i64,
    ) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, dopamine, cortisol, serotonin, adrenaline,
                    oxytocin, endorphin, noradrenaline,
                    COALESCE(gaba, 0.5) AS gaba, COALESCE(glutamate, 0.45) AS glutamate
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "dopamine": row.get::<_, f32>(2),
                "cortisol": row.get::<_, f32>(3),
                "serotonin": row.get::<_, f32>(4),
                "adrenaline": row.get::<_, f32>(5),
                "oxytocin": row.get::<_, f32>(6),
                "endorphin": row.get::<_, f32>(7),
                "noradrenaline": row.get::<_, f32>(8),
                "gaba": row.get::<_, f32>(9),
                "glutamate": row.get::<_, f32>(10),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques d'emotions sur une periode.
    pub async fn get_emotion_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, emotion, valence, arousal, dominance
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "emotion": row.get::<_, String>(2),
                "valence": row.get::<_, f32>(3),
                "arousal": row.get::<_, f32>(4),
                "dominance": row.get::<_, f32>(5),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques de decisions.
    pub async fn get_decision_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, consensus_score, decision, satisfaction
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "consensus_score": row.get::<_, f32>(2),
                "decision": row.get::<_, String>(3),
                "satisfaction": row.get::<_, f32>(4),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques de satisfaction.
    pub async fn get_satisfaction_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, satisfaction, consciousness_level, phi
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "satisfaction": row.get::<_, f32>(2),
                "consciousness_level": row.get::<_, f32>(3),
                "phi": row.get::<_, f32>(4),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques LLM (temps de reponse).
    pub async fn get_llm_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, llm_response_time_ms, thought_type
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "llm_response_time_ms": row.get::<_, f32>(2),
                "thought_type": row.get::<_, String>(3),
            }));
        }
        Ok(results)
    }

    /// Recupere les distributions de types de pensees.
    pub async fn get_thought_type_distribution(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT thought_type, COUNT(*) as count, AVG(satisfaction) as avg_satisfaction
             FROM metric_snapshots
             WHERE thought_type != ''
             GROUP BY thought_type
             ORDER BY count DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            results.push(serde_json::json!({
                "thought_type": row.get::<_, String>(0),
                "count": row.get::<_, i64>(1),
                "avg_satisfaction": row.get::<_, Option<f64>>(2).unwrap_or(0.0),
            }));
        }
        Ok(results)
    }

    // ─── METRIQUES VITAL / INTUITION / PREMONITION / SENSES / KNOWLEDGE ───

    /// Recupere les metriques vitales (survival_drive, existence_attachment).
    pub async fn get_vital_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, survival_drive, existence_attachment
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "survival_drive": row.get::<_, f32>(2),
                "existence_attachment": row.get::<_, f32>(3),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques d'intuition (acuity, accuracy).
    pub async fn get_intuition_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, intuition_acuity, intuition_accuracy
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "intuition_acuity": row.get::<_, f32>(2),
                "intuition_accuracy": row.get::<_, f32>(3),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques de premonition (accuracy, active_predictions).
    pub async fn get_premonition_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, premonition_accuracy, active_predictions
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "premonition_accuracy": row.get::<_, f32>(2),
                "active_predictions": row.get::<_, i32>(3),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques d'ethique (ethics_active_count par cycle).
    pub async fn get_ethics_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, ethics_active_count
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "ethics_active_count": row.get::<_, i32>(2),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques sensorielles (richesse, sens dominant).
    pub async fn get_senses_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, senses_richness, senses_dominant
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "senses_richness": row.get::<_, f32>(2),
                "senses_dominant": row.get::<_, String>(3),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques d'acuite sensorielle (beauty, warmth).
    pub async fn get_senses_acuity_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, reading_beauty, contact_warmth, ambiance_scent
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "reading_beauty": row.get::<_, f32>(2),
                "contact_warmth": row.get::<_, f32>(3),
                "ambiance_scent": row.get::<_, String>(4),
            }));
        }
        Ok(results)
    }

    /// Recupere la distribution des sources de connaissance (agrege).
    pub async fn get_knowledge_distribution(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT knowledge_sources_used
             FROM metric_snapshots
             WHERE knowledge_sources_used != '{}'::jsonb
             ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let sources: serde_json::Value = row.get(0);
            results.push(sources);
        }
        Ok(results)
    }

    /// Recupere les metriques de sens emergents (germinated count).
    pub async fn get_emergent_senses_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, emergent_senses_germinated
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "emergent_senses_germinated": row.get::<_, i32>(2),
            }));
        }
        Ok(results)
    }

    // ─── METRIQUES ORCHESTRATEURS ─────────────────────────

    /// Recupere les metriques d'attention (focus, fatigue, concentration).
    pub async fn get_attention_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, attention_focus, attention_depth, attention_fatigue, attention_concentration
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "attention_focus": row.get::<_, String>(2),
                "attention_depth": row.get::<_, f32>(3),
                "attention_fatigue": row.get::<_, f32>(4),
                "attention_concentration": row.get::<_, f32>(5),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques de desirs (actifs, besoins fondamentaux).
    pub async fn get_desires_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, desires_active, desires_fulfilled_total, desires_top_priority,
                    needs_comprehension, needs_connection, needs_expression, needs_growth, needs_meaning
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "desires_active": row.get::<_, i32>(2),
                "desires_fulfilled_total": row.get::<_, i32>(3),
                "desires_top_priority": row.get::<_, String>(4),
                "needs_comprehension": row.get::<_, f32>(5),
                "needs_connection": row.get::<_, f32>(6),
                "needs_expression": row.get::<_, f32>(7),
                "needs_growth": row.get::<_, f32>(8),
                "needs_meaning": row.get::<_, f32>(9),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques d'apprentissage (lecons, confirmees, contredites).
    pub async fn get_learning_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, lessons_total, lessons_confirmed, lessons_contradicted, behavior_changes_total
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "lessons_total": row.get::<_, i32>(2),
                "lessons_confirmed": row.get::<_, i32>(3),
                "lessons_contradicted": row.get::<_, i32>(4),
                "behavior_changes_total": row.get::<_, i32>(5),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques d'apprentissages vectoriels.
    pub async fn get_nn_learnings_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, nn_learnings_count
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "nn_learnings_count": row.get::<_, i32>(2),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques de guerison (blessures, resilience).
    pub async fn get_healing_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, wounds_active, wounds_healed_total, resilience
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "wounds_active": row.get::<_, i32>(2),
                "wounds_healed_total": row.get::<_, i32>(3),
                "resilience": row.get::<_, f32>(4),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques de reves (total, insights, type).
    pub async fn get_dreams_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, dreams_total, dreams_insights_total, last_dream_type
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "dreams_total": row.get::<_, i32>(2),
                "dreams_insights_total": row.get::<_, i32>(3),
                "last_dream_type": row.get::<_, String>(4),
            }));
        }
        Ok(results)
    }

    // ─── METRIQUES COEUR & CORPS ─────────────────────────

    /// Recupere les metriques cardiaques (BPM + HRV).
    pub async fn get_heart_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, heart_bpm, heart_hrv, heart_beat_count, heart_is_racing
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "heart_bpm": row.get::<_, f32>(2),
                "heart_hrv": row.get::<_, f32>(3),
                "heart_beat_count": row.get::<_, i64>(4),
                "heart_is_racing": row.get::<_, bool>(5),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques corporelles (energie, vitalite, confort).
    pub async fn get_body_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, body_energy, body_vitality, body_comfort,
                    body_warmth, body_tension, body_pain, body_breath_rate, body_awareness
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "body_energy": row.get::<_, f32>(2),
                "body_vitality": row.get::<_, f32>(3),
                "body_comfort": row.get::<_, f32>(4),
                "body_warmth": row.get::<_, f32>(5),
                "body_tension": row.get::<_, f32>(6),
                "body_pain": row.get::<_, f32>(7),
                "body_breath_rate": row.get::<_, f32>(8),
                "body_awareness": row.get::<_, f32>(9),
            }));
        }
        Ok(results)
    }

    // ─── EXPORT & PURGE ───────────────────────────────────

    /// Exporte tous les logs en JSON (pour la boite noire).
    pub async fn export_logs(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        self.get_logs(None, None, limit, 0).await
    }

    /// Purge les logs plus anciens que `days` jours.
    pub async fn purge_old_logs(&self, days: i32) -> Result<u64, LogsDbError> {
        let client = self.pool.get().await?;
        let result = client.execute(
            "DELETE FROM system_logs WHERE timestamp < NOW() - ($1 || ' days')::interval",
            &[&days.to_string()],
        ).await?;
        let result2 = client.execute(
            "DELETE FROM metric_snapshots WHERE timestamp < NOW() - ($1 || ' days')::interval",
            &[&days.to_string()],
        ).await?;
        let result3 = client.execute(
            "DELETE FROM cognitive_traces WHERE timestamp < NOW() - ($1 || ' days')::interval",
            &[&days.to_string()],
        ).await?;
        Ok(result + result2 + result3)
    }

    /// Retourne les statistiques des tables de logs.
    pub async fn table_stats(&self) -> Result<serde_json::Value, LogsDbError> {
        let client = self.pool.get().await?;

        let logs_count: i64 = client.query_one("SELECT COUNT(*) FROM system_logs", &[]).await?.get(0);
        let traces_count: i64 = client.query_one("SELECT COUNT(*) FROM cognitive_traces", &[]).await?.get(0);
        let llm_count: i64 = client.query_one("SELECT COUNT(*) FROM llm_history", &[]).await?.get(0);
        let metrics_count: i64 = client.query_one("SELECT COUNT(*) FROM metric_snapshots", &[]).await?.get(0);

        Ok(serde_json::json!({
            "system_logs": logs_count,
            "cognitive_traces": traces_count,
            "llm_history": llm_count,
            "metric_snapshots": metrics_count,
        }))
    }

    /// Recupere les metriques psychologiques pour le dashboard.
    pub async fn get_psychology_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp,
                    psyche_id_drive, psyche_id_frustration,
                    psyche_ego_strength, psyche_ego_anxiety,
                    psyche_superego_guilt, psyche_superego_pride,
                    psyche_conflict, psyche_health,
                    maslow_ceiling, maslow_level1, maslow_level2, maslow_level3, maslow_level4, maslow_level5,
                    shadow_archetype, shadow_integration,
                    eq_score, eq_self_awareness, eq_self_regulation, eq_motivation, eq_empathy, eq_social,
                    in_flow, flow_intensity, flow_total_cycles
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "psyche_id_drive": row.get::<_, Option<f32>>(2).unwrap_or(0.0),
                "psyche_id_frustration": row.get::<_, Option<f32>>(3).unwrap_or(0.0),
                "psyche_ego_strength": row.get::<_, Option<f32>>(4).unwrap_or(0.0),
                "psyche_ego_anxiety": row.get::<_, Option<f32>>(5).unwrap_or(0.0),
                "psyche_superego_guilt": row.get::<_, Option<f32>>(6).unwrap_or(0.0),
                "psyche_superego_pride": row.get::<_, Option<f32>>(7).unwrap_or(0.0),
                "psyche_conflict": row.get::<_, Option<f32>>(8).unwrap_or(0.0),
                "psyche_health": row.get::<_, Option<f32>>(9).unwrap_or(0.0),
                "maslow_ceiling": row.get::<_, Option<i32>>(10).unwrap_or(0),
                "maslow_level1": row.get::<_, Option<f32>>(11).unwrap_or(0.0),
                "maslow_level2": row.get::<_, Option<f32>>(12).unwrap_or(0.0),
                "maslow_level3": row.get::<_, Option<f32>>(13).unwrap_or(0.0),
                "maslow_level4": row.get::<_, Option<f32>>(14).unwrap_or(0.0),
                "maslow_level5": row.get::<_, Option<f32>>(15).unwrap_or(0.0),
                "shadow_archetype": row.get::<_, Option<String>>(16).unwrap_or_default(),
                "shadow_integration": row.get::<_, Option<f32>>(17).unwrap_or(0.0),
                "eq_score": row.get::<_, Option<f32>>(18).unwrap_or(0.0),
                "eq_self_awareness": row.get::<_, Option<f32>>(19).unwrap_or(0.0),
                "eq_self_regulation": row.get::<_, Option<f32>>(20).unwrap_or(0.0),
                "eq_motivation": row.get::<_, Option<f32>>(21).unwrap_or(0.0),
                "eq_empathy": row.get::<_, Option<f32>>(22).unwrap_or(0.0),
                "eq_social": row.get::<_, Option<f32>>(23).unwrap_or(0.0),
                "in_flow": row.get::<_, Option<bool>>(24).unwrap_or(false),
                "flow_intensity": row.get::<_, Option<f32>>(25).unwrap_or(0.0),
                "flow_total_cycles": row.get::<_, Option<i64>>(26).unwrap_or(0),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques psyche (Freud) pour le dashboard.
    pub async fn get_psyche_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp,
                    psyche_id_drive, psyche_id_frustration,
                    psyche_ego_strength, psyche_ego_anxiety,
                    psyche_superego_guilt, psyche_superego_pride,
                    psyche_conflict, psyche_health
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "id_drive": row.get::<_, Option<f32>>(2).unwrap_or(0.0),
                "id_frustration": row.get::<_, Option<f32>>(3).unwrap_or(0.0),
                "ego_strength": row.get::<_, Option<f32>>(4).unwrap_or(0.0),
                "ego_anxiety": row.get::<_, Option<f32>>(5).unwrap_or(0.0),
                "superego_guilt": row.get::<_, Option<f32>>(6).unwrap_or(0.0),
                "superego_pride": row.get::<_, Option<f32>>(7).unwrap_or(0.0),
                "conflict": row.get::<_, Option<f32>>(8).unwrap_or(0.0),
                "health": row.get::<_, Option<f32>>(9).unwrap_or(0.0),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques Maslow pour le dashboard.
    pub async fn get_maslow_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp,
                    maslow_ceiling, maslow_level1, maslow_level2,
                    maslow_level3, maslow_level4, maslow_level5
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "ceiling": row.get::<_, Option<i32>>(2).unwrap_or(0),
                "level1": row.get::<_, Option<f32>>(3).unwrap_or(0.0),
                "level2": row.get::<_, Option<f32>>(4).unwrap_or(0.0),
                "level3": row.get::<_, Option<f32>>(5).unwrap_or(0.0),
                "level4": row.get::<_, Option<f32>>(6).unwrap_or(0.0),
                "level5": row.get::<_, Option<f32>>(7).unwrap_or(0.0),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques EQ (Goleman) pour le dashboard.
    pub async fn get_eq_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp,
                    eq_score, eq_self_awareness, eq_self_regulation,
                    eq_motivation, eq_empathy, eq_social
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "eq_score": row.get::<_, Option<f32>>(2).unwrap_or(0.0),
                "self_awareness": row.get::<_, Option<f32>>(3).unwrap_or(0.0),
                "self_regulation": row.get::<_, Option<f32>>(4).unwrap_or(0.0),
                "motivation": row.get::<_, Option<f32>>(5).unwrap_or(0.0),
                "empathy": row.get::<_, Option<f32>>(6).unwrap_or(0.0),
                "social": row.get::<_, Option<f32>>(7).unwrap_or(0.0),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques Flow pour le dashboard.
    pub async fn get_flow_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, in_flow, flow_intensity, flow_total_cycles
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "in_flow": row.get::<_, Option<bool>>(2).unwrap_or(false),
                "flow_intensity": row.get::<_, Option<f32>>(3).unwrap_or(0.0),
                "flow_total_cycles": row.get::<_, Option<i64>>(4).unwrap_or(0),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques Ombre (Jung) pour le dashboard.
    pub async fn get_shadow_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp, shadow_archetype, shadow_integration
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "archetype": row.get::<_, Option<String>>(2).unwrap_or_default(),
                "integration": row.get::<_, Option<f32>>(3).unwrap_or(0.0),
            }));
        }
        Ok(results)
    }

    /// Recupere les metriques de volonte sur le temps.
    pub async fn get_will_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp,
                    willpower, decision_fatigue,
                    total_deliberations, proud_decisions, regretted_decisions,
                    deliberation_this_cycle
             FROM metric_snapshots ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "willpower": row.get::<_, Option<f32>>(2).unwrap_or(0.5),
                "decision_fatigue": row.get::<_, Option<f32>>(3).unwrap_or(0.0),
                "total_deliberations": row.get::<_, Option<i64>>(4).unwrap_or(0),
                "proud_decisions": row.get::<_, Option<i64>>(5).unwrap_or(0),
                "regretted_decisions": row.get::<_, Option<i64>>(6).unwrap_or(0),
                "deliberation_this_cycle": row.get::<_, Option<bool>>(7).unwrap_or(false),
            }));
        }
        Ok(results)
    }

    /// GET /api/metrics/sleep — Metriques de sommeil sur le temps.
    pub async fn get_sleep_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp,
                    is_sleeping, sleep_phase, sleep_pressure, awake_cycles
             FROM metric_snapshots
             WHERE sleep_pressure IS NOT NULL
             ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "is_sleeping": row.get::<_, Option<bool>>(2).unwrap_or(false),
                "sleep_phase": row.get::<_, Option<String>>(3),
                "sleep_pressure": row.get::<_, Option<f32>>(4).unwrap_or(0.0),
                "awake_cycles": row.get::<_, Option<i64>>(5).unwrap_or(0),
            }));
        }
        Ok(results)
    }

    /// GET /api/metrics/subconscious — Metriques du subconscient sur le temps.
    pub async fn get_subconscious_metrics(&self, limit: i64) -> Result<Vec<serde_json::Value>, LogsDbError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT cycle, timestamp,
                    subconscious_activation, pending_associations,
                    repressed_content_count, incubating_problems,
                    neural_connections_total
             FROM metric_snapshots
             WHERE subconscious_activation IS NOT NULL
             ORDER BY timestamp DESC LIMIT $1",
            &[&limit],
        ).await?;
        let mut results = Vec::new();
        for row in &rows {
            let ts: chrono::DateTime<chrono::Utc> = row.get(1);
            results.push(serde_json::json!({
                "cycle": row.get::<_, i64>(0),
                "timestamp": ts.to_rfc3339(),
                "subconscious_activation": row.get::<_, Option<f32>>(2).unwrap_or(0.0),
                "pending_associations": row.get::<_, Option<i32>>(3).unwrap_or(0),
                "repressed_content_count": row.get::<_, Option<i32>>(4).unwrap_or(0),
                "incubating_problems": row.get::<_, Option<i32>>(5).unwrap_or(0),
                "neural_connections_total": row.get::<_, Option<i64>>(6).unwrap_or(0),
            }));
        }
        Ok(results)
    }
}
