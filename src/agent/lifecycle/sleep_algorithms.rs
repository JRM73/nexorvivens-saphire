// =============================================================================
// lifecycle/sleep_algorithms.rs — Algorithms during sleep and vectorization
//
// Role: Executes ML algorithms during sleep phases (K-Means,
// PCA, cosine similarity, association rules, sentiment) and vectorizes
// dreams, neural connections and subconscious insights in pgvector.
// =============================================================================

use crate::algorithms::orchestrator::AlgorithmInput;
use crate::db::vectors::{VectorSource, NewMemoryVector};
use crate::logging::{LogLevel, LogCategory};
use crate::psychology::subconscious::SubconsciousInsight;
use super::SaphireAgent;

impl SaphireAgent {
    // ─── Algorithms by sleep phase ────────────────────────────
    /// Light sleep: K-Means on episodic memories + chemistry anomaly detection
    pub(super) async fn sleep_light_algorithms(&mut self) {
        if !self.config.sleep.algorithms.enabled || !self.orchestrator.enabled {
            return;
        }

        let algo_cfg = self.config.sleep.algorithms.clone();

        // K-Means on recent episodic memories
        if let Some(ref db) = self.db {
            match db.recent_episodic(100).await {
                Ok(episodes) if episodes.len() >= 10 => {
                    // Build 5D vectors: emotion_numeric, intensity, satisfaction, strength, access/10
                    let vectors: Vec<Vec<f64>> = episodes.iter().map(|ep| {
                        vec![
                            Self::emotion_to_numeric(&ep.emotion),
                            ep.emotional_intensity as f64,
                            ep.satisfaction as f64,
                            ep.strength as f64,
                            ep.access_count as f64 / 10.0,
                        ]
                    }).collect();

                    let input = AlgorithmInput {
                        vectors: Some(vectors),
                        params: [("k".into(), algo_cfg.light_kmeans_k as f64)].into(),
                        ..Default::default()
                    };

                    match self.orchestrator.execute_auto("kmeans", input) {
                        Ok(output) => {
                            self.sleep_last_clusters = Some(output.structured_result.clone());
                            self.log(LogLevel::Info, LogCategory::Sleep,
                                format!("Sommeil leger — clustering : {}", output.natural_language_result),
                                serde_json::json!({"algorithm": "kmeans", "sleep_phase": "light"}));
                        }
                        Err(e) => {
                            self.log(LogLevel::Debug, LogCategory::Sleep,
                                format!("Sommeil leger — kmeans echoue : {}", e),
                                serde_json::json!({}));
                        }
                    }
                }
                _ => {}
            }
        }

        // IsolationForest on chemical history
        if self.chemistry_history.len() >= 10 {
            let vectors: Vec<Vec<f64>> = self.chemistry_history.iter()
                .map(|h| h.to_vec())
                .collect();
            let input = AlgorithmInput {
                vectors: Some(vectors),
                ..Default::default()
            };

            match self.orchestrator.execute_auto("isolation_forest", input) {
                Ok(output) => {
                    if let Some(anomalies) = output.structured_result.get("anomaly_count")
                        .and_then(|v| v.as_u64())
                    {
                        if anomalies > 0 {
                            self.log(LogLevel::Info, LogCategory::Sleep,
                                format!("Sommeil leger — {} anomalies chimiques detectees", anomalies),
                                serde_json::json!({
                                    "algorithm": "isolation_forest",
                                    "anomaly_count": anomalies,
                                    "sleep_phase": "light",
                                }));
                        }
                    }
                }
                Err(_) => {}
            }
        }
    }

