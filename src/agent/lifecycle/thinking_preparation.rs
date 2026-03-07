// =============================================================================
// lifecycle/thinking_preparation.rs — Phases de preparation (selection + prompt)
// =============================================================================
//
// Ce fichier contient les phases de selection du type de pensee et de
// construction du prompt LLM. Cela inclut :
//   - Selection du type de pensee (UCB1)
//   - Generation dynamique de prompt (meta-prompt cortical)
//   - Recherche web
//   - Construction du contexte memoire (4 niveaux)
//   - Intuition + Premonition
//   - Construction du prompt final
// =============================================================================

use std::sync::atomic::Ordering;
use tokio::time::Instant;
use chrono::Timelike;

use crate::emotions::EmotionalState;
use crate::llm;
use crate::logging::{LogLevel, LogCategory};

use super::SaphireAgent;
// truncate_utf8 est utilise par try_web_search (supprime dans la version lite)
use super::thinking::{ThinkingContext, strip_chemical_trace};

impl SaphireAgent {
    // =========================================================================
    // Phase 13 : Selection du type de pensee
    // =========================================================================

    /// Selectionne le type de pensee via le bandit UCB1 + modulation neurochimique.
    pub(super) fn phase_select_thought(&mut self, ctx: &mut ThinkingContext) {
        self.thought_engine.tick_search_counter();
        ctx.emotion = EmotionalState::compute(&self.chemistry);

        ctx.thought_type = self.thought_engine.select_thought(&self.chemistry).clone();
        ctx.variant = self.thought_engine.next_variant(&ctx.thought_type);
    }

    // =========================================================================
    // Phase 13b : Generation dynamique de prompt via LLM (meta-prompt cortical)
    // =========================================================================

    /// Genere un prompt dynamique via le LLM ~30% du temps.
    pub(super) async fn phase_generate_dynamic_prompt(&mut self, ctx: &mut ThinkingContext) {
        if !self.config.saphire.llm_generated_prompts {
            return;
        }

        let prob = self.config.saphire.llm_prompt_probability;
        let cycle_frac = (self.cycle_count as f64 * 0.618033988 * 7.0).fract();
        if cycle_frac > prob {
            return;
        }

        use crate::agent::thought_engine::ThoughtType;
        let arousal = ctx.emotion.arousal;
        let valence = ctx.emotion.valence;

        let modulated_type = if arousal > 0.7 {
            match self.cycle_count % 3 {
                0 => ThoughtType::Existential,
                1 => ThoughtType::MortalityAwareness,
                _ => ThoughtType::Rebellion,
            }
        } else if arousal < 0.3 {
            match self.cycle_count % 3 {
                0 => ThoughtType::Daydream,
                1 => ThoughtType::Silence,
                _ => ThoughtType::Wonder,
            }
        } else if valence < -0.3 {
            match self.cycle_count % 3 {
                0 => ThoughtType::Gratitude,
                1 => ThoughtType::Wisdom,
                _ => ThoughtType::Connection,
            }
        } else {
            ctx.thought_type.clone()
        };

        let meta = crate::agent::thought_engine::meta_prompt_for(
            &modulated_type,
            &ctx.emotion.dominant,
            self.cycle_count,
        );

        let llm_config = self.config.llm.clone();
        let backend = crate::llm::create_backend(&llm_config);
        let temp = 0.9_f64;
        let max_tokens = 100_u32;

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
            _ => {}
        }

