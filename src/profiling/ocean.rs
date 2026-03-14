// =============================================================================
// ocean.rs — OCEAN (Big Five) model: OceanProfile, DimensionScore and
//            sub-facets of the 5 personality dimensions
//
// Role: Defines the central data structures for psychological profiling
//       according to the Big Five (OCEAN) model:
//         O = Openness (to experience)
//         C = Conscientiousness
//         E = Extraversion
//         A = Agreeableness
//         N = Neuroticism (emotional sensitivity)
//
//       Each dimension is decomposed into 6 sub-facets following the
//       NEO-PI-R model by Costa & McCrae.
//
// Dependencies:
//   - chrono: timestamping of profile computation
//   - serde: serialization/deserialization for persistence and WebSocket
//   - serde_json: JSON data generation for the WebSocket interface
//
// Place in architecture:
//   Data structure shared between self_profiler (Saphire's self-profile),
//   human_profiler (human profiles), adaptation (style generation) and
//   narrative (textual description). It is the core of the profiling system.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Complete psychological profile according to the Big Five OCEAN model.
///
/// Contains the 5 personality dimensions, each with a global score,
/// 6 sub-facets, a trend and a volatility. Also includes metadata
/// (computation date, number of data points, confidence).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanProfile {
    /// Openness to experience: intellectual curiosity, imagination, aesthetic sensitivity
    pub openness: DimensionScore,
    /// Conscientiousness: self-discipline, sense of duty, order, prudence
    pub conscientiousness: DimensionScore,
    /// Extraversion: sociability, assertiveness, stimulation seeking
    pub extraversion: DimensionScore,
    /// Agreeableness: trust, altruism, cooperation, empathy
    pub agreeableness: DimensionScore,
    /// Neuroticism: anxiety, irritability, vulnerability to stress
    pub neuroticism: DimensionScore,
    /// Timestamp of the last profile computation
    pub computed_at: DateTime<Utc>,
    /// Total number of data points (observations) used to build this profile.
    /// The higher this number, the more reliable the profile.
    pub data_points: u64,
    /// Confidence score in [0.0, 1.0] indicating profile reliability.
    /// Increases with the number of data points (saturates at 500 observations).
    pub confidence: f64,
}

impl Default for OceanProfile {
    /// Default OCEAN profile: all dimensions at 0.5 (neutral).
    ///
    /// A neutral profile is the starting point before any observation.
    /// Confidence is at 0.0 because no data has been collected yet.
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

/// Detailed score of an OCEAN dimension.
///
/// Each dimension is described by a global score, 6 sub-facets,
/// a trend (direction of change) and a volatility (amplitude
/// of recent variations).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    /// Global score of the dimension in [0.0, 1.0].
    /// 0.0 = extremely low, 0.5 = neutral, 1.0 = extremely high.
    pub score: f64,
    /// The 6 sub-facets of the dimension, each in [0.0, 1.0].
    /// The order corresponds to the *_FACETS constants defined below.
    pub facets: [f64; 6],
    /// Trend in [-1.0, +1.0]: direction of change between
    /// old and new score. Positive = rising, negative = declining.
    pub trend: f64,
    /// Volatility in [0.0, 1.0]: absolute amplitude of change
    /// between old and new score. High = unstable.
    pub volatility: f64,
}

impl DimensionScore {
    /// Creates a new dimension score with a uniform initial value.
    ///
    /// All sub-facets are initialized to the same value as the global score.
    /// Trend and volatility are at zero (no change observed yet).
    ///
    /// Parameters:
    ///   - initial: the initial value in [0.0, 1.0] for the score and facets
    ///
    /// Returns: an initialized DimensionScore
    pub fn new(initial: f64) -> Self {
        Self {
            score: initial,
            facets: [initial; 6],
            trend: 0.0,
            volatility: 0.0,
        }
    }
}

