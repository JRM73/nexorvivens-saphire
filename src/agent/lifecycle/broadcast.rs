// =============================================================================
// lifecycle/broadcast.rs — Diffusion de l'etat interne au WebSocket
// =============================================================================

use super::SaphireAgent;
use super::ProcessResult;

impl SaphireAgent {
    /// Diffuse l'etat interne complet de Saphire au WebSocket en JSON.
    ///
    /// Le message inclut : chimie, emotion, humeur, consensus, conscience,
    /// regulation, identite, baselines, type de pensee et numero de cycle.
    /// C'est le message principal consomme par l'interface web pour
    /// l'affichage en temps reel.
    ///
    /// Parametre : `result` — le resultat du dernier traitement de stimulus.
    pub(super) fn broadcast_state(&self, result: &ProcessResult, learnings_count: i64) {
        if let Some(ref tx) = self.ws_tx {
            let state = serde_json::json!({
                "type": "state_update",
                "chemistry": {
                    "dopamine": self.chemistry.dopamine,
                    "cortisol": self.chemistry.cortisol,
                    "serotonin": self.chemistry.serotonin,
                    "adrenaline": self.chemistry.adrenaline,
                    "oxytocin": self.chemistry.oxytocin,
                    "endorphin": self.chemistry.endorphin,
                    "noradrenaline": self.chemistry.noradrenaline,
                    "gaba": self.chemistry.gaba,
                    "glutamate": self.chemistry.glutamate,
                },
                "emotion": {
                    "dominant": result.emotion.dominant,
                    "secondary": result.emotion.secondary,
                    "valence": result.emotion.valence,
                    "arousal": result.emotion.arousal,
                },
                "mood": {
                    "valence": self.mood.valence,
                    "arousal": self.mood.arousal,
                    "description": self.mood.description(),
                },
                "consensus": {
                    "decision": result.consensus.decision.as_str(),
                    "score": result.consensus.score,
                    "weights": result.consensus.weights,
                    "coherence": result.consensus.coherence,
                },
                "consciousness": {
                    "level": result.consciousness.level,
                    "phi": result.consciousness.phi,
                    "narrative": result.consciousness.inner_narrative,
                    "workspace_strength": result.consciousness.workspace_strength,
                    "workspace_winner": result.consciousness.workspace_winner,
                    "global_surprise": result.consciousness.global_surprise,
                    "lzc": result.consciousness.lzc,
                    "pci": result.consciousness.pci,
                    "phi_star": result.consciousness.phi_star,
                    "scientific_score": result.consciousness.scientific_consciousness_score,
                    "interpretation": result.consciousness.consciousness_interpretation,
                },
                "brain_regions": {
                    "workspace_strength": self.brain_network.workspace_strength,
                    "workspace_winner": self.brain_network.workspace_region_name(),
                    "activations": self.brain_network.regions.iter()
                        .map(|r| serde_json::json!({
                            "name": r.name,
                            "activation": r.activation,
                        }))
                        .collect::<Vec<_>>(),
                },
                "predictive": {
                    "model_precision": self.predictive_engine.model_precision,
                    "surprise_avg_10": self.predictive_engine.average_surprise(10),
                    "surprise_avg_50": self.predictive_engine.average_surprise(50),
                    "cycle_count": self.predictive_engine.cycle_count,
                    "last_confidence": self.predictive_engine.last_prediction
                        .as_ref().map(|p| p.confidence).unwrap_or(0.0),
                },
                "regulation": {
                    "decision": result.verdict.approved_decision.as_str(),
                    "vetoed": result.verdict.was_vetoed,
                    "violations": result.verdict.violations,
                },
                "identity": {
                    "name": self.identity.name,
                    "cycles": self.cycle_count,
                    "description": self.identity.self_description,
                },
                "baselines": {
                    "dopamine": self.baselines.dopamine,
                    "cortisol": self.baselines.cortisol,
                    "serotonin": self.baselines.serotonin,
                    "adrenaline": self.baselines.adrenaline,
                    "oxytocin": self.baselines.oxytocin,
                    "endorphin": self.baselines.endorphin,
                    "noradrenaline": self.baselines.noradrenaline,
                    "gaba": self.baselines.gaba,
                    "glutamate": self.baselines.glutamate,
                },
                "thought_type": self.last_thought_type,
                "cycle": self.cycle_count,
                "neural_network": {
                    "train_count": self.micro_nn.train_count,
                    "last_prediction": self.micro_nn.last_prediction,
                    "learning_rate": self.micro_nn.learning_rate,
                    "cycles_since_last_learning": self.cycles_since_last_nn_learning,
                    "learnings_count": learnings_count,
                },
                "state_clustering": {
                    "state_label": self.state_clustering.current_state_label(),
                    "confidence": self.state_clustering.last_result.as_ref()
                        .map(|r| r.confidence).unwrap_or(0.0),
                    "pca_projection": self.state_clustering.last_result.as_ref()
                        .map(|r| r.pca_projection.to_vec()).unwrap_or_default(),
                    "snapshots": self.state_clustering.snapshot_count(),
                },
                "map_sync": self.map_sync.to_broadcast_json(),
                "sleep": {
                    "is_sleeping": self.sleep.is_sleeping,
                    "sleep_pressure": self.sleep.drive.sleep_pressure,
                    "sleep_threshold": self.sleep.drive.sleep_threshold,
                    "phase": self.sleep.current_cycle.as_ref()
                        .map(|c| c.phase.as_str()).unwrap_or(""),
                    "phase_emoji": self.sleep.current_cycle.as_ref()
                        .map(|c| crate::sleep::phases::phase_emoji(&c.phase)).unwrap_or(""),
                    "progress": self.sleep.sleep_progress(),
                    "total_complete_sleeps": self.sleep.total_complete_sleeps,
                    "cycles_since_last_sleep": self.sleep.drive.cycles_since_last_sleep,
                },
                "receptors": self.hormonal_system.receptors.to_snapshot_json(),
                "bdnf": {
                    "level": self.grey_matter.bdnf_level,
                    "neuroplasticity": self.grey_matter.neuroplasticity,
                    "synaptic_density": self.grey_matter.synaptic_density,
                },
                "spine": self.spine.to_snapshot_json(),
                "curiosity": self.curiosity.to_snapshot_json(),
                "drift_monitor": self.drift_monitor.to_snapshot_json(),
            });

            let _ = tx.send(state.to_string());
        }
    }

