// =============================================================================
// lifecycle/thinking_processing.rs — Post-LLM processing phases
// =============================================================================
//
// This file contains the processing phases executed after the LLM call.
// It includes:
//   - LLM history logging
//   - Brain pipeline (process_stimulus)
//   - Working memory + memory echo
//   - UCB1 reward + ethics tracking + moral formulation
//   - Human RLHF feedback
//   - LoRA data collection
//   - Web knowledge bonus
//   - Thought logging to PostgreSQL
//   - Complete cognitive trace construction
//   - WebSocket broadcast + metric snapshots
// =============================================================================

use crate::memory::WorkingItemSource;
use crate::neurochemistry::Molecule;
use crate::stimulus::Stimulus;
use crate::logging::{LogLevel, LogCategory};

use super::SaphireAgent;
use super::truncate_utf8;
use super::thinking::{ThinkingContext, FeedbackRequest, strip_chemical_trace};

impl SaphireAgent {
    // =========================================================================
    // Phase 20: Log LLM call history to the logs database
    // =========================================================================

    /// Asynchronously logs the LLM call details (model, response, timing)
    /// to the logs database for later analysis.
    pub(super) fn phase_llm_history(&mut self, ctx: &mut ThinkingContext) {
        if let Some(ref logs_db) = self.logs_db {
            let db = logs_db.clone();
            let cycle = self.cycle_count;
            let model = self.config.llm.model.clone();
            let resp_clone = ctx.thought_text.clone();
            let elapsed_ms = (ctx.llm_elapsed * 1000.0) as f32;
            let session_id = self.session_id;
            let temp_f32 = self.config.llm.temperature as f32;
            let max_tok = self.config.llm.max_tokens_thought as i32;
            let tt = ctx.thought_type.as_str().to_string();
            tokio::spawn(async move {
                let _ = db.save_llm_history(
                    cycle, &tt, &model, "(thought_prompt)", &tt,
                    &resp_clone, temp_f32, max_tok, elapsed_ms,
                    true, "", session_id,
                ).await;
            });
        }
    }

    // =========================================================================
    // Phase 22: Brain pipeline (process_stimulus)
    // =========================================================================

    /// Runs the brain pipeline on the generated thought text.
    /// Creates an autonomous stimulus, analyzes it, and processes it through
    /// the three brain modules (reptilian, limbic, neocortex) + consensus.
    /// Also applies a micro-drift of oxytocin based on coherence.
    pub(super) fn phase_pipeline(&mut self, ctx: &mut ThinkingContext) {
        let mut stimulus = Stimulus::autonomous(&ctx.thought_text);
        stimulus.analyze_content();
        let result = self.process_stimulus(&stimulus);

        // Autonomous micro-drift of oxytocin based on coherence
        let oxy_drift = (result.consensus.coherence * 0.03) - 0.01;
        self.chemistry.boost(Molecule::Oxytocin, oxy_drift);

        ctx.process_result = Some(result);
    }

    // =========================================================================
    // Phase 23: Working memory — push thought, handle ejection
    // =========================================================================

    /// Pushes a preview of the generated thought into working memory.
    /// If an older item is ejected (capacity exceeded), it is stored as
    /// an episodic memory in the database.
    pub(super) async fn phase_working_memory(&mut self, ctx: &mut ThinkingContext) {
        let result = ctx.process_result.as_ref().unwrap();
        let thought_preview: String = ctx.thought_text.chars().take(200).collect();
        let chem_sig = crate::neurochemistry::ChemicalSignature::from(&self.chemistry);
        let wm_ejected = self.working_memory.push(
            thought_preview,
            WorkingItemSource::OwnThought(ctx.thought_text.clone()),
            result.emotion.dominant.clone(),
            chem_sig,
        );
        ctx.had_wm_ejection = wm_ejected.is_some();
        if let (Some(ejected), Some(ref db)) = (wm_ejected, &self.db) {
            let satisfaction = if result.consensus.coherence > 0.5 { 0.7 } else { 0.4 };
            let _ = db.store_episodic(
                &ejected.content, ejected.source.label(),
                &serde_json::json!({}), 0, &serde_json::json!({}),
                &ejected.emotion_at_creation, satisfaction, result.emotion.arousal as f32,
                self.conversation_id.as_deref(),
                Some(&ejected.chemical_signature),
            ).await;
        }
    }

    // =========================================================================
    // Phase 23b: Post-thought memory echo
    // =========================================================================

