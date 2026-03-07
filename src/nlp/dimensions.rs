// =============================================================================
// dimensions.rs — Extraction of the 5 Stimulus dimensions from text
//
// Purpose: Layer 2C of the NLP pipeline. Converts normalized tokens into a
//          5-dimensional Stimulus vector representing emotional axes:
//            - danger:    perceived threat level (aggression, violence, risk)
//            - reward:    gratification potential (gift, success, gain)
//            - urgency:   perceived time pressure (hurry, now, SOS)
//            - social:    relational charge (family, friends, social pronouns)
//            - novelty:   degree of unexpectedness or discovery (mystery,
//                         innovation)
//
//          Each dimension is scored via lexical matching with negation
//          handling, then adjusted by structural features (punctuation,
//          uppercase ratio).
//
// Dependencies:
//   - std::collections::HashMap: storage for dimension-specific lexicons
//   - crate::stimulus: the Stimulus struct and StimulusSource enum
//   - super::dictionaries: per-dimension keyword lists (danger, reward, etc.)
//   - super::preprocessor: StructuralFeatures for structural adjustments
//
// Role in the architecture:
//   Invoked by NlpPipeline::analyze() after preprocessing. The resulting
//   Stimulus is the central vector of Saphire's cognitive system: it feeds
//   the tri-cerebral consensus (reptilian, limbic, neocortex), the emotional
//   neurochemistry engine, and the human profiling system.
// =============================================================================

use std::collections::HashMap;
use crate::stimulus::Stimulus;
use super::dictionaries;
use super::preprocessor::StructuralFeatures;

/// Dimension extractor: converts tokenized text into a 5-axis Stimulus vector.
///
/// Contains 5 lexicons (one per dimension) and a negation word list. Each
/// lexicon maps words to an intensity score in the range [0.0, 1.0].
pub struct DimensionExtractor {
    /// Lexicon of danger indicator words with their intensity scores
    /// (e.g., "kill" = 0.95, "risk" = 0.6).
    danger_words: HashMap<String, f64>,
    /// Lexicon of reward indicator words with their intensity scores
    /// (e.g., "victory" = 0.8, "gift" = 0.7).
    reward_words: HashMap<String, f64>,
    /// Lexicon of urgency indicator words with their intensity scores
    /// (e.g., "urgent" = 0.9, "hurry" = 0.8).
    urgency_words: HashMap<String, f64>,
    /// Lexicon of social relevance indicator words with their intensity scores
    /// (e.g., "family" = 0.8, "friend" = 0.7).
    social_words: HashMap<String, f64>,
    /// Lexicon of novelty indicator words with their intensity scores
    /// (e.g., "discovery" = 0.8, "unprecedented" = 0.8).
    novelty_words: HashMap<String, f64>,
    /// List of negation words used to partially invert the effect of matched
    /// keywords within a 3-token scope.
    negations: Vec<String>,
}