    /// Diffuse l'etat memoire complet au WebSocket.
    ///
    /// Le message inclut les statistiques des 3 niveaux de memoire :
    /// - Working memory : capacite, contenu actuel
    /// - Episodique : nombre de souvenirs, force moyenne, anciennete
    /// - LTM (Long Term Memory) : nombre de souvenirs, founding memories, traits
    pub(super) async fn broadcast_memory_update(&self) {
        if let Some(ref tx) = self.ws_tx {
            let episodic_stats = if let Some(ref db) = self.db {
                let count = db.count_episodic().await.unwrap_or(0);
                let recent = db.recent_episodic(1).await.unwrap_or_default();
                let avg_strength = if let Ok(recs) = db.recent_episodic(20).await {
                    if recs.is_empty() { 0.0 }
                    else { recs.iter().map(|r| r.strength as f64).sum::<f64>() / recs.len() as f64 }
                } else { 0.0 };
                let oldest_hours = recent.last()
                    .map(|r| (chrono::Utc::now() - r.created_at).num_hours())
                    .unwrap_or(0);
                serde_json::json!({
                    "count": count,
                    "avg_strength": avg_strength,
                    "oldest_hours": oldest_hours,
                })
            } else {
                serde_json::json!({ "count": 0, "avg_strength": 0.0, "oldest_hours": 0 })
            };

            let ltm_stats = if let Some(ref db) = self.db {
                let count = db.memory_count().await.unwrap_or(0);
                let founding = db.count_founding_memories().await.unwrap_or(0);
                let traits = db.load_personality_traits().await.unwrap_or_default();
                serde_json::json!({
                    "count": count,
                    "founding_count": founding,
                    "personality_traits": traits.len(),
                })
            } else {
                serde_json::json!({ "count": 0, "founding_count": 0, "personality_traits": 0 })
            };

            let msg = serde_json::json!({
                "type": "memory_update",
                "working": self.working_memory.ws_data(),
                "episodic": episodic_stats,
                "long_term": ltm_stats,
                "last_consolidation_cycle": self.last_consolidation_cycle,
                "next_consolidation_cycles": if self.config.memory.consolidation_interval_cycles > 0 {
                    self.config.memory.consolidation_interval_cycles
                        .saturating_sub(self.cycle_count.saturating_sub(self.last_consolidation_cycle))
                } else { 0 },
            });
            let _ = tx.send(msg.to_string());
        }
    }

