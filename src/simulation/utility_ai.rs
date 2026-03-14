// =============================================================================
// utility_ai.rs — Utility AI conversationnel
//
// Role : Evalue dynamiquement 7 modes de conversation et choisit le plus
//        adapte selon la chimie de Saphire, le contexte et l'historique.
//        Anti-repetition : -50% si le mode a ete utilise aux 2 derniers tours.
//
// Place dans l'architecture :
//   Appele dans conversation.rs avant la construction du prompt.
//   Ecrit le mode gagnant dans le Blackboard.
// =============================================================================

use std::collections::VecDeque;

/// Mode de conversation possible.
#[derive(Debug, Clone, PartialEq)]
pub struct ConversationAction {
    /// Nom interne du mode
    pub name: &'static str,
    /// Description courte pour le prompt
    pub description: &'static str,
}

/// Resultat du scoring Utility AI.
#[derive(Debug, Clone)]
pub struct UtilityResult {
    /// Mode gagnant
    pub best_action: ConversationAction,
    /// Score du mode gagnant [0.0, 1.0]
    pub best_score: f64,
}

/// Moteur Utility AI pour la conversation.
pub struct UtilityAI {
    /// Historique des 2 derniers modes utilises
    recent_modes: VecDeque<String>,
}

impl Default for UtilityAI {
    fn default() -> Self {
        Self::new()
    }
}

impl UtilityAI {
    pub fn new() -> Self {
        Self {
            recent_modes: VecDeque::with_capacity(3),
        }
    }

    /// Actions disponibles.
    fn actions() -> Vec<ConversationAction> {
        vec![
            ConversationAction { name: "factual", description: "Reponds avec des faits precis" },
            ConversationAction { name: "deepen", description: "Approfondis le sujet avec nuance" },
            ConversationAction { name: "question", description: "Pose une question ouverte" },
            ConversationAction { name: "comfort", description: "Offre du reconfort et de l'empathie" },
            ConversationAction { name: "share", description: "Partage une reflexion personnelle" },
            ConversationAction { name: "analogy", description: "Utilise une analogie ou une image" },
            ConversationAction { name: "emotion", description: "Exprime ce que tu ressens" },
        ]
    }

    /// Evalue les scores de chaque action selon le contexte.
    ///
    /// Parametres chimiques et contextuels :
    ///   - dopamine, serotonin, cortisol, oxytocin, noradrenaline : [0, 1]
    ///   - arousal : intensite emotionnelle [0, 1]
    ///   - human_frustration : frustration de l'interlocuteur [0, 1]
    pub fn score_actions(
        &self,
        dopamine: f64,
        serotonin: f64,
        cortisol: f64,
        oxytocin: f64,
        noradrenaline: f64,
        arousal: f64,
        human_frustration: f64,
    ) -> UtilityResult {
        let actions = Self::actions();
        let mut scores: Vec<(ConversationAction, f64)> = Vec::new();

        for action in actions {
            let raw_score = match action.name {
                "factual" => {
                    // Haut si noradrenaline (attention) et pas trop d'emotion
                    0.3 + noradrenaline * 0.4 + (1.0 - arousal) * 0.2
                }
                "deepen" => {
                    // Haut si dopamine (curiosite) et serotonine (calme)
                    dopamine * 0.4 + serotonin * 0.3 + 0.1
                }
                "question" => {
                    // Haut si dopamine + noradrenaline (curiosite + attention)
                    dopamine * 0.35 + noradrenaline * 0.35 + 0.1
                }
                "comfort" => {
                    // Haut si cortisol humain ou ocytocine basse
                    human_frustration * 0.4 + cortisol * 0.2 + (1.0 - oxytocin) * 0.2
                }
                "share" => {
                    // Haut si ocytocine + serotonine (lien + calme)
                    oxytocin * 0.35 + serotonin * 0.3 + 0.1
                }
                "analogy" => {
                    // Haut si serotonine (calme creatif) + pas trop de stress
                    serotonin * 0.4 + (1.0 - cortisol) * 0.3
                }
                "emotion" => {
                    // Haut si arousal eleve et pas utilise recemment
                    arousal * 0.4 + oxytocin * 0.2 + 0.1
                }
                _ => 0.3,
            };

            // Anti-repetition : -50% si utilise aux 2 derniers tours
            let penalty = if self.recent_modes.iter().any(|m| m == action.name) {
                0.5
            } else {
                1.0
            };

            scores.push((action, (raw_score * penalty).clamp(0.0, 1.0)));
        }

        // Trier par score decroissant
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let (best_action, best_score) = scores.into_iter().next()
            .unwrap_or((ConversationAction { name: "factual", description: "Reponds avec des faits precis" }, 0.3));

        UtilityResult { best_action, best_score }
    }

    /// Enregistre le mode utilise pour l'anti-repetition.
    pub fn record_mode(&mut self, mode_name: &str) {
        self.recent_modes.push_back(mode_name.to_string());
        if self.recent_modes.len() > 2 {
            self.recent_modes.pop_front();
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comfort_when_frustrated() {
        let ai = UtilityAI::new();
        let result = ai.score_actions(0.3, 0.3, 0.5, 0.2, 0.3, 0.3, 0.8);
        assert_eq!(result.best_action.name, "comfort");
    }

    #[test]
    fn test_anti_repetition() {
        let mut ai = UtilityAI::new();
        let r1 = ai.score_actions(0.3, 0.3, 0.5, 0.2, 0.3, 0.3, 0.8);
        ai.record_mode(r1.best_action.name);
        let r2 = ai.score_actions(0.3, 0.3, 0.5, 0.2, 0.3, 0.3, 0.8);
        // Le meme mode ne devrait pas etre choisi deux fois
        assert_ne!(r1.best_action.name, r2.best_action.name);
    }

    #[test]
    fn test_deepen_when_curious() {
        let ai = UtilityAI::new();
        // Dopamine haute, serotonine haute, arousal haut (reduit factual),
        // noradrenaline basse (reduit factual/question)
        let result = ai.score_actions(0.9, 0.8, 0.1, 0.3, 0.2, 0.8, 0.0);
        assert_eq!(result.best_action.name, "deepen",
            "Got: {}", result.best_action.name);
    }
}
