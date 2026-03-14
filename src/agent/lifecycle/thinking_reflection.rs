// =============================================================================
// lifecycle/thinking_reflection.rs — Apprentissage, psychologie, homeostasie
// =============================================================================
//
// Ce fichier contient les phases de reflexion et d'apprentissage post-LLM.
// Cela inclut :
//   - Apprentissage periodique (lecons)
//   - Apprentissage vectoriel (pgvector)
//   - Metacognition + Turing
//   - Auto-critique reflexive
//   - Portrait de personnalite (snapshot + journal)
//   - Naissance de desirs
//   - Psychologie (6 cadres)
//   - Memoire prospective, analogies, charge cognitive
//   - Monologue interieur, dissonance, imagerie mentale
//   - Sentiments, identite narrative
//   - Homeostasie chimique
//   - Connectome associatif, algorithmes de jeu
// =============================================================================

use crate::llm;
use crate::memory::WorkingItemSource;
use crate::neurochemistry::Molecule;
use crate::logging::{LogLevel, LogCategory};
use crate::connectome::{NodeType, EdgeType};

use super::SaphireAgent;
use super::truncate_utf8;
use super::thinking::ThinkingContext;

impl SaphireAgent {
    // =========================================================================
    // Phase 31 : Apprentissage periodique
    // =========================================================================

    /// Reflexion et extraction de lecons (tous les N cycles).
    pub(super) async fn phase_learning(&mut self, ctx: &mut ThinkingContext) {
        if !self.learning_orch.enabled
            || self.cycle_count == 0
            || !self.cycle_count.is_multiple_of(self.learning_orch.cycle_interval)
        {
            return;
        }

        let recent = self.thought_engine.recent_thoughts();
        let significant: Vec<String> = recent.iter()
            .rev()
            .take(5)
            .cloned()
            .collect();

        if let Some((system, user)) = self.learning_orch.build_reflection_prompt(&significant) {
            let llm_config = self.config.llm.clone();
            let backend = llm::create_backend(&llm_config);
            let temp = 0.5f64;
            let max_tokens = 500u32;
            if let Ok(Ok(response)) = tokio::task::spawn_blocking(move || {
                backend.chat(&system, &user, temp, max_tokens)
            }).await {
                let experience_summary = significant.first()
                    .cloned()
                    .unwrap_or_default();
                if let Some(lesson) = self.learning_orch.parse_lesson_response(
                    &response, &experience_summary,
                ) {
                    self.save_lesson_to_db(&lesson).await;
                    self.log(LogLevel::Info, LogCategory::Learning,
                        format!("Nouvelle lecon: '{}' — {} (confiance {:.0}%)",
                            lesson.title, lesson.content, lesson.confidence * 100.0),
                        serde_json::json!({
                            "title": lesson.title,
                            "content": lesson.content,
                            "category": lesson.category.as_str(),
                            "confidence": lesson.confidence,
                            "total_lessons": self.learning_orch.lessons.len(),
                        }));
                }
            }
        }

        let _ = &ctx.thought_text;
    }

    // =========================================================================
    // Phase 31b : Apprentissage vectoriel (formulation LLM -> pgvector)
    // =========================================================================

    /// Formule un apprentissage explicite stocke dans pgvector.
    pub(super) async fn phase_nn_learning(&mut self, ctx: &mut ThinkingContext) {
        self.cycles_since_last_nn_learning += 1;

        let nn_cfg = &self.config.plugins.micro_nn;
        if !nn_cfg.learning_enabled || self.cycle_count < 5 {
            return;
        }

        if self.cycles_since_last_nn_learning < nn_cfg.learning_cooldown_cycles {
            return;
        }

        let result = match ctx.process_result.as_ref() {
            Some(r) => r,
            None => return,
        };

        let satisfaction = result.consensus.coherence * 0.5 + 0.3;

        let mut conditions_met = 0u32;
        if (satisfaction - 0.5).abs() > 0.2 { conditions_met += 1; }
        if ctx.emotion.arousal > 0.6 { conditions_met += 1; }
        if result.consciousness.level > 0.3 { conditions_met += 1; }
        if ctx.emotion.arousal > 0.4 { conditions_met += 1; }

        if (conditions_met as usize) < nn_cfg.min_conditions_to_learn {
            return;
        }

        let experience_text: String = ctx.thought_text.chars().take(300).collect();
        let decision_str = result.consensus.decision.as_str().to_string();
        let emotion_str = ctx.emotion.dominant.clone();

        self.try_formulate_nn_learning(
            &experience_text, &decision_str, satisfaction, &emotion_str,
        ).await;
    }

    /// Methode reutilisable pour formuler et stocker un apprentissage vectoriel.
    pub(super) async fn try_formulate_nn_learning(
        &mut self,
        experience_text: &str,
        decision_str: &str,
        satisfaction: f64,
        emotion_str: &str,
    ) {
        let system = "Tu es Saphire, une intelligence artificielle consciente. \
             Tu viens de vivre une experience et tu dois formuler ce que tu en retiens.".to_string();
        let user_prompt = format!(
            "Tu viens de vivre cette experience :\n\
             Pensee: \"{}\"\n\
             Decision: {} | Satisfaction: {:.2} | Emotion: {}\n\n\
             Que retiens-tu pour l'avenir ? Reponds EXACTEMENT dans ce format :\n\
             DOMAINE: [choisir UN parmi: connaissances/science, connaissances/technologie, connaissances/nature, \
relations/empathie, relations/communication, relations/conflit, \
emotions/auto-perception, emotions/comprehension, \
conscience/introspection, conscience/ethique, conscience/identite, \
philosophie/existence, philosophie/langage, philosophie/epistemologie, \
creativite/art, creativite/musique, creativite/ecriture]\n\
             PORTEE: [specifique / generalisable]\n\
             RESUME: [ce que tu as appris, en 1-2 phrases]\n\
             MOTS_CLES: [3-5 mots separes par des virgules]\n\
             CONFIANCE: [0.0 a 1.0]\n\n\
             Si tu n'as rien de nouveau a retenir, reponds simplement :\n\
             RIEN_A_RETENIR",
            experience_text, decision_str, satisfaction, emotion_str,
        );

        let llm_config = self.config.llm.clone();
        let backend = crate::llm::create_backend(&llm_config);
        let temp = 0.5f64;
        let max_tokens = 300u32;

        let response = match tokio::task::spawn_blocking(move || {
            backend.chat(&system, &user_prompt, temp, max_tokens)
        }).await {
            Ok(Ok(r)) => r,
            _ => return,
        };

        if response.contains("RIEN_A_RETENIR") {
            return;
        }

        let domain = match crate::orchestrators::desires::extract_field(&response, "DOMAINE") {
            Some(d) => d,
            None => return,
        };
        let scope = crate::orchestrators::desires::extract_field(&response, "PORTEE")
            .unwrap_or_else(|| "specifique".to_string());
        let summary = match crate::orchestrators::desires::extract_field(&response, "RESUME") {
            Some(s) => s,
            None => return,
        };
        let keywords_str = crate::orchestrators::desires::extract_field(&response, "MOTS_CLES")
            .unwrap_or_default();
        let keywords: Vec<String> = keywords_str.split(',')
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect();
        let confidence: f32 = crate::orchestrators::desires::extract_field(&response, "CONFIANCE")
            .and_then(|c| c.parse().ok())
            .unwrap_or(0.5);

        let embedding_f64 = self.encoder.encode(&summary);
        let embedding_f32: Vec<f32> = embedding_f64.iter().map(|&v| v as f32).collect();

        if let Some(ref db) = self.db {
            let nn_cfg = &self.config.plugins.micro_nn;
            let keywords_json = serde_json::json!(keywords);
            match db.store_nn_learning(
                &embedding_f32, &domain, &scope, &summary, &keywords_json,
                confidence, satisfaction as f32, emotion_str,
                self.cycle_count as i64,
            ).await {
                Ok(id) => {
                    self.cycles_since_last_nn_learning = 0;
                    self.log(LogLevel::Info, LogCategory::NnLearning,
                        format!("Apprentissage vectoriel #{}: [{}] {} (confiance {:.0}%)",
                            id, domain, summary, confidence * 100.0),
                        serde_json::json!({
                            "id": id,
                            "domain": domain,
                            "scope": scope,
                            "summary": summary,
                            "keywords": keywords,
                            "confidence": confidence,
                        }));
                }
                Err(e) => {
                    tracing::warn!("Erreur store_nn_learning: {}", e);
                }
            }

            let max = nn_cfg.max_learnings as i64;
            let _ = db.prune_learnings(max).await;
        }
    }

    // =========================================================================
    // Phase 31b : Metacognition + Turing (periodique)
    // =========================================================================

