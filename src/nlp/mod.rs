// =============================================================================
// nlp/mod.rs — Pipeline NLP (Natural Language Processing / Traitement Automatique
//              du Langage Naturel) hybride a 3 couches
//
// Role : Point d'entree du module NLP. Orchestre le pipeline complet d'analyse
//        textuelle en 3 couches successives :
//          Couche 1 — Pretraitement (tokenisation, normalisation, features structurelles)
//          Couche 2A — Analyse de sentiment (lexique VADER-like bilingue)
//          Couche 2B — Detection d'intention (classificateur Naive Bayes simplifie)
//          Couche 2C — Extraction de dimensions (danger, recompense, urgence, social, nouveaute)
//
// Dependances :
//   - serde : serialisation/deserialisation des resultats
//   - crate::stimulus : structure Stimulus (vecteur de dimensions emotionnelles)
//   - Sous-modules : preprocessor, sentiment, intent, dimensions, dictionaries
//
// Place dans l'architecture :
//   Ce module est appele par la boucle cognitive principale de Saphire pour
//   transformer le texte brut de l'utilisateur en un vecteur de stimulus
//   multidimensionnel exploitable par le systeme de profilage et le consensus.
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

/// Resultat complet de l'analyse NLP (Natural Language Processing).
///
/// Regroupe toutes les informations extraites du texte brut apres passage
/// dans le pipeline a 3 couches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlpResult {
    /// Le stimulus multidimensionnel extrait du texte (danger, recompense, urgence, social, nouveaute)
    pub stimulus: Stimulus,
    /// Le resultat de l'analyse de sentiment (polarite positive/negative, score compose)
    pub sentiment: SentimentResult,
    /// L'intention detectee dans le message (question, ordre, expression d'emotion, etc.)
    pub intent: IntentResult,
    /// La langue detectee du message (francais, anglais ou inconnue)
    pub language: Language,
    /// Les caracteristiques structurelles du texte (majuscules, ponctuation, longueur)
    pub structural_features: StructuralFeatures,
    /// Le registre linguistique detecte (technique, poetique, emotionnel, etc.)
    pub register: RegisterResult,
}

/// Pipeline NLP complet.
///
/// Contient les 4 composants du pipeline : le preprocesseur, le lexique de sentiment,
/// le classificateur d'intention et l'extracteur de dimensions. Chaque composant est
/// initialise une seule fois et reutilise a chaque appel d'analyse.
pub struct NlpPipeline {
    /// Composant de pretraitement : normalisation, tokenisation, extraction de features structurelles
    preprocessor: TextPreprocessor,
    /// Lexique de sentiment bilingue FR+EN avec intensifieurs et negations
    sentiment_lexicon: SentimentLexicon,
    /// Classificateur d'intention par correspondance de mots-cles ponderes
    intent_classifier: IntentClassifier,
    /// Extracteur de dimensions texte vers Stimulus (5 axes emotionnels)
    dimension_extractor: DimensionExtractor,
    /// Detecteur de registre linguistique (technique, poetique, emotionnel, etc.)
    register_detector: RegisterDetector,
}

impl Default for NlpPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl NlpPipeline {
    /// Cree une nouvelle instance du pipeline NLP.
    ///
    /// Initialise tous les composants (preprocesseur, lexique, classificateur, extracteur)
    /// avec leurs dictionnaires par defaut.
    ///
    /// Retour : une instance prete a analyser du texte
    pub fn new() -> Self {
        Self {
            preprocessor: TextPreprocessor::new(),
            sentiment_lexicon: SentimentLexicon::new(),
            intent_classifier: IntentClassifier::new(),
            dimension_extractor: DimensionExtractor::new(),
            register_detector: RegisterDetector::new(),
        }
    }

    /// Analyse complete d'un texte brut.
    ///
    /// Deroule le pipeline en 3 couches :
    ///   1. Pretraitement : tokenisation + extraction de features structurelles
    ///   2. Analyses paralleles : sentiment (2A), intention (2B), dimensions (2C)
    ///   3. Fusion : le sentiment ajuste le stimulus (recompense si positif, danger si negatif)
    ///
    /// Parametre :
    ///   - text : le texte brut a analyser (message de l'utilisateur)
    ///
    /// Retour : un NlpResult contenant le stimulus, le sentiment, l'intention,
    ///          la langue et les features structurelles
    pub fn analyze(&self, text: &str) -> NlpResult {
        // Couche 1 : Pretraitement — tokenisation et extraction des features structurelles
        let (tokens, features) = self.preprocessor.process(text);

        // Couche 2A : Sentiment — analyse de polarite positive/negative
        let sentiment = self.sentiment_lexicon.analyze(&tokens);

        // Couche 2B : Intention — classification de l'intention communicative
        let intent = self.intent_classifier.classify(&tokens, text);

        // Couche 2C : Extraction dimensions — conversion des tokens en vecteur Stimulus
        let mut stimulus = self.dimension_extractor.extract(&tokens, text, &features);

        // Ajuster le stimulus par le sentiment :
        // Un sentiment fortement positif (compound > 0.3) renforce la dimension "recompense"
        // en y ajoutant 30% de la composante positive du sentiment.
        if sentiment.compound > 0.3 {
            stimulus.reward = (stimulus.reward + sentiment.positive * 0.3).min(1.0);
        }
        // Un sentiment fortement negatif (compound < -0.3) renforce la dimension "danger"
        // en y ajoutant 20% de la composante negative du sentiment.
        if sentiment.compound < -0.3 {
            stimulus.danger = (stimulus.danger + sentiment.negative * 0.2).min(1.0);
        }

        // Couche 2D : Registre — detection du registre linguistique dominant
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

    /// Analyse enrichie par LLM : sentiment, intention et registre sont determines
    /// par le LLM au lieu des regles. Structural features, langue et stimulus restent
    /// par regles. Fallback sur analyze() si le LLM echoue.
    pub async fn analyze_with_llm(&self, text: &str, llm_config: &crate::llm::LlmConfig) -> NlpResult {
        // D'abord, analyse par regles (base)
        let mut result = self.analyze(text);

        // Appel LLM pour sentiment + intent + register
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
                // Extraire le JSON de la reponse (peut etre entoure de ```json ... ```)
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
                        // Re-ajuster le stimulus avec le nouveau sentiment
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