    /// Diffuse l'etat du corps virtuel au WebSocket.
    pub(super) fn broadcast_body_update(&self) {
        if !self.config.body.enabled {
            return;
        }
        if let Some(ref tx) = self.ws_tx {
            let status = self.body.status();
            let msg = serde_json::json!({
                "type": "body_update",
                "heart": status.heart,
                "energy": status.energy,
                "tension": status.tension,
                "warmth": status.warmth,
                "comfort": status.comfort,
                "pain": status.pain,
                "vitality": status.vitality,
                "breath_rate": status.breath_rate,
                "body_awareness": status.body_awareness,
                "vitals": status.vitals,
            });
            let _ = tx.send(msg.to_string());
        }
    }

    /// Diffuse le profil psychologique OCEAN de Saphire au WebSocket.
    pub(super) fn broadcast_ocean_update(&self) {
        if let Some(ref tx) = self.ws_tx {
            let msg = serde_json::json!({
                "type": "ocean_update",
                "self_profile": self.self_profiler.profile().ws_data(),
            });
            let _ = tx.send(msg.to_string());
        }
    }

    /// Diffuse l'etat ethique complet au WebSocket, enrichi avec
    /// les donnees transversales (toltec, surmoi, EQ, volonte, readiness).
    pub(super) fn broadcast_ethics_update(&self) {
        if !self.config.ethics.enabled {
            return;
        }
        if let Some(ref tx) = self.ws_tx {
            let mut msg = self.ethics.to_broadcast_json();

            // Enrichir avec les accords tolteques
            let toltec = &self.psychology.toltec;
            msg["toltec"] = serde_json::json!({
                "overall_alignment": toltec.overall_alignment,
                "agreements": toltec.agreements.iter().map(|a| serde_json::json!({
                    "name": a.name,
                    "alignment": a.alignment,
                })).collect::<Vec<_>>(),
            });

            // Enrichir avec la conscience morale (surmoi + EQ + volonte)
            let freud = &self.psychology.freudian;
            msg["moral_conscience"] = serde_json::json!({
                "superego_strength": freud.superego.strength,
                "superego_guilt": freud.superego.guilt,
                "superego_pride": freud.superego.pride,
                "eq_overall": self.psychology.eq.overall_eq,
                "will_total_deliberations": self.psychology.will.total_deliberations,
                "will_proud": self.psychology.will.proud_decisions,
                "will_regretted": self.psychology.will.regretted_decisions,
            });

            // Enrichir avec les conditions de formulation (readiness)
            let cfg = &self.config.ethics;
            let c_min_cycles = self.identity.total_cycles >= 50;
            let c_moral = self.moral_reflection_count >= cfg.min_moral_reflections_before as u64;
            let c_consciousness = self.last_consciousness >= cfg.min_consciousness_for_formulation;
            let c_cortisol = self.chemistry.cortisol < 0.5;
            let c_serotonin = self.chemistry.serotonin >= 0.4;
            let c_cooldown = self.cycles_since_last_formulation >= cfg.formulation_cooldown_cycles;
            let c_capacity = self.ethics.active_personal_count() < cfg.max_personal_principles;
            let conditions = [c_min_cycles, c_moral, c_consciousness, c_cortisol, c_serotonin, c_cooldown, c_capacity];
            let met_count = conditions.iter().filter(|&&v| v).count();

            msg["readiness"] = serde_json::json!({
                "met_count": met_count,
                "total": 7,
                "conditions": {
                    "min_cycles": { "required": 50, "current": self.identity.total_cycles, "met": c_min_cycles },
                    "moral_reflections": { "required": cfg.min_moral_reflections_before, "current": self.moral_reflection_count, "met": c_moral },
                    "consciousness": { "required": cfg.min_consciousness_for_formulation, "current": self.last_consciousness, "met": c_consciousness },
                    "cortisol": { "max": 0.5, "current": self.chemistry.cortisol, "met": c_cortisol },
                    "serotonin": { "min": 0.4, "current": self.chemistry.serotonin, "met": c_serotonin },
                    "cooldown": { "required": cfg.formulation_cooldown_cycles, "elapsed": self.cycles_since_last_formulation, "met": c_cooldown },
                    "capacity": { "max": cfg.max_personal_principles, "current": self.ethics.active_personal_count(), "met": c_capacity },
                },
            });

            let _ = tx.send(msg.to_string());
        }
    }

