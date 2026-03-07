// =============================================================================
// lifecycle/controls.rs — Controles interactifs (API REST) (version lite)
// =============================================================================
//
// Version allegee pour le papier ArXiv.
// Supprime : needs_status, knowledge_stats, suggest_topic,
//            apply_cognitive_profile, apply_personality_preset,
//            load_and_apply_personality, load_and_apply_profile.
// Simplifie : set_module_weight, set_threshold, set_param, config_json
//             utilisent config.consensus et config.feedback au lieu de tuner.
// =============================================================================

use tokio::time::Duration;

use crate::config::SaphireConfig;

use super::SaphireAgent;

impl SaphireAgent {
    /// Retourne les statistiques de la memoire de travail pour l'API REST.
    pub fn memory_data(&self) -> serde_json::Value {
        serde_json::json!({
            "working": self.working_memory.ws_data(),
        })
    }

    /// Retourne une reference vers la configuration globale de Saphire.
    pub fn config(&self) -> &SaphireConfig {
        &self.config
    }

    /// Retourne le nom du modele LLM actuellement utilise.
    pub fn llm_model(&self) -> &str {
        self.llm.model_name()
    }

    /// Retourne l'intervalle de temps entre deux pensees autonomes.
    /// Auto-ajuste si le LLM est plus lent que l'intervalle configure.
    pub fn thought_interval(&self) -> Duration {
        if self.avg_response_time > self.thought_interval.as_secs_f64() * 0.8 {
            Duration::from_secs_f64(self.avg_response_time * 1.5)
        } else {
            self.thought_interval
        }
    }

    // ─── Controles interactifs (API REST) ──────────────────────

    /// Modifie la baseline d'un neurotransmetteur.
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

    /// Ajuste une baseline par un offset.
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

    /// Modifie un seuil de consensus.
    /// - "yes" : seuil "Oui" ([0.0, 0.8])
    /// - "no"  : seuil "Non" ([-0.8, 0.0])
    pub fn set_threshold(&mut self, which: &str, value: f64) {
        match which {
            "yes" => self.config.consensus.threshold_yes = value.clamp(0.0, 0.8),
            "no" => self.config.consensus.threshold_no = value.clamp(-0.8, 0.0),
            _ => {}
        }
        tracing::info!("Threshold {} → {:.2}", which, value);
    }

    /// Modifie un parametre systeme de l'agent.
    /// - "thought_interval" : [5, 60] secondes
    /// - "homeostasis_rate" : [0.01, 0.20]
    /// - "temperature"      : [0.1, 1.5]
    pub fn set_param(&mut self, param: &str, value: f64) {
        match param {
            "thought_interval" => {
                let secs = value.clamp(5.0, 60.0) as u64;
                self.thought_interval = Duration::from_secs(secs);
            },
            "homeostasis_rate" => {
                self.config.feedback.homeostasis_rate = value.clamp(0.01, 0.20);
            },
            "temperature" => {
                self.config.llm.temperature = value.clamp(0.1, 1.5);
            },
            _ => {}
        }
        tracing::info!("Param {} → {:.2}", param, value);
    }

    /// Stabilisation d'urgence : remet la chimie dans un etat calme.
    pub fn emergency_stabilize(&mut self) {
        self.chemistry.cortisol = self.baselines.cortisol;
        self.chemistry.adrenaline = self.baselines.adrenaline;
        self.chemistry.serotonin = (self.chemistry.serotonin + 0.2).min(1.0);
        self.chemistry.endorphin = (self.chemistry.endorphin + 0.2).min(1.0);
        self.chemistry.dopamine = (self.chemistry.dopamine + 0.1).min(1.0);
        tracing::info!("Emergency stabilize applied");
    }

    /// Retourne l'etat du corps virtuel pour l'API.
    pub fn body_status(&self) -> crate::body::BodyStatus {
        self.body.status()
    }

    /// Retourne l'etat du coeur pour l'API.
    pub fn heart_status(&self) -> crate::body::heart::HeartStatus {
        self.body.heart.status()
    }

    /// Retourne l'etat chimique actuel en JSON pour l'API.
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
            "thresholds": {
                "yes": self.config.consensus.threshold_yes,
                "no": self.config.consensus.threshold_no,
            },
            "params": {
                "thought_interval": self.thought_interval.as_secs(),
                "homeostasis_rate": self.config.feedback.homeostasis_rate,
                "temperature": self.config.llm.temperature,
            }
        })
    }
}
