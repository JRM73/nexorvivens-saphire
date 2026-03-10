// =============================================================================
// dimensions.rs — Extraction des 5 dimensions du Stimulus a partir du texte
//
// Role : Couche 2C du pipeline NLP. Convertit les tokens normalises en un
//        vecteur Stimulus a 5 dimensions emotionnelles :
//          - danger : niveau de menace percu (agression, violence, risque)
//          - recompense (reward) : potentiel de gratification (cadeau, succes, gain)
//          - urgence : pression temporelle percue (vite, maintenant, SOS)
//          - social : charge relationnelle (famille, amis, pronoms sociaux)
//          - nouveaute : degre de nouveaute ou d'inconnu (decouverte, mystere)
//
//        Chaque dimension est scoree par correspondance lexicale avec gestion
//        des negations, puis ajustee par les features structurelles (ponctuation,
//        majuscules).
//
// Dependances :
//   - std::collections::HashMap : stockage des lexiques dimension -> score
//   - crate::stimulus : structure Stimulus et StimulusSource
//   - super::dictionaries : fournit les lexiques par dimension (danger, reward, etc.)
//   - super::preprocessor : StructuralFeatures pour les ajustements structurels
//
// Place dans l'architecture :
//   Appele par NlpPipeline::analyze() apres le pretraitement. Le Stimulus produit
//   est le vecteur central du systeme cognitif de Saphire : il alimente le consensus
//   tri-cerbral (reptilien, limbique, neocortex), la chimie emotionnelle et le
//   systeme de profilage.
// =============================================================================

use std::collections::HashMap;
use crate::stimulus::Stimulus;
use super::dictionaries;
use super::preprocessor::StructuralFeatures;

/// Extracteur de dimensions : convertit du texte tokenise en un vecteur Stimulus.
///
/// Contient 5 lexiques (un par dimension) et une liste de negations. Chaque lexique
/// associe des mots a un score d'intensite dans [0.0, 1.0].
pub struct DimensionExtractor {
    /// Lexique de mots indicateurs de danger avec leur intensite (ex: "tuer" = 0.95, "risque" = 0.6)
    danger_words: HashMap<String, f64>,
    /// Lexique de mots indicateurs de recompense (ex: "victoire" = 0.8, "cadeau" = 0.7)
    reward_words: HashMap<String, f64>,
    /// Lexique de mots indicateurs d'urgence (ex: "urgent" = 0.9, "vite" = 0.8)
    urgency_words: HashMap<String, f64>,
    /// Lexique de mots indicateurs de socialite (ex: "famille" = 0.8, "ami" = 0.7)
    social_words: HashMap<String, f64>,
    /// Lexique de mots indicateurs de nouveaute (ex: "decouverte" = 0.8, "inedit" = 0.8)
    novelty_words: HashMap<String, f64>,
    /// Liste des mots de negation pour inverser partiellement l'effet des mots-cles
    negations: Vec<String>,
}

