// =============================================================================
// dimensions.rs — Extraction of the 5 Stimulus dimensions from text
//
// Role: Layer 2C of the NLP pipeline. Converts normalized tokens into a
//       5-dimensional emotional Stimulus vector:
//         - danger: perceived threat level (aggression, violence, risk)
//         - reward: gratification potential (gift, success, gain)
//         - urgency: perceived time pressure (hurry, now, SOS)
//         - social: relational charge (family, friends, social pronouns)
//         - novelty: degree of novelty or unknown (discovery, mystery)
//
//       Each dimension is scored by lexical matching with negation handling,
//       then adjusted by structural features (punctuation, uppercase).
//
// Dependencies:
//   - std::collections::HashMap: storage of dimension -> score lexicons
//   - crate::stimulus: Stimulus structure and StimulusSource
//   - super::dictionaries: provides per-dimension lexicons (danger, reward, etc.)
//   - super::preprocessor: StructuralFeatures for structural adjustments
//
// Place in the architecture:
//   Called by NlpPipeline::analyze() after preprocessing. The produced Stimulus
//   is the central vector of Saphire's cognitive system: it feeds the tri-brain
//   consensus (reptilian, limbic, neocortex), the emotional chemistry, and the
//   profiling system.
// =============================================================================

use std::collections::HashMap;
use crate::stimulus::Stimulus;
use super::dictionaries;
use super::preprocessor::StructuralFeatures;

/// Dimension extractor: converts tokenized text into a Stimulus vector.
///
/// Contains 5 lexicons (one per dimension) and a negation list. Each lexicon
/// maps words to an intensity score in [0.0, 1.0].
pub struct DimensionExtractor {
    /// Lexicon of danger indicator words with their intensity (e.g., "tuer" = 0.95, "risque" = 0.6)
    danger_words: HashMap<String, f64>,
    /// Lexicon of reward indicator words (e.g., "victoire" = 0.8, "cadeau" = 0.7)
    reward_words: HashMap<String, f64>,
    /// Lexicon of urgency indicator words (e.g., "urgent" = 0.9, "vite" = 0.8)
    urgency_words: HashMap<String, f64>,
    /// Lexicon of social indicator words (e.g., "famille" = 0.8, "ami" = 0.7)
    social_words: HashMap<String, f64>,
    /// Lexicon of novelty indicator words (e.g., "decouverte" = 0.8, "inedit" = 0.8)
    novelty_words: HashMap<String, f64>,
    /// List of negation words to partially invert keyword effects
    negations: Vec<String>,
}

