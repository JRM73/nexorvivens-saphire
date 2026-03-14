// =============================================================================
// intent.rs — Intent detection via simplified Naive Bayes
//
// Role: Layer 2B of the NLP pipeline. Classifies the communicative intent of
//       the message (question, command, emotion expression, greeting, threat,
//       etc.) using a simplified approach inspired by Naive Bayes: tokens and
//       raw text are compared to keyword lists associated with each intent,
//       weighted by a base weight.
//
// Dependencies:
//   - serde: serialization/deserialization of results and enumerations
//
// Place in the architecture:
//   Called by NlpPipeline::analyze() after preprocessing. The detected intent
//   is used by the human profiling system (human_profiler) to estimate the
//   interlocutor's OCEAN profile and to guide communication adaptation
//   strategies.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Enumeration of detectable communicative intents.
///
/// Each variant represents a type of intent the message sender may have.
/// Detection is based on bilingual (FR+EN) indicator keywords.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    /// Question: information request (pourquoi, comment, who, what, etc.)
    Question,
    /// Polite request: request formulated with politeness markers (s'il te plait, please)
    Request,
    /// Emotion expression: the speaker expresses an emotional state (je suis triste, I feel)
    EmotionExpression,
    /// Information sharing: the speaker recounts or informs (hier, j'ai vu, yesterday)
    InfoSharing,
    /// Direct command: imperative command (fais, arrete, do, make)
    Command,
    /// Threat: the speaker formulates a threat (je vais te, tu vas voir, gare a)
    Threat,
    /// Compliment: the speaker gives praise (bravo, tu es super, well done)
    Compliment,
    /// Help seeking: the speaker asks for help (aide, help, besoin, au secours)
    HelpSeeking,
    /// Greeting: greeting formula (bonjour, salut, hello, hi)
    Greeting,
    /// Farewell: departure formula (au revoir, bye, bonne nuit, farewell)
    Farewell,
    /// Philosophical reflection: existential or metacognitive question (consciousness, existence)
    Philosophical,
    /// Unidentified intent: no pattern matches sufficiently
    Unknown,
}

impl Intent {
    /// Returns the textual representation of the intent.
    ///
    /// Returns: a static string describing the intent
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

/// Intent classification result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResult {
    /// The detected primary intent (most probable)
    pub primary_intent: Intent,
    /// The classification confidence score in [0.0, 1.0].
    /// Higher score means more certain classification.
    pub confidence: f64,
}

/// Intent classifier based on weighted keyword matching.
///
/// Simplified approach inspired by Naive Bayes: for each intent, a list of
/// indicator keywords and a base weight are defined. An intent's score is
/// computed as the ratio of found keywords multiplied by the base weight.
/// The intent with the best score wins.
pub struct IntentClassifier {
    /// List of patterns: (intent, indicator keywords, base weight).
    /// The base weight modulates the maximum achievable confidence for each intent.
    /// For example, greetings have a weight of 0.9 because they are very reliable.
    patterns: Vec<(Intent, Vec<&'static str>, f64)>,
}

impl Default for IntentClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl IntentClassifier {
    /// Creates a new intent classifier with default patterns.
    ///
    /// Patterns cover 11 intents with bilingual FR+EN keywords.
    /// Each pattern has a base weight reflecting detection reliability
    /// for that intent.
    ///
    /// Returns: an IntentClassifier instance ready to classify
    pub fn new() -> Self {
        let patterns = vec![
            // (Intent, bilingual indicator keywords, base weight)
            // Greetings and farewells have a high weight (0.9) because
            // formulas are very specific and unambiguous.
            (Intent::Greeting, vec![
                "bonjour", "salut", "hello", "hi", "hey", "coucou", "bonsoir",
                "yo", "wesh", "good morning", "good evening",
            ], 0.9),
            (Intent::Farewell, vec![
                "au revoir", "bye", "adieu", "bonne nuit", "goodbye", "ciao",
                "à bientôt", "à plus", "tchao", "farewell",
            ], 0.9),
            // Questions have a medium weight (0.7) because interrogative words
            // can appear in other contexts.
            (Intent::Question, vec![
                "pourquoi", "comment", "quand", "où", "qui", "quoi", "quel",
                "quelle", "combien", "est-ce", "why", "how", "when", "where",
                "who", "what", "which", "do you", "can you", "is it",
            ], 0.7),
            // Polite requests have a weight of 0.8 thanks to politeness formulas
            // that make them fairly distinctive.
            (Intent::Request, vec![
                "peux-tu", "pourrais-tu", "voudrais-tu", "s'il te plaît",
                "svp", "please", "could you", "would you", "can you",
                "j'aimerais", "je voudrais", "i'd like",
            ], 0.8),
            // Commands have a weight of 0.7 because imperative verbs can
            // also appear in narratives.
            (Intent::Command, vec![
                "fais", "fait", "do", "make", "exécute", "lance", "arrête",
                "stop", "commence", "démarre", "éteins", "allume", "supprime",
                "delete", "run", "start", "shut down",
            ], 0.7),
            // Emotion expressions have a weight of 0.7 because they rely
            // on formulas like "je suis" + emotional adjective.
            (Intent::EmotionExpression, vec![
                "je suis triste", "je suis content", "j'ai peur", "je suis heureux",
                "je me sens", "i feel", "i am sad", "i am happy", "je suis en colère",
                "ça me rend", "je ressens", "j'éprouve",
                "triste", "content", "heureux", "peur", "angoisse",
            ], 0.7),
            // Threats have a high weight (0.85) because it is important to
            // detect them accurately for safety reasons.
            (Intent::Threat, vec![
                "je vais te", "je vais vous", "tu vas voir", "gare à",
                "attention", "menace", "i'll hurt", "i will destroy",
                "tu vas le regretter", "prends garde",
            ], 0.85),
            // Compliments have a weight of 0.8 thanks to their distinctive formulas.
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
            // Information sharing has a low weight (0.6) because temporal
            // and narrative markers are common in many contexts.
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

    /// Classifies the intent of tokenized text.
    ///
    /// The algorithm proceeds in two steps:
    ///   1. Punctuation check (a trailing '?' = default Question intent at 0.6)
    ///   2. Traverse all patterns: for each intent, count keywords found in
    ///      the full text (for multi-word expressions) and in individual tokens.
    ///      Score = (matches / total_keywords) * base_weight.
    ///      The intent with the best score wins.
    ///
    /// Parameters:
    ///   - tokens: the list of normalized lowercase tokens
    ///   - raw_text: the original raw text (for multi-word expression detection)
    ///
    /// Returns: an IntentResult with the primary intent and confidence score
    pub fn classify(&self, tokens: &[String], raw_text: &str) -> IntentResult {
        let lower_text = raw_text.to_lowercase();
        let mut best_intent = Intent::Unknown;
        let mut best_score = 0.0;

        // Check punctuation patterns first:
        // a text ending with '?' is probably a question (base score 0.6)
        if raw_text.trim().ends_with('?') {
            best_intent = Intent::Question;
            best_score = 0.6;
        }

        // Traverse each intent pattern
        for (intent, keywords, base_weight) in &self.patterns {
            let mut match_count = 0;
            let total = keywords.len().max(1);

            for kw in keywords {
                // Check in the full text (for multi-word expressions
                // like "je suis triste" or "au secours")
                if lower_text.contains(kw) {
                    match_count += 1;
                }
                // Also check in individual tokens (for single words)
                for token in tokens {
                    if token == kw {
                        match_count += 1;
                        break;
                    }
                }
            }

            // Compute score: match ratio * base weight
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