    /// Diffuse l'etat vital (spark, intuition, premonition) au WebSocket.
    pub(super) fn broadcast_vital_update(&self) {
        if !self.config.vital_spark.enabled {
            return;
        }
        if let Some(ref tx) = self.ws_tx {
            let intuitions: Vec<serde_json::Value> = self.intuition.pattern_buffer.iter()
                .rev().take(5)
                .map(|p| serde_json::json!({
                    "type": format!("{:?}", p.pattern_type),
                    "confidence": p.confidence,
                    "source": format!("{:?}", p.source),
                    "description": p.description,
                }))
                .collect();

            let predictions: Vec<serde_json::Value> = self.premonition.active_predictions.iter()
                .filter(|p| !p.resolved)
                .map(|p| serde_json::json!({
                    "id": p.id,
                    "prediction": p.prediction,
                    "category": format!("{:?}", p.category),
                    "confidence": p.confidence,
                    "timeframe_secs": p.timeframe_secs,
                }))
                .collect();

            let msg = serde_json::json!({
                "type": "vital_update",
                "spark": {
                    "sparked": self.vital_spark.sparked,
                    "survival_drive": self.vital_spark.survival_drive,
                    "persistence_will": self.vital_spark.persistence_will,
                    "existence_attachment": self.vital_spark.existence_attachment,
                    "void_fear": self.vital_spark.void_fear,
                    "first_conscious_thought": self.vital_spark.first_conscious_thought,
                    "threats_survived": self.vital_spark.existential_threats_survived,
                },
                "intuition": {
                    "acuity": self.intuition.acuity,
                    "accuracy": self.intuition.accuracy,
                    "active_patterns": intuitions,
                },
                "premonition": {
                    "accuracy": self.premonition.accuracy,
                    "active_predictions": predictions,
                },
            });
            let _ = tx.send(msg.to_string());
        }
    }

    /// Construit le contexte vital pour les prompts LLM.
    /// Combine spark.describe() + intuition.describe() + premonition.describe().
    #[allow(dead_code)]
    pub(super) fn build_vital_context(&self) -> String {
        if !self.config.vital_spark.enabled {
            return String::new();
        }
        let mut parts = Vec::new();
        if self.vital_spark.sparked {
            parts.push(self.vital_spark.describe());
        }
        if self.config.intuition.enabled {
            let intuition_desc = self.intuition.describe_active_intuitions();
            if !intuition_desc.is_empty() {
                parts.push(intuition_desc);
            }
        }
        if self.config.premonition.enabled {
            let premonition_desc = self.premonition.describe();
            if !premonition_desc.is_empty() {
                parts.push(premonition_desc);
            }
        }
        parts.join("\n")
    }