impl Default for DimensionExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl DimensionExtractor {
    /// Cree un nouvel extracteur de dimensions en chargeant les lexiques.
    ///
    /// Utilise une fonction utilitaire `to_map` pour convertir les vecteurs de tuples
    /// du module dictionaries en HashMaps performantes.
    ///
    /// Retour : une instance de DimensionExtractor prete a extraire
    pub fn new() -> Self {
        // Fonction utilitaire : convertit un Vec<(&str, f64)> en HashMap<String, f64>
        let to_map = |words: Vec<(&str, f64)>| -> HashMap<String, f64> {
            words.into_iter().map(|(w, s)| (w.to_string(), s)).collect()
        };

        Self {
            danger_words: to_map(dictionaries::danger_words()),
            reward_words: to_map(dictionaries::reward_words()),
            urgency_words: to_map(dictionaries::urgency_words()),
            social_words: to_map(dictionaries::social_words()),
            novelty_words: to_map(dictionaries::novelty_words()),
            negations: dictionaries::negations().iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Extrait les 5 dimensions d'un texte tokenise et construit un Stimulus.
    ///
    /// Le processus :
    ///   1. Score chaque dimension par correspondance lexicale (avec gestion des negations)
    ///   2. Ajuste l'urgence en fonction de la ponctuation (exclamation) et des majuscules
    ///   3. Ajuste la nouveaute en fonction des points d'interrogation
    ///   4. Calcule la familiarite comme l'inverse de la nouveaute
    ///
    /// Parametres :
    ///   - tokens : la liste de tokens normalises en minuscules
    ///   - raw_text : le texte brut original (conserve pour le champ text du Stimulus)
    ///   - features : les caracteristiques structurelles du texte (ponctuation, majuscules)
    ///
    /// Retour : un Stimulus avec les 5 dimensions scorees et bornees a [0.0, 1.0]
    pub fn extract(
        &self,
        tokens: &[String],
        raw_text: &str,
        features: &StructuralFeatures,
    ) -> Stimulus {
        // Scorer chaque dimension par correspondance lexicale
        let danger = self.score_dimension(tokens, &self.danger_words);
        let reward = self.score_dimension(tokens, &self.reward_words);
        let mut urgency = self.score_dimension(tokens, &self.urgency_words);
        let social = self.score_dimension(tokens, &self.social_words);
        let novelty = self.score_dimension(tokens, &self.novelty_words);

        // Ajuster l'urgence par la ponctuation :
        // Plus de 2 points d'exclamation indique une forte intensite (+0.2)
        if features.exclamation_marks > 2 {
            urgency = (urgency + 0.2).min(1.0);
        }
        // Un ratio de majuscules superieur a 50% indique un cri ou une insistance (+0.15)
        if features.uppercase_ratio > 0.5 {
            urgency = (urgency + 0.15).min(1.0);
        }

        // Ajuster la nouveaute par les points d'interrogation :
        // Les questions signalent de la curiosite, ce qui augmente legerement la nouveaute (+0.1)
        let novelty = if features.question_marks > 0 {
            (novelty + 0.1).min(1.0)
        } else {
            novelty
        };

        Stimulus {
            text: raw_text.to_string(),
            danger: danger.clamp(0.0, 1.0),
            reward: reward.clamp(0.0, 1.0),
            urgency: urgency.clamp(0.0, 1.0),
            social: social.clamp(0.0, 1.0),
            novelty: novelty.clamp(0.0, 1.0),
            // La familiarite est l'inverse de la nouveaute : plus c'est nouveau, moins c'est familier
            familiarity: (1.0 - novelty).clamp(0.0, 1.0),
            source: crate::stimulus::StimulusSource::Human,
        }
    }

    /// Calcule le score d'une dimension en tenant compte des negations.
    ///
    /// L'algorithme parcourt les tokens sequentiellement :
    ///   - Si un mot de negation est detecte, il active le mode negation pour 3 tokens.
    ///   - Si un mot du lexique est trouve, son score est reduit a 20% s'il est sous negation.
    ///   - Le score final est la moyenne des scores trouves, amplifiee par un facteur
    ///     logarithmique (racine carree du nombre de correspondances * 0.3) pour avantager
    ///     les textes avec de nombreux marqueurs d'une dimension.
    ///
    /// Parametres :
    ///   - tokens : la liste de tokens normalises en minuscules
    ///   - lexicon : le dictionnaire de mots-cles pour la dimension a scorer
    ///
    /// Retour : le score de la dimension dans [0.0, 1.0]
    fn score_dimension(&self, tokens: &[String], lexicon: &HashMap<String, f64>) -> f64 {
        let mut total_score = 0.0;
        let mut match_count = 0;
        let mut negation_active = false;
        let mut negation_countdown: i32 = 0;

        for token in tokens {
            let lower = token.to_lowercase();

            // Verifier si le token est un mot de negation
            if self.negations.contains(&lower) {
                negation_active = true;
                negation_countdown = 3;
                continue;
            }

            // Verifier si le token est dans le lexique de la dimension
            if let Some(&score) = lexicon.get(&lower) {
                // Si une negation est active, le score est reduit a 20% de sa valeur
                // (au lieu de 0 ou inversion, car "pas de danger" n'implique pas la securite)
                let effective = if negation_active { score * 0.2 } else { score };
                total_score += effective;
                match_count += 1;
            }

            // Decompte de la portee de la negation (3 tokens maximum)
            if negation_countdown > 0 {
                negation_countdown -= 1;
                if negation_countdown == 0 {
                    negation_active = false;
                }
            }
        }

        if match_count == 0 {
            0.0
        } else {
            // Normalisation : la moyenne brute est amplifiee par un facteur logarithmique
            // (sqrt(match_count) * 0.3) pour recompenser les textes riches en marqueurs.
            // Ce boost evite qu'un seul mot "danger" donne le meme score qu'un texte
            // avec 5 mots de danger differents. Le resultat est borne a [0.0, 1.0].
            let raw = total_score / match_count as f64;
            let boost = (match_count as f64).sqrt() * 0.3;
            (raw * (1.0 + boost)).clamp(0.0, 1.0)
        }
    }
}
