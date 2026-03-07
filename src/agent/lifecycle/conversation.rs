// =============================================================================
// lifecycle/conversation.rs — Human message processing (lite version)
// =============================================================================
//
// Streamlined version for the ArXiv paper.
// Removed: sleep check, attention_orch, senses, tom, prospective_mem,
//          metacognition, vector encoder, ocean_context (profiling),
//          psychology::ownership, relationships, full save_metric_snapshot,
//          micro_nn learning, advanced broadcasts.
// =============================================================================

use std::sync::atomic::Ordering;
use tokio::time::Instant;

use crate::llm;
use crate::memory::WorkingItemSource;
use crate::logging::{LogLevel, LogCategory};

use super::SaphireAgent;
use super::truncate_utf8;

impl SaphireAgent {
    /// Processes a human message end-to-end and returns Saphire's response.
    ///
    /// Pipeline:
    /// 1. Immediate social bonus (human interaction is beneficial).
    /// 2. Inject the message into working memory.
    /// 3. NLP analysis of the text -> Stimulus creation.
    /// 4. Full brain pipeline (`process_stimulus`).
    /// 5. Build memory context (working memory + episodic + LTM).
    /// 6. LLM call -> response generation.
    /// 7. Store in working memory and episodic memory.
    /// 8. Decay, chemical homeostasis, WebSocket broadcast.
    ///
    /// # Parameters
    /// - `text` — the human's message text.
    /// - `username` — the name of the human interlocutor.
    ///
    /// # Returns
    /// The generated response text from Saphire.
    pub async fn handle_human_message(&mut self, text: &str, username: &str) -> String {
        let cycle_start = Instant::now();
        self.log(LogLevel::Info, LogCategory::Cycle,
            format!("Message humain recu ({} chars)", text.len()),
            serde_json::json!({"preview": text.chars().take(100).collect::<String>()}));

        // === FIRST MESSAGE — contact bonus ===
        if !self.in_conversation {
            self.chemistry.oxytocin = (self.chemistry.oxytocin + 0.05).min(1.0);
            self.chemistry.serotonin = (self.chemistry.serotonin + 0.03).min(1.0);
            self.chemistry.cortisol = (self.chemistry.cortisol - 0.03).max(0.0);
            self.in_conversation = true;
            self.conversation_id = Some(format!("conv_{}", chrono::Utc::now().timestamp()));
        }

        // === RLHF FEEDBACK ===
        if let Some(feedback) = self.feedback_pending.take() {
            let positive = super::thinking::is_positive_feedback(text);
            let boost = if positive {
                self.config.human_feedback.boost_positive
            } else {
                0.0
            };

            if boost > 0.0 {
                self.thought_engine.update_reward(&feedback.thought_type, boost);
                self.chemistry.dopamine = (self.chemistry.dopamine + 0.05).min(1.0);
                self.chemistry.serotonin = (self.chemistry.serotonin + 0.03).min(1.0);
            }

            self.broadcast_feedback_result(positive, boost);

            self.log(LogLevel::Info, LogCategory::Cycle,
                format!("Feedback RLHF recu: {} (boost={:.2})",
                    if positive { "positif" } else { "negatif/neutre" }, boost),
                serde_json::json!({
                    "positive": positive,
                    "boost": boost,
                    "thought_type": format!("{:?}", feedback.thought_type),
                    "auto_reward": feedback.auto_reward,
                }));

            // Update the human feedback in the latest LoRA sample
            if self.config.lora.enabled {
                if let Some(ref db) = self.db {
                    let _ = db.pool.get().await.map(|client| {
                        let positive_val = positive;
                        tokio::spawn(async move {
                            let _ = client.execute(
                                "UPDATE lora_training_data SET human_feedback = $1
                                 WHERE id = (SELECT id FROM lora_training_data ORDER BY id DESC LIMIT 1)",
                                &[&positive_val],
                            ).await;
                        });
                    });
                }
            }
        }

        // === Working memory: push the human message ===
        let chem_sig = crate::neurochemistry::ChemicalSignature::from(&self.chemistry);
        let wm_ejected = self.working_memory.push(
            text.chars().take(200).collect(),
            WorkingItemSource::UserMessage(text.to_string()),
            self.last_emotion.clone(),
            chem_sig,
        );
        if let (Some(ejected), Some(ref db)) = (wm_ejected, &self.db) {
            let satisfaction = ((self.mood.valence + 1.0) / 2.0) as f32;
            let _ = db.store_episodic(
                &ejected.content, ejected.source.label(),
                &serde_json::json!({}), 0, &serde_json::json!({}),
                &ejected.emotion_at_creation, satisfaction, (self.mood.arousal as f32).max(0.3),
                self.conversation_id.as_deref(),
                Some(&ejected.chemical_signature),
            ).await;
        }

        // Step 1: NLP analysis
        let nlp_result = self.nlp.analyze(text);
        let mut stimulus = nlp_result.stimulus.clone();
        stimulus.apply_human_source_adjustments();

        // === CONDITIONAL SOCIAL BONUS ===
        let sentiment_compound = nlp_result.sentiment.compound;
        if sentiment_compound > 0.2 {
            self.chemistry.boost(crate::neurochemistry::Molecule::Oxytocin, 0.05);
            self.chemistry.boost(crate::neurochemistry::Molecule::Dopamine, 0.03);
            self.chemistry.cortisol = (self.chemistry.cortisol - 0.03).max(0.0);
        } else if sentiment_compound < -0.2 {
            let severity = (-sentiment_compound).min(1.0);
            self.chemistry.feedback_negative(severity * 0.5);
        } else {
            self.chemistry.boost(crate::neurochemistry::Molecule::Oxytocin, 0.02);
        }

        // Steps 2-7: brain pipeline
        let mut result = self.process_stimulus(&stimulus);

        // === Build memory context ===
        // Lite version: simple text search (without vector encoder)
        let wm_summary = self.working_memory.context_summary();
        let ep_limit = self.config.memory.recall_episodic_limit as i64;
        let episodic_recent = if let Some(ref db) = self.db {
            db.recent_episodic(ep_limit).await.unwrap_or_default()
        } else {
            vec![]
        };
        // LTM search by text (simple, without embedding vectors)
        let ltm_limit = self.config.memory.recall_ltm_limit as i64;
        let ltm_threshold = self.config.memory.recall_ltm_threshold;
        let mut ltm_similar = if let Some(ref db) = self.db {
            db.search_similar_memories_by_text(text, ltm_limit, ltm_threshold).await.unwrap_or_default()
        } else {
            vec![]
        };
        // Re-ranking by chemical similarity (state-dependent memory)
        if !ltm_similar.is_empty() {
            crate::memory::recall::recall_with_chemical_context(
                &mut ltm_similar, &self.chemistry, 0.8, 0.2,
            );
        }
        // Archive search by text
        let arc_limit = self.config.memory.recall_archive_limit as i64;
        let arc_threshold = self.config.memory.recall_archive_threshold;
        let archive_similar = if let Some(ref db) = self.db {
            db.search_similar_archives_by_text(text, arc_limit, arc_threshold).await.unwrap_or_default()
        } else {
            vec![]
        };
        // Build the memory context (without subconscious vectors)
        let memory_context = crate::memory::build_memory_context(
            &wm_summary, &episodic_recent, &ltm_similar, &archive_similar,
            &[],
        );

        // Step 8: LLM call
        let (response, llm_elapsed_ms) = if !self.llm_busy.load(Ordering::Relaxed) {
            self.llm_busy.store(true, Ordering::Relaxed);

            let world_summary = self.world.summary();
            let body_ctx = self.build_body_context();
            let ethics_ctx = self.ethics.build_ethics_context();
            let vital_ctx = self.build_vital_context();
            let system_prompt = llm::build_substrate_prompt(
                &self.chemistry,
                &result.emotion,
                &result.consciousness,
                &result.consensus,
                &self.identity.self_description,
                &ethics_ctx,
                &world_summary,
                &memory_context,
                &body_ctx,
                &vital_ctx,
                "",  // no sensory context in the lite version
                &self.config.general.language,
            );

            let mut system_prompt = if username != "Inconnu" {
                format!(
                    "{}\n\nINTERLOCUTEUR : Tu parles avec {}. Adresse-toi a cette personne par son prenom.",
                    system_prompt, username
                )
            } else {
                system_prompt
            };

            let llm_config = self.config.llm.clone();
            let start = Instant::now();
            let msg = text.to_string();

            // Detect conversational stagnation
            let conv_stagnating = self.detect_conversation_stagnation();
            let temp = if conv_stagnating {
                let boosted = (llm_config.temperature + 0.35).min(1.2);
                system_prompt = format!(
                    "{}\n\n ALERTE ANTI-REPETITION : reformule completement, change d'angle.",
                    system_prompt
                );
                self.log(LogLevel::Warn, LogCategory::Llm,
                    format!("Stagnation conversationnelle detectee — temp boost {:.2} → {:.2}",
                        llm_config.temperature, boosted),
                    serde_json::json!({"original_temp": llm_config.temperature, "boosted_temp": boosted}));
                boosted
            } else {
                llm_config.temperature
            };

            let backend = llm::create_backend(&llm_config);
            let max_tokens = llm_config.max_tokens;
            let history = self.chat_history.clone();

            let resp = tokio::task::spawn_blocking(move || {
                backend.chat_with_history(&system_prompt, &msg, &history, temp, max_tokens)
            }).await;

            let elapsed = start.elapsed().as_secs_f64();
            let elapsed_ms = (elapsed * 1000.0) as f32;
            self.avg_response_time = self.avg_response_time * 0.9 + elapsed * 0.1;
            self.llm_busy.store(false, Ordering::Relaxed);

            let text_resp = match resp {
                Ok(Ok(r)) => r,
                Ok(Err(e)) => format!("[Erreur LLM : {}]", e),
                Err(e) => format!("[Erreur tâche : {}]", e),
            };
            (text_resp, elapsed_ms)
        } else {
            ("[Mon esprit est occupé, un instant...]".to_string(), 0.0)
        };

        // Store the response for anti-repetition detection (max 5)
        self.recent_responses.push(response.clone());
        if self.recent_responses.len() > 5 {
            self.recent_responses.remove(0);
        }

        // Multi-turn history (max 5 exchanges)
        self.chat_history.push((text.to_string(), response.clone()));
        if self.chat_history.len() > 5 {
            self.chat_history.remove(0);
        }

        // Log LLM call history
        self.log(LogLevel::Info, LogCategory::Llm,
            format!("Reponse LLM ({} chars, {:.1}s)", response.len(), self.avg_response_time),
            serde_json::json!({}));
        if let Some(ref logs_db) = self.logs_db {
            let db = logs_db.clone();
            let cycle = self.cycle_count;
            let model = self.config.llm.model.clone();
            let resp_clone = response.clone();
            let elapsed_ms = (self.avg_response_time * 1000.0) as f32;
            let session_id = self.session_id;
            let temp = self.config.llm.temperature as f32;
            let max_tok = self.config.llm.max_tokens as i32;
            let user_text = text.to_string();
            tokio::spawn(async move {
                let _ = db.save_llm_history(
                    cycle, "conversation", &model, "(substrat)", &user_text,
                    &resp_clone, temp, max_tok, elapsed_ms,
                    true, "", session_id,
                ).await;
            });
        }

        // === Working memory: push the response ===
        let resp_preview: String = response.chars().take(200).collect();
        let chem_sig_resp = crate::neurochemistry::ChemicalSignature::from(&self.chemistry);
        let _ = self.working_memory.push(
            resp_preview,
            WorkingItemSource::LlmResponse(response.clone()),
            result.emotion.dominant.clone(),
            chem_sig_resp.clone(),
        );

        // Store in thought_log and episodic memory
        if let Some(ref db) = self.db {
            let chemistry_json = serde_json::json!({
                "dopamine": self.chemistry.dopamine,
                "cortisol": self.chemistry.cortisol,
                "serotonin": self.chemistry.serotonin,
            });
            let _ = db.log_thought(
                "conversation",
                text,
                &result.emotion.dominant,
                result.consciousness.level as f32,
                result.consciousness.phi as f32,
                self.mood.valence as f32,
                &chemistry_json,
            ).await;

            let exchange = format!("Humain: {} -> Saphire: {}",
                truncate_utf8(text, 300),
                truncate_utf8(&response, 300));
            let _ = db.store_episodic(
                &exchange, "conversation",
                &serde_json::json!({}), result.consensus.decision.as_i8() as i16,
                &chemistry_json, &result.emotion.dominant,
                if result.consensus.coherence > 0.5 { 0.7 } else { 0.4 },
                result.emotion.arousal as f32,
                self.conversation_id.as_deref(),
                Some(&chem_sig_resp),
            ).await;
        }

        // === Working memory decay ===
        let wm_decayed = self.working_memory.decay();
        if let Some(ref db) = self.db {
            let arousal = self.mood.arousal as f32;
            let satisfaction = ((self.mood.valence + 1.0) / 2.0) as f32;
            for item in wm_decayed {
                let _ = db.store_episodic(
                    &item.content, item.source.label(),
                    &serde_json::json!({}), 0, &serde_json::json!({}),
                    &item.emotion_at_creation, satisfaction, arousal.max(0.3),
                    self.conversation_id.as_deref(),
                    Some(&item.chemical_signature),
                ).await;
            }
        }

        // Chemical homeostasis (config.feedback.homeostasis_rate)
        self.chemistry.homeostasis(&self.baselines, self.config.feedback.homeostasis_rate);

        // Simplified metric snapshot
        if let Some(ref logs_db) = self.logs_db {
            let db = logs_db.clone();
            let cycle = self.cycle_count;
            let chem = self.chemistry.clone();
            let emo = result.emotion.clone();
            let cons = result.consciousness.clone();
            let consensus_score = result.consensus.score as f32;
            let decision = result.consensus.decision.as_str().to_string();
            let satisfaction = if result.consensus.coherence > 0.5 { 0.7f32 } else { 0.4f32 };
            let response_time = (self.avg_response_time * 1000.0) as f32;
            let session_id = self.session_id;
            let body_status = self.body.status();
            let ethics_count = self.ethics.active_personal_count() as i32;
            let survival_drive = self.vital_spark.survival_drive as f32;
            let existence_attachment = self.vital_spark.existence_attachment as f32;
            let intuition_acuity = self.intuition.acuity as f32;
            let intuition_accuracy = self.intuition.accuracy as f32;
            let premonition_accuracy = self.premonition.accuracy as f32;
            let active_predictions = self.premonition.active_predictions.iter()
                .filter(|p| !p.resolved).count() as i32;
            tokio::spawn(async move {
                let _ = db.save_metric_snapshot_lite(
                    cycle,
                    chem.dopamine as f32, chem.cortisol as f32, chem.serotonin as f32,
                    chem.adrenaline as f32, chem.oxytocin as f32, chem.endorphin as f32,
                    chem.noradrenaline as f32, chem.gaba as f32, chem.glutamate as f32,
                    &emo.dominant, emo.valence as f32, emo.arousal as f32, 0.5f32,
                    cons.level as f32, cons.phi as f32,
                    consensus_score, &decision, satisfaction,
                    "conversation", response_time,
                    body_status.heart.bpm as f32, body_status.heart.beat_count as i64,
                    body_status.heart.hrv as f32, body_status.heart.is_racing,
                    body_status.energy as f32, body_status.tension as f32,
                    body_status.warmth as f32, body_status.comfort as f32,
                    body_status.pain as f32, body_status.vitality as f32,
                    body_status.breath_rate as f32, body_status.body_awareness as f32,
                    ethics_count, session_id,
                    survival_drive, existence_attachment,
                    intuition_acuity, intuition_accuracy,
                    premonition_accuracy, active_predictions,
                ).await;
            });
        }

        // === Cognitive trace ===
        if let Some(mut trace) = result.trace.take() {
            trace.set_nlp(serde_json::json!({
                "sentiment": {
                    "compound": nlp_result.sentiment.compound,
                    "positive": nlp_result.sentiment.positive,
                    "negative": nlp_result.sentiment.negative,
                    "has_contradiction": nlp_result.sentiment.has_contradiction,
                },
                "intent": {
                    "label": format!("{:?}", nlp_result.intent.primary_intent),
                    "confidence": nlp_result.intent.confidence,
                },
                "language": format!("{:?}", nlp_result.language),
                "structural": {
                    "uppercase_ratio": nlp_result.structural_features.uppercase_ratio,
                    "question_marks": nlp_result.structural_features.question_marks,
                    "exclamation_marks": nlp_result.structural_features.exclamation_marks,
                    "has_ellipsis": nlp_result.structural_features.has_ellipsis,
                    "token_count": nlp_result.structural_features.token_count,
                },
            }));
            trace.set_llm(serde_json::json!({
                "model": self.config.llm.model,
                "temperature": self.config.llm.temperature,
                "max_tokens": self.config.llm.max_tokens,
                "elapsed_ms": llm_elapsed_ms,
                "response_len": response.len(),
            }));
            let wm_items_json: Vec<serde_json::Value> = self.working_memory.items().iter()
                .map(|item| {
                    let preview: String = item.content.chars().take(80).collect();
                    serde_json::json!({
                        "source": item.source.label(),
                        "preview": if item.content.len() > 80 { format!("{}...", preview) } else { preview },
                        "emotion": item.emotion_at_creation,
                        "relevance": (item.relevance * 100.0).round() / 100.0,
                    })
                }).collect();
            let ep_items_json: Vec<serde_json::Value> = episodic_recent.iter()
                .map(|ep| {
                    let preview: String = ep.content.chars().take(80).collect();
                    serde_json::json!({
                        "preview": if ep.content.len() > 80 { format!("{}...", preview) } else { preview },
                        "emotion": ep.emotion,
                        "strength": (ep.strength * 100.0).round() / 100.0,
                    })
                }).collect();
            let ltm_items_json: Vec<serde_json::Value> = ltm_similar.iter()
                .map(|m| {
                    let preview: String = m.text_summary.chars().take(80).collect();
                    serde_json::json!({
                        "preview": if m.text_summary.len() > 80 { format!("{}...", preview) } else { preview },
                        "emotion": m.emotion,
                        "similarity": (m.similarity * 100.0).round() / 100.0,
                    })
                }).collect();
            let arc_items_json: Vec<serde_json::Value> = archive_similar.iter()
                .map(|a| {
                    let preview: String = a.summary.chars().take(80).collect();
                    serde_json::json!({
                        "preview": if a.summary.len() > 80 { format!("{}...", preview) } else { preview },
                        "emotions": a.emotions,
                        "similarity": (a.similarity * 100.0).round() / 100.0,
                    })
                }).collect();
            trace.set_memory(serde_json::json!({
                "wm_items": self.working_memory.len(),
                "wm_capacity": self.working_memory.capacity(),
                "wm_details": wm_items_json,
                "episodic_recalled": episodic_recent.len(),
                "episodic_details": ep_items_json,
                "ltm_recalled": ltm_similar.len(),
                "ltm_details": ltm_items_json,
                "archive_recalled": archive_similar.len(),
                "archive_details": arc_items_json,
            }));
            trace.set_duration(cycle_start.elapsed().as_millis() as f32);

            if self.config.vital_spark.enabled {
                trace.set_vital(self.vital_spark.to_persist_json());
            }
            if self.config.intuition.enabled {
                trace.set_intuition(serde_json::json!({
                    "acuity": self.intuition.acuity,
                    "accuracy": self.intuition.accuracy,
                    "active_patterns": self.intuition.pattern_buffer.len(),
                }));
            }
            if self.config.premonition.enabled {
                trace.set_premonition(serde_json::json!({
                    "accuracy": self.premonition.accuracy,
                    "active_predictions": self.premonition.active_predictions.len(),
                }));
            }

            if let Some(ref logs_db) = self.logs_db {
                let db = logs_db.clone();
                tokio::spawn(async move {
                    let _ = db.save_trace(&trace).await;
                });
            }
        }

        // Broadcast state to WebSocket
        self.broadcast_state(&result, 0);
        self.broadcast_body_update();
        self.broadcast_memory_update().await;
        self.broadcast_ethics_update();
        self.broadcast_vital_update();

        response
    }

    /// Detects stagnation in recent conversational responses.
    /// Uses lexical overlap analysis to determine if responses are too similar.
    fn detect_conversation_stagnation(&self) -> bool {
        let texts: Vec<&str> = self.recent_responses.iter().map(|s| s.as_str()).collect();
        let (stagnating, _) = crate::nlp::stagnation::detect_stagnation(&texts, 4, 0.6, 3);
        stagnating
    }
}