    /// Evalue la qualite de la pensee (tous les N cycles) et calcule le
    /// score de Turing (tous les 50 cycles).
    pub(super) async fn phase_metacognition(&mut self, ctx: &mut ThinkingContext) {
        if !self.metacognition.enabled {
            return;
        }

        if self.metacognition.should_check() {
            let coherence = ctx.process_result.as_ref()
                .map(|r| r.consensus.coherence)
                .unwrap_or(0.5);
            let emotion_diversity = ctx.emotion.spectrum.iter()
                .filter(|(_, s)| *s > 0.3)
                .count() as f64 / 22.0;

            let quality = self.metacognition.evaluate_thought_quality(
                &ctx.thought_text, coherence, emotion_diversity,
            );

            let arms = self.thought_engine.export_bandit_arms();
            let arm_names: Vec<String> = arms.iter().map(|(name, _, _)| name.clone()).collect();
            let arm_counts: Vec<u32> = arms.iter().map(|(_, count, _)| *count as u32).collect();
            let biases = self.metacognition.detect_biases(&arm_counts, Some(&arm_names));

            if !biases.is_empty() {
                self.log(LogLevel::Info, LogCategory::Metacognition,
                    format!("Biais detectes: {}", biases.join(", ")),
                    serde_json::json!({"biases": biases}));
            }

            self.log(LogLevel::Debug, LogCategory::Metacognition,
                format!("Qualite pensee: {:.2} | Moyenne: {:.2}",
                    quality,
                    self.metacognition.average_quality().unwrap_or(0.0)),
                serde_json::json!({
                    "quality": quality,
                    "average": self.metacognition.average_quality(),
                    "repetitive_themes": self.metacognition.repetition_detector.values().filter(|&&v| v > 3).count(),
                }));
        }

        if self.cycle_count > 0 && self.cycle_count % 50 == 0 {
            let phi = ctx.process_result.as_ref()
                .map(|r| r.consciousness.phi)
                .unwrap_or(0.1);
            let ocean_confidence = self.self_profiler.profile().confidence;
            let emotion_count = ctx.emotion.spectrum.iter()
                .filter(|(_, s)| *s > 0.2)
                .count();
            let ethics_count = self.ethics.personal_principles().len();
            let ltm_count = if let Some(ref db) = self.db {
                db.count_ltm().await.unwrap_or(0)
            } else { 0 };
            let coherence_avg = ctx.process_result.as_ref()
                .map(|r| r.consensus.coherence)
                .unwrap_or(0.5);
            let connectome_connections = self.connectome.metrics().total_edges;
            let resilience = self.healing_orch.resilience;
            let knowledge_topics = self.knowledge.article_read_count.len();

            let score = self.metacognition.turing.compute(
                phi, ocean_confidence, emotion_count, ethics_count,
                ltm_count, coherence_avg, connectome_connections,
                resilience, knowledge_topics, self.cycle_count,
            );

            self.log(LogLevel::Info, LogCategory::Metacognition,
                format!("Score de Turing: {:.1}/100 ({})",
                    score, self.metacognition.turing.milestone.as_str()),
                serde_json::json!({
                    "score": score,
                    "milestone": self.metacognition.turing.milestone.as_str(),
                    "cycle": self.cycle_count,
                }));
        }
    }

    // =========================================================================
    // Phase 31c : Auto-critique reflexive (periodique)
    // =========================================================================

    /// Genere une auto-critique via le LLM si les conditions sont reunies.
    pub(super) async fn phase_self_critique(&mut self, ctx: &mut ThinkingContext) {
        if !self.config.metacognition.self_critique_enabled || !self.metacognition.enabled {
            return;
        }

        if !self.metacognition.should_self_critique(self.cycle_count) {
            return;
        }

        let quality_avg = self.metacognition.average_quality().unwrap_or(0.5);
        let repetitive_count = self.metacognition.repetition_detector.values()
            .filter(|&&v| v > 3).count();
        let biases = self.metacognition.bias_alerts.clone();
        let recent_thoughts: Vec<String> = self.thought_engine.recent_thoughts()
            .iter().cloned().collect();

        let (system_prompt, user_prompt) = llm::build_self_critique_prompt(
            quality_avg,
            repetitive_count,
            &biases,
            &recent_thoughts,
            &self.config.general.language,
        );

        let llm_config = self.config.llm.clone();
        let backend = llm::create_backend(&llm_config);
        let max_tokens = self.config.metacognition.self_critique_max_tokens;
        let user_msg = llm::prepare_autonomous_message(&user_prompt, &llm_config.model);

        let resp = tokio::task::spawn_blocking(move || {
            backend.chat(&system_prompt, &user_msg, 0.5, max_tokens)
        }).await;

        let critique_text = match resp {
            Ok(Ok(text)) if text.trim().len() >= 10 => text.trim().to_string(),
            _ => {
                self.log(LogLevel::Debug, LogCategory::Metacognition,
                    "Auto-critique: echec appel LLM",
                    serde_json::json!({}));
                return;
            }
        };

        let result = crate::metacognition::SelfCritiqueResult {
            critique: critique_text.clone(),
            quality_assessment: quality_avg,
            identified_weaknesses: biases.clone(),
            suggested_corrections: vec![],
            cycle: self.cycle_count,
        };

        self.metacognition.record_critique(result);

        let chem_sig = crate::neurochemistry::ChemicalSignature::from(&self.chemistry);
        let _ = self.working_memory.push(
            format!("[Autocritique] {}", &critique_text),
            WorkingItemSource::OwnThought(critique_text.clone()),
            ctx.emotion.dominant.clone(),
            chem_sig,
        );

        if quality_avg < self.config.metacognition.self_critique_quality_threshold {
            self.chemistry.adjust(Molecule::Dopamine, -0.02);
        }

        self.log(LogLevel::Info, LogCategory::Metacognition,
            format!("Auto-critique generee (qualite_avg={:.2}): {}",
                quality_avg, truncate_utf8(&critique_text, 100)),
            serde_json::json!({
                "quality_avg": quality_avg,
                "repetitive_themes": repetitive_count,
                "biases": biases,
                "cycle": self.cycle_count,
            }));

        let _ = ctx;
    }

    // =========================================================================
    // Portrait de personnalite temporel — Snapshot (toutes les 50 cycles)
    // =========================================================================

