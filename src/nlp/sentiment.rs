// =============================================================================
// sentiment.rs — Analyse de sentiment VADER-like bilingue (FR+EN, 400+ mots)
//
// Role : Couche 2A du pipeline NLP. Calcule la polarite emotionnelle d'un texte
//        (positif, negatif, compose) en utilisant un lexique de sentiment enrichi
//        d'intensifieurs (boosters/attenuateurs), de negations et de pivots
//        adversatifs (conjonctions de contraste comme "mais", "however").
//
// Approche inspiree de VADER (Valence Aware Dictionary and sEntiment Reasoner),
// adaptee au bilinguisme francais-anglais avec un lexique de 400+ mots.
//
// Dependances :
//   - std::collections::HashMap : stockage performant des lexiques mot -> score
//   - serde : serialisation des resultats de sentiment
//   - super::dictionaries : fournit les lexiques bilingues (mots, intensifieurs,
//     negations, conjonctions adversatives)
//
// Place dans l'architecture :
//   Appele par NlpPipeline::analyze() apres le pretraitement. Le resultat de
//   sentiment influence ensuite le Stimulus (ajustement des dimensions recompense
//   et danger) et est transmis dans NlpResult.
// =============================================================================

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::dictionaries;

/// Resultat de l'analyse de sentiment d'un texte.
///
/// Les scores sont calcules a partir du lexique de sentiment, en tenant compte
/// des intensifieurs, des negations et des pivots adversatifs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentResult {
    /// Score compose normalise dans [-1, +1].
    /// -1 = sentiment tres negatif, 0 = neutre, +1 = sentiment tres positif.
    pub compound: f64,
    /// Score positif dans [0, 1] — extrait de la partie positive du compound.
    pub positive: f64,
    /// Score negatif dans [0, 1] — valeur absolue de la partie negative du compound.
    pub negative: f64,
    /// Indique si une contradiction a ete detectee (pivot adversatif dans le texte,
    /// par exemple "mais", "however"). Utile pour signaler l'ambivalence du message.
    pub has_contradiction: bool,
}

/// Lexique de sentiment avec intensifieurs, negations et pivots adversatifs.
///
/// Le lexique est construit une seule fois a l'initialisation a partir du module
/// dictionaries, puis reutilise pour chaque analyse.
pub struct SentimentLexicon {
    /// Dictionnaire des mots de sentiment : mot -> polarite dans [-1.0, +1.0].
    /// Les mots positifs ont une polarite > 0, les mots negatifs < 0.
    words: HashMap<String, f64>,
    /// Dictionnaire des intensifieurs : mot -> multiplicateur.
    /// Les boosters (> 1.0) amplifient le sentiment (ex: "tres" = 1.5x).
    /// Les attenuateurs (< 1.0) le reduisent (ex: "un peu" = 0.5x).
    boosters: HashMap<String, f64>,
    /// Liste des mots de negation (ex: "ne", "pas", "not", "never").
    /// Une negation active inverse partiellement la polarite des mots suivants.
    negations: Vec<String>,
    /// Liste des conjonctions adversatives (ex: "mais", "cependant", "but", "however").
    /// Un pivot adversatif donne plus de poids a la partie du texte qui suit la conjonction
    /// (70% apres, 30% avant), car en linguistique la clause post-adversative porte
    /// generalement le sentiment dominant.
    adversatives: Vec<String>,
}

impl Default for SentimentLexicon {
    fn default() -> Self {
        Self::new()
    }
}

impl SentimentLexicon {
    /// Cree un nouveau lexique de sentiment en chargeant les dictionnaires.
    ///
    /// Les dictionnaires sont definis dans le module dictionaries et contiennent
    /// les lexiques bilingues FR+EN.
    ///
    /// Retour : une instance de SentimentLexicon prete a analyser
    pub fn new() -> Self {
        // Charger le dictionnaire de mots de sentiment (mot -> polarite)
        let mut words = HashMap::new();
        for (word, polarity) in dictionaries::sentiment_words() {
            words.insert(word.to_string(), polarity);
        }

        // Charger le dictionnaire d'intensifieurs (mot -> multiplicateur)
        let mut boosters = HashMap::new();
        for (word, mult) in dictionaries::boosters() {
            boosters.insert(word.to_string(), mult);
        }

        // Charger les listes de negations et de conjonctions adversatives
        let negations = dictionaries::negations().iter().map(|s| s.to_string()).collect();
        let adversatives = dictionaries::adversatives().iter().map(|s| s.to_string()).collect();

        Self { words, boosters, negations, adversatives }
    }

