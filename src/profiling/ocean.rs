// =============================================================================
// ocean.rs — Modele OCEAN (Big Five) : OceanProfile, DimensionScore et
//            sous-facettes des 5 dimensions de personnalite
//
// Role : Definit les structures de donnees centrales du profilage psychologique
//        selon le modele Big Five (OCEAN) :
//          O = Openness (Ouverture a l'experience)
//          C = Conscientiousness (Rigueur / Conscience professionnelle)
//          E = Extraversion
//          A = Agreeableness (Agreabilite / Amabilite)
//          N = Neuroticism (Nevrosisme / Sensibilite emotionnelle)
//
//        Chaque dimension est decomposee en 6 sous-facettes conformement au
//        modele NEO-PI-R de Costa & McCrae.
//
// Dependances :
//   - chrono : horodatage du moment de calcul du profil
//   - serde : serialisation/deserialisation pour la persistance et le WebSocket
//   - serde_json : generation de donnees JSON pour l'interface WebSocket
//
// Place dans l'architecture :
//   Structure de donnees partagee entre self_profiler (auto-profil de Saphire),
//   human_profiler (profil des humains), adaptation (generation de style) et
//   narrative (description textuelle). C'est le coeur du systeme de profilage.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Profil psychologique complet selon le modele Big Five OCEAN.
///
/// Contient les 5 dimensions de personnalite, chacune avec un score global,
/// 6 sous-facettes, une tendance et une volatilite. Inclut aussi des metadonnees
/// (date de calcul, nombre de points de donnees, confiance).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanProfile {
    /// Ouverture a l'experience : curiosite intellectuelle, imagination, sensibilite esthetique
    pub openness: DimensionScore,
    /// Rigueur : auto-discipline, sens du devoir, ordre, prudence
    pub conscientiousness: DimensionScore,
    /// Extraversion : sociabilite, assertivite, recherche de stimulation
    pub extraversion: DimensionScore,
    /// Agreabilite : confiance, altruisme, cooperation, empathie
    pub agreeableness: DimensionScore,
    /// Nevrosisme : anxiete, irritabilite, vulnerabilite au stress
    pub neuroticism: DimensionScore,
    /// Horodatage du dernier calcul du profil
    pub computed_at: DateTime<Utc>,
    /// Nombre total de points de donnees (observations) utilises pour construire ce profil.
    /// Plus ce nombre est eleve, plus le profil est fiable.
    pub data_points: u64,
    /// Score de confiance dans [0.0, 1.0] indiquant la fiabilite du profil.
    /// Augmente avec le nombre de points de donnees (sature a 500 observations).
    pub confidence: f64,
}

impl Default for OceanProfile {
    /// Profil OCEAN par defaut : toutes les dimensions a 0.5 (neutre).
    ///
    /// Un profil neutre est le point de depart avant toute observation.
    /// La confiance est a 0.0 car aucune donnee n'a encore ete collectee.
    fn default() -> Self {
        Self {
            openness: DimensionScore::new(0.5),
            conscientiousness: DimensionScore::new(0.5),
            extraversion: DimensionScore::new(0.5),
            agreeableness: DimensionScore::new(0.5),
            neuroticism: DimensionScore::new(0.5),
            computed_at: Utc::now(),
            data_points: 0,
            confidence: 0.0,
        }
    }
}

/// Score detaille d'une dimension OCEAN.
///
/// Chaque dimension est decrite par un score global, 6 sous-facettes,
/// une tendance (direction du changement) et une volatilite (amplitude
/// des variations recentes).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    /// Score global de la dimension dans [0.0, 1.0].
    /// 0.0 = extremement bas, 0.5 = neutre, 1.0 = extremement haut.
    pub score: f64,
    /// Les 6 sous-facettes de la dimension, chacune dans [0.0, 1.0].
    /// L'ordre correspond aux constantes *_FACETS definies ci-dessous.
    pub facets: [f64; 6],
    /// Tendance dans [-1.0, +1.0] : direction du changement entre
    /// l'ancien et le nouveau score. Positif = en hausse, negatif = en baisse.
    pub trend: f64,
    /// Volatilite dans [0.0, 1.0] : amplitude absolue du changement
    /// entre l'ancien et le nouveau score. Eleve = instable.
    pub volatility: f64,
}

impl DimensionScore {
    /// Cree un nouveau score de dimension avec une valeur initiale uniforme.
    ///
    /// Toutes les sous-facettes sont initialisees a la meme valeur que le score global.
    /// La tendance et la volatilite sont a zero (pas encore de changement observe).
    ///
    /// Parametres :
    ///   - initial : la valeur initiale dans [0.0, 1.0] pour le score et les facettes
    ///
    /// Retour : un DimensionScore initialise
    pub fn new(initial: f64) -> Self {
        Self {
            score: initial,
            facets: [initial; 6],
            trend: 0.0,
            volatility: 0.0,
        }
    }
}

/// Noms des 6 sous-facettes de la dimension Ouverture (Openness).
///
/// Basees sur le modele NEO-PI-R :
///   [0] Imagination : capacite a creer des scenarios mentaux
///   [1] Curiosite intellectuelle : attrait pour les idees nouvelles
///   [2] Sensibilite esthetique : appreciation de l'art et de la beaute
///   [3] Aventurisme : gout pour l'exploration et la decouverte
///   [4] Profondeur emotionnelle : richesse de la vie emotionnelle interieure
///   [5] Liberalisme intellectuel : ouverture aux idees non conventionnelles
pub const OPENNESS_FACETS: [&str; 6] = [
    "Imagination",
    "Curiosite intellectuelle",
    "Sensibilite esthetique",
    "Aventurisme",
    "Profondeur emotionnelle",
    "Liberalisme intellectuel",
];

