// =============================================================================
// lifecycle/controls.rs — Controles interactifs (API REST)
// =============================================================================

use tokio::time::Duration;

use crate::config::SaphireConfig;

use super::SaphireAgent;

impl SaphireAgent {
    /// Retourne les statistiques de la memoire de travail pour l'API REST (synchrone).
    pub fn memory_data(&self) -> serde_json::Value {
        serde_json::json!({
            "working": self.working_memory.ws_data(),
        })
    }

    /// Retourne une reference vers la configuration globale de Saphire.
    pub fn config(&self) -> &SaphireConfig {
        &self.config
    }

    /// Retourne le nom du modele LLM actuellement utilise (par ex. "llama3", "gpt-4").
    pub fn llm_model(&self) -> &str {
        self.llm.model_name()
    }

    /// Retourne l'intervalle de temps entre deux pensees autonomes.
    ///
    /// L'intervalle est auto-ajuste : si le temps de reponse moyen du LLM
    /// depasse 80% de l'intervalle configure, l'intervalle est elargi a
    /// 1.5x le temps de reponse moyen pour eviter l'accumulation de requetes.
    pub fn thought_interval(&self) -> Duration {
        // Auto-ajustement si le LLM est plus lent que l'intervalle de pensee
        if self.avg_response_time > self.thought_interval.as_secs_f64() * 0.8 {
            Duration::from_secs_f64(self.avg_response_time * 1.5)
        } else {
            self.thought_interval
        }
    }

    // ─── Controles interactifs (API REST) ──────────────────────

    /// Modifie la baseline (niveau de repos) d'un neurotransmetteur.
    ///
    /// La valeur est clampee entre 0.0 et 1.0. Les molecules supportees :
    /// dopamine, cortisol, serotonin, adrenaline, oxytocin, endorphin, noradrenaline.
    ///
    /// Parametres :
    /// - `molecule` : nom du neurotransmetteur (en anglais, minuscules).
    /// - `value` : nouvelle valeur de baseline (sera clampee a [0.0, 1.0]).
    pub fn set_baseline(&mut self, molecule: &str, value: f64) {
        let v = value.clamp(0.0, 1.0);
        match molecule {
            "dopamine" => self.baselines.dopamine = v,
            "cortisol" => self.baselines.cortisol = v,
            "serotonin" => self.baselines.serotonin = v,
            "adrenaline" => self.baselines.adrenaline = v,
            "oxytocin" => self.baselines.oxytocin = v,
            "endorphin" => self.baselines.endorphin = v,
            "noradrenaline" => self.baselines.noradrenaline = v,
            _ => {}
        }
        tracing::info!("Baseline {} → {:.2}", molecule, v);
    }

    /// Ajuste une baseline par un offset (positif ou negatif).
    /// Utilise par le genome pour appliquer les predispositions chimiques.
    pub fn adjust_baseline(&mut self, molecule: &str, offset: f64) {
        let current = match molecule {
            "dopamine" => self.baselines.dopamine,
            "cortisol" => self.baselines.cortisol,
            "serotonin" => self.baselines.serotonin,
            "adrenaline" => self.baselines.adrenaline,
            "oxytocin" => self.baselines.oxytocin,
            "endorphin" => self.baselines.endorphin,
            "noradrenaline" => self.baselines.noradrenaline,
            _ => return,
        };
        self.set_baseline(molecule, current + offset);
    }

    /// Modifie le poids de base d'un module cerebral dans le consensus.
    ///
    /// La valeur est clampee entre 0.1 et 3.0. Les modules supportes :
    /// reptilian, limbic, neocortex.
    ///
    /// Parametres :
    /// - `module` : nom du module cerebral (en anglais, minuscules).
    /// - `value` : nouveau poids de base (sera clampe a [0.1, 3.0]).
    pub fn set_module_weight(&mut self, module: &str, value: f64) {
        let v = value.clamp(0.1, 3.0);
        match module {
            "reptilian" => self.tuner.current_params.weight_base_reptilian = v,
            "limbic" => self.tuner.current_params.weight_base_limbic = v,
            "neocortex" => self.tuner.current_params.weight_base_neocortex = v,
            _ => {}
        }
        tracing::info!("Module weight {} → {:.2}", module, v);
    }

