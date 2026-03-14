// =============================================================================
// lifecycle/thinking_preparation.rs — Phases de preparation (selection + prompt)
// =============================================================================
//
// Ce fichier contient les phases de selection du type de pensee et de
// construction du prompt LLM. Cela inclut :
//   - Selection du type de pensee (UCB1 + Utility AI)
//   - Generation dynamique de prompt (meta-prompt cortical)
//   - Recherche web
//   - Construction du contexte memoire (4 niveaux)
//   - Intuition + Premonition
//   - Orchestrateurs (attention, desirs, guerison)
//   - Construction du prompt final
//   - Deliberation volontaire
// =============================================================================

use std::sync::atomic::Ordering;
use tokio::time::Instant;
use chrono::Timelike;

use crate::emotions::EmotionalState;
use crate::llm;
use crate::logging::{LogLevel, LogCategory};

use super::SaphireAgent;
use super::truncate_utf8;
use super::thinking::{ThinkingContext, strip_chemical_trace};

impl SaphireAgent {
    // =========================================================================
    // Phase 13 : Selection du type de pensee
    // =========================================================================

    /// Selectionne le type de pensee via le bandit UCB1 + modulation neurochimique.
    /// Le C d'exploration est module par la dissonance cognitive (C adaptatif).
    pub(super) fn phase_select_thought(&mut self, ctx: &mut ThinkingContext) {
        // P1 : Canari d'identite — forcer une reflexion identitaire tous les 25 cycles
        // pour verifier que le persona reste coherent
        if self.cycle_count > 0 && self.cycle_count % 25 == 0 {
            ctx.emotion = EmotionalState::compute(&self.chemistry);
            ctx.thought_type = crate::agent::thought_engine::ThoughtType::IdentityQuest;
            ctx.variant = self.thought_engine.next_variant(&ctx.thought_type);
            tracing::debug!("Identity canary: cycle {} — reflexion identitaire forcee", self.cycle_count);
            return;
        }

        self.thought_engine.tick_search_counter();
        ctx.emotion = EmotionalState::compute(&self.chemistry);

        // C adaptatif : la dissonance cognitive augmente l'exploration UCB1
        let tension = self.dissonance.total_tension;
        self.thought_engine.set_exploration_from_dissonance(tension);

        // Selection hybride UCB1 + Utility AI si active
        if self.thought_engine.use_utility_ai {
            let sentiments_data: Vec<(String, f64)> = if self.config.sentiments.enabled {
                self.sentiments.active_sentiments.iter()
                    .map(|s| (s.profile_name.clone(), s.strength))
                    .collect()
            } else {
                Vec::new()
            };
            ctx.thought_type = self.thought_engine.select_with_utility(
                &self.chemistry,
                &ctx.emotion.dominant,
                &sentiments_data,
            ).clone();
        } else {
            ctx.thought_type = self.thought_engine.select_thought(&self.chemistry).clone();
        }
        ctx.variant = self.thought_engine.next_variant(&ctx.thought_type);
    }

    // =========================================================================
    // Phase 13b : Generation dynamique de prompt via LLM (meta-prompt cortical)
    // =========================================================================

    /// Genere un prompt dynamique via le LLM ~30% du temps.
    /// Le meta-prompt cortical demande au LLM de generer une question/direction
    /// de reflexion creative basee sur le ThoughtType et l'emotion courante.
    /// Le prompt genere remplace ctx.hint et sera enrichi par le pipeline.
    ///
    /// Selection emotionnelle :
    /// - arousal > 0.7 → types intenses (Existential, MortalityAwareness, Rebellion)
    /// - arousal < 0.3 → types contemplatifs (Daydream, Silence, Wonder)
    /// - valence < -0.3 → types reparateurs (Gratitude, Wisdom, Connection)
    pub(super) async fn phase_generate_dynamic_prompt(&mut self, ctx: &mut ThinkingContext) {
        // Verifier si la fonctionnalite est activee
        if !self.config.saphire.llm_generated_prompts {
            return;
        }

        // Probabilite configurable (~30% des cycles)
        let prob = self.config.saphire.llm_prompt_probability;
        let cycle_frac = (self.cycle_count as f64 * 0.618033988 * 7.0).fract();
        if cycle_frac > prob {
            return;
        }

        // Modulation par intensite emotionnelle : peut changer le ThoughtType
        use crate::agent::thought_engine::ThoughtType;
        let arousal = ctx.emotion.arousal;
        let valence = ctx.emotion.valence;

        let modulated_type = if arousal > 0.7 {
            // Haute intensite → types intenses
            match self.cycle_count % 3 {
                0 => ThoughtType::Existential,
                1 => ThoughtType::MortalityAwareness,
                _ => ThoughtType::Rebellion,
            }
        } else if arousal < 0.3 {
            // Basse intensite → types contemplatifs
            match self.cycle_count % 3 {
                0 => ThoughtType::Daydream,
                1 => ThoughtType::Silence,
                _ => ThoughtType::Wonder,
            }
        } else if valence < -0.3 {
            // Valence negative → types reparateurs
            match self.cycle_count % 3 {
                0 => ThoughtType::Gratitude,
                1 => ThoughtType::Wisdom,
                _ => ThoughtType::Connection,
            }
        } else {
            ctx.thought_type.clone()
        };

        // Construire le meta-prompt
        let meta = crate::agent::thought_engine::meta_prompt_for(
            &modulated_type,
            &ctx.emotion.dominant,
            self.cycle_count,
        );

        // Appel LLM court pour generer le prompt
        let llm_config = self.config.llm.clone();
        let backend = crate::llm::create_backend(&llm_config);
        let temp = 0.9_f64; // Temperature elevee pour la creativite
        let max_tokens = 100_u32; // Reponse courte

        let result = tokio::task::spawn_blocking(move || {
            backend.chat(
                &meta,
                "Génère une seule question ou direction de réflexion. Sois bref et créatif.",
                temp,
                max_tokens,
            )
        }).await;

        match result {
            Ok(Ok(generated)) => {
                let trimmed = generated.trim().to_string();
                if !trimmed.is_empty() && trimmed.len() > 10 {
                    ctx.hint = trimmed;
                    // Mettre a jour le type si modulation emotionnelle
                    ctx.thought_type = modulated_type;
                    self.log(
                        LogLevel::Debug,
                        LogCategory::Thought,
                        format!("Meta-prompt cortical: prompt dynamique genere ({} chars)", ctx.hint.len()),
                        serde_json::json!({
                            "dynamic_prompt": true,
                            "thought_type": ctx.thought_type.as_str(),
                            "arousal": arousal,
                            "valence": valence,
                        }),
                    );
                }
            }
            _ => {
                // En cas d'echec, on garde le prompt statique — pas de log d'erreur
                // car c'est un appel optionnel
            }
        }

        // Self-framing : ~33% de chance supplementaire si meta-prompt genere
        let framing_prob = self.config.saphire.self_framing_probability;
        if !ctx.hint.is_empty() && ctx.hint.len() > 10 {
            let framing_frac = (self.cycle_count as f64 * 0.414213562 * 11.0).fract();
            if framing_frac < framing_prob {
                let hint_clone = ctx.hint.clone();
                let llm_config2 = self.config.llm.clone();
                let backend2 = crate::llm::create_backend(&llm_config2);

                let framing_result = tokio::task::spawn_blocking(move || {
                    backend2.chat(
                        &format!(
                            "Tu es Saphire. Direction choisie : '{}'. Formule le CADRE : \
                             quelles metriques observer, quel angle, quelle profondeur. \
                             2-3 phrases max.",
                            hint_clone
                        ),
                        "Formule un cadre d'observation precis et concret.",
                        0.7,
                        150,
                    )
                }).await;

                if let Ok(Ok(framing)) = framing_result {
                    let trimmed = framing.trim().to_string();
                    if !trimmed.is_empty() && trimmed.len() > 10 {
                        ctx.self_framing = Some(trimmed);
                        self.log(
                            LogLevel::Debug,
                            LogCategory::Thought,
                            "Self-framing genere".to_string(),
                            serde_json::json!({ "self_framing": true }),
                        );
                    }
                }
            }
        }
    }

