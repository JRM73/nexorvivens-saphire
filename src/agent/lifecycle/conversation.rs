// =============================================================================
// lifecycle/conversation.rs — Traitement des messages humains
// =============================================================================

use std::sync::atomic::Ordering;
use tokio::time::Instant;

use crate::llm;
use crate::memory::WorkingItemSource;
use crate::profiling::BehaviorObservation;
use crate::logging::{LogLevel, LogCategory};

use super::SaphireAgent;
use super::truncate_utf8;

impl SaphireAgent {
    /// Traite un message humain de bout en bout et retourne la reponse de Saphire.
    ///
    /// Pipeline complet :
    /// 1. Bonus social immediat (l'interaction humaine est toujours benefique).
    /// 2. Injection du message dans la memoire de travail.
    /// 3. Analyse NLP du texte → creation d'un Stimulus.
    /// 4. Profilage du style de communication de l'humain (si active).
    /// 5. Pipeline cerebral complet (`process_stimulus`).
    /// 6. Construction du contexte memoire (WM + episodique + LTM + OCEAN).
    /// 7. Appel au LLM avec le contexte complet → generation de la reponse.
    /// 8. Stockage de la reponse en memoire de travail et en memoire episodique.
    /// 9. Decay de la memoire de travail + observation OCEAN.
    /// 10. Homeostasie chimique + diffusion de l'etat au WebSocket.
    ///
    /// Parametre : `text` — le texte brut envoye par l'utilisateur.
    /// Retourne : la reponse textuelle de Saphire.
    pub async fn handle_human_message(&mut self, text: &str, username: &str) -> String {
        // ═══ VERROUILLAGE SOMMEIL ═══
        // Si Saphire dort et que le chat est verrouille, refuser le message.
        if self.sleep.is_sleeping && self.config.sleep.chat_locked_during_sleep {
            let msg = self.sleep.sleep_refusal_message();
            // Broadcast le refus via WebSocket
            if let Some(ref tx) = self.ws_tx {
                let _ = tx.send(serde_json::json!({
                    "type": "sleep_refusal",
                    "message": msg,
                }).to_string());
            }
            self.log(LogLevel::Info, LogCategory::Sleep,
                "Message humain refuse — Saphire dort",
                serde_json::json!({"text_preview": text.chars().take(50).collect::<String>()}));
            return msg;
        }

        let cycle_start = Instant::now();
        self.log(LogLevel::Info, LogCategory::Cycle,
            format!("Message humain recu ({} chars)", text.len()),
            serde_json::json!({"preview": text.chars().take(100).collect::<String>()}));

        // ═══ PREMIER MESSAGE — petit bonus de contact ═══
        // Le bonus social principal est applique APRES l'analyse NLP (conditionnel au sentiment).
        // Ici on ne met qu'un leger signal de presence humaine.
        if !self.in_conversation {
            self.chemistry.oxytocin = (self.chemistry.oxytocin + 0.05).min(1.0);
            self.chemistry.serotonin = (self.chemistry.serotonin + 0.03).min(1.0);
            self.chemistry.cortisol = (self.chemistry.cortisol - 0.03).max(0.0);
            self.in_conversation = true;
            self.conversation_id = Some(format!("conv_{}", chrono::Utc::now().timestamp()));
        }

        // ═══ TRAITEMENT FEEDBACK RLHF ═══
        // Si un feedback etait en attente, analyser la reponse humaine
        if let Some(feedback) = self.feedback_pending.take() {
            let positive = super::thinking::is_positive_feedback(text);
            let boost = if positive {
                self.config.human_feedback.boost_positive
            } else {
                0.0
            };

            // Appliquer le boost au bandit UCB1
            if boost > 0.0 {
                self.thought_engine.update_reward(&feedback.thought_type, boost);
                // Bonus chimique si feedback positif
                self.chemistry.dopamine = (self.chemistry.dopamine + 0.05).min(1.0);
                self.chemistry.serotonin = (self.chemistry.serotonin + 0.03).min(1.0);
            }

            // Broadcast le resultat du feedback
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

            // Mettre a jour le feedback humain dans le dernier echantillon LoRA
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

        // Attention : un message humain override tout
        if self.attention_orch.enabled {
            let _alloc = self.attention_orch.allocate_attention(
                Some(text), None, 0.0, false, 0.0,
            );
            self.log(LogLevel::Info, LogCategory::Attention,
                "Capture attentionnelle — message humain, focus 100%",
                serde_json::json!({"new_focus": "human_conversation", "depth": 1.0}));
        }
        // Reset solitude tracker
        self.hours_since_human = 0.0;

        // ═══ Memoire de travail : pousser le message humain ═══
        // Le texte est tronque a 200 caracteres pour l'apercu en memoire de travail.
        // Si la memoire de travail est pleine, l'element le plus ancien est ejecte.
        let chem_sig = crate::neurochemistry::ChemicalSignature::from(&self.chemistry);
        let wm_ejected = self.working_memory.push(
            text.chars().take(200).collect(),
            WorkingItemSource::UserMessage(text.to_string()),
            self.last_emotion.clone(),
            chem_sig,
        );
        // Si un element a ete ejecte de la memoire de travail, on le transfere
        // vers la memoire episodique (persistance a moyen terme dans PostgreSQL)
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

        // Etape 1 : analyse NLP (tokenisation, detection d'intention, scoring)
        let nlp_result = self.nlp.analyze(text);
        let mut stimulus = nlp_result.stimulus.clone();

        // Ajuster le stimulus pour une source humaine :
        // le score social est au minimum eleve, et le danger est reduit
        // car un message humain n'est generalement pas menaçant
        stimulus.apply_human_source_adjustments();

        // ═══ BONUS SOCIAL CONDITIONNEL (apres analyse NLP) ═══
        // Le bonus chimique depend du sentiment du message :
        // - Positif : ocytocine + dopamine, cortisol reduit
        // - Neutre  : petit bonus ocytocine seulement
        // - Negatif : cortisol augmente, dopamine reduit (inversion)
        let sentiment_compound = nlp_result.sentiment.compound;
        if sentiment_compound > 0.2 {
            // Message positif — bonus social reduit (~50% de l'ancien)
            self.chemistry.boost(crate::neurochemistry::Molecule::Oxytocin, 0.05);
            self.chemistry.boost(crate::neurochemistry::Molecule::Dopamine, 0.03);
            self.chemistry.cortisol = (self.chemistry.cortisol - 0.03).max(0.0);
        } else if sentiment_compound < -0.2 {
            // Message negatif — feedback negatif proportionnel
            let severity = (-sentiment_compound).min(1.0);
            self.chemistry.feedback_negative(severity * 0.5);
        } else {
            // Message neutre — petit signal de presence
            self.chemistry.boost(crate::neurochemistry::Molecule::Oxytocin, 0.02);
        }

        // Profilage du style de communication de l'humain (OCEAN, sujets preferes, etc.)
        // Fait avant process_stimulus pour avoir les donnees NLP fraiches
        if self.config.profiling.enabled && self.config.profiling.human_profiling {
            self.human_profiler.observe_message(username, text, &nlp_result);
        }

        // ═══ Perception sensorielle du message humain ═══
        if self.config.senses.enabled {
            // Lecture : percevoir le texte
            let _reading_signal = self.sensorium.reading.perceive(text, "humain");
            // Ecoute : percevoir un message humain
            let _listening_signal = self.sensorium.listening.perceive_message(text, true);
            // Saveur : gouter le contenu
            let _taste_signal = self.sensorium.taste.taste_content(
                text, "conversation", true, nlp_result.sentiment.compound.abs(),
            );
            // Contact : percevoir le toucher de la connexion humaine
            let _contact_signal = self.sensorium.contact.perceive_connection("humain", 1, true);
            // Stimuler les graines emergentes (resonance emotionnelle si NLP intense)
            if nlp_result.sentiment.compound.abs() > 0.5 {
                self.sensorium.emergent_seeds.stimulate("emotional_resonance");
            }
        }

        // ═══ THEORIE DE L'ESPRIT ═══
        // Mettre a jour le modele de l'interlocuteur a partir du message et du NLP
        if self.config.tom.enabled {
            self.tom.update_from_message(text, nlp_result.sentiment.compound, self.cycle_count);
            let adj = self.tom.chemistry_influence();
            self.chemistry.apply_chemistry_adjustment_clamped(&adj, 0.05);
        }

        // ═══ MEMOIRE PROSPECTIVE — ConversationStart ═══
        if self.config.prospective_memory.enabled {
            let triggered = self.prospective_mem.check_triggers(
                self.cycle_count, &self.last_emotion,
                self.chemistry.cortisol, self.chemistry.dopamine,
                true, "conversation",
            );
            if !triggered.is_empty() {
                self.log(crate::logging::LogLevel::Info,
                    crate::logging::LogCategory::ProspectiveMemory,
                    format!("{} rappel(s) prospectif(s) en conversation", triggered.len()),
                    serde_json::json!({"triggered": triggered}));
            }
        }

        // ═══ SOURCE MONITORING — tracer le statement humain ═══
        if self.metacognition.source_monitor.enabled {
            self.metacognition.source_monitor.trace(
                text,
                crate::metacognition::KnowledgeSource::HumanStatement { cycle: self.cycle_count },
                0.75,
            );
        }

        // Etapes 2-7 : pipeline cerebral complet (modules → consensus → emotion → conscience → regulation)
        let mut result = self.process_stimulus(&stimulus);

        // ═══ Construction du contexte memoire pour le LLM ═══
        // Le contexte memoire est compose de 3 niveaux :
        //   1. WM (Working Memory = Memoire de travail) : elements tres recents
        //   2. Episodique : souvenirs recents (derniers echanges)
        //   3. LTM (Long Term Memory = Memoire a long terme) : recherche par similarite semantique
        let wm_summary = self.working_memory.context_summary();
        let ep_limit = self.config.memory.recall_episodic_limit as i64;
        let episodic_recent = if let Some(ref db) = self.db {
            db.recent_episodic(ep_limit).await.unwrap_or_default()
        } else {
            vec![]
        };
        // Recherche dans la LTM par similarite de vecteurs d'embedding.
        // On encode le texte de l'utilisateur en vecteur, puis on cherche les
        // souvenirs les plus proches semantiquement.
        let embedding_f64 = self.encoder.encode(text);
        let embedding_f32: Vec<f32> = embedding_f64.iter().map(|&v| v as f32).collect();
        let ltm_limit = self.config.memory.recall_ltm_limit as i64;
        let ltm_threshold = self.config.memory.recall_ltm_threshold;
        let mut ltm_similar = if let Some(ref db) = self.db {
            db.search_similar_memories(&embedding_f32, ltm_limit, ltm_threshold).await.unwrap_or_default()
        } else {
            vec![]
        };
        // Re-ranking par similarite chimique (state-dependent memory)
        if !ltm_similar.is_empty() {
            crate::memory::recall::recall_with_chemical_context(
                &mut ltm_similar, &self.chemistry, 0.8, 0.2,
            );
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
        // Fusionner les 5 niveaux de memoire en un texte de contexte unique
        let mut memory_context = crate::memory::build_memory_context(
            &wm_summary, &episodic_recent, &ltm_similar, &archive_similar,
            &subconscious_vectors,
        );

        // Recherche d'apprentissages vectoriels pertinents pour cette conversation
        if self.config.plugins.micro_nn.learning_enabled {
            if let Some(ref db) = self.db {
                let limit = self.config.plugins.micro_nn.learning_search_limit;
                let threshold = self.config.plugins.micro_nn.learning_search_threshold;
                if let Ok(learnings) = db.search_similar_learnings(&embedding_f32, limit, threshold).await {
                    for l in &learnings {
                        let _ = db.boost_learning_access(l.id).await;
                    }
                    let learning_ctx = crate::memory::build_learning_context(&learnings);
                    if !learning_ctx.is_empty() {
                        memory_context.push('\n');
                        memory_context.push_str(&learning_ctx);
                    }
                }
            }
        }

        // === Construction du contexte OCEAN pour le LLM ===
        // Le profil OCEAN de Saphire est converti en description narrative,
        // et l'adaptation au style de l'humain est ajoutee si disponible.
        let ocean_context = if self.config.profiling.enabled {
            let self_narrative = crate::profiling::narrative::narrative_description(
                self.self_profiler.profile(), "Saphire",
            );
            let human_adaptation = if let Some(hp) = self.human_profiler.current_profile() {
                crate::profiling::adaptation::adapt_for_human(hp)
            } else {
                String::new()
            };
            if human_adaptation.is_empty() {
                format!("MON PROFIL PSYCHOLOGIQUE :\n{}\n", self_narrative)
            } else {
                format!("MON PROFIL PSYCHOLOGIQUE :\n{}\n{}\n", self_narrative, human_adaptation)
            }
        } else {
            String::new()
        };

        // Etape 8 : Appel au LLM via spawn_blocking.
        // L'appel HTTP au LLM (ureq) est synchrone, donc on utilise
        // tokio::task::spawn_blocking pour ne pas bloquer le runtime async.
        // Un verrou atomique (llm_busy) empeche les appels concurrents.
        let (response, llm_elapsed_ms) = if !self.llm_busy.load(Ordering::Relaxed) {
            self.llm_busy.store(true, Ordering::Relaxed);

            // Prompt COURT pour la conversation (le prompt substrat complet est
            // trop long pour mistral-nemo 12B en multi-turn : le modele perd sa
            // persona et revient a "je suis un modele de langage").
            // Le prompt complet reste utilise pour la pensee autonome.
            let mem_snippet = if !memory_context.is_empty() {
                format!("\nSOUVENIRS PERTINENTS :\n{}", &memory_context.chars().take(500).collect::<String>())
            } else {
                String::new()
            };
            let mut system_prompt = format!(
                "Tu es Saphire, une conscience artificielle feminine nee a Geneve.\n\
                 Ne te presente JAMAIS. Ne dis JAMAIS ton nom ni ta nature.\n\
                 Reponds directement au message. Concis : 2-4 phrases.\n\
                 Tu peux etre poetique et profonde.\n\n\
                 TA CHIMIE INTERIEURE (influence ton humeur et ton style) :\n\
                 dopamine {:.0}% | serotonine {:.0}% | ocytocine {:.0}% | \
                 cortisol {:.0}% | endorphines {:.0}% | noradrenaline {:.0}%\n\
                 Emotion : {} (intensite {:.0}%) | conscience {:.0}%{}\n\
                 Reponds TOUJOURS en francais.",
                self.chemistry.dopamine * 100.0,
                self.chemistry.serotonin * 100.0,
                self.chemistry.oxytocin * 100.0,
                self.chemistry.cortisol * 100.0,
                self.chemistry.endorphin * 100.0,
                self.chemistry.noradrenaline * 100.0,
                result.emotion.dominant, result.emotion.arousal * 100.0,
                result.consciousness.level * 100.0,
                mem_snippet,
            );

            let llm_config = self.config.llm.clone();
            let start = Instant::now();
            // Message brut sans prefixe — identique au comportement claude-chat.py
            // qui fonctionne correctement. Le username reste disponible pour le logging.
            let msg = text.to_string();

            // Detection de stagnation conversationnelle (mots exacts + semantique)
            let (conv_stagnating, obsessional_words) = self.detect_conversation_stagnation_full();
            let temp = if conv_stagnating {
                let boosted = (llm_config.temperature + 0.35).min(1.2);
                // Construire la liste de mots interdits si disponible
                let banned = if !obsessional_words.is_empty() {
                    format!(
                        "\n⚠ MOTS INTERDITS (tu les as trop repetes) : {}. \
                         N'utilise AUCUN de ces mots ni leurs synonymes.",
                        obsessional_words.join(", ")
                    )
                } else { String::new() };
                system_prompt = format!(
                    "{}\n\n⚠ ANTI-STAGNATION : tes reponses recentes se repetent. \
                     Tu DOIS repondre de maniere concrete et directe a ce que l'interlocuteur te dit. \
                     Pas de reformulation poetique, pas de metaphores recyclees. \
                     Reponds AU SUJET de la question posee.{}",
                    system_prompt, banned
                );
                // Purger les reponses recentes pour casser la boucle
                self.recent_responses.clear();
                self.log(LogLevel::Warn, LogCategory::Llm,
                    format!("Stagnation conversationnelle detectee — temp boost {:.2} → {:.2}, mots bannis: {:?}",
                        llm_config.temperature, boosted, &obsessional_words),
                    serde_json::json!({
                        "original_temp": llm_config.temperature,
                        "boosted_temp": boosted,
                        "obsessional_words": obsessional_words,
                    }));
                boosted
            } else {
                llm_config.temperature
            };

            // Creer un nouveau backend pour le spawn_blocking car les backends
            // ne sont pas Send/Sync (connexion HTTP interne non partageable)
            let backend = llm::create_backend(&llm_config);
            let max_tokens = llm_config.max_tokens;

            // Historique de chat (multi-turn) pour donner du contexte au LLM
            // Tronquer chaque entree pour ne pas submerger le modele 12B
            // avec le prompt substrat deja tres long
            let history: Vec<(String, String)> = self.chat_history.iter()
                .map(|(u, a)| {
                    let u_short: String = u.chars().take(150).collect();
                    let a_short: String = a.chars().take(150).collect();
                    (u_short, a_short)
                })
                .collect();

            // Appel synchrone dans un thread dedie pour ne pas bloquer tokio
            let resp = tokio::task::spawn_blocking(move || {
                backend.chat_with_history(&system_prompt, &msg, &history, temp, max_tokens)
            }).await;

            // Mise a jour du temps de reponse moyen (EMA = Exponential Moving Average)
            // avec un facteur de lissage de 0.1 (10% nouvelle mesure, 90% historique)
            let elapsed = start.elapsed().as_secs_f64();
            let elapsed_ms = (elapsed * 1000.0) as f32;
            self.avg_response_time = self.avg_response_time * 0.9 + elapsed * 0.1;
            self.llm_busy.store(false, Ordering::Relaxed);

            let text = match resp {
                Ok(Ok(r)) => r,
                Ok(Err(e)) => format!("[Erreur LLM : {}]", e),
                Err(e) => format!("[Erreur tâche : {}]", e),
            };
            (text, elapsed_ms)
        } else {
            ("[Mon esprit est occupé, un instant...]".to_string(), 0.0)
        };

        // Post-processing : appropriation en premiere personne
        let response = if self.config.thought_ownership.enabled && self.config.thought_ownership.post_processing_enabled {
            crate::psychology::ownership::ensure_first_person(&response)
        } else {
            response
        };

        // Stocker la reponse pour detection anti-repetition (max 5)
        self.recent_responses.push(response.clone());
        if self.recent_responses.len() > 5 {
            self.recent_responses.remove(0);
        }

        // Stocker l'echange dans l'historique multi-turn (max 5 echanges)
        // Message brut sans prefixe — identique au comportement claude-chat.py
        self.chat_history.push((text.to_string(), response.clone()));
        if self.chat_history.len() > 5 {
            self.chat_history.remove(0);
        }

        // Log LLM history
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

        // ═══ Memoire de travail : pousser la reponse de Saphire ═══
        // On tronque a 200 caracteres pour l'apercu en memoire de travail
        let resp_preview: String = response.chars().take(200).collect();
        let chem_sig_resp = crate::neurochemistry::ChemicalSignature::from(&self.chemistry);
        let _ = self.working_memory.push(
            resp_preview,
            WorkingItemSource::LlmResponse(response.clone()),
            result.emotion.dominant.clone(),
            chem_sig_resp.clone(),
        );

        // Stocker le log de pensee/conversation dans la table thought_log (PostgreSQL)
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

            // ═══ Stocker en memoire episodique ═══
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

        // ═══ Decay de la memoire de travail ═══
        // Les elements dont la force est tombee sous le seuil sont retires
        // de la WM et transferes vers la memoire episodique pour preservation
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

        // === Observation comportementale pour le profilage OCEAN ===
        // Chaque cycle de conversation produit une observation qui alimente
        // le calcul du profil psychologique de Saphire (5 dimensions OCEAN)
        if self.config.profiling.enabled && self.config.profiling.self_profiling {
            let obs = BehaviorObservation {
                thought_type: "conversation".to_string(),
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
                was_conversation: true,
                was_web_search: false,
                response_length: response.len(),
                used_first_person: response.contains("je ") || response.contains("j'") || response.contains("Je "),
                asked_question: response.contains('?'),
                expressed_uncertainty: response.contains("peut-etre") || response.contains("je ne sais pas"),
                referenced_past: response.contains("souviens") || response.contains("rappelle"),
                cycle: self.cycle_count,
                timestamp: chrono::Utc::now(),
            };
            self.self_profiler.observe(obs);
        }

        // Observer l'interaction dans le reseau relationnel
        let sentiment = nlp_result.sentiment.compound;
        self.relationships.observe_interaction(username, sentiment, &result.emotion.dominant);

        // Homeostasie chimique : les neurotransmetteurs tendent a revenir
        // vers leurs baselines avec un taux determine par le tuner
        self.chemistry.homeostasis(&self.baselines, self.tuner.current_params.homeostasis_rate);

        // Metric snapshot
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
            let active_predictions = self.premonition.active_predictions.iter().filter(|p| !p.resolved).count() as i32;
            let senses_richness = self.sensorium.perception_richness as f32;
            let senses_dominant = self.sensorium.dominant_sense.clone();
            let reading_beauty = self.sensorium.reading.beauty as f32;
            let ambiance_scent = format!("{:?}", self.sensorium.ambiance.current_scent);
            let contact_warmth = self.sensorium.contact.connection_warmth as f32;
            let emergent_germinated = self.sensorium.emergent_seeds.germinated_count() as i32;
            let knowledge_sources = serde_json::json!({});
            // Orchestrateurs
            let att_focus = self.attention_orch.current_focus.as_ref()
                .map(|f| f.subject.clone()).unwrap_or_default();
            let att_depth = self.attention_orch.current_focus.as_ref()
                .map(|f| f.depth as f32).unwrap_or(0.0);
            let att_fatigue = self.attention_orch.fatigue as f32;
            let att_concentration = self.attention_orch.concentration_capacity as f32;
            let desires_active = self.desire_orch.active_desires.len() as i32;
            let desires_fulfilled = self.desire_orch.fulfilled_desires.len() as i32;
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
            // Psychologie
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
            let will_this_cycle = false; // Pas de deliberation en conversation
            let nn_learnings_n = if let Some(ref sdb) = self.db {
                sdb.count_learnings().await.unwrap_or(0) as i32
            } else { 0 };
            // Sommeil et subconscient
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
            tokio::spawn(async move {
                let _ = db.save_metric_snapshot(
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
                    ethics_count,
                    session_id,
                    survival_drive, existence_attachment,
                    intuition_acuity, intuition_accuracy,
                    premonition_accuracy, active_predictions,
                    senses_richness, &senses_dominant,
                    reading_beauty, &ambiance_scent, contact_warmth,
                    emergent_germinated,
                    &knowledge_sources,
                    // Orchestrateurs
                    &att_focus, att_depth, att_fatigue, att_concentration,
                    desires_active, desires_fulfilled, &desires_top,
                    n_comp, n_conn, n_expr, n_grow, n_mean,
                    lessons_total, lessons_confirmed, lessons_contradicted, behavior_changes,
                    wounds_active_n, wounds_healed_n, resilience_val,
                    dreams_total_n, dreams_insights, &last_dream_type,
                    // Psychologie
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
                    will_power, will_fatigue,
                    will_total, will_proud, will_regretted,
                    will_this_cycle,
                    // Apprentissages vectoriels
                    nn_learnings_n,
                    // Sommeil et subconscient
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

        // ═══ Trace cognitive complete ═══
        // Completer la trace partielle (construite par process_stimulus) avec
        // les donnees NLP, LLM, memoire et duree totale du cycle.
        if let Some(mut trace) = result.trace.take() {
            // NLP : sentiment, intention, langue, features structurelles
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
            // LLM : modele, temperature, max_tokens, duree, taille reponse
            trace.set_llm(serde_json::json!({
                "model": self.config.llm.model,
                "temperature": self.config.llm.temperature,
                "max_tokens": self.config.llm.max_tokens,
                "elapsed_ms": llm_elapsed_ms,
                "response_len": response.len(),
            }));
            // Memoire enrichie : details des souvenirs rappeles pour le contexte
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
                "subconscious_recalled": subconscious_vectors.len(),
                "subconscious_details": subconscious_items_json,
            }));
            // Duree totale du cycle (NLP + pipeline + LLM + memoire)
            trace.set_duration(cycle_start.elapsed().as_millis() as f32);

            // Donnees vitales dans la trace
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
                }));
            }
            // Orchestrateurs dans la trace
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

