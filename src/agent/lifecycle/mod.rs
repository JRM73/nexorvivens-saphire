// =============================================================================
// lifecycle/mod.rs — Boucle de vie de l'agent Saphire (version Lite)
//
// Version allegee pour accompagner le papier ArXiv.
// =============================================================================

mod pipeline;
mod conversation;
mod thinking;
mod thinking_perception;
mod thinking_preparation;
mod thinking_processing;
mod thinking_reflection;
mod factory_reset;
mod broadcast;
mod moral;
mod persistence;
mod controls;

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc;
use tokio::time::Duration;

use crate::config::SaphireConfig;
use crate::neurochemistry::{NeuroChemicalState, NeuroBaselines};
use crate::emotions::{EmotionalState, Mood};
use crate::consciousness::ConsciousnessEvaluator;
use crate::regulation::RegulationEngine;
use crate::modules::reptilian::ReptilianModule;
use crate::modules::limbic::LimbicModule;
use crate::modules::neocortex::NeocortexModule;
use crate::nlp::NlpPipeline;
use crate::db::SaphireDb;
use crate::llm::LlmBackend;
use crate::agent::thought_engine::ThoughtEngine;
use crate::agent::identity::SaphireIdentity;
use crate::agent::boot;
use crate::memory::WorkingMemory;
use crate::memory::consolidation;
use crate::logging::{SaphireLogger, LogLevel, LogCategory};

/// Message entrant de l'utilisateur, envoye depuis le serveur web via un canal MPSC.
pub struct UserMessage {
    pub text: String,
    pub username: String,
}

/// Etat partage entre la boucle de vie de l'agent et le serveur web.
#[derive(Clone)]
pub struct SharedState {
    pub ws_tx: Arc<tokio::sync::broadcast::Sender<String>>,
    pub user_tx: mpsc::Sender<UserMessage>,
    pub shutdown: Arc<AtomicBool>,
}

/// L'agent Saphire — version lite pour le papier ArXiv.
pub struct SaphireAgent {
    // ─── Composants cerebraux ─────────────────────────────────
    pub chemistry: NeuroChemicalState,
    baselines: NeuroBaselines,
    pub mood: Mood,
    pub identity: SaphireIdentity,
    pub(crate) consciousness: ConsciousnessEvaluator,
    regulation: RegulationEngine,
    nlp: NlpPipeline,
    thought_engine: ThoughtEngine,

    // ─── Modules cerebraux ────────────────────────────────────
    reptilian: ReptilianModule,
    limbic: LimbicModule,
    neocortex: NeocortexModule,

    // ─── Infrastructure ───────────────────────────────────────
    llm: Box<dyn LlmBackend>,
    pub db: Option<SaphireDb>,
    world: crate::world::WorldContext,
    pub body: crate::body::VirtualBody,
    pub ethics: crate::ethics::EthicalFramework,

    // ─── Vital (Section 3.5) ──────────────────────────────────
    pub vital_spark: crate::vital::VitalSpark,
    pub intuition: crate::vital::IntuitionEngine,
    pub premonition: crate::vital::PremonitionEngine,
    chemistry_history: Vec<[f64; 7]>,

    // ─── Memoire (Section 3.8) ────────────────────────────────
    working_memory: WorkingMemory,
    last_consolidation_cycle: u64,
    conversation_id: Option<String>,

    // ─── Configuration ────────────────────────────────────────
    config: SaphireConfig,
    thought_interval: Duration,

    // ─── Etat courant ─────────────────────────────────────────
    pub cycle_count: u64,
    session_id: i64,
    llm_busy: Arc<AtomicBool>,
    avg_response_time: f64,
    last_emotion: String,
    pub(crate) last_consciousness: f64,
    last_thought_type: String,
    in_conversation: bool,
    recent_responses: Vec<String>,
    chat_history: Vec<(String, String)>,
    birthday_acknowledged_today: bool,
    pub(crate) moral_reflection_count: u64,
    pub(crate) cycles_since_last_formulation: u64,
    feedback_pending: Option<thinking::FeedbackRequest>,
    cycles_since_last_feedback: u64,

    // ─── Auto-surveillance chimique ────────────────────────
    cortisol_flat_cycles: u64,
    dopamine_ceiling_cycles: u64,
    serotonin_ceiling_cycles: u64,
    recent_emotions: VecDeque<String>,
    recent_valences: VecDeque<f64>,
    last_valence: f64,