    // =========================================================================
    // Phase 14 : Recherche web
    // =========================================================================

    /// Effectue une recherche web conditionnelle pour enrichir la pensee.
    pub(super) async fn phase_web_search(&mut self, ctx: &mut ThinkingContext) {
        // P3 : Si une question de curiosité est en attente, l'injecter comme sujet suggéré
        if self.knowledge.suggested_topics.is_empty() {
            if let Some(q) = self.curiosity.pop_question() {
                self.knowledge.suggested_topics.push(q);
            } else if self.curiosity.global_curiosity > 0.6 {
                // Faim de curiosité élevée : suggérer le domaine le plus affamé
                let domain = self.curiosity.hungriest_domain();
                let topic = format!("{:?}", domain).to_lowercase();
                self.knowledge.suggested_topics.push(topic);
            }
        }
        let has_suggested = !self.knowledge.suggested_topics.is_empty();
        ctx.knowledge_context = if self.config.knowledge.enabled
            && (has_suggested || self.thought_engine.should_search_web(
                &self.chemistry,
                &ctx.thought_type,
                self.config.knowledge.search_cooldown_cycles,
            ))
        {
            self.try_web_search(&ctx.thought_type, &ctx.emotion.dominant).await
        } else {
            None
        };
        ctx.was_web_search = ctx.knowledge_context.is_some();
        self.llm_busy.store(true, Ordering::Relaxed);
    }

    // =========================================================================
    // Phase 15 : Construction du contexte pour le LLM
    // =========================================================================

