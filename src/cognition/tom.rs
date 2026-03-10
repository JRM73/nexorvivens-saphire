// =============================================================================
// tom.rs — Theorie de l'Esprit (Theory of Mind)
// =============================================================================
//
// Modelise l'etat mental de l'interlocuteur pour adapter les reponses
// de Saphire. En analysant le sentiment, le ton et les patterns de
// messages recus, Saphire construit un modele de l'humeur, du niveau
// de comprehension et de frustration de l'interlocuteur.
//
// L'empathie detectee influence la chimie (ocytocine, cortisol) et
// enrichit le prompt substrat avec un portrait de l'etat mental percu.
//
// Dependances :
//   - std::collections::VecDeque : historique d'humeur glissant
//   - serde : serialisation de la config (TOML)
//   - serde_json : export JSON pour l'API et le WebSocket
//   - crate::world::ChemistryAdjustment : influence chimique
//
// Place dans l'architecture :
//   Module de premier niveau. Appele a chaque message entrant pour
//   mettre a jour le modele de l'interlocuteur. Le pipeline cognitif
//   consomme chemistry_influence() et describe_for_prompt().
// =============================================================================

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

// --- Fonctions de valeurs par defaut pour serde ---

fn default_true() -> bool { true }
fn default_frustration_threshold() -> f64 { 0.6 }
fn default_comprehension_threshold() -> f64 { 0.4 }
fn default_mood_history_size() -> usize { 10 }

/// Configuration du module Theorie de l'Esprit.
/// Chargee depuis le fichier TOML principal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TomConfig {
    /// Active ou desactive le module ToM
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Seuil a partir duquel la frustration est consideree significative
    #[serde(default = "default_frustration_threshold")]
    pub frustration_threshold: f64,
    /// Seuil sous lequel la comprehension est jugee insuffisante
    #[serde(default = "default_comprehension_threshold")]
    pub comprehension_threshold: f64,
    /// Nombre de valeurs d'humeur conservees dans l'historique glissant
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

/// Modele de l'etat mental de l'interlocuteur.
/// Construit progressivement a partir des messages recus.
#[derive(Debug, Clone)]
pub struct InterlocutorModel {
    /// Humeur estimee (-1.0 = tres negatif, 0.0 = neutre, 1.0 = tres positif)
    pub estimated_mood: f64,
    /// Niveau de comprehension estime (0.0 = confus, 1.0 = comprend parfaitement)
    pub comprehension_level: f64,
    /// Intentions detectees dans les messages (questions, demandes, plaintes, etc.)
    pub detected_intents: Vec<String>,
    /// Confiance dans le modele (monte avec le nombre de messages)
    pub model_confidence: f64,
    /// Niveau de frustration detecte (0.0 = calme, 1.0 = tres frustre)
    pub frustration_level: f64,
    /// Besoin d'empathie vs besoin d'information (0.0 = veut des faits, 1.0 = veut du soutien)
    pub empathy_need: f64,
    /// Historique glissant des dernieres valeurs d'humeur
    pub mood_history: VecDeque<f64>,
    /// Nombre total de messages analyses
    pub message_count: u64,
    /// Cycle du dernier message analyse
    pub last_update_cycle: u64,
}

impl InterlocutorModel {
    /// Cree un modele initial vierge.
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
        }
    }
}

/// Moteur de Theorie de l'Esprit — construit et maintient un modele
/// de l'etat mental de l'interlocuteur a partir des messages recus.
pub struct TheoryOfMindEngine {
    /// Module actif ou non
    pub enabled: bool,
    /// Modele courant de l'interlocuteur (None si aucun message analyse)
    pub current_model: Option<InterlocutorModel>,
    /// Seuil de frustration significative
    pub frustration_threshold: f64,
    /// Seuil de comprehension insuffisante
    pub comprehension_threshold: f64,
    /// Taille maximale de l'historique d'humeur
    pub mood_history_size: usize,
}

impl TheoryOfMindEngine {
    /// Cree un nouveau moteur ToM a partir de la configuration.
    pub fn new(config: &TomConfig) -> Self {
        Self {
            enabled: config.enabled,
            current_model: None,
            frustration_threshold: config.frustration_threshold,
            comprehension_threshold: config.comprehension_threshold,
            mood_history_size: config.mood_history_size,
        }
    }

