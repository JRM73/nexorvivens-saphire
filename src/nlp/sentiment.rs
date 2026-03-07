// =============================================================================
// sentiment.rs — Bilingual VADER-inspired sentiment analysis (FR+EN, 400+
//                words)
//
// Purpose: Layer 2A of the NLP pipeline. Computes the emotional polarity of a
//          text (positive, negative, compound) using a sentiment lexicon
//          enriched with intensity modifiers (boosters/dampeners), negations,
//          and adversative pivots (contrastive conjunctions such as "mais",
//          "however").
//
// The approach is inspired by VADER (Valence Aware Dictionary and sEntiment
// Reasoner), adapted for bilingual French-English processing with a lexicon
// of 400+ words.
//
// Dependencies:
//   - std::collections::HashMap: efficient word-to-score lookups
//   - serde: serialization of sentiment results
//   - super::dictionaries: bilingual lexicons (sentiment words, boosters,
//     negations, adversative conjunctions)
//
// Role in the architecture:
//   Invoked by NlpPipeline::analyze() after preprocessing. The sentiment
//   result subsequently influences the Stimulus (adjusting the reward and
//   danger dimensions) and is propagated within the NlpResult.
// =============================================================================

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::dictionaries;

/// Result of the sentiment analysis for a text.
///
/// Scores are computed from the sentiment lexicon, taking into account
/// intensity modifiers, negations, and adversative pivots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentResult {
    /// Normalized compound score in [-1.0, +1.0].
    /// -1.0 = very negative sentiment, 0.0 = neutral, +1.0 = very positive.
    pub compound: f64,
    /// Positive score in [0.0, 1.0] — derived from the positive portion of
    /// the compound score.
    pub positive: f64,
    /// Negative score in [0.0, 1.0] — absolute value of the negative portion
    /// of the compound score.
    pub negative: f64,
    /// Indicates whether a contradiction was detected (an adversative pivot
    /// in the text, e.g., "mais", "however"). Useful for signaling
    /// ambivalence in the message.
    pub has_contradiction: bool,
}

/// Sentiment lexicon with intensity modifiers, negations, and adversative
/// pivots.
///
/// The lexicon is constructed once at initialization from the dictionaries
/// module, then reused for every subsequent analysis call.
pub struct SentimentLexicon {
    /// Dictionary of sentiment words: word -> polarity in [-1.0, +1.0].
    /// Positive words have polarity > 0, negative words have polarity < 0.
    words: HashMap<String, f64>,
    /// Dictionary of intensity modifiers: word -> multiplier.
    /// Boosters (> 1.0) amplify sentiment (e.g., "very" = 1.5x).
    /// Dampeners (< 1.0) attenuate sentiment (e.g., "a little" = 0.5x).
    boosters: HashMap<String, f64>,
    /// List of negation words (e.g., "ne", "pas", "not", "never").
    /// An active negation partially inverts the polarity of subsequent words.
    negations: Vec<String>,
    /// List of adversative conjunctions (e.g., "mais", "cependant", "but",
    /// "however"). An adversative pivot assigns more weight to the text
    /// following the conjunction (70% after, 30% before), reflecting the
    /// well-established linguistic principle that the post-adversative clause
    /// generally carries the speaker's dominant sentiment.
    adversatives: Vec<String>,
}

impl Default for SentimentLexicon {
    fn default() -> Self {
        Self::new()
    }
}

impl SentimentLexicon {
    /// Creates a new sentiment lexicon by loading all dictionaries.
    ///
    /// The dictionaries are defined in the `dictionaries` module and contain
    /// bilingual (FR+EN) lexicons.
    ///
    /// # Returns
    /// A fully initialized `SentimentLexicon` ready to analyze text.
    pub fn new() -> Self {
        // Load the sentiment word dictionary (word -> polarity)
        let mut words = HashMap::new();
        for (word, polarity) in dictionaries::sentiment_words() {
            words.insert(word.to_string(), polarity);
        }

        // Load the intensity modifier dictionary (word -> multiplier)
        let mut boosters = HashMap::new();
        for (word, mult) in dictionaries::boosters() {
            boosters.insert(word.to_string(), mult);
        }

        // Load the negation and adversative conjunction lists
        let negations = dictionaries::negations().iter().map(|s| s.to_string()).collect();
        let adversatives = dictionaries::adversatives().iter().map(|s| s.to_string()).collect();

        Self { words, boosters, negations, adversatives }
    }

