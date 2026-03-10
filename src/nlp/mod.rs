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

use serde::{Deserialize, Serialize};
use crate::stimulus::Stimulus;
use self::preprocessor::{TextPreprocessor, StructuralFeatures, Language};
use self::sentiment::{SentimentLexicon, SentimentResult};
use self::intent::{IntentClassifier, IntentResult};
use self::dimensions::DimensionExtractor;

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

        NlpResult {
            stimulus,
            sentiment,
            intent,
            language: features.language,
            structural_features: features,
        }
    }
}
