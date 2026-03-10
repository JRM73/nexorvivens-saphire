// =============================================================================
// lifecycle/sleep_tick.rs — Tick de sommeil et initiation du sommeil
//
// Role : Gere chaque cycle quand Saphire dort. Chaque phase produit des
// effets specifiques sur la chimie, le corps, la memoire, les reves, etc.
// =============================================================================

use crate::logging::{LogLevel, LogCategory};
use crate::neurochemistry::Molecule;
use crate::orchestrators::dreams::SleepPhase;
use crate::llm;

use super::SaphireAgent;

impl SaphireAgent {
    /// Execute un tick de sommeil : applique les effets de la phase courante,
    /// decremente le compteur, et transitionne vers la phase suivante si besoin.
    pub async fn sleep_tick(&mut self) {
        let phase = match self.sleep.current_cycle {
            Some(ref c) => c.phase.clone(),
            None => return,
        };

        // Collecter les facteurs de qualite a chaque tick
        if let Some(ref mut c) = self.sleep.current_cycle {
            c.quality_factors.cortisol_sum += self.chemistry.cortisol;
            c.quality_factors.tick_count += 1;
        }

        match phase {
            SleepPhase::Hypnagogic => {
                // Chimie : apaisement progressif
                self.chemistry.cortisol = (self.chemistry.cortisol - 0.02).max(0.0);
                self.chemistry.boost(Molecule::Serotonin, 0.02);
                self.chemistry.adrenaline = (self.chemistry.adrenaline - 0.03).max(0.0);
                self.chemistry.noradrenaline = (self.chemistry.noradrenaline - 0.02).max(0.0);
                // Subconscient s'active
                self.subconscious.activation = (self.subconscious.activation + 0.1).min(1.0);
                // Energie remonte legerement
                self.body.soma.energy = (self.body.soma.energy + 0.005).min(1.0);

                // Image hypnagogique en fin de phase
                if let Some(ref c) = self.sleep.current_cycle {
                    if c.phase_remaining <= 3 {
                        self.log(LogLevel::Debug, LogCategory::Sleep,
                            "Image hypnagogique : les pensees se fragmentent",
                            serde_json::json!({
                                "phase": "hypnagogic",
                                "remaining": c.phase_remaining,
                            }));
                    }
                }
            },

            SleepPhase::LightSleep => {
                // Chimie douce
                self.chemistry.cortisol = (self.chemistry.cortisol - 0.01).max(0.0);
                // Energie remonte
                self.body.soma.energy = (self.body.soma.energy + 0.01).min(1.0);
                // Coeur ralentit
                self.body.heart.calm_down(60.0);
                // Subconscient travaille
                self.subconscious.background_process(0.0, "sommeil leger");
                // Fatigue attentionnelle se reduit
                self.attention_orch.reduce_fatigue(0.02);

                // Algorithmes de sommeil leger (une seule fois, debut de phase)
                if let Some(ref c) = self.sleep.current_cycle {
                    let dur = self.sleep.phase_duration(&SleepPhase::LightSleep);
                    if c.phase_remaining == dur.saturating_sub(1) {
                        self.sleep_light_algorithms().await;
                    }
                }
            },

            SleepPhase::DeepSleep => {
                // Compteur sommeil profond pour qualite
                if let Some(ref mut c) = self.sleep.current_cycle {
                    c.quality_factors.deep_sleep_ticks += 1;
                }

                // 1. Consolidation memoire
                let report = self.run_consolidation().await;
                if let Some(count) = report.get("consolidated").and_then(|v| v.as_u64()) {
                    if let Some(ref mut c) = self.sleep.current_cycle {
                        c.memories_consolidated += count;
                    }
                    if count > 0 {
                        self.log(LogLevel::Info, LogCategory::Sleep,
                            format!("{} souvenirs consolides en memoire profonde", count),
                            serde_json::json!({"consolidated": count}));
                    }
                }

                // 2. Guerison acceleree
                self.healing_orch.accelerated_heal(0.05);

                // 3. Integration jungienne nocturne
                self.psychology.jung.nocturnal_integration(0.003);

                // 4. Corps se restaure
                self.body.soma.energy = (self.body.soma.energy + 0.03).min(1.0);
                self.body.heart.calm_down(55.0);

                // 5. Chimie restauratrice
                self.chemistry.boost(Molecule::Endorphin, 0.02);
                self.chemistry.boost(Molecule::Serotonin, 0.01);

                // 6. Algorithmes de sommeil profond (milieu de phase)
                if let Some(ref c) = self.sleep.current_cycle {
                    let dur = self.sleep.phase_duration(&SleepPhase::DeepSleep);
                    if c.phase_remaining == dur / 2 {
                        self.sleep_deep_algorithms().await;
                    }
                }
            },

            SleepPhase::REM => {
                // Subconscient au maximum
                self.subconscious.activation = 1.0;

                // Debut de phase REM : generer un reve
                let is_rem_start = self.sleep.current_cycle.as_ref()
                    .map(|c| {
                        let rem_dur = self.sleep.phase_duration(&SleepPhase::REM);
                        c.phase_remaining == rem_dur.saturating_sub(1)
                    })
                    .unwrap_or(false);

                if is_rem_start && self.dream_orch.enabled {
                    // Determiner le type de reve
                    let dream_type = self.dream_orch.determine_dream_type(
                        self.chemistry.cortisol,
                        self.chemistry.dopamine,
                        self.chemistry.noradrenaline,
                        !self.subconscious.incubating_problems.is_empty(),
                    );

                    // Construire le prompt
                    let recent_memories: Vec<String> = self.working_memory.items()
                        .iter().take(3)
                        .map(|i| i.content.clone())
                        .collect();
                    let emotions = vec![self.last_emotion.clone()];
                    let unresolved: Vec<String> = self.subconscious.incubating_problems.iter()
                        .map(|p| p.question.clone())
                        .collect();

                    let (system_prompt, user_prompt) = self.dream_orch.build_dream_prompt(
                        &dream_type, &recent_memories, &emotions, &unresolved,
                    );

                    // Appel LLM a temperature REM
                    let temp = self.dream_orch.rem_temperature;
                    let llm_config = self.config.llm.clone();
                    let backend = llm::create_backend(&llm_config);
                    let response = tokio::task::spawn_blocking(move || {
                        backend.chat(&system_prompt, &user_prompt, temp, 400)
                    }).await;

                    if let Ok(Ok(dream_text)) = response {
                        let dream = self.dream_orch.parse_dream_response(
                            &dream_text, dream_type, vec![], &emotions, &unresolved,
                        );

                        // Log enrichi du reve
                        let narrative_preview: String = dream.narrative.chars().take(100).collect();
                        self.log(LogLevel::Info, LogCategory::Sleep,
                            format!("Reve ({}) : {}...", dream.dream_type.as_str(), narrative_preview),
                            serde_json::json!({
                                "dream_type": dream.dream_type.as_str(),
                                "has_insight": dream.insight.is_some(),
                                "insight": dream.insight,
                            }));

                        // Collecter les facteurs de qualite du reve
                        let is_nightmare = matches!(dream.dream_type,
                            crate::orchestrators::dreams::DreamType::Nightmare);
                        if let Some(ref mut c) = self.sleep.current_cycle {
                            c.quality_factors.dreams_total += 1;
                        }
                        if is_nightmare {
                            // Perturbation chimique du cauchemar
                            self.chemistry.boost(Molecule::Cortisol, 0.05);
                            self.chemistry.boost(Molecule::Adrenaline, 0.03);
                            self.chemistry.boost(Molecule::Noradrenaline, 0.02);
                            if let Some(ref mut c) = self.sleep.current_cycle {
                                c.quality_factors.nightmare_count += 1;
                            }
                            self.log(LogLevel::Warn, LogCategory::Sleep,
                                "Cauchemar — perturbation chimique nocturne",
                                serde_json::json!({
                                    "cortisol_boost": 0.05,
                                    "adrenaline_boost": 0.03,
                                }));
                        }

                        // Vectoriser le reve
                        let narr = dream.narrative.clone();
                        let emo = dream.dominant_emotion.clone();
                        self.dream_orch.record_dream(dream, true);
                        self.subconscious.total_dreams_fueled += 1;
                        self.vectorize_dream(&narr, &emo).await;
                    }
                }

                // Algorithmes REM (milieu de phase)
                if let Some(ref c) = self.sleep.current_cycle {
                    let dur = self.sleep.phase_duration(&SleepPhase::REM);
                    if c.phase_remaining == dur / 2 {
                        self.sleep_rem_algorithms().await;
                    }
                }

                // Chimie REM
                self.chemistry.boost(Molecule::Dopamine, 0.01);
                self.chemistry.boost(Molecule::Noradrenaline, 0.01);
                // Coeur varie (mouvements oculaires rapides)
                self.body.heart.vary_bpm(5.0);
                // Traiter les emotions refoulees
                self.subconscious.process_repressed_emotions();
            },

            SleepPhase::Hypnopompic => {
                // Subconscient se calme
                self.subconscious.activation = (self.subconscious.activation - 0.15).max(0.0);

                // Calculer la qualite prevue pour adapter le reveil
                let mut quality = self.sleep.current_cycle.as_ref()
                    .map(|c| {
                        let expected_deep = self.sleep.config().deep_duration
                            * c.total_sleep_cycles as u64;
                        c.quality_factors.compute(c.memories_consolidated, expected_deep)
                    })
                    .unwrap_or(1.0);

                // Orages magnetiques degradent la qualite de sommeil
                if self.config.fields.enabled {
                    let storm = self.em_fields.storm_intensity();
                    if storm > self.config.fields.storm_threshold {
                        let penalty = (storm - self.config.fields.storm_threshold)
                            * self.config.fields.storm_sleep_factor;
                        quality = (quality - penalty).max(0.1);
                    }
                }

                // Chimie de reveil influencee par la qualite
                if quality < 0.5 {
                    // Mauvaise nuit : cortisol residuel, peu de dopamine
                    self.chemistry.boost(Molecule::Cortisol, 0.04);
                    self.chemistry.boost(Molecule::Adrenaline, 0.01);
                } else {
                    // Bonne nuit : chimie equilibree
                    self.chemistry.boost(Molecule::Cortisol, 0.02);
                    self.chemistry.boost(Molecule::Adrenaline, 0.02);
                    self.chemistry.boost(Molecule::Serotonin, 0.02 * quality);
                }
                self.chemistry.boost(Molecule::Dopamine, 0.02 * quality);

                // Corps se reveille
                self.body.soma.energy = (self.body.soma.energy + 0.02).min(1.0);
                self.body.heart.wake_up_bpm(70.0);

                // Fatigue proportionnelle a la qualite du sommeil
                // Bonne nuit (1.0) → reset total, mauvaise nuit (0.3) → 70% fatigue reste
                self.attention_orch.partial_reset_fatigue(quality);
                self.psychology.will.decision_fatigue *= 1.0 - quality;
            },

            SleepPhase::Awake => {
                // Ne devrait pas arriver pendant sleep_tick
            },
        }

        // Decrementer le compteur de phase et gerer les transitions
        let should_transition;
        let mut next_phase_opt = None;

        {
            let cycle = self.sleep.current_cycle.as_mut().unwrap();
            cycle.phase_remaining = cycle.phase_remaining.saturating_sub(1);
            cycle.sleep_cycle_counter += 1;
            should_transition = cycle.phase_remaining == 0;
        }

        if should_transition {
            // Calculer la prochaine phase (borrow immutable)
            if let Some(ref cycle) = self.sleep.current_cycle {
                next_phase_opt = self.sleep.next_phase(cycle);
            }
        }

        if should_transition {
            if let Some(next_phase) = next_phase_opt {
                self.log(LogLevel::Debug, LogCategory::Sleep,
                    format!("Phase {} -> {}",
                        phase.as_str(), next_phase.as_str()),
                    serde_json::json!({}));

                // Si on revient en LightSleep depuis REM, c'est un nouveau cycle
                if next_phase == SleepPhase::LightSleep && phase == SleepPhase::REM {
                    if let Some(ref mut c) = self.sleep.current_cycle {
                        c.sleep_cycle_number += 1;
                    }
                }

                let duration = self.sleep.phase_duration(&next_phase);
                if let Some(ref mut c) = self.sleep.current_cycle {
                    c.phase = next_phase.clone();
                    c.phase_remaining = duration;
                }

                // Synchroniser la phase avec le DreamOrchestrator
                self.dream_orch.current_phase = next_phase;
            } else {
                // Fin du sommeil — capturer les stats avant finalize
                let (dreams_count, mem_consolidated, conn_created, nightmare_count) = {
                    let c = self.sleep.current_cycle.as_ref().unwrap();
                    (self.dream_orch.dream_journal.len(),
                     c.memories_consolidated, c.connections_created,
                     c.quality_factors.nightmare_count)
                };

                // finalize_wake_up calcule la qualite dynamique
                self.sleep.finalize_wake_up();
                self.dream_orch.wake_up();

                // Recuperer la qualite calculee depuis le dernier record
                let quality = self.sleep.sleep_history.last()
                    .map(|r| r.quality).unwrap_or(1.0);

                // Sauvegarder le record de sommeil en DB
                if let Some(ref db) = self.db {
                    if let Some(record) = self.sleep.sleep_history.last() {
                        let _ = db.save_sleep_record(record).await;
                    }
                }

                self.log(LogLevel::Info, LogCategory::Sleep,
                    format!("Reveil complet — qualite {:.0}%, {} reves, {} cauchemars, {} souvenirs consolides",
                        quality * 100.0, dreams_count, nightmare_count, mem_consolidated),
                    serde_json::json!({
                        "total_sleeps": self.sleep.total_complete_sleeps,
                        "quality": quality,
                        "dreams_count": dreams_count,
                        "nightmare_count": nightmare_count,
                        "memories_consolidated": mem_consolidated,
                        "connections_created": conn_created,
                        "sleep_debt": self.sleep.drive.sleep_debt,
                        "interrupted": false,
                    }));

                // Broadcast reveil
                self.broadcast_wake_up(quality, dreams_count, mem_consolidated, conn_created);
            }
        }

        // Broadcast l'etat du sommeil
        self.broadcast_sleep_state().await;
    }

