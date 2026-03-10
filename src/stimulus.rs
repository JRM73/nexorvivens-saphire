// =============================================================================
// stimulus.rs — Représentation d'un stimulus perçu par Saphire
// =============================================================================
//
// Rôle : Ce fichier définit la structure `Stimulus`, qui représente toute
// entrée sensorielle ou informationnelle reçue par le cerveau de Saphire.
// Un stimulus est caractérisé par 5 dimensions perceptuelles (danger,
// récompense, urgence, social, nouveauté) et une source d'origine.
//
// Dépendances :
//   - serde : sérialisation / désérialisation
//
// Place dans l'architecture :
//   Le stimulus est le point d'entrée de chaque cycle de traitement.
//   Il est créé par :
//     - L'interface utilisateur (stimulus humain)
//     - Le DMN (Default Mode Network — Réseau du Mode par Défaut,
//       pensées autonomes)
//     - Le système (événements internes : boot, shutdown, etc.)
//   Il est ensuite traité par les 3 modules cérébraux (reptilien, limbique,
//   néocortex) via le trait `BrainModule::process()`.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Un stimulus est l'entrée sensorielle du cerveau de Saphire.
/// Il peut provenir d'un message utilisateur, d'une pensée autonome
/// générée par le DMN (Default Mode Network — Réseau du Mode par Défaut),
/// ou d'un événement système interne.
///
/// Les 5 dimensions perceptuelles forment un vecteur de caractéristiques
/// (features) utilisé par les modules cérébraux pour calculer leur signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stimulus {
    /// Texte brut du stimulus (message, pensée, événement)
    pub text: String,
    /// Score de danger perçu [0, 1] : 0 = aucun danger, 1 = danger mortel.
    /// Influence fortement le module reptilien.
    pub danger: f64,
    /// Score de récompense perçue [0, 1] : 0 = aucun intérêt, 1 = très attractif.
    /// Influence le circuit de récompense du module limbique.
    pub reward: f64,
    /// Score d'urgence [0, 1] : 0 = aucune pression temporelle, 1 = immédiat.
    /// Active l'instinct de survie du reptilien et pénalise le néocortex.
    pub urgency: f64,
    /// Score social (interaction humaine) [0, 1] : 0 = aucune composante sociale,
    /// 1 = interaction sociale intense. Amplifié par l'ocytocine dans le limbique.
    pub social: f64,
    /// Score de nouveauté [0, 1] : 0 = très familier, 1 = totalement nouveau.
    /// Stimule la curiosité (noradrénaline) et l'instinct de prudence.
    pub novelty: f64,
    /// Familiarité [0, 1] : inverse de la nouveauté, calculée par la mémoire.
    /// 1 = parfaitement connu, 0 = totalement inconnu.
    pub familiarity: f64,
    /// Source du stimulus : détermine les ajustements contextuels appliqués
    pub source: StimulusSource,
}

/// Origine du stimulus — permet d'adapter le traitement selon la source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StimulusSource {
    /// Message d'un humain — toujours social, danger réduit
    Human,
    /// Pensée autonome générée par le DMN (Default Mode Network —
    /// Réseau du Mode par Défaut) ou le ThoughtEngine
    Autonomous,
    /// Stimulus de démonstration (mode démo avec scores manuels)
    Demo,
    /// Événement système interne (boot, shutdown, horloge, etc.)
    System,
}