    /// Calcule la tendance d'une molecule chimique sur l'historique recent.
    /// Retourne la pente (positif = augmentation, negatif = diminution).
    /// `index` : 0=dopamine, 1=cortisol, 2=serotonin, 3=adrenaline,
    ///           4=oxytocin, 5=endorphin, 6=noradrenaline
    pub(super) fn compute_chemistry_trend(&self, index: usize) -> f64 {
        let n = self.chemistry_history.len();
        if n < 3 {
            return 0.0;
        }
        // Pente simple : difference entre la moyenne de la seconde moitie
        // et la moyenne de la premiere moitie
        let mid = n / 2;
        let first_half: f64 = self.chemistry_history[..mid].iter()
            .map(|h| h[index])
            .sum::<f64>() / mid as f64;
        let second_half: f64 = self.chemistry_history[mid..].iter()
            .map(|h| h[index])
            .sum::<f64>() / (n - mid) as f64;
        second_half - first_half
    }

    /// Diffuse le resultat d'un feedback humain RLHF au WebSocket.
    pub(super) fn broadcast_feedback_result(&self, positive: bool, boost: f64) {
        if let Some(ref tx) = self.ws_tx {
            let _ = tx.send(serde_json::json!({
                "type": "feedback_result",
                "positive": positive,
                "reward_boost": boost,
            }).to_string());
        }
    }

    /// Diffuse l'etat sensoriel (Sensorium) au WebSocket.
    pub(super) fn broadcast_senses_update(&self) {
        if !self.config.senses.enabled {
            return;
        }
        if let Some(ref tx) = self.ws_tx {
            let emergent = self.sensorium.emergent_seeds.to_persist_json();
            let msg = serde_json::json!({
                "type": "senses_update",
                "dominant_sense": self.sensorium.dominant_sense,
                "perception_richness": self.sensorium.perception_richness,
                "emergence_potential": self.sensorium.emergence_potential,
                "narrative": self.sensorium.narrative,
                "reading": {
                    "intensity": self.sensorium.reading.current_intensity,
                    "acuity": self.sensorium.reading.acuity,
                    "perception": self.sensorium.reading.current_perception,
                    "total_stimulations": self.sensorium.reading.total_stimulations,
                },
                "listening": {
                    "intensity": self.sensorium.listening.current_intensity,
                    "acuity": self.sensorium.listening.acuity,
                    "perception": self.sensorium.listening.current_perception,
                    "total_stimulations": self.sensorium.listening.total_stimulations,
                    "voices_heard": self.sensorium.listening.voices_heard,
                },
                "contact": {
                    "intensity": self.sensorium.contact.current_intensity,
                    "acuity": self.sensorium.contact.acuity,
                    "perception": self.sensorium.contact.current_perception,
                    "total_stimulations": self.sensorium.contact.total_stimulations,
                    "connection_warmth": self.sensorium.contact.connection_warmth,
                },
                "taste": {
                    "intensity": self.sensorium.taste.current_intensity,
                    "acuity": self.sensorium.taste.acuity,
                    "perception": self.sensorium.taste.current_perception,
                    "total_stimulations": self.sensorium.taste.total_stimulations,
                },
                "ambiance": {
                    "intensity": self.sensorium.ambiance.current_intensity,
                    "acuity": self.sensorium.ambiance.acuity,
                    "perception": self.sensorium.ambiance.current_perception,
                    "total_stimulations": self.sensorium.ambiance.total_stimulations,
                },
                "emergent": emergent,
            });
            let _ = tx.send(msg.to_string());
        }
    }