    /// Searches for similar memories that resonate with the generated thought.
    /// If echoes are found, their access count is boosted (reinforcement).
    /// This implements state-dependent memory recall: thinking about a topic
    /// strengthens related memories.
    pub(super) async fn phase_memory_echo(&mut self, ctx: &mut ThinkingContext) {
        if ctx.thought_text.is_empty() {
            return;
        }
        if let Some(ref db) = self.db {
            if let Ok(echoes) = db.search_similar_memories_by_text(&ctx.thought_text, 2, 0.4).await {
                for echo in &echoes {
                    let _ = db.boost_memory_access(echo.id).await;
                }
                if !echoes.is_empty() {
                    self.log(LogLevel::Debug, LogCategory::Memory,
                        format!("Echo memoriel: {} souvenir(s) resonne(nt) avec la pensee (sim: {:.2})",
                            echoes.len(), echoes[0].similarity),
                        serde_json::json!({
                            "echo_count": echoes.len(),
                            "top_similarity": echoes[0].similarity,
                            "top_summary": &echoes[0].text_summary,
                        }));
                }
            }
        }
    }

    // =========================================================================
    // Phase 24: UCB1 reward + ethics tracking + moral formulation
    // =========================================================================

    /// Computes the UCB1 reward for this cycle based on coherence, emotional
    /// diversity, and neurochemical "umami". Also detects thought stagnation
    /// (lexical and semantic) and triggers moral formulation if conditions are met.
    pub(super) async fn phase_reward_and_ethics(&mut self, ctx: &mut ThinkingContext) {
        let result = ctx.process_result.as_ref().unwrap();

        let coherence = result.consensus.coherence;
        let emotion_diversity = ctx.emotion.spectrum.iter()
            .filter(|(_, s)| *s > 0.3)
            .count() as f64 / 22.0;

        // Composite reward: coherence + emotional diversity + umami
        let quality = (coherence * 0.5 + emotion_diversity * 0.3 + 0.2).clamp(0.0, 1.0);
        ctx.quality = quality;
        let umami = self.chemistry.compute_umami();
        ctx.reward = quality * 0.50 + coherence * 0.20 + umami * 0.15 + 0.15;

        if quality < 0.4 {
            self.thought_engine.bandit_decay(&ctx.thought_type, 0.95);
        }

        self.thought_engine.update_reward(&ctx.thought_type, ctx.reward);
        self.thought_engine.add_recent(ctx.thought_text.clone());

        let recents = self.thought_engine.recent_thoughts();
        let texts: Vec<&str> = recents.iter().map(|s| s.as_str()).collect();
        let (is_stagnating, obsessional_words) =
            crate::nlp::stagnation::detect_stagnation(&texts, 4, 0.6, 3);
        let (is_semantic_stag, semantic_sim) =
            crate::nlp::stagnation::detect_semantic_stagnation(&texts, 4, 0.55);
        if is_stagnating || is_semantic_stag {
            if is_semantic_stag && !is_stagnating {
                tracing::warn!(
                    "Stagnation SEMANTIQUE detectee (similarite cosinus={:.2}) — purge pensees recentes",
                    semantic_sim
                );
            } else {
                tracing::warn!(
                    "Stagnation persistante ({} mots obsessionnels: {:?}, sim={:.2}) — purge pensees recentes",
                    obsessional_words.len(), &obsessional_words[..obsessional_words.len().min(5)], semantic_sim
                );
            }
            self.thought_engine.clear_recent();
            self.stagnation_break = true;
        }

        if ctx.thought_type == crate::agent::thought_engine::ThoughtType::MoralReflection
            || ctx.thought_type == crate::agent::thought_engine::ThoughtType::MoralFormulation {
            self.moral_reflection_count += 1;
        }
        self.cycles_since_last_formulation += 1;

        if self.should_attempt_moral_formulation(&result.consciousness) {
            if let Some(_principle) = self.attempt_moral_formulation(
                &ctx.thought_text, &result.emotion.dominant, &result.consciousness
            ).await {
                self.broadcast_ethics_update();
            }
        }
    }

    // =========================================================================
    // Phase 24b: RLHF human feedback — ask for an opinion
    // =========================================================================

