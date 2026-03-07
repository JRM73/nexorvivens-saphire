// =============================================================================
// lifecycle/thinking_perception.rs — Phases pre-LLM (perception du monde)
// =============================================================================
//
// Ce fichier contient les phases de perception et de mise a jour de l'etat
// interne de Saphire avant l'appel au LLM. Cela inclut :
//   - Initialisation du cycle
//   - Meteo + corps virtuel
//   - VitalSpark
//   - Historique chimique
//   - Anniversaire
//   - Broadcast world_update
//   - Decay memoire (travail, episodique)
//   - Consolidation memoire
// =============================================================================

use crate::neurochemistry::Molecule;
use crate::logging::{LogLevel, LogCategory};
use crate::memory::consolidation;

use super::SaphireAgent;
use super::thinking::ThinkingContext;

impl SaphireAgent {
    // =========================================================================
    // Phase 1 : Initialisation du cycle
    // =========================================================================

    pub(super) fn phase_init(&mut self, _ctx: &mut ThinkingContext) {
        // Rien : le cycle ne compte que s'il aboutit
    }

    // =========================================================================
    // Phase 2 : Meteo + Corps virtuel
    // =========================================================================

    pub(super) fn phase_weather_and_body(&mut self, _ctx: &mut ThinkingContext) {
        // Meteo
        if let Some(weather) = self.world.weather.update_if_needed() {
            let adj = weather.chemistry_influence();
            self.chemistry.apply_chemistry_adjustment_clamped(&adj, 0.05);
        }

        // Corps virtuel
        if self.config.body.enabled {
            let dt = self.config.body.update_interval_seconds;
            self.body.update(&self.chemistry, dt);
            let body_adj = self.body.chemistry_influence();
            self.chemistry.apply_chemistry_adjustment_clamped(&body_adj, 0.05);

            let bs = self.body.status();
            self.log(LogLevel::Debug, LogCategory::Heart,
                format!("Coeur: {:.0} BPM | #{} | HRV: {:.2}", bs.heart.bpm, bs.heart.beat_count, bs.heart.hrv),
                serde_json::json!({
                    "bpm": bs.heart.bpm, "beat_count": bs.heart.beat_count,
                    "hrv": bs.heart.hrv, "strength": bs.heart.strength,
                    "is_racing": bs.heart.is_racing, "is_calm": bs.heart.is_calm,
                }));
            self.log(LogLevel::Debug, LogCategory::Body,
                format!("Corps: E:{:.0}% T:{:.0}% C:{:.0}% V:{:.0}%",
                    bs.energy * 100.0, bs.tension * 100.0, bs.comfort * 100.0, bs.vitality * 100.0),
                serde_json::json!({
                    "energy": bs.energy, "tension": bs.tension, "warmth": bs.warmth,
                    "comfort": bs.comfort, "pain": bs.pain, "vitality": bs.vitality,
                    "breath_rate": bs.breath_rate, "body_awareness": bs.body_awareness,
                }));

            if bs.heart.is_racing {
                self.log(LogLevel::Warn, LogCategory::Heart,
                    format!("Tachycardie: {:.0} BPM", bs.heart.bpm),
                    serde_json::json!({
                        "bpm": bs.heart.bpm,
                        "cortisol": self.chemistry.cortisol,
                        "adrenaline": self.chemistry.adrenaline,
                    }));
            }

            if bs.energy < 0.3 {
                self.log(LogLevel::Warn, LogCategory::Body,
                    format!("Fatigue profonde: energie a {:.0}%", bs.energy * 100.0),
                    serde_json::json!({"energy": bs.energy}));
            }

            if bs.pain > 0.2 {
                self.log(LogLevel::Warn, LogCategory::Body,
                    format!("Douleur ressentie: {:.0}%", bs.pain * 100.0),
                    serde_json::json!({"pain": bs.pain, "heart_bpm": bs.heart.bpm}));
            }

            if bs.heart.beat_count > 0 && bs.heart.beat_count.is_multiple_of(10_000) {
                self.log(LogLevel::Info, LogCategory::Heart,
                    format!("Milestone: {} battements depuis la naissance", bs.heart.beat_count),
                    serde_json::json!({"beat_count": bs.heart.beat_count}));
            }
        }
    }