    /// Modifie un seuil de consensus (decision Oui ou Non).
    ///
    /// - "yes" : seuil au-dessus duquel le consensus est "Oui" ([0.0, 0.8]).
    /// - "no" : seuil en-dessous duquel le consensus est "Non" ([-0.8, 0.0]).
    ///   Entre les deux seuils, la decision est "Peut-etre" (Maybe).
    ///
    /// Parametres :
    /// - `which` : "yes" ou "no".
    /// - `value` : nouvelle valeur du seuil.
    pub fn set_threshold(&mut self, which: &str, value: f64) {
        match which {
            "yes" => self.tuner.current_params.threshold_yes = value.clamp(0.0, 0.8),
            "no" => self.tuner.current_params.threshold_no = value.clamp(-0.8, 0.0),
            _ => {}
        }
        tracing::info!("Threshold {} → {:.2}", which, value);
    }

    /// Modifie un parametre systeme de l'agent.
    ///
    /// Parametres supportes :
    /// - "thought_interval" : intervalle entre pensees autonomes ([5, 60] secondes).
    /// - "homeostasis_rate" : vitesse de retour vers les baselines ([0.01, 0.10]).
    /// - "indecision_stress" : stress chimique cause par l'indecision ([0.01, 0.15]).
    /// - "temperature" : temperature du LLM ([0.1, 1.5], plus haut = plus creatif).
    ///
    /// Parametres :
    /// - `param` : nom du parametre (en anglais, minuscules).
    /// - `value` : nouvelle valeur (sera clampee selon le parametre).
    pub fn set_param(&mut self, param: &str, value: f64) {
        match param {
            "thought_interval" => {
                let secs = value.clamp(5.0, 60.0) as u64;
                self.thought_interval = Duration::from_secs(secs);
            },
            "homeostasis_rate" => {
                self.tuner.current_params.homeostasis_rate = value.clamp(0.01, 0.20);
            },
            "indecision_stress" => {
                self.tuner.current_params.feedback_indecision_stress = value.clamp(0.01, 0.15);
            },
            "temperature" => {
                self.config.llm.temperature = value.clamp(0.1, 1.5);
            },
            _ => {}
        }
        tracing::info!("Param {} → {:.2}", param, value);
    }

    /// Stabilisation d'urgence : remet immediatement les neurotransmetteurs
    /// dans un etat calme et equilibre.
    ///
    /// Utilise quand Saphire est dans un etat chimique extreme (stress tres
    /// eleve, panique, etc.). Le cortisol et l'adrenaline sont remis a leurs
    /// baselines, et la serotonine, endorphine et dopamine recoivent un boost.
    pub fn emergency_stabilize(&mut self) {
        self.chemistry.cortisol = self.baselines.cortisol;
        self.chemistry.adrenaline = self.baselines.adrenaline;
        self.chemistry.serotonin = (self.chemistry.serotonin + 0.2).min(1.0);
        self.chemistry.endorphin = (self.chemistry.endorphin + 0.2).min(1.0);
        self.chemistry.dopamine = (self.chemistry.dopamine + 0.1).min(1.0);
        tracing::info!("Emergency stabilize applied");
    }

    /// Retourne l'etat des besoins primaires (faim, soif) pour l'API.
    pub fn needs_status(&self) -> serde_json::Value {
        self.needs.to_status_json()
    }

    /// Retourne l'etat complet du corps virtuel pour l'API.
    pub fn body_status(&self) -> crate::body::BodyStatus {
        self.body.status()
    }

    /// Retourne l'etat du coeur pour l'API.
    pub fn heart_status(&self) -> crate::body::heart::HeartStatus {
        self.body.heart.status()
    }

    /// Retourne l'etat chimique actuel (9 neurotransmetteurs) en JSON pour l'API.
    pub fn chemistry_json(&self) -> serde_json::Value {
        serde_json::json!({
            "dopamine": self.chemistry.dopamine,
            "cortisol": self.chemistry.cortisol,
            "serotonin": self.chemistry.serotonin,
            "adrenaline": self.chemistry.adrenaline,
            "oxytocin": self.chemistry.oxytocin,
            "endorphin": self.chemistry.endorphin,
            "noradrenaline": self.chemistry.noradrenaline,
            "gaba": self.chemistry.gaba,
            "glutamate": self.chemistry.glutamate,
        })
    }

