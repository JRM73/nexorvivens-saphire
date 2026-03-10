// =============================================================================
// intent.rs — Detection d'intention par Naive Bayes simplifie
//
// Role : Couche 2B du pipeline NLP. Classifie l'intention communicative du
//        message (question, ordre, expression d'emotion, salutation, menace,
//        etc.) en utilisant une approche simplifiee inspiree de Naive Bayes :
//        on compare les tokens et le texte brut a des listes de mots-cles
//        associes a chaque intention, ponderes par un poids de base.
//
// Dependances :
//   - serde : serialisation/deserialisation des resultats et des enumerations
//
// Place dans l'architecture :
//   Appele par NlpPipeline::analyze() apres le pretraitement. L'intention
//   detectee est utilisee par le systeme de profilage humain (human_profiler)
//   pour estimer le profil OCEAN de l'interlocuteur et pour guider les
//   strategies d'adaptation de la communication.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Enumeration des intentions communicatives detectables.
///
/// Chaque variante represente un type d'intention que l'emetteur du message
/// peut avoir. La detection est basee sur des mots-cles indicateurs bilingues (FR+EN).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    /// Question : demande d'information (pourquoi, comment, who, what, etc.)
    Question,
    /// Demande polie : requete formulee avec des marqueurs de politesse (s'il te plait, please)
    Request,
    /// Expression d'emotion : le locuteur exprime un etat emotionnel (je suis triste, I feel)
    EmotionExpression,
    /// Partage d'information : le locuteur raconte ou informe (hier, j'ai vu, yesterday)
    InfoSharing,
    /// Ordre direct : commande imperative (fais, arrete, do, make)
    Command,
    /// Menace : le locuteur formule une menace (je vais te, tu vas voir, gare a)
    Threat,
    /// Compliment : le locuteur fait un eloge (bravo, tu es super, well done)
    Compliment,
    /// Recherche d'aide : le locuteur demande de l'aide (aide, help, besoin, au secours)
    HelpSeeking,
    /// Salutation : formule de salutation (bonjour, salut, hello, hi)
    Greeting,
    /// Adieu : formule de depart (au revoir, bye, bonne nuit, farewell)
    Farewell,
    /// Reflexion philosophique : question existentielle ou metacognitive (conscience, existence)
    Philosophical,
    /// Intention non identifiee : aucun pattern ne correspond suffisamment
    Unknown,
}

impl Intent {
    /// Retourne la representation textuelle de l'intention (en francais).
    ///
    /// Retour : une chaine de caracteres statique decrivant l'intention
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

/// Resultat de la classification d'intention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResult {
    /// L'intention principale detectee (la plus probable)
    pub primary_intent: Intent,
    /// Le score de confiance de la classification dans [0.0, 1.0].
    /// Plus le score est eleve, plus la classification est sure.
    pub confidence: f64,
}