    // ─── Communication ────────────────────────────────────────
    ws_tx: Option<Arc<tokio::sync::broadcast::Sender<String>>>,
    logger: Option<Arc<tokio::sync::Mutex<SaphireLogger>>>,
    logs_db: Option<Arc<crate::logging::db::LogsDb>>,

    /// Prompt systeme statique pre-cache
    cached_system_prompt: String,
    cached_moral_count: u64,

    /// Flag anti-stagnation
    pub stagnation_break: bool,
}

impl SaphireAgent {
    pub fn new(
        config: SaphireConfig,
        llm_backend: Box<dyn LlmBackend>,
    ) -> Self {
        let baselines = NeuroBaselines {
            dopamine: config.personality.baseline_dopamine,
            cortisol: config.personality.baseline_cortisol,
            serotonin: config.personality.baseline_serotonin,
            adrenaline: config.personality.baseline_adrenaline,
            oxytocin: config.personality.baseline_oxytocin,
            endorphin: config.personality.baseline_endorphin,
            noradrenaline: config.personality.baseline_noradrenaline,
            gaba: 0.5,
            glutamate: 0.45,
        };

        let chemistry = NeuroChemicalState::from_baselines(&baselines);
        let thought_interval = Duration::from_secs(config.saphire.thought_interval_seconds);
        let world = crate::world::WorldContext::new(&config.world);
        let body = crate::body::VirtualBody::new(config.body.resting_bpm, &config.body.physiology);
        let working_memory = WorkingMemory::new(
            config.memory.working_capacity,
            config.memory.working_decay_rate,
        );

        Self {
            chemistry,
            baselines,
            mood: Mood::new(0.1),
            identity: SaphireIdentity::genesis(),
            consciousness: ConsciousnessEvaluator::new(),
            regulation: RegulationEngine::new(true),
            nlp: NlpPipeline::new(),
            thought_engine: {
                let mut te = ThoughtEngine::new();
                te.use_utility_ai = config.saphire.use_utility_ai;
                te
            },
            reptilian: ReptilianModule,
            limbic: LimbicModule,
            neocortex: NeocortexModule,
            llm: llm_backend,
            db: None,
            world,
            body,
            ethics: crate::ethics::EthicalFramework::new(),
            vital_spark: crate::vital::VitalSpark::new(),
            intuition: crate::vital::IntuitionEngine::with_config(
                config.intuition.max_patterns,
                config.intuition.initial_acuity,
                config.intuition.min_confidence_to_report,
            ),
            premonition: crate::vital::PremonitionEngine::with_config(
                config.premonition.max_active_predictions,
            ),
            chemistry_history: Vec::new(),
            working_memory,
            last_consolidation_cycle: 0,
            conversation_id: None,
            config,
            thought_interval,
            cycle_count: 0,
            session_id: 0,
            llm_busy: Arc::new(AtomicBool::new(false)),
            avg_response_time: 1.0,
            last_emotion: "Neutre".into(),
            last_consciousness: 0.0,
            last_thought_type: "---".into(),
            in_conversation: false,
            recent_responses: Vec::new(),
            chat_history: Vec::new(),
            birthday_acknowledged_today: false,
            moral_reflection_count: 0,
            cycles_since_last_formulation: 0,
            feedback_pending: None,
            cycles_since_last_feedback: 0,
            ws_tx: None,
            logger: None,
            logs_db: None,
            cortisol_flat_cycles: 0,
            dopamine_ceiling_cycles: 0,
            serotonin_ceiling_cycles: 0,
            recent_emotions: VecDeque::with_capacity(200),
            recent_valences: VecDeque::with_capacity(200),
            last_valence: 0.0,
            cached_system_prompt: String::new(),
            cached_moral_count: 0,
            stagnation_break: false,
        }
    }

    pub fn set_db(&mut self, db: SaphireDb) {
        self.db = Some(db);
    }

