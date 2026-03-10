// =============================================================================
// logging/trace.rs — CognitiveTrace : trace complete d'un cycle cognitif
//
// Role : Structure qui accumule incrementalement les donnees de chaque
// etape du pipeline cognitif (NLP, brain, consensus, chimie, emotion,
// conscience, regulation, LLM, memoire) pour un cycle donne.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Trace cognitive complete d'un cycle.
/// Accumule les donnees de chaque etape du pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveTrace {
    pub cycle: u64,
    pub timestamp: DateTime<Utc>,
    pub source_type: String,
    pub input_text: String,
    pub nlp_data: serde_json::Value,
    pub brain_data: serde_json::Value,
    pub consensus_data: serde_json::Value,
    pub chemistry_before: serde_json::Value,
    pub chemistry_after: serde_json::Value,
    pub emotion_data: serde_json::Value,
    pub consciousness_data: serde_json::Value,
    pub regulation_data: serde_json::Value,
    pub llm_data: serde_json::Value,
    pub memory_data: serde_json::Value,
    pub heart_data: serde_json::Value,
    pub body_data: serde_json::Value,
    pub ethics_data: serde_json::Value,
    pub vital_data: serde_json::Value,
    pub intuition_data: serde_json::Value,
    pub premonition_data: serde_json::Value,
    pub senses_data: serde_json::Value,
    pub attention_data: serde_json::Value,
    pub algorithm_data: serde_json::Value,
    pub desire_data: serde_json::Value,
    pub learning_data: serde_json::Value,
    pub healing_data: serde_json::Value,
    pub psychology_data: serde_json::Value,
    pub will_data: serde_json::Value,
    pub nn_learning_data: serde_json::Value,
    pub subconscious_data: serde_json::Value,
    pub sleep_data: serde_json::Value,
    // ─── Modules cognitifs avances ─────────────────────
    pub tom_data: serde_json::Value,
    pub monologue_data: serde_json::Value,
    pub dissonance_data: serde_json::Value,
    pub prospective_data: serde_json::Value,
    pub narrative_data: serde_json::Value,
    pub analogical_data: serde_json::Value,
    pub cognitive_load_data: serde_json::Value,
    pub imagery_data: serde_json::Value,
    pub sentiments_data: serde_json::Value,
    // ─── Recepteurs et BDNF ─────────────────────
    pub receptor_data: serde_json::Value,
    pub bdnf_data: serde_json::Value,
    pub duration_ms: f32,
    pub session_id: i64,
}

impl CognitiveTrace {
    /// Cree une nouvelle trace pour un cycle donne.
    pub fn new(cycle: u64, source_type: &str, session_id: i64) -> Self {
        Self {
            cycle,
            timestamp: Utc::now(),
            source_type: source_type.to_string(),
            input_text: String::new(),
            nlp_data: serde_json::json!({}),
            brain_data: serde_json::json!({}),
            consensus_data: serde_json::json!({}),
            chemistry_before: serde_json::json!({}),
            chemistry_after: serde_json::json!({}),
            emotion_data: serde_json::json!({}),
            consciousness_data: serde_json::json!({}),
            regulation_data: serde_json::json!({}),
            llm_data: serde_json::json!({}),
            memory_data: serde_json::json!({}),
            heart_data: serde_json::json!({}),
            body_data: serde_json::json!({}),
            ethics_data: serde_json::json!({}),
            vital_data: serde_json::json!({}),
            intuition_data: serde_json::json!({}),
            premonition_data: serde_json::json!({}),
            senses_data: serde_json::json!({}),
            attention_data: serde_json::json!({}),
            algorithm_data: serde_json::json!({}),
            desire_data: serde_json::json!({}),
            learning_data: serde_json::json!({}),
            healing_data: serde_json::json!({}),
            psychology_data: serde_json::json!({}),
            will_data: serde_json::json!({}),
            nn_learning_data: serde_json::json!({}),
            subconscious_data: serde_json::json!({}),
            sleep_data: serde_json::json!({}),
            tom_data: serde_json::json!({}),
            monologue_data: serde_json::json!({}),
            dissonance_data: serde_json::json!({}),
            prospective_data: serde_json::json!({}),
            narrative_data: serde_json::json!({}),
            analogical_data: serde_json::json!({}),
            cognitive_load_data: serde_json::json!({}),
            imagery_data: serde_json::json!({}),
            sentiments_data: serde_json::json!({}),
            receptor_data: serde_json::json!({}),
            bdnf_data: serde_json::json!({}),
            duration_ms: 0.0,
            session_id,
        }
    }

    /// Enregistre le texte d'entree.
    pub fn set_input(&mut self, text: &str) {
        self.input_text = text.to_string();
    }

    /// Enregistre les donnees NLP.
    pub fn set_nlp(&mut self, data: serde_json::Value) {
        self.nlp_data = data;
    }

    /// Enregistre les signaux des modules cerebraux.
    pub fn set_brain(&mut self, data: serde_json::Value) {
        self.brain_data = data;
    }

    /// Enregistre le resultat du consensus.
    pub fn set_consensus(&mut self, data: serde_json::Value) {
        self.consensus_data = data;
    }

    /// Enregistre la chimie avant le cycle.
    pub fn set_chemistry_before(&mut self, data: serde_json::Value) {
        self.chemistry_before = data;
    }

    /// Enregistre la chimie apres le cycle.
    pub fn set_chemistry_after(&mut self, data: serde_json::Value) {
        self.chemistry_after = data;
    }

