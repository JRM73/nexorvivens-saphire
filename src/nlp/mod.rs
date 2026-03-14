// =============================================================================
// nlp/mod.rs — Hybrid 3-layer NLP (Natural Language Processing) pipeline
//
// Role: Entry point for the NLP module. Orchestrates the complete text analysis
//       pipeline in 3 successive layers:
//         Layer 1 — Preprocessing (tokenization, normalization, structural features)
//         Layer 2A — Sentiment analysis (bilingual VADER-like lexicon)
//         Layer 2B — Intent detection (simplified Naive Bayes classifier)
//         Layer 2C — Dimension extraction (danger, reward, urgency, social, novelty)
//
// Dependencies:
//   - serde: serialization/deserialization of results
//   - crate::stimulus: Stimulus structure (emotional dimension vector)
//   - Submodules: preprocessor, sentiment, intent, dimensions, dictionaries
//
// Place in the architecture:
//   This module is called by Saphire's main cognitive loop to transform
//   raw user text into a multidimensional stimulus vector usable by the
//   profiling system and the consensus mechanism.
// =============================================================================

pub mod preprocessor;
pub mod sentiment;
pub mod intent;
pub mod dimensions;
pub mod dictionaries;
pub mod stagnation;
pub mod register;
pub mod extractor;

use serde::{Deserialize, Serialize};
use crate::stimulus::Stimulus;
use self::preprocessor::{TextPreprocessor, StructuralFeatures, Language};
use self::sentiment::{SentimentLexicon, SentimentResult};
use self::intent::{IntentClassifier, IntentResult};
use self::dimensions::DimensionExtractor;
use self::register::{RegisterDetector, RegisterResult};

/// Complete NLP (Natural Language Processing) analysis result.
///
/// Groups all information extracted from raw text after passing through
/// the 3-layer pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlpResult {
    /// The multidimensional stimulus extracted from text (danger, reward, urgency, social, novelty)
    pub stimulus: Stimulus,
    /// The sentiment analysis result (positive/negative polarity, compound score)
    pub sentiment: SentimentResult,
    /// The detected intent in the message (question, command, emotion expression, etc.)
    pub intent: IntentResult,
    /// The detected language of the message (French, English, or unknown)
    pub language: Language,
    /// The structural features of the text (uppercase, punctuation, length)
    pub structural_features: StructuralFeatures,
    /// The detected linguistic register (technical, poetic, emotional, etc.)
    pub register: RegisterResult,
}

/// Complete NLP pipeline.
///
/// Contains the 4 pipeline components: the preprocessor, the sentiment lexicon,
/// the intent classifier, and the dimension extractor. Each component is
/// initialized once and reused for each analysis call.
pub struct NlpPipeline {
    /// Preprocessing component: normalization, tokenization, structural feature extraction
    preprocessor: TextPreprocessor,
    /// Bilingual FR+EN sentiment lexicon with intensifiers and negations
    sentiment_lexicon: SentimentLexicon,
    /// Intent classifier via weighted keyword matching
    intent_classifier: IntentClassifier,
    /// Dimension extractor from text to Stimulus (5 emotional axes)
    dimension_extractor: DimensionExtractor,
    /// Linguistic register detector (technical, poetic, emotional, etc.)
    register_detector: RegisterDetector,
}

impl Default for NlpPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl NlpPipeline {
    /// Creates a new NLP pipeline instance.
    ///
    /// Initializes all components (preprocessor, lexicon, classifier, extractor)
    /// with their default dictionaries.
    ///
    /// Returns: an instance ready to analyze text
    pub fn new() -> Self {
        Self {
            preprocessor: TextPreprocessor::new(),
            sentiment_lexicon: SentimentLexicon::new(),
            intent_classifier: IntentClassifier::new(),
            dimension_extractor: DimensionExtractor::new(),
            register_detector: RegisterDetector::new(),
        }
    }

    /// Complete analysis of raw text.
    ///
    /// Runs the 3-layer pipeline:
    ///   1. Preprocessing: tokenization + structural feature extraction
    ///   2. Parallel analyses: sentiment (2A), intent (2B), dimensions (2C)
    ///   3. Fusion: sentiment adjusts the stimulus (reward if positive, danger if negative)
    ///
    /// Parameter:
    ///   - text: the raw text to analyze (user message)
    ///
    /// Returns: an NlpResult containing the stimulus, sentiment, intent,
    ///          language, and structural features
    pub fn analyze(&self, text: &str) -> NlpResult {
        // Layer 1: Preprocessing — tokenization and structural feature extraction
        let (tokens, features) = self.preprocessor.process(text);

        // Layer 2A: Sentiment — positive/negative polarity analysis
        let sentiment = self.sentiment_lexicon.analyze(&tokens);

        // Layer 2B: Intent — communicative intent classification
        let intent = self.intent_classifier.classify(&tokens, text);

        // Layer 2C: Dimension extraction — convert tokens to Stimulus vector
        let mut stimulus = self.dimension_extractor.extract(&tokens, text, &features);

        // Adjust the stimulus by sentiment:
        // A strongly positive sentiment (compound > 0.3) reinforces the "reward" dimension
        // by adding 30% of the sentiment's positive component.
        if sentiment.compound > 0.3 {
            stimulus.reward = (stimulus.reward + sentiment.positive * 0.3).min(1.0);
        }
        // A strongly negative sentiment (compound < -0.3) reinforces the "danger" dimension
        // by adding 20% of the sentiment's negative component.
        if sentiment.compound < -0.3 {
            stimulus.danger = (stimulus.danger + sentiment.negative * 0.2).min(1.0);
        }

        // Layer 2D: Register — detect the dominant linguistic register
        let register = self.register_detector.detect(&tokens, text);

        NlpResult {
            stimulus,
            sentiment,
            intent,
            language: features.language,
            structural_features: features,
            register,
        }
    }

