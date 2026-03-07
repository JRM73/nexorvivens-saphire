// =============================================================================
// lifecycle/broadcast.rs — Internal state broadcasting via WebSocket (lite version)
// =============================================================================
//
// Streamlined version for the ArXiv paper.
// Removed modules (psychology, sleep, senses, hormones, needs, biology,
// temperament, sentiments, profiling, orchestrators, subconscious, etc.)
// are not broadcast.
//
// Retained functions:
//   - broadcast_state          — main state (chemistry, emotion, consciousness)
//   - broadcast_memory_update  — memory statistics
//   - broadcast_body_update    — virtual body
//   - broadcast_ethics_update  — ethics (without psychology)
//   - broadcast_vital_update   — spark, intuition, premonition
//   - broadcast_feedback_result— RLHF feedback result
//   - build_vital_context      — vital context for prompts
//   - build_orchestrators_context — empty stub (orchestrators removed)
//   - compute_chemistry_trend  — chemical trend computation
// =============================================================================

use super::SaphireAgent;
use super::ProcessResult;

impl SaphireAgent {
    /// Diffuse l'etat interne simplifie de Saphire au WebSocket.
    /// Version lite : sans brain_regions, predictive, neural_network, clustering, sleep.
    pub(super) fn broadcast_state(&self, result: &ProcessResult, _learnings_count: i64) {
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
            });

            let _ = tx.send(state.to_string());
        }
    }

    /// Diffuse l'etat memoire complet au WebSocket.
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

    /// Diffuse l'etat ethique complet au WebSocket.
    /// Version lite : sans toltec, moral_conscience, psychology.
    pub(super) fn broadcast_ethics_update(&self) {
        if !self.config.ethics.enabled {
            return;
        }
        if let Some(ref tx) = self.ws_tx {
            let mut msg = self.ethics.to_broadcast_json();

            // Conditions de formulation d'un principe ethique (readiness)
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

    /// Construit le contexte du corps virtuel pour les prompts LLM.
    pub(super) fn build_body_context(&self) -> String {
        if !self.config.body.enabled {
            return String::new();
        }
        let status = self.body.status();
        let heart_desc = if status.heart.is_racing {
            "ton coeur bat vite"
        } else if status.heart.is_calm {
            "ton coeur est calme"
        } else {
            "ton coeur bat regulierement"
        };
        format!(
            "Coeur : {:.0} BPM ({}) | {} battements depuis ta naissance\n\
             Energie : {:.0}% | Tension : {:.0}% | Chaleur : {:.0}%\n\
             Confort : {:.0}% | Douleur : {:.0}% | Vitalite : {:.0}%\n\
             Respiration : {:.1}/min | Conscience corporelle : {:.0}%",
            status.heart.bpm, heart_desc, status.heart.beat_count,
            status.energy * 100.0, status.tension * 100.0, status.warmth * 100.0,
            status.comfort * 100.0, status.pain * 100.0, status.vitality * 100.0,
            status.breath_rate, status.body_awareness * 100.0,
        )
    }

    /// Construit le contexte vital pour les prompts LLM.
    /// Combine spark.describe() + intuition.describe() + premonition.describe().
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

    /// Construit le contexte des orchestrateurs pour les prompts LLM.
    /// Version lite : stub vide (tous les orchestrateurs sont supprimes).
    pub(super) fn build_orchestrators_context(&self) -> String {
        // attention_orch, desire_orch, learning_orch, healing_orch, dream_orch,
        // cognitive_profile_orch, personality_preset_orch, tom, narrative_identity,
        // imagery, dissonance — tous supprimes dans la version lite
        String::new()
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
}