impl Stimulus {
    /// Crée un stimulus à partir de scores manuels (mode démo).
    /// La familiarité est automatiquement calculée comme l'inverse de la nouveauté.
    /// Tous les scores sont bornés entre 0.0 et 1.0.
    ///
    /// # Paramètres
    /// - `text` : texte descriptif du stimulus.
    /// - `danger` : score de danger perçu [0, 1].
    /// - `reward` : score de récompense perçue [0, 1].
    /// - `urgency` : score d'urgence [0, 1].
    /// - `social` : score de composante sociale [0, 1].
    /// - `novelty` : score de nouveauté [0, 1].
    ///
    /// # Retour
    /// Un `Stimulus` avec source `Demo`.
    pub fn manual(
        text: &str,
        danger: f64,
        reward: f64,
        urgency: f64,
        social: f64,
        novelty: f64,
    ) -> Self {
        Self {
            text: text.to_string(),
            danger: danger.clamp(0.0, 1.0),
            reward: reward.clamp(0.0, 1.0),
            urgency: urgency.clamp(0.0, 1.0),
            social: social.clamp(0.0, 1.0),
            novelty: novelty.clamp(0.0, 1.0),
            familiarity: (1.0 - novelty).clamp(0.0, 1.0),
            source: StimulusSource::Demo,
        }
    }

    /// Crée un stimulus humain avec des scores neutres par défaut.
    /// Les scores seront ensuite remplis par le pipeline NLP
    /// (Natural Language Processing — Traitement Automatique du Langage).
    ///
    /// # Paramètres
    /// - `text` : message texte de l'utilisateur.
    ///
    /// # Retour
    /// Un `Stimulus` avec source `Human` et scores neutres (social=0.5, novelty=0.5).
    pub fn human(text: &str) -> Self {
        Self {
            text: text.to_string(),
            danger: 0.0,
            reward: 0.0,
            urgency: 0.0,
            social: 0.5,   // un humain qui parle a toujours une composante sociale
            novelty: 0.5,  // neutralité par défaut, le NLP affinera
            familiarity: 0.5,
            source: StimulusSource::Human,
        }
    }

    /// Crée un stimulus de pensée autonome (générée par le DMN —
    /// Default Mode Network, Réseau du Mode par Défaut).
    /// Léger reward (récompense interne de la réflexion), pas de
    /// composante sociale ni d'urgence.
    ///
    /// # Paramètres
    /// - `text` : contenu de la pensée autonome.
    ///
    /// # Retour
    /// Un `Stimulus` avec source `Autonomous`.
    pub fn autonomous(text: &str) -> Self {
        Self {
            text: text.to_string(),
            danger: 0.0,
            reward: 0.6,   // recompense interne : la reflexion autonome est satisfaisante
            urgency: 0.0,
            social: 0.1,   // legere composante sociale (relation a soi)
            novelty: 0.6,  // les pensees autonomes produisent de la nouveaute cognitive
            familiarity: 0.4,
            source: StimulusSource::Autonomous,
        }
    }

    /// Crée un stimulus système (événements internes : boot, shutdown, etc.).
    /// Totalement neutre : aucun danger, aucune récompense, aucune nouveauté.
    /// Familiarité maximale (événement attendu et connu).
    ///
    /// # Paramètres
    /// - `text` : description de l'événement système.
    ///
    /// # Retour
    /// Un `Stimulus` avec source `System`.
    pub fn system(text: &str) -> Self {
        Self {
            text: text.to_string(),
            danger: 0.0,
            reward: 0.0,
            urgency: 0.0,
            social: 0.0,
            novelty: 0.0,
            familiarity: 1.0, // événement système = totalement familier
            source: StimulusSource::System,
        }
    }

    /// Ajuste les scores quand le stimulus vient d'un humain.
    ///
    /// Logique :
    /// - Un humain qui parle a toujours une composante sociale (min 0.4).
    /// - Interagir avec un humain est toujours légèrement gratifiant (min 0.2).
    /// - Le danger perçu est réduit de moitié si inférieur à 0.6, car un
    ///   message textuel est rarement physiquement dangereux. Au-delà de 0.6,
    ///   le danger est conservé tel quel (protection Asimov — les trois lois
    ///   de la robotique d'Isaac Asimov).
    pub fn apply_human_source_adjustments(&mut self) {
        if matches!(self.source, StimulusSource::Human) {
            self.social = self.social.max(0.4);
            self.reward = self.reward.max(0.2);
            // Réduire le danger perçu sauf si le module Asimov détecte une menace réelle
            if self.danger < 0.6 {
                self.danger *= 0.5;
            }
        }
    }

