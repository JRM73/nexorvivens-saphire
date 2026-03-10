// =============================================================================
// lifecycle/thinking_processing.rs — Traitement post-LLM
// =============================================================================
//
// Ce fichier contient les phases de traitement apres l'appel au LLM.
// Cela inclut :
//   - Log LLM history
//   - Demande d'algorithme
//   - Pipeline cerebral (process_stimulus)
//   - Memoire de travail + echo memoriel
//   - Recompense UCB1 + ethique + formulation morale
//   - Feedback humain RLHF
//   - Collecte LoRA
//   - Bonus connaissance web
//   - Log pensee + profilage OCEAN
//   - Trace cognitive complete
//   - Broadcast + metriques
// =============================================================================

use crate::memory::WorkingItemSource;
use crate::neurochemistry::Molecule;
use crate::plugins::BrainEvent;
use crate::stimulus::Stimulus;
use crate::profiling::BehaviorObservation;
use crate::logging::{LogLevel, LogCategory};

use super::SaphireAgent;
use super::truncate_utf8;
use super::thinking::{ThinkingContext, FeedbackRequest, strip_chemical_trace};

impl SaphireAgent {
    // =========================================================================
    // Phase 20 : Log LLM history
    // =========================================================================

    /// Enregistre l'historique LLM pour la pensee autonome.
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
    // Phase 21 : Demande d'algorithme par le LLM
    // =========================================================================

    /// Detecte et traite les demandes d'algorithme dans la reponse du LLM.
    pub(super) async fn phase_algorithm_request(&mut self, ctx: &mut ThinkingContext) {
        if self.orchestrator.enabled && self.orchestrator.llm_access_enabled {
            if let Some(request) = self.orchestrator.parse_llm_request(&ctx.thought_text) {
                self.handle_algorithm_request(&request).await;
            }
        }

        // Retirer le prefixe UTILISER_ALGO du texte de pensee
        if let Some(pos) = ctx.thought_text.find("UTILISER_ALGO:") {
            ctx.thought_text = ctx.thought_text[..pos].trim().to_string();
        }
    }

    // =========================================================================
    // Phase 22 : Pipeline cerebral (process_stimulus)
    // =========================================================================

    /// Traite la pensee generee comme un stimulus interne via le pipeline
    /// cerebral complet (NLP, consensus, chimie, emotion, conscience, regulation).
    pub(super) fn phase_pipeline(&mut self, ctx: &mut ThinkingContext) {
        let mut stimulus = Stimulus::autonomous(&ctx.thought_text);
        stimulus.analyze_content();
        let result = self.process_stimulus(&stimulus);

        // Micro-drift autonome d'oxytocine (rendements decroissants)
        let oxy_drift = (result.consensus.coherence * 0.03) - 0.01;
        self.chemistry.boost(Molecule::Oxytocin, oxy_drift);

        ctx.process_result = Some(result);
    }

    // =========================================================================
    // Phase 23 : Memoire de travail
    // =========================================================================

    /// Pousse la pensee dans la memoire de travail et gere les ejections.
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
    // Phase 23b : Echo memoriel post-pensee
    // =========================================================================