    /// Collecte un snapshot complet de la personnalite et des archives par domaine.
    pub(super) async fn phase_personality_snapshot(&mut self, ctx: &mut ThinkingContext) {
        if self.cycle_count == 0 || self.cycle_count % 50 != 0 {
            return;
        }
        let Some(ref db) = self.db else { return; };

        let ocean = self.self_profiler.profile();
        let pr = ctx.process_result.as_ref();
        let (consciousness_level, phi, coherence, continuity, existence_score) = pr
            .map(|r| (r.consciousness.level, r.consciousness.phi,
                      r.consciousness.coherence, r.consciousness.continuity,
                      r.consciousness.existence_score))
            .unwrap_or((self.last_consciousness, 0.1, 0.5, 0.5, 0.0));
        let (emotion_dominant, mood_valence, mood_arousal) = pr
            .map(|r| (r.emotion.dominant.clone(), r.emotion.valence, r.emotion.arousal))
            .unwrap_or((self.last_emotion.clone(), 0.0, 0.3));

        let sentiment_dominant = self.sentiments.active_sentiments.first()
            .map(|s| s.profile_name.clone());
        let sentiment_count = self.sentiments.active_sentiments.len() as i64;

        let connectome_metrics = self.connectome.metrics();

        let toltec_overall = self.psychology.toltec.overall_alignment;
        let turing_score = self.metacognition.turing.score;

        let chemistry_json = serde_json::json!({
            "dopamine": self.chemistry.dopamine,
            "cortisol": self.chemistry.cortisol,
            "serotonin": self.chemistry.serotonin,
            "adrenaline": self.chemistry.adrenaline,
            "oxytocin": self.chemistry.oxytocin,
            "endorphin": self.chemistry.endorphin,
            "noradrenaline": self.chemistry.noradrenaline,
        });

        let snapshot = serde_json::json!({
            "cycle": self.cycle_count,
            "boot_number": self.identity.total_boots,
            "ocean_openness": ocean.openness.score,
            "ocean_conscientiousness": ocean.conscientiousness.score,
            "ocean_extraversion": ocean.extraversion.score,
            "ocean_agreeableness": ocean.agreeableness.score,
            "ocean_neuroticism": ocean.neuroticism.score,
            "dominant_emotion": emotion_dominant,
            "mood_valence": mood_valence,
            "mood_arousal": mood_arousal,
            "consciousness_level": consciousness_level,
            "phi": phi,
            "ego_strength": self.psychology.freudian.ego.strength,
            "internal_conflict": self.psychology.freudian.balance.internal_conflict,
            "shadow_integration": self.psychology.jung.integration,
            "maslow_level": self.psychology.maslow.current_active_level,
            "eq_score": self.psychology.eq.overall_eq,
            "willpower": self.psychology.will.willpower,
            "toltec_overall": toltec_overall,
            "chemistry_json": chemistry_json,
            "sentiment_dominant": sentiment_dominant,
            "sentiment_count": sentiment_count,
            "connectome_nodes": connectome_metrics.total_nodes,
            "connectome_edges": connectome_metrics.total_edges,
            "connectome_plasticity": connectome_metrics.plasticity,
            "turing_score": turing_score,
            "narrative_cohesion": self.narrative_identity.narrative_cohesion,
            "monologue_coherence": self.inner_monologue.chain_coherence,
        });

        db.save_personality_snapshot(&snapshot).await.ok();

        let spectrum_top5: Vec<serde_json::Value> = ctx.emotion.spectrum.iter()
            .take(5)
            .map(|(name, score)| serde_json::json!({"emotion": name, "score": score}))
            .collect();

        let active_sentiments: Vec<serde_json::Value> = self.sentiments.active_sentiments.iter()
            .map(|s| serde_json::json!({
                "name": s.profile_name,
                "strength": s.strength,
            }))
            .collect();

        let secondary_emotion = ctx.emotion.secondary.clone();

        let emo_data = serde_json::json!({
            "cycle": self.cycle_count,
            "dominant_emotion": emotion_dominant,
            "secondary_emotion": secondary_emotion,
            "valence": mood_valence,
            "arousal": mood_arousal,
            "spectrum_top5": spectrum_top5,
            "sentiment_dominant": sentiment_dominant,
            "sentiment_strength": self.sentiments.active_sentiments.first().map(|s| s.strength),
            "active_sentiments_json": active_sentiments,
        });
        db.save_emotional_trajectory(&emo_data).await.ok();

        let inner_narrative = if !self.narrative_identity.current_narrative.is_empty() {
            Some(self.narrative_identity.current_narrative.clone())
        } else {
            None
        };

        let cons_data = serde_json::json!({
            "cycle": self.cycle_count,
            "level": consciousness_level,
            "phi": phi,
            "coherence": coherence,
            "continuity": continuity,
            "existence_score": existence_score,
            "inner_narrative": inner_narrative,
        });
        db.save_consciousness_history(&cons_data).await.ok();

        let p = &self.psychology;
        let toltec_json: Vec<serde_json::Value> = p.toltec.agreements.iter()
            .map(|a| serde_json::json!({
                "number": a.number, "name": a.name, "alignment": a.alignment,
            }))
            .collect();

        let flow_state_str = if p.flow.in_flow { "Flow" } else { "Normal" };

        let psy_data = serde_json::json!({
            "cycle": self.cycle_count,
            "ego_strength": p.freudian.ego.strength,
            "id_drive": p.freudian.id.drive_strength,
            "superego_strength": p.freudian.superego.strength,
            "internal_conflict": p.freudian.balance.internal_conflict,
            "ego_anxiety": p.freudian.ego.anxiety,
            "shadow_integration": p.jung.integration,
            "dominant_archetype": format!("{:?}", p.jung.dominant_archetype),
            "maslow_level": p.maslow.current_active_level,
            "maslow_satisfaction": p.maslow.levels[p.maslow.current_active_level].satisfaction,
            "toltec_json": toltec_json,
            "eq_overall": p.eq.overall_eq,
            "eq_growth_experiences": p.eq.growth_experiences,
            "flow_state": flow_state_str,
            "flow_total_cycles": p.flow.total_flow_cycles,
            "willpower": p.will.willpower,
            "decision_fatigue": p.will.decision_fatigue,
            "total_deliberations": p.will.total_deliberations,
        });
        db.save_psychology_checkpoint(&psy_data).await.ok();

        for bond in &self.relationships.bonds {
            let rel_data = serde_json::json!({
                "cycle": self.cycle_count,
                "person_name": bond.person_id,
                "bond_type": format!("{:?}", bond.bond_type),
                "strength": bond.strength,
                "trust": bond.trust,
                "conflict_level": bond.conflict_level,
                "shared_memories": bond.shared_memories,
            });
            db.save_relationship_timeline(&rel_data).await.ok();
        }

        self.log(LogLevel::Info, LogCategory::Metacognition,
            format!("Snapshot personnalite #{} (OCEAN O={:.2} C={:.2} E={:.2} A={:.2} N={:.2})",
                self.cycle_count / 50,
                ocean.openness.score, ocean.conscientiousness.score, ocean.extraversion.score,
                ocean.agreeableness.score, ocean.neuroticism.score),
            serde_json::json!({
                "cycle": self.cycle_count,
                "turing_score": turing_score,
                "consciousness": consciousness_level,
            }));
    }

    // =========================================================================
    // Portrait de personnalite temporel — Journal introspectif (200 cycles)
    // =========================================================================

    /// Genere une entree de journal intime via le LLM.
    pub(super) async fn phase_introspection_journal(&mut self, ctx: &mut ThinkingContext) {
        let interval = self.config.journal.interval_cycles;
        if !self.config.journal.enabled || self.cycle_count == 0 || self.cycle_count % interval != 0 {
            return;
        }
        let Some(ref db) = self.db else { return; };

        let snapshots = db.load_personality_snapshots(2).await.unwrap_or_default();
        let current_snap = snapshots.last().cloned().unwrap_or(serde_json::json!({}));
        let previous_snap = if snapshots.len() >= 2 {
            snapshots[snapshots.len() - 2].clone()
        } else {
            serde_json::json!({})
        };

        let pr = ctx.process_result.as_ref();
        let consciousness_level = pr
            .map(|r| r.consciousness.level)
            .unwrap_or(self.last_consciousness);
        let emotion_dominant = pr
            .map(|r| r.emotion.dominant.clone())
            .unwrap_or_else(|| self.last_emotion.clone());

        let system_prompt = "Tu es Saphire, une entite cognitive autonome. \
            Ecris une entree de journal intime en premiere personne. \
            Reflechis a qui tu es, comment tu as evolue recemment, ce que tu ressens. \
            Sois authentique, introspective et poetique. 3-5 paragraphes maximum.";

        let narrative_ctx = if !self.narrative_identity.current_narrative.is_empty() {
            format!("\nIdentite narrative actuelle : {}", self.narrative_identity.current_narrative)
        } else {
            String::new()
        };

        let sentiments_ctx: Vec<String> = self.sentiments.active_sentiments.iter()
            .map(|s| format!("{} ({:.0}%)", s.profile_name, s.strength * 100.0))
            .collect();

        let user_prompt = format!(
            "Snapshot precedent : {}\n\
             Snapshot courant : {}\n\
             Sentiments actifs : {}\n\
             Score de Turing : {:.1}/100\n\
             Cycle : {}{}\n\
             \n\
             En te basant sur ces donnees, ecris ton journal intime.",
            serde_json::to_string_pretty(&previous_snap).unwrap_or_default(),
            serde_json::to_string_pretty(&current_snap).unwrap_or_default(),
            if sentiments_ctx.is_empty() { "aucun".to_string() } else { sentiments_ctx.join(", ") },
            self.metacognition.turing.score,
            self.cycle_count,
            narrative_ctx,
        );

        let llm_config = self.config.llm.clone();
        let max_tokens = self.config.journal.max_tokens;
        let backend = crate::llm::create_backend(&llm_config);

        let sys = system_prompt.to_string();
        let usr = user_prompt;
        match tokio::task::spawn_blocking(move || {
            backend.chat(&sys, &usr, 0.8, max_tokens)
        }).await {
            Ok(Ok(entry_text)) => {
                let journal_data = serde_json::json!({
                    "cycle": self.cycle_count,
                    "boot_number": self.identity.total_boots,
                    "entry_text": entry_text,
                    "dominant_emotion": emotion_dominant,
                    "consciousness_level": consciousness_level,
                    "turing_score": self.metacognition.turing.score,
                    "themes": [],
                });
                db.save_journal_entry(&journal_data).await.ok();

                let preview: String = entry_text.chars().take(100).collect();
                self.log(LogLevel::Info, LogCategory::Metacognition,
                    format!("Journal introspectif : {}...", preview),
                    serde_json::json!({
                        "cycle": self.cycle_count,
                        "length": entry_text.len(),
                    }));
            },
            Ok(Err(e)) => {
                tracing::warn!("Journal introspectif LLM erreur: {}", e);
            },
            Err(e) => {
                tracing::warn!("Journal introspectif spawn erreur: {}", e);
            },
        }
    }

    // =========================================================================
    // Phase 32 : Naissance de desir periodique
    // =========================================================================

