// =============================================================================
// sentiment.rs — Bilingual VADER-like sentiment analysis (FR+EN, 400+ words)
//
// Role: Layer 2A of the NLP pipeline. Computes the emotional polarity of a text
//       (positive, negative, compound) using an enriched sentiment lexicon with
//       intensifiers (boosters/attenuators), negations, and adversative pivots
//       (contrast conjunctions like "mais", "however").
//
// Approach inspired by VADER (Valence Aware Dictionary and sEntiment Reasoner),
// adapted for French-English bilingualism with a 400+ word lexicon.
//
// Dependencies:
//   - std::collections::HashMap: performant storage for word -> score lexicons
//   - serde: serialization of sentiment results
//   - super::dictionaries: provides bilingual lexicons (words, intensifiers,
//     negations, adversative conjunctions)
//
// Place in the architecture:
//   Called by NlpPipeline::analyze() after preprocessing. The sentiment result
//   then influences the Stimulus (adjustment of reward and danger dimensions)
//   and is passed along in NlpResult.
// =============================================================================

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::dictionaries;

/// Result of a text's sentiment analysis.
///
/// Scores are computed from the sentiment lexicon, accounting for
/// intensifiers, negations, and adversative pivots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentResult {
    /// Normalized compound score in [-1, +1].
    /// -1 = very negative sentiment, 0 = neutral, +1 = very positive sentiment.
    pub compound: f64,
    /// Positive score in [0, 1] — extracted from the positive part of the compound.
    pub positive: f64,
    /// Negative score in [0, 1] — absolute value of the negative part of the compound.
    pub negative: f64,
    /// Indicates whether a contradiction was detected (adversative pivot in the text,
    /// e.g. "mais", "however"). Useful for signaling message ambivalence.
    pub has_contradiction: bool,
}

/// Sentiment lexicon with intensifiers, negations, and adversative pivots.
///
/// The lexicon is built once at initialization from the dictionaries module,
/// then reused for each analysis.
pub struct SentimentLexicon {
    /// Sentiment word dictionary: word -> polarity in [-1.0, +1.0].
    /// Positive words have polarity > 0, negative words < 0.
    words: HashMap<String, f64>,
    /// Intensifier dictionary: word -> multiplier.
    /// Boosters (> 1.0) amplify sentiment (e.g., "tres" = 1.5x).
    /// Attenuators (< 1.0) reduce it (e.g., "un peu" = 0.5x).
    boosters: HashMap<String, f64>,
    /// List of negation words (e.g., "ne", "pas", "not", "never").
    /// An active negation partially inverts the polarity of following words.
    negations: Vec<String>,
    /// List of adversative conjunctions (e.g., "mais", "cependant", "but", "however").
    /// An adversative pivot gives more weight to the part of the text after the conjunction
    /// (70% after, 30% before), because in linguistics the post-adversative clause
    /// generally carries the dominant sentiment.
    adversatives: Vec<String>,
}

impl Default for SentimentLexicon {
    fn default() -> Self {
        Self::new()
    }
}

impl SentimentLexicon {
    /// Creates a new sentiment lexicon by loading the dictionaries.
    ///
    /// Dictionaries are defined in the dictionaries module and contain
    /// the bilingual FR+EN lexicons.
    ///
    /// Returns: a SentimentLexicon instance ready to analyze
    pub fn new() -> Self {
        // Load the sentiment word dictionary (word -> polarity)
        let mut words = HashMap::new();
        for (word, polarity) in dictionaries::sentiment_words() {
            words.insert(word.to_string(), polarity);
        }

        // Load the intensifier dictionary (word -> multiplier)
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
    /// The algorithm traverses tokens sequentially while maintaining state:
    ///   - An intensity multiplier (current_booster, reset after use)
    ///   - An active negation flag (with a 3-token scope countdown)
    ///   - An adversative pivot point (separates scores before/after the conjunction)
    ///
    /// Processing rules for each token:
    ///   1. If it's an adversative pivot: separate the scores and continue
    ///   2. If it's a negation: activate negation mode for 3 tokens
    ///   3. If it's an intensifier: store the multiplier
    ///   4. If it's a sentiment word: compute the adjusted score
    ///
    /// Parameters:
    ///   - tokens: the list of normalized lowercase tokens
    ///
    /// Returns: a SentimentResult with compound, positive, negative scores,
    ///          and the contradiction indicator
    pub fn analyze(&self, tokens: &[String]) -> SentimentResult {
        let mut scores: Vec<f64> = Vec::new();
        // Current multiplier applied to the next sentiment word (1.0 = no modification)
        let mut current_booster = 1.0;
        // Indicates whether a negation is currently active
        let mut negation_active = false;
        // Countdown of tokens remaining under negation effect (3-token scope)
        let mut negation_countdown: i32 = 0;
        // Indicates whether an adversative pivot was found in the text
        let mut pivot_found = false;
        // Scores accumulated before the adversative pivot
        let mut before_pivot: Vec<f64> = Vec::new();

        for token in tokens.iter() {
            let lower = token.to_lowercase();

            // Adversative pivot detection (e.g., "mais", "however")
            // When a pivot is found, scores before the pivot are saved
            // and we restart from zero for the post-pivot part.
            if self.adversatives.contains(&lower) {
                pivot_found = true;
                before_pivot = scores.clone();
                scores.clear();
                // Reset negation and intensification state
                negation_active = false;
                negation_countdown = 0;
                current_booster = 1.0;
                continue;
            }

            // Negation detection (e.g., "ne", "pas", "not", "never")
            // Negation is active for the following 3 tokens.
            if self.negations.contains(&lower) {
                negation_active = true;
                negation_countdown = 3;
                continue;
            }

            // Intensifier detection (e.g., "tres" = 1.5x, "un peu" = 0.5x)
            // The multiplier is stored and applied to the next sentiment word.
            if let Some(&boost) = self.boosters.get(&lower) {
                current_booster = boost;
                continue;
            }

            // Sentiment word detection
            // Score is computed as: polarity * multiplier * negation factor.
            // Negation does not fully invert the score (factor -0.75 instead of -1.0)
            // because "not sad" is not as strong as "happy" in linguistics.
            if let Some(&polarity) = self.words.get(&lower) {
                let mut score = polarity * current_booster;
                if negation_active {
                    score *= -0.75; // Partial inversion, not total
                }
                scores.push(score);
                // Reset the multiplier after use
                current_booster = 1.0;
            }

            // Negation scope countdown (3 tokens maximum)
            if negation_countdown > 0 {
                negation_countdown -= 1;
                if negation_countdown == 0 {
                    negation_active = false;
                }
            }
        }

        // If an adversative pivot was found: 30% before / 70% after weighting.
        // This reflects the linguistic principle that the clause after "but"
        // carries the speaker's dominant sentiment.
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

        // Compound score computation: mean of scores, clamped to [-1, +1]
        let compound = if final_scores.is_empty() {
            0.0
        } else {
            (final_scores.iter().sum::<f64>() / final_scores.len() as f64).clamp(-1.0, 1.0)
        };

        SentimentResult {
            compound,
            // Positive score is the > 0 part of the compound
            positive: compound.max(0.0),
            // Negative score is the absolute value of the < 0 part of the compound
            negative: compound.min(0.0).abs(),
            has_contradiction: pivot_found,
        }
    }
}