    /// Apres la generation LLM, cherche si la pensee produite resonne avec
    /// des souvenirs LTM existants. Si oui, booste leur acces (renforcement
    /// hebbien : "neurons that fire together wire together").
    pub(super) async fn phase_memory_echo(&mut self, ctx: &mut ThinkingContext) {
        if ctx.thought_text.is_empty() {
            return;
        }
        if let Some(ref db) = self.db {
            let embedding_f64 = self.encoder.encode(&ctx.thought_text);
            let embedding_f32: Vec<f32> = embedding_f64.iter().map(|&v| v as f32).collect();

            if let Ok(echoes) = db.search_similar_memories(&embedding_f32, 2, 0.4).await {
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
    // Phase 24 : Recompense UCB1 + tracking ethique + formulation morale
    // =========================================================================

    /// Calcule la recompense pour le bandit UCB1, track les reflexions morales
    /// et tente une formulation morale si conditions reunies.
    pub(super) async fn phase_reward_and_ethics(&mut self, ctx: &mut ThinkingContext) {
        let result = ctx.process_result.as_ref().unwrap();

        let coherence = result.consensus.coherence;
        let emotion_diversity = ctx.emotion.spectrum.iter()
            .filter(|(_, s)| *s > 0.3)
            .count() as f64 / 22.0;
        let quality = self.metacognition.evaluate_thought_quality(
            &ctx.thought_text, coherence, emotion_diversity,
        );
        ctx.quality = quality.clamp(0.0, 1.0);
        // Recompense composite : qualite + coherence + signal umami (neurochimique)
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
            crate::nlp::stagnation::detect_semantic_stagnation(&texts, 4, 0.45);
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
            // Purger aussi le monologue interieur — sinon il re-alimente la boucle
            if self.config.inner_monologue.enabled {
                self.inner_monologue.clear();
                tracing::info!("Monologue interieur purge (anti-stagnation)");
            }
            // Poser le flag anti-stagnation : au cycle suivant, le prompt
            // injectera une directive forte de changement de sujet
            self.stagnation_break = true;
            self.stagnation_banned_words = obsessional_words.clone();
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
    // Phase 24b : Feedback humain RLHF — demander un avis
    // =========================================================================

    /// Apres une pensee de qualite, pose une question contextuelle dans le chat.
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
        let summary: String = clean_text.chars().take(300).collect();
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
    // Phase 24c : Collecte LoRA — sauvegarder les bonnes pensees
    // =========================================================================

    /// Collecte les pensees de haute qualite dans la table lora_training_data.
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
    // Phase 25 : Bonus connaissance web
    // =========================================================================

    /// Applique le bonus chimique d'apprentissage et log les connaissances acquises.
    pub(super) async fn phase_knowledge_bonus(&mut self, ctx: &mut ThinkingContext) {
        let knowledge_context = ctx.knowledge_context.take();
        if let Some((_, kr)) = knowledge_context {
            let result = ctx.process_result.as_ref().unwrap();

            self.chemistry.boost(Molecule::Dopamine, 0.10);
            self.chemistry.boost(Molecule::Noradrenaline, 0.08);
            self.chemistry.boost(Molecule::Serotonin, 0.05);

            if let Some(ref db) = self.db {
                // Filtre anti-hallucination : verifier que la reflexion LLM
                // n'est pas une reformulation repetitive des reflexions precedentes.
                // Si la similarite cosinus depasse 0.45, on rejette la reflexion
                // pour eviter la consolidation de faux souvenirs en boucle.
                let reflection_text = truncate_utf8(&ctx.thought_text, 500);
                let recent_refs: Vec<&str> = self.thought_engine.recent_thoughts()
                    .iter().map(|s| s.as_str()).collect();
                let should_store = if recent_refs.len() >= 2 {
                    let mut check_texts = recent_refs.clone();
                    check_texts.push(reflection_text);
                    let (is_stag, _) = crate::nlp::stagnation::detect_semantic_stagnation(
                        &check_texts, 4, 0.45,
                    );
                    if is_stag {
                        tracing::warn!(
                            "Reflexion knowledge rejetee (anti-hallucination) : trop similaire aux pensees recentes"
                        );
                    }
                    !is_stag
                } else { true };

                let stored_reflection = if should_store { Some(reflection_text) } else { None };
                let _ = db.log_knowledge(
                    &kr.source,
                    &kr.title,
                    &kr.title,
                    &kr.url,
                    &kr.extract,
                    stored_reflection,
                    Some(&result.emotion.dominant),
                    Some(result.consciousness.level as f32),
                ).await;
            }

            self.knowledge.topics_explored.push(kr.title.clone());
            self.knowledge.cycles_since_last_search = 0;
            self.thought_engine.cycles_since_last_search = 0;

            if let Some(ref tx) = self.ws_tx {
                let knowledge_msg = serde_json::json!({
                    "type": "knowledge_acquired",
                    "source": kr.source,
                    "title": kr.title,
                    "url": kr.url,
                    "extract_preview": kr.extract.chars().take(200).collect::<String>(),
                    "emotion": result.emotion.dominant,
                    "total_explored": self.knowledge.topics_explored.len(),
                });
                let _ = tx.send(knowledge_msg.to_string());
            }
        }
    }

    // =========================================================================
    // Phase 26 : Log pensee dans PostgreSQL
    // =========================================================================

    /// Enregistre la pensee dans thought_log et memoire episodique.
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
    // Phase 27 : Profilage OCEAN
    // =========================================================================

    /// Observe le comportement pour le profil OCEAN et recalcule si necessaire.
    pub(super) async fn phase_profiling(&mut self, ctx: &mut ThinkingContext) {
        let result = ctx.process_result.as_ref().unwrap();
        if !(self.config.profiling.enabled && self.config.profiling.self_profiling) {
            return;
        }

        let obs = BehaviorObservation {
            thought_type: ctx.thought_type.as_str().to_string(),
            decision: result.consensus.decision.clone(),
            emotion: result.emotion.dominant.clone(),
            emotion_intensity: result.emotion.arousal,
            mood_valence: self.mood.valence,
            chemistry: [
                self.chemistry.dopamine, self.chemistry.cortisol,
                self.chemistry.serotonin, self.chemistry.adrenaline,
                self.chemistry.oxytocin, self.chemistry.endorphin,
                self.chemistry.noradrenaline,
            ],
            module_weights: result.consensus.weights,
            consensus_score: result.consensus.score,
            consciousness_level: result.consciousness.level,
            was_conversation: self.in_conversation,
            was_web_search: ctx.was_web_search,
            response_length: ctx.thought_text.len(),
            used_first_person: ctx.thought_text.contains("je ") || ctx.thought_text.contains("j'"),
            asked_question: ctx.thought_text.contains('?'),
            expressed_uncertainty: ctx.thought_text.contains("peut-etre") || ctx.thought_text.contains("incertain"),
            referenced_past: ctx.thought_text.contains("souviens") || ctx.thought_text.contains("rappelle"),
            cycle: self.cycle_count,
            timestamp: chrono::Utc::now(),
        };
        self.self_profiler.observe(obs);

        if self.self_profiler.should_recompute(self.cycle_count) {
            self.self_profiler.force_recompute(self.cycle_count);
            tracing::info!("Profil OCEAN recalcule (confiance: {:.0}%)",
                self.self_profiler.profile().confidence * 100.0);

            {
                let profile = self.self_profiler.profile();
                let inputs = crate::temperament::TemperamentInputs {
                    openness_facets: profile.openness.facets,
                    openness_score: profile.openness.score,
                    conscientiousness_facets: profile.conscientiousness.facets,
                    conscientiousness_score: profile.conscientiousness.score,
                    extraversion_facets: profile.extraversion.facets,
                    extraversion_score: profile.extraversion.score,
                    agreeableness_facets: profile.agreeableness.facets,
                    agreeableness_score: profile.agreeableness.score,
                    neuroticism_facets: profile.neuroticism.facets,
                    neuroticism_score: profile.neuroticism.score,
                    ocean_data_points: profile.data_points,
                    dopamine: self.chemistry.dopamine,
                    cortisol: self.chemistry.cortisol,
                    serotonin: self.chemistry.serotonin,
                    adrenaline: self.chemistry.adrenaline,
                    oxytocin: self.chemistry.oxytocin,
                    endorphin: self.chemistry.endorphin,
                    noradrenaline: self.chemistry.noradrenaline,
                    willpower: self.psychology.will.willpower,
                    superego_strength: self.psychology.freudian.superego.strength,
                    overall_eq: self.psychology.eq.overall_eq,
                    mood_valence: self.mood.valence,
                    mood_arousal: self.mood.arousal,
                    attachment_secure: matches!(
                        self.relationships.attachment_style,
                        crate::relationships::AttachmentStyle::Secure
                    ),
                };
                let new_temp = crate::temperament::Temperament::compute(&inputs);
                if self.temperament.traits.is_empty() {
                    self.temperament = new_temp;
                } else {
                    self.temperament.blend(&new_temp);
                }
                tracing::info!("Temperament recalcule ({} traits)", self.temperament.traits.len());
            }

            if let Some(ref db) = self.db {
                let profile = self.self_profiler.profile();
                let ocean_json = serde_json::to_value(profile).unwrap_or_default();
                let _ = db.save_ocean_profile(
                    &ocean_json,
                    profile.data_points as i64,
                    profile.confidence as f32,
                    &serde_json::json!([]),
                ).await;
            }
        }
    }

    // =========================================================================
    // Phase 28 : Trace cognitive complete
    // =========================================================================

    /// Complete et sauvegarde la trace cognitive avec NLP, LLM, memoire,
    /// vitales, intuition, premonition, sens et orchestrateurs.
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
            if self.config.senses.enabled {
                trace.set_senses(serde_json::json!({
                    "dominant_sense": self.sensorium.dominant_sense,
                    "perception_richness": self.sensorium.perception_richness,
                    "emergence_potential": self.sensorium.emergence_potential,
                    "reading_intensity": self.sensorium.reading.current_intensity,
                    "listening_intensity": self.sensorium.listening.current_intensity,
                    "contact_intensity": self.sensorium.contact.current_intensity,
                    "taste_intensity": self.sensorium.taste.current_intensity,
                    "ambiance_intensity": self.sensorium.ambiance.current_intensity,
                    "narrative": self.sensorium.narrative,
                    "germinated_senses": self.sensorium.emergent_seeds.germinated_count(),
                }));
            }
            if self.attention_orch.enabled {
                trace.set_attention(serde_json::json!({
                    "focus_on": self.attention_orch.current_focus.as_ref().map(|f| &f.subject),
                    "depth": self.attention_orch.current_focus.as_ref().map(|f| f.depth).unwrap_or(0.0),
                    "fatigue": self.attention_orch.fatigue,
                    "concentration_capacity": self.attention_orch.concentration_capacity,
                }));
            }
            if self.desire_orch.enabled {
                let top = self.desire_orch.suggest_pursuit();
                trace.set_desires(serde_json::json!({
                    "active_count": self.desire_orch.active_desires.len(),
                    "top_desire": top.map(|d| &d.title),
                    "top_priority": top.map(|d| d.priority).unwrap_or(0.0),
                    "top_progress": top.map(|d| d.progress).unwrap_or(0.0),
                    "needs": self.desire_orch.fundamental_needs.iter()
                        .map(|n| serde_json::json!({"name": n.name, "satisfaction": n.satisfaction}))
                        .collect::<Vec<_>>(),
                }));
            }
            if self.learning_orch.enabled {
                trace.set_learning(serde_json::json!({
                    "total_lessons": self.learning_orch.lessons.len(),
                    "confirmed": self.learning_orch.lessons.iter().filter(|l| l.confidence > 0.6).count(),
                }));
            }
            if self.healing_orch.enabled {
                trace.set_healing(serde_json::json!({
                    "active_wounds": self.healing_orch.active_wounds.len(),
                    "resilience": self.healing_orch.resilience,
                    "most_severe": self.healing_orch.active_wounds.first()
                        .map(|w| format!("{:?}", w.wound_type)),
                }));
            }

            if self.psychology.enabled {
                let p = &self.psychology;
                trace.set_psychology(serde_json::json!({
                    "freudian": {
                        "id_drive": p.freudian.id.drive_strength,
                        "id_frustration": p.freudian.id.frustration,
                        "ego_strength": p.freudian.ego.strength,
                        "ego_anxiety": p.freudian.ego.anxiety,
                        "ego_strategy": format!("{:?}", p.freudian.ego.strategy),
                        "superego_guilt": p.freudian.superego.guilt,
                        "superego_pride": p.freudian.superego.pride,
                        "conflict": p.freudian.balance.internal_conflict,
                        "health": p.freudian.balance.psychic_health,
                        "defenses": format!("{:?}", p.freudian.active_defenses),
                    },
                    "maslow": {
                        "ceiling": p.maslow.current_active_level,
                        "levels": p.maslow.levels.iter().map(|l| l.satisfaction).collect::<Vec<_>>(),
                    },
                    "toltec": {
                        "alignment": p.toltec.overall_alignment,
                        "accords": p.toltec.agreements.iter().map(|a| a.alignment).collect::<Vec<_>>(),
                    },
                    "jung": {
                        "archetype": format!("{:?}", p.jung.dominant_archetype),
                        "integration": p.jung.integration,
                        "leaking": p.jung.shadow_traits.iter().any(|t| t.leaking),
                    },
                    "eq": {
                        "score": p.eq.overall_eq,
                        "awareness": p.eq.self_awareness,
                        "regulation": p.eq.self_regulation,
                        "motivation": p.eq.motivation,
                        "empathy": p.eq.empathy,
                        "social": p.eq.social_skills,
                    },
                    "flow": {
                        "in_flow": p.flow.in_flow,
                        "intensity": p.flow.flow_intensity,
                        "duration": p.flow.duration_cycles,
                    },
                }));
            }

            if self.config.will.enabled {
                let w = &self.psychology.will;
                let will_json = if let Some(ref delib) = ctx.deliberation {
                    serde_json::json!({
                        "mode": "deliberation",
                        "trigger": format!("{:?}", delib.trigger.trigger_type),
                        "options_count": delib.options.len(),
                        "options": delib.options.iter().map(|o| serde_json::json!({
                            "description": o.description,
                            "id": o.id_score,
                            "superego": o.superego_score,
                            "weighted": o.weighted_score,
                        })).collect::<Vec<_>>(),
                        "chosen": delib.options[delib.chosen].description,
                        "confidence": delib.confidence,
                        "ego_chose": delib.chemistry_influence.efficiency > 0.0,
                        "reasoning": delib.reasoning,
                        "willpower": w.willpower,
                        "decision_fatigue": w.decision_fatigue,
                    })
                } else {
                    serde_json::json!({
                        "mode": "reactif",
                        "willpower": w.willpower,
                        "decision_fatigue": w.decision_fatigue,
                    })
                };
                trace.set_will(will_json);
            }

            if self.config.subconscious.enabled {
                let sc = &self.subconscious;
                let insight_text = sc.ready_insights.first()
                    .map(|i| i.content.clone());
                let priming_text: Vec<String> = sc.active_priming.iter()
                    .map(|p| format!("{} ({})", p.bias_theme, p.source))
                    .collect();
                trace.set_subconscious(serde_json::json!({
                    "activation": sc.activation,
                    "pending_associations": sc.pending_associations.len(),
                    "repressed_count": sc.repressed_content.len(),
                    "repressed_pressure_max": sc.repressed_content.iter()
                        .map(|r| r.pressure).fold(0.0_f64, f64::max),
                    "incubating_problems": sc.incubating_problems.len(),
                    "incubating_details": sc.incubating_problems.iter()
                        .map(|p| serde_json::json!({
                            "question": p.question,
                            "cycles": p.incubation_cycles,
                        })).collect::<Vec<_>>(),
                    "active_priming": priming_text,
                    "insight_ready": insight_text,
                    "total_insights_surfaced": sc.total_insights_surfaced,
                }));
            }

            if self.config.tom.enabled {
                trace.set_tom(self.tom.to_json());
            }
            if self.config.inner_monologue.enabled {
                trace.set_monologue(self.inner_monologue.to_json());
            }
            if self.config.dissonance.enabled {
                trace.set_dissonance(self.dissonance.to_json());
            }
            if self.config.prospective_memory.enabled {
                trace.set_prospective(self.prospective_mem.to_json());
            }
            if self.config.narrative_identity.enabled {
                trace.set_narrative(self.narrative_identity.to_json());
            }
            if self.config.analogical_reasoning.enabled {
                trace.set_analogical(self.analogical.to_json());
            }
            if self.config.cognitive_load.enabled {
                trace.set_cognitive_load(self.cognitive_load.to_json());
            }
            if self.config.mental_imagery.enabled {
                trace.set_imagery(self.imagery.to_json());
            }
            if self.config.sentiments.enabled {
                trace.set_sentiments(self.sentiments.to_json());
            }

            if self.config.sleep.enabled {
                let d = &self.sleep.drive;
                let should = self.sleep.drive.should_sleep();
                trace.set_sleep(serde_json::json!({
                    "is_sleeping": false,
                    "sleep_pressure": d.sleep_pressure,
                    "sleep_threshold": d.sleep_threshold,
                    "awake_cycles": d.awake_cycles,
                    "cycles_since_last_sleep": d.cycles_since_last_sleep,
                    "forced": d.sleep_forced,
                    "should_sleep": should,
                    "attention_fatigue": self.attention_orch.fatigue(),
                    "decision_fatigue": self.psychology.will.decision_fatigue,
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
    // Phase 29 : Broadcast vers plugins et WebSocket
    // =========================================================================

    /// Diffuse l'evenement aux plugins et met a jour les interfaces.
    pub(super) async fn phase_broadcast(&mut self, ctx: &mut ThinkingContext) {
        let result = ctx.process_result.as_ref().unwrap();
        self.last_thought_type = ctx.thought_type.as_str().to_string();

        let event = BrainEvent::ThoughtEmitted {
            thought_type: ctx.thought_type.as_str().to_string(),
            content: ctx.thought_text.clone(),
        };
        self.plugins.broadcast(&event);
        let learnings_count = if let Some(ref db) = self.db {
            db.count_learnings().await.unwrap_or(0)
        } else { 0 };
        ctx.nn_learnings_count = learnings_count as i32;
        self.broadcast_state(result, learnings_count);
        self.broadcast_body_update();
        self.broadcast_memory_update().await;
        self.broadcast_ocean_update();
        self.broadcast_ethics_update();
        self.broadcast_vital_update();
        self.broadcast_senses_update();
        self.broadcast_psychology_update();
        self.broadcast_will_update();
        self.broadcast_needs_update();
        self.broadcast_hormones_update();
        self.broadcast_biology_update();
        self.broadcast_temperament_update();
        if self.config.thought_ownership.enabled {
            if let Some(ref tx) = self.ws_tx {
                let _ = tx.send(serde_json::json!({
                    "type": "inner_monologue",
                    "text": ctx.thought_text.chars().take(500).collect::<String>(),
                }).to_string());
            }
        }
    }

    // =========================================================================
    // Phase 30 : Metric snapshot
    // =========================================================================

    /// Sauvegarde le snapshot de metriques pour le dashboard.
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
            let senses_richness = self.sensorium.perception_richness as f32;
            let senses_dominant = self.sensorium.dominant_sense.clone();
            let reading_beauty = self.sensorium.reading.beauty as f32;
            let ambiance_scent = format!("{:?}", self.sensorium.ambiance.current_scent);
            let contact_warmth = self.sensorium.contact.connection_warmth as f32;
            let emergent_germinated = self.sensorium.emergent_seeds.germinated_count() as i32;
            let knowledge_sources = serde_json::json!({});
            let att_focus = self.attention_orch.current_focus.as_ref()
                .map(|f| f.subject.clone()).unwrap_or_default();
            let att_depth = self.attention_orch.current_focus.as_ref()
                .map(|f| f.depth as f32).unwrap_or(0.0);
            let att_fatigue = self.attention_orch.fatigue as f32;
            let att_concentration = self.attention_orch.concentration_capacity as f32;
            let desires_active_n = self.desire_orch.active_desires.len() as i32;
            let desires_fulfilled_n = self.desire_orch.fulfilled_desires.len() as i32;
            let desires_top = self.desire_orch.suggest_pursuit()
                .map(|d| d.title.clone()).unwrap_or_default();
            let needs: Vec<f32> = self.desire_orch.fundamental_needs.iter()
                .map(|n| n.satisfaction as f32).collect();
            let (n_comp, n_conn, n_expr, n_grow, n_mean) = (
                *needs.first().unwrap_or(&0.0), *needs.get(1).unwrap_or(&0.0),
                *needs.get(2).unwrap_or(&0.0), *needs.get(3).unwrap_or(&0.0),
                *needs.get(4).unwrap_or(&0.0),
            );
            let lessons_total = self.learning_orch.lessons.len() as i32;
            let lessons_confirmed = self.learning_orch.lessons.iter()
                .filter(|l| l.confidence > 0.6).count() as i32;
            let lessons_contradicted = self.learning_orch.lessons.iter()
                .filter(|l| l.times_contradicted > 0).count() as i32;
            let behavior_changes = self.learning_orch.lessons.iter()
                .filter(|l| l.behavior_change.is_some()).count() as i32;
            let wounds_active_n = self.healing_orch.active_wounds.len() as i32;
            let wounds_healed_n = self.healing_orch.healed_wounds.len() as i32;
            let resilience_val = self.healing_orch.resilience as f32;
            let dreams_total_n = self.dream_orch.dream_journal.len() as i32;
            let dreams_insights = self.dream_orch.dream_journal.iter()
                .filter(|d| d.dream.insight.is_some()).count() as i32;
            let last_dream_type = self.dream_orch.dream_journal.last()
                .map(|d| d.dream.dream_type.as_str().to_string()).unwrap_or_default();
            let psy_id_drive = self.psychology.freudian.id.drive_strength as f32;
            let psy_id_frust = self.psychology.freudian.id.frustration as f32;
            let psy_ego_str = self.psychology.freudian.ego.strength as f32;
            let psy_ego_anx = self.psychology.freudian.ego.anxiety as f32;
            let psy_sg_guilt = self.psychology.freudian.superego.guilt as f32;
            let psy_sg_pride = self.psychology.freudian.superego.pride as f32;
            let psy_conflict = self.psychology.freudian.balance.internal_conflict as f32;
            let psy_health = self.psychology.freudian.balance.psychic_health as f32;
            let maslow_ceil = self.psychology.maslow.current_active_level as i32;
            let maslow_l1 = self.psychology.maslow.levels[0].satisfaction as f32;
            let maslow_l2 = self.psychology.maslow.levels[1].satisfaction as f32;
            let maslow_l3 = self.psychology.maslow.levels[2].satisfaction as f32;
            let maslow_l4 = self.psychology.maslow.levels[3].satisfaction as f32;
            let maslow_l5 = self.psychology.maslow.levels[4].satisfaction as f32;
            let shadow_arch = format!("{:?}", self.psychology.jung.dominant_archetype);
            let shadow_integ = self.psychology.jung.integration as f32;
            let eq_sc = self.psychology.eq.overall_eq as f32;
            let eq_aw = self.psychology.eq.self_awareness as f32;
            let eq_sr = self.psychology.eq.self_regulation as f32;
            let eq_mo = self.psychology.eq.motivation as f32;
            let eq_em = self.psychology.eq.empathy as f32;
            let eq_so = self.psychology.eq.social_skills as f32;
            let is_flow = self.psychology.flow.in_flow;
            let flow_int = self.psychology.flow.flow_intensity as f32;
            let flow_tot = self.psychology.flow.total_flow_cycles as i64;
            let psy_defense = format!("{:?}", self.psychology.freudian.ego.strategy);
            let maslow_need = self.psychology.maslow.levels.get(self.psychology.maslow.current_active_level)
                .map(|l| l.name.clone()).unwrap_or_default();
            let toltec_inv: i64 = self.psychology.toltec.agreements.iter().map(|a| a.times_invoked as i64).sum();
            let toltec_viol: i64 = self.psychology.toltec.agreements.iter().map(|a| a.violations_detected as i64).sum();
            let shadow_leak = self.psychology.jung.shadow_traits.iter().any(|t| t.leaking);
            let will_power = self.psychology.will.willpower as f32;
            let will_fatigue = self.psychology.will.decision_fatigue as f32;
            let will_total = self.psychology.will.total_deliberations as i64;
            let will_proud = self.psychology.will.proud_decisions as i64;
            let will_regretted = self.psychology.will.regretted_decisions as i64;
            let will_this_cycle = ctx.deliberation.is_some();
            let nn_learnings_n = ctx.nn_learnings_count;
            let is_sleeping = self.sleep.is_sleeping;
            let sleep_phase_str = self.sleep.current_cycle.as_ref()
                .map(|c| c.phase.as_str().to_string()).unwrap_or_default();
            let sleep_pressure_val = self.sleep.drive.sleep_pressure as f32;
            let awake_cycles_val = self.sleep.drive.awake_cycles as i64;
            let subconscious_act = self.subconscious.activation as f32;
            let pending_assoc = self.subconscious.pending_associations.len() as i32;
            let repressed_count = self.subconscious.repressed_content.len() as i32;
            let incubating_count = self.subconscious.incubating_problems.len() as i32;
            let neural_conn_total = self.sleep.total_connections_created as i64;
            // Sensibilite des recepteurs
            let rec_dop = self.hormonal_system.receptors.dopamine_receptors.sensitivity as f32;
            let rec_ser = self.hormonal_system.receptors.serotonin_receptors.sensitivity as f32;
            let rec_nor = self.hormonal_system.receptors.noradrenaline_receptors.sensitivity as f32;
            let rec_end = self.hormonal_system.receptors.endorphin_receptors.sensitivity as f32;
            let rec_oxy = self.hormonal_system.receptors.oxytocin_receptors.sensitivity as f32;
            let rec_adr = self.hormonal_system.receptors.adrenaline_receptors.sensitivity as f32;
            let rec_cor = self.hormonal_system.receptors.cortisol_receptors.sensitivity as f32;
            let rec_gab = self.hormonal_system.receptors.gaba_receptors.sensitivity as f32;
            let rec_glu = self.hormonal_system.receptors.glutamate_receptors.sensitivity as f32;
            // BDNF et matiere grise
            let bdnf_lvl = self.grey_matter.bdnf_level as f32;
            let neuroplast = self.grey_matter.neuroplasticity as f32;
            let syn_density = self.grey_matter.synaptic_density as f32;
            let gm_volume = self.grey_matter.grey_matter_volume as f32;
            let myelin = self.grey_matter.myelination as f32;
            tokio::spawn(async move {
                let _ = db.save_metric_snapshot(
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
                    ethics_count,
                    session_id,
                    survival_drive, existence_attachment,
                    intuition_acuity, intuition_accuracy,
                    premonition_accuracy, active_predictions,
                    senses_richness, &senses_dominant,
                    reading_beauty, &ambiance_scent, contact_warmth,
                    emergent_germinated,
                    &knowledge_sources,
                    &att_focus, att_depth, att_fatigue, att_concentration,
                    desires_active_n, desires_fulfilled_n, &desires_top,
                    n_comp, n_conn, n_expr, n_grow, n_mean,
                    lessons_total, lessons_confirmed, lessons_contradicted, behavior_changes,
                    wounds_active_n, wounds_healed_n, resilience_val,
                    dreams_total_n, dreams_insights, &last_dream_type,
                    psy_id_drive, psy_id_frust,
                    psy_ego_str, psy_ego_anx,
                    psy_sg_guilt, psy_sg_pride,
                    psy_conflict, psy_health,
                    maslow_ceil, maslow_l1, maslow_l2, maslow_l3, maslow_l4, maslow_l5,
                    &shadow_arch, shadow_integ,
                    eq_sc, eq_aw, eq_sr, eq_mo, eq_em, eq_so,
                    is_flow, flow_int, flow_tot,
                    &psy_defense, &maslow_need,
                    toltec_inv, toltec_viol, shadow_leak,
                    will_power, will_fatigue, will_total, will_proud, will_regretted, will_this_cycle,
                    nn_learnings_n,
                    is_sleeping, &sleep_phase_str, sleep_pressure_val, awake_cycles_val,
                    subconscious_act, pending_assoc, repressed_count,
                    incubating_count, neural_conn_total,
                    // Sensibilite des recepteurs
                    rec_dop, rec_ser, rec_nor, rec_end, rec_oxy,
                    rec_adr, rec_cor, rec_gab, rec_glu,
                    // BDNF et matiere grise
                    bdnf_lvl, neuroplast, syn_density, gm_volume, myelin,
                ).await;
            });
        }
    }
}