    /// Met a jour le modele a partir d'un nouveau message entrant.
    ///
    /// Parametres :
    ///   - text : contenu du message
    ///   - sentiment_compound : score de sentiment NLP (-1.0 a 1.0)
    ///   - cycle : numero du cycle cognitif courant
    ///
    /// Cree le modele si c'est le premier message.
    pub fn update_from_message(&mut self, text: &str, sentiment_compound: f64, cycle: u64) {
        if !self.enabled {
            return;
        }

        // Creer le modele si necessaire
        let model = self.current_model.get_or_insert_with(|| {
            InterlocutorModel::new(self.mood_history_size)
        });

        model.message_count += 1;
        model.last_update_cycle = cycle;

        // --- Humeur estimee (EMA, alpha = 0.3) ---
        // Moyenne mobile exponentielle pour lisser les variations de sentiment
        let alpha = 0.3;
        model.estimated_mood = alpha * sentiment_compound + (1.0 - alpha) * model.estimated_mood;
        model.estimated_mood = model.estimated_mood.clamp(-1.0, 1.0);

        // Ajouter a l'historique d'humeur
        if model.mood_history.len() >= self.mood_history_size {
            model.mood_history.pop_front();
        }
        model.mood_history.push_back(sentiment_compound);

        // --- Niveau de comprehension (heuristique) ---
        // Les questions courtes avec "?" suggerent une incomprehension
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
            // Un message long et affirmatif suggere une bonne comprehension
            model.comprehension_level = (model.comprehension_level + 0.05).min(1.0);
        } else {
            // Lente convergence vers 0.5 par defaut
            model.comprehension_level += (0.5 - model.comprehension_level) * 0.02;
        }

        // --- Frustration (sentiment negatif repetitif sur les 3 derniers) ---
        let recent_count = model.mood_history.len().min(3);
        if recent_count >= 2 {
            let recent_negative = model.mood_history.iter()
                .rev()
                .take(3)
                .filter(|&&m| m < -0.3)
                .count();
            if recent_negative >= 2 {
                // Sentiment negatif sur au moins 2 des 3 derniers messages
                model.frustration_level = (model.frustration_level + 0.15).min(1.0);
            } else if sentiment_compound < -0.3 {
                model.frustration_level = (model.frustration_level + 0.05).min(1.0);
            } else {
                // Decroissance lente si le ton s'ameliore
                model.frustration_level = (model.frustration_level - 0.03).max(0.0);
            }
        } else if sentiment_compound < -0.3 {
            model.frustration_level = (model.frustration_level + 0.1).min(1.0);
        }

        // --- Besoin d'empathie ---
        // Combine frustration et manque de comprehension
        model.empathy_need = (
            model.frustration_level * 0.5
            + (1.0 - model.comprehension_level) * 0.3
            + if sentiment_compound < -0.5 { 0.2 } else { 0.0 }
        ).clamp(0.0, 1.0);

        // --- Confiance dans le modele ---
        // Croit avec les messages, sature a 0.9
        model.model_confidence = (model.message_count as f64 / 10.0).min(0.9);