    /// Potentially asks the human for feedback on a high-quality thought.
    /// Conditions: RLHF enabled, in conversation, no pending request,
    /// minimum cycles elapsed, and reward exceeds threshold.
    /// Sends the question via WebSocket and stores a `FeedbackRequest`.
    pub(super) fn phase_maybe_ask_feedback(&mut self, ctx: &mut ThinkingContext) {
        self.cycles_since_last_feedback += 1;

        if let Some(ref fb) = self.feedback_pending {
            if self.cycle_count.saturating_sub(fb.asked_at_cycle) >= self.config.human_feedback.timeout_cycles {
                self.feedback_pending = None;
                self.log(LogLevel::Debug, LogCategory::Cycle,
                    "Feedback RLHF timeout — pas de reponse humaine",
                    serde_json::json!({}));
            }
        }

        if !self.config.human_feedback.enabled { return; }
        if !self.in_conversation { return; }
        if self.feedback_pending.is_some() { return; }
        if self.cycles_since_last_feedback < self.config.human_feedback.min_cycles_between { return; }
        if ctx.reward < self.config.human_feedback.min_reward_to_ask { return; }

        let clean_text = strip_chemical_trace(&ctx.thought_text);
        let summary: String = clean_text.chars().take(120).collect();
        let question = format!(
            "Je viens de penser que {}... Qu'en penses-tu ?",
            summary.trim_end_matches('.')
        );

        if let Some(ref tx) = self.ws_tx {
            let _ = tx.send(serde_json::json!({
                "type": "chat_response",
                "content": question,
            }).to_string());
        }

        self.feedback_pending = Some(FeedbackRequest {
            thought_text: ctx.thought_text.clone(),
            thought_type: ctx.thought_type.clone(),
            auto_reward: ctx.reward,
            asked_at_cycle: self.cycle_count,
        });
        self.cycles_since_last_feedback = 0;

        self.log(LogLevel::Info, LogCategory::Cycle,
            "Feedback RLHF demande a l'humain",
            serde_json::json!({
                "reward": ctx.reward,
                "thought_type": format!("{:?}", ctx.thought_type),
            }));
    }

    // =========================================================================
    // Phase 24c: LoRA training data collection
    // =========================================================================

    /// Collects high-quality thought cycles as LoRA fine-tuning samples.
    /// Only stores samples whose quality exceeds the configured threshold.
    /// Also prunes old samples if the maximum count is exceeded.
    pub(super) async fn phase_lora_collect(&mut self, ctx: &mut ThinkingContext) {
        if !self.config.lora.enabled { return; }

        let quality = ctx.quality;
        if quality < self.config.lora.min_quality_threshold { return; }

        if let Some(ref db) = self.db {
            let result = ctx.process_result.as_ref().unwrap();
            let _ = db.store_lora_sample(
                &ctx.system_prompt,
                &ctx.prompt,
                &ctx.thought_text,
                &format!("{:?}", ctx.thought_type),
                quality as f32,
                ctx.reward as f32,
                None,
                Some(&result.emotion.dominant),
                Some(result.consciousness.level as f32),
                self.cycle_count as i64,
            ).await;

            let _ = db.prune_lora_samples(self.config.lora.max_samples).await;

            self.log(LogLevel::Debug, LogCategory::Cycle,
                format!("LoRA sample collecte (quality={:.2})", quality),
                serde_json::json!({"quality": quality, "reward": ctx.reward}));
        }
    }

    // =========================================================================
    // Phase 25: Web knowledge bonus
    // =========================================================================

    /// If a web search was performed this cycle, applies a neurochemical
    /// reward (dopamine + noradrenaline + serotonin boost), logs the
    /// knowledge acquired, resets the search counter, and broadcasts the
    /// acquisition to the WebSocket.
    pub(super) async fn phase_knowledge_bonus(&mut self, ctx: &mut ThinkingContext) {
        let knowledge_context = ctx.knowledge_context.take();
        if let Some((_, kr)) = knowledge_context {
            let result = ctx.process_result.as_ref().unwrap();

            self.chemistry.boost(Molecule::Dopamine, 0.10);
            self.chemistry.boost(Molecule::Noradrenaline, 0.08);
            self.chemistry.boost(Molecule::Serotonin, 0.05);

            if let Some(ref db) = self.db {
                let _ = db.log_knowledge(
                    &kr.source,
                    &kr.title,
                    &kr.title,
                    &kr.url,
                    &kr.extract,
                    Some(truncate_utf8(&ctx.thought_text, 500)),
                    Some(&result.emotion.dominant),
                    Some(result.consciousness.level as f32),
                ).await;
            }

            self.thought_engine.cycles_since_last_search = 0;

            if let Some(ref tx) = self.ws_tx {
                let knowledge_msg = serde_json::json!({
                    "type": "knowledge_acquired",
                    "source": kr.source,
                    "title": kr.title,
                    "url": kr.url,
                    "extract_preview": kr.extract.chars().take(200).collect::<String>(),
                    "emotion": result.emotion.dominant,
                    "total_explored": 0,
                });
                let _ = tx.send(knowledge_msg.to_string());
            }
        }
    }

