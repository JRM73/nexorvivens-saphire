// =============================================================================
// preprocessor.rs — Layer 1 of the NLP pipeline: text preprocessing
//
// Purpose: Transforms raw text into a list of normalized tokens and extracts
//          structural features (punctuation counts, uppercase ratio, detected
//          language). This is the first stage of the NLP pipeline, executed
//          before sentiment analysis, intent detection, and dimension
//          extraction.
//
// Dependencies:
//   - serde: serialization / deserialization of result structures
//
// Role in the architecture:
//   Invoked by NlpPipeline::analyze() as the first layer. The produced tokens
//   feed into Layers 2A (sentiment), 2B (intent), and 2C (dimensions). The
//   structural features are consumed by the dimension extractor to adjust
//   urgency and novelty scores.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Structural features extracted from the raw text before normalization.
///
/// These metrics capture non-lexical cues in the message (punctuation,
/// writing format) that are indicative of emotional intensity and the
/// sender's communicative intent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralFeatures {
    /// Ratio of uppercase characters among all alphabetic characters.
    /// A high ratio (> 0.5) typically indicates shouting or strong emotional
    /// intensity. Range: [0.0, 1.0].
    pub uppercase_ratio: f64,
    /// Number of question marks '?' in the text.
    /// Serves as an indicator of questioning or curiosity.
    pub question_marks: usize,
    /// Number of exclamation marks '!' in the text.
    /// Serves as an indicator of intensity, urgency, or enthusiasm.
    pub exclamation_marks: usize,
    /// Whether the text contains an ellipsis '...'.
    /// Indicates hesitation, implication, or an unfinished thought.
    pub has_ellipsis: bool,
    /// Length of the message measured in token count after tokenization.
    pub token_count: usize,
    /// The detected language, determined by a heuristic based on frequent
    /// grammatical markers (French, English, or Unknown).
    pub language: Language,
}

/// Language detected via a heuristic based on frequent grammatical markers.
///
/// The detection is simplified: the counts of French and English markers
/// found in the tokens are compared to determine the dominant language of
/// the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    /// French detected (majority of French grammatical markers).
    French,
    /// English detected (majority of English grammatical markers).
    English,
    /// Indeterminate language (insufficient markers to decide).
    Unknown,
}

/// Text preprocessor — the first layer of the NLP pipeline.
///
/// This is a stateless unit struct: all processing logic resides in the
/// `process()` method.
pub struct TextPreprocessor;

impl Default for TextPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TextPreprocessor {
    /// Creates a new preprocessor instance.
    ///
    /// # Returns
    /// A `TextPreprocessor` instance.
    pub fn new() -> Self {
        Self
    }

    /// Normalizes and tokenizes the raw text input.
    ///
    /// The processing occurs in 4 sequential steps:
    ///   1. **Structural feature extraction** (performed before normalization
    ///      to preserve original casing information).
    ///   2. **Normalization**: conversion to lowercase for uniform lexical
    ///      comparison.
    ///   3. **Tokenization**: splitting on whitespace and punctuation
    ///      delimiters (comma, semicolon, colon), then stripping trailing
    ///      punctuation from each token.
    ///   4. **Language detection**: heuristic comparison of frequent French
    ///      vs. English grammatical markers.
    ///
    /// # Parameters
    /// - `text`: the raw text to preprocess.
    ///
    /// # Returns
    /// A tuple `(tokens, features)` where:
    /// - `tokens`: list of normalized, lowercased words with punctuation
    ///   removed.
    /// - `features`: the structural features extracted from the raw text.
    pub fn process(&self, text: &str) -> (Vec<String>, StructuralFeatures) {
        // Step 1: Extract structural features before normalization to preserve casing.
        // Count uppercase characters and total alphabetic characters.
        let uppercase_count = text.chars().filter(|c| c.is_uppercase()).count();
        let total_alpha = text.chars().filter(|c| c.is_alphabetic()).count().max(1);
        // Compute the ratio, using max(1) to avoid division by zero
        let uppercase_ratio = uppercase_count as f64 / total_alpha as f64;

        let question_marks = text.chars().filter(|c| *c == '?').count();
        let exclamation_marks = text.chars().filter(|c| *c == '!').count();
        let has_ellipsis = text.contains("...");

        // Step 2: Normalize to lowercase for uniform lexical comparison
        let normalized = text.to_lowercase();

        // Step 3: Simple tokenization — split on whitespace and common punctuation
        // delimiters (comma, semicolon, colon). Trailing punctuation (period,
        // exclamation, question mark, quotes, apostrophe) is then stripped from
        // each token.
        let tokens: Vec<String> = normalized
            .split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == ':')
            .filter(|s| !s.is_empty())
            .map(|s| {
                // Strip trailing punctuation to retain only the bare word
                s.trim_matches(|c: char| c == '.' || c == '!' || c == '?' || c == '"' || c == '\'')
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .collect();

        // Step 4: Detect language via heuristic (French vs. English frequent markers)
        let language = detect_language(&tokens);

        let features = StructuralFeatures {
            uppercase_ratio,
            question_marks,
            exclamation_marks,
            has_ellipsis,
            token_count: tokens.len(),
            language,
        };

        (tokens, features)
    }
}

/// Detects the language of the text using a heuristic based on frequent
/// grammatical markers.
///
/// Method: counts how many tokens match frequent grammatical markers for
/// French and English respectively. The language with the most matches wins,
/// provided there is at least one match. In case of a tie or zero matches,
/// the language is classified as `Unknown`.
///
/// # Parameters
/// - `tokens`: the list of normalized, lowercased tokens.
///
/// # Returns
/// The detected language (`French`, `English`, or `Unknown`).
fn detect_language(tokens: &[String]) -> Language {
    // Frequent French grammatical markers (pronouns, articles, prepositions, verbs)
    let fr_markers = [
        "je", "tu", "il", "elle", "nous", "vous", "les", "des", "une", "est",
        "dans", "sur", "avec", "pour", "qui", "que", "pas", "mais", "cette",
        "son", "ses", "mon", "mes", "ton", "tes", "notre", "votre", "suis",
    ];
    // Frequent English grammatical markers (pronouns, articles, prepositions, verbs)
    let en_markers = [
        "i", "you", "he", "she", "we", "they", "the", "is", "are", "was",
        "in", "on", "with", "for", "who", "that", "not", "but", "this",
        "my", "your", "his", "her", "our", "their", "am", "have", "has",
    ];

    // Count marker matches for each language
    let fr_count = tokens.iter().filter(|t| fr_markers.contains(&t.as_str())).count();
    let en_count = tokens.iter().filter(|t| en_markers.contains(&t.as_str())).count();

    // The language with the most marker matches wins, provided at least one match exists
    if fr_count > en_count && fr_count > 0 {
        Language::French
    } else if en_count > fr_count && en_count > 0 {
        Language::English
    } else {
        Language::Unknown
    }
}