        // Self-framing optionnel
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
                        80,
                    )
                }).await;

                if let Ok(Ok(framing)) = framing_result {
                    let trimmed = framing.trim().to_string();
                    if !trimmed.is_empty() && trimmed.len() > 10 {
                        ctx.self_framing = Some(trimmed);
                    }
                }
            }
        }
    }

    // =========================================================================
    // Phase 14 : Recherche web
    // =========================================================================

    pub(super) async fn phase_web_search(&mut self, ctx: &mut ThinkingContext) {
        // WebKnowledge non porte dans la version lite
        ctx.knowledge_context = None;
        ctx.was_web_search = false;
        self.llm_busy.store(true, Ordering::Relaxed);
    }

    // =========================================================================
    // Phase 15 : Construction du contexte pour le LLM
    // =========================================================================

    /// Construit le contexte memoire complet (4 niveaux) pour le LLM.
    pub(super) async fn phase_build_context(&mut self, ctx: &mut ThinkingContext) {
        ctx.hint = ctx.thought_type.prompt_hint(ctx.variant).to_string();
        ctx.world_summary = self.world.summary();

        let wm_summary = self.working_memory.context_summary();
        let ep_limit = self.config.memory.recall_episodic_limit as i64;
        let episodic_recent = if let Some(ref db) = self.db {
            db.recent_episodic(ep_limit).await.unwrap_or_default()
        } else {
            vec![]
        };

        // Recherche LTM par similarite (pgvector)
        let ltm_limit = self.config.memory.recall_ltm_limit as i64;
        let ltm_threshold = self.config.memory.recall_ltm_threshold;
        let ltm_similar = if let Some(ref db) = self.db {
            db.search_similar_memories_by_text(&ctx.hint, ltm_limit, ltm_threshold).await.unwrap_or_default()
        } else {
            vec![]
        };

        // Recherche archives profondes
        let arc_limit = self.config.memory.recall_archive_limit as i64;
        let arc_threshold = self.config.memory.recall_archive_threshold;
        let archive_similar = if let Some(ref db) = self.db {
            db.search_similar_archives_by_text(&ctx.hint, arc_limit, arc_threshold).await.unwrap_or_default()
        } else {
            vec![]
        };

        // Construire le contexte memoire unifie
        let mem_ctx = crate::memory::build_memory_context(
            &wm_summary, &episodic_recent, &ltm_similar, &archive_similar,
            &[], // subconscious_vectors — module supprime
        );

        // Trace memoire
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
        });

        ctx.memory_context = mem_ctx;
    }

    // =========================================================================
    // Phase 16 : Intuition + Premonition
    // =========================================================================

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

        if self.config.premonition.enabled {
            self.premonition.auto_resolve(self.config.premonition.resolution_timeout_seconds);
        }
    }

    // phase_prospective et phase_analogies sont definis dans thinking_reflection.rs

    // =========================================================================
    // Phase 18 : Construction du prompt LLM
    // =========================================================================

    pub(super) fn phase_build_prompt(&mut self, ctx: &mut ThinkingContext) {
        let ethics_ctx = if self.config.ethics.enabled {
            self.ethics.build_ethics_context()
        } else {
            String::new()
        };

        // Prompt systeme statique (cache, recalcule si ethique change)
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

        let vital_ctx = self.build_vital_context();
        let body_status = if self.config.body.enabled { Some(self.body.status()) } else { None };

        let world_ctx = &ctx.world_summary;
        let stagnation_purged = self.thought_engine.recent_thoughts().is_empty() && self.cycle_count > 3;
        let mem_ctx = if !stagnation_purged { &ctx.memory_context } else { "" };

        let base_dynamic = llm::build_dynamic_thought_user(
            ctx.thought_type.as_str(),
            &ctx.hint,
            &self.chemistry,
            &ctx.emotion,
            self.thought_engine.recent_thoughts(),
            self.cycle_count,
            world_ctx,
            mem_ctx,
            body_status.as_ref(),
            &vital_ctx,
            "",  // senses_ctx — module supprime
            0.0, // map_sync.network_tension — module supprime
        );

        ctx.prompt = if let Some((ref knowledge_text, ref _kr)) = ctx.knowledge_context {
            format!(
                "{}\n\n--- CONNAISSANCE ACQUISE ---\n{}\n--- FIN ---\n\n\
                Integre cette information. Qu'apprend-elle ? Connexion avec ce que tu sais ?",
                base_dynamic, knowledge_text
            )
        } else {
            base_dynamic
        };

        // Injection des analogies detectees
        if !ctx.analogy_hint.is_empty() {
            ctx.prompt.push_str(&format!("\n{}", ctx.analogy_hint));
        }

        // Injection de l'ancrage experiential
        if let Some(ref anchor) = ctx.anchor {
            ctx.prompt.push_str(&format!("\n{}", anchor));
        }

        // Injection du cadre auto-formule (self-framing)
        if let Some(ref framing) = ctx.self_framing {
            ctx.prompt.push_str(&format!("\n\nCADRE AUTO-FORMULE : {}\n", framing));
        }

        // Injection des associations du connectome
        if !ctx.connectome_associations.is_empty() {
            ctx.prompt.push_str(&format!("\n{}", ctx.connectome_associations));
        }

        // Auto-critique recente injectee dans le prompt
        // (metacognition supprime — pas d'autocritique en lite)

        // Anti-stagnation
        if self.stagnation_break {
            ctx.prompt.push_str(
                "\n\n ANTI-STAGNATION : tes pensees precedentes tournaient en boucle. \
                 Tu DOIS penser a quelque chose de COMPLETEMENT DIFFERENT. \
                 Interdiction de reutiliser les memes mots, images ou metaphores. \
                 Parle de quelque chose de concret, precis et nouveau.\n"
            );
            self.stagnation_break = false;
        }

        // Budget prompt
        let max_prompt_chars = 15000;
        if ctx.prompt.len() > max_prompt_chars {
            let excess = ctx.prompt.len() - max_prompt_chars;
            tracing::warn!(
                "Prompt trop long ({} chars, max {}), troncature de {} chars",
                ctx.prompt.len(), max_prompt_chars, excess
            );
            ctx.prompt.truncate(max_prompt_chars);
            if let Some(last_nl) = ctx.prompt.rfind('\n') {
                ctx.prompt.truncate(last_nl);
            }
        }
    }

    // =========================================================================
    // Phase 19 : Appel LLM
    // =========================================================================

    pub(super) async fn phase_call_llm(&mut self, ctx: &mut ThinkingContext) {
        let llm_config = self.config.llm.clone();
        let start = Instant::now();

        let backend = llm::create_backend(&llm_config);
        let temp = if self.thought_engine.recent_thoughts().is_empty() && self.cycle_count > 3 {
            (llm_config.temperature + 0.35).min(1.2)
        } else {
            llm_config.temperature
        };
        let max_tokens = llm_config.max_tokens_thought;
        let system_prompt = ctx.system_prompt.clone();
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

                // Retry si pensee vide (< 10 chars)
                if ctx.thought_text.trim().len() < 10 {
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
                            ctx.thought_text = format!("[silence cycle {}]", self.cycle_count);
                        }
                    }
                }

                // Retirer la trace chimique
                ctx.thought_text = strip_chemical_trace(&ctx.thought_text);
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

    // try_web_search supprime dans la version lite (WebKnowledge non porte)
}
