// =============================================================================
// nlp/mod.rs — Hybrid 3-layer NLP (Natural Language Processing) pipeline
//
// Purpose: Entry point for the NLP module. Orchestrates the complete text
//          analysis pipeline across 3 successive layers:
//            Layer 1  — Preprocessing (tokenization, normalization, structural
//                       feature extraction)
//            Layer 2A — Sentiment analysis (bilingual VADER-inspired lexicon,
//                       FR+EN)
//            Layer 2B — Intent detection (simplified Naive Bayes keyword
//                       classifier)
//            Layer 2C — Dimension extraction (danger, reward, urgency, social,
//                       novelty)
//
// Dependencies:
//   - serde: serialization / deserialization of analysis results
//   - crate::stimulus: the Stimulus struct (multi-dimensional emotional vector)
//   - Sub-modules: preprocessor, sentiment, intent, dimensions, dictionaries,
//                  stagnation
//
// Role in the architecture:
//   This module is invoked by Saphire's main cognitive loop to transform the
//   user's raw text input into a multi-dimensional stimulus vector that is
//   consumed by the profiling system and the tri-cerebral consensus engine.
// =============================================================================

/// Sub-module providing Layer 1 text preprocessing (tokenization, normalization,
/// structural feature extraction).
pub mod preprocessor;

/// Sub-module providing Layer 2A sentiment analysis using a bilingual
/// VADER-inspired lexicon with boosters, negations, and adversative pivots.
pub mod sentiment;

/// Sub-module providing Layer 2B intent classification through weighted keyword
/// matching across 11 communicative intent categories.
pub mod intent;

/// Sub-module providing Layer 2C dimension extraction, converting tokenized text
/// into a 5-axis Stimulus vector (danger, reward, urgency, social, novelty).
pub mod dimensions;

/// Sub-module containing all bilingual (FR+EN) lexical dictionaries used across
/// the NLP pipeline (sentiment words, boosters, negations, adversatives, and
/// per-dimension keyword lists).
pub mod dictionaries;

/// Sub-module providing thematic and semantic stagnation detection utilities,
/// used to identify obsessive repetition across recent texts.
pub mod stagnation;

use serde::{Deserialize, Serialize};
use crate::stimulus::Stimulus;
use self::preprocessor::{TextPreprocessor, StructuralFeatures, Language};
use self::sentiment::{SentimentLexicon, SentimentResult};
use self::intent::{IntentClassifier, IntentResult};
use self::dimensions::DimensionExtractor;

/// Complete result of the NLP analysis pipeline.
///
/// Aggregates all information extracted from the raw text after it has been
/// processed through the 3-layer pipeline: preprocessing, parallel analyses
/// (sentiment, intent, dimensions), and cross-layer fusion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlpResult {
    /// The multi-dimensional stimulus extracted from the text, with axes for
    /// danger, reward, urgency, social relevance, and novelty.
    pub stimulus: Stimulus,
    /// The sentiment analysis result containing positive/negative polarity
    /// scores and the composite compound score.
    pub sentiment: SentimentResult,
    /// The detected communicative intent of the message (e.g., question,
    /// command, emotion expression, greeting).
    pub intent: IntentResult,
    /// The detected language of the message (French, English, or Unknown),
    /// determined by heuristic marker counting.
    pub language: Language,
    /// The structural features of the text (uppercase ratio, punctuation
    /// counts, ellipsis presence, token count).
    pub structural_features: StructuralFeatures,
}

/// Complete NLP processing pipeline.
///
/// Contains the 4 components of the pipeline: the text preprocessor, the
/// sentiment lexicon, the intent classifier, and the dimension extractor.
/// Each component is initialized once at construction time and reused for
/// every subsequent analysis call.
pub struct NlpPipeline {
    /// Preprocessing component: normalization, tokenization, and structural
    /// feature extraction from raw text.
    preprocessor: TextPreprocessor,
    /// Bilingual (FR+EN) sentiment lexicon with support for intensity
    /// modifiers (boosters/dampeners) and negation handling.
    sentiment_lexicon: SentimentLexicon,
    /// Intent classifier based on weighted keyword matching across bilingual
    /// pattern lists for 11 communicative intent categories.
    intent_classifier: IntentClassifier,
    /// Dimension extractor that converts tokenized text into a Stimulus
    /// vector spanning 5 emotional axes.
    dimension_extractor: DimensionExtractor,
}

impl Default for NlpPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl NlpPipeline {
    /// Creates a new NLP pipeline instance.
    ///
    /// Initializes all four components (preprocessor, sentiment lexicon, intent
    /// classifier, dimension extractor) with their default bilingual
    /// dictionaries.
    ///
    /// # Returns
    /// A fully initialized `NlpPipeline` ready to analyze text.
    pub fn new() -> Self {
        Self {
            preprocessor: TextPreprocessor::new(),
            sentiment_lexicon: SentimentLexicon::new(),
            intent_classifier: IntentClassifier::new(),
            dimension_extractor: DimensionExtractor::new(),
        }
    }

    /// Performs a complete analysis of raw text input.
    ///
    /// Executes the pipeline across 3 layers:
    ///   1. **Preprocessing**: tokenization + structural feature extraction.
    ///   2. **Parallel analyses**: sentiment (Layer 2A), intent (Layer 2B),
    ///      dimensions (Layer 2C).
    ///   3. **Cross-layer fusion**: sentiment adjusts the stimulus — strongly
    ///      positive sentiment reinforces the reward dimension, while strongly
    ///      negative sentiment reinforces the danger dimension.
    ///
    /// # Parameters
    /// - `text`: the raw text to analyze (typically a user message).
    ///
    /// # Returns
    /// An `NlpResult` containing the stimulus vector, sentiment scores,
    /// detected intent, language, and structural features.
    pub fn analyze(&self, text: &str) -> NlpResult {
        // Layer 1: Preprocessing — tokenization and structural feature extraction
        let (tokens, features) = self.preprocessor.process(text);

        // Layer 2A: Sentiment — positive/negative polarity analysis
        let sentiment = self.sentiment_lexicon.analyze(&tokens);

        // Layer 2B: Intent — communicative intent classification
        let intent = self.intent_classifier.classify(&tokens, text);

        // Layer 2C: Dimension extraction — convert tokens into a Stimulus vector
        let mut stimulus = self.dimension_extractor.extract(&tokens, text, &features);

        // Cross-layer fusion: adjust the stimulus based on sentiment scores.
        // A strongly positive sentiment (compound > 0.3) reinforces the "reward"
        // dimension by adding 30% of the positive sentiment component.
        if sentiment.compound > 0.3 {
            stimulus.reward = (stimulus.reward + sentiment.positive * 0.3).min(1.0);
        }
        // A strongly negative sentiment (compound < -0.3) reinforces the "danger"
        // dimension by adding 20% of the negative sentiment component.
        if sentiment.compound < -0.3 {
            stimulus.danger = (stimulus.danger + sentiment.negative * 0.2).min(1.0);
        }

        NlpResult {
            stimulus,
            sentiment,
            intent,
            language: features.language,
            structural_features: features,
        }
    }
}