    /// Initie le processus d'endormissement.
    pub async fn initiate_sleep(&mut self) {
        if !self.config.sleep.enabled { return; }

        let pressure = self.sleep.drive.sleep_pressure;
        let forced = self.sleep.drive.sleep_forced;
        let awake_cycles = self.sleep.drive.awake_cycles;
        let energy = self.body.soma.energy;

        self.sleep.initiate();

        let planned_cycles = self.sleep.current_cycle.as_ref()
            .map(|c| c.total_sleep_cycles).unwrap_or(1);

        // Synchroniser la phase avec le DreamOrchestrator
        self.dream_orch.current_phase = SleepPhase::Hypnagogic;

        // Log enrichi
        if forced {
            self.log(LogLevel::Warn, LogCategory::Sleep,
                format!("Sommeil force — pression {:.0}%", pressure * 100.0),
                serde_json::json!({
                    "pressure": pressure,
                    "awake_cycles": awake_cycles,
                }));
        } else {
            self.log(LogLevel::Info, LogCategory::Sleep,
                format!("Endormissement (pression: {:.0}%, {} cycles prevus)",
                    pressure * 100.0, planned_cycles),
                serde_json::json!({
                    "sleep_pressure": pressure,
                    "awake_cycles": awake_cycles,
                    "planned_sleep_cycles": planned_cycles,
                    "energy_at_sleep": energy,
                    "forced": false,
                }));
        }

        // Broadcast debut de sommeil
        self.broadcast_sleep_started();
        self.broadcast_sleep_state().await;
    }