    /// Deep sleep: PCA chemistry + cosine similarity connections + association rules
    pub(super) async fn sleep_deep_algorithms(&mut self) {
        if !self.config.sleep.algorithms.enabled || !self.orchestrator.enabled {
            return;
        }

        let algo_cfg = self.config.sleep.algorithms.clone();
        let vec_cfg = self.config.subconscious.vectors.clone();

        // PCA on chemical history (>=30 points, 7D -> 3 components)
        if self.chemistry_history.len() >= 30 {
            let vectors: Vec<Vec<f64>> = self.chemistry_history.iter()
                .map(|h| h.to_vec())
                .collect();
            let input = AlgorithmInput {
                vectors: Some(vectors),
                params: [("n_components".into(), algo_cfg.deep_pca_components as f64)].into(),
                ..Default::default()
            };

            match self.orchestrator.execute_auto("pca", input) {
                Ok(output) => {
                    self.log(LogLevel::Info, LogCategory::Sleep,
                        format!("Sommeil profond — PCA : {}", output.natural_language_result),
                        serde_json::json!({"algorithm": "pca", "sleep_phase": "deep"}));
                }
                Err(_) => {}
            }
        }

        // Cosine similarity between recent LTM memories → create connections
        // Collect memories first to avoid borrow conflicts
        let memories_opt = if let Some(ref db) = self.db {
            db.recent_memories(50).await.ok()
        } else { None };

        if let Some(memories) = memories_opt {
            if memories.len() >= 5 {
                // Encode each memory
                let encoded: Vec<(i64, Vec<f64>, String)> = memories.iter().map(|m| {
                    let vec = self.encoder.encode(&m.text_summary);
                    (m.id, vec, m.text_summary.clone())
                }).collect();

                let threshold = algo_cfg.deep_connection_similarity_threshold;
                let max_conn = algo_cfg.deep_max_connections_per_phase;
                let mut conn_count = 0u64;

                // Collect connections to create
                let mut connections_to_make: Vec<(usize, usize, f64, &str)> = Vec::new();
                for i in 0..encoded.len() {
                    if connections_to_make.len() as u64 >= max_conn { break; }
                    for j in (i+1)..encoded.len() {
                        if connections_to_make.len() as u64 >= max_conn { break; }
                        let sim = crate::vectorstore::cosine_similarity(&encoded[i].1, &encoded[j].1);
                        if sim > threshold && sim < 0.95 {
                            let link_type = if sim > 0.8 { "forte_ressemblance" }
                                else { "similarite_thematique" };
                            connections_to_make.push((i, j, sim, link_type));
                        }
                    }
                }

                // Insert the connections
                for (i, j, sim, link_type) in connections_to_make {
                    if let Some(ref db) = self.db {
                        match db.insert_neural_connection(
                            encoded[i].0, encoded[j].0,
                            sim as f32, link_type, None, true, "sommeil_profond",
                        ).await {
                            Ok(conn_id) => {
                                conn_count += 1;
                                if let Some(ref mut c) = self.sleep.current_cycle {
                                    c.connections_created += 1;
                                }
                                self.sleep.total_connections_created += 1;
                                // Vectorize the connection
                                if vec_cfg.enabled && vec_cfg.vectorize_connections {
                                    self.vectorize_neural_connection(
                                        &encoded[i].2, &encoded[j].2, link_type, conn_id,
                                    ).await;
                                }
                            }
                            Err(_) => {}
                        }
                    }
                }

                if conn_count > 0 {
                    self.log(LogLevel::Info, LogCategory::Sleep,
                        format!("Sommeil profond — {} connexions neuronales creees", conn_count),
                        serde_json::json!({
                            "connections_created": conn_count,
                            "sleep_phase": "deep",
                        }));
                }
            }
        }

        // Association rules on recent episodes
        let episodes_opt = if let Some(ref db) = self.db {
            db.recent_episodic(200).await.ok()
        } else { None };

        if let Some(episodes) = episodes_opt {
            if episodes.len() >= 20 {
                let texts: Vec<String> = episodes.iter()
                    .map(|ep| format!("{}>{}",
                        ep.source_type.chars().take(15).collect::<String>(),
                        ep.emotion.chars().take(15).collect::<String>()))
                    .collect();
                let input = AlgorithmInput {
                    texts: Some(texts),
                    ..Default::default()
                };

                match self.orchestrator.execute_auto("association_rules", input) {
                    Ok(output) => {
                        self.log(LogLevel::Debug, LogCategory::Sleep,
                            format!("Sommeil profond — regles d'association : {}", output.natural_language_result),
                            serde_json::json!({"algorithm": "association_rules", "sleep_phase": "deep"}));
                    }
                    Err(_) => {}
                }
            }
        }
    }

    /// REM: sentiment analysis on recent dreams
    pub(super) async fn sleep_rem_algorithms(&mut self) {
        if !self.config.sleep.algorithms.enabled || !self.orchestrator.enabled {
            return;
        }

        let min_dreams = self.config.sleep.algorithms.rem_sentiment_min_dreams;

        // Sentiment on dreams from the journal
        let dream_count = self.dream_orch.dream_journal.len();
        if dream_count >= min_dreams {
            let texts: Vec<String> = self.dream_orch.dream_journal.iter()
                .rev().take(5)
                .map(|d| d.dream.narrative.clone())
                .collect();

            let input = AlgorithmInput {
                texts: Some(texts),
                ..Default::default()
            };

            match self.orchestrator.execute_auto("sentiment_vader", input) {
                Ok(output) => {
                    self.log(LogLevel::Info, LogCategory::Sleep,
                        format!("REM — sentiment des reves : {}", output.natural_language_result),
                        serde_json::json!({"algorithm": "sentiment_vader", "sleep_phase": "rem"}));
                }
                Err(_) => {}
            }
        }
    }