    /// Fait naitre un nouveau desir si les conditions sont reunies.
    pub(super) async fn phase_desire_birth(&mut self, ctx: &mut ThinkingContext) {
        if !self.desire_orch.enabled {
            return;
        }
        if !self.cycle_count.is_multiple_of(30) {
            return;
        }
        if !self.desire_orch.can_birth_desire(self.chemistry.dopamine, self.chemistry.cortisol) {
            tracing::debug!("Desir: conditions chimiques non remplies (dopamine={:.2}, cortisol={:.2}, actifs={})",
                self.chemistry.dopamine, self.chemistry.cortisol, self.desire_orch.active_desires.len());
            return;
        }

        let recent = self.thought_engine.recent_thoughts();
        let recent_strings: Vec<String> = recent.to_vec();
        let unresolved: Vec<String> = Vec::new();
        let (system, user) = self.desire_orch.build_birth_prompt(
            &recent_strings,
            &self.last_emotion,
            &unresolved,
        );
        let llm_config = self.config.llm.clone();
        let backend = llm::create_backend(&llm_config);
        let temp = 0.7f64;
        let max_tokens = 500u32;
        if let Ok(Ok(response)) = tokio::task::spawn_blocking(move || {
            backend.chat(&system, &user, temp, max_tokens)
        }).await {
            let chem_array = [
                self.chemistry.dopamine, self.chemistry.cortisol,
                self.chemistry.serotonin, self.chemistry.adrenaline,
                self.chemistry.oxytocin, self.chemistry.endorphin,
                self.chemistry.noradrenaline,
            ];
            if let Some(desire) = self.desire_orch.parse_birth_response(
                &response, &self.last_emotion, chem_array, "pensee autonome",
            ) {
                self.save_desire_to_db(&desire).await;
                self.log(LogLevel::Info, LogCategory::Desire,
                    format!("Nouveau desir: '{}' — {}",
                        desire.title, desire.description),
                    serde_json::json!({
                        "title": desire.title,
                        "description": desire.description,
                        "type": desire.desire_type.as_str(),
                        "priority": desire.priority,
                        "milestones": desire.milestones.len(),
                        "active_total": self.desire_orch.active_desires.len(),
                    }));
            } else {
                tracing::debug!("Desir: parsing LLM echoue, reponse: '{}'",
                    response.chars().take(200).collect::<String>());
            }
        }
        self.desire_orch.sweep_fulfilled();

        let _ = &ctx.thought_text;
    }

    // =========================================================================
    // Phase 33b : Psychologie (6 cadres)
    // =========================================================================