/// Names of the 6 sub-facets of the Openness dimension.
///
/// Based on the NEO-PI-R model:
///   [0] Imagination: ability to create mental scenarios
///   [1] Intellectual curiosity: attraction to new ideas
///   [2] Aesthetic sensitivity: appreciation of art and beauty
///   [3] Adventurism: taste for exploration and discovery
///   [4] Emotional depth: richness of inner emotional life
///   [5] Intellectual liberalism: openness to unconventional ideas
pub const OPENNESS_FACETS: [&str; 6] = [
    "Imagination",
    "Curiosite intellectuelle",
    "Sensibilite esthetique",
    "Aventurisme",
    "Profondeur emotionnelle",
    "Liberalisme intellectuel",
];

/// Names of the 6 sub-facets of the Conscientiousness dimension.
///
///   [0] Self-efficacy: confidence in one's ability to accomplish tasks
///   [1] Order: need for structure and organization
///   [2] Sense of duty: respect for rules and commitments
///   [3] Ambition: motivation to achieve high goals
///   [4] Self-discipline: ability to persevere despite distractions
///   [5] Prudence: thinking before acting, risk avoidance
pub const CONSCIENTIOUSNESS_FACETS: [&str; 6] = [
    "Auto-efficacite",
    "Ordre",
    "Sens du devoir",
    "Ambition",
    "Auto-discipline",
    "Prudence",
];

/// Names of the 6 sub-facets of the Extraversion dimension.
///
///   [0] Social warmth: ability to create emotional bonds
///   [1] Gregariousness: attraction to company and groups
///   [2] Assertiveness: confidence and dominance in interactions
///   [3] Activity level: general pace of action and energy
///   [4] Stimulation seeking: attraction to excitement and novelty
///   [5] Positive emotions: tendency to experience joy and enthusiasm
pub const EXTRAVERSION_FACETS: [&str; 6] = [
    "Chaleur sociale",
    "Gregarite",
    "Assertivite",
    "Niveau d'activite",
    "Recherche de stimulation",
    "Emotions positives",
];

/// Names of the 6 sub-facets of the Agreeableness dimension.
///
///   [0] Trust: tendency to believe in others' benevolence
///   [1] Sincerity: frankness and authenticity in interactions
///   [2] Altruism: concern for others' well-being
///   [3] Cooperation: seeking compromise and harmony
///   [4] Modesty: humility and absence of pretension
///   [5] Social sensitivity: empathy and awareness of others' emotions
pub const AGREEABLENESS_FACETS: [&str; 6] = [
    "Confiance",
    "Sincerite",
    "Altruisme",
    "Cooperation",
    "Modestie",
    "Sensibilite sociale",
];

/// Names of the 6 sub-facets of the Neuroticism dimension.
///
///   [0] Anxiety: tendency toward worry and tension
///   [1] Irritability: propensity for anger and annoyance
///   [2] Depressiveness: tendency toward sadness and discouragement
///   [3] Self-consciousness: sensitivity to others' judgment
///   [4] Impulsivity: difficulty controlling urges
///   [5] Vulnerability: sensitivity to stress and difficulty coping
pub const NEUROTICISM_FACETS: [&str; 6] = [
    "Anxiete",
    "Irritabilite",
    "Depressivite",
    "Conscience de soi",
    "Impulsivite",
    "Vulnerabilite",
];

impl OceanProfile {
    /// Identifies the dominant trait of the OCEAN profile.
    ///
    /// Compares the global scores of the 5 dimensions and returns the name
    /// (in French) of the highest dimension.
    ///
    /// Returns: a static string describing the dominant trait
    ///          (e.g.: "l'Ouverture", "la Rigueur", "l'Extraversion", etc.)
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

    /// Generates JSON data of the OCEAN profile for WebSocket transmission.
    ///
    /// Produces a JSON object containing the 5 dimensions with their scores,
    /// sub-facets, trends and volatilities, plus global metadata
    /// (confidence, data points count, dominant trait).
    ///
    /// Returns: a serde_json::Value representing the complete profile in JSON
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