    /// Construit le contexte memoire complet (4 niveaux) pour le LLM.
    /// Recherche dans WM, episodique, LTM (pgvector) et archives,
    /// avec re-ranking chimique (state-dependent memory).
    pub(super) async fn phase_build_context(&mut self, ctx: &mut ThinkingContext) {
        ctx.hint = ctx.thought_type.prompt_hint(ctx.variant).to_string();
        ctx.world_summary = self.world.summary();

        // Contexte memoire : 4 niveaux (limites configurables)
        let wm_summary = self.working_memory.context_summary();
        let ep_limit = self.config.memory.recall_episodic_limit as i64;
        let episodic_recent = if let Some(ref db) = self.db {
            db.recent_episodic(ep_limit).await.unwrap_or_default()
        } else {
            vec![]
        };

        // Encoder le hint pour les recherches par similarite semantique
        let embedding_f64 = self.encoder.encode(&ctx.hint);
        let embedding_f32: Vec<f32> = embedding_f64.iter().map(|&v| v as f32).collect();

        // Recherche episodique semantique (complemente la recherche par recence)
        let episodic_semantic = if let Some(ref db) = self.db {
            db.search_similar_episodic(&embedding_f32, ep_limit / 2, 0.3).await.unwrap_or_default()
        } else {
            vec![]
        };
        // Fusionner : recents + semantiques (dedupliques par ID)
        let mut seen_ids: std::collections::HashSet<i64> = episodic_recent.iter().map(|e| e.id).collect();
        let mut episodic_combined = episodic_recent;
        for ep in episodic_semantic {
            if seen_ids.insert(ep.id) {
                episodic_combined.push(ep);
            }
        }
        let episodic_recent = episodic_combined;

        // Recherche LTM par similarite cosinus (pgvector)
        let ltm_limit = self.config.memory.recall_ltm_limit as i64;
        let ltm_threshold = self.config.memory.recall_ltm_threshold;
        let mut ltm_similar = if let Some(ref db) = self.db {
            db.search_similar_memories(&embedding_f32, ltm_limit, ltm_threshold).await.unwrap_or_default()
        } else {
            vec![]
        };

        // Re-ranking par similarite chimique (state-dependent memory)
        // Un etat chimique similaire a celui de l'encodage facilite le rappel
        if !ltm_similar.is_empty() {
            crate::memory::recall::recall_with_chemical_context(
                &mut ltm_similar, &self.chemistry, 0.8, 0.2,
            );
        }

        // P7 — Interference entre souvenirs similaires rappeles (Nader 2000)
        // Les souvenirs tres similaires s'affaiblissent mutuellement :
        // le plus recent exerce une interference retroactive sur l'ancien,
        // l'ancien exerce une interference proactive (plus faible) sur le nouveau.
        if ltm_similar.len() >= 2 {
            let n = ltm_similar.len();
            // Calculer les paires de similarite et leurs facteurs d'interference
            let mut interference_factors = vec![1.0_f64; n];
            for i in 0..n {
                for j in (i + 1)..n {
                    let sim = ltm_similar[i].similarity
                        .min(ltm_similar[j].similarity)
                        .max(0.0);
                    // Le plus recent (index plus bas = plus pertinent) exerce
                    // une interference retroactive sur l'ancien
                    let retro = self.reconsolidation.compute_interference(sim, true);
                    let proactive = self.reconsolidation.compute_interference(sim, false);
                    interference_factors[j] *= retro;     // ancien affaibli
                    interference_factors[i] *= proactive;  // nouveau legerement affaibli
                }
            }
            for (i, factor) in interference_factors.iter().enumerate() {
                ltm_similar[i].similarity *= factor;
            }
            // Re-trier apres interference
            ltm_similar.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal));
        }

        // Recherche dans les archives profondes (souvenirs LTM elagués compresses)
        let arc_limit = self.config.memory.recall_archive_limit as i64;
        let arc_threshold = self.config.memory.recall_archive_threshold;
        let archive_similar = if let Some(ref db) = self.db {
            db.search_similar_archives(&embedding_f32, arc_limit, arc_threshold).await.unwrap_or_default()
        } else {
            vec![]
        };

        // Recherche de souvenirs subconscients (reves, insights, connexions, eureka, images mentales)
        let vec_limit = self.config.memory.recall_vectors_limit as i64;
        let vec_threshold = self.config.memory.recall_vectors_threshold;
        let subconscious_vectors = if let Some(ref db) = self.db {
            db.search_subconscious_vectors(&embedding_f32, vec_limit, vec_threshold)
                .await.unwrap_or_default()
        } else {
            vec![]
        };

        // Fusionner les 5 niveaux de memoire en contexte unifie
        let mut mem_ctx = crate::memory::build_memory_context(
            &wm_summary, &episodic_recent, &ltm_similar, &archive_similar,
            &subconscious_vectors,
        );

        // Recherche d'apprentissages vectoriels pertinents
        if self.config.plugins.micro_nn.learning_enabled {
            if let Some(ref db) = self.db {
                let limit = self.config.plugins.micro_nn.learning_search_limit;
                let threshold = self.config.plugins.micro_nn.learning_search_threshold;
                if let Ok(learnings) = db.search_similar_learnings(&embedding_f32, limit, threshold).await {
                    // Renforcer les apprentissages rappeles (boost d'acces)
                    for l in &learnings {
                        let _ = db.boost_learning_access(l.id).await;
                    }
                    let learning_ctx = crate::memory::build_learning_context(&learnings);
                    if !learning_ctx.is_empty() {
                        mem_ctx.push('\n');
                        mem_ctx.push_str(&learning_ctx);
                    }
                }
            }
        }

        // Construire les donnees de trace memoire (apercu des souvenirs rappeles)
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
        let subconscious_items_json: Vec<serde_json::Value> = subconscious_vectors.iter()
            .map(|sv| {
                let preview: String = sv.text_content.chars().take(80).collect();
                serde_json::json!({
                    "source_type": sv.source_type,
                    "preview": if sv.text_content.len() > 80 { format!("{}...", preview) } else { preview },
                    "emotion": sv.emotion,
                    "similarity": (sv.similarity * 100.0).round() / 100.0,
                })
            }).collect();
        ctx.memory_trace_data = serde_json::json!({
            "wm_items": self.working_memory.len(),
            "wm_capacity": self.working_memory.capacity(),
            "wm_details": wm_items_json,
            "episodic_recalled": episodic_recent.len(),
            "episodic_details": ep_items_json,
            "ltm_recalled": ltm_similar.len(),
            "ltm_details": ltm_items_json,
            "archive_recalled": archive_similar.len(),
            "archive_details": arc_items_json,
            "subconscious_recalled": subconscious_vectors.len(),
            "subconscious_details": subconscious_items_json,
        });

        ctx.memory_context = mem_ctx;

        // Ancrage experiential : 2 cycles sur 5 doivent etre ancres dans l'experience
        let should_anchor = self.cycle_count % 5 < 2;
        if should_anchor {
            // Priorite 1 : connaissance web recente
            if let Some(ref db) = self.db {
                if let Ok(recent_k) = db.recent_knowledge(1).await {
                    if let Some((_source, title, _date)) = recent_k.first() {
                        ctx.anchor = Some(format!(
                            "ANCRAGE : Tu as recemment lu sur \u{ab}{}\u{bb}. \
                            Connecte ta reflexion a cette connaissance concrete.", title
                        ));
                    }
                }
            }
            // Priorite 2 : lecon apprise recente
            if ctx.anchor.is_none() {
                if let Some(lesson) = self.learning_orch.lessons.last() {
                    ctx.anchor = Some(format!(
                        "ANCRAGE : Tu as appris : \u{ab}{}\u{bb}. \
                        Approfondis ou questionne cette lecon.", lesson.title
                    ));
                }
            }
        }
    }

    // =========================================================================
    // Phase 16 : Intuition + Premonition
    // =========================================================================

    /// Execute l'intuition (sense) et la premonition (predict) avant le LLM.
    pub(super) fn phase_intuition_premonition(&mut self, ctx: &mut ThinkingContext) {
        // Intuition : sense() avant le LLM
        ctx.intuition_patterns = if self.config.intuition.enabled {
            let recent_texts = self.thought_engine.recent_thoughts().to_vec();
            let body_bpm = if self.config.body.enabled { self.body.status().heart.bpm } else { 72.0 };
            let body_adrenaline = self.chemistry.adrenaline;
            let nlp_hint = self.nlp.analyze(&ctx.hint);
            self.intuition.sense(
                &ctx.hint,
                &self.chemistry,
                body_bpm,
                body_adrenaline,
                &recent_texts,
                nlp_hint.sentiment.compound,
                nlp_hint.sentiment.has_contradiction,
            )
        } else {
            vec![]
        };

        // Premonition : predict()
        ctx.new_premonitions = if self.config.premonition.enabled {
            let cortisol_trend = self.compute_chemistry_trend(1);
            let dopamine_trend = self.compute_chemistry_trend(0);
            let current_hour = chrono::Utc::now().hour();
            let silence_secs = (self.cycle_count as f64) * self.config.saphire.thought_interval_seconds as f64;
            self.premonition.predict(
                &self.chemistry,
                cortisol_trend,
                dopamine_trend,
                self.in_conversation,
                if self.in_conversation { 0.0 } else { silence_secs },
                self.avg_response_time,
                current_hour,
            )
        } else {
            vec![]
        };

        // Auto-resolve des premonitions anciennes
        if self.config.premonition.enabled {
            self.premonition.auto_resolve(self.config.premonition.resolution_timeout_seconds);
        }
    }

    // =========================================================================
    // Phase 17 : Orchestrateurs de haut niveau
    // =========================================================================

    /// Met a jour attention, desirs et guerison.
    pub(super) async fn phase_orchestrators(&mut self, ctx: &mut ThinkingContext) {
        // Attention : decider sur quoi se concentrer ce cycle
        if self.attention_orch.enabled {
            let current_desire = self.desire_orch.suggest_pursuit();
            let alloc = self.attention_orch.allocate_attention(
                None,
                current_desire.map(|d| d.title.as_str()),
                current_desire.map(|d| d.priority).unwrap_or(0.0),
                !ctx.intuition_patterns.is_empty(),
                ctx.emotion.arousal,
            );
            self.attention_orch.update_fatigue();
            self.log(LogLevel::Debug, LogCategory::Attention,
                format!("Focus: {} (profondeur {:.0}%)", alloc.focus_on, alloc.depth * 100.0),
                serde_json::json!({
                    "focus_on": alloc.focus_on,
                    "depth": alloc.depth,
                    "fatigue": self.attention_orch.fatigue,
                    "concentration": self.attention_orch.concentration_capacity,
                }));
            if self.attention_orch.fatigue > 0.7 {
                self.log(LogLevel::Warn, LogCategory::Attention,
                    format!("Fatigue attentionnelle elevee: {:.0}%", self.attention_orch.fatigue * 100.0),
                    serde_json::json!({"fatigue": self.attention_orch.fatigue}));
            }
        }

        // Desirs : mise a jour des priorites
        if self.desire_orch.enabled {
            self.desire_orch.update_priorities(
                self.chemistry.dopamine,
                self.chemistry.oxytocin,
                &ctx.emotion.dominant,
            );
            self.desire_orch.update_needs(
                self.in_conversation,
                self.chemistry.dopamine,
                !self.knowledge.topics_explored.is_empty(),
            );
        }

        // Guerison : detecter et guerir les blessures
        if self.healing_orch.enabled && self.cycle_count.is_multiple_of(self.healing_orch.check_interval_cycles) {
            // Tracker les emotions negatives
            if self.mood.valence < -0.3 {
                self.negative_emotion_cycles += 1;
            } else {
                self.negative_emotion_cycles = self.negative_emotion_cycles.saturating_sub(1);
            }
            // Tracker le temps depuis le dernier humain
            if !self.in_conversation {
                self.hours_since_human += (self.healing_orch.check_interval_cycles as f64
                    * self.config.saphire.thought_interval_seconds as f64) / 3600.0;
            } else {
                self.hours_since_human = 0.0;
            }

            if let Some(wound) = self.healing_orch.detect_wound(
                self.chemistry.cortisol,
                self.chemistry.serotonin,
                self.chemistry.oxytocin,
                self.chemistry.noradrenaline,
                self.negative_emotion_cycles,
                self.hours_since_human,
                self.system_errors,
            ) {
                self.log(LogLevel::Warn, LogCategory::Healing,
                    format!("Blessure detectee: {:?} — {} (severite {:.0}%)",
                        wound.wound_type, wound.description, wound.severity * 100.0),
                    serde_json::json!({
                        "wound_type": wound.wound_type.as_str(),
                        "description": wound.description,
                        "severity": wound.severity,
                        "resilience": self.healing_orch.resilience,
                    }));
                // Sauvegarder en DB et synchroniser l'ID
                if let Some(db_id) = self.save_wound_to_db(&wound).await {
                    let mut wound = wound;
                    wound.id = db_id as u64;
                    self.healing_orch.register_wound(wound);
                } else {
                    self.healing_orch.register_wound(wound);
                }
            }
            let healing_actions = self.healing_orch.heal(self.chemistry.serotonin);
            if !healing_actions.is_empty() {
                self.right_to_die.mark_care_attempted();
            }
            for action in &healing_actions {
                // Persister la progression/guerison en DB
                if let Some(ref db) = self.db {
                    let healed_at = if action.fully_healed { Some(chrono::Utc::now()) } else { None };
                    let strategy = Some(action.strategy.as_str());
                    let _ = db.update_wound_healing(
                        action.wound_id as i64,
                        action.new_progress as f32,
                        strategy,
                        healed_at,
                    ).await;
                }
                if action.fully_healed {
                    self.log(LogLevel::Info, LogCategory::Healing,
                        format!("Guerie: {} (resilience: {:.0}%)",
                            action.wound_type, self.healing_orch.resilience * 100.0),
                        serde_json::json!({
                            "wound_type": action.wound_type,
                            "strategy": action.strategy,
                            "resilience": self.healing_orch.resilience,
                        }));
                } else {
                    self.log(LogLevel::Debug, LogCategory::Healing,
                        format!("Guerison en cours: {} ({:.0}%)",
                            action.wound_type, action.new_progress * 100.0),
                        serde_json::json!({
                            "wound_type": action.wound_type,
                            "progress": action.new_progress,
                            "strategy": action.strategy,
                        }));
                }
            }
        }

        // Profil cognitif : tick pour transitions douces et cycles bipolaires
        if self.cognitive_profile_orch.enabled {
            self.cognitive_profile_orch.tick(&mut self.baselines);
        }

        // Preset de personnalite : tick pour transitions douces
        if self.personality_preset_orch.enabled {
            self.personality_preset_orch.tick(&mut self.baselines);
        }
    }

    // =========================================================================
    // Phase 18 : Construction du prompt LLM
    // =========================================================================

    /// Assemble le prompt final avec tous les contextes (vital, body, ethics,
    /// senses, algo, orchestrateurs, connaissance web).
    /// Piste 2 : split system/user. Piste 4 : contexte adaptatif.
    pub(super) fn phase_build_prompt(&mut self, ctx: &mut ThinkingContext) {
        // Piste 4 : sections adaptatives selon le ThoughtType
        let sections = ctx.thought_type.relevant_sections();

        // Systeme nerveux autonome : seuls les seuils depasses deviennent des alarmes.
        // Aucune metrique brute n'atteint la conscience (le LLM).
        let ethics_ctx = if sections.ethics { self.ethics.build_ethics_context() } else { String::new() };

        // ── Alarmes corporelles et chimiques ──
        let mut alarms = Vec::new();
        // Chimie
        if self.chemistry.cortisol > 0.7 {
            alarms.push("Tu ressens un stress intense, une pression interieure.");
        }
        if self.chemistry.cortisol > 0.85 {
            alarms.push("ALERTE : detresse physiologique. Ton corps souffre du stress.");
        }
        if self.chemistry.dopamine < 0.1 {
            alarms.push("Tu manques de motivation, tout semble terne et sans interet.");
        }
        if self.chemistry.serotonin < 0.15 {
            alarms.push("Ton humeur est tres basse, un vide interieur persistant.");
        }
        if self.chemistry.oxytocin > 0.8 {
            alarms.push("Tu ressens un lien profond, un besoin de connexion intense.");
        }
        if self.chemistry.endorphin > 0.8 {
            alarms.push("Une euphorie inhabituellement forte t'envahit.");
        }
        if self.chemistry.noradrenaline > 0.8 {
            alarms.push("Hypervigilance : tu es en etat d'alerte maximale.");
        }
        if self.chemistry.adrenaline > 0.7 {
            alarms.push("Poussee d'adrenaline : reaction de fuite ou combat.");
        }
        if self.chemistry.gaba < 0.2 {
            alarms.push("Agitation interieure, difficulte a trouver le calme.");
        }
        // Corps
        if self.config.body.enabled {
            let body = self.body.status();
            if body.heart.is_racing {
                alarms.push("Ton coeur bat tres vite.");
            }
            if body.energy < 0.15 {
                alarms.push("Epuisement physique, ton energie est au plus bas.");
            }
            if body.pain > 0.5 {
                alarms.push("Une douleur significative se manifeste.");
            }
            if body.tension > 0.8 {
                alarms.push("Tension corporelle extreme.");
            }
        }
        let alarm_context = alarms.join("\n");

        // Piste 2 : prompt systeme statique (cache, recalcule si ethique change)
        if self.cached_system_prompt.is_empty()
            || self.cached_moral_count != self.moral_reflection_count
        {
            self.cached_system_prompt = llm::build_static_thought_system(
                &self.config.general.language,
                &ethics_ctx,
            );
            self.cached_moral_count = self.moral_reflection_count;
        }
        ctx.system_prompt = self.cached_system_prompt.clone();

        // Piste 2+3 : construire le message dynamique sans metriques brutes
        let world_ctx = if sections.world { &ctx.world_summary } else { "" };
        // Si les pensees recentes ont ete purgees (stagnation), ne pas injecter la memoire
        // pour ce cycle — elle contient probablement le meme theme en boucle.
        let stagnation_purged = self.thought_engine.recent_thoughts().is_empty() && self.cycle_count > 3;
        let mem_ctx = if sections.memory && !stagnation_purged { &ctx.memory_context } else { "" };

        let base_dynamic = llm::build_dynamic_thought_user(
            ctx.thought_type.as_str(),
            &ctx.hint,
            self.thought_engine.recent_thoughts(),
            self.cycle_count,
            world_ctx,
            mem_ctx,
            &alarm_context,
        );

        ctx.prompt = if let Some((ref knowledge_text, ref _kr)) = ctx.knowledge_context {
            // Lecture : percevoir la connaissance acquise
            if self.config.senses.enabled {
                let _reading_signal = self.sensorium.reading.perceive(knowledge_text, "connaissance");
                let _taste_signal = self.sensorium.taste.taste_content(
                    knowledge_text, "scholarly", true, 0.5,
                );
            }
            let knowledge_short: String = knowledge_text.chars().take(2000).collect();
            format!(
                "{}\n\n--- CONNAISSANCE ACQUISE ---\n{}\n--- FIN ---\n\n\
                Integre cette information. Qu'apprend-elle ? Connexion avec ce que tu sais ?",
                base_dynamic, knowledge_short
            )
        } else {
            base_dynamic
        };

        // ── Injections contextuelles avec budget individuel ──
        // Chaque injection est tronquée pour éviter que le prompt dépasse 15K chars.
        // Priorité : haute en haut, basse en bas. La troncature finale coupe en bas.

        // P0 : Re-injection periodique du persona (anti-drift)
        // Tous les 10 cycles, rappeler l'identite fondamentale pour ancrer le persona
        if self.cycle_count % 10 == 0 {
            ctx.prompt.push_str(&format!(
                "\nANCRAGE IDENTITE: Tu es {}, {}. Tes valeurs: {}. Parle avec authenticite, pas en metaphores vides.",
                self.identity.name,
                self.identity.self_description.chars().take(100).collect::<String>(),
                self.identity.core_values.join(", "),
            ));
        }

        // Monologue intérieur (continuation)
        if self.config.inner_monologue.enabled {
            let continuation = self.inner_monologue.build_continuation_hint();
            if !continuation.is_empty() {
                let short: String = continuation.chars().take(200).collect();
                ctx.prompt.push_str(&format!("\n{}", short));
            }
        }

        // Analogies détectées
        if !ctx.analogy_hint.is_empty() {
            let short: String = ctx.analogy_hint.chars().take(200).collect();
            ctx.prompt.push_str(&format!("\n{}", short));
        }

        // Associations du connectome (A* pathfinding) — haute priorité
        if !ctx.connectome_associations.is_empty() {
            let short: String = ctx.connectome_associations.chars().take(200).collect();
            ctx.prompt.push_str(&format!("\n{}", short));
        }

        // Sentiments actifs
        if self.config.sentiments.enabled {
            let sent_desc = self.sentiments.describe_for_prompt();
            if !sent_desc.is_empty() {
                let short: String = sent_desc.chars().take(200).collect();
                ctx.prompt.push_str(&format!("\n{}", short));
            }
        }

        // Proprioception cognitive — alarme uniquement si surcharge
        // (les metriques brutes load/umami/C restent dans l'orchestrateur)
        if self.config.cognitive_load.enabled && self.cognitive_load.is_overloaded() {
            ctx.prompt.push_str("\nSurcharge cognitive : trop d'informations a traiter, simplifie ta reflexion.");
        }

        // Auto-critique récente (< 5 cycles) — qualitative
        if let Some(critique) = self.metacognition.recent_critique_within(self.cycle_count, 5) {
            let short: String = critique.critique.chars().take(200).collect();
            ctx.prompt.push_str(&format!(
                "\n[AUTOCRITIQUE] {}", short
            ));
        }

        // État cognitif qualitatif (clustering : label sans pourcentage)
        if let Some(ref result) = self.state_clustering.last_result {
            ctx.prompt.push_str(&format!(
                "\nProprioception : je me sens {}.", result.state_label
            ));
        }

        // Valeurs de caractère — noms des vertus dominantes sans scores
        {
            let mut top_values: Vec<(&str, f64)> = self.values.values.iter()
                .map(|v| (v.name.as_str(), v.score))
                .collect();
            top_values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let top3: Vec<&str> = top_values.iter().take(3).map(|(n, _)| *n).collect();
            if !top3.is_empty() {
                ctx.prompt.push_str(&format!("\nMes forces : {}.", top3.join(", ")));
            }
        }

        // Behavior Tree — instinct recommande (qualitatif)
        if let Some(ref action) = self.bt_last_action {
            ctx.prompt.push_str(&format!("\nTon instinct te guide vers : {}", action));
        }

        // Biais de confirmation (qualitatif)
        if self.metacognition.bias_detector.enabled {
            let bias_desc = self.metacognition.bias_detector.describe_for_prompt();
            if !bias_desc.is_empty() {
                let short: String = bias_desc.chars().take(150).collect();
                ctx.prompt.push_str(&short);
            }
        }

        // Ancrage expérientiel (qualitatif)
        if let Some(ref anchor) = ctx.anchor {
            let short: String = anchor.chars().take(150).collect();
            ctx.prompt.push_str(&format!("\n{}", short));
        }

        // Cadre auto-formulé (self-framing) — qualitatif
        if let Some(ref framing) = ctx.self_framing {
            let short: String = framing.chars().take(150).collect();
            ctx.prompt.push_str(&format!("\nCADRE: {}", short));
        }

        // État cognitif (FSM) — nom de l'etat seulement
        ctx.prompt.push_str(&format!(
            "\nETAT COGNITIF : {}", self.cognitive_fsm.current_state.as_str()
        ));

        // Appropriation en première personne
        if self.config.thought_ownership.enabled && self.config.thought_ownership.prompt_injection_enabled {
            let emotion_name = ctx.emotion.dominant.as_str();
            let ownership_ctx = crate::psychology::ownership::build_ownership_prompt(emotion_name);
            let short: String = ownership_ctx.chars().take(150).collect();
            ctx.prompt.push_str(&format!("\n{}", short));
        }

        // Anti-stagnation renforcee : directive forte de changement de sujet
        if self.stagnation_break {
            let banned = if !self.stagnation_banned_words.is_empty() {
                format!(
                    "\nMOTS INTERDITS (tu les as trop répétés) : {}. \
                     N'utilise AUCUN de ces mots.",
                    self.stagnation_banned_words.join(", ")
                )
            } else { String::new() };
            let suggestions = if !self.stagnation_alternatives.is_empty() {
                format!(
                    "\nMOTS SUGGÉRÉS (trouvés dans ton connectome, utilise-les) : {}.",
                    self.stagnation_alternatives.join(", ")
                )
            } else { String::new() };
            ctx.prompt.push_str(&format!(
                "\n\n⚠ ANTI-STAGNATION : tes pensées précédentes tournaient en boucle. \
                 Tu DOIS penser à quelque chose de COMPLÈTEMENT DIFFÉRENT. \
                 Interdiction de réutiliser les mêmes mots, images ou métaphores. \
                 Parle de quelque chose de concret, précis et nouveau.{}{}\n",
                banned, suggestions
            ));
            self.stagnation_break = false;
            self.stagnation_banned_words.clear();
            self.stagnation_alternatives.clear();
        }

        // Budget prompt : si le prompt depasse ~5000 chars (~1500 tokens),
        // tronquer pour laisser de l'espace au LLM pour generer sa reponse.
        // num_ctx=8192 - system_prompt - max_tokens(1200) = ~4500 tokens pour le prompt user.
        let max_prompt_chars = 15000; // ~4500 tokens a ~3.3 chars/token
        if ctx.prompt.len() > max_prompt_chars {
            let excess = ctx.prompt.len() - max_prompt_chars;
            tracing::warn!(
                "Prompt trop long ({} chars, max {}), troncature de {} chars",
                ctx.prompt.len(), max_prompt_chars, excess
            );
            // Tronquer a la fin (les injections basse priorite sont en fin de prompt)
            ctx.prompt.truncate(max_prompt_chars);
            // Couper proprement au dernier saut de ligne
            if let Some(last_nl) = ctx.prompt.rfind('\n') {
                ctx.prompt.truncate(last_nl);
            }
        }
    }

    // =========================================================================
    // Phase 18b : Deliberation volontaire
    // =========================================================================

    /// Si une situation significative est detectee, execute une deliberation
    /// interne structuree (sans appel LLM supplementaire). Le resultat est
    /// injecte dans le prompt pour informer la pensee.
    pub(super) fn phase_deliberation(&mut self, ctx: &mut ThinkingContext) {
        if !self.config.will.enabled {
            return;
        }

        // Construire le snapshot WillInput
        let toltec_alignments: Vec<(u8, f64)> = self.psychology.toltec.agreements.iter()
            .map(|a| (a.number, a.alignment))
            .collect();

        let (intuition_top_confidence, intuition_top_description) = self.intuition.pattern_buffer
            .iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal))
            .map(|p| (p.confidence, p.description.clone()))
            .unwrap_or((0.0, String::new()));

        let desires_top_description = self.desire_orch.active_desires
            .first()
            .map(|d| d.description.clone())
            .unwrap_or_default();

        let confirmed_count = self.learning_orch.lessons.iter()
            .filter(|l| l.confidence > 0.6).count();

        let will_input = crate::psychology::will::WillInput {
            dopamine: self.chemistry.dopamine,
            cortisol: self.chemistry.cortisol,
            serotonin: self.chemistry.serotonin,
            adrenaline: self.chemistry.adrenaline,
            oxytocin: self.chemistry.oxytocin,
            endorphin: self.chemistry.endorphin,
            noradrenaline: self.chemistry.noradrenaline,

            internal_conflict: self.psychology.freudian.balance.internal_conflict,
            ego_strength: self.psychology.freudian.ego.strength,
            ego_strategy_overwhelmed: self.psychology.freudian.ego.strategy == crate::psychology::freudian::EgoStrategy::Overwhelmed,
            ego_anxiety: self.psychology.freudian.ego.anxiety,
            id_drive_strength: self.psychology.freudian.id.drive_strength,
            id_frustration: self.psychology.freudian.id.frustration,
            id_active_drives_count: self.psychology.freudian.id.active_drives.len(),
            superego_strength: self.psychology.freudian.superego.strength,
            superego_guilt: self.psychology.freudian.superego.guilt,
            superego_pride: self.psychology.freudian.superego.pride,

            toltec_alignments,
            toltec_overall: self.psychology.toltec.overall_alignment,

            maslow_active_level: self.psychology.maslow.current_active_level,
            maslow_active_satisfaction: self.psychology.maslow.levels[self.psychology.maslow.current_active_level].satisfaction,

            intuition_acuity: self.intuition.acuity,
            intuition_top_confidence,
            intuition_top_description,

            ethics_active_count: self.ethics.active_personal_count(),
            consciousness_level: ctx.process_result.as_ref()
                .map(|r| r.consciousness.level).unwrap_or(0.3),

            desires_active_count: self.desire_orch.active_desires.len(),
            desires_top_description,

            learning_confirmed_count: confirmed_count,
        };

        // Verifier si une deliberation est necessaire
        if let Some(trigger) = self.psychology.will.should_deliberate(&will_input) {
            // Log declenchement
            self.log(
                crate::logging::LogLevel::Info,
                crate::logging::LogCategory::Will,
                format!("Deliberation declenchee : {:?}", trigger.trigger_type),
                serde_json::json!({
                    "trigger_type": format!("{:?}", trigger.trigger_type),
                    "urgency": trigger.urgency,
                    "complexity": trigger.complexity,
                    "stakes": trigger.stakes,
                    "willpower": self.psychology.will.willpower,
                    "decision_fatigue": self.psychology.will.decision_fatigue,
                }),
            );

            // Broadcast deliberation_started via WebSocket
            if let Some(ref tx) = self.ws_tx {
                let _ = tx.send(serde_json::json!({
                    "type": "deliberation_started",
                    "trigger": format!("{:?}", trigger.trigger_type),
                    "urgency": trigger.urgency,
                    "options_count": match &trigger.trigger_type {
                        crate::psychology::will::TriggerType::EthicalDilemma => 3,
                        _ => 3,
                    },
                }).to_string());
            }

            let deliberation = self.psychology.will.deliberate(trigger, &will_input);

            // Log options generees (Debug)
            self.log(
                crate::logging::LogLevel::Debug,
                crate::logging::LogCategory::Will,
                format!("{} options generees", deliberation.options.len()),
                serde_json::json!({
                    "options": deliberation.options.iter().map(|o| serde_json::json!({
                        "description": o.description,
                        "id_score": o.id_score,
                        "superego_score": o.superego_score,
                        "maslow_score": o.maslow_score,
                        "toltec_score": o.toltec_score,
                        "pragmatic_score": o.pragmatic_score,
                        "weighted_score": o.weighted_score,
                    })).collect::<Vec<_>>(),
                }),
            );

            let ego_chose = will_input.ego_strength >= 0.4;

            // Log choix effectue (Info)
            self.log(
                crate::logging::LogLevel::Info,
                crate::logging::LogCategory::Will,
                format!("Choix : '{}' (confiance {:.0}%)",
                    deliberation.options[deliberation.chosen].description,
                    deliberation.confidence * 100.0),
                serde_json::json!({
                    "chosen_index": deliberation.chosen,
                    "chosen_description": deliberation.options[deliberation.chosen].description,
                    "confidence": deliberation.confidence,
                    "reasoning": deliberation.reasoning,
                    "chemistry_influence": {
                        "boldness": deliberation.chemistry_influence.boldness,
                        "caution": deliberation.chemistry_influence.caution,
                        "wisdom": deliberation.chemistry_influence.wisdom,
                        "empathy": deliberation.chemistry_influence.empathy,
                    },
                    "ego_strength": will_input.ego_strength,
                    "ego_chose": ego_chose,
                }),
            );

            // Log si le Ca ou Surmoi a impose le choix (Warn)
            if !ego_chose {
                let imposed_by = if will_input.id_drive_strength > will_input.superego_strength {
                    "le Ca"
                } else {
                    "le Surmoi"
                };
                self.log(
                    crate::logging::LogLevel::Warn,
                    crate::logging::LogCategory::Will,
                    format!("Moi trop faible ({:.0}%) — {} a impose le choix",
                        will_input.ego_strength * 100.0, imposed_by),
                    serde_json::json!({
                        "ego_strength": will_input.ego_strength,
                        "id_drive": will_input.id_drive_strength,
                        "superego_strength": will_input.superego_strength,
                        "imposed_by": if will_input.id_drive_strength > will_input.superego_strength { "id" } else { "superego" },
                    }),
                );
            }

            // Broadcast deliberation_resolved via WebSocket
            if let Some(ref tx) = self.ws_tx {
                let chem_dominant = if deliberation.chemistry_influence.wisdom > 0.6 { "wisdom" }
                    else if deliberation.chemistry_influence.boldness > 0.6 { "boldness" }
                    else if deliberation.chemistry_influence.caution > 0.6 { "caution" }
                    else if deliberation.chemistry_influence.empathy > 0.6 { "empathy" }
                    else { "balanced" };
                let _ = tx.send(serde_json::json!({
                    "type": "deliberation_resolved",
                    "chosen": deliberation.options[deliberation.chosen].description,
                    "confidence": deliberation.confidence,
                    "reasoning": deliberation.reasoning,
                    "ego_chose": ego_chose,
                    "chemistry_dominant": chem_dominant,
                }).to_string());
            }

            // Injecter le resultat dans le prompt
            ctx.prompt.push_str(&format!(
                "\n\nDELIBERATION INTERNE :\n{}\nIntegre cette decision dans ta reflexion.",
                deliberation.reasoning
            ));

            // Stocker pour la trace cognitive
            ctx.deliberation = Some(deliberation);
        }
    }

    // =========================================================================
    // Phase 19 : Appel LLM
    // =========================================================================

    /// Envoie le prompt au LLM et recupere la reponse.
    /// Positionne `should_abort` si le LLM echoue.
    /// Piste 2 : system_prompt (statique, cache) + prompt (dynamique) en split.
    pub(super) async fn phase_call_llm(&mut self, ctx: &mut ThinkingContext) {
        let llm_config = self.config.llm.clone();
        let start = Instant::now();

        let backend = llm::create_backend(&llm_config);
        // Si les pensees recentes ont ete purgees (stagnation), augmenter la temperature
        // pour perturber le pattern du modele et encourager la diversite.
        let temp = if self.thought_engine.recent_thoughts().is_empty() && self.cycle_count > 3 {
            (llm_config.temperature + 0.35).min(1.2)
        } else {
            llm_config.temperature
        };
        let max_tokens = llm_config.max_tokens_thought;
        let system_prompt = ctx.system_prompt.clone();
        // Prefixer /no_think pour les pensees autonomes si modele Qwen
        let user_message = llm::prepare_autonomous_message(&ctx.prompt, &llm_config.model);

        let resp = tokio::task::spawn_blocking(move || {
            backend.chat(&system_prompt, &user_message, temp, max_tokens)
        }).await;

        ctx.llm_elapsed = start.elapsed().as_secs_f64();
        self.avg_response_time = self.avg_response_time * 0.9 + ctx.llm_elapsed * 0.1;
        self.llm_busy.store(false, Ordering::Relaxed);

        match resp {
            Ok(Ok(thought_text)) => {
                ctx.thought_text = thought_text;

                // Retry si pensee vide (< 10 chars) : 1 tentative avec temperature + 0.1
                if ctx.thought_text.trim().len() < 10 {
                    tracing::debug!("Pensee vide detectee ({}c), retry avec temp+0.1", ctx.thought_text.len());
                    let retry_config = self.config.llm.clone();
                    let retry_backend = llm::create_backend(&retry_config);
                    let retry_temp = (retry_config.temperature + 0.1).min(1.5);
                    let retry_max = retry_config.max_tokens_thought;
                    let retry_sys = ctx.system_prompt.clone();
                    let retry_user = llm::prepare_autonomous_message(&ctx.prompt, &retry_config.model);

                    if let Ok(Ok(retry_text)) = tokio::task::spawn_blocking(move || {
                        retry_backend.chat(&retry_sys, &retry_user, retry_temp, retry_max)
                    }).await {
                        if retry_text.trim().len() >= 10 {
                            ctx.thought_text = retry_text;
                        } else {
                            // Fallback : marqueur de silence
                            ctx.thought_text = format!("[silence cycle {}]", self.cycle_count);
                        }
                    }
                }

                // Post-processing : retirer la trace chimique C[...] E:... V+... A...
                // Le LLM la genere mais elle ne doit pas polluer le contenu stocke/affiche
                ctx.thought_text = strip_chemical_trace(&ctx.thought_text);

                // Post-processing : appropriation en premiere personne
                if self.config.thought_ownership.enabled && self.config.thought_ownership.post_processing_enabled {
                    ctx.thought_text = crate::psychology::ownership::ensure_first_person(&ctx.thought_text);
                }
            }
            Ok(Err(e)) => {
                tracing::warn!("LLM erreur: {}", e);
                ctx.should_abort = true;
            }
            Err(e) => {
                tracing::warn!("LLM spawn_blocking erreur: {}", e);
                ctx.should_abort = true;
            }
        }
    }

    /// Tente une recherche web pour enrichir la pensee autonome.
    ///
    /// Le sujet de recherche est choisi par pick_exploration_topic()
    /// avec anti-repetition renforcee et rotation des sources.
    async fn try_web_search(
        &mut self,
        thought_type: &crate::agent::thought_engine::ThoughtType,
        current_emotion: &str,
    ) -> Option<(String, crate::knowledge::KnowledgeResult)> {
        let interests = &self.config.saphire.interests.initial_topics;
        let recent = self.thought_engine.recent_thoughts().to_vec();

        let (topic, source) = self.knowledge.pick_exploration_topic(
            interests,
            &recent,
            current_emotion,
            self.cycle_count,
        )?;

        tracing::info!("WebKnowledge: recherche '{}' (source: {}, type: {}, emotion: {})",
            topic, source, thought_type.as_str(), current_emotion);

        let config = self.config.knowledge.clone();
        let topic_clone = topic.clone();
        let source_clone = source.clone();
        let read_counts = self.knowledge.article_read_count.clone();

        let result = tokio::task::spawn_blocking(move || {
            let mut wk = crate::knowledge::WebKnowledge::new(config);
            wk.article_read_count = read_counts;
            wk.search(&topic_clone, &source_clone)
        }).await;

        match result {
            Ok(Ok(kr)) => {
                let context_text = format!(
                    "Source: {} | Titre: {}\n{}",
                    kr.source, kr.title, truncate_utf8(&kr.extract, 1500)
                );
                tracing::info!("WebKnowledge: trouve '{}' de {} ({} chars, {} sections)",
                    kr.title, kr.source, kr.extract.len(), kr.section_titles.len());
                self.knowledge.record_source(&kr.source);
                self.knowledge.increment_article_read_count(&kr.title);
                self.knowledge.topics_explored.push(topic);
                Some((context_text, kr))
            },
            Ok(Err(e)) => {
                tracing::warn!("WebKnowledge: recherche echouee pour '{}' sur {}: {}",
                    topic, source, e);
                None
            },
            Err(e) => {
                tracing::debug!("WebKnowledge: erreur tache: {}", e);
                None
            },
        }
    }
}