    // =========================================================================
    // Phase 26: Log thought to PostgreSQL
    // =========================================================================

    /// Persists the thought text, emotion, consciousness level, and chemistry
    /// snapshot to the thought log and episodic memory tables in PostgreSQL.
    pub(super) async fn phase_thought_log(&mut self, ctx: &mut ThinkingContext) {
        let result = ctx.process_result.as_ref().unwrap();
        if let Some(ref db) = self.db {
            let chemistry_json = serde_json::json!({
                "dopamine": self.chemistry.dopamine,
                "cortisol": self.chemistry.cortisol,
                "serotonin": self.chemistry.serotonin,
            });
            let _ = db.log_thought(
                ctx.thought_type.as_str(),
                &ctx.thought_text,
                &result.emotion.dominant,
                result.consciousness.level as f32,
                result.consciousness.phi as f32,
                self.mood.valence as f32,
                &chemistry_json,
            ).await;

            let chem_sig = crate::neurochemistry::ChemicalSignature::from(&self.chemistry);
            let _ = db.store_episodic(
                truncate_utf8(&ctx.thought_text, 500),
                ctx.thought_type.as_str(),
                &serde_json::json!({}),
                result.consensus.decision.as_i8() as i16,
                &chemistry_json,
                &result.emotion.dominant,
                if result.consensus.coherence > 0.5 { 0.7 } else { 0.4 },
                result.emotion.arousal as f32,
                self.conversation_id.as_deref(),
                Some(&chem_sig),
            ).await;
        }
    }

    // =========================================================================
    // Phase 28: Complete cognitive trace
    // =========================================================================

    /// Builds a complete cognitive trace for this cycle, including NLP analysis,
    /// LLM metadata, memory details, vital/intuition/premonition states, and
    /// timing. The trace is asynchronously saved to the logs database.
    pub(super) fn phase_cognitive_trace(&mut self, ctx: &mut ThinkingContext) {
        let result = ctx.process_result.as_mut().unwrap();
        if let Some(mut trace) = result.trace.take() {
            let nlp_on_thought = self.nlp.analyze(&ctx.thought_text);
            trace.set_nlp(serde_json::json!({
                "sentiment": {
                    "compound": nlp_on_thought.sentiment.compound,
                    "positive": nlp_on_thought.sentiment.positive,
                    "negative": nlp_on_thought.sentiment.negative,
                    "has_contradiction": nlp_on_thought.sentiment.has_contradiction,
                },
                "intent": {
                    "label": format!("{:?}", nlp_on_thought.intent.primary_intent),
                    "confidence": nlp_on_thought.intent.confidence,
                },
                "language": format!("{:?}", nlp_on_thought.language),
                "structural": {
                    "token_count": nlp_on_thought.structural_features.token_count,
                    "has_ellipsis": nlp_on_thought.structural_features.has_ellipsis,
                    "question_marks": nlp_on_thought.structural_features.question_marks,
                    "exclamation_marks": nlp_on_thought.structural_features.exclamation_marks,
                },
                "source": "autonomous_thought",
            }));
            trace.set_llm(serde_json::json!({
                "model": self.config.llm.model,
                "temperature": self.config.llm.temperature,
                "max_tokens": self.config.llm.max_tokens_thought,
                "elapsed_ms": (ctx.llm_elapsed * 1000.0) as f32,
                "response_len": ctx.thought_text.len(),
                "thought_type": ctx.thought_type.as_str(),
            }));
            let mut mem_data = ctx.memory_trace_data.clone();
            if let Some(obj) = mem_data.as_object_mut() {
                obj.insert("wm_ejected".into(), serde_json::json!(ctx.had_wm_ejection));
                obj.insert("episodic_stored".into(), serde_json::json!(true));
            }
            trace.set_memory(mem_data);
            trace.set_duration(ctx.cycle_start.elapsed().as_millis() as f32);

            if self.config.vital_spark.enabled {
                trace.set_vital(self.vital_spark.to_persist_json());
            }
            if self.config.intuition.enabled {
                trace.set_intuition(serde_json::json!({
                    "acuity": self.intuition.acuity,
                    "accuracy": self.intuition.accuracy,
                    "active_patterns": ctx.intuition_patterns.len(),
                    "patterns": ctx.intuition_patterns.iter().map(|p| {
                        serde_json::json!({
                            "type": format!("{:?}", p.pattern_type),
                            "confidence": p.confidence,
                            "source": format!("{:?}", p.source),
                            "description": p.description,
                        })
                    }).collect::<Vec<_>>(),
                }));
            }
            if self.config.premonition.enabled {
                trace.set_premonition(serde_json::json!({
                    "accuracy": self.premonition.accuracy,
                    "active_predictions": self.premonition.active_predictions.len(),
                    "new_predictions": ctx.new_premonitions.iter().map(|p| {
                        serde_json::json!({
                            "id": p.id,
                            "prediction": p.prediction,
                            "category": format!("{:?}", p.category),
                            "confidence": p.confidence,
                        })
                    }).collect::<Vec<_>>(),
                }));
            }

            if let Some(ref logs_db) = self.logs_db {
                let db = logs_db.clone();
                tokio::spawn(async move {
                    let _ = db.save_trace(&trace).await;
                });
            }
        }
    }

