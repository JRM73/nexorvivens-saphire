// =============================================================================
// preprocessor.rs — Layer 1 of the NLP pipeline: Text preprocessing
//
// Role: Transforms raw text into a list of normalized tokens and extracts
//       structural features (punctuation, uppercase, language).
//       This is the first step of the NLP pipeline, before sentiment analysis,
//       intent detection, and dimension extraction.
//
// Dependencies:
//   - serde: serialization/deserialization of result structures
//
// Place in the architecture:
//   Called by NlpPipeline::analyze() as the first layer. The produced tokens
//   then feed layers 2A (sentiment), 2B (intent), and 2C (dimensions).
//   Structural features are used by the dimension extractor to adjust
//   urgency and novelty.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Structural features extracted from raw text before normalization.
///
/// These metrics capture non-lexical cues from the message (punctuation,
/// writing format) that reveal emotional intensity and sender intent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralFeatures {
    /// Ratio of uppercase characters among alphabetic characters.
    /// A high ratio (> 0.5) often indicates shouting or strong emotional intensity.
    /// Range: [0.0, 1.0]
    pub uppercase_ratio: f64,
    /// Number of question marks '?' in the text.
    /// Indicator of questioning or curiosity.
    pub question_marks: usize,
    /// Number of exclamation marks '!' in the text.
    /// Indicator of intensity, urgency, or enthusiasm.
    pub exclamation_marks: usize,
    /// Presence of ellipsis '...' in the text.
    /// Indicator of hesitation, implication, or unfinished thought.
    pub has_ellipsis: bool,
    /// Message length in number of tokens after tokenization.
    pub token_count: usize,
    /// Language detected by heuristic (French, English, or unknown).
    pub language: Language,
}

/// Language detected by heuristic based on frequent words.
///
/// Detection is simplified: we compare the number of French and English markers
/// found in the tokens to determine the dominant language of the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    /// French detected (majority of FR markers)
    French,
    /// English detected (majority of EN markers)
    English,
    /// Undetermined language (not enough markers to decide)
    Unknown,
}

/// Text preprocessor — first layer of the NLP pipeline.
///
/// Stateless structure (unit struct): all logic is in the process() method.
pub struct TextPreprocessor;

impl Default for TextPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TextPreprocessor {
    /// Creates a new preprocessor instance.
    ///
    /// Returns: a TextPreprocessor instance
    pub fn new() -> Self {
        Self
    }

    /// Normalizes and tokenizes raw text.
    ///
    /// Processing happens in 4 steps:
    ///   1. Structural feature extraction (before normalization, to preserve case)
    ///   2. Lowercase normalization
    ///   3. Tokenization by splitting on whitespace and punctuation
    ///   4. Language detection by heuristic
    ///
    /// Parameters:
    ///   - text: the raw text to preprocess
    ///
    /// Returns: a tuple (tokens, features) where:
    ///   - tokens: list of normalized lowercase words, without punctuation
    ///   - features: the extracted structural features
    pub fn process(&self, text: &str) -> (Vec<String>, StructuralFeatures) {
        // 1. Structural feature extraction (before normalization to preserve case)
        // Count uppercase and total alphabetic characters
        let uppercase_count = text.chars().filter(|c| c.is_uppercase()).count();
        let total_alpha = text.chars().filter(|c| c.is_alphabetic()).count().max(1);
        // Ratio computed avoiding division by zero (max(1))
        let uppercase_ratio = uppercase_count as f64 / total_alpha as f64;

        let question_marks = text.chars().filter(|c| *c == '?').count();
        let exclamation_marks = text.chars().filter(|c| *c == '!').count();
        let has_ellipsis = text.contains("...");

        // 2. Normalization: lowercase to unify lexical comparison
        let normalized = text.to_lowercase();

        // 3. Simple tokenization: split on whitespace and common punctuation
        //    (comma, semicolon, colon). Trailing punctuation on each token
        //    is then removed (period, exclamation, question mark, quotes, apostrophe).
        let tokens: Vec<String> = normalized
            .split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == ':')
            .filter(|s| !s.is_empty())
            .map(|s| {
                // Strip trailing punctuation to keep only the bare word
                s.trim_matches(|c: char| c == '.' || c == '!' || c == '?' || c == '"' || c == '\'')
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .collect();

        // 4. Language detection by heuristic (comparing frequent FR vs EN words)
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

/// Detects text language by heuristic based on frequent words.
///
/// Method: count how many tokens match frequent grammatical markers for
/// French and English. The language with the most matches is chosen,
/// provided there is at least one. In case of tie or no matches,
/// the language is Unknown.
///
/// Parameters:
///   - tokens: the list of normalized lowercase tokens
///
/// Returns: the detected language (French, English, or Unknown)
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

    // Count matches for each language
    let fr_count = tokens.iter().filter(|t| fr_markers.contains(&t.as_str())).count();
    let en_count = tokens.iter().filter(|t| en_markers.contains(&t.as_str())).count();

    // The language with the most markers wins, provided it has at least one
    if fr_count > en_count && fr_count > 0 {
        Language::French
    } else if en_count > fr_count && en_count > 0 {
        Language::English
    } else {
        Language::Unknown
    }
}