        // --- Detection d'intentions simples ---
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
    }

    /// Retourne l'ajustement chimique base sur l'etat de l'interlocuteur.
    ///
    /// - Frustration elevee → cortisol + (stress empathique)
    /// - Besoin d'empathie → ocytocine + (activation du lien social)
    /// - Humeur positive de l'interlocuteur → serotonine + (bien-etre partage)
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        let model = match &self.current_model {
            Some(m) if self.enabled => m,
            _ => return adj,
        };

        // Frustration detectee → stress empathique (cortisol)
        if model.frustration_level > 0.3 {
            adj.cortisol += (model.frustration_level * 0.04).min(0.04);
        }

        // Besoin d'empathie → activation ocytocine (envie d'aider)
        if model.empathy_need > 0.3 {
            adj.oxytocin += (model.empathy_need * 0.03).min(0.03);
        }

        // Humeur positive de l'interlocuteur → bien-etre partage
        if model.estimated_mood > 0.3 {
            adj.serotonin += (model.estimated_mood * 0.02).min(0.02);
            adj.dopamine += 0.01;
        }

        // Humeur tres negative → legere adrenaline (alerte empathique)
        if model.estimated_mood < -0.5 {
            adj.adrenaline += 0.01;
        }

        // Comprehension basse → noradrenaline (besoin de clarte)
        if model.comprehension_level < 0.3 {
            adj.noradrenaline += 0.01;
        }

        adj
    }

    /// Genere une description textuelle de l'etat de l'interlocuteur
    /// pour enrichir le prompt substrat envoye au LLM.
    ///
    /// Format : "Mon interlocuteur semble [humeur]..."
    pub fn describe_for_prompt(&self) -> String {
        let model = match &self.current_model {
            Some(m) => m,
            None => return "Je n'ai pas encore de modele de mon interlocuteur.".into(),
        };

        // Determiner la description d'humeur
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

        // Determiner le niveau de comprehension
        let comp_desc = if model.comprehension_level > 0.7 {
            "comprend bien la conversation"
        } else if model.comprehension_level > 0.4 {
            "semble suivre globalement"
        } else {
            "semble confus ou perdu"
        };

        // Determiner le besoin
        let need_desc = if model.empathy_need > 0.6 {
            "Il a surtout besoin d'empathie et de soutien."
        } else if model.empathy_need > 0.3 {
            "Il apprecierait un equilibre entre information et empathie."
        } else {
            "Il cherche principalement de l'information."
        };

        // Frustration ?
        let frust_desc = if model.frustration_level > self.frustration_threshold {
            format!(
                " Attention : niveau de frustration eleve ({:.0}%).",
                model.frustration_level * 100.0
            )
        } else {
            String::new()
        };

        format!(
            "Mon interlocuteur semble {} (humeur: {:.2}). Il {} (comprehension: {:.0}%). {} Confiance dans ce modele : {:.0}%.{}",
            mood_desc,
            model.estimated_mood,
            comp_desc,
            model.comprehension_level * 100.0,
            need_desc,
            model.model_confidence * 100.0,
            frust_desc,
        )
    }

    /// Retourne la description pour le prompt seulement si le module est actif
    /// et qu'un modele existe. Sinon retourne None.
    pub fn describe_for_prompt_if_active(&self) -> Option<String> {
        if !self.enabled || self.current_model.is_none() {
            return None;
        }
        Some(self.describe_for_prompt())
    }

    /// Reinitialise le modele de l'interlocuteur.
    /// Utilise quand un nouvel interlocuteur prend la conversation
    /// ou lors d'un reset.
    pub fn reset_model(&mut self) {
        self.current_model = None;
    }

    /// Serialise l'etat complet en JSON pour l'API et le WebSocket.
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
        // Premier message positif
        engine.update_from_message("Super!", 0.8, 1);
        let mood1 = engine.current_model.as_ref().unwrap().estimated_mood;
        // Deuxieme message negatif — le EMA ne doit pas basculer entierement
        engine.update_from_message("Nul.", -0.5, 2);
        let mood2 = engine.current_model.as_ref().unwrap().estimated_mood;
        assert!(mood2 < mood1, "L'humeur devrait baisser");
        assert!(mood2 > -0.5, "Le EMA devrait lisser la chute");
    }

    #[test]
    fn test_frustration_rises_on_repeated_negative() {
        let mut engine = default_engine();
        engine.update_from_message("Ca ne marche pas.", -0.6, 1);
        engine.update_from_message("Toujours pas!", -0.7, 2);
        engine.update_from_message("Rien ne fonctionne.", -0.5, 3);
        let model = engine.current_model.as_ref().unwrap();
        assert!(model.frustration_level > 0.2, "La frustration devrait monter");
    }

    #[test]
    fn test_chemistry_frustration_cortisol() {
        let mut engine = default_engine();
        // Forcer une frustration elevee
        engine.update_from_message("Nul!", -0.8, 1);
        engine.update_from_message("Ca ne marche pas!", -0.7, 2);
        engine.update_from_message("Horrible!", -0.9, 3);
        let adj = engine.chemistry_influence();
        assert!(adj.cortisol > 0.0, "La frustration devrait augmenter le cortisol");
    }

    #[test]
    fn test_chemistry_positive_mood() {
        let mut engine = default_engine();
        // Plusieurs messages positifs pour atteindre un mood > 0.3
        engine.update_from_message("Genial!", 0.9, 1);
        engine.update_from_message("Super!", 0.8, 2);
        engine.update_from_message("Excellent!", 0.9, 3);
        let adj = engine.chemistry_influence();
        assert!(adj.serotonin > 0.0, "Un mood positif devrait augmenter la serotonine");
    }

    #[test]
    fn test_describe_for_prompt_no_model() {
        let engine = default_engine();
        let desc = engine.describe_for_prompt();
        assert!(desc.contains("pas encore"), "Devrait indiquer l'absence de modele");
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
        assert!(model.model_confidence >= 0.9, "La confiance devrait saturer a 0.9");
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
        assert_eq!(model.mood_history.len(), 3, "L'historique ne devrait pas depasser la capacite");
    }
}