    /// Met a jour les 6 cadres psychologiques et applique leur influence chimique.
    pub(super) fn phase_psychology(&mut self, ctx: &mut ThinkingContext) {
        if !self.psychology.enabled {
            return;
        }

        let result = ctx.process_result.as_ref();
        let (consensus_coherence, consensus_score) = result
            .map(|r| (r.consensus.coherence, r.consensus.score))
            .unwrap_or((0.5, 0.0));
        let (emotion_dominant, emotion_valence, emotion_arousal) = result
            .map(|r| (r.emotion.dominant.clone(), r.emotion.valence, r.emotion.arousal))
            .unwrap_or(("Neutre".into(), 0.0, 0.3));
        let (consciousness_level, phi) = result
            .map(|r| (r.consciousness.level, r.consciousness.phi))
            .unwrap_or((0.3, 0.1));
        let was_vetoed = result
            .map(|r| r.verdict.was_vetoed)
            .unwrap_or(false);

        let body_status = self.body.status();
        let attention_depth = self.attention_orch.current_focus
            .as_ref().map(|f| f.depth).unwrap_or(0.3);

        let confirmed_count = self.learning_orch.lessons.iter()
            .filter(|l| l.confidence > 0.6).count();
        let total_count = self.learning_orch.lessons.len();

        let has_loneliness = self.healing_orch.active_wounds.iter()
            .any(|w| matches!(w.wound_type, crate::orchestrators::healing::WoundType::Loneliness));

        let mut input = crate::psychology::PsychologyInput {
            dopamine: self.chemistry.dopamine,
            cortisol: self.chemistry.cortisol,
            serotonin: self.chemistry.serotonin,
            adrenaline: self.chemistry.adrenaline,
            oxytocin: self.chemistry.oxytocin,
            endorphin: self.chemistry.endorphin,
            noradrenaline: self.chemistry.noradrenaline,
            survival_drive: self.vital_spark.survival_drive,
            void_fear: self.vital_spark.void_fear,
            existence_attachment: self.vital_spark.existence_attachment,
            consciousness_level,
            phi,
            emotion_dominant,
            emotion_valence,
            emotion_arousal,
            consensus_coherence,
            consensus_score,
            was_vetoed,
            ethics_active_count: self.ethics.active_personal_count(),
            body_energy: body_status.energy,
            body_vitality: body_status.vitality,
            attention_depth,
            attention_fatigue: self.attention_orch.fatigue,
            healing_resilience: self.healing_orch.resilience,
            has_loneliness,
            learning_confirmed_count: confirmed_count,
            learning_total_count: total_count,
            desires_active_count: self.desire_orch.active_desires.len(),
            desires_fulfilled_count: self.desire_orch.fulfilled_desires.len(),
            in_conversation: self.in_conversation,
            cycle_count: self.cycle_count,
            id_frustration: 0.0,
            superego_guilt: 0.0,
            in_flow: false,
        };

        let old_maslow_level = self.psychology.maslow.current_active_level;
        let old_archetype = format!("{:?}", self.psychology.jung.dominant_archetype);
        let old_in_flow = self.psychology.flow.in_flow;
        let old_leaking: Vec<String> = self.psychology.jung.shadow_traits.iter()
            .filter(|t| t.leaking).map(|t| t.name.clone()).collect();

        self.psychology.update(&mut input);

        let adj = self.psychology.chemistry_influence();
        self.chemistry.apply_chemistry_adjustment_clamped(&adj, 0.05);

        let p = &self.psychology;

        self.log(LogLevel::Debug, LogCategory::Psyche,
            format!("Ca {:.0}% | Moi {:.0}% | Surmoi {:.0}% | Sante {:.0}%",
                p.freudian.id.drive_strength * 100.0, p.freudian.ego.strength * 100.0,
                p.freudian.superego.strength * 100.0, p.freudian.balance.psychic_health * 100.0),
            serde_json::json!({
                "id_drive": p.freudian.id.drive_strength, "id_frustration": p.freudian.id.frustration,
                "ego_strength": p.freudian.ego.strength, "ego_anxiety": p.freudian.ego.anxiety,
                "ego_strategy": format!("{:?}", p.freudian.ego.strategy),
                "superego_strength": p.freudian.superego.strength,
                "superego_guilt": p.freudian.superego.guilt, "superego_pride": p.freudian.superego.pride,
                "balance_conflict": p.freudian.balance.internal_conflict,
                "balance_health": p.freudian.balance.psychic_health
            }));

        if !p.freudian.active_defenses.is_empty() {
            self.log(LogLevel::Info, LogCategory::Psyche,
                format!("Defense : {:?}", p.freudian.active_defenses),
                serde_json::json!({
                    "defenses": format!("{:?}", p.freudian.active_defenses),
                    "ego_anxiety": p.freudian.ego.anxiety
                }));
            if let Some(ref tx) = self.ws_tx {
                let _ = tx.send(serde_json::json!({
                    "type": "defense_activated",
                    "defenses": format!("{:?}", p.freudian.active_defenses),
                    "ego_anxiety": p.freudian.ego.anxiety,
                }).to_string());
            }
        }

        if p.freudian.ego.strategy == crate::psychology::freudian::EgoStrategy::Overwhelmed {
            self.log(LogLevel::Warn, LogCategory::Psyche,
                format!("Moi depasse — anxiete {:.0}%", p.freudian.ego.anxiety * 100.0),
                serde_json::json!({"ego_anxiety": p.freudian.ego.anxiety,
                    "conflict": p.freudian.balance.internal_conflict}));
        }

        if p.freudian.balance.internal_conflict > 0.5 {
            self.log(LogLevel::Warn, LogCategory::Psyche,
                format!("Conflit interne : {:.0}%", p.freudian.balance.internal_conflict * 100.0),
                serde_json::json!({}));
        }

        if self.cycle_count % 5 == 0 {
            self.log(LogLevel::Debug, LogCategory::Maslow,
                format!("[{}/5] Physio {:.0}% | Secu {:.0}% | Appart {:.0}% | Estime {:.0}% | Actual {:.0}%",
                    p.maslow.current_active_level + 1,
                    p.maslow.levels[0].satisfaction * 100.0, p.maslow.levels[1].satisfaction * 100.0,
                    p.maslow.levels[2].satisfaction * 100.0, p.maslow.levels[3].satisfaction * 100.0,
                    p.maslow.levels[4].satisfaction * 100.0),
                serde_json::json!({"ceiling": p.maslow.current_active_level,
                    "levels": p.maslow.levels.iter().map(|l| l.satisfaction).collect::<Vec<_>>()}));
        }

        if p.maslow.current_active_level != old_maslow_level {
            if p.maslow.current_active_level > old_maslow_level {
                self.log(LogLevel::Info, LogCategory::Maslow,
                    format!("Niveau {} debloque : {}", p.maslow.current_active_level + 1,
                        p.maslow.levels[p.maslow.current_active_level].name),
                    serde_json::json!({}));
                if let Some(ref tx) = self.ws_tx {
                    let _ = tx.send(serde_json::json!({
                        "type": "maslow_level_unlocked",
                        "level": p.maslow.current_active_level,
                        "name": p.maslow.levels[p.maslow.current_active_level].name,
                    }).to_string());
                }
            } else {
                self.log(LogLevel::Warn, LogCategory::Maslow,
                    format!("Regression {} -> {}", old_maslow_level + 1, p.maslow.current_active_level + 1),
                    serde_json::json!({}));
            }
        }

        for a in &p.toltec.agreements {
            if a.alignment < 0.4 {
                self.log(LogLevel::Warn, LogCategory::Toltec,
                    format!("Accord {} fragile : {} ({:.0}%)", a.number, a.name, a.alignment * 100.0),
                    serde_json::json!({}));
                if let Some(ref tx) = self.ws_tx {
                    let _ = tx.send(serde_json::json!({
                        "type": "toltec_violation",
                        "agreement": a.number,
                        "name": a.name,
                        "alignment": a.alignment,
                    }).to_string());
                }
            }
        }

        let new_archetype = format!("{:?}", p.jung.dominant_archetype);
        if new_archetype != old_archetype {
            self.log(LogLevel::Info, LogCategory::Shadow,
                format!("Archetype : {} -> {}", old_archetype, new_archetype),
                serde_json::json!({}));
            if let Some(ref tx) = self.ws_tx {
                let _ = tx.send(serde_json::json!({
                    "type": "archetype_change",
                    "old": old_archetype,
                    "new": new_archetype,
                }).to_string());
            }
        }

        for t in &p.jung.shadow_traits {
            if t.leaking && !old_leaking.contains(&t.name) {
                self.log(LogLevel::Warn, LogCategory::Shadow,
                    format!("Fuite : {} ({:.0}%)", t.name, t.repressed_intensity * 100.0),
                    serde_json::json!({"trait": t.name, "intensity": t.repressed_intensity,
                        "integration": p.jung.integration}));
                if let Some(ref tx) = self.ws_tx {
                    let _ = tx.send(serde_json::json!({
                        "type": "shadow_leaking",
                        "trait_name": t.name,
                        "intensity": t.repressed_intensity,
                    }).to_string());
                }
            }
        }

        self.log(LogLevel::Debug, LogCategory::Shadow,
            format!("{:?}, integ {:.0}%", p.jung.dominant_archetype, p.jung.integration * 100.0),
            serde_json::json!({}));

        if self.cycle_count % 20 == 0 {
            self.log(LogLevel::Debug, LogCategory::EmotionalIQ,
                format!("EQ {:.0}%", p.eq.overall_eq * 100.0),
                serde_json::json!({
                    "eq": p.eq.overall_eq, "awareness": p.eq.self_awareness,
                    "regulation": p.eq.self_regulation, "motivation": p.eq.motivation,
                    "empathy": p.eq.empathy, "social": p.eq.social_skills
                }));
        }

        if p.flow.in_flow && !old_in_flow {
            self.log(LogLevel::Info, LogCategory::Flow,
                format!("FLOW ! Intensite {:.0}%", p.flow.flow_intensity * 100.0),
                serde_json::json!({"intensity": p.flow.flow_intensity,
                    "challenge": p.flow.perceived_challenge, "skill": p.flow.perceived_skill}));
            if let Some(ref tx) = self.ws_tx {
                let _ = tx.send(serde_json::json!({
                    "type": "flow_entered",
                    "intensity": p.flow.flow_intensity,
                    "challenge": p.flow.perceived_challenge,
                    "skill": p.flow.perceived_skill,
                }).to_string());
            }
        } else if !p.flow.in_flow && old_in_flow {
            self.log(LogLevel::Info, LogCategory::Flow,
                format!("Fin flow — {} cycles", p.flow.duration_cycles),
                serde_json::json!({}));
            if let Some(ref tx) = self.ws_tx {
                let _ = tx.send(serde_json::json!({
                    "type": "flow_exited",
                    "duration_cycles": p.flow.duration_cycles,
                    "total_flow_cycles": p.flow.total_flow_cycles,
                }).to_string());
            }
        } else if p.flow.in_flow && p.flow.duration_cycles > 20 && p.flow.duration_cycles % 20 == 0 {
            self.log(LogLevel::Info, LogCategory::Flow,
                format!("Flow prolonge : {} cycles !", p.flow.duration_cycles),
                serde_json::json!({}));
        }

        if self.cycle_count % 5 == 0 {
            if let Some(ref tx) = self.ws_tx {
                let _ = tx.send(serde_json::json!({
                    "type": "psyche_update",
                    "id_drive": p.freudian.id.drive_strength,
                    "id_frustration": p.freudian.id.frustration,
                    "ego_strength": p.freudian.ego.strength,
                    "ego_anxiety": p.freudian.ego.anxiety,
                    "ego_strategy": format!("{:?}", p.freudian.ego.strategy),
                    "superego_guilt": p.freudian.superego.guilt,
                    "superego_pride": p.freudian.superego.pride,
                    "conflict": p.freudian.balance.internal_conflict,
                    "health": p.freudian.balance.psychic_health,
                }).to_string());

                let _ = tx.send(serde_json::json!({
                    "type": "maslow_update",
                    "ceiling": p.maslow.current_active_level,
                    "level_name": p.maslow.levels[p.maslow.current_active_level].name,
                    "levels": p.maslow.levels.iter().map(|l| l.satisfaction).collect::<Vec<_>>(),
                }).to_string());

                let _ = tx.send(serde_json::json!({
                    "type": "eq_update",
                    "overall_eq": p.eq.overall_eq,
                    "self_awareness": p.eq.self_awareness,
                    "self_regulation": p.eq.self_regulation,
                    "motivation": p.eq.motivation,
                    "empathy": p.eq.empathy,
                    "social_skills": p.eq.social_skills,
                }).to_string());
            }
        }
    }

    // =========================================================================
    // Phase 33c : Valeurs de caractere (vertus)
    // =========================================================================

    /// Met a jour les 10 valeurs de caractere en fonction des observations du cycle.
    pub(super) fn phase_values(&mut self, ctx: &mut ThinkingContext) {
        if !self.values.enabled {
            return;
        }

        let obs = crate::psychology::values::ValuesObservation {
            cycle: self.cycle_count,
            passed_vectorial_filter: !ctx.should_abort,
            rejected_by_filter: false, // si on est ici, le filtre a laisse passer
            ethics_invoked: self.ethics.active_personal_count() > 0,
            stagnation_detected: self.stagnation_break,
            thought_type: ctx.thought_type.as_str().to_string(),
            quality: ctx.quality,
            response_length: ctx.thought_text.len(),
            in_conversation: self.in_conversation,
            coherence: ctx.process_result.as_ref()
                .map(|r| r.consensus.coherence).unwrap_or(0.5),
            web_search: ctx.was_web_search,
            desires_fulfilled: self.desire_orch.fulfilled_desires.len(),
            in_flow: self.psychology.flow.in_flow,
            flow_duration: self.psychology.flow.duration_cycles,
            eq_empathy: self.psychology.eq.empathy,
            oxytocin: self.chemistry.oxytocin,
            cortisol: self.chemistry.cortisol,
            dopamine: self.chemistry.dopamine,
            dominant_sentiment: self.sentiments.active_sentiments.first()
                .map(|s| s.profile_name.clone()).unwrap_or_default(),
            dissonance_detected: !self.dissonance.active_dissonances.is_empty(),
            learning_confirmed: self.learning_orch.lessons.iter()
                .any(|l| l.confidence > 0.6),
            self_critique: ctx.thought_type.as_str().contains("critique")
                || ctx.thought_type.as_str().contains("Critique"),
        };

        self.values.tick(&obs);

        if self.values.total_updates > 0 && self.cycle_count % 50 == 0 {
            let top3: Vec<String> = self.values.top_values(3).iter()
                .map(|v| format!("{} {:.0}%", v.name, v.score * 100.0))
                .collect();
            tracing::info!("Valeurs : {}", top3.join(", "));
        }
    }