    // =========================================================================
    // Phase 29: Broadcast to WebSocket
    // =========================================================================

    /// Broadcasts the agent's full internal state to connected WebSocket
    /// clients, including state update, body, memory, ethics, vital data,
    /// and an inner monologue preview (first 500 chars of the thought).
    pub(super) async fn phase_broadcast(&mut self, ctx: &mut ThinkingContext) {
        let result = ctx.process_result.as_ref().unwrap();
        self.last_thought_type = ctx.thought_type.as_str().to_string();

        self.broadcast_state(result, 0);
        self.broadcast_body_update();
        self.broadcast_memory_update().await;
        self.broadcast_ethics_update();
        self.broadcast_vital_update();
        // thought_ownership not ported in the lite version
        if let Some(ref tx) = self.ws_tx {
            let _ = tx.send(serde_json::json!({
                "type": "inner_monologue",
                "text": ctx.thought_text.chars().take(500).collect::<String>(),
            }).to_string());
        }
    }

    // =========================================================================
    // Phase 30: Metric snapshot
    // =========================================================================

    /// Asynchronously saves a comprehensive metric snapshot to the logs
    /// database, capturing chemistry, emotion, consciousness, consensus,
    /// body status, ethics count, vital spark, intuition, and premonition.
    pub(super) fn phase_metrics(&mut self, ctx: &mut ThinkingContext) {
        let result = ctx.process_result.as_ref().unwrap();
        if let Some(ref logs_db) = self.logs_db {
            let db = logs_db.clone();
            let cycle = self.cycle_count;
            let chem = self.chemistry.clone();
            let emo = result.emotion.clone();
            let cons = result.consciousness.clone();
            let consensus_score = result.consensus.score as f32;
            let decision = result.consensus.decision.as_str().to_string();
            let satisfaction = ctx.reward as f32;
            let response_time = (ctx.llm_elapsed * 1000.0) as f32;
            let session_id = self.session_id;
            let tt = ctx.thought_type.as_str().to_string();
            let body_status = self.body.status();
            let ethics_count = self.ethics.active_personal_count() as i32;
            let survival_drive = self.vital_spark.survival_drive as f32;
            let existence_attachment = self.vital_spark.existence_attachment as f32;
            let intuition_acuity = self.intuition.acuity as f32;
            let intuition_accuracy = self.intuition.accuracy as f32;
            let premonition_accuracy = self.premonition.accuracy as f32;
            let active_predictions = self.premonition.active_predictions.iter().filter(|p| !p.resolved).count() as i32;

            tokio::spawn(async move {
                let _ = db.save_metric_snapshot_lite(
                    cycle,
                    chem.dopamine as f32, chem.cortisol as f32, chem.serotonin as f32,
                    chem.adrenaline as f32, chem.oxytocin as f32, chem.endorphin as f32,
                    chem.noradrenaline as f32, chem.gaba as f32, chem.glutamate as f32,
                    &emo.dominant, emo.valence as f32, emo.arousal as f32, 0.5f32,
                    cons.level as f32, cons.phi as f32,
                    consensus_score, &decision, satisfaction,
                    &tt, response_time,
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
    }
}
