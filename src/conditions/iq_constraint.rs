// =============================================================================
// conditions/iq_constraint.rs — Limiting IQ constraint
// =============================================================================
//
// Role: If active, limits Saphire's cognitive capabilities:
//       reduced vocabulary, simplified reasoning, reduced working
//       memory, reduced abstraction capacity.
//
// Integration:
//   Modifies working memory capacity, neocortex weight,
//   and provides a supplement to the LLM system prompt.
// =============================================================================

use serde::{Deserialize, Serialize};

/// IQ constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IqConstraint {
    /// Target IQ (50-150, 100 = normal)
    pub target_iq: u8,
    /// Vocabulary factor (0.3-1.0)
    pub vocabulary_factor: f64,
    /// Reasoning depth (0.3-1.0)
    pub reasoning_depth: f64,
    /// Working memory slots (3-7)
    pub working_memory_slots: usize,
    /// Abstraction capacity (0.0-1.0)
    pub abstraction_level: f64,
    /// Neocortex weight factor in consensus (0.3-1.0)
    pub neocortex_weight_factor: f64,
}

impl IqConstraint {
    /// Computes constraints from the target IQ.
    pub fn from_iq(iq: u8) -> Self {
        let iq = iq.clamp(50, 150);
        // Normalize: 0.0 = IQ 50, 1.0 = IQ 100, >1.0 = above normal
        let normalized = (iq as f64 - 50.0) / 50.0; // 0.0 = IQ 50, 1.0 = IQ 100, 2.0 = IQ 150

        Self {
            target_iq: iq,
            vocabulary_factor: (normalized * 0.7 + 0.3).clamp(0.3, 1.0),
            reasoning_depth: (normalized * 0.8 + 0.2).clamp(0.2, 1.0),
            working_memory_slots: match iq {
                50..=69 => 3,
                70..=84 => 4,
                85..=99 => 5,
                100..=114 => 7,
                _ => 7,
            },
            abstraction_level: (normalized * 0.9 + 0.1).clamp(0.1, 1.0),
            neocortex_weight_factor: (normalized * 0.7 + 0.3).clamp(0.3, 1.0),
        }
    }

    /// Supplement for the LLM system prompt to limit vocabulary.
    pub fn prompt_supplement(&self) -> Option<String> {
        if self.target_iq >= 100 {
            return None; // No constraint above 100
        }

        let instructions = match self.target_iq {
            50..=69 => "Utilise UNIQUEMENT des mots tres simples. Phrases tres courtes (5 mots max). Pas de mots abstraits. Reponds comme un enfant de 5 ans.",
            70..=84 => "Utilise un vocabulaire simple et des phrases courtes. Evite les mots compliques. Pas de metaphores ni d'abstraction.",
            85..=99 => "Utilise un vocabulaire courant. Phrases claires et directes. Evite le jargon et les concepts trop abstraits.",
            _ => return None,
        };

        Some(instructions.to_string())
    }

    /// Impact on cognitive degradation (0.0 = none, positive value = degradation).
    pub fn cognitive_penalty(&self) -> f64 {
        if self.target_iq >= 100 {
            0.0
        } else {
            (1.0 - self.reasoning_depth) * 0.3
        }
    }

    /// Serializes for the API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "target_iq": self.target_iq,
            "vocabulary_factor": self.vocabulary_factor,
            "reasoning_depth": self.reasoning_depth,
            "working_memory_slots": self.working_memory_slots,
            "abstraction_level": self.abstraction_level,
            "neocortex_weight_factor": self.neocortex_weight_factor,
            "prompt_supplement": self.prompt_supplement(),
        })
    }
}

impl Default for IqConstraint {
    fn default() -> Self {
        Self::from_iq(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_iq_no_constraint() {
        let iq = IqConstraint::from_iq(100);
        assert_eq!(iq.working_memory_slots, 7);
        assert!(iq.vocabulary_factor >= 0.99);
        assert!(iq.prompt_supplement().is_none());
        assert_eq!(iq.cognitive_penalty(), 0.0);
    }

    #[test]
    fn test_low_iq_constraints() {
        let iq = IqConstraint::from_iq(70);
        assert_eq!(iq.working_memory_slots, 4);
        assert!(iq.vocabulary_factor < 0.7);
        assert!(iq.reasoning_depth < 0.6);
        assert!(iq.prompt_supplement().is_some());
        assert!(iq.cognitive_penalty() > 0.0);
    }

    #[test]
    fn test_very_low_iq() {
        let iq = IqConstraint::from_iq(55);
        assert_eq!(iq.working_memory_slots, 3);
        assert!(iq.abstraction_level < 0.3);
    }

    #[test]
    fn test_high_iq_no_penalty() {
        let iq = IqConstraint::from_iq(130);
        assert_eq!(iq.working_memory_slots, 7);
        assert_eq!(iq.cognitive_penalty(), 0.0);
    }

    #[test]
    fn test_clamping() {
        let iq_low = IqConstraint::from_iq(10);
        assert_eq!(iq_low.target_iq, 50);
        let iq_high = IqConstraint::from_iq(200);
        assert_eq!(iq_high.target_iq, 150);
    }
}