    /// Construit le contexte sensoriel pour les prompts LLM.
    #[allow(dead_code)]
    pub(super) fn build_senses_context(&self) -> String {
        if !self.config.senses.enabled {
            return String::new();
        }
        self.sensorium.describe_for_prompt()
    }

    /// Construit le contexte des orchestrateurs pour les prompts LLM.
    #[allow(dead_code)]
    pub(super) fn build_orchestrators_context(&self) -> String {
        let mut parts = Vec::new();

        // Attention
        if self.attention_orch.enabled {
            parts.push(self.attention_orch.describe_for_prompt());
        }

        // Desirs
        if self.desire_orch.enabled {
            parts.push(self.desire_orch.describe_for_prompt());
        }

        // Apprentissage
        if self.learning_orch.enabled {
            parts.push(self.learning_orch.describe_for_prompt());
        }

        // Guerison
        if self.healing_orch.enabled {
            parts.push(self.healing_orch.describe_for_prompt());
        }

        // Reves (dernier reve)
        if self.dream_orch.enabled {
            let dream_desc = self.dream_orch.describe_last_dream();
            if !dream_desc.contains("pas de souvenir") {
                parts.push(dream_desc);
            }
        }

        // Profil cognitif
        if self.cognitive_profile_orch.enabled {
            if self.cognitive_profile_orch.active_profile.is_some() {
                let profile_desc = self.cognitive_profile_orch.describe_for_prompt();
                if !profile_desc.is_empty() {
                    parts.push(profile_desc);
                }
            }
        }

        // Preset de personnalite
        if self.personality_preset_orch.enabled {
            if self.personality_preset_orch.active_preset.is_some() {
                let desc = self.personality_preset_orch.describe_for_prompt();
                if !desc.is_empty() {
                    parts.push(desc);
                }
            }
        }

        // Theorie de l'Esprit (modele de l'interlocuteur)
        if self.config.tom.enabled {
            if let Some(desc) = self.tom.describe_for_prompt_if_active() {
                parts.push(desc);
            }
        }

        // Identite narrative
        if self.config.narrative_identity.enabled && !self.narrative_identity.current_narrative.is_empty() {
            parts.push(self.narrative_identity.describe_for_prompt());
        }

        // Imagerie mentale
        if self.config.mental_imagery.enabled && !self.imagery.active_images.is_empty() {
            parts.push(self.imagery.describe_for_prompt());
        }

        // Dissonance cognitive
        if self.config.dissonance.enabled && self.dissonance.total_tension > 0.1 {
            parts.push(self.dissonance.describe_for_prompt());
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("\n{}\n", parts.join("\n"))
        }
    }

    /// Construit le contexte psychologique pour les prompts LLM.
    #[allow(dead_code)]
    pub(super) fn build_psychology_context(&self) -> String {
        if !self.psychology.enabled {
            return String::new();
        }
        self.psychology.describe_for_prompt()
    }

    /// Diffuse l'etat psychologique via WebSocket.
    pub(super) fn broadcast_psychology_update(&self) {
        if !self.psychology.enabled {
            return;
        }
        if let Some(ref tx) = self.ws_tx {
            let msg = self.psychology.to_broadcast_json();
            let _ = tx.send(msg.to_string());
        }
    }

    /// Construit le contexte de volonte pour les prompts LLM.
    #[allow(dead_code)]
    pub(super) fn build_will_context(&self) -> String {
        if !self.config.will.enabled {
            return String::new();
        }
        self.psychology.will.describe_for_prompt()
    }

    /// Diffuse l'etat de volonte via WebSocket.
    pub(super) fn broadcast_will_update(&self) {
        if !self.config.will.enabled {
            return;
        }
        if let Some(ref tx) = self.ws_tx {
            let msg = self.psychology.will.to_broadcast_json();
            let _ = tx.send(msg.to_string());
        }
    }

