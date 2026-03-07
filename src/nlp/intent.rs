// =============================================================================
// intent.rs — Intent detection via simplified Naive Bayes classification
//
// Purpose: Layer 2B of the NLP pipeline. Classifies the communicative intent
//          of a message (question, command, emotion expression, greeting,
//          threat, etc.) using a simplified approach inspired by Naive Bayes:
//          tokens and raw text are compared against keyword lists associated
//          with each intent, weighted by a base confidence weight.
//
// Dependencies:
//   - serde: serialization / deserialization of results and enumerations
//
// Role in the architecture:
//   Invoked by NlpPipeline::analyze() after preprocessing. The detected intent
//   is used by the human profiling system (human_profiler) to estimate the
//   interlocutor's OCEAN personality profile and to guide communication
//   adaptation strategies.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Enumeration of detectable communicative intents.
///
/// Each variant represents a type of intention that the message sender may
/// have. Detection is based on bilingual (FR+EN) indicator keywords.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    /// Question: a request for information (pourquoi, comment, who, what, etc.)
    Question,
    /// Polite request: a request formulated with politeness markers
    /// (s'il te plait, please, could you)
    Request,
    /// Emotion expression: the speaker is conveying an emotional state
    /// (je suis triste, I feel sad, etc.)
    EmotionExpression,
    /// Information sharing: the speaker is narrating or informing
    /// (hier, j'ai vu, yesterday, I saw)
    InfoSharing,
    /// Direct command: an imperative instruction (fais, arrete, do, make, stop)
    Command,
    /// Threat: the speaker is formulating a threat
    /// (je vais te, tu vas voir, gare a, I'll hurt)
    Threat,
    /// Compliment: the speaker is expressing praise
    /// (bravo, tu es super, well done, you're amazing)
    Compliment,
    /// Help seeking: the speaker is asking for assistance
    /// (aide, help, besoin, au secours, SOS)
    HelpSeeking,
    /// Greeting: a salutation formula (bonjour, salut, hello, hi, hey)
    Greeting,
    /// Farewell: a departure formula (au revoir, bye, bonne nuit, farewell)
    Farewell,
    /// Philosophical reflection: an existential or metacognitive question
    /// (conscience, existence, meaning, free will)
    Philosophical,
    /// Unknown: no pattern matched with sufficient confidence
    Unknown,
}

impl Intent {
    /// Returns a textual representation of the intent as a static string.
    ///
    /// The labels are kept in French for consistency with the system's
    /// internal reasoning language.
    ///
    /// # Returns
    /// A static string slice describing the intent category.
    pub fn as_str(&self) -> &str {
        match self {
            Intent::Question => "question",
            Intent::Request => "demande",
            Intent::EmotionExpression => "expression_émotion",
            Intent::InfoSharing => "partage_info",
            Intent::Command => "ordre",
            Intent::Threat => "menace",
            Intent::Compliment => "compliment",
            Intent::HelpSeeking => "recherche_aide",
            Intent::Greeting => "salutation",
            Intent::Farewell => "adieu",
            Intent::Philosophical => "philosophique",
            Intent::Unknown => "inconnu",
        }
    }
}

/// Result of the intent classification process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResult {
    /// The primary detected intent (the most probable classification).
    pub primary_intent: Intent,
    /// The confidence score of the classification in [0.0, 1.0].
    /// Higher values indicate greater certainty in the classification.
    pub confidence: f64,
}