/// Noms des 6 sous-facettes de la dimension Rigueur (Conscientiousness).
///
///   [0] Auto-efficacite : confiance dans ses capacites a accomplir des taches
///   [1] Ordre : besoin de structure et d'organisation
///   [2] Sens du devoir : respect des regles et des engagements
///   [3] Ambition : motivation a atteindre des objectifs eleves
///   [4] Auto-discipline : capacite a perseverer malgre les distractions
///   [5] Prudence : reflexion avant l'action, evitement des risques
pub const CONSCIENTIOUSNESS_FACETS: [&str; 6] = [
    "Auto-efficacite",
    "Ordre",
    "Sens du devoir",
    "Ambition",
    "Auto-discipline",
    "Prudence",
];

/// Noms des 6 sous-facettes de la dimension Extraversion.
///
///   [0] Chaleur sociale : capacite a creer des liens affectifs
///   [1] Gregarite : attrait pour la compagnie et les groupes
///   [2] Assertivite : confiance et dominance dans les interactions
///   [3] Niveau d'activite : rythme general d'action et d'energie
///   [4] Recherche de stimulation : attrait pour l'excitation et la nouveaute
///   [5] Emotions positives : tendance a eprouver de la joie et de l'enthousiasme
pub const EXTRAVERSION_FACETS: [&str; 6] = [
    "Chaleur sociale",
    "Gregarite",
    "Assertivite",
    "Niveau d'activite",
    "Recherche de stimulation",
    "Emotions positives",
];

/// Noms des 6 sous-facettes de la dimension Agreabilite (Agreeableness).
///
///   [0] Confiance : tendance a croire en la bienveillance des autres
///   [1] Sincerite : franchise et authenticite dans les interactions
///   [2] Altruisme : preoccupation pour le bien-etre d'autrui
///   [3] Cooperation : recherche de compromis et d'harmonie
///   [4] Modestie : humilite et absence de pretention
///   [5] Sensibilite sociale : empathie et conscience des emotions d'autrui
pub const AGREEABLENESS_FACETS: [&str; 6] = [
    "Confiance",
    "Sincerite",
    "Altruisme",
    "Cooperation",
    "Modestie",
    "Sensibilite sociale",
];

/// Noms des 6 sous-facettes de la dimension Nevrosisme (Neuroticism).
///
///   [0] Anxiete : tendance a l'inquietude et a la tension
///   [1] Irritabilite : propension a la colere et a l'agacement
///   [2] Depressivite : tendance a la tristesse et au decouragement
///   [3] Conscience de soi : sensibilite au regard des autres
///   [4] Impulsivite : difficulte a controler les pulsions
///   [5] Vulnerabilite : sensibilite au stress et difficulte a faire face
pub const NEUROTICISM_FACETS: [&str; 6] = [
    "Anxiete",
    "Irritabilite",
    "Depressivite",
    "Conscience de soi",
    "Impulsivite",
    "Vulnerabilite",
];

impl OceanProfile {
    /// Identifie le trait dominant du profil OCEAN.
    ///
    /// Compare les scores globaux des 5 dimensions et retourne le nom
    /// (en francais) de la dimension la plus elevee.
    ///
    /// Retour : une chaine de caracteres statique decrivant le trait dominant
    ///          (ex: "l'Ouverture", "la Rigueur", "l'Extraversion", etc.)
    pub fn dominant_trait(&self) -> &str {
        let scores = [
            (self.openness.score, "l'Ouverture"),
            (self.conscientiousness.score, "la Rigueur"),
            (self.extraversion.score, "l'Extraversion"),
            (self.agreeableness.score, "l'Agreabilite"),
            (self.neuroticism.score, "la Sensibilite emotionnelle"),
        ];
        scores.iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, name)| *name)
            .unwrap_or("indetermine")
    }

    /// Genere les donnees JSON du profil OCEAN pour transmission par WebSocket.
    ///
    /// Produit un objet JSON contenant les 5 dimensions avec leurs scores,
    /// sous-facettes, tendances et volatilites, plus les metadonnees globales
    /// (confiance, nombre de points, trait dominant).
    ///
    /// Retour : un serde_json::Value representant le profil complet en JSON
    pub fn ws_data(&self) -> serde_json::Value {
        serde_json::json!({
            "openness": {
                "score": self.openness.score,
                "facets": self.openness.facets,
                "trend": self.openness.trend,
                "volatility": self.openness.volatility,
            },
            "conscientiousness": {
                "score": self.conscientiousness.score,
                "facets": self.conscientiousness.facets,
                "trend": self.conscientiousness.trend,
                "volatility": self.conscientiousness.volatility,
            },
            "extraversion": {
                "score": self.extraversion.score,
                "facets": self.extraversion.facets,
                "trend": self.extraversion.trend,
                "volatility": self.extraversion.volatility,
            },
            "agreeableness": {
                "score": self.agreeableness.score,
                "facets": self.agreeableness.facets,
                "trend": self.agreeableness.trend,
                "volatility": self.agreeableness.volatility,
            },
            "neuroticism": {
                "score": self.neuroticism.score,
                "facets": self.neuroticism.facets,
                "trend": self.neuroticism.trend,
                "volatility": self.neuroticism.volatility,
            },
            "confidence": self.confidence,
            "data_points": self.data_points,
            "dominant_trait": self.dominant_trait(),
        })
    }
}