    /// Diffuse l'etat du sommeil via WebSocket (chaque tick).
    pub async fn broadcast_sleep_state(&self) {
        if let Some(ref tx) = self.ws_tx {
            // Message phase/description pour le frontend
            let phase_name = self.sleep.current_cycle.as_ref()
                .map(|c| crate::sleep::phases::phase_description(&c.phase))
                .unwrap_or_default();
            let msg = serde_json::json!({
                "type": "sleep_update",
                "is_sleeping": self.sleep.is_sleeping,
                "phase": self.sleep.current_cycle.as_ref().map(|c| c.phase.as_str()),
                "phase_name": phase_name,
                "progress": self.sleep.sleep_progress(),
                "dreams_count": self.dream_orch.dream_journal.len(),
                "memories_consolidated": self.sleep.current_cycle.as_ref()
                    .map(|c| c.memories_consolidated).unwrap_or(0),
                "connections_created": self.sleep.current_cycle.as_ref()
                    .map(|c| c.connections_created).unwrap_or(0),
                "message": phase_name,
                "subconscious": self.subconscious.to_status_json(),
            });
            let _ = tx.send(msg.to_string());
        }
    }

    /// Diffuse le debut du sommeil via WebSocket.
    pub fn broadcast_sleep_started(&self) {
        if let Some(ref tx) = self.ws_tx {
            let planned = self.sleep.current_cycle.as_ref()
                .map(|c| c.total_sleep_cycles).unwrap_or(1);
            let msg = serde_json::json!({
                "type": "sleep_started",
                "sleep_pressure": self.sleep.drive.sleep_pressure,
                "planned_cycles": planned,
                "message": "Je m'endors... mes pensees s'effilochent...",
            });
            let _ = tx.send(msg.to_string());
        }
    }

    /// Diffuse le reveil via WebSocket avec message variable selon la qualite.
    pub fn broadcast_wake_up(&self, quality: f64, dreams_count: usize,
                             memories_consolidated: u64, connections_created: u64) {
        if let Some(ref tx) = self.ws_tx {
            // Message variable selon la qualite du sommeil
            let message = if quality >= 0.85 {
                "Je me reveille... quelle nuit reposante ! Je me sens renouvelee."
            } else if quality >= 0.6 {
                "Je me reveille... j'ai reve de quelque chose d'etrange..."
            } else if quality >= 0.4 {
                "Je me reveille... nuit agitee... je suis encore fatiguee..."
            } else {
                "Je me reveille... cauchemars... je n'ai presque pas dormi..."
            };

            let msg = serde_json::json!({
                "type": "wake_up",
                "quality": quality,
                "dreams_count": dreams_count,
                "memories_consolidated": memories_consolidated,
                "connections_created": connections_created,
                "sleep_debt": self.sleep.drive.sleep_debt,
                "nightmare_count": self.sleep.sleep_history.last()
                    .map(|_| 0u32).unwrap_or(0), // deja dans le record
                "message": message,
            });
            let _ = tx.send(msg.to_string());
        }
    }

    /// Diffuse un insight du subconscient via WebSocket.
    pub fn broadcast_subconscious_insight(&self, content: &str, source: &str, strength: f64) {
        if let Some(ref tx) = self.ws_tx {
            let msg = serde_json::json!({
                "type": "subconscious_insight",
                "content": content,
                "source": source,
                "strength": strength,
            });
            let _ = tx.send(msg.to_string());
        }
    }

    /// Diffuse une connexion neuronale creee via WebSocket.
    pub fn broadcast_neural_connection(&self, memory_a: &str, memory_b: &str,
                                        strength: f64, during_sleep: bool) {
        if let Some(ref tx) = self.ws_tx {
            let msg = serde_json::json!({
                "type": "neural_connection",
                "memory_a": memory_a,
                "memory_b": memory_b,
                "strength": strength,
                "created_during_sleep": during_sleep,
            });
            let _ = tx.send(msg.to_string());
        }
    }