    /// LLM-enriched analysis: sentiment, intent, and register are determined
    /// by the LLM instead of rules. Structural features, language, and stimulus
    /// remain rule-based. Falls back to analyze() if the LLM fails.
    pub async fn analyze_with_llm(&self, text: &str, llm_config: &crate::llm::LlmConfig) -> NlpResult {
        // First, rule-based analysis (baseline)
        let mut result = self.analyze(text);

        // LLM call for sentiment + intent + register
        let backend = crate::llm::create_backend(llm_config);
        let system = "Tu es un analyseur linguistique. Analyse le message suivant et retourne \
                      UNIQUEMENT un JSON avec ces champs :\n\
                      - \"sentiment\": nombre entre -1.0 (tres negatif) et 1.0 (tres positif)\n\
                      - \"positive\": nombre entre 0.0 et 1.0 (intensite positive)\n\
                      - \"negative\": nombre entre 0.0 et 1.0 (intensite negative)\n\
                      - \"contradiction\": true/false (le message contient-il une contradiction ou ambivalence)\n\
                      - \"intent\": un parmi [\"question\", \"demande\", \"expression_emotion\", \"partage_info\", \
                        \"ordre\", \"menace\", \"compliment\", \"recherche_aide\", \"salutation\", \"adieu\", \
                        \"philosophique\", \"inconnu\"]\n\
                      - \"intent_confidence\": nombre entre 0.0 et 1.0\n\
                      - \"register\": un parmi [\"technique\", \"poetique\", \"emotionnel\", \"factuel\", \
                        \"philosophique\", \"familier\", \"neutre\"]\n\
                      - \"register_confidence\": nombre entre 0.0 et 1.0\n\n\
                      Reponds UNIQUEMENT avec le JSON, sans commentaire.".to_string();
        let user_msg = text.to_string();

        let llm_result = tokio::task::spawn_blocking(move || {
            backend.chat(&system, &user_msg, 0.1, 120)
        }).await;

        match llm_result {
            Ok(Ok(raw)) => {
                // Extract JSON from response (may be wrapped in ```json ... ```)
                let json_str = raw.trim()
                    .trim_start_matches("```json")
                    .trim_start_matches("```")
                    .trim_end_matches("```")
                    .trim();
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                    // Sentiment
                    if let Some(compound) = parsed["sentiment"].as_f64() {
                        let compound = compound.clamp(-1.0, 1.0);
                        let positive = parsed["positive"].as_f64().unwrap_or(0.0).clamp(0.0, 1.0);
                        let negative = parsed["negative"].as_f64().unwrap_or(0.0).clamp(0.0, 1.0);
                        let has_contradiction = parsed["contradiction"].as_bool().unwrap_or(false);
                        result.sentiment = SentimentResult {
                            compound,
                            positive,
                            negative,
                            has_contradiction,
                        };
                        // Re-adjust stimulus with the new sentiment
                        if compound > 0.3 {
                            result.stimulus.reward = (result.stimulus.reward + positive * 0.3).min(1.0);
                        }
                        if compound < -0.3 {
                            result.stimulus.danger = (result.stimulus.danger + negative * 0.2).min(1.0);
                        }
                    }
                    // Intent
                    if let Some(intent_str) = parsed["intent"].as_str() {
                        let intent = match intent_str {
                            "question" => intent::Intent::Question,
                            "demande" => intent::Intent::Request,
                            "expression_emotion" | "expression_émotion" => intent::Intent::EmotionExpression,
                            "partage_info" => intent::Intent::InfoSharing,
                            "ordre" => intent::Intent::Command,
                            "menace" => intent::Intent::Threat,
                            "compliment" => intent::Intent::Compliment,
                            "recherche_aide" => intent::Intent::HelpSeeking,
                            "salutation" => intent::Intent::Greeting,
                            "adieu" => intent::Intent::Farewell,
                            "philosophique" => intent::Intent::Philosophical,
                            _ => intent::Intent::Unknown,
                        };
                        let confidence = parsed["intent_confidence"].as_f64().unwrap_or(0.7).clamp(0.0, 1.0);
                        result.intent = intent::IntentResult {
                            primary_intent: intent,
                            confidence,
                        };
                    }
                    // Register
                    if let Some(reg_str) = parsed["register"].as_str() {
                        let reg = match reg_str {
                            "technique" => register::Register::Technical,
                            "poetique" | "poétique" => register::Register::Poetic,
                            "emotionnel" | "émotionnel" => register::Register::Emotional,
                            "factuel" => register::Register::Factual,
                            "philosophique" => register::Register::Philosophical,
                            "familier" => register::Register::Playful,
                            _ => register::Register::Neutral,
                        };
                        let confidence = parsed["register_confidence"].as_f64().unwrap_or(0.7).clamp(0.0, 1.0);
                        result.register = register::RegisterResult {
                            primary: reg,
                            confidence,
                            secondary: None,
                        };
                    }
                    tracing::debug!("NLP LLM: sentiment={:.2}, intent={:?}, register={}",
                        result.sentiment.compound,
                        result.intent.primary_intent,
                        result.register.primary.as_str());
                } else {
                    tracing::warn!("NLP LLM: JSON invalide '{}', fallback regles", json_str.chars().take(100).collect::<String>());
                }
            }
            _ => {
                tracing::warn!("NLP LLM: echec appel, fallback regles");
            }
        }

        result
    }
}
