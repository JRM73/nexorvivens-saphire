// =============================================================================
// tom.rs — Theory of Mind
// =============================================================================
//
// Models the interlocutor's mental state to adapt Saphire's responses.
// By analyzing sentiment, tone, and patterns of received messages,
// Saphire builds a model of the interlocutor's mood, comprehension
// level, and frustration.
//
// Detected empathy influences chemistry (oxytocin, cortisol) and
// enriches the substrate prompt with a portrait of the perceived mental state.
//
// Dependencies:
//  - std::collections::VecDeque: sliding mood history
//  - serde: config serialization (TOML)
//  - serde_json: JSON export for the API and WebSocket
//  - crate::world::ChemistryAdjustment: chemical influence
//
// Place in architecture:
//  Top-level module. Called at each incoming message to update
//  the interlocutor model. The cognitive pipeline consumes
//  chemistry_influence() and describe_for_prompt().
// =============================================================================

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

// --- Default value functions for serde ---

fn default_true() -> bool { true }
fn default_frustration_threshold() -> f64 { 0.6 }
fn default_comprehension_threshold() -> f64 { 0.4 }
fn default_mood_history_size() -> usize { 10 }

/// Configuration for the Theory of Mind module.
/// Loaded from the main TOML configuration file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TomConfig {
    /// Enables or disables the ToM module
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Threshold above which frustration is considered significant
    #[serde(default = "default_frustration_threshold")]
    pub frustration_threshold: f64,
    /// Threshold below which comprehension is deemed insufficient
    #[serde(default = "default_comprehension_threshold")]
    pub comprehension_threshold: f64,
    /// Number of mood values kept in the sliding history
    #[serde(default = "default_mood_history_size")]
    pub mood_history_size: usize,
}

impl Default for TomConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            frustration_threshold: default_frustration_threshold(),
            comprehension_threshold: default_comprehension_threshold(),
            mood_history_size: default_mood_history_size(),
        }
    }
}

/// Model of the interlocutor's mental state.
/// Built progressively from received messages.
#[derive(Debug, Clone)]
pub struct InterlocutorModel {
    /// Estimated mood (-1.0 = very negative, 0.0 = neutral, 1.0 = very positive)
    pub estimated_mood: f64,
    /// Estimated comprehension level (0.0 = confused, 1.0 = perfectly understands)
    pub comprehension_level: f64,
    /// Detected intents in messages (questions, requests, complaints, etc.)
    pub detected_intents: Vec<String>,
    /// Confidence in the model (increases with message count)
    pub model_confidence: f64,
    /// Detected frustration level (0.0 = calm, 1.0 = very frustrated)
    pub frustration_level: f64,
    /// Need for empathy vs need for information (0.0 = wants facts, 1.0 = wants support)
    pub empathy_need: f64,
    /// Sliding history of recent mood values
    pub mood_history: VecDeque<f64>,
    /// Total number of analyzed messages
    pub message_count: u64,
    /// Cycle of the last analyzed message
    pub last_update_cycle: u64,
    /// Detected dominant linguistic register
    pub detected_register: String,
    /// Estimated knowledge level (0.0 = novice, 1.0 = expert)
    pub knowledge_level: f64,
    /// Recurring interest topics with counter
    pub interest_topics: Vec<(String, u32)>,
    /// Interlocutor engagement (0.0 = passive, 1.0 = highly engaged)
    pub engagement: f64,
    /// Depth preference (0.0 = simple, 1.0 = deep)
    pub depth_preference: f64,
}

impl InterlocutorModel {
    /// Creates a blank initial model.
    fn new(mood_history_size: usize) -> Self {
        Self {
            estimated_mood: 0.0,
            comprehension_level: 0.5,
            detected_intents: Vec::new(),
            model_confidence: 0.0,
            frustration_level: 0.0,
            empathy_need: 0.0,
            mood_history: VecDeque::with_capacity(mood_history_size),
            message_count: 0,
            last_update_cycle: 0,
            detected_register: String::new(),
            knowledge_level: 0.5,
            interest_topics: Vec::new(),
            engagement: 0.5,
            depth_preference: 0.5,
        }
    }
}

/// Theory of Mind engine — builds and maintains a model of the
/// interlocutor's mental state from received messages.
pub struct TheoryOfMindEngine {
    /// Module enabled or not
    pub enabled: bool,
    /// Current interlocutor model (None if no message analyzed)
    pub current_model: Option<InterlocutorModel>,
    /// Significant frustration threshold
    pub frustration_threshold: f64,
    /// Insufficient comprehension threshold
    pub comprehension_threshold: f64,
    /// Maximum mood history size
    pub mood_history_size: usize,
}