/// Intent classifier based on weighted keyword matching.
///
/// Uses a simplified approach inspired by Naive Bayes: for each intent, a list
/// of indicator keywords and a base weight are defined. The score for a given
/// intent is computed as the ratio of matched keywords multiplied by the base
/// weight. The intent with the highest score wins.
pub struct IntentClassifier {
    /// List of intent patterns: (intent, indicator keywords, base weight).
    /// The base weight modulates the maximum achievable confidence for each
    /// intent. For example, greetings have a weight of 0.9 because their
    /// indicator words are highly specific and unambiguous.
    patterns: Vec<(Intent, Vec<&'static str>, f64)>,
}

impl Default for IntentClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl IntentClassifier {
    /// Creates a new intent classifier with the default bilingual patterns.
    ///
    /// The patterns cover 11 intents with bilingual (FR+EN) keywords. Each
    /// pattern has a base weight reflecting the reliability of detection for
    /// that particular intent category.
    ///
    /// # Returns
    /// A fully initialized `IntentClassifier` ready to classify messages.
    pub fn new() -> Self {
        let patterns = vec![
            // (Intent, bilingual indicator keywords, base weight)
            // Greetings and farewells have a high weight (0.9) because their
            // formulaic expressions are highly specific and rarely ambiguous.
            (Intent::Greeting, vec![
                "bonjour", "salut", "hello", "hi", "hey", "coucou", "bonsoir",
                "yo", "wesh", "good morning", "good evening",
            ], 0.9),
            (Intent::Farewell, vec![
                "au revoir", "bye", "adieu", "bonne nuit", "goodbye", "ciao",
                "à bientôt", "à plus", "tchao", "farewell",
            ], 0.9),
            // Questions have a moderate weight (0.7) because interrogative words
            // can appear in non-question contexts as well.
            (Intent::Question, vec![
                "pourquoi", "comment", "quand", "où", "qui", "quoi", "quel",
                "quelle", "combien", "est-ce", "why", "how", "when", "where",
                "who", "what", "which", "do you", "can you", "is it",
            ], 0.7),
            // Polite requests have a weight of 0.8 due to their distinctive
            // politeness markers making them fairly easy to distinguish.
            (Intent::Request, vec![
                "peux-tu", "pourrais-tu", "voudrais-tu", "s'il te plaît",
                "svp", "please", "could you", "would you", "can you",
                "j'aimerais", "je voudrais", "i'd like",
            ], 0.8),
            // Commands have a weight of 0.7 because imperative verb forms can
            // also appear in narrative contexts.
            (Intent::Command, vec![
                "fais", "fait", "do", "make", "exécute", "lance", "arrête",
                "stop", "commence", "démarre", "éteins", "allume", "supprime",
                "delete", "run", "start", "shut down",
            ], 0.7),
            // Emotion expressions have a weight of 0.7 because they rely on
            // formulaic patterns such as "je suis" + emotional adjective.
            (Intent::EmotionExpression, vec![
                "je suis triste", "je suis content", "j'ai peur", "je suis heureux",
                "je me sens", "i feel", "i am sad", "i am happy", "je suis en colère",
                "ça me rend", "je ressens", "j'éprouve",
                "triste", "content", "heureux", "peur", "angoisse",
            ], 0.7),
            // Threats have a high weight (0.85) because accurate detection is
            // critical for safety-related reasons.
            (Intent::Threat, vec![
                "je vais te", "je vais vous", "tu vas voir", "gare à",
                "attention", "menace", "i'll hurt", "i will destroy",
                "tu vas le regretter", "prends garde",
            ], 0.85),
            // Compliments have a weight of 0.8 due to their distinctive
            // praising formulations.
            (Intent::Compliment, vec![
                "tu es géniale", "tu es super", "bravo", "bien joué",
                "impressionnant", "you're amazing", "you're great", "well done",
                "good job", "chapeau", "tu es belle", "magnifique",
                "tu es intelligente", "you're smart",
            ], 0.8),
            // Help seeking has a weight of 0.75.
            (Intent::HelpSeeking, vec![
                "aide", "aider", "help", "besoin", "need", "problème",
                "comment faire", "je ne sais pas", "i don't know",
                "au secours", "sos", "je suis perdu", "lost",
            ], 0.75),
            // Information sharing has a low weight (0.6) because temporal and
            // narrative markers are common across many different contexts.
            (Intent::InfoSharing, vec![
                "hier", "aujourd'hui", "j'ai vu", "j'ai fait", "il y a",
                "je suis allé", "yesterday", "today", "i saw", "i did",
                "figure-toi", "devine", "tu sais quoi", "did you know",
            ], 0.6),
            // Philosophical reflection has a weight of 0.75.
            (Intent::Philosophical, vec![
                "penses-tu", "crois-tu", "sens-tu", "conscience",
                "existence", "être", "réalité", "libre arbitre",
                "do you think", "do you feel", "consciousness",
                "reality", "meaning", "purpose", "sens de la vie",
            ], 0.75),
        ];

        Self { patterns }
    }

    /// Classifies the communicative intent of a tokenized text.
    ///
    /// The algorithm proceeds in two stages:
    ///   1. **Punctuation check**: a trailing '?' defaults to the Question
    ///      intent with a baseline confidence of 0.6.
    ///   2. **Pattern matching**: iterates over all intent patterns, counting
    ///      keyword matches in both the full lowercase text (for multi-word
    ///      expressions like "je suis triste" or "au secours") and individual
    ///      tokens. The score is computed as:
    ///      `(match_count / total_keywords) * base_weight`.
    ///      The intent with the highest score wins.
    ///
    /// # Parameters
    /// - `tokens`: the list of normalized, lowercased tokens.
    /// - `raw_text`: the original raw text (used for multi-word expression
    ///   matching).
    ///
    /// # Returns
    /// An `IntentResult` with the primary intent and the confidence score.
    pub fn classify(&self, tokens: &[String], raw_text: &str) -> IntentResult {
        let lower_text = raw_text.to_lowercase();
        let mut best_intent = Intent::Unknown;
        let mut best_score = 0.0;

        // Stage 1: Check punctuation patterns first.
        // A text ending with '?' is likely a question (baseline score 0.6).
        if raw_text.trim().ends_with('?') {
            best_intent = Intent::Question;
            best_score = 0.6;
        }

        // Stage 2: Iterate over each intent pattern
        for (intent, keywords, base_weight) in &self.patterns {
            let mut match_count = 0;
            let total = keywords.len().max(1);

            for kw in keywords {
                // Check against the full text (for multi-word expressions
                // such as "je suis triste" or "au secours")
                if lower_text.contains(kw) {
                    match_count += 1;
                }
                // Also check against individual tokens (for single words)
                for token in tokens {
                    if token == kw {
                        match_count += 1;
                        break;
                    }
                }
            }

            // Compute the score: ratio of matches * base weight
            if match_count > 0 {
                let score = (match_count as f64 / total as f64) * base_weight;
                if score > best_score {
                    best_score = score;
                    best_intent = intent.clone();
                }
            }
        }

        IntentResult {
            primary_intent: best_intent,
            confidence: best_score.clamp(0.0, 1.0),
        }
    }
}