    // ─── Subconscious algorithms (awake) ────────────────────────────
    /// DBSCAN on episodic memories (every 100 cycles)
    pub(super) async fn subconscious_dbscan(&mut self) {
        if !self.orchestrator.enabled { return; }

        if let Some(ref db) = self.db {
            match db.recent_episodic(200).await {
                Ok(episodes) if episodes.len() >= 20 => {
                    let vectors: Vec<Vec<f64>> = episodes.iter().map(|ep| {
                        vec![
                            Self::emotion_to_numeric(&ep.emotion),
                            ep.emotional_intensity as f64,
                            ep.satisfaction as f64,
                            ep.strength as f64,
                            ep.access_count as f64 / 10.0,
                        ]
                    }).collect();

                    let input = AlgorithmInput {
                        vectors: Some(vectors),
                        params: [
                            ("eps".into(), 0.5),
                            ("min_points".into(), 3.0),
                        ].into(),
                        ..Default::default()
                    };

                    match self.orchestrator.execute_auto("dbscan", input) {
                        Ok(output) => {
                            // If lots of noise, generate an insight
                            if let Some(noise) = output.structured_result.get("noise_count")
                                .and_then(|v| v.as_u64())
                            {
                                if noise > 5 {
                                    let content = format!(
                                        "DBSCAN detecte {} souvenirs non-classes (bruit) — experiences atypiques a integrer",
                                        noise
                                    );
                                    self.subconscious.ready_insights.push(
                                        SubconsciousInsight {
                                            content,
                                            source_type: "algorithme_subconscient".into(),
                                            strength: 0.6,
                                            emotional_charge: 0.3,
                                        }
                                    );
                                }
                            }
                            self.log(LogLevel::Debug, LogCategory::Subconscious,
                                format!("Subconscient DBSCAN : {}", output.natural_language_result),
                                serde_json::json!({"algorithm": "dbscan"}));
                        }
                        Err(_) => {}
                    }
                }
                _ => {}
            }
        }
    }

    /// IsolationForest on chemistry (every 100 cycles, offset 50)
    pub(super) async fn subconscious_isolation_forest(&mut self) {
        if !self.orchestrator.enabled { return; }

        if self.chemistry_history.len() >= 10 {
            let vectors: Vec<Vec<f64>> = self.chemistry_history.iter()
                .map(|h| h.to_vec())
                .collect();
            let input = AlgorithmInput {
                vectors: Some(vectors),
                params: [("threshold".into(), 2.5)].into(),
                ..Default::default()
            };

            match self.orchestrator.execute_auto("isolation_forest", input) {
                Ok(output) => {
                    if let Some(anomalies) = output.structured_result.get("anomaly_count")
                        .and_then(|v| v.as_u64())
                    {
                        if anomalies > 0 {
                            let content = format!(
                                "IsolationForest detecte {} anomalies chimiques — etats inhabituels recents",
                                anomalies
                            );
                            self.subconscious.ready_insights.push(
                                SubconsciousInsight {
                                    content,
                                    source_type: "algorithme_subconscient".into(),
                                    strength: 0.5,
                                    emotional_charge: 0.4,
                                }
                            );
                        }
                    }
                    self.log(LogLevel::Debug, LogCategory::Subconscious,
                        format!("Subconscient IsolationForest : {}", output.natural_language_result),
                        serde_json::json!({"algorithm": "isolation_forest"}));
                }
                Err(_) => {}
            }
        }
    }

    // ─── Vectorization ───────────────────────────────────────────────
    /// Vectorizes a dream (during REM sleep)
    pub(super) async fn vectorize_dream(&mut self, narrative: &str, emotion: &str) {
        if !self.config.subconscious.vectors.enabled
            || !self.config.subconscious.vectors.vectorize_dreams
        {
            return;
        }

        let embedding_f64 = self.encoder.encode(narrative);
        let embedding_f32: Vec<f32> = embedding_f64.iter().map(|&v| v as f32).collect();

        let phase = self.sleep.current_cycle.as_ref()
            .map(|c| c.phase.as_str().to_string());

        let vec = NewMemoryVector {
            embedding: embedding_f32,
            source_type: VectorSource::Dream,
            text_content: narrative.chars().take(500).collect(),
            emotion: emotion.to_string(),
            strength: 0.8,
            created_during_sleep: true,
            sleep_phase: phase,
            source_ref_id: None,
            metadata_json: serde_json::json!({}),
        };

        if let Some(ref db) = self.db {
            match db.store_memory_vector(&vec).await {
                Ok(id) => {
                    self.log(LogLevel::Debug, LogCategory::Sleep,
                        format!("Reve vectorise (id={})", id),
                        serde_json::json!({"vector_id": id, "source": "dream"}));
                }
                Err(e) => {
                    self.log(LogLevel::Debug, LogCategory::Sleep,
                        format!("Erreur vectorisation reve : {}", e),
                        serde_json::json!({}));
                }
            }
        }
    }