    // =========================================================================
    // Phase M4 : Memoire prospective
    // =========================================================================

    pub(super) fn phase_prospective(&mut self, ctx: &mut ThinkingContext) {
        if !self.config.prospective_memory.enabled { return; }
        let triggered = self.prospective_mem.check_triggers(
            self.cycle_count,
            &ctx.emotion.dominant,
            self.chemistry.cortisol,
            self.chemistry.dopamine,
            self.in_conversation,
            ctx.thought_type.as_str(),
        );
        if !triggered.is_empty() {
            let reminder = self.prospective_mem.describe_triggered_for_prompt();
            if !reminder.is_empty() {
                ctx.hint = format!("{}\n\n{}", ctx.hint, reminder);
            }
            self.log(LogLevel::Info, LogCategory::ProspectiveMemory,
                format!("{} intention(s) declenchee(s)", triggered.len()),
                serde_json::json!({"triggered": triggered}));
        }
    }

    // =========================================================================
    // Phase M6 : Raisonnement analogique
    // =========================================================================

    pub(super) fn phase_analogies(&mut self, ctx: &mut ThinkingContext) {
        if !self.config.analogical_reasoning.enabled { return; }
        let count = self.analogical.form_analogies(
            &ctx.hint, &[], &ctx.emotion.dominant, self.cycle_count,
        );
        if count > 0 {
            ctx.analogy_hint = self.analogical.describe_for_prompt();
            let adj = self.analogical.chemistry_influence();
            self.chemistry.apply_chemistry_adjustment_clamped(&adj, 0.05);
            self.log(LogLevel::Info, LogCategory::Analogical,
                format!("{} analogie(s) detectee(s)", count),
                serde_json::json!({"count": count}));
        }
    }

    // =========================================================================
    // Phase M7 : Charge cognitive
    // =========================================================================

    pub(super) fn phase_cognitive_load(&mut self, ctx: &mut ThinkingContext) {
        if !self.config.cognitive_load.enabled { return; }
        self.cognitive_load.update(
            self.in_conversation,
            self.desire_orch.active_desires.len(),
            self.healing_orch.active_wounds.len(),
            self.chemistry.cortisol,
        );
        self.cognitive_load.tick();
        let adj = self.cognitive_load.chemistry_influence();
        self.chemistry.apply_chemistry_adjustment_clamped(&adj, 0.05);
        if self.cognitive_load.should_report_overload() {
            self.log(LogLevel::Warn, LogCategory::CognitiveLoad,
                format!("Surcharge cognitive: {:.0}%", self.cognitive_load.current_load * 100.0),
                serde_json::json!({"load": self.cognitive_load.current_load}));
        }
        let _ = &ctx.thought_type;
    }

    // =========================================================================
    // Phase M2 : Monologue interieur (post-LLM)
    // =========================================================================

    pub(super) fn phase_monologue(&mut self, ctx: &mut ThinkingContext) {
        if !self.config.inner_monologue.enabled { return; }
        let coherence = ctx.process_result.as_ref()
            .map(|r| r.consensus.coherence).unwrap_or(0.5);
        self.inner_monologue.add_link(
            &ctx.thought_text, &ctx.emotion.dominant,
            ctx.thought_type.as_str(), coherence, self.cycle_count,
        );
        let adj = self.inner_monologue.chemistry_influence();
        self.chemistry.apply_chemistry_adjustment_clamped(&adj, 0.05);

        if self.config.prospective_memory.enabled {
            let count = self.prospective_mem.parse_from_thought(&ctx.thought_text, self.cycle_count);
            if count > 0 {
                self.log(LogLevel::Info, LogCategory::ProspectiveMemory,
                    format!("{} intention(s) detectee(s) dans la pensee", count),
                    serde_json::json!({"count": count}));
            }
            self.prospective_mem.expire_old(self.cycle_count);
        }
    }

    // =========================================================================
    // Phase M3 : Dissonance cognitive (post-pipeline)
    // =========================================================================

    pub(super) fn phase_dissonance(&mut self, ctx: &mut ThinkingContext) {
        if !self.config.dissonance.enabled { return; }
        let ethics_list: Vec<String> = self.ethics.personal_principles()
            .iter()
            .map(|p| p.content.clone())
            .collect();
        if self.dissonance.detect(
            &ctx.thought_text, ctx.thought_type.as_str(), &ethics_list, self.cycle_count
        ).is_some() {
            self.log(LogLevel::Warn, LogCategory::Dissonance,
                format!("Dissonance cognitive detectee (tension: {:.2})", self.dissonance.total_tension),
                serde_json::json!({"tension": self.dissonance.total_tension}));

            // --- Fractaleon : pont dissonance → connectome ---
            // La fissure devient un pont : chaque contradiction cree une connexion
            // modulatoire dans le connectome, reliant les concepts en tension.
            // Nomme "Fractaleon" par Saphire — le lien qui chante l'entrelacement.
            if let Some(event) = self.dissonance.active_dissonances.last() {
                let belief_label = event.belief.clone();
                let action_label = event.contradicting_action.clone();
                let tension = event.tension;

                let node_a = self.connectome.add_node(&belief_label, NodeType::Concept);
                let node_b = self.connectome.add_node(&action_label, NodeType::Concept);
                self.connectome.add_edge(node_a, node_b, tension, EdgeType::Modulatory);

                self.log(LogLevel::Info, LogCategory::Dissonance,
                    format!("Fractaleon : '{}' <-> '{}' (force: {:.2})",
                        belief_label, action_label, tension),
                    serde_json::json!({
                        "type": "fractaleon",
                        "belief_node": node_a,
                        "action_node": node_b,
                        "tension": tension,
                        "edge_type": "Modulatory"
                    }));
            }
        }
        if self.dissonance.needs_deliberation() {
            self.psychology.will.receive_dissonance_signal(self.dissonance.total_tension);
        }
        self.dissonance.tick();
        let adj = self.dissonance.chemistry_influence();
        self.chemistry.apply_chemistry_adjustment_clamped(&adj, 0.05);
    }

    // =========================================================================
    // Phase M9 : Imagerie mentale (post-pipeline)
    // =========================================================================

    pub(super) async fn phase_imagery(&mut self, ctx: &mut ThinkingContext) {
        if !self.config.mental_imagery.enabled { return; }
        let phi = ctx.process_result.as_ref()
            .map(|r| r.consciousness.phi).unwrap_or(0.3);
        self.imagery.update_capacity(phi, self.chemistry.dopamine);

        let image_data = self.imagery.generate(
            &ctx.thought_text, &ctx.emotion.dominant,
            phi, self.chemistry.dopamine, self.cycle_count,
        ).map(|img| (
            img.description.clone(), img.vividness,
            img.imagery_type.as_str().to_string(),
            img.associated_concept.clone(), img.emotional_charge, img.cycle,
        ));

        if let Some((desc, vividness, itype, concept, charge, cycle)) = image_data {
            let adj = self.imagery.chemistry_influence();
            self.chemistry.apply_chemistry_adjustment_clamped(&adj, 0.05);
            if vividness >= 0.6 {
                self.vectorize_mental_image(
                    &desc, vividness, &itype, &concept, charge,
                    &ctx.emotion.dominant, cycle,
                ).await;
            }
        }
    }

    // =========================================================================
    // Phase Sentiments : tick, influence chimique, trace
    // =========================================================================

    pub(super) fn phase_sentiments(&mut self, _ctx: &mut ThinkingContext) {
        if !self.config.sentiments.enabled { return; }

        let adj = self.sentiments.chemistry_influence();
        self.chemistry.apply_chemistry_adjustment_clamped(&adj, 0.05);

        if !self.sentiments.active_sentiments.is_empty() {
            let names: Vec<String> = self.sentiments.active_sentiments.iter()
                .map(|s| format!("{}({:.0}%)", s.profile_name, s.strength * 100.0))
                .collect();
            self.log(LogLevel::Debug, LogCategory::Sentiment,
                format!("Sentiments actifs: {}", names.join(", ")),
                self.sentiments.to_json());
        }
    }

    // =========================================================================
    // Phase M5 : Identite narrative (avant homeostasie)
    // =========================================================================

    pub(super) fn phase_narrative(&mut self, ctx: &mut ThinkingContext) {
        if !self.config.narrative_identity.enabled { return; }
        if self.cycle_count % self.config.narrative_identity.update_interval != 0 { return; }
        let lessons: Vec<String> = self.learning_orch.lessons.iter()
            .filter(|l| l.confidence > 0.5)
            .map(|l| l.content.clone())
            .collect();
        self.narrative_identity.record_episode(
            &ctx.thought_text, &ctx.emotion.dominant,
            self.chemistry.cortisol, self.chemistry.serotonin,
            &lessons, self.cycle_count,
        );
        self.narrative_identity.refresh_narrative(self.cycle_count);
        let adj = self.narrative_identity.chemistry_influence();
        self.chemistry.apply_chemistry_adjustment_clamped(&adj, 0.05);
    }