impl Default for DimensionExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl DimensionExtractor {
    /// Creates a new dimension extractor by loading all lexicons from the
    /// dictionaries module.
    ///
    /// Uses a helper closure `to_map` to convert the tuple vectors from the
    /// dictionaries module into `HashMap`s for efficient O(1) lookups.
    ///
    /// # Returns
    /// A fully initialized `DimensionExtractor` ready to extract dimensions.
    pub fn new() -> Self {
        // Helper closure: converts a Vec<(&str, f64)> into a HashMap<String, f64>
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

    /// Extracts the 5 dimensions from tokenized text and constructs a Stimulus.
    ///
    /// The extraction process:
    ///   1. Scores each dimension via lexical matching (with negation handling).
    ///   2. Adjusts urgency based on exclamation mark count and uppercase ratio.
    ///   3. Adjusts novelty based on question mark presence (curiosity signal).
    ///   4. Computes familiarity as the inverse of novelty.
    ///
    /// # Parameters
    /// - `tokens`: the list of normalized, lowercased tokens.
    /// - `raw_text`: the original raw text (preserved for the Stimulus `text`
    ///   field).
    /// - `features`: the structural features of the text (punctuation counts,
    ///   uppercase ratio).
    ///
    /// # Returns
    /// A `Stimulus` with all 5 dimensions scored and clamped to [0.0, 1.0].
    pub fn extract(
        &self,
        tokens: &[String],
        raw_text: &str,
        features: &StructuralFeatures,
    ) -> Stimulus {
        // Score each dimension via lexical matching against the corresponding lexicon
        let danger = self.score_dimension(tokens, &self.danger_words);
        let reward = self.score_dimension(tokens, &self.reward_words);
        let mut urgency = self.score_dimension(tokens, &self.urgency_words);
        let social = self.score_dimension(tokens, &self.social_words);
        let novelty = self.score_dimension(tokens, &self.novelty_words);

        // Adjust urgency based on punctuation:
        // More than 2 exclamation marks indicates high emotional intensity (+0.2)
        if features.exclamation_marks > 2 {
            urgency = (urgency + 0.2).min(1.0);
        }
        // An uppercase ratio above 50% indicates shouting or emphasis (+0.15)
        if features.uppercase_ratio > 0.5 {
            urgency = (urgency + 0.15).min(1.0);
        }

        // Adjust novelty based on question marks:
        // Questions signal curiosity, which slightly increases the novelty score (+0.1)
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
            // Familiarity is the inverse of novelty: the more novel a stimulus is,
            // the less familiar it feels.
            familiarity: (1.0 - novelty).clamp(0.0, 1.0),
            source: crate::stimulus::StimulusSource::Human,
        }
    }

    /// Computes the score of a single dimension with negation handling.
    ///
    /// The algorithm iterates over tokens sequentially:
    ///   - If a negation word is detected, negation mode is activated for the
    ///     next 3 tokens.
    ///   - If a lexicon word is found, its score is reduced to 20% of its
    ///     original value when under negation (rather than fully inverted,
    ///     because "no danger" does not imply safety).
    ///   - The final score is the mean of matched scores, amplified by a
    ///     logarithmic factor (sqrt(match_count) * 0.3) to reward texts with
    ///     many markers for a given dimension.
    ///
    /// # Parameters
    /// - `tokens`: the list of normalized, lowercased tokens.
    /// - `lexicon`: the keyword dictionary for the dimension being scored.
    ///
    /// # Returns
    /// The dimension score, clamped to [0.0, 1.0].
    fn score_dimension(&self, tokens: &[String], lexicon: &HashMap<String, f64>) -> f64 {
        let mut total_score = 0.0;
        let mut match_count = 0;
        let mut negation_active = false;
        let mut negation_countdown: i32 = 0;

        for token in tokens {
            let lower = token.to_lowercase();

            // Check whether the current token is a negation word
            if self.negations.contains(&lower) {
                negation_active = true;
                negation_countdown = 3;
                continue;
            }

            // Check whether the current token matches a word in the dimension lexicon
            if let Some(&score) = lexicon.get(&lower) {
                // Under active negation, the score is reduced to 20% of its value
                // (rather than zeroed or inverted, because "no danger" does not
                // imply the presence of safety)
                let effective = if negation_active { score * 0.2 } else { score };
                total_score += effective;
                match_count += 1;
            }

            // Decrement the negation scope countdown (maximum 3 tokens)
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
            // Normalization: the raw mean is amplified by a logarithmic boost
            // factor (sqrt(match_count) * 0.3) to reward texts rich in dimension
            // markers. This prevents a single "danger" word from producing the
            // same score as a text containing 5 distinct danger words. The result
            // is clamped to [0.0, 1.0].
            let raw = total_score / match_count as f64;
            let boost = (match_count as f64).sqrt() * 0.3;
            (raw * (1.0 + boost)).clamp(0.0, 1.0)
        }
    }
}
