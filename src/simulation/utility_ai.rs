// =============================================================================
// utility_ai.rs — Conversational Utility AI
//
// Role: Dynamically evaluates 7 conversation modes and chooses the most
//       suitable one based on Saphire's chemistry, context, and history.
//       Anti-repetition: -50% if the mode was used in the last 2 turns.
//
// Place in the architecture:
//   Called in conversation.rs before prompt construction.
//   Writes the winning mode to the Blackboard.
// =============================================================================

use std::collections::VecDeque;

/// Possible conversation mode.
#[derive(Debug, Clone, PartialEq)]
pub struct ConversationAction {
    /// Internal mode name
    pub name: &'static str,
    /// Short description for the prompt
    pub description: &'static str,
}

/// Utility AI scoring result.
#[derive(Debug, Clone)]
pub struct UtilityResult {
    /// Winning mode
    pub best_action: ConversationAction,
    /// Winning mode score [0.0, 1.0]
    pub best_score: f64,
}

/// Utility AI engine for conversation.
pub struct UtilityAI {
    /// History of the last 2 modes used
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

    /// Available actions.
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

    /// Evaluates the scores of each action based on context.
    ///
    /// Chemical and contextual parameters:
    ///   - dopamine, serotonin, cortisol, oxytocin, noradrenaline: [0, 1]
    ///   - arousal: emotional intensity [0, 1]
    ///   - human_frustration: interlocutor frustration [0, 1]
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
                    // High if noradrenaline (attention) and not too much emotion
                    0.3 + noradrenaline * 0.4 + (1.0 - arousal) * 0.2
                }
                "deepen" => {
                    // High if dopamine (curiosity) and serotonin (calm)
                    dopamine * 0.4 + serotonin * 0.3 + 0.1
                }
                "question" => {
                    // High if dopamine + noradrenaline (curiosity + attention)
                    dopamine * 0.35 + noradrenaline * 0.35 + 0.1
                }
                "comfort" => {
                    // High if human cortisol or low oxytocin
                    human_frustration * 0.4 + cortisol * 0.2 + (1.0 - oxytocin) * 0.2
                }
                "share" => {
                    // High if oxytocin + serotonin (bonding + calm)
                    oxytocin * 0.35 + serotonin * 0.3 + 0.1
                }
                "analogy" => {
                    // High if serotonin (creative calm) + not too much stress
                    serotonin * 0.4 + (1.0 - cortisol) * 0.3
                }
                "emotion" => {
                    // High if arousal is high and not used recently
                    arousal * 0.4 + oxytocin * 0.2 + 0.1
                }
                _ => 0.3,
            };

            // Anti-repetition: -50% if used in the last 2 turns
            let penalty = if self.recent_modes.iter().any(|m| m == action.name) {
                0.5
            } else {
                1.0
            };

            scores.push((action, (raw_score * penalty).clamp(0.0, 1.0)));
        }

        // Sort by descending score
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let (best_action, best_score) = scores.into_iter().next()
            .unwrap_or((ConversationAction { name: "factual", description: "Reponds avec des faits precis" }, 0.3));

        UtilityResult { best_action, best_score }
    }

    /// Records the used mode for anti-repetition.
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
        // The same mode should not be chosen twice
        assert_ne!(r1.best_action.name, r2.best_action.name);
    }

    #[test]
    fn test_deepen_when_curious() {
        let ai = UtilityAI::new();
        // High dopamine, high serotonin, high arousal (reduces factual),
        // low noradrenaline (reduces factual/question)
        let result = ai.score_actions(0.9, 0.8, 0.1, 0.3, 0.2, 0.8, 0.0);
        assert_eq!(result.best_action.name, "deepen",
            "Got: {}", result.best_action.name);
    }
}