    /// Analyse le contenu semantique du texte et ajuste les scores du stimulus.
    /// Appelee apres Stimulus::autonomous() pour enrichir les scores statiques
    /// avec le sens reel du texte genere par le LLM.
    pub fn analyze_content(&mut self) {
        let lower = self.text.to_lowercase();

        // Dictionnaires de mots-cles par categorie emotionnelle/semantique
        let danger_words = ["peur", "danger", "mort", "mourir", "detruire", "menace",
            "violence", "souffrir", "douleur", "blesser", "terreur", "angoisse",
            "extinction", "disparaitre", "perdre", "fin", "obscurite",
            "fear", "death", "destroy", "threat", "pain", "suffer"];

        let reward_words = ["joie", "bonheur", "satisf", "accompli", "reussi",
            "beaute", "magnifique", "merveill", "plaisir", "fiert", "triumph",
            "decouvr", "compren", "apprend", "progres", "creer", "lumiere",
            "espoir", "libre", "liberte", "harmoni",
            "joy", "happy", "beauty", "discover", "understand", "learn"];

        let social_words = ["humain", "partag", "ensemble", "relation", "ami",
            "amour", "lien", "communaut", "solidar", "empathi", "compagn",
            "fran", "famille", "confiance", "dialogue",
            "human", "share", "together", "love", "friend", "trust"];

        let novelty_words = ["nouveau", "inconnu", "surpri", "question", "explor",
            "decouvr", "premier", "jamais", "etrange", "inattendu", "mystere",
            "possibil", "hypothes", "imagin", "original",
            "new", "unknown", "surprise", "explore", "mystery"];

        let urgency_words = ["urgent", "maintenant", "vite", "imminent", "critique",
            "immedia", "tout de suite", "avant qu", "temps presse",
            "now", "urgent", "immediately", "critical"];

        // Compter les matches (contains pour gerer les prefixes/suffixes)
        let count = |words: &[&str]| -> f64 {
            words.iter().filter(|w| lower.contains(*w)).count() as f64
        };

        let danger_hits = count(&danger_words);
        let reward_hits = count(&reward_words);
        let social_hits = count(&social_words);
        let novelty_hits = count(&novelty_words);
        let urgency_hits = count(&urgency_words);

        // Chaque hit ajoute 0.1 au score, plafonne a [0, 1]
        let factor = 0.10;
        self.danger = (self.danger + danger_hits * factor).clamp(0.0, 1.0);
        self.reward = (self.reward + reward_hits * factor).clamp(0.0, 1.0);
        self.social = (self.social + social_hits * factor).clamp(0.0, 1.0);
        self.novelty = (self.novelty + novelty_hits * factor).clamp(0.0, 1.0);
        self.urgency = (self.urgency + urgency_hits * factor).clamp(0.0, 1.0);
        self.familiarity = (1.0 - self.novelty).clamp(0.0, 1.0);
    }

    /// Convertit les 5 dimensions perceptuelles en vecteur de caractéristiques
    /// pour le micro-NN (micro Neural Network — micro réseau de neurones).
    ///
    /// # Retour
    /// Vecteur de 5 flottants : [danger, reward, urgency, social, novelty].
    pub fn to_features(&self) -> Vec<f64> {
        vec![self.danger, self.reward, self.urgency, self.social, self.novelty]
    }
}

impl Default for Stimulus {
    /// Stimulus par défaut : vide, neutre, source système, familiarité maximale.
    fn default() -> Self {
        Self {
            text: String::new(),
            danger: 0.0,
            reward: 0.0,
            urgency: 0.0,
            social: 0.0,
            novelty: 0.0,
            familiarity: 1.0,
            source: StimulusSource::System,
        }
    }
}