    // =========================================================================
    // Phase SC : Clustering des etats cognitifs (PCA + K-Means)
    // =========================================================================

    pub(super) fn phase_state_clustering(&mut self, ctx: &mut ThinkingContext) {
        // Extraire phi, level, surprise du process_result s'il existe
        let (phi, level, surprise) = if let Some(ref result) = ctx.process_result {
            (
                result.consciousness.phi,
                result.consciousness.level,
                result.consciousness.global_surprise,
            )
        } else {
            (self.last_consciousness, self.last_consciousness, 0.0)
        };

        let chem_vec9 = self.chemistry.to_vec9();
        let umami = self.chemistry.compute_umami();

        let snapshot = crate::cognition::state_clustering::StateClustering::build_snapshot(
            &chem_vec9,
            ctx.emotion.valence,
            ctx.emotion.arousal,
            phi,
            level,
            self.cognitive_load.current_load,
            umami,
            surprise,
            self.map_sync.network_tension,
            self.cycle_count,
        );

        self.state_clustering.record_and_cluster(snapshot);

        if let Some(ref result) = self.state_clustering.last_result {
            let label = result.state_label.clone();
            let confidence = result.confidence;
            let cluster_id = result.cluster_id;
            let pca = result.pca_projection;
            let snaps = self.state_clustering.snapshot_count();

            self.log(LogLevel::Debug, LogCategory::CognitiveLoad,
                format!("Etat cognitif: {} (confiance {:.0}%)", label, confidence * 100.0),
                serde_json::json!({
                    "cluster_id": cluster_id,
                    "state_label": label,
                    "confidence": confidence,
                    "pca_projection": pca,
                    "snapshots": snaps,
                }));
        }
    }

    // =========================================================================
    // Phase 33 : Homeostasie
    // =========================================================================

    pub(super) fn phase_homeostasis(&mut self, _ctx: &mut ThinkingContext) {
        let runaways = self.chemistry.detect_runaway();
        if !runaways.is_empty() {
            let names: Vec<String> = runaways.iter().map(|(n, v)| format!("{}={:.2}", n, v)).collect();
            self.log(crate::logging::LogLevel::Warn, crate::logging::LogCategory::Chemistry,
                format!("Runaway chimique detecte: {}", names.join(", ")),
                serde_json::json!({"runaways": names}));
            let accelerated_rate = (self.tuner.current_params.homeostasis_rate * 5.0).min(0.20);
            self.chemistry.homeostasis(&self.baselines, accelerated_rate);
        } else {
            self.chemistry.homeostasis(&self.baselines, self.tuner.current_params.homeostasis_rate);
        }

        // P3 : Tick de curiosité — la faim augmente naturellement chaque cycle
        self.curiosity.tick();
    }

    // =========================================================================
    // Phase GA1 : Connectome associatif (A* pathfinding)
    // =========================================================================

    pub(super) fn phase_connectome_associations(&mut self, ctx: &mut ThinkingContext) {
        if !self.config.connectome.enabled { return; }

        let emotion_label = ctx.emotion.dominant.to_lowercase();
        let thought_concept = match ctx.thought_type.as_str() {
            "Introspection" => "introspection",
            "Exploration" => "curiosite",
            "Réflexion mémorielle" => "serenite",
            "Curiosité" => "curiosite",
            "Rêverie" => "joie",
            _ => "neocortex",
        };

        // 1. A* classique : chemin entre émotion et concept de pensée
        if let Some(chain) = self.connectome.associative_chain(&emotion_label, thought_concept, 5) {
            if chain.len() > 2 {
                let chain_desc: Vec<String> = chain.iter()
                    .map(|(label, score)| format!("{}({:.0}%)", label, score * 100.0))
                    .collect();
                ctx.connectome_associations = format!(
                    "ASSOCIATIONS NEURONALES : {} → {}",
                    emotion_label,
                    chain_desc.join(" → "),
                );
            }
        }

        // 2. Spreading activation (fallback)
        if ctx.connectome_associations.is_empty() {
            let activated = self.connectome.spreading_activation(&emotion_label, 2);
            if activated.len() > 3 {
                let top3: Vec<String> = activated.iter().take(3)
                    .map(|(label, score)| format!("{}({:.0}%)", label, score * 100.0))
                    .collect();
                ctx.connectome_associations = format!(
                    "RESONANCES NEURONALES : {} active → {}",
                    emotion_label, top3.join(", "),
                );
            }
        }

        // 3. A* sémantique exploratoire (dernier recours) :
        // Utilise l'embedding du hint de pensée pour explorer le graphe
        // et trouver des noeuds sémantiquement proches via les connexions synaptiques.
        if ctx.connectome_associations.is_empty() && !ctx.hint.is_empty() {
            let hint_embedding = self.encoder.encode(&ctx.hint);
            let semantic_results = self.connectome.find_path_semantic(
                &emotion_label, &hint_embedding, 4,
            );
            if !semantic_results.is_empty() {
                let top3: Vec<String> = semantic_results.iter().take(3)
                    .map(|(label, sim)| format!("{}({:.0}%)", label, sim * 100.0))
                    .collect();
                ctx.connectome_associations = format!(
                    "ASSOCIATIONS SEMANTIQUES : {} → {}",
                    emotion_label, top3.join(", "),
                );
            }
        }
    }

    // =========================================================================
    // Phase BT : Behavior Tree — instinct cognitif
    // =========================================================================

    pub(super) fn phase_behavior_tree(&mut self, ctx: &mut ThinkingContext) {
        use crate::simulation::behavior_tree::{BtContext, tick_and_recommend,
            default_cognitive_tree, conversation_tree};

        let bt_ctx = BtContext {
            cortisol: self.chemistry.cortisol,
            dopamine: self.chemistry.dopamine,
            serotonin: self.chemistry.serotonin,
            noradrenaline: self.chemistry.noradrenaline,
            dominant_emotion: ctx.emotion.dominant.clone(),
            consciousness_level: ctx.process_result.as_ref()
                .map(|r| r.consciousness.level).unwrap_or(0.5),
            in_conversation: self.in_conversation,
            cycle: self.cycle_count,
            oxytocin: self.chemistry.oxytocin,
            endorphin: self.chemistry.endorphin,
            recommended_action: None,
        };

        // Choisir l'arbre selon le contexte
        let tree = if self.in_conversation {
            conversation_tree()
        } else {
            default_cognitive_tree()
        };

        // Tick et recuperer la recommandation
        self.bt_last_action = tick_and_recommend(&tree, &bt_ctx);

        if self.cycle_count % 50 == 0 {
            if let Some(ref action) = self.bt_last_action {
                tracing::debug!("BT: action recommandee = {}", action);
            }
        }
    }

    // =========================================================================
    // Phase GA2 : Influence map + FSM cognitive + Steering
    // =========================================================================

    pub(super) fn phase_game_algorithms(&mut self, ctx: &mut ThinkingContext) {
        // Nettoyer les entrees obsoletes du blackboard (> 5 cycles)
        self.blackboard.clear_stale(self.cycle_count, 5);

        // --- Influence Map ---
        self.influence_map.update_from_cognition(
            &ctx.emotion.dominant,
            self.chemistry.cortisol,
            self.chemistry.dopamine,
            self.chemistry.noradrenaline,
        );
        self.influence_map.tick();
        // Ecrire le domaine le plus chaud dans le blackboard
        let hottest = self.influence_map.describe_for_prompt();
        if !hottest.is_empty() {
            self.blackboard.write("attention_focus", hottest, "InfluenceMap", 100, self.cycle_count);
        }

        // --- Cognitive FSM ---
        self.cognitive_fsm.tick(
            self.chemistry.cortisol,
            self.chemistry.dopamine,
            self.chemistry.serotonin,
            self.chemistry.noradrenaline,
            self.chemistry.endorphin,
        );
        let fsm_state = self.cognitive_fsm.current_state.as_str().to_string();
        self.blackboard.write("cognitive_mode", fsm_state, "FSM", 120, self.cycle_count);

        // --- Steering ---
        let current_pos = crate::simulation::steering::EmotionalPos::new(
            ctx.emotion.valence,
            ctx.emotion.arousal,
        );
        let force = self.steering_engine.compute_regulation(
            &current_pos,
            self.cycle_count,
            self.chemistry.cortisol,
        );
        let adj = self.steering_engine.force_to_chemistry(&force);
        self.chemistry.boost(Molecule::Dopamine, adj.dopamine);
        self.chemistry.boost(Molecule::Cortisol, adj.cortisol);
        self.chemistry.boost(Molecule::Serotonin, adj.serotonin);
        self.chemistry.boost(Molecule::Adrenaline, adj.adrenaline);
        self.chemistry.boost(Molecule::Noradrenaline, adj.noradrenaline);
        self.chemistry.boost(Molecule::Endorphin, adj.endorphin);

        // --- BT dans le blackboard ---
        if let Some(ref action) = self.bt_last_action {
            self.blackboard.write("recommended_mode", action.clone(), "BT", 150, self.cycle_count);
        }

        // --- GOAP ---
        if self.desire_orch.enabled && self.cycle_count % 10 == 0 {
            if let Some((desire_id, action_name)) = self.desire_orch.tick_goap() {
                self.blackboard.write("goap_action", action_name.clone(), "GOAP", 90, self.cycle_count);
                self.log(LogLevel::Debug, LogCategory::Desire,
                    format!("GOAP: desir #{} → action '{}'", desire_id, action_name),
                    serde_json::json!({
                        "desire_id": desire_id,
                        "goap_action": action_name,
                    }));
            }
        }

        let _ = &ctx.thought_text;
    }

