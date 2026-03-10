// =============================================================================
// factory.rs — Valeurs d'usine et mecanisme de reset
// =============================================================================
//
// Role : Charge les valeurs de conception originales de Saphire depuis
//        factory_defaults.toml (embarque dans le binaire). Fournit 9 niveaux
//        de reset : chimie, parametres, sens, intuition, ethique, psychologie,
//        sommeil, biologie, et reset complet.
//
// Dependances :
//   - toml : parsing du fichier TOML
//   - serde/serde_json : serialisation
//
// Place dans l'architecture :
//   Ce module est utilise par l'API REST et le WebSocket pour restaurer
//   les parametres d'usine quand une modification manuelle desequilibre
//   le systeme.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Le fichier factory_defaults.toml est embarque dans le binaire
const FACTORY_DEFAULTS_TOML: &str = include_str!("../factory_defaults.toml");

/// Niveaux de reset disponibles
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResetLevel {
    /// Reset chimie seulement — remet les 7 molecules aux baselines.
    /// Ne touche PAS aux souvenirs, personnalite, ou identite.
    ChemistryOnly,
    /// Reset parametres — remet tous les parametres de fonctionnement
    /// aux valeurs d'usine. Ne touche PAS aux souvenirs ou identite.
    ParametersOnly,
    /// Reset sens — remet les acuites des 5 sens aux valeurs initiales
    /// et reinitialise la stimulation des graines emergentes.
    /// Les sens germes sont PRESERVES.
    SensesOnly,
    /// Reset intuition + premonition — remet acuite et precision a l'initial.
    /// Efface les predictions actives mais pas les patterns appris.
    IntuitionOnly,
    /// Reset ethique personnelle — desactive tous les principes personnels.
    /// NE TOUCHE PAS aux couches 0 (Suisse) et 1 (Asimov).
    PersonalEthicsOnly,
    /// Reset psychologie — reinitialise tous les frameworks psychologiques
    /// (Freud, Maslow, Tolteques, Jung, EQ, Flow) aux valeurs initiales.
    /// Ne touche PAS aux souvenirs, chimie, ou identite.
    PsychologyOnly,
    /// Reset sommeil — remet la pression de sommeil, les compteurs de fatigue.
    /// NE TOUCHE PAS : sleep_history, neural_connections, subconscient.
    SleepOnly,
    /// Reset biologie — remet les recepteurs (sensibilite=1.0, tolerance=0.0),
    /// le BDNF (0.5 baseline), et la matiere grise aux valeurs par defaut.
    /// Ne touche PAS aux souvenirs, chimie courante, ou identite.
    BiologyReset,
    /// Reset complet — remet TOUT aux valeurs d'usine.
    /// ATTENTION : efface les souvenirs episodiques et reinitialise le profil.
    /// Les founding_memories, la LTM, et l'etincelle sont PRESERVEES.
    /// Le beat_count du coeur n'est JAMAIS remis a zero.
    FullReset,
}

/// Parametres de fonctionnement extraits des valeurs d'usine
#[derive(Debug, Clone, Serialize)]
pub struct FactoryParams {
    // Chimie
    pub homeostasis_rate: f64,
    pub baselines: [f64; 7],
    // Cerveau
    pub threshold_yes: f64,
    pub threshold_no: f64,
    // Pensee
    pub thought_interval: u64,
    // LLM
    pub temperature: f64,
    pub max_tokens: u32,
    pub max_tokens_thought: u32,
    // Corps
    pub resting_bpm: f64,
    // Memoire
    pub wm_capacity: usize,
    pub wm_decay: f64,
    pub episodic_decay: f64,
    pub episodic_max: usize,
    pub consolidation_interval: u64,
    pub consolidation_threshold: f64,
    // Intuition
    pub intuition_initial_acuity: f64,
    pub intuition_initial_accuracy: f64,
    // Premonition
    pub premonition_initial_accuracy: f64,
    // Sens
    pub senses_detection_threshold: f64,
    pub reading_initial_acuity: f64,
    pub listening_initial_acuity: f64,
    pub contact_initial_acuity: f64,
    pub taste_initial_acuity: f64,
    pub ambiance_initial_acuity: f64,
    // Ethique
    pub ethics_max_principles: usize,
    pub ethics_cooldown_cycles: u64,
    // Apprentissages vectoriels (nn_learning)
    pub nn_learning_enabled: bool,
    pub nn_learning_cooldown: u64,
    pub nn_max_learnings: u64,
    pub nn_learning_decay_rate: f64,
    pub nn_min_conditions: u64,
    pub nn_learning_search_limit: u64,
    pub nn_learning_search_threshold: f64,
    // Sommeil
    pub sleep_enabled: bool,
    pub sleep_threshold: f64,
    pub forced_sleep_threshold: f64,
    // Subconscient
    pub subconscious_enabled: bool,
    pub subconscious_awake_activation: f64,
    // Recepteurs
    pub receptor_adaptation_rate: f64,
    pub receptor_recovery_rate: f64,
    // BDNF
    pub bdnf_dopamine_weight: f64,
    pub bdnf_novelty_bonus: f64,
    pub bdnf_flow_state_bonus: f64,
    pub bdnf_cortisol_penalty_weight: f64,
    pub bdnf_cortisol_penalty_threshold: f64,
    pub bdnf_homeostasis_rate: f64,
    pub bdnf_homeostasis_baseline: f64,
    pub bdnf_consolidation_floor: f64,
    pub bdnf_consolidation_range: f64,
    pub bdnf_connectome_boost_threshold: f64,
    pub bdnf_connectome_boost_factor: f64,
}