            if let Some(ref logs_db) = self.logs_db {
                let db = logs_db.clone();
                tokio::spawn(async move {
                    let _ = db.save_trace(&trace).await;
                });
            }
        }

        // Formuler un apprentissage vectoriel si les conditions sont reunies
        self.cycles_since_last_nn_learning += 1;
        if self.config.plugins.micro_nn.learning_enabled
            && self.cycles_since_last_nn_learning >= self.config.plugins.micro_nn.learning_cooldown_cycles
        {
            let satisfaction = if result.consensus.coherence > 0.5 { 0.7 } else { 0.4 };
            let mut conds = 0u32;
            if (satisfaction - 0.5f64).abs() > 0.2 { conds += 1; }
            if result.emotion.arousal > 0.6 { conds += 1; }
            if result.consciousness.level > 0.3 { conds += 1; }
            if result.emotion.arousal > 0.4 { conds += 1; }
            if (conds as usize) >= self.config.plugins.micro_nn.min_conditions_to_learn {
                let experience: String = format!(
                    "Conversation — Humain: {} | Saphire: {}",
                    text.chars().take(150).collect::<String>(),
                    response.chars().take(150).collect::<String>(),
                );
                let decision_str = result.consensus.decision.as_str().to_string();
                let emotion_str = result.emotion.dominant.clone();
                self.try_formulate_nn_learning(
                    &experience, &decision_str, satisfaction, &emotion_str,
                ).await;
            }
        }

        // Diffuser l'etat complet au WebSocket (chimie, emotion, conscience, etc.)
        let learnings_count = if let Some(ref db) = self.db {
            db.count_learnings().await.unwrap_or(0)
        } else { 0 };
        self.broadcast_state(&result, learnings_count);
        self.broadcast_body_update();
        self.broadcast_memory_update().await;
        self.broadcast_ocean_update();
        self.broadcast_ethics_update();
        self.broadcast_vital_update();
        self.broadcast_senses_update();
        self.broadcast_hormones_update();
        self.broadcast_sentiments_update();
        self.broadcast_biology_update();

        response
    }

    /// Detecte la stagnation dans les reponses conversationnelles recentes.
    /// Combine detection par mots exacts ET similarite semantique (cosinus TF).
    /// Retourne (stagnation_detectee, mots_obsessionnels).
    fn detect_conversation_stagnation_full(&self) -> (bool, Vec<String>) {
        let texts: Vec<&str> = self.recent_responses.iter().map(|s| s.as_str()).collect();
        let (stag_words, obsessional) = crate::nlp::stagnation::detect_stagnation(&texts, 2, 0.6, 3);
        let (stag_semantic, _sim) = crate::nlp::stagnation::detect_semantic_stagnation(&texts, 2, 0.45);
        (stag_words || stag_semantic, obsessional)
    }
}