    /// Vectorizes a neural connection (during deep sleep)
    pub(super) async fn vectorize_neural_connection(
        &mut self, a_text: &str, b_text: &str, link_type: &str, conn_id: i64,
    ) {
        if !self.config.subconscious.vectors.enabled
            || !self.config.subconscious.vectors.vectorize_connections
        {
            return;
        }

        let combined = format!("{} <-> {}", a_text, b_text);
        let embedding_f64 = self.encoder.encode(&combined);
        let embedding_f32: Vec<f32> = embedding_f64.iter().map(|&v| v as f32).collect();

        let vec = NewMemoryVector {
            embedding: embedding_f32,
            source_type: VectorSource::NeuralConnection,
            text_content: combined.chars().take(500).collect(),
            emotion: String::new(),
            strength: 0.7,
            created_during_sleep: true,
            sleep_phase: Some("deep".into()),
            source_ref_id: Some(conn_id),
            metadata_json: serde_json::json!({"link_type": link_type}),
        };

        if let Some(ref db) = self.db {
            let _ = db.store_memory_vector(&vec).await;
        }
    }

    /// Vectorizes a subconscious insight
    pub(super) async fn vectorize_insight(&mut self, content: &str, source: &str) {
        if !self.config.subconscious.vectors.enabled
            || !self.config.subconscious.vectors.vectorize_insights
        {
            return;
        }

        let embedding_f64 = self.encoder.encode(content);
        let embedding_f32: Vec<f32> = embedding_f64.iter().map(|&v| v as f32).collect();

        let is_sleeping = self.sleep.current_cycle.is_some();
        let phase = self.sleep.current_cycle.as_ref()
            .map(|c| c.phase.as_str().to_string());

        let vec = NewMemoryVector {
            embedding: embedding_f32,
            source_type: VectorSource::SubconsciousInsight,
            text_content: content.chars().take(500).collect(),
            emotion: String::new(),
            strength: 0.6,
            created_during_sleep: is_sleeping,
            sleep_phase: phase,
            source_ref_id: None,
            metadata_json: serde_json::json!({"source": source}),
        };

        if let Some(ref db) = self.db {
            let _ = db.store_memory_vector(&vec).await;
        }
    }

    /// Vectorizes a vivid mental image (vividness >= 0.6)
    pub(super) async fn vectorize_mental_image(
        &mut self, description: &str, vividness: f64,
        imagery_type: &str, concept: &str,
        emotional_charge: f64, emotion: &str, cycle: u64,
    ) {
        if !self.config.subconscious.vectors.enabled
            || !self.config.subconscious.vectors.vectorize_imagery
        {
            return;
        }

        let embedding_f64 = self.encoder.encode(description);
        let embedding_f32: Vec<f32> = embedding_f64.iter().map(|&v| v as f32).collect();

        let vec = NewMemoryVector {
            embedding: embedding_f32,
            source_type: VectorSource::MentalImagery,
            text_content: description.chars().take(500).collect(),
            emotion: emotion.to_string(),
            strength: vividness as f32,
            created_during_sleep: false,
            sleep_phase: None,
            source_ref_id: None,
            metadata_json: serde_json::json!({
                "imagery_type": imagery_type,
                "concept": concept,
                "emotional_charge": emotional_charge,
                "cycle": cycle,
            }),
        };

        if let Some(ref db) = self.db {
            match db.store_memory_vector(&vec).await {
                Ok(id) => {
                    self.log(LogLevel::Debug, LogCategory::Thought,
                        format!("Image mentale vectorisee (id={}, vivacite={:.0}%, type={})",
                            id, vividness * 100.0, imagery_type),
                        serde_json::json!({
                            "vector_id": id,
                            "source": "mental_imagery",
                            "vividness": vividness,
                            "imagery_type": imagery_type,
                        }));
                }
                Err(e) => {
                    self.log(LogLevel::Debug, LogCategory::Thought,
                        format!("Erreur vectorisation image mentale : {}", e),
                        serde_json::json!({}));
                }
            }
        }
    }

    // ─── Utilities ─────────────────────────────────────────────────
    /// Converts an emotion name to a numeric value for clustering
    fn emotion_to_numeric(emotion: &str) -> f64 {
        match emotion.to_lowercase().as_str() {
            "joie" | "joy" => 0.9,
            "curiosite" | "curiosité" => 0.8,
            "serenite" | "sérénité" | "calme" => 0.7,
            "surprise" => 0.6,
            "anticipation" => 0.5,
            "neutre" | "" => 0.5,
            "melancolie" | "mélancolie" => 0.3,
            "inquietude" | "inquiétude" => 0.25,
            "tristesse" => 0.2,
            "peur" => 0.15,
            "colere" | "colère" => 0.1,
            "douleur" => 0.05,
            _ => 0.5,
        }
    }
}