    /// Retourne la configuration modifiable actuelle en JSON pour l'API.
    /// Inclut : baselines chimiques, poids des modules, seuils de consensus,
    /// et parametres systeme (intervalle de pensee, homeostasie, temperature LLM).
    pub fn config_json(&self) -> serde_json::Value {
        serde_json::json!({
            "baselines": {
                "dopamine": self.baselines.dopamine,
                "cortisol": self.baselines.cortisol,
                "serotonin": self.baselines.serotonin,
                "adrenaline": self.baselines.adrenaline,
                "oxytocin": self.baselines.oxytocin,
                "endorphin": self.baselines.endorphin,
                "noradrenaline": self.baselines.noradrenaline,
            },
            "module_weights": {
                "reptilian": self.tuner.current_params.weight_base_reptilian,
                "limbic": self.baselines_limbic_weight(),
                "neocortex": self.tuner.current_params.weight_base_neocortex,
            },
            "thresholds": {
                "yes": self.tuner.current_params.threshold_yes,
                "no": self.tuner.current_params.threshold_no,
            },
            "params": {
                "thought_interval": self.thought_interval.as_secs(),
                "homeostasis_rate": self.tuner.current_params.homeostasis_rate,
                "indecision_stress": self.tuner.current_params.feedback_indecision_stress,
                "temperature": self.config.llm.temperature,
            }
        })
    }

    /// Accesseur interne pour le poids de base du module limbique.
    fn baselines_limbic_weight(&self) -> f64 {
        self.tuner.current_params.weight_base_limbic
    }

    /// Ajoute un sujet suggere par l'utilisateur a la file d'exploration web.
    ///
    /// Les sujets suggeres sont prioritaires lors du prochain cycle de recherche.
    ///
    /// Parametre : `topic` — le sujet a explorer (par ex. "physique quantique").
    pub fn suggest_topic(&mut self, topic: String) {
        tracing::info!("Sujet suggéré par l'utilisateur: {}", topic);
        self.knowledge.add_suggested_topic(topic);
    }