/// Gestionnaire des valeurs d'usine
pub struct FactoryDefaults {
    config: toml::Value,
}

impl FactoryDefaults {
    /// Charge les valeurs d'usine depuis le TOML embarque
    pub fn load() -> Result<Self, String> {
        let config: toml::Value = toml::from_str(FACTORY_DEFAULTS_TOML)
            .map_err(|e| format!("Erreur parsing factory_defaults.toml: {}", e))?;
        Ok(Self { config })
    }

    /// Retourne les 7 baselines chimiques d'usine
    /// Ordre : [dopamine, cortisol, serotonin, adrenaline, oxytocin, endorphin, noradrenaline]
    pub fn reset_chemistry(&self) -> [f64; 7] {
        let b = &self.config["chemistry"]["baselines"];
        [
            self.val(b, "dopamine", 0.50),
            self.val(b, "cortisol", 0.15),
            self.val(b, "serotonin", 0.60),
            self.val(b, "adrenaline", 0.10),
            self.val(b, "oxytocin", 0.40),
            self.val(b, "endorphin", 0.40),
            self.val(b, "noradrenaline", 0.40),
        ]
    }

    /// Retourne tous les parametres d'usine
    pub fn reset_parameters(&self) -> FactoryParams {
        let baselines = self.reset_chemistry();
        FactoryParams {
            homeostasis_rate: self.get_f64("chemistry.homeostasis.rate", 0.03),
            baselines,
            threshold_yes: self.get_f64("brain.consensus.threshold_yes", 0.33),
            threshold_no: self.get_f64("brain.consensus.threshold_no", -0.33),
            thought_interval: self.get_u64("thought.interval_seconds", 15),
            temperature: self.get_f64("llm.temperature", 0.7),
            max_tokens: self.get_u64("llm.max_tokens", 800) as u32,
            max_tokens_thought: self.get_u64("llm.max_tokens_thought", 600) as u32,
            resting_bpm: self.get_f64("body.resting_bpm", 72.0),
            wm_capacity: self.get_u64("memory.working.capacity", 7) as usize,
            wm_decay: self.get_f64("memory.working.decay_rate", 0.05),
            episodic_decay: self.get_f64("memory.episodic.decay_rate", 0.05),
            episodic_max: self.get_u64("memory.episodic.max_count", 500) as usize,
            consolidation_interval: self.get_u64("memory.consolidation.interval_cycles", 50),
            consolidation_threshold: self.get_f64("memory.consolidation.threshold", 0.35),
            // Intuition
            intuition_initial_acuity: self.get_f64("intuition.initial_acuity", 0.2),
            intuition_initial_accuracy: self.get_f64("intuition.initial_accuracy", 0.5),
            // Premonition
            premonition_initial_accuracy: self.get_f64("premonition.initial_accuracy", 0.5),
            // Sens
            senses_detection_threshold: self.get_f64("senses.detection_threshold", 0.1),
            reading_initial_acuity: self.get_f64("senses.reading.initial_acuity", 0.3),
            listening_initial_acuity: self.get_f64("senses.listening.initial_acuity", 0.3),
            contact_initial_acuity: self.get_f64("senses.contact.initial_acuity", 0.3),
            taste_initial_acuity: self.get_f64("senses.taste.initial_acuity", 0.3),
            ambiance_initial_acuity: self.get_f64("senses.ambiance.initial_acuity", 0.2),
            // Ethique
            ethics_max_principles: self.get_u64("ethics.personal.max_personal_principles", 20) as usize,
            ethics_cooldown_cycles: self.get_u64("ethics.personal.formulation_cooldown_cycles", 100),
            // Apprentissages vectoriels
            nn_learning_enabled: self.get_bool("nn_learning.enabled", true),
            nn_learning_cooldown: self.get_u64("nn_learning.cooldown_cycles", 15),
            nn_max_learnings: self.get_u64("nn_learning.max_learnings", 200),
            nn_learning_decay_rate: self.get_f64("nn_learning.decay_rate", 0.02),
            nn_min_conditions: self.get_u64("nn_learning.min_conditions", 2),
            nn_learning_search_limit: self.get_u64("nn_learning.search_limit", 3),
            nn_learning_search_threshold: self.get_f64("nn_learning.search_threshold", 0.35),
            // Sommeil
            sleep_enabled: self.get_bool("sleep.enabled", true),
            sleep_threshold: self.get_f64("sleep.sleep_threshold", 0.7),
            forced_sleep_threshold: self.get_f64("sleep.forced_sleep_threshold", 0.95),
            // Subconscient
            subconscious_enabled: self.get_bool("subconscious.enabled", true),
            subconscious_awake_activation: self.get_f64("subconscious.awake_activation", 0.2),
            // Recepteurs
            receptor_adaptation_rate: self.get_f64("receptors.adaptation_rate", 0.02),
            receptor_recovery_rate: self.get_f64("receptors.recovery_rate", 0.005),
            // BDNF
            bdnf_dopamine_weight: self.get_f64("bdnf.dopamine_weight", 0.15),
            bdnf_novelty_bonus: self.get_f64("bdnf.novelty_bonus", 0.1),
            bdnf_flow_state_bonus: self.get_f64("bdnf.flow_state_bonus", 0.1),
            bdnf_cortisol_penalty_weight: self.get_f64("bdnf.cortisol_penalty_weight", 0.4),
            bdnf_cortisol_penalty_threshold: self.get_f64("bdnf.cortisol_penalty_threshold", 0.6),
            bdnf_homeostasis_rate: self.get_f64("bdnf.homeostasis_rate", 0.01),
            bdnf_homeostasis_baseline: self.get_f64("bdnf.homeostasis_baseline", 0.5),
            bdnf_consolidation_floor: self.get_f64("bdnf.consolidation_floor", 0.8),
            bdnf_consolidation_range: self.get_f64("bdnf.consolidation_range", 0.4),
            bdnf_connectome_boost_threshold: self.get_f64("bdnf.connectome_boost_threshold", 0.4),
            bdnf_connectome_boost_factor: self.get_f64("bdnf.connectome_boost_factor", 0.5),
        }
    }

    /// Retourne tout le contenu du factory_defaults.toml en JSON
    pub fn as_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.config).unwrap_or_default()
    }

    /// Naviguer dans le TOML par chemin pointe (ex: "chemistry.baselines.dopamine")
    pub fn get_f64(&self, path: &str, default: f64) -> f64 {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.config;
        for part in &parts {
            current = match current.get(part) {
                Some(v) => v,
                None => return default,
            };
        }
        current.as_float().unwrap_or(default)
    }

    fn get_bool(&self, path: &str, default: bool) -> bool {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.config;
        for part in &parts {
            current = match current.get(part) {
                Some(v) => v,
                None => return default,
            };
        }
        current.as_bool().unwrap_or(default)
    }

    fn get_u64(&self, path: &str, default: u64) -> u64 {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.config;
        for part in &parts {
            current = match current.get(part) {
                Some(v) => v,
                None => return default,
            };
        }
        current.as_integer().unwrap_or(default as i64) as u64
    }

    fn val(&self, table: &toml::Value, key: &str, default: f64) -> f64 {
        table.get(key)
            .and_then(|v| v.as_float())
            .unwrap_or(default)
    }
}