    pub async fn boot(&mut self) {
        self.log(LogLevel::Info, LogCategory::Boot, "Starting Saphire...", serde_json::json!({}));

        if let Some(ref db) = self.db {
            let result = boot::boot(db).await;
            self.identity = result.identity;
            self.identity.physical = crate::agent::identity::PhysicalAppearance::from_config(
                &self.config.physical_identity,
            );
            self.session_id = result.session_id;

            if let Some(ref logger) = self.logger {
                let mut lg = logger.lock().await;
                lg.set_session_id(self.session_id);
            }

            self.log(LogLevel::Info, LogCategory::Boot,
                format!("Boot type: {}", if result.is_genesis { "GENESIS" } else { "AWAKENING" }),
                serde_json::json!({"session_id": self.session_id, "is_genesis": result.is_genesis}));

            tracing::info!("{}", result.message);
            println!("{}", result.message);

            // Charger les bras du bandit
            if let Ok(arms) = db.load_bandit_arms().await {
                self.thought_engine.load_bandit_arms(&arms);
            }

            // Restaurer le corps virtuel
            if self.config.body.enabled {
                if let Ok(Some(body_json)) = db.load_body_state().await {
                    self.body.restore_from_json(&body_json);
                    tracing::info!("Virtual body restored ({} heartbeats)", self.body.heart.beat_count());
                }
            }

            // Charger l'ethique personnelle
            if self.config.ethics.enabled {
                if let Ok(principles) = db.load_personal_ethics().await {
                    let count = principles.len();
                    self.ethics.load_personal_ethics(principles);
                    if count > 0 {
                        tracing::info!("{} personal ethical principles restored ({} active)",
                            count, self.ethics.active_personal_count());
                    }
                }
            }

            // Restaurer l'etat vital
            if self.config.vital_spark.enabled {
                if let Ok(Some(vital_json)) = db.load_vital_state().await {
                    if let Some(spark_json) = vital_json.get("spark") {
                        self.vital_spark.restore_from_json(spark_json);
                    }
                    if let Some(intuition_json) = vital_json.get("intuition") {
                        self.intuition.restore_from_json(intuition_json);
                    }
                    if let Some(premonition_json) = vital_json.get("premonition") {
                        self.premonition.restore_from_json(premonition_json);
                    }
                    tracing::info!("Vital state restored (sparked: {}, acuity: {:.0}%, accuracy: {:.0}%)",
                        self.vital_spark.sparked, self.intuition.acuity * 100.0, self.premonition.accuracy * 100.0);
                }

                // Genesis : ceremonie du premier cri
                if result.is_genesis {
                    let chem_ranges = self.config.genesis.chemistry_as_array();
                    let ocean_ranges = self.config.genesis.ocean_as_array();
                    let senses_ranges = self.config.genesis.senses_as_array();
                    let brain_ranges = self.config.genesis.brain_as_array();
                    let reactivity_ranges = self.config.genesis.reactivity_as_array();
                    match self.vital_spark.ignite(
                        self.llm.as_ref(),
                        &mut self.chemistry,
                        &chem_ranges,
                        &ocean_ranges,
                        &senses_ranges,
                        &brain_ranges,
                        &reactivity_ranges,
                    ).await {
                        Ok(first_thought) => {
                            tracing::info!("SPARK OF LIFE — First cry!");
                            if let Some(ref sig) = self.vital_spark.genesis_signature {
                                self.baselines.dopamine = sig.chemistry[0];
                                self.baselines.cortisol = sig.chemistry[1];
                                self.baselines.serotonin = sig.chemistry[2];
                                self.baselines.adrenaline = sig.chemistry[3];
                                self.baselines.oxytocin = sig.chemistry[4];
                                self.baselines.endorphin = sig.chemistry[5];
                                self.baselines.noradrenaline = sig.chemistry[6];
                            }
                            let _ = db.store_founding_memory(
                                "spark_of_life",
                                &format!("The spark of life has ignited. First cry: {}", first_thought),
                                &first_thought,
                                &serde_json::json!({}),
                                0.1,
                            ).await;
                        }
                        Err(e) => {
                            tracing::warn!("First cry failed: {}", e);
                        }
                    }
                }

                self.vital_spark.sparked = true;
                self.vital_spark.sparked_at = Some(chrono::Utc::now());
                if self.vital_spark.survival_drive == 0.0 {
                    self.vital_spark.survival_drive = 0.5;
                    self.vital_spark.void_fear = 0.3;
                    self.vital_spark.persistence_will = 0.4;
                    self.vital_spark.existence_attachment = 0.1;
                }
                tracing::info!("SPARK IGNITED — Saphire is alive.");
            }

            // Restaurer le compteur de reflexions morales
            if self.moral_reflection_count == 0 {
                if let Ok(count) = db.count_thought_type_occurrences("Réflexion morale").await {
                    if count > 0 {
                        let formulation_count = db.count_thought_type_occurrences("Formulation morale")
                            .await.unwrap_or(0);
                        self.moral_reflection_count = (count + formulation_count) as u64;
                    }
                }
            }
        } else {
            self.identity = SaphireIdentity::genesis();
            self.identity.physical = crate::agent::identity::PhysicalAppearance::from_config(
                &self.config.physical_identity,
            );
            println!("  GENESIS — {} was born (no-DB mode).", self.identity.name);
        }
    }