/// Classificateur d'intention base sur la correspondance de mots-cles ponderes.
///
/// Approche simplifiee inspiree de Naive Bayes : pour chaque intention, on definit
/// une liste de mots-cles indicateurs et un poids de base. Le score d'une intention
/// est calcule comme le ratio de mots-cles trouves multiplie par le poids de base.
/// L'intention avec le meilleur score gagne.
pub struct IntentClassifier {
    /// Liste des patterns : (intention, mots-cles indicateurs, poids de base).
    /// Le poids de base module la confiance maximale atteignable pour chaque intention.
    /// Par exemple, les salutations ont un poids de 0.9 car elles sont tres fiables.
    patterns: Vec<(Intent, Vec<&'static str>, f64)>,
}

impl Default for IntentClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl IntentClassifier {
    /// Cree un nouveau classificateur d'intention avec les patterns par defaut.
    ///
    /// Les patterns couvrent 11 intentions avec des mots-cles bilingues FR+EN.
    /// Chaque pattern a un poids de base qui reflete la fiabilite de la detection
    /// pour cette intention.
    ///
    /// Retour : une instance de IntentClassifier prete a classifier
    pub fn new() -> Self {
        let patterns = vec![
            // (Intention, mots-cles indicateurs bilingues, poids de base)
            // Les salutations et les adieux ont un poids eleve (0.9) car
            // les formules sont tres specifiques et peu ambigues.
            (Intent::Greeting, vec![
                "bonjour", "salut", "hello", "hi", "hey", "coucou", "bonsoir",
                "yo", "wesh", "good morning", "good evening",
            ], 0.9),
            (Intent::Farewell, vec![
                "au revoir", "bye", "adieu", "bonne nuit", "goodbye", "ciao",
                "à bientôt", "à plus", "tchao", "farewell",
            ], 0.9),
            // Les questions ont un poids moyen (0.7) car les mots interrogatifs
            // peuvent apparaitre dans d'autres contextes.
            (Intent::Question, vec![
                "pourquoi", "comment", "quand", "où", "qui", "quoi", "quel",
                "quelle", "combien", "est-ce", "why", "how", "when", "where",
                "who", "what", "which", "do you", "can you", "is it",
            ], 0.7),
            // Les demandes polies ont un poids de 0.8 grace aux formules de politesse
            // qui les rendent assez distinctives.
            (Intent::Request, vec![
                "peux-tu", "pourrais-tu", "voudrais-tu", "s'il te plaît",
                "svp", "please", "could you", "would you", "can you",
                "j'aimerais", "je voudrais", "i'd like",
            ], 0.8),
            // Les ordres ont un poids de 0.7 car les verbes imperatifs peuvent
            // aussi apparaitre dans des recits.
            (Intent::Command, vec![
                "fais", "fait", "do", "make", "exécute", "lance", "arrête",
                "stop", "commence", "démarre", "éteins", "allume", "supprime",
                "delete", "run", "start", "shut down",
            ], 0.7),
            // Les expressions d'emotion ont un poids de 0.7 car elles reposent
            // sur des formules comme "je suis" + adjectif emotionnel.
            (Intent::EmotionExpression, vec![
                "je suis triste", "je suis content", "j'ai peur", "je suis heureux",
                "je me sens", "i feel", "i am sad", "i am happy", "je suis en colère",
                "ça me rend", "je ressens", "j'éprouve",
                "triste", "content", "heureux", "peur", "angoisse",
            ], 0.7),
            // Les menaces ont un poids eleve (0.85) car il est important de
            // les detecter avec precision pour des raisons de securite.
            (Intent::Threat, vec![
                "je vais te", "je vais vous", "tu vas voir", "gare à",
                "attention", "menace", "i'll hurt", "i will destroy",
                "tu vas le regretter", "prends garde",
            ], 0.85),
            // Les compliments ont un poids de 0.8 grace a leurs formules distinctives.
            (Intent::Compliment, vec![
                "tu es géniale", "tu es super", "bravo", "bien joué",
                "impressionnant", "you're amazing", "you're great", "well done",
                "good job", "chapeau", "tu es belle", "magnifique",
                "tu es intelligente", "you're smart",
            ], 0.8),
            // La recherche d'aide a un poids de 0.75.
            (Intent::HelpSeeking, vec![
                "aide", "aider", "help", "besoin", "need", "problème",
                "comment faire", "je ne sais pas", "i don't know",
                "au secours", "sos", "je suis perdu", "lost",
            ], 0.75),
            // Le partage d'information a un poids faible (0.6) car les marqueurs
            // temporels et narratifs sont communs dans de nombreux contextes.
            (Intent::InfoSharing, vec![
                "hier", "aujourd'hui", "j'ai vu", "j'ai fait", "il y a",
                "je suis allé", "yesterday", "today", "i saw", "i did",
                "figure-toi", "devine", "tu sais quoi", "did you know",
            ], 0.6),
            // La reflexion philosophique a un poids de 0.75.
            (Intent::Philosophical, vec![
                "penses-tu", "crois-tu", "sens-tu", "conscience",
                "existence", "être", "réalité", "libre arbitre",
                "do you think", "do you feel", "consciousness",
                "reality", "meaning", "purpose", "sens de la vie",
            ], 0.75),
        ];

        Self { patterns }
    }

    /// Classifie l'intention d'un texte tokenise.
    ///
    /// L'algorithme procede en deux temps :
    ///   1. Verification de la ponctuation (un '?' final = intention Question par defaut a 0.6)
    ///   2. Parcours de tous les patterns : pour chaque intention, on compte les mots-cles
    ///      trouves dans le texte complet (pour les expressions multi-mots) et dans les
    ///      tokens individuels. Le score est : (correspondances / total_mots_cles) * poids_base.
    ///      L'intention avec le meilleur score gagne.
    ///
    /// Parametres :
    ///   - tokens : la liste de tokens normalises en minuscules
    ///   - raw_text : le texte brut original (pour la detection d'expressions multi-mots)
    ///
    /// Retour : un IntentResult avec l'intention primaire et le score de confiance
    pub fn classify(&self, tokens: &[String], raw_text: &str) -> IntentResult {
        let lower_text = raw_text.to_lowercase();
        let mut best_intent = Intent::Unknown;
        let mut best_score = 0.0;

        // Verifier les patterns de ponctuation en premier :
        // un texte qui se termine par '?' est probablement une question (score de base 0.6)
        if raw_text.trim().ends_with('?') {
            best_intent = Intent::Question;
            best_score = 0.6;
        }

        // Parcourir chaque pattern d'intention
        for (intent, keywords, base_weight) in &self.patterns {
            let mut match_count = 0;
            let total = keywords.len().max(1);

            for kw in keywords {
                // Verifier dans le texte complet (pour les expressions multi-mots
                // comme "je suis triste" ou "au secours")
                if lower_text.contains(kw) {
                    match_count += 1;
                }
                // Verifier aussi dans les tokens individuels (pour les mots simples)
                for token in tokens {
                    if token == kw {
                        match_count += 1;
                        break;
                    }
                }
            }

            // Calculer le score : ratio de correspondances * poids de base
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