    /// Analyse le sentiment d'une sequence de tokens.
    ///
    /// L'algorithme parcourt les tokens sequentiellement en maintenant un etat :
    ///   - Un multiplicateur d'intensite (current_booster, reinitialise apres utilisation)
    ///   - Un drapeau de negation active (avec un decompte de 3 tokens de portee)
    ///   - Un point de pivot adversatif (separe les scores avant/apres la conjonction)
    ///
    /// Regles de traitement pour chaque token :
    ///   1. Si c'est un pivot adversatif : separer les scores et continuer
    ///   2. Si c'est une negation : activer le mode negation pour 3 tokens
    ///   3. Si c'est un intensifieur : memoriser le multiplicateur
    ///   4. Si c'est un mot de sentiment : calculer le score ajuste
    ///
    /// Parametres :
    ///   - tokens : la liste de tokens normalises en minuscules
    ///
    /// Retour : un SentimentResult avec le score compose, positif, negatif et
    ///          l'indicateur de contradiction
    pub fn analyze(&self, tokens: &[String]) -> SentimentResult {
        let mut scores: Vec<f64> = Vec::new();
        // Multiplicateur courant applique au prochain mot de sentiment (1.0 = pas de modification)
        let mut current_booster = 1.0;
        // Indique si une negation est actuellement active
        let mut negation_active = false;
        // Decompte de tokens restants sous l'effet de la negation (portee de 3 tokens)
        let mut negation_countdown: i32 = 0;
        // Indique si un pivot adversatif a ete trouve dans le texte
        let mut pivot_found = false;
        // Scores accumules avant le pivot adversatif
        let mut before_pivot: Vec<f64> = Vec::new();

        for token in tokens.iter() {
            let lower = token.to_lowercase();

            // Detection de pivot adversatif (ex: "mais", "however")
            // Quand un pivot est trouve, les scores avant le pivot sont sauvegardes
            // et on recommence a zero pour la partie post-pivot.
            if self.adversatives.contains(&lower) {
                pivot_found = true;
                before_pivot = scores.clone();
                scores.clear();
                // Reinitialiser l'etat de negation et d'intensification
                negation_active = false;
                negation_countdown = 0;
                current_booster = 1.0;
                continue;
            }

            // Detection de negation (ex: "ne", "pas", "not", "never")
            // La negation est active pendant les 3 tokens suivants.
            if self.negations.contains(&lower) {
                negation_active = true;
                negation_countdown = 3;
                continue;
            }

            // Detection d'intensifieur (ex: "tres" = 1.5x, "un peu" = 0.5x)
            // Le multiplicateur est memorise et applique au prochain mot de sentiment.
            if let Some(&boost) = self.boosters.get(&lower) {
                current_booster = boost;
                continue;
            }

            // Detection de mot de sentiment
            // Le score est calcule comme : polarite * multiplicateur * facteur de negation.
            // La negation n'inverse pas totalement le score (facteur -0.75 au lieu de -1.0)
            // car "pas triste" n'est pas aussi fort que "heureux" en linguistique.
            if let Some(&polarity) = self.words.get(&lower) {
                let mut score = polarity * current_booster;
                if negation_active {
                    score *= -0.75; // Inversion partielle, pas totale
                }
                scores.push(score);
                // Reinitialiser le multiplicateur apres utilisation
                current_booster = 1.0;
            }

            // Decompte de la portee de la negation (3 tokens maximum)
            if negation_countdown > 0 {
                negation_countdown -= 1;
                if negation_countdown == 0 {
                    negation_active = false;
                }
            }
        }

        // Si un pivot adversatif a ete trouve : ponderation 30% avant / 70% apres.
        // Ceci reflete le principe linguistique selon lequel la clause apres "mais"
        // porte le sentiment dominant du locuteur.
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

        // Calcul du score compose : moyenne des scores, bornee a [-1, +1]
        let compound = if final_scores.is_empty() {
            0.0
        } else {
            (final_scores.iter().sum::<f64>() / final_scores.len() as f64).clamp(-1.0, 1.0)
        };

        SentimentResult {
            compound,
            // Le score positif est la partie > 0 du compound
            positive: compound.max(0.0),
            // Le score negatif est la valeur absolue de la partie < 0 du compound
            negative: compound.min(0.0).abs(),
            has_contradiction: pivot_found,
        }
    }
}