impl TheoryOfMindEngine {
    /// Creates a new ToM engine from configuration.
    pub fn new(config: &TomConfig) -> Self {
        Self {
            enabled: config.enabled,
            current_model: None,
            frustration_threshold: config.frustration_threshold,
            comprehension_threshold: config.comprehension_threshold,
            mood_history_size: config.mood_history_size,
        }
    }

    /// Updates the model from a new incoming message.
    ///
    /// Parameters:
    ///  - text: message content
    ///  - sentiment_compound: NLP sentiment score (-1.0 to 1.0)
    ///  - cycle: current cognitive cycle number
    ///
    /// Creates the model if this is the first message.
    pub fn update_from_message(&mut self, text: &str, sentiment_compound: f64, cycle: u64) {
        if !self.enabled {
            return;
        }

        // Create the model if necessary
        let model = self.current_model.get_or_insert_with(|| {
            InterlocutorModel::new(self.mood_history_size)
        });

        model.message_count += 1;
        model.last_update_cycle = cycle;

        // --- Estimated mood (EMA, alpha = 0.3) ---
        // Exponential moving average to smooth sentiment variations
        let alpha = 0.3;
        model.estimated_mood = alpha * sentiment_compound + (1.0 - alpha) * model.estimated_mood;
        model.estimated_mood = model.estimated_mood.clamp(-1.0, 1.0);

        // Add to mood history
        if model.mood_history.len() >= self.mood_history_size {
            model.mood_history.pop_front();
        }
        model.mood_history.push_back(sentiment_compound);

        // --- Comprehension level (heuristic) ---
        // Short questions with "?" suggest lack of understanding
        let is_short = text.len() < 40;
        let has_question = text.contains('?');
        let text_lower = text.to_lowercase();
        let has_confusion_markers = text_lower.contains("comprends pas")
            || text_lower.contains("je ne sais pas")
            || text_lower.contains("c'est quoi")
            || text_lower.contains("hein")
            || text_lower.contains("pardon")
            || text_lower.contains("what")
            || text_lower.contains("unclear");

        if has_confusion_markers {
            model.comprehension_level = (model.comprehension_level - 0.15).max(0.0);
        } else if is_short && has_question {
            model.comprehension_level = (model.comprehension_level - 0.05).max(0.0);
        } else if text.len() > 100 && !has_question {
            // A long affirmative message suggests good comprehension
            model.comprehension_level = (model.comprehension_level + 0.05).min(1.0);
        } else {
            // Slow convergence toward 0.5 by default
            model.comprehension_level += (0.5 - model.comprehension_level) * 0.02;
        }

        // --- Frustration (repeated negative sentiment over the last 3) ---
        let recent_count = model.mood_history.len().min(3);
        if recent_count >= 2 {
            let recent_negative = model.mood_history.iter()
                .rev()
                .take(3)
                .filter(|&&m| m < -0.3)
                .count();
            if recent_negative >= 2 {
                // Negative sentiment on at least 2 of the last 3 messages
                model.frustration_level = (model.frustration_level + 0.15).min(1.0);
            } else if sentiment_compound < -0.3 {
                model.frustration_level = (model.frustration_level + 0.05).min(1.0);
            } else {
                // Slow decrease if tone improves
                model.frustration_level = (model.frustration_level - 0.03).max(0.0);
            }
        } else if sentiment_compound < -0.3 {
            model.frustration_level = (model.frustration_level + 0.1).min(1.0);
        }

        // --- Empathy need ---
        // Combines frustration and lack of comprehension
        model.empathy_need = (
            model.frustration_level * 0.5
            + (1.0 - model.comprehension_level) * 0.3
            + if sentiment_compound < -0.5 { 0.2 } else { 0.0 }
        ).clamp(0.0, 1.0);

        // --- Model confidence ---
        // Grows with messages, saturates at 0.9
        model.model_confidence = (model.message_count as f64 / 10.0).min(0.9);

        // --- Simple intent detection ---
        model.detected_intents.clear();
        if has_question {
            model.detected_intents.push("question".into());
        }
        if text_lower.contains("aide") || text_lower.contains("help") {
            model.detected_intents.push("demande_aide".into());
        }
        if text_lower.contains("merci") || text_lower.contains("thank") {
            model.detected_intents.push("gratitude".into());
        }
        if text_lower.contains("arret") || text_lower.contains("stop") || text_lower.contains("assez") {
            model.detected_intents.push("arret".into());
        }
        if has_confusion_markers {
            model.detected_intents.push("confusion".into());
        }
        if text_lower.contains("pourquoi") || text_lower.contains("why") {
            model.detected_intents.push("explication".into());
        }

        // --- Engagement (length + questions + frequency) ---
        let length_signal = (text.len() as f64 / 200.0).min(1.0);
        let question_signal = if has_question { 0.2 } else { 0.0 };
        let raw_engagement = length_signal * 0.6 + question_signal + 0.2;
        model.engagement = 0.3 * raw_engagement + 0.7 * model.engagement;
        model.engagement = model.engagement.clamp(0.0, 1.0);

        // --- Knowledge level (technical vocabulary + length + no confusion) ---
        let technical_markers = ["algorithme", "module", "api", "code", "fonction",
            "variable", "architecture", "protocole", "implementation",
            "algorithm", "function", "server", "database", "framework"];
        let tech_count = technical_markers.iter()
            .filter(|m| text_lower.contains(*m))
            .count();
        if tech_count >= 2 {
            model.knowledge_level = (model.knowledge_level + 0.05).min(1.0);
        } else if has_confusion_markers {
            model.knowledge_level = (model.knowledge_level - 0.05).max(0.0);
        }

        // --- Depth preference (philosophical/technical register = high depth) ---
        let philosophical_markers = ["conscience", "existence", "sens", "verite",
            "liberte", "ame", "ethique", "consciousness", "meaning", "truth"];
        let phil_count = philosophical_markers.iter()
            .filter(|m| text_lower.contains(*m))
            .count();
        if phil_count >= 1 || tech_count >= 2 {
            model.depth_preference = (model.depth_preference + 0.05).min(1.0);
        } else if is_short && !has_question {
            model.depth_preference += (0.4 - model.depth_preference) * 0.05;
        }

        // --- Interest topics (salient words of 5+ letters, max 10) ---
        let stop_words = ["dans", "avec", "pour", "cette", "mais", "aussi",
            "plus", "comme", "tout", "bien", "faire", "etre", "avoir",
            "sont", "nous", "vous", "leur", "entre", "cette", "quand"];
        for token in text_lower.split_whitespace() {
            let clean: String = token.chars().filter(|c| c.is_alphanumeric()).collect();
            if clean.len() >= 5 && !stop_words.contains(&clean.as_str()) {
                if let Some(entry) = model.interest_topics.iter_mut().find(|(t, _)| *t == clean) {
                    entry.1 += 1;
                } else if model.interest_topics.len() < 10 {
                    model.interest_topics.push((clean, 1));
                }
            }
        }
        // Keep the 10 most frequent
        model.interest_topics.sort_by(|a, b| b.1.cmp(&a.1));
        model.interest_topics.truncate(10);
    }