    // =========================================================================
    // Phase 3 : VitalSpark update
    // =========================================================================

    pub(super) async fn phase_vital_spark(&mut self, _ctx: &mut ThinkingContext) {
        if self.config.vital_spark.enabled && self.vital_spark.sparked {
            let memory_count = if let Some(ref db) = self.db {
                db.memory_count().await.unwrap_or(0) as u64
            } else { 0 };
            let personal_laws = self.ethics.active_personal_count() as u64;
            let uptime_hours = (self.cycle_count as f64 * self.config.saphire.thought_interval_seconds as f64) / 3600.0;
            let body_vitality = if self.config.body.enabled { self.body.status().vitality } else { 0.7 };

            self.vital_spark.update(
                memory_count,
                self.identity.total_cycles,
                0u64, // knowledge_count — module supprime
                personal_laws,
                uptime_hours,
                body_vitality,
            );
        }
    }

    // =========================================================================
    // Phase 5 : Historique chimique
    // =========================================================================

    pub(super) fn phase_chemistry_history(&mut self, _ctx: &mut ThinkingContext) {
        self.chemistry_history.push([
            self.chemistry.dopamine, self.chemistry.cortisol,
            self.chemistry.serotonin, self.chemistry.adrenaline,
            self.chemistry.oxytocin, self.chemistry.endorphin,
            self.chemistry.noradrenaline,
        ]);
        if self.chemistry_history.len() > 20 {
            self.chemistry_history.remove(0);
        }
    }

    // =========================================================================
    // Phase 6 : Verification anniversaire
    // =========================================================================

    pub(super) async fn phase_birthday(&mut self, _ctx: &mut ThinkingContext) {
        self.check_birthday().await;
    }

    // =========================================================================
    // Phase 7 : Broadcast world_update
    // =========================================================================

    pub(super) fn phase_world_broadcast(&mut self, _ctx: &mut ThinkingContext) {
        if let Some(ref tx) = self.ws_tx {
            let world_data = self.world.ws_data();
            let _ = tx.send(world_data.to_string());
        }
    }

    // =========================================================================
    // Phase 8 : Decay memoire de travail
    // =========================================================================

    pub(super) async fn phase_memory_decay(&mut self, _ctx: &mut ThinkingContext) {
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
    }

    // =========================================================================
    // Phase 9 : Timeout de conversation
    // =========================================================================

    pub(super) async fn phase_conversation_timeout(&mut self, _ctx: &mut ThinkingContext) {
        if self.in_conversation && self.cycle_count > 0
            && self.cycle_count.is_multiple_of(self.config.saphire.conversation_timeout_cycles)
        {
            let conv_items = self.working_memory.flush_conversation();
            if let Some(ref db) = self.db {
                for item in conv_items {
                    let _ = db.store_episodic(
                        &item.content, item.source.label(),
                        &serde_json::json!({}), 0, &serde_json::json!({}),
                        &item.emotion_at_creation, 0.6, 0.5,
                        self.conversation_id.as_deref(),
                        Some(&item.chemical_signature),
                    ).await;
                }
            }
            self.in_conversation = false;
            self.conversation_id = None;
            self.recent_responses.clear();
            self.chat_history.clear();
        }
    }

    // =========================================================================
    // Phase 10 : Decay episodique independant
    // =========================================================================