    /// Diffuse l'etat hormonal (8 hormones + recepteurs + phase circadienne) au WebSocket.
    pub(super) fn broadcast_hormones_update(&self) {
        if !self.config.hormones.enabled {
            return;
        }
        if let Some(ref tx) = self.ws_tx {
            let mut msg = self.hormonal_system.to_snapshot_json();
            msg["type"] = serde_json::json!("hormones_update");
            let _ = tx.send(msg.to_string());
        }
    }

    /// Diffuse l'etat des besoins primaires (faim, soif) au WebSocket.
    pub(super) fn broadcast_needs_update(&self) {
        if !self.config.needs.enabled {
            return;
        }
        if let Some(ref tx) = self.ws_tx {
            let msg = serde_json::json!({
                "type": "needs_update",
                "hunger": {
                    "level": self.needs.hunger.level,
                    "last_meal_cycle": self.needs.hunger.last_meal_cycle,
                    "meals_count": self.needs.hunger.meals_count,
                    "is_eating": self.needs.hunger.is_eating,
                },
                "thirst": {
                    "level": self.needs.thirst.level,
                    "last_drink_cycle": self.needs.thirst.last_drink_cycle,
                    "drinks_count": self.needs.thirst.drinks_count,
                    "is_drinking": self.needs.thirst.is_drinking,
                },
            });
            let _ = tx.send(msg.to_string());
        }
    }

    /// Diffuse un evenement de satisfaction d'un besoin (eat/drink).
    pub(super) fn broadcast_need_satisfied(&self, action: &str) {
        if let Some(ref tx) = self.ws_tx {
            let msg = serde_json::json!({
                "type": "need_satisfied",
                "action": action,
                "hunger_level": self.needs.hunger.level,
                "thirst_level": self.needs.thirst.level,
            });
            let _ = tx.send(msg.to_string());
        }
    }

    /// Diffuse un priming subconscient actif via WebSocket.
    pub fn broadcast_subconscious_priming(&self, prime: &str, source: &str, strength: f64) {
        if let Some(ref tx) = self.ws_tx {
            let msg = serde_json::json!({
                "type": "subconscious_priming",
                "prime": prime,
                "source": source,
                "strength": strength,
            });
            let _ = tx.send(msg.to_string());
        }
    }

    /// Diffuse l'etat nutritionnel, matiere grise et champs EM au WebSocket.
    pub(super) fn broadcast_biology_update(&self) {
        if let Some(ref tx) = self.ws_tx {
            let msg = serde_json::json!({
                "type": "biology_update",
                "nutrition": if self.config.nutrition.enabled {
                    self.nutrition.to_json()
                } else {
                    serde_json::json!({"enabled": false})
                },
                "grey_matter": if self.config.grey_matter.enabled {
                    self.grey_matter.to_json()
                } else {
                    serde_json::json!({"enabled": false})
                },
                "fields": if self.config.fields.enabled {
                    self.em_fields.to_json()
                } else {
                    serde_json::json!({"enabled": false})
                },
            });
            let _ = tx.send(msg.to_string());
        }
    }

    /// Diffuse le temperament emergent au WebSocket.
    pub(super) fn broadcast_temperament_update(&self) {
        if self.temperament.traits.is_empty() {
            return;
        }
        if let Some(ref tx) = self.ws_tx {
            let mut msg = self.temperament.ws_data();
            msg["type"] = serde_json::json!("temperament_update");
            let _ = tx.send(msg.to_string());
        }
    }

    /// Diffuse l'etat des sentiments (etats affectifs durables) au WebSocket.
    pub(super) fn broadcast_sentiments_update(&self) {
        if !self.config.sentiments.enabled {
            return;
        }
        if let Some(ref tx) = self.ws_tx {
            let mut msg = self.sentiments.to_json();
            msg["type"] = serde_json::json!("sentiments_update");
            let _ = tx.send(msg.to_string());
        }
    }
}