    /// Updates the detected linguistic register in the ToM model.
    pub fn update_register(&mut self, register_name: &str) {
        if let Some(model) = &mut self.current_model {
            model.detected_register = register_name.to_string();
        }
    }

    /// Returns the chemical adjustment based on the interlocutor's state.
    ///
    /// - High frustration -> cortisol+ (empathic stress)
    /// - Empathy need -> oxytocin+ (social bonding activation)
    /// - Positive interlocutor mood -> serotonin+ (shared well-being)
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        let model = match &self.current_model {
            Some(m) if self.enabled => m,
            _ => return adj,
        };

        // Detected frustration -> empathic stress (cortisol)
        if model.frustration_level > 0.3 {
            adj.cortisol += (model.frustration_level * 0.04).min(0.04);
        }

        // Empathy need -> oxytocin activation (desire to help)
        if model.empathy_need > 0.3 {
            adj.oxytocin += (model.empathy_need * 0.03).min(0.03);
        }

        // Positive interlocutor mood -> shared well-being
        if model.estimated_mood > 0.3 {
            adj.serotonin += (model.estimated_mood * 0.02).min(0.02);
            adj.dopamine += 0.01;
        }

        // Very negative mood -> slight adrenaline (empathic alert)
        if model.estimated_mood < -0.5 {
            adj.adrenaline += 0.01;
        }

        // Low comprehension -> noradrenaline (need for clarity)
        if model.comprehension_level < 0.3 {
            adj.noradrenaline += 0.01;
        }