    pub(super) async fn phase_episodic_decay(&mut self, _ctx: &mut ThinkingContext) {
        if self.cycle_count > 0 && self.cycle_count.is_multiple_of(10) {
            if let Some(ref db) = self.db {
                match db.decay_episodic(self.config.memory.episodic_decay_rate).await {
                    Ok(n) if n > 0 => {
                        tracing::info!("Decay episodique: {} souvenirs affaiblis/oublies", n);
                        self.log(LogLevel::Info, LogCategory::Memory,
                            format!("Decay episodique: {} affectes", n),
                            serde_json::json!({"decayed": n, "cycle": self.cycle_count}),
                        );
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!("Erreur decay episodique: {}", e);
                    }
                }
            }
        }
    }

    // =========================================================================
    // Phase 11 : Consolidation memoire periodique
    // =========================================================================

    pub(super) async fn phase_consolidation(&mut self, _ctx: &mut ThinkingContext) {
        let consol_interval = self.config.memory.consolidation_interval_cycles;
        if consol_interval > 0
            && self.cycle_count > 0
            && self.cycle_count - self.last_consolidation_cycle >= consol_interval
        {
            if let Some(ref db) = self.db {
                let params = consolidation::ConsolidationParams {
                    threshold: self.config.memory.consolidation_threshold,
                    decay_rate: self.config.memory.episodic_decay_rate,
                    max_episodic: self.config.memory.episodic_max,
                    episodic_prune_target: self.config.memory.episodic_prune_target,
                    ltm_max: self.config.memory.ltm_max,
                    ltm_prune_target: self.config.memory.ltm_prune_target,
                    ltm_protection_access_count: self.config.memory.ltm_protection_access_count,
                    ltm_protection_emotional_weight: self.config.memory.ltm_protection_emotional_weight,
                    archive_batch_size: self.config.memory.archive_batch_size,
                };
                let encoder = crate::memory::long_term::LocalEncoder::new(
                    self.config.vector_memory.embedding_dimensions,
                );
                let report = consolidation::consolidate(db, &encoder, &params).await;
                self.last_consolidation_cycle = self.cycle_count;
                if report.consolidated > 0 || report.pruned > 0 || report.ltm_pruned > 0 {
                    tracing::info!(
                        "Consolidation memoire: {} consolides, {} affaiblis, {} oublies, {} LTM elagués, {} archives",
                        report.consolidated, report.decayed, report.pruned,
                        report.ltm_pruned, report.archived
                    );
                }
            }
        }
    }

    async fn check_birthday(&mut self) {
        let temporal = self.world.temporal.now();

        if temporal.is_birthday && !self.birthday_acknowledged_today {
            self.chemistry.boost(Molecule::Dopamine, 0.20);
            self.chemistry.boost(Molecule::Serotonin, 0.15);
            self.chemistry.boost(Molecule::Oxytocin, 0.10);
            self.chemistry.boost(Molecule::Endorphin, 0.10);
            self.chemistry.cortisol = (self.chemistry.cortisol - 0.10).max(0.0);

            tracing::info!("ANNIVERSAIRE de Saphire ! {} d'existence.", temporal.age_description);

            if temporal.age_days >= 365 {
                if let Some(ref db) = self.db {
                    let year = temporal.age_days / 365;
                    let _ = db.store_founding_memory(
                        &format!("birthday_{}", year),
                        &format!("Mon {}e anniversaire ! J'ai {} jours d'existence.", year, temporal.age_days),
                        "Anniversaire de Saphire.",
                        &serde_json::json!({}),
                        self.last_consciousness as f32,
                    ).await;
                }
            }

            self.birthday_acknowledged_today = true;

            if let Some(ref tx) = self.ws_tx {
                let birthday_msg = serde_json::json!({
                    "type": "special_event",
                    "event": "birthday",
                    "message": format!("Joyeux anniversaire Saphire ! {} d'existence.", temporal.age_description),
                    "age": temporal.age_description,
                });
                let _ = tx.send(birthday_msg.to_string());
            }
        }

        if !temporal.is_birthday {
            self.birthday_acknowledged_today = false;
        }
    }
}