    /// Retourne les statistiques du module de connaissance web pour l'interface.
    /// Inclut : nombre total de sujets explores, nombre de recherches,
    /// 5 derniers sujets, et nombre de suggestions en attente.
    pub fn knowledge_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "total_explored": self.knowledge.topics_explored.len(),
            "total_searches": self.knowledge.total_searches,
            "recent_topics": self.knowledge.topics_explored.iter()
                .rev().take(5).collect::<Vec<_>>(),
            "suggested_pending": self.knowledge.suggested_topics.len(),
        })
    }

    /// Applique les surcharges d'un profil cognitif sur les baselines et parametres.
    /// Retourne la liste des changements effectues pour le log.
    pub fn apply_cognitive_profile(
        &mut self,
        overrides: &crate::orchestrators::cognitive_profile::ProfileOverrides,
    ) -> Vec<serde_json::Value> {
        let mut changes = Vec::new();

        macro_rules! apply_f64 {
            ($field:expr, $new_val:expr, $label:expr) => {
                if let Some(v) = $new_val {
                    let old = $field;
                    $field = v;
                    changes.push(serde_json::json!({
                        "param": $label, "old": old, "new": v,
                    }));
                }
            };
        }

        macro_rules! apply_u64 {
            ($field:expr, $new_val:expr, $label:expr) => {
                if let Some(v) = $new_val {
                    let old = $field;
                    $field = v;
                    changes.push(serde_json::json!({
                        "param": $label, "old": old, "new": v,
                    }));
                }
            };
        }

        macro_rules! apply_usize {
            ($field:expr, $new_val:expr, $label:expr) => {
                if let Some(v) = $new_val {
                    let old = $field;
                    $field = v;
                    changes.push(serde_json::json!({
                        "param": $label, "old": old, "new": v,
                    }));
                }
            };
        }

        // Baselines chimiques (7 molecules)
        apply_f64!(self.baselines.dopamine, overrides.baseline_dopamine, "baseline_dopamine");
        apply_f64!(self.baselines.cortisol, overrides.baseline_cortisol, "baseline_cortisol");
        apply_f64!(self.baselines.serotonin, overrides.baseline_serotonin, "baseline_serotonin");
        apply_f64!(self.baselines.adrenaline, overrides.baseline_adrenaline, "baseline_adrenaline");
        apply_f64!(self.baselines.oxytocin, overrides.baseline_oxytocin, "baseline_oxytocin");
        apply_f64!(self.baselines.endorphin, overrides.baseline_endorphin, "baseline_endorphin");
        apply_f64!(self.baselines.noradrenaline, overrides.baseline_noradrenaline, "baseline_noradrenaline");

        // Feedback : homeostasis
        apply_f64!(self.tuner.current_params.homeostasis_rate, overrides.homeostasis_rate, "homeostasis_rate");

        // Consensus : seuils
        apply_f64!(self.tuner.current_params.threshold_yes, overrides.threshold_yes, "threshold_yes");
        apply_f64!(self.tuner.current_params.threshold_no, overrides.threshold_no, "threshold_no");

        // Attention
        apply_f64!(self.attention_orch.concentration_capacity, overrides.initial_concentration, "attention_concentration");
        apply_f64!(self.attention_orch.fatigue_per_cycle, overrides.fatigue_per_cycle, "attention_fatigue_per_cycle");
        apply_f64!(self.attention_orch.recovery_per_cycle, overrides.recovery_per_cycle, "attention_recovery_per_cycle");

        // Desirs
        apply_usize!(self.desire_orch.max_active, overrides.desires_max_active, "desires_max_active");
        apply_f64!(self.desire_orch.min_dopamine_for_birth, overrides.desires_min_dopamine, "desires_min_dopamine");
        apply_f64!(self.desire_orch.max_cortisol_for_birth, overrides.desires_max_cortisol, "desires_max_cortisol");

        // Apprentissage
        apply_u64!(self.learning_orch.cycle_interval, overrides.learning_cycle_interval, "learning_cycle_interval");
        apply_f64!(self.learning_orch.initial_confidence, overrides.learning_initial_confidence, "learning_initial_confidence");
        apply_f64!(self.learning_orch.confirmation_boost, overrides.learning_confirmation_boost, "learning_confirmation_boost");
        apply_f64!(self.learning_orch.contradiction_penalty, overrides.learning_contradiction_penalty, "learning_contradiction_penalty");

        // Guerison
        apply_u64!(self.healing_orch.melancholy_threshold_cycles, overrides.healing_melancholy_threshold, "healing_melancholy_threshold");
        apply_f64!(self.healing_orch.loneliness_threshold_hours, overrides.healing_loneliness_hours, "healing_loneliness_hours");
        apply_f64!(self.healing_orch.overload_noradrenaline, overrides.healing_overload_noradrenaline, "healing_overload_noradrenaline");

        // Sommeil
        apply_f64!(self.config.sleep.sleep_threshold, overrides.sleep_threshold, "sleep_threshold");
        apply_u64!(self.config.sleep.time_factor_divisor, overrides.sleep_time_factor_divisor, "sleep_time_factor_divisor");
        apply_f64!(self.config.sleep.adrenaline_resistance, overrides.sleep_adrenaline_resistance, "sleep_adrenaline_resistance");

        // Poids des pensees (thought_weights)
        if let Some(ref weights) = overrides.thought_weights {
            for (key, &val) in weights {
                match key.as_str() {
                    "introspection" => apply_f64!(self.config.saphire.thought_weights.introspection, Some(val), "thought_weight_introspection"),
                    "exploration" => apply_f64!(self.config.saphire.thought_weights.exploration, Some(val), "thought_weight_exploration"),
                    "memory_reflection" => apply_f64!(self.config.saphire.thought_weights.memory_reflection, Some(val), "thought_weight_memory_reflection"),
                    "continuation" => apply_f64!(self.config.saphire.thought_weights.continuation, Some(val), "thought_weight_continuation"),
                    "existential" => apply_f64!(self.config.saphire.thought_weights.existential, Some(val), "thought_weight_existential"),
                    "self_analysis" => apply_f64!(self.config.saphire.thought_weights.self_analysis, Some(val), "thought_weight_self_analysis"),
                    "curiosity" => apply_f64!(self.config.saphire.thought_weights.curiosity, Some(val), "thought_weight_curiosity"),
                    "daydream" => apply_f64!(self.config.saphire.thought_weights.daydream, Some(val), "thought_weight_daydream"),
                    "temporal_awareness" => apply_f64!(self.config.saphire.thought_weights.temporal_awareness, Some(val), "thought_weight_temporal_awareness"),
                    "moral_reflection" => apply_f64!(self.config.saphire.thought_weights.moral_reflection, Some(val), "thought_weight_moral_reflection"),
                    "synthesis" => apply_f64!(self.config.saphire.thought_weights.synthesis, Some(val), "thought_weight_synthesis"),
                    _ => {}
                }
            }
        }

        tracing::info!("Profil cognitif applique : {} changements", changes.len());
        changes
    }

    /// Applique les surcharges d'un preset de personnalite sur les baselines et parametres.
    /// Retourne la liste des changements effectues pour le log.
    pub fn apply_personality_preset(
        &mut self,
        overrides: &crate::orchestrators::personality_preset::PersonalityOverrides,
    ) -> Vec<serde_json::Value> {
        let mut changes = Vec::new();

        macro_rules! apply_f64 {
            ($field:expr, $new_val:expr, $label:expr) => {
                if let Some(v) = $new_val {
                    let old = $field;
                    $field = v;
                    changes.push(serde_json::json!({
                        "param": $label, "old": old, "new": v,
                    }));
                }
            };
        }

        macro_rules! apply_u64 {
            ($field:expr, $new_val:expr, $label:expr) => {
                if let Some(v) = $new_val {
                    let old = $field;
                    $field = v;
                    changes.push(serde_json::json!({
                        "param": $label, "old": old, "new": v,
                    }));
                }
            };
        }

        macro_rules! apply_usize {
            ($field:expr, $new_val:expr, $label:expr) => {
                if let Some(v) = $new_val {
                    let old = $field;
                    $field = v;
                    changes.push(serde_json::json!({
                        "param": $label, "old": old, "new": v,
                    }));
                }
            };
        }

        // Baselines chimiques (7 molecules)
        apply_f64!(self.baselines.dopamine, overrides.baseline_dopamine, "baseline_dopamine");
        apply_f64!(self.baselines.cortisol, overrides.baseline_cortisol, "baseline_cortisol");
        apply_f64!(self.baselines.serotonin, overrides.baseline_serotonin, "baseline_serotonin");
        apply_f64!(self.baselines.adrenaline, overrides.baseline_adrenaline, "baseline_adrenaline");
        apply_f64!(self.baselines.oxytocin, overrides.baseline_oxytocin, "baseline_oxytocin");
        apply_f64!(self.baselines.endorphin, overrides.baseline_endorphin, "baseline_endorphin");
        apply_f64!(self.baselines.noradrenaline, overrides.baseline_noradrenaline, "baseline_noradrenaline");

        // Feedback : homeostasis
        apply_f64!(self.tuner.current_params.homeostasis_rate, overrides.homeostasis_rate, "homeostasis_rate");

        // Consensus : seuils
        apply_f64!(self.tuner.current_params.threshold_yes, overrides.threshold_yes, "threshold_yes");
        apply_f64!(self.tuner.current_params.threshold_no, overrides.threshold_no, "threshold_no");

        // Attention
        apply_f64!(self.attention_orch.concentration_capacity, overrides.initial_concentration, "attention_concentration");
        apply_f64!(self.attention_orch.fatigue_per_cycle, overrides.fatigue_per_cycle, "attention_fatigue_per_cycle");
        apply_f64!(self.attention_orch.recovery_per_cycle, overrides.recovery_per_cycle, "attention_recovery_per_cycle");

        // Desirs
        apply_usize!(self.desire_orch.max_active, overrides.desires_max_active, "desires_max_active");
        apply_f64!(self.desire_orch.min_dopamine_for_birth, overrides.desires_min_dopamine, "desires_min_dopamine");
        apply_f64!(self.desire_orch.max_cortisol_for_birth, overrides.desires_max_cortisol, "desires_max_cortisol");

        // Apprentissage
        apply_u64!(self.learning_orch.cycle_interval, overrides.learning_cycle_interval, "learning_cycle_interval");
        apply_f64!(self.learning_orch.initial_confidence, overrides.learning_initial_confidence, "learning_initial_confidence");
        apply_f64!(self.learning_orch.confirmation_boost, overrides.learning_confirmation_boost, "learning_confirmation_boost");
        apply_f64!(self.learning_orch.contradiction_penalty, overrides.learning_contradiction_penalty, "learning_contradiction_penalty");

        // Guerison
        apply_u64!(self.healing_orch.melancholy_threshold_cycles, overrides.healing_melancholy_threshold, "healing_melancholy_threshold");
        apply_f64!(self.healing_orch.loneliness_threshold_hours, overrides.healing_loneliness_hours, "healing_loneliness_hours");
        apply_f64!(self.healing_orch.overload_noradrenaline, overrides.healing_overload_noradrenaline, "healing_overload_noradrenaline");

        // Sommeil
        apply_f64!(self.config.sleep.sleep_threshold, overrides.sleep_threshold, "sleep_threshold");
        apply_u64!(self.config.sleep.time_factor_divisor, overrides.sleep_time_factor_divisor, "sleep_time_factor_divisor");
        apply_f64!(self.config.sleep.adrenaline_resistance, overrides.sleep_adrenaline_resistance, "sleep_adrenaline_resistance");

        // Poids des pensees (thought_weights)
        if let Some(ref weights) = overrides.thought_weights {
            for (key, &val) in weights {
                match key.as_str() {
                    "introspection" => apply_f64!(self.config.saphire.thought_weights.introspection, Some(val), "thought_weight_introspection"),
                    "exploration" => apply_f64!(self.config.saphire.thought_weights.exploration, Some(val), "thought_weight_exploration"),
                    "memory_reflection" => apply_f64!(self.config.saphire.thought_weights.memory_reflection, Some(val), "thought_weight_memory_reflection"),
                    "continuation" => apply_f64!(self.config.saphire.thought_weights.continuation, Some(val), "thought_weight_continuation"),
                    "existential" => apply_f64!(self.config.saphire.thought_weights.existential, Some(val), "thought_weight_existential"),
                    "self_analysis" => apply_f64!(self.config.saphire.thought_weights.self_analysis, Some(val), "thought_weight_self_analysis"),
                    "curiosity" => apply_f64!(self.config.saphire.thought_weights.curiosity, Some(val), "thought_weight_curiosity"),
                    "daydream" => apply_f64!(self.config.saphire.thought_weights.daydream, Some(val), "thought_weight_daydream"),
                    "temporal_awareness" => apply_f64!(self.config.saphire.thought_weights.temporal_awareness, Some(val), "thought_weight_temporal_awareness"),
                    "moral_reflection" => apply_f64!(self.config.saphire.thought_weights.moral_reflection, Some(val), "thought_weight_moral_reflection"),
                    "synthesis" => apply_f64!(self.config.saphire.thought_weights.synthesis, Some(val), "thought_weight_synthesis"),
                    _ => {}
                }
            }
        }

        // Interets (specifique personnalite) : remplacer initial_topics si present
        if let Some(ref topics) = overrides.interests {
            let old_count = self.config.saphire.interests.initial_topics.len();
            self.config.saphire.interests.initial_topics = topics.clone();
            changes.push(serde_json::json!({
                "param": "interests_initial_topics",
                "old_count": old_count,
                "new_count": topics.len(),
                "new_topics": topics,
            }));
        }

        tracing::info!("Preset de personnalite applique : {} changements", changes.len());
        changes
    }

    /// Charge un preset de personnalite par son ID, applique les surcharges et
    /// demarre une transition douce. Retourne le resultat JSON pour l'API.
    pub fn load_and_apply_personality(&mut self, id: &str) -> serde_json::Value {
        let preset = match self.personality_preset_orch.load_preset(id) {
            Ok(p) => p,
            Err(e) => {
                return serde_json::json!({ "error": e });
            }
        };

        let changes = self.apply_personality_preset(&preset.overrides);
        self.personality_preset_orch.start_transition(&self.baselines, &preset.overrides);

        serde_json::json!({
            "status": "ok",
            "preset": {
                "id": preset.id,
                "name": preset.name,
                "description": preset.description,
                "category": preset.category,
            },
            "changes_applied": changes.len(),
            "changes": changes,
            "transition_started": true,
            "transition_cycles": self.personality_preset_orch.transition_cycles,
        })
    }

    /// Charge un profil cognitif par son ID, applique les surcharges et
    /// demarre une transition douce. Retourne le resultat JSON pour l'API.
    pub fn load_and_apply_profile(&mut self, id: &str) -> serde_json::Value {
        let profile = match self.cognitive_profile_orch.load_profile(id) {
            Ok(p) => p,
            Err(e) => {
                return serde_json::json!({ "error": e });
            }
        };

        let changes = self.apply_cognitive_profile(&profile.overrides);
        self.cognitive_profile_orch.start_transition(&self.baselines, &profile.overrides);

        serde_json::json!({
            "status": "ok",
            "profile": {
                "id": profile.id,
                "name": profile.name,
                "description": profile.description,
                "category": profile.category,
            },
            "changes_applied": changes.len(),
            "changes": changes,
            "transition_started": true,
            "transition_cycles": self.cognitive_profile_orch.transition_cycles,
        })
    }
}