    /// Analyzes the sentiment of a token sequence.
    ///
    /// The algorithm iterates over tokens sequentially while maintaining
    /// state:
    ///   - An intensity multiplier (`current_booster`, reset after use).
    ///   - A negation flag (active for a scope of 3 tokens).
    ///   - An adversative pivot point (separates scores into before/after
    ///     segments).
    ///
    /// Processing rules for each token:
    ///   1. If it is an adversative pivot: save pre-pivot scores, reset state.
    ///   2. If it is a negation: activate negation mode for the next 3 tokens.
    ///   3. If it is an intensity modifier: store the multiplier.
    ///   4. If it is a sentiment word: compute the adjusted score.
    ///
    /// # Parameters
    /// - `tokens`: the list of normalized, lowercased tokens.
    ///
    /// # Returns
    /// A `SentimentResult` with the compound score, positive/negative
    /// components, and the contradiction indicator.
    pub fn analyze(&self, tokens: &[String]) -> SentimentResult {
        let mut scores: Vec<f64> = Vec::new();
        // Current multiplier applied to the next sentiment word (1.0 = no modification)
        let mut current_booster = 1.0;
        // Whether a negation is currently active
        let mut negation_active = false;
        // Countdown of remaining tokens under the negation's effect (scope of 3)
        let mut negation_countdown: i32 = 0;
        // Whether an adversative pivot has been encountered in the text
        let mut pivot_found = false;
        // Scores accumulated before the adversative pivot
        let mut before_pivot: Vec<f64> = Vec::new();

        for token in tokens.iter() {
            let lower = token.to_lowercase();

            // Adversative pivot detection (e.g., "mais", "however").
            // When a pivot is found, the pre-pivot scores are saved and the
            // score accumulator is reset for the post-pivot segment.
            if self.adversatives.contains(&lower) {
                pivot_found = true;
                before_pivot = scores.clone();
                scores.clear();
                // Reset negation and intensity state at the pivot boundary
                negation_active = false;
                negation_countdown = 0;
                current_booster = 1.0;
                continue;
            }

            // Negation detection (e.g., "ne", "pas", "not", "never").
            // The negation remains active for the next 3 tokens.
            if self.negations.contains(&lower) {
                negation_active = true;
                negation_countdown = 3;
                continue;
            }

            // Intensity modifier detection (e.g., "very" = 1.5x, "a little" = 0.5x).
            // The multiplier is stored and applied to the next sentiment word.
            if let Some(&boost) = self.boosters.get(&lower) {
                current_booster = boost;
                continue;
            }

            // Sentiment word detection.
            // The score is computed as: polarity * multiplier * negation_factor.
            // Negation does not fully invert the score (factor -0.75 instead of
            // -1.0) because "not sad" is not linguistically as strong as "happy".
            if let Some(&polarity) = self.words.get(&lower) {
                let mut score = polarity * current_booster;
                if negation_active {
                    score *= -0.75; // Partial inversion, not total
                }
                scores.push(score);
                // Reset the multiplier after it has been consumed
                current_booster = 1.0;
            }

            // Decrement the negation scope countdown (maximum 3 tokens)
            if negation_countdown > 0 {
                negation_countdown -= 1;
                if negation_countdown == 0 {
                    negation_active = false;
                }
            }
        }

        // If an adversative pivot was found: apply 30% weight to pre-pivot
        // scores and 70% weight to post-pivot scores. This reflects the
        // linguistic principle that the clause following "but" carries the
        // speaker's dominant sentiment.
        let final_scores = if pivot_found && !scores.is_empty() {
            let before_avg = if before_pivot.is_empty() {
                0.0
            } else {
                before_pivot.iter().sum::<f64>() / before_pivot.len() as f64
            };
            let after_avg = scores.iter().sum::<f64>() / scores.len() as f64;
            vec![before_avg * 0.3 + after_avg * 0.7]
        } else {
            scores
        };

        // Compute the compound score: mean of all scores, clamped to [-1.0, +1.0]
        let compound = if final_scores.is_empty() {
            0.0
        } else {
            (final_scores.iter().sum::<f64>() / final_scores.len() as f64).clamp(-1.0, 1.0)
        };

        SentimentResult {
            compound,
            // The positive score is the portion of compound that is > 0
            positive: compound.max(0.0),
            // The negative score is the absolute value of the portion < 0
            negative: compound.min(0.0).abs(),
            has_contradiction: pivot_found,
        }
    }
}