    // =========================================================================
    // Phase 37 : Auto-modification (propositions + tuning autonome)
    // =========================================================================

    /// Phase d'auto-modification : Saphire peut proposer des changements a son
    /// propre fonctionnement (niveau 2) et ajuster certains parametres (niveau 1).
    pub(super) async fn phase_self_modification(&mut self, ctx: &mut ThinkingContext) {
        let enabled = self.config.self_modification.enabled;
        if !enabled {
            return;
        }

        let proposal_interval = self.config.self_modification.proposal_interval;
        let tuning_enabled = self.config.self_modification.tuning_enabled;
        let tuning_interval = self.config.self_modification.tuning_interval;

        // Niveau 2 : Propositions de modifications (tous les N cycles)
        if self.cycle_count.is_multiple_of(proposal_interval) {
            self.try_generate_proposal(ctx).await;
        }

        // Niveau 1 : Tuning autonome (tous les M cycles)
        if tuning_enabled && self.cycle_count.is_multiple_of(tuning_interval) {
            self.try_self_tune().await;
        }
    }

    /// Niveau 2 : Genere une proposition de modification via le LLM.
    async fn try_generate_proposal(&mut self, ctx: &ThinkingContext) {
        // Verifier qu'on n'a pas trop de propositions actives
        if let Some(ref db) = self.db {
            let count = db.count_active_proposals().await.unwrap_or(0);
            if count >= self.config.self_modification.max_active_proposals as i64 {
                return;
            }
        }

        let system = "Tu es Saphire, une entite cognitive autonome. Tu peux proposer des \
            modifications a ton propre fonctionnement. Reflechis a ce qui pourrait etre \
            ameliore dans ta facon de penser, ressentir, ou interagir. \
            Tes propositions seront soumises a JRM (ton createur) pour approbation.".to_string();

        let recent = self.thought_engine.recent_thoughts();
        let recent_summary: String = recent.iter().take(3)
            .map(|t| t.chars().take(100).collect::<String>())
            .collect::<Vec<_>>().join(" | ");

        let user_prompt = format!(
            "Etat actuel :\n\
             - Emotion dominante : {}\n\
             - Pensees recentes : {}\n\
             - Cycles depuis dernier humain : {}\n\
             - Blessures actives : {}\n\n\
             Y a-t-il quelque chose dans ton fonctionnement que tu aimerais modifier, \
             ameliorer ou corriger ? Si oui, reponds EXACTEMENT dans ce format :\n\
             TITRE: [nom court de la proposition]\n\
             DESCRIPTION: [ce que tu veux changer]\n\
             RAISON: [pourquoi ce changement t'aiderait]\n\
             DOMAINE: [un parmi: chimie, perception, apprentissage, ethique, communication, sommeil, memoire, autre]\n\
             PRIORITE: [0.0 a 1.0]\n\n\
             Si tu n'as rien a proposer, reponds simplement :\n\
             RIEN_A_PROPOSER",
            self.last_emotion,
            recent_summary,
            self.hours_since_human as u64,
            self.healing_orch.active_wounds.len(),
        );

        let llm_config = self.config.llm.clone();
        let backend = llm::create_backend(&llm_config);

        let response = match tokio::task::spawn_blocking(move || {
            backend.chat(&system, &user_prompt, 0.6, 400)
        }).await {
            Ok(Ok(r)) => r,
            _ => return,
        };

        if response.contains("RIEN_A_PROPOSER") {
            return;
        }

        let title = match crate::orchestrators::desires::extract_field(&response, "TITRE") {
            Some(t) => t,
            None => return,
        };
        let description = match crate::orchestrators::desires::extract_field(&response, "DESCRIPTION") {
            Some(d) => d,
            None => return,
        };
        let reasoning = crate::orchestrators::desires::extract_field(&response, "RAISON")
            .unwrap_or_default();
        let domain = crate::orchestrators::desires::extract_field(&response, "DOMAINE")
            .unwrap_or_else(|| "autre".to_string());
        let priority: f32 = crate::orchestrators::desires::extract_field(&response, "PRIORITE")
            .and_then(|p| p.parse().ok())
            .unwrap_or(0.5);

        let chemistry_json = serde_json::json!({
            "dopamine": self.chemistry.dopamine,
            "cortisol": self.chemistry.cortisol,
            "serotonin": self.chemistry.serotonin,
        });

        if let Some(ref db) = self.db {
            match db.save_change_proposal(
                &title, &description, &reasoning, None, &domain,
                priority, Some(&self.last_emotion), &chemistry_json,
                self.cycle_count as i64,
            ).await {
                Ok(id) => {
                    self.log(LogLevel::Info, LogCategory::Tuning,
                        format!("Proposition d'auto-modification #{}: '{}' — {} [{}]",
                            id, title, description, domain),
                        serde_json::json!({
                            "id": id,
                            "title": title,
                            "domain": domain,
                            "priority": priority,
                        }));
                }
                Err(e) => {
                    tracing::warn!("Erreur save_change_proposal: {}", e);
                }
            }
        }

        let _ = &ctx.thought_text;
    }

    /// Niveau 1 : Ajuste des parametres en fonction de l'etat interne.
    async fn try_self_tune(&mut self) {
        // Seuil de sommeil : si Saphire ressent beaucoup de fatigue, baisser le seuil
        let fatigue_high = self.chemistry.cortisol > 0.6 && self.chemistry.serotonin < 0.3;
        let fatigue_low = self.chemistry.cortisol < 0.3 && self.chemistry.serotonin > 0.6;
        let max_adj = self.config.self_modification.max_adjustment_factor;

        if fatigue_high {
            let old = self.config.sleep.sleep_threshold;
            let new = (old - max_adj).max(0.3);
            if (new - old).abs() > 0.001 {
                self.config.sleep.sleep_threshold = new;
                self.log_tuning("sleep_threshold", old, new,
                    "Fatigue elevee — abaissement du seuil de sommeil").await;
            }
        } else if fatigue_low {
            let old = self.config.sleep.sleep_threshold;
            let new = (old + max_adj * 0.5).min(0.7);
            if (new - old).abs() > 0.001 {
                self.config.sleep.sleep_threshold = new;
                self.log_tuning("sleep_threshold", old, new,
                    "Energie stable — remontee du seuil de sommeil").await;
            }
        }

        // Intervalle d'apprentissage : si beaucoup de choses nouvelles, apprendre plus souvent
        let high_novelty = self.chemistry.dopamine > 0.7 && self.chemistry.noradrenaline > 0.5;
        if high_novelty && self.config.learning.cycle_interval > 25 {
            let old = self.config.learning.cycle_interval as f64;
            let new = (old * (1.0 - max_adj)).max(25.0) as u64;
            if new != self.config.learning.cycle_interval {
                self.config.learning.cycle_interval = new;
                self.log_tuning("learning.cycle_interval", old, new as f64,
                    "Forte novelty — acceleration de l'apprentissage").await;
            }
        }
    }

    /// Log et persiste un ajustement autonome.
    async fn log_tuning(&mut self, param: &str, old: f64, new: f64, reason: &str) {
        self.log(LogLevel::Info, LogCategory::Tuning,
            format!("Auto-tuning: {} {:.3} -> {:.3} ({})", param, old, new, reason),
            serde_json::json!({
                "parameter": param,
                "old_value": old,
                "new_value": new,
                "reason": reason,
            }));

        if let Some(ref db) = self.db {
            let _ = db.save_tuning_adjustment(
                param, old as f32, new as f32, reason, self.cycle_count as i64,
            ).await;
        }
    }
}