    /// Met a jour la pression de sommeil et le subconscient de fond.
    /// Appele depuis la boucle principale quand Saphire est eveillee.
    pub async fn update_sleep_pressure(&mut self) {
        let energy = self.body.soma.energy;
        let attn_fatigue = self.attention_orch.fatigue();
        let dec_fatigue = self.psychology.will.decision_fatigue;
        let cortisol = self.chemistry.cortisol;
        let adrenaline = self.chemistry.adrenaline;
        let in_conv = self.in_conversation;
        self.sleep.drive.update(energy, attn_fatigue, dec_fatigue, cortisol, adrenaline, in_conv);

        // Subconscient travaille en arriere-plan meme eveillee
        self.subconscious.background_process(0.0, "eveil");

        // Analyses algorithmiques periodiques du subconscient
        if self.orchestrator.enabled && self.cycle_count > 0 {
            if self.cycle_count % 100 == 0 {
                self.subconscious_dbscan().await;
            }
            if (self.cycle_count + 50) % 100 == 0 {
                self.subconscious_isolation_forest().await;
            }
        }

        // Detecter et broadcaster les insights du subconscient
        if let Some(insight) = self.subconscious.surface_insight() {
            self.log(LogLevel::Info, LogCategory::Subconscious,
                format!("Insight emerge : {}", insight.content),
                serde_json::json!({
                    "source": insight.source_type,
                    "strength": insight.strength,
                    "emotional_charge": insight.emotional_charge,
                    "total_surfaced": self.subconscious.total_insights_surfaced,
                }));
            self.broadcast_subconscious_insight(
                &insight.content, &insight.source_type, insight.strength);
            // Vectoriser l'insight
            self.vectorize_insight(&insight.content, &insight.source_type).await;
        }
    }

    /// Verifie si Saphire devrait s'endormir (pression suffisante + pas en conversation).
    pub fn should_initiate_sleep(&self) -> bool {
        self.config.sleep.enabled
            && self.sleep.drive.should_sleep()
            && !self.in_conversation
    }
}