        adj
    }

    /// Generates a textual description of the interlocutor's state
    /// to enrich the substrate prompt sent to the LLM.
    ///
    /// Format: "Mon interlocuteur semble [mood]..."
    pub fn describe_for_prompt(&self) -> String {
        let model = match &self.current_model {
            Some(m) => m,
            None => return "Je n'ai pas encore de modele de mon interlocuteur.".into(),
        };

        // Determine the mood description
        let mood_desc = if model.estimated_mood > 0.5 {
            "tres positif et enthousiaste"
        } else if model.estimated_mood > 0.2 {
            "plutot de bonne humeur"
        } else if model.estimated_mood > -0.2 {
            "neutre"
        } else if model.estimated_mood > -0.5 {
            "un peu contrarie"
        } else {
            "frustre ou mecontent"
        };

        // Determine the comprehension level
        let comp_desc = if model.comprehension_level > 0.7 {
            "comprend bien la conversation"
        } else if model.comprehension_level > 0.4 {
            "semble suivre globalement"
        } else {
            "semble confus ou perdu"
        };

        // Determine the need
        let need_desc = if model.empathy_need > 0.6 {
            "Il a surtout besoin d'empathie et de soutien."
        } else if model.empathy_need > 0.3 {
            "Il apprecierait un equilibre entre information et empathie."
        } else {
            "Il cherche principalement de l'information."
        };

        // Frustration?
        let frust_desc = if model.frustration_level > self.frustration_threshold {
            format!(
                " Attention : niveau de frustration eleve ({:.0}%).",
                model.frustration_level * 100.0
            )
        } else {
            String::new()
        };

        // Knowledge level
        let knowledge_desc = if model.knowledge_level > 0.7 {
            " Expert."
        } else if model.knowledge_level < 0.3 {
            " Novice."
        } else {
            ""
        };

        // Depth preference
        let depth_desc = if model.depth_preference > 0.7 {
            " Aime la profondeur."
        } else if model.depth_preference < 0.3 {
            " Prefere la simplicite."
        } else {
            ""
        };

        format!(
            "Interlocuteur {} (humeur {:.2}). {} {}{}{}{}",
            mood_desc,
            model.estimated_mood,
            comp_desc,
            need_desc,
            knowledge_desc,
            depth_desc,
            frust_desc,
        )
    }

    /// Returns the prompt description only if the module is active
    /// and a model exists. Otherwise returns None.
    pub fn describe_for_prompt_if_active(&self) -> Option<String> {
        if !self.enabled || self.current_model.is_none() {
            return None;
        }
        Some(self.describe_for_prompt())
    }

    /// Resets the interlocutor model.
    /// Used when a new interlocutor joins the conversation
    /// or during a reset.
    pub fn reset_model(&mut self) {
        self.current_model = None;
    }

    /// Serializes the complete state to JSON for the API and WebSocket.
    pub fn to_json(&self) -> serde_json::Value {
        match &self.current_model {
            Some(model) => serde_json::json!({
                "enabled": self.enabled,
                "has_model": true,
                "estimated_mood": model.estimated_mood,
                "comprehension_level": model.comprehension_level,
                "frustration_level": model.frustration_level,
                "empathy_need": model.empathy_need,
                "model_confidence": model.model_confidence,
                "message_count": model.message_count,
                "last_update_cycle": model.last_update_cycle,
                "detected_intents": model.detected_intents,
                "detected_register": model.detected_register,
                "knowledge_level": model.knowledge_level,
                "engagement": model.engagement,
                "depth_preference": model.depth_preference,
                "interest_topics": model.interest_topics.iter()
                    .map(|(t, c)| serde_json::json!({"topic": t, "count": c}))
                    .collect::<Vec<_>>(),
                "mood_history": model.mood_history.iter().collect::<Vec<_>>(),
                "frustration_threshold": self.frustration_threshold,
                "comprehension_threshold": self.comprehension_threshold,
            }),
            None => serde_json::json!({
                "enabled": self.enabled,
                "has_model": false,
            }),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn default_engine() -> TheoryOfMindEngine {
        TheoryOfMindEngine::new(&TomConfig::default())
    }

    #[test]
    fn test_new_engine_has_no_model() {
        let engine = default_engine();
        assert!(engine.current_model.is_none());
        assert!(engine.enabled);
    }

    #[test]
    fn test_first_message_creates_model() {
        let mut engine = default_engine();
        engine.update_from_message("Bonjour!", 0.5, 1);
        assert!(engine.current_model.is_some());
        let model = engine.current_model.as_ref().unwrap();
        assert_eq!(model.message_count, 1);
        assert!(model.estimated_mood > 0.0);
    }

    #[test]
    fn test_ema_smoothing() {
        let mut engine = default_engine();
        // First positive message
        engine.update_from_message("Super!", 0.8, 1);
        let mood1 = engine.current_model.as_ref().unwrap().estimated_mood;
        // Second negative message — the EMA should not swing entirely
        engine.update_from_message("Nul.", -0.5, 2);
        let mood2 = engine.current_model.as_ref().unwrap().estimated_mood;
        assert!(mood2 < mood1, "Mood should decrease");
        assert!(mood2 > -0.5, "The EMA should smooth the drop");
    }

    #[test]
    fn test_frustration_rises_on_repeated_negative() {
        let mut engine = default_engine();
        engine.update_from_message("Ca ne marche pas.", -0.6, 1);
        engine.update_from_message("Toujours pas!", -0.7, 2);
        engine.update_from_message("Rien ne fonctionne.", -0.5, 3);
        let model = engine.current_model.as_ref().unwrap();
        assert!(model.frustration_level > 0.2, "Frustration should rise");
    }

    #[test]
    fn test_chemistry_frustration_cortisol() {
        let mut engine = default_engine();
        // Force high frustration
        engine.update_from_message("Nul!", -0.8, 1);
        engine.update_from_message("Ca ne marche pas!", -0.7, 2);
        engine.update_from_message("Horrible!", -0.9, 3);
        let adj = engine.chemistry_influence();
        assert!(adj.cortisol > 0.0, "Frustration should increase cortisol");
    }

    #[test]
    fn test_chemistry_positive_mood() {
        let mut engine = default_engine();
        // Multiple positive messages to reach mood > 0.3
        engine.update_from_message("Genial!", 0.9, 1);
        engine.update_from_message("Super!", 0.8, 2);
        engine.update_from_message("Excellent!", 0.9, 3);
        let adj = engine.chemistry_influence();
        assert!(adj.serotonin > 0.0, "A positive mood should increase serotonin");
    }

    #[test]
    fn test_describe_for_prompt_no_model() {
        let engine = default_engine();
        let desc = engine.describe_for_prompt();
        assert!(desc.contains("pas encore"), "Should indicate the absence of a model");
    }

    #[test]
    fn test_describe_for_prompt_if_active_none() {
        let engine = default_engine();
        assert!(engine.describe_for_prompt_if_active().is_none());
    }

    #[test]
    fn test_describe_for_prompt_if_active_some() {
        let mut engine = default_engine();
        engine.update_from_message("Bonjour!", 0.5, 1);
        assert!(engine.describe_for_prompt_if_active().is_some());
    }

    #[test]
    fn test_reset_model() {
        let mut engine = default_engine();
        engine.update_from_message("Salut", 0.3, 1);
        assert!(engine.current_model.is_some());
        engine.reset_model();
        assert!(engine.current_model.is_none());
    }

    #[test]
    fn test_disabled_engine_ignores_messages() {
        let config = TomConfig { enabled: false, ..Default::default() };
        let mut engine = TheoryOfMindEngine::new(&config);
        engine.update_from_message("Hello", 0.5, 1);
        assert!(engine.current_model.is_none());
    }

    #[test]
    fn test_confidence_grows_with_messages() {
        let mut engine = default_engine();
        for i in 0..10 {
            engine.update_from_message("Message", 0.0, i);
        }
        let model = engine.current_model.as_ref().unwrap();
        assert!(model.model_confidence >= 0.9, "Confidence should saturate at 0.9");
    }

    #[test]
    fn test_to_json_no_model() {
        let engine = default_engine();
        let json = engine.to_json();
        assert_eq!(json["has_model"], false);
    }

    #[test]
    fn test_to_json_with_model() {
        let mut engine = default_engine();
        engine.update_from_message("Test", 0.2, 1);
        let json = engine.to_json();
        assert_eq!(json["has_model"], true);
        assert!(json["estimated_mood"].is_number());
        assert!(json["message_count"].is_number());
    }

    #[test]
    fn test_intent_detection_question() {
        let mut engine = default_engine();
        engine.update_from_message("Qu'est-ce que c'est?", 0.0, 1);
        let model = engine.current_model.as_ref().unwrap();
        assert!(model.detected_intents.contains(&"question".into()));
    }

    #[test]
    fn test_intent_detection_confusion() {
        let mut engine = default_engine();
        engine.update_from_message("Je comprends pas du tout", -0.3, 1);
        let model = engine.current_model.as_ref().unwrap();
        assert!(model.detected_intents.contains(&"confusion".into()));
    }

    #[test]
    fn test_mood_history_capacity() {
        let config = TomConfig {
            mood_history_size: 3,
            ..Default::default()
        };
        let mut engine = TheoryOfMindEngine::new(&config);
        for i in 0..5 {
            engine.update_from_message("Msg", 0.1 * i as f64, i);
        }
        let model = engine.current_model.as_ref().unwrap();
        assert_eq!(model.mood_history.len(), 3, "History should not exceed capacity");
    }
}
