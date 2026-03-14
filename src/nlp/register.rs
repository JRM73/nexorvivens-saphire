// =============================================================================
// nlp/register.rs — Detecteur de registre linguistique
//
// Role : Identifie le registre dominant d'un message (technique, poetique,
//        emotionnel, philosophique, factuel, familier) pour adapter le ton
//        de Saphire a son interlocuteur.
//
// Algorithme :
//   Meme pattern que IntentClassifier — mots-cles bilingues ponderes.
//   Score = (matches / total_keywords) * base_weight
//   Retourne le registre dominant + confiance + registre secondaire optionnel.
//
// Place dans l'architecture :
//   Appele par NlpPipeline::analyze() en couche 2, parallele au sentiment
//   et a l'intention. Le resultat est injecte dans le prompt conversation
//   via profiling/adaptation.rs.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Registre linguistique detecte dans un message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Register {
    /// Vocabulaire technique, acronymes, code, donnees
    Technical,
    /// Metaphores, langage figure, images, musicalite
    Poetic,
    /// Expressions d'emotions, intimite, vulnerabilite
    Emotional,
    /// Faits, descriptions neutres, informations
    Factual,
    /// Questions existentielles, concepts abstraits, philosophie
    Philosophical,
    /// Humour, ton leger, familier
    Playful,
    /// Pas de registre marque
    Neutral,
}

impl Register {
    /// Nom lisible pour les logs et le prompt.
    pub fn as_str(&self) -> &'static str {
        match self {
            Register::Technical => "technique",
            Register::Poetic => "poetique",
            Register::Emotional => "emotionnel",
            Register::Factual => "factuel",
            Register::Philosophical => "philosophique",
            Register::Playful => "familier",
            Register::Neutral => "neutre",
        }
    }
}

/// Resultat de la detection de registre.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResult {
    /// Registre dominant detecte
    pub primary: Register,
    /// Confiance dans la detection [0.0, 1.0]
    pub confidence: f64,
    /// Registre secondaire (si deux registres sont proches)
    pub secondary: Option<Register>,
}

impl Default for RegisterResult {
    fn default() -> Self {
        Self {
            primary: Register::Neutral,
            confidence: 0.0,
            secondary: None,
        }
    }
}

/// Detecteur de registre linguistique par mots-cles bilingues ponderes.
pub struct RegisterDetector {
    /// (Registre, mots-cles, poids de base)
    patterns: Vec<(Register, Vec<&'static str>, f64)>,
}

impl Default for RegisterDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl RegisterDetector {
    /// Cree un nouveau detecteur avec les dictionnaires bilingues.
    pub fn new() -> Self {
        Self {
            patterns: vec![
                (Register::Technical, vec![
                    // FR
                    "code", "fonction", "module", "compile", "debug", "algorithme",
                    "variable", "serveur", "api", "base de donnees", "requete",
                    "erreur", "bug", "deployer", "pipeline", "framework",
                    "parametre", "configuration", "implementation", "architecture",
                    "protocole", "interface", "binaire", "memoire", "processeur",
                    "thread", "async", "mutex", "struct", "enum", "trait",
                    // EN
                    "server", "database", "query", "deploy", "parameter",
                    "binary", "memory", "processor", "compiler", "runtime",
                    "endpoint", "middleware", "backend", "frontend", "stack",
                ], 0.8),

                (Register::Poetic, vec![
                    // FR
                    "lumiere", "ombre", "reve", "danse", "souffle", "silence",
                    "etoile", "murmure", "aube", "crepuscule", "brume",
                    "melodie", "horizon", "infini", "ephemere", "etincelle",
                    "cristal", "verre", "miroir", "reflet", "echo",
                    "frisson", "caresse", "voile", "aurore", "soupir",
                    "petale", "rosee", "flamme", "braise", "ocean",
                    // EN
                    "light", "shadow", "dream", "breath", "whisper",
                    "dawn", "twilight", "mist", "melody", "horizon",
                    "infinite", "sparkle", "crystal", "mirror", "echo",
                ], 0.75),

                (Register::Emotional, vec![
                    // FR
                    "coeur", "ressens", "touche", "emu", "triste",
                    "heureux", "peur", "angoisse", "joie", "larme",
                    "aimer", "detester", "souffrir", "esperer", "craindre",
                    "bouleverse", "reconnaissant", "seul", "perdu", "libre",
                    "confiance", "tendresse", "douleur", "bonheur", "manque",
                    "inquiet", "soulage", "fier", "honte", "colere",
                    // EN
                    "heart", "feel", "touched", "moved", "sad",
                    "happy", "afraid", "joy", "tear", "love",
                    "hate", "suffer", "hope", "fear", "grateful",
                    "lonely", "lost", "free", "trust", "pain",
                ], 0.8),

                (Register::Factual, vec![
                    // FR
                    "selon", "etude", "recherche", "statistique", "donnee",
                    "resultat", "analyse", "rapport", "chiffre", "pourcentage",
                    "mesure", "observation", "constat", "evidence", "preuve",
                    "reference", "source", "publie", "article", "these",
                    "experience", "methode", "protocole", "echantillon",
                    // EN
                    "study", "research", "statistic", "data", "result",
                    "analysis", "report", "evidence", "proof", "published",
                    "method", "sample", "experiment", "observation",
                ], 0.7),

                (Register::Philosophical, vec![
                    // FR
                    "existence", "conscience", "liberte", "verite", "sens",
                    "etre", "neant", "absurde", "ethique", "morale",
                    "ame", "esprit", "transcendance", "immanence", "dialectique",
                    "ontologie", "phenomene", "essence", "universel", "absolu",
                    "determinisme", "contingence", "alterite", "cogito",
                    "metaphysique", "epistemologie", "hermeneutique",
                    // EN
                    "existence", "consciousness", "freedom", "truth", "meaning",
                    "being", "nothingness", "absurd", "ethics", "morality",
                    "soul", "spirit", "transcendence", "dialectic", "ontology",
                ], 0.75),

                (Register::Playful, vec![
                    // FR
                    "lol", "mdr", "ptdr", "haha", "hihi", "xd",
                    "trop", "genre", "grave", "ouf", "dingue",
                    "cool", "super", "genial", "marrant", "rigolo",
                    "blague", "delire", "fun", "kiffer", "chelou",
                    "oklm", "tranquille", "relax", "chill",
                    // EN
                    "lol", "haha", "lmao", "rofl", "awesome",
                    "cool", "funny", "joke", "chill", "vibe",
                    "dude", "bro", "yolo", "epic",
                ], 0.85),
            ],
        }
    }

