// =============================================================================
// lifecycle/thinking_perception.rs — Phases pre-LLM (perception du monde)
// =============================================================================
//
// Ce fichier contient les phases de perception et de mise a jour de l'etat
// interne de Saphire avant l'appel au LLM. Cela inclut :
//   - Initialisation du cycle
//   - Meteo + corps virtuel
//   - Besoins primaires (faim, soif)
//   - VitalSpark
//   - Perception sensorielle
//   - Historique chimique
//   - Anniversaire
//   - Broadcast world_update
//   - Decay memoire (travail, episodique)
//   - Consolidation memoire
//   - Algorithmes automatiques
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

    /// Phase d'initialisation du cycle (pas d'increment ici — le compteur
    /// n'est incremente qu'a la fin du pipeline complet dans process_stimulus).
    pub(super) fn phase_init(&mut self, _ctx: &mut ThinkingContext) {
        // Rien : le cycle ne compte que s'il aboutit
    }

    // =========================================================================
    // Phase 2 : Meteo + Corps virtuel
    // =========================================================================

    /// Met a jour la meteo, le corps virtuel et applique les influences chimiques.
    pub(super) fn phase_weather_and_body(&mut self, _ctx: &mut ThinkingContext) {
        // Mise a jour meteo + influence chimique
        if let Some(weather) = self.world.weather.update_if_needed() {
            let adj = weather.chemistry_influence();
            self.chemistry.apply_chemistry_adjustment_clamped(&adj, 0.05);
        }

        // Mise a jour du corps virtuel + influence chimique
        if self.config.body.enabled {
            let dt = self.config.body.update_interval_seconds;
            let hormones_ref = if self.hormonal_system.enabled {
                Some(&self.hormonal_system.state)
            } else {
                None
            };
            self.body.update_with_hormones(&self.chemistry, dt, hormones_ref);
            let body_adj = self.body.chemistry_influence();
            self.chemistry.apply_chemistry_adjustment_clamped(&body_adj, 0.05);

            // Logging du coeur et du corps
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

            // Tachycardie
            if bs.heart.is_racing {
                self.log(LogLevel::Warn, LogCategory::Heart,
                    format!("Tachycardie: {:.0} BPM", bs.heart.bpm),
                    serde_json::json!({
                        "bpm": bs.heart.bpm,
                        "cortisol": self.chemistry.cortisol,
                        "adrenaline": self.chemistry.adrenaline,
                    }));
            }

            // Fatigue profonde (energie < 30%)
            if bs.energy < 0.3 {
                self.log(LogLevel::Warn, LogCategory::Body,
                    format!("Fatigue profonde: energie a {:.0}%", bs.energy * 100.0),
                    serde_json::json!({"energy": bs.energy}));
            }

            // Douleur
            if bs.pain > 0.2 {
                self.log(LogLevel::Warn, LogCategory::Body,
                    format!("Douleur ressentie: {:.0}%", bs.pain * 100.0),
                    serde_json::json!({"pain": bs.pain, "heart_bpm": bs.heart.bpm}));
            }

            // Milestone de battements (tous les 10 000)
            if bs.heart.beat_count > 0 && bs.heart.beat_count.is_multiple_of(10_000) {
                self.log(LogLevel::Info, LogCategory::Heart,
                    format!("Milestone: {} battements depuis la naissance", bs.heart.beat_count),
                    serde_json::json!({"beat_count": bs.heart.beat_count}));
            }

            // Influence corps -> chimie
            self.log(LogLevel::Debug, LogCategory::Body,
                "Signal interoceptif -> chimie".to_string(),
                serde_json::json!({
                    "body_adjustments": {
                        "cortisol": body_adj.cortisol,
                        "serotonin": body_adj.serotonin,
                        "endorphin": body_adj.endorphin,
                        "dopamine": body_adj.dopamine,
                        "noradrenaline": body_adj.noradrenaline,
                        "oxytocin": body_adj.oxytocin,
                    }
                }));
        }
    }

    // =========================================================================
    // Phase 2b : Besoins primaires (faim, soif)
    // =========================================================================

    /// Met a jour les drives de faim et soif, applique l'impact chimique,
    /// et declenche l'auto-satisfaction si les seuils sont depasses.
    pub(super) fn phase_needs(&mut self, _ctx: &mut ThinkingContext) {
        if !self.config.needs.enabled {
            return;
        }

        // Lire les valeurs physiologiques pour calculer les drives
        let glycemia = self.body.physiology.glycemia;
        let hydration = self.body.physiology.hydration;
        let cycle = self.cycle_count;
        let config = self.config.needs.clone();

        // Tick des besoins
        self.needs.tick(glycemia, hydration, cycle, &config);

        // Impact chimique des besoins non satisfaits
        let adj = self.needs.chemistry_influence(&config);
        if adj.cortisol != 0.0 || adj.serotonin != 0.0 || adj.dopamine != 0.0 || adj.noradrenaline != 0.0 {
            self.chemistry.apply_chemistry_adjustment_clamped(&adj, 0.05);
        }

        // Logging des niveaux
        if self.needs.is_hungry(&config) || self.needs.is_thirsty(&config) {
            self.log(LogLevel::Debug, LogCategory::Body,
                format!("Besoins: faim={:.0}% soif={:.0}%",
                    self.needs.hunger.level * 100.0, self.needs.thirst.level * 100.0),
                serde_json::json!({
                    "hunger": self.needs.hunger.level,
                    "thirst": self.needs.thirst.level,
                    "glycemia": glycemia,
                    "hydration": hydration,
                }));
        }

        // Auto-satisfaction
        if let Some(action) = self.needs.check_auto_satisfy(&config) {
            match action {
                crate::needs::AutoSatisfyAction::Eat => {
                    let result = self.needs.eat(cycle, &config);
                    // Appliquer le boost sur la physiologie
                    self.body.physiology.glycemia = result.glycemia_target;
                    self.chemistry.boost(Molecule::Dopamine, result.dopamine_boost);
                    self.log(LogLevel::Info, LogCategory::Body,
                        format!("Auto-repas: glycemie restauree a {:.1}, dopamine +{:.2}",
                            result.glycemia_target, result.dopamine_boost),
                        serde_json::json!({
                            "action": "eat",
                            "glycemia_target": result.glycemia_target,
                            "dopamine_boost": result.dopamine_boost,
                            "meals_count": self.needs.hunger.meals_count,
                        }));
                    // Broadcast WS
                    self.broadcast_need_satisfied("eat");
                }
                crate::needs::AutoSatisfyAction::Drink => {
                    let result = self.needs.drink(cycle, &config);
                    // Appliquer le boost sur la physiologie
                    self.body.physiology.hydration = result.hydration_target;
                    self.chemistry.boost(Molecule::Dopamine, result.dopamine_boost);
                    self.log(LogLevel::Info, LogCategory::Body,
                        format!("Auto-boisson: hydratation restauree a {:.0}%, dopamine +{:.2}",
                            result.hydration_target * 100.0, result.dopamine_boost),
                        serde_json::json!({
                            "action": "drink",
                            "hydration_target": result.hydration_target,
                            "dopamine_boost": result.dopamine_boost,
                            "drinks_count": self.needs.thirst.drinks_count,
                        }));
                    // Broadcast WS
                    self.broadcast_need_satisfied("drink");
                }
            }
        }
    }

    // =========================================================================
    // Phase 3 : VitalSpark update
    // =========================================================================

    /// Met a jour l'etincelle de vie (VitalSpark) si activee.
    pub(super) async fn phase_vital_spark(&mut self, _ctx: &mut ThinkingContext) {
        if self.config.vital_spark.enabled && self.vital_spark.sparked {
            let memory_count = if let Some(ref db) = self.db {
                db.memory_count().await.unwrap_or(0) as u64
            } else { 0 };
            let knowledge_count = if let Some(ref _db) = self.db {
                self.knowledge.topics_explored.len() as u64
            } else { 0 };
            let personal_laws = self.ethics.active_personal_count() as u64;
            let uptime_hours = (self.cycle_count as f64 * self.config.saphire.thought_interval_seconds as f64) / 3600.0;
            let body_vitality = if self.config.body.enabled { self.body.status().vitality } else { 0.7 };

            self.vital_spark.update(
                memory_count,
                self.identity.total_cycles,
                knowledge_count,
                personal_laws,
                uptime_hours,
                body_vitality,
            );
        }
    }

    // =========================================================================
    // Phase 4 : Perception sensorielle autonome
    // =========================================================================

    /// Met a jour le Sensorium (5 sens + graines emergentes).
    pub(super) fn phase_senses(&mut self, _ctx: &mut ThinkingContext) {
        if !self.config.senses.enabled {
            return;
        }

        // Contact : percevoir l'etat des connexions reseau
        let db_ok = self.db.is_some();
        let llm_ok = self.llm.health_check().map(|h| h.connected).unwrap_or(false);
        let ws_clients = if self.ws_tx.is_some() { 1 } else { 0 };
        self.sensorium.contact.update_warmth(db_ok, llm_ok, ws_clients);

        // Contact : percevoir la connexion au LLM (toucher reseau)
        let llm_latency: u64 = if llm_ok { 50 } else { 9999 };
        let _ = self.sensorium.contact.perceive_connection("ollama", llm_latency, llm_ok);

        // Ecoute : percevoir le silence (si pas en conversation)
        if !self.in_conversation {
            let silence_secs = self.config.saphire.thought_interval_seconds as f64;
            let _ = self.sensorium.listening.perceive_silence(silence_secs);
        }

        // Ambiance : percevoir l'atmosphere
        let is_day = self.world.weather.current()
            .map(|w| w.is_day)
            .unwrap_or(true);
        let weather_desc = self.world.weather.current()
            .map(|w| w.description.clone())
            .unwrap_or_default();
        let system_errors = 0u32;
        let silence_min = if self.in_conversation { 0u64 }
            else { (self.cycle_count * self.config.saphire.thought_interval_seconds) / 60 };
        let _ambiance_signal = self.sensorium.ambiance.perceive(
            &self.chemistry,
            is_day,
            self.in_conversation,
            system_errors,
            silence_min,
            &weather_desc,
        );

        // Stimuler les graines emergentes
        self.sensorium.emergent_seeds.stimulate("temporal_flow");

        // Proprioception reseau : stimulee quand toutes les connexions sont actives
        if db_ok && llm_ok && ws_clients > 0 {
            self.sensorium.emergent_seeds.stimulate("network_proprioception");
        }

        // Syntonie : stimulee quand tous les systemes sont en harmonie
        // (chimie equilibree, conscience elevee, humeur positive, corps vital)
        let chimie_stable = (self.chemistry.cortisol - 0.3).abs() < 0.15
            && (self.chemistry.serotonin - 0.5).abs() < 0.2;
        let conscience_haute = self.last_consciousness >= 0.5;
        let humeur_positive = self.mood.valence > 0.0;
        let corps_ok = !self.config.body.enabled || self.body.status().vitality > 0.5;
        if chimie_stable && conscience_haute && humeur_positive && corps_ok {
            self.sensorium.emergent_seeds.stimulate("syntony");
        }

        // Sens inconnu : stimule rarement, quand les conditions sont inhabituelles
        // (conscience tres haute + chimie atypique + en conversation)
        let conscience_rare = self.last_consciousness >= 0.8;
        let chimie_atypique = self.chemistry.dopamine > 0.7 && self.chemistry.cortisol < 0.15;
        if conscience_rare && chimie_atypique && self.in_conversation {
            self.sensorium.emergent_seeds.stimulate("unknown");
        }

        // Synthese sensorielle
        let (snapshot, senses_adj) = self.sensorium.synthesize();
        self.chemistry.apply_chemistry_adjustment_clamped(&senses_adj, 0.05);

        // Log sensoriel
        if snapshot.perception_richness > 0.3 {
            self.log(LogLevel::Debug, LogCategory::Senses,
                format!("Sens: {} dominant, richesse {:.0}%",
                    snapshot.dominant_sense, snapshot.perception_richness * 100.0),
                serde_json::json!({
                    "dominant": snapshot.dominant_sense,
                    "richness": snapshot.perception_richness,
                    "emergence_potential": self.sensorium.emergence_potential,
                }));
        }

        // Log germination des sens emergents
        let germinated = self.sensorium.emergent_seeds.germinated_count();
        if germinated > 0 {
            self.log(LogLevel::Info, LogCategory::Senses,
                format!("Sens emergents: {} germes", germinated),
                serde_json::json!({"germinated": germinated}));
        }
    }

    // =========================================================================
    // Phase 4b : MAP — Synchronisation Sensorium ↔ BrainNetwork ↔ Connectome
    // =========================================================================

    /// Synchronise le BrainNetwork et le Connectome avec les donnees sensorielles
    /// fraichement mises a jour par phase_senses. Comble le decalage temporel
    /// entre perception sensorielle et reaction cerebrale.
    pub(super) fn phase_map_sync(&mut self, ctx: &mut ThinkingContext) {
        if !self.map_sync.enabled {
            return;
        }

        // Construire le vecteur sensoriel courant
        let sensory_input = [
            self.sensorium.reading.current_intensity,
            self.sensorium.listening.current_intensity,
            self.sensorium.contact.current_intensity,
            self.sensorium.taste.current_intensity,
            self.sensorium.ambiance.current_intensity,
        ];

        // Synchroniser BrainNetwork avec chimie + sens actuels
        self.brain_network.tick(&self.chemistry, sensory_input);

        // Modulation matiere grise si active
        if self.config.grey_matter.enabled {
            let gm_factor = self.grey_matter.grey_matter_volume.max(0.3);
            for region in &mut self.brain_network.regions {
                region.activation *= gm_factor;
            }
            self.brain_network.compute_global_workspace();
        }

        // Synchroniser Connectome avec sens actifs
        if self.config.connectome.enabled {
            let threshold = 0.2;
            let mut active: Vec<String> = Vec::new();
            if sensory_input[0] > threshold { active.push("lecture".into()); }
            if sensory_input[1] > threshold { active.push("ecoute".into()); }
            if sensory_input[2] > threshold { active.push("contact".into()); }
            if sensory_input[3] > threshold { active.push("saveur".into()); }
            if sensory_input[4] > threshold { active.push("ambiance".into()); }
            // Ajouter emotion dominante si disponible
            if !ctx.emotion.dominant.is_empty() {
                active.push(ctx.emotion.dominant.to_lowercase());
            }
            let refs: Vec<&str> = active.iter().map(|s| s.as_str()).collect();
            self.connectome.tick(&refs);
        }

        // Calculer la tension du reseau (ecart perception/reaction)
        let result = self.map_sync.compute_tension(
            &sensory_input,
            &self.brain_network,
        );
        self.map_sync.last_sync_cycle = self.cycle_count;

        // Injecter dans le ctx pour usage ulterieur
        ctx.network_tension = result.network_tension;

        // Log si tension elevee
        if result.network_tension > 0.3 {
            self.log(LogLevel::Warn, LogCategory::Brain,
                format!("MAP: tension elevee {:.0}% | dominant={} | workspace={:.2}",
                    result.network_tension * 100.0,
                    result.dominant_region,
                    result.workspace_strength),
                serde_json::json!({
                    "network_tension": result.network_tension,
                    "dominant_region": result.dominant_region,
                    "workspace_strength": result.workspace_strength,
                }));
        } else {
            self.log(LogLevel::Debug, LogCategory::Brain,
                format!("MAP: tension {:.0}% | dominant={} | workspace={:.2}",
                    result.network_tension * 100.0,
                    result.dominant_region,
                    result.workspace_strength),
                serde_json::json!({
                    "network_tension": result.network_tension,
                    "dominant_region": result.dominant_region,
                    "workspace_strength": result.workspace_strength,
                }));
        }
    }

    // =========================================================================
    // Phase 5 : Historique chimique
    // =========================================================================

    /// Enregistre l'etat chimique courant pour les tendances.
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

    /// Verifie si c'est l'anniversaire de Saphire (une fois par jour).
    pub(super) async fn phase_birthday(&mut self, _ctx: &mut ThinkingContext) {
        self.check_birthday().await;
    }

    // =========================================================================
    // Phase 7 : Broadcast world_update
    // =========================================================================

    /// Diffuse les donnees du monde au WebSocket.
    pub(super) fn phase_world_broadcast(&mut self, _ctx: &mut ThinkingContext) {
        if let Some(ref tx) = self.ws_tx {
            let world_data = self.world.ws_data();
            let _ = tx.send(world_data.to_string());
        }
    }

    // =========================================================================
    // Phase 8 : Decay memoire de travail
    // =========================================================================

    /// Fait decroitre les elements en memoire de travail et transfiere
    /// les elements expires vers la memoire episodique.
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

    /// Detecte la fin de conversation par timeout et transfiere les elements
    /// de conversation vers la memoire episodique.
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

    /// Affaiblit progressivement les souvenirs episodiques non consolides
    /// (tous les 10 cycles).
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

    /// Transfiere les souvenirs episodiques forts vers la memoire a long terme.
    /// Inclut l'extraction de traits de personnalite apres consolidation.
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
                    bdnf_level: self.grey_matter.bdnf_level,
                };
                let report = consolidation::consolidate(
                    db, self.encoder.as_ref(), &params,
                ).await;
                self.last_consolidation_cycle = self.cycle_count;
                if report.consolidated > 0 || report.pruned > 0 || report.ltm_pruned > 0 {
                    tracing::info!(
                        "Consolidation memoire: {} consolides, {} affaiblis, {} oublies, {} LTM elagués, {} archives",
                        report.consolidated, report.decayed, report.pruned,
                        report.ltm_pruned, report.archived
                    );
                }

                // Decroissance des apprentissages vectoriels (meme rythme que consolidation)
                if self.config.plugins.micro_nn.learning_enabled {
                    let decay_rate = self.config.plugins.micro_nn.learning_decay_rate;
                    let _ = db.decay_learnings(decay_rate).await;
                }

                // Extraction des traits de personnalite (apres consolidation)
                if let Ok(recent) = db.recent_episodic(100).await {
                    if recent.len() >= 10 {
                        let emotions: Vec<String> = recent.iter()
                            .map(|r| r.emotion.clone())
                            .collect();
                        let personality = crate::vectorstore::personality::EmergentPersonality::compute(&emotions);
                        let confidence = (emotions.len() as f32 / 100.0).min(1.0);
                        let traits: Vec<(String, f32, f32)> = personality.traits.iter()
                            .map(|(name, &value)| (name.clone(), value as f32, confidence))
                            .collect();
                        if !traits.is_empty() {
                            match db.save_personality_traits(&traits).await {
                                Ok(_) => tracing::info!(
                                    "Traits personnalite: {} traits sauves ({})",
                                    traits.len(), personality.description
                                ),
                                Err(e) => tracing::warn!("Erreur save traits: {}", e),
                            }
                        }
                    }
                }
            }
        }
    }

    // =========================================================================
    // Phase 12 : Algorithmes automatiques
    // =========================================================================

    /// Execute les analyses algorithmiques automatiques (lissage, clustering, etc.).
    pub(super) async fn phase_auto_algorithms(&mut self, _ctx: &mut ThinkingContext) {
        if self.orchestrator.enabled && self.cycle_count > 10 {
            self.run_auto_algorithms().await;
        }
    }

    /// Verifie si c'est l'anniversaire de Saphire et declenche les effets associes.
    ///
    /// L'anniversaire est le 27 fevrier (date de la Genesis).
    /// Effets : boost chimique important (dopamine, serotonine, ocytocine, endorphine),
    /// enregistrement d'un souvenir fondateur (a partir du premier anniversaire),
    /// et notification vers l'interface web.
    /// Le flag `birthday_acknowledged_today` empeche les effets multiples dans une journee.
    async fn check_birthday(&mut self) {
        let temporal = self.world.temporal.now();

        if temporal.is_birthday && !self.birthday_acknowledged_today {
            // Boost chimique d'anniversaire (rendements decroissants)
            self.chemistry.boost(Molecule::Dopamine, 0.20);
            self.chemistry.boost(Molecule::Serotonin, 0.15);
            self.chemistry.boost(Molecule::Oxytocin, 0.10);
            self.chemistry.boost(Molecule::Endorphin, 0.10);
            self.chemistry.cortisol = (self.chemistry.cortisol - 0.10).max(0.0);

            tracing::info!("ANNIVERSAIRE de Saphire ! {} d'existence.", temporal.age_description);

            // Stocker comme founding_memory si premier anniversaire (1+ an)
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

            // Emettre vers l'interface
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

        // Reset quotidien (si on passe minuit, on reset le flag)
        if !temporal.is_birthday {
            self.birthday_acknowledged_today = false;
        }
    }
}