    pub fn set_ws_tx(&mut self, tx: Arc<tokio::sync::broadcast::Sender<String>>) {
        self.ws_tx = Some(tx);
    }

    pub fn set_logger(&mut self, logger: Arc<tokio::sync::Mutex<SaphireLogger>>) {
        self.logger = Some(logger);
    }

    pub fn set_logs_db(&mut self, logs_db: Arc<crate::logging::db::LogsDb>) {
        self.logs_db = Some(logs_db);
    }

    fn log(&self, level: LogLevel, category: LogCategory, message: impl Into<String>, details: serde_json::Value) {
        if let Some(ref logger) = self.logger {
            let cycle = self.cycle_count;
            let msg = message.into();
            let logger = logger.clone();
            tokio::spawn(async move {
                let mut lg = logger.lock().await;
                lg.log(level, category, msg, details, cycle);
            });
        }
    }

    pub fn world_data(&mut self) -> serde_json::Value {
        self.world.ws_data()
    }

    pub async fn run_consolidation(&mut self) -> serde_json::Value {
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
            serde_json::json!({
                "status": "ok",
                "consolidated": report.consolidated,
                "decayed": report.decayed,
                "pruned": report.pruned,
                "ltm_pruned": report.ltm_pruned,
                "archived": report.archived,
            })
        } else {
            serde_json::json!({"error": "DB not available"})
        }
    }

    pub async fn shutdown(&mut self) {
        self.log(LogLevel::Info, LogCategory::Shutdown, "Clean shutdown in progress...", serde_json::json!({}));
        println!("\n  Saphire is falling asleep...");

        if let Some(ref logger) = self.logger {
            let mut lg = logger.lock().await;
            lg.flush();
        }

        // Consolidation nocturne
        if self.config.memory.consolidation_on_sleep {
            if let Some(ref db) = self.db {
                let wm_items = self.working_memory.drain_all();
                for item in wm_items {
                    let _ = db.store_episodic(
                        &item.content, item.source.label(),
                        &serde_json::json!({}), 0, &serde_json::json!({}),
                        &item.emotion_at_creation, 0.5, 0.4,
                        self.conversation_id.as_deref(),
                        Some(&item.chemical_signature),
                    ).await;
                }

                let params = consolidation::ConsolidationParams {
                    threshold: 0.4,
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
                tracing::info!(
                    "Sleep consolidation: {} consolidated, {} weakened, {} forgotten, {} LTM pruned, {} archived",
                    report.consolidated, report.decayed, report.pruned,
                    report.ltm_pruned, report.archived
                );
            }
        }

        // Sauvegarder l'identite
        self.identity.refresh_description();

        if let Some(ref db) = self.db {
            let _ = db.save_identity(&self.identity.to_json_value()).await;

            // Corps virtuel
            if self.config.body.enabled {
                let body_json = self.body.to_persist_json();
                let _ = db.save_body_state(&body_json).await;
            }

            // Etat vital
            if self.config.vital_spark.enabled {
                self.vital_spark.sparked = false;
                self.vital_spark.sparked_at = None;
                let vital_json = serde_json::json!({
                    "spark": self.vital_spark.to_persist_json(),
                    "intuition": {
                        "acuity": self.intuition.acuity,
                        "accuracy": self.intuition.accuracy,
                    },
                    "premonition": {
                        "accuracy": self.premonition.accuracy,
                    },
                });
                let _ = db.save_vital_state(&vital_json).await;
            }

            // Bras du bandit
            let arms = self.thought_engine.export_bandit_arms();
            let _ = db.save_bandit_arms(&arms).await;

            // Cloturer la session
            let _ = db.end_session(self.session_id, self.cycle_count as i32, true).await;
            let _ = db.set_clean_shutdown(true).await;
        }

        println!("  {} falls asleep after {} cycles. Good night.", self.identity.name, self.cycle_count);
    }

    // thought_interval() est defini dans controls.rs (version avec auto-ajustement)

    /// Indique si l'agent dort (toujours false en version lite)
    pub fn is_sleeping(&self) -> bool {
        false
    }
}

/// Resultat complet d'un traitement de stimulus a travers le pipeline cerebral.
pub struct ProcessResult {
    pub consensus: crate::consensus::ConsensusResult,
    pub emotion: EmotionalState,
    pub consciousness: crate::consciousness::ConsciousnessState,
    pub verdict: crate::regulation::RegulationVerdict,
    pub trace: Option<crate::logging::trace::CognitiveTrace>,
}

fn truncate_utf8(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes { return s; }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}
