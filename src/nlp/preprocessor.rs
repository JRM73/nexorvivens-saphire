// =============================================================================
// preprocessor.rs — Couche 1 du pipeline NLP : Pretraitement du texte
//
// Role : Transforme le texte brut en une liste de tokens normalises et extrait
//        les caracteristiques structurelles (ponctuation, majuscules, langue).
//        C'est la premiere etape du pipeline NLP, avant l'analyse de sentiment,
//        la detection d'intention et l'extraction de dimensions.
//
// Dependances :
//   - serde : serialisation/deserialisation des structures de resultats
//
// Place dans l'architecture :
//   Appele par NlpPipeline::analyze() en tant que premiere couche. Les tokens
//   produits alimentent ensuite les couches 2A (sentiment), 2B (intention) et
//   2C (dimensions). Les features structurelles sont utilisees par l'extracteur
//   de dimensions pour ajuster l'urgence et la nouveaute.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Caracteristiques structurelles extraites du texte brut avant normalisation.
///
/// Ces metriques capturent les indices non-lexicaux du message (ponctuation,
/// format d'ecriture) qui sont revelateurs de l'intensite emotionnelle et
/// de l'intention de l'emetteur.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralFeatures {
    /// Ratio de caracteres en majuscules parmi les caracteres alphabetiques.
    /// Un ratio eleve (> 0.5) indique souvent un cri ou une forte intensite emotionnelle.
    /// Plage : [0.0, 1.0]
    pub uppercase_ratio: f64,
    /// Nombre de points d'interrogation '?' dans le texte.
    /// Indicateur de questionnement ou de curiosite.
    pub question_marks: usize,
    /// Nombre de points d'exclamation '!' dans le texte.
    /// Indicateur d'intensite, d'urgence ou d'enthousiasme.
    pub exclamation_marks: usize,
    /// Presence de points de suspension '...' dans le texte.
    /// Indicateur d'hesitation, de sous-entendu ou de reflexion inachevee.
    pub has_ellipsis: bool,
    /// Longueur du message en nombre de tokens apres tokenisation.
    pub token_count: usize,
    /// Langue detectee par heuristique (francais, anglais ou inconnue).
    pub language: Language,
}

/// Langue detectee par heuristique basee sur les mots frequents.
///
/// La detection est simplifiee : on compare le nombre de marqueurs
/// francais et anglais trouves dans les tokens pour determiner la langue
/// dominante du message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    /// Francais detecte (majorite de marqueurs FR)
    French,
    /// Anglais detecte (majorite de marqueurs EN)
    English,
    /// Langue indeterminee (pas assez de marqueurs pour trancher)
    Unknown,
}

/// Preprocesseur de texte — premiere couche du pipeline NLP.
///
/// Structure sans etat (unit struct) : toute la logique est dans la methode process().
pub struct TextPreprocessor;

impl Default for TextPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TextPreprocessor {
    /// Cree une nouvelle instance du preprocesseur.
    ///
    /// Retour : une instance de TextPreprocessor
    pub fn new() -> Self {
        Self
    }

    /// Normalise et tokenise le texte brut.
    ///
    /// Le traitement se deroule en 4 etapes :
    ///   1. Extraction des features structurelles (avant normalisation, pour conserver la casse)
    ///   2. Normalisation en minuscules
    ///   3. Tokenisation par separation sur les espaces et la ponctuation
    ///   4. Detection de la langue par heuristique
    ///
    /// Parametres :
    ///   - text : le texte brut a pretraiter
    ///
    /// Retour : un tuple (tokens, features) ou :
    ///   - tokens : liste de mots normalises en minuscules, sans ponctuation
    ///   - features : les caracteristiques structurelles extraites
    pub fn process(&self, text: &str) -> (Vec<String>, StructuralFeatures) {
        // 1. Extraction des features structurelles (avant normalisation pour conserver la casse)
        // On compte les majuscules et le total de caracteres alphabetiques
        let uppercase_count = text.chars().filter(|c| c.is_uppercase()).count();
        let total_alpha = text.chars().filter(|c| c.is_alphabetic()).count().max(1);
        // Le ratio est calcule en evitant la division par zero (max(1))
        let uppercase_ratio = uppercase_count as f64 / total_alpha as f64;

        let question_marks = text.chars().filter(|c| *c == '?').count();
        let exclamation_marks = text.chars().filter(|c| *c == '!').count();
        let has_ellipsis = text.contains("...");

        // 2. Normalisation : passage en minuscules pour uniformiser la comparaison lexicale
        let normalized = text.to_lowercase();

        // 3. Tokenisation simple : separation sur les espaces et la ponctuation courante
        //    (virgule, point-virgule, deux-points). La ponctuation finale de chaque token
        //    est ensuite retiree (point, exclamation, interrogation, guillemets, apostrophe).
        let tokens: Vec<String> = normalized
            .split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == ':')
            .filter(|s| !s.is_empty())
            .map(|s| {
                // Separer la ponctuation finale pour ne garder que le mot nu
                s.trim_matches(|c: char| c == '.' || c == '!' || c == '?' || c == '"' || c == '\'')
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .collect();

        // 4. Detection de la langue par heuristique (comparaison de mots frequents FR vs EN)
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

/// Detecte la langue du texte par heuristique basee sur les mots frequents.
///
/// Methode : on compte combien de tokens correspondent a des marqueurs grammaticaux
/// frequents du francais et de l'anglais. La langue qui a le plus de correspondances
/// est retenue, a condition qu'il y en ait au moins une. En cas d'egalite ou d'absence
/// de correspondances, la langue est Unknown.
///
/// Parametres :
///   - tokens : la liste de tokens normalises en minuscules
///
/// Retour : la langue detectee (French, English ou Unknown)
fn detect_language(tokens: &[String]) -> Language {
    // Marqueurs grammaticaux frequents du francais (pronoms, articles, prepositions, verbes)
    let fr_markers = [
        "je", "tu", "il", "elle", "nous", "vous", "les", "des", "une", "est",
        "dans", "sur", "avec", "pour", "qui", "que", "pas", "mais", "cette",
        "son", "ses", "mon", "mes", "ton", "tes", "notre", "votre", "suis",
    ];
    // Marqueurs grammaticaux frequents de l'anglais (pronoms, articles, prepositions, verbes)
    let en_markers = [
        "i", "you", "he", "she", "we", "they", "the", "is", "are", "was",
        "in", "on", "with", "for", "who", "that", "not", "but", "this",
        "my", "your", "his", "her", "our", "their", "am", "have", "has",
    ];

    // Comptage des correspondances pour chaque langue
    let fr_count = tokens.iter().filter(|t| fr_markers.contains(&t.as_str())).count();
    let en_count = tokens.iter().filter(|t| en_markers.contains(&t.as_str())).count();

    // La langue avec le plus de marqueurs gagne, a condition d'en avoir au moins un
    if fr_count > en_count && fr_count > 0 {
        Language::French
    } else if en_count > fr_count && en_count > 0 {
        Language::English
    } else {
        Language::Unknown
    }
}