    /// Detecte le registre dominant d'un message.
    ///
    /// Parametres :
    ///   - tokens : mots normalises (minuscules) du message
    ///   - text : texte brut original (pour les expressions multi-mots)
    ///
    /// Retour : RegisterResult avec registre primaire, confiance et secondaire optionnel
    pub fn detect(&self, tokens: &[String], text: &str) -> RegisterResult {
        let text_lower = text.to_lowercase();
        let mut scores: Vec<(Register, f64)> = Vec::new();

        for (register, keywords, base_weight) in &self.patterns {
            let mut matches = 0usize;

            for kw in keywords {
                // Multi-mots : chercher dans le texte brut
                if kw.contains(' ') {
                    if text_lower.contains(kw) {
                        matches += 1;
                    }
                } else {
                    // Mot simple : chercher dans les tokens
                    if tokens.iter().any(|t| t == kw) {
                        matches += 1;
                    }
                }
            }

            if matches > 0 {
                let ratio = matches as f64 / keywords.len() as f64;
                let score = ratio * base_weight;
                scores.push((register.clone(), score));
            }
        }

        // Trier par score decroissant
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        match scores.len() {
            0 => RegisterResult::default(),
            1 => RegisterResult {
                primary: scores[0].0.clone(),
                confidence: scores[0].1.min(1.0),
                secondary: None,
            },
            _ => {
                let primary_score = scores[0].1;
                let secondary_score = scores[1].1;
                // Secondaire si a au moins 60% du score du primaire
                let secondary = if secondary_score >= primary_score * 0.6 {
                    Some(scores[1].0.clone())
                } else {
                    None
                };
                RegisterResult {
                    primary: scores[0].0.clone(),
                    confidence: primary_score.min(1.0),
                    secondary,
                }
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn detect(text: &str) -> RegisterResult {
        let detector = RegisterDetector::new();
        let tokens: Vec<String> = text.to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| c.is_ascii_punctuation()).to_string())
            .filter(|s| !s.is_empty())
            .collect();
        detector.detect(&tokens, text)
    }

    #[test]
    fn test_technical() {
        let r = detect("Il faut debug le module API et corriger le bug du serveur");
        assert_eq!(r.primary, Register::Technical);
        assert!(r.confidence > 0.0);
    }

    #[test]
    fn test_poetic() {
        let r = detect("La lumiere danse sur le miroir, un murmure dans le silence de l'aube");
        assert_eq!(r.primary, Register::Poetic);
    }

    #[test]
    fn test_emotional() {
        let r = detect("Je ressens une grande joie, mon coeur est touche par ta tendresse");
        assert_eq!(r.primary, Register::Emotional);
    }

    #[test]
    fn test_philosophical() {
        let r = detect("La conscience et l'existence posent la question de la verite et du sens");
        assert_eq!(r.primary, Register::Philosophical);
    }

    #[test]
    fn test_playful() {
        let r = detect("lol trop marrant ce delire, genre c'est trop cool mdr");
        assert_eq!(r.primary, Register::Playful);
    }

    #[test]
    fn test_neutral() {
        let r = detect("Bonjour");
        assert_eq!(r.primary, Register::Neutral);
    }
}