    /// Enregistre les donnees d'emotion.
    pub fn set_emotion(&mut self, data: serde_json::Value) {
        self.emotion_data = data;
    }

    /// Enregistre les donnees de conscience.
    pub fn set_consciousness(&mut self, data: serde_json::Value) {
        self.consciousness_data = data;
    }

    /// Enregistre les donnees de regulation.
    pub fn set_regulation(&mut self, data: serde_json::Value) {
        self.regulation_data = data;
    }

    /// Enregistre les donnees LLM.
    pub fn set_llm(&mut self, data: serde_json::Value) {
        self.llm_data = data;
    }

    /// Enregistre les donnees memoire.
    pub fn set_memory(&mut self, data: serde_json::Value) {
        self.memory_data = data;
    }

    /// Enregistre les donnees du coeur.
    pub fn set_heart(&mut self, data: serde_json::Value) {
        self.heart_data = data;
    }

    /// Enregistre les donnees du corps.
    pub fn set_body(&mut self, data: serde_json::Value) {
        self.body_data = data;
    }

    /// Enregistre les donnees ethiques.
    pub fn set_ethics(&mut self, data: serde_json::Value) {
        self.ethics_data = data;
    }

    /// Enregistre les donnees vitales (etincelle de vie).
    pub fn set_vital(&mut self, data: serde_json::Value) {
        self.vital_data = data;
    }

    /// Enregistre les donnees d'intuition.
    pub fn set_intuition(&mut self, data: serde_json::Value) {
        self.intuition_data = data;
    }

    /// Enregistre les donnees de premonition.
    pub fn set_premonition(&mut self, data: serde_json::Value) {
        self.premonition_data = data;
    }

    /// Enregistre les donnees sensorielles (Sensorium).
    pub fn set_senses(&mut self, data: serde_json::Value) {
        self.senses_data = data;
    }

    /// Enregistre les donnees d'attention (focus, fatigue, concentration).
    pub fn set_attention(&mut self, data: serde_json::Value) {
        self.attention_data = data;
    }

    /// Enregistre les donnees de desirs (actifs, top priorite, besoins).
    pub fn set_desires(&mut self, data: serde_json::Value) {
        self.desire_data = data;
    }

    /// Enregistre les donnees d'apprentissage (lecons ce cycle).
    pub fn set_learning(&mut self, data: serde_json::Value) {
        self.learning_data = data;
    }

    /// Enregistre les donnees de guerison (blessures, resilience).
    pub fn set_healing(&mut self, data: serde_json::Value) {
        self.healing_data = data;
    }

    /// Enregistre les donnees psychologiques (6 cadres).
    pub fn set_psychology(&mut self, data: serde_json::Value) {
        self.psychology_data = data;
    }

    /// Enregistre les donnees de volonte (deliberation).
    pub fn set_will(&mut self, data: serde_json::Value) {
        self.will_data = data;
    }

    /// Enregistre les donnees d'apprentissage vectoriel (nn_learning).
    pub fn set_nn_learning(&mut self, data: serde_json::Value) {
        self.nn_learning_data = data;
    }

    /// Enregistre les donnees du subconscient.
    pub fn set_subconscious(&mut self, data: serde_json::Value) {
        self.subconscious_data = data;
    }

    /// Enregistre les donnees de sommeil.
    pub fn set_sleep(&mut self, data: serde_json::Value) {
        self.sleep_data = data;
    }

    /// Enregistre les donnees de la Theorie de l'Esprit.
    pub fn set_tom(&mut self, data: serde_json::Value) {
        self.tom_data = data;
    }

    /// Enregistre les donnees du monologue interieur.
    pub fn set_monologue(&mut self, data: serde_json::Value) {
        self.monologue_data = data;
    }

    /// Enregistre les donnees de dissonance cognitive.
    pub fn set_dissonance(&mut self, data: serde_json::Value) {
        self.dissonance_data = data;
    }

    /// Enregistre les donnees de memoire prospective.
    pub fn set_prospective(&mut self, data: serde_json::Value) {
        self.prospective_data = data;
    }

    /// Enregistre les donnees d'identite narrative.
    pub fn set_narrative(&mut self, data: serde_json::Value) {
        self.narrative_data = data;
    }

    /// Enregistre les donnees de raisonnement analogique.
    pub fn set_analogical(&mut self, data: serde_json::Value) {
        self.analogical_data = data;
    }

    /// Enregistre les donnees de charge cognitive.
    pub fn set_cognitive_load(&mut self, data: serde_json::Value) {
        self.cognitive_load_data = data;
    }

    /// Enregistre les donnees d'imagerie mentale.
    pub fn set_imagery(&mut self, data: serde_json::Value) {
        self.imagery_data = data;
    }

    /// Enregistre les donnees du systeme de sentiments.
    pub fn set_sentiments(&mut self, data: serde_json::Value) {
        self.sentiments_data = data;
    }

    /// Enregistre les donnees de sensibilite des recepteurs.
    pub fn set_receptors(&mut self, data: serde_json::Value) {
        self.receptor_data = data;
    }

    /// Enregistre les donnees BDNF et matiere grise.
    pub fn set_bdnf(&mut self, data: serde_json::Value) {
        self.bdnf_data = data;
    }

    /// Enregistre la duree totale du cycle.
    pub fn set_duration(&mut self, ms: f32) {
        self.duration_ms = ms;
    }
}