impl Default for DimensionExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl DimensionExtractor {
    /// Creates a new dimension extractor by loading the lexicons.
    ///
    /// Uses a utility function `to_map` to convert the tuple vectors
    /// from the dictionaries module into performant HashMaps.
    ///
    /// Returns: a DimensionExtractor instance ready to extract
    pub fn new() -> Self {
        // Utility function: converts Vec<(&str, f64)> to HashMap<String, f64>
        let to_map = |words: Vec<(&str, f64)>| -> HashMap<String, f64> {
            words.into_iter().map(|(w, s)| (w.to_string(), s)).collect()
        };

        Self {
            danger_words: to_map(dictionaries::danger_words()),
            reward_words: to_map(dictionaries::reward_words()),
            urgency_words: to_map(dictionaries::urgency_words()),
            social_words: to_map(dictionaries::social_words()),
            novelty_words: to_map(dictionaries::novelty_words()),
            negations: dictionaries::negations().iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Extracts the 5 dimensions from tokenized text and builds a Stimulus.
    ///
    /// The process:
    ///   1. Score each dimension by lexical matching (with negation handling)
    ///   2. Adjust urgency based on punctuation (exclamation) and uppercase
    ///   3. Adjust novelty based on question marks
    ///   4. Compute familiarity as the inverse of novelty
    ///
    /// Parameters:
    ///   - tokens: the list of normalized lowercase tokens
    ///   - raw_text: the original raw text (kept for the Stimulus text field)
    ///   - features: the structural features of the text (punctuation, uppercase)
    ///
    /// Returns: a Stimulus with all 5 dimensions scored and clamped to [0.0, 1.0]
    pub fn extract(
        &self,
        tokens: &[String],
        raw_text: &str,
        features: &StructuralFeatures,
    ) -> Stimulus {
        // Score each dimension by lexical matching
        let danger = self.score_dimension(tokens, &self.danger_words);
        let reward = self.score_dimension(tokens, &self.reward_words);
        let mut urgency = self.score_dimension(tokens, &self.urgency_words);
        let social = self.score_dimension(tokens, &self.social_words);
        let novelty = self.score_dimension(tokens, &self.novelty_words);

        // Adjust urgency by punctuation:
        // More than 2 exclamation marks indicates strong intensity (+0.2)
        if features.exclamation_marks > 2 {
            urgency = (urgency + 0.2).min(1.0);
        }
        // An uppercase ratio above 50% indicates shouting or emphasis (+0.15)
        if features.uppercase_ratio > 0.5 {
            urgency = (urgency + 0.15).min(1.0);
        }

        // Adjust novelty by question marks:
        // Questions signal curiosity, which slightly increases novelty (+0.1)
        let novelty = if features.question_marks > 0 {
            (novelty + 0.1).min(1.0)
        } else {
            novelty
        };

        Stimulus {
            text: raw_text.to_string(),
            danger: danger.clamp(0.0, 1.0),
            reward: reward.clamp(0.0, 1.0),
            urgency: urgency.clamp(0.0, 1.0),
            social: social.clamp(0.0, 1.0),
            novelty: novelty.clamp(0.0, 1.0),
            // Familiarity is the inverse of novelty: the newer something is, the less familiar
            familiarity: (1.0 - novelty).clamp(0.0, 1.0),
            source: crate::stimulus::StimulusSource::Human,
        }
    }

    /// Computes a dimension's score accounting for negations.
    ///
    /// The algorithm traverses tokens sequentially:
    ///   - If a negation word is detected, it activates negation mode for 3 tokens.
    ///   - If a lexicon word is found, its score is reduced to 20% under negation.
    ///   - The final score is the average of found scores, amplified by a logarithmic
    ///     factor (square root of match count * 0.3) to favor texts with many
    ///     dimension markers.
    ///
    /// Parameters:
    ///   - tokens: the list of normalized lowercase tokens
    ///   - lexicon: the keyword dictionary for the dimension to score
    ///
    /// Returns: the dimension score in [0.0, 1.0]
    fn score_dimension(&self, tokens: &[String], lexicon: &HashMap<String, f64>) -> f64 {
        let mut total_score = 0.0;
        let mut match_count = 0;
        let mut negation_active = false;
        let mut negation_countdown: i32 = 0;

        for token in tokens {
            let lower = token.to_lowercase();

            // Check if the token is a negation word
            if self.negations.contains(&lower) {
                negation_active = true;
                negation_countdown = 3;
                continue;
            }

            // Check if the token is in the dimension lexicon
            if let Some(&score) = lexicon.get(&lower) {
                // If negation is active, the score is reduced to 20% of its value
                // (instead of 0 or inversion, because "no danger" doesn't imply safety)
                let effective = if negation_active { score * 0.2 } else { score };
                total_score += effective;
                match_count += 1;
            }

            // Negation scope countdown (3 tokens maximum)
            if negation_countdown > 0 {
                negation_countdown -= 1;
                if negation_countdown == 0 {
                    negation_active = false;
                }
            }
        }

        if match_count == 0 {
            0.0
        } else {
            // Normalization: the raw average is amplified by a logarithmic factor
            // (sqrt(match_count) * 0.3) to reward texts rich in markers.
            // This boost prevents a single "danger" word from scoring the same
            // as a text with 5 different danger words. Result clamped to [0.0, 1.0].
            let raw = total_score / match_count as f64;
            let boost = (match_count as f64).sqrt() * 0.3;
            (raw * (1.0 + boost)).clamp(0.0, 1.0)
        }
    }
}
