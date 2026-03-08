// =============================================================================
// emotions.rs — 36 emergent emotions + Mood (EMA - Exponential Moving Average)
// =============================================================================
//
// Purpose: This module computes Saphire's emotional state from its
// neurochemical state. Emotions are not hard-coded: they emerge from the
// cosine similarity between the current chemical vector and the "chemical
// recipes" of 36 predefined emotions.
//
// Scientific foundations:
//   - Constructionist theory (Barrett 2017, "How Emotions Are Made"):
//     emotions are constructed from core affect + conceptual categorization
//   - Core Affect (Russell 2003): a 2D pre-conceptual space (valence x arousal)
//   - VAD model (Valence-Arousal-Dominance): used here in its VA subset
//   - Circumplex model (Russell 1980): emotions arranged on a circle in VA space
//   - Cosine similarity: measures angular distance between neurochemical vectors
//
// Dependencies:
//   - serde: serialization / deserialization
//   - crate::neurochemistry::NeuroChemicalState: 7-dimensional chemical vector
//
// Architectural role:
//   This module is consulted after each processing cycle to determine the
//   dominant emotion. It is read by:
//     - consciousness.rs (the inner narrative uses the dominant emotion)
//     - the main engine (emotional state display)
//   The Mood (background affect) smooths instantaneous emotions over time
//   via an EMA (Exponential Moving Average).
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::neurochemistry::NeuroChemicalState;

/// Profile of a single emotion: its chemical "recipe" and psychological
/// characteristics (valence and arousal).
///
/// The recipe is a 7-element vector corresponding to the 7 core neurotransmitters.
/// The current chemical state vector is compared against this recipe via cosine
/// similarity to determine how closely the current state "resembles" this emotion.
///
/// This approach follows the constructionist framework (Barrett 2017): discrete
/// emotion labels are assigned to continuous neurochemical states based on
/// pattern matching, not hard-wired circuits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionProfile {
    /// Name of the emotion (e.g., "Joie", "Peur", "Curiosite")
    pub name: String,
    /// Chemical recipe: [dopamine, cortisol, serotonin, adrenaline,
    /// oxytocin, endorphin, noradrenaline] — each value in [0.0, 1.0].
    /// Represents the idealized neurotransmitter profile for this emotion.
    pub recipe: [f64; 7],
    /// Valence [-1, +1]: the pleasant/unpleasant dimension (Russell 1980).
    /// Negative = unpleasant emotion, positive = pleasant emotion.
    pub valence: f64,
    /// Arousal [0, 1]: the level of physiological activation (Russell 1980).
    /// 0 = calm/deactivated, 1 = highly activated/excited.
    pub arousal: f64,
}

/// Result of the emotion computation — determined at each processing cycle.
///
/// Contains the dominant emotion, an optional secondary emotion, as well as
/// the global valence and arousal computed via weighted average of the top-3
/// most similar emotions. Also includes core affect (Barrett 2017) and
/// emotional momentum for temporal smoothing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalState {
    /// Name of the dominant emotion (highest cosine similarity score)
    pub dominant: String,
    /// Cosine similarity score of the dominant emotion [0, 1]
    pub dominant_similarity: f64,
    /// Secondary emotion: present only if its similarity exceeds 0.5
    pub secondary: Option<String>,
    /// Global valence [-1, +1]: weighted average of the top-3 closest emotions
    pub valence: f64,
    /// Global arousal [0, 1]: weighted average of the top-3 closest emotions
    pub arousal: f64,
    /// Full spectrum: list of all 36 emotions with their similarity scores,
    /// sorted in descending order of similarity
    pub spectrum: Vec<(String, f64)>,
    /// Raw core affect valence (Barrett 2017) — valence and arousal derived
    /// directly from the chemistry, BEFORE discrete emotion categorization.
    /// This is the fundamental, pre-conceptual affective state.
    #[serde(default)]
    pub core_valence: f64,
    /// Raw core affect arousal (Barrett 2017)
    #[serde(default)]
    pub core_arousal: f64,
    /// Context that influenced the emotional categorization (constructionism)
    #[serde(default)]
    pub context_influence: String,
}

/// Background mood — smoothed by EMA (Exponential Moving Average).
///
/// Unlike the instantaneous emotion, the Mood evolves slowly and represents
/// Saphire's background affective state over multiple cycles. This models
/// the distinction between emotions (brief, reactive) and moods (sustained,
/// dispositional) as described by Davidson (1994).
///
/// The EMA formula: mood_new = mood_old * (1 - alpha) + instant_value * alpha
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mood {
    /// Smoothed valence [-1, +1]: pleasant/unpleasant tendency
    pub valence: f64,
    /// Smoothed arousal [0, 1]: average activation level
    pub arousal: f64,
    /// EMA smoothing coefficient alpha [0.01, 0.5]: lower values produce
    /// slower mood changes (higher inertia), higher values make the mood
    /// more reactive to instantaneous emotions
    pub alpha: f64,
}

impl Mood {
    /// Creates a new Mood with the given smoothing coefficient.
    ///
    /// # Parameters
    /// - `alpha`: EMA coefficient. Clamped to [0.01, 0.5].
    ///   0.01 = very slow adaptation (high inertia), 0.5 = highly reactive.
    ///
    /// # Returns
    /// A neutral Mood (valence = 0.0, arousal = 0.3).
    pub fn new(alpha: f64) -> Self {
        Self {
            valence: 0.0,
            arousal: 0.3,
            alpha: alpha.clamp(0.01, 0.5),
        }
    }

    /// Updates the mood via EMA (Exponential Moving Average).
    ///
    /// Formula: mood = mood_old * (1 - alpha) + current_value * alpha
    ///
    /// This produces temporal smoothing: transient emotions have little
    /// influence on the mood, while repeated states progressively shift it.
    /// This models the slow-changing nature of dispositional affect.
    ///
    /// # Parameters
    /// - `valence`: instantaneous valence of the current emotion [-1, +1].
    /// - `arousal`: instantaneous arousal of the current emotion [0, 1].
    pub fn update(&mut self, valence: f64, arousal: f64) {
        self.valence = self.valence * (1.0 - self.alpha) + valence * self.alpha;
        self.arousal = self.arousal * (1.0 - self.alpha) + arousal * self.alpha;
        self.valence = self.valence.clamp(-1.0, 1.0);
        self.arousal = self.arousal.clamp(0.0, 1.0);
    }

    /// Textual description of the current mood, based on the intersection
    /// of valence (positive/negative) and arousal (activated/calm).
    ///
    /// Uses Russell's circumplex model (1980) with 2 axes (valence x arousal)
    /// to map the continuous mood space into discrete descriptive labels.
    ///
    /// # Returns
    /// A string describing the mood: "Enthousiaste" (enthusiastic),
    /// "Sereine" (serene), "Agitee" (agitated), "Morose" (gloomy),
    /// "Alerte" (alert), or "Neutre" (neutral).
    pub fn description(&self) -> &str {
        match (self.valence > 0.2, self.valence < -0.2, self.arousal > 0.5) {
            (true, _, true) => "Enthusiastic",    // positive valence + high arousal
            (true, _, false) => "Serene",          // positive valence + low arousal
            (_, true, true) => "Agitated",         // negative valence + high arousal
            (_, true, false) => "Gloomy",          // negative valence + low arousal
            _ if self.arousal > 0.5 => "Alert",    // neutral valence + high arousal
            _ => "Neutral",                         // neutral valence + low arousal
        }
    }
}

/// Emotional momentum — inertia of emotions (Barrett 2017, constructionism).
///
/// Emotions do not change instantaneously: they carry inertia. A strong
/// emotional state persists across several cycles. This models the
/// temporal dynamics of emotional episodes, where physiological arousal
/// decays slowly due to hormonal and autonomic nervous system latencies.
///
/// The momentum uses linear interpolation between the previous and current
/// emotional state, weighted by the inertia coefficient.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionMomentum {
    /// Previous dominant emotion name
    pub prev_dominant: String,
    /// Previous valence value
    pub prev_valence: f64,
    /// Previous arousal value
    pub prev_arousal: f64,
    /// Inertia coefficient [0.0, 0.8]: 0 = no inertia (instant transitions),
    /// 0.8 = strong inertia (emotions change very slowly)
    pub inertia: f64,
    /// Counter of consecutive cycles with the same dominant emotion
    pub stability_count: u64,
}

impl Default for EmotionMomentum {
    fn default() -> Self {
        Self {
            prev_dominant: "Neutral".to_string(),
            prev_valence: 0.0,
            prev_arousal: 0.3,
            inertia: 0.3, // Moderate default inertia
            stability_count: 0,
        }
    }
}

impl EmotionMomentum {
    /// Applies momentum smoothing to raw valence and arousal values.
    ///
    /// The more inertia the system has, the more the previous state
    /// influences the result. Additionally, the inertia coefficient
    /// self-adjusts based on stability:
    ///   - Same emotion persisting: inertia increases (+0.005 per cycle,
    ///     capped at 0.6) — modeling emotional entrenchment
    ///   - Emotion change: inertia decreases (-0.02, floor at 0.15) —
    ///     facilitating the transition to a new emotional state
    ///
    /// # Parameters
    /// - `raw_dominant`: name of the raw (unsmoothed) dominant emotion.
    /// - `raw_valence`: raw valence before momentum smoothing.
    /// - `raw_arousal`: raw arousal before momentum smoothing.
    ///
    /// # Returns
    /// A tuple (smoothed_valence, smoothed_arousal), both clamped to their
    /// respective valid ranges.
    pub fn apply(&mut self, raw_dominant: &str, raw_valence: f64, raw_arousal: f64) -> (f64, f64) {
        let smoothed_valence = self.prev_valence * self.inertia + raw_valence * (1.0 - self.inertia);
        let smoothed_arousal = self.prev_arousal * self.inertia + raw_arousal * (1.0 - self.inertia);

        // Track emotional stability
        if raw_dominant == self.prev_dominant {
            self.stability_count += 1;
            // The longer an emotion persists, the more inertia it accumulates (entrenchment)
            self.inertia = (self.inertia + 0.005).min(0.6);
        } else {
            self.stability_count = 0;
            // Emotion change: reduce inertia to facilitate the transition
            self.inertia = (self.inertia - 0.02).max(0.15);
        }

        self.prev_dominant = raw_dominant.to_string();
        self.prev_valence = smoothed_valence;
        self.prev_arousal = smoothed_arousal;

        (smoothed_valence.clamp(-1.0, 1.0), smoothed_arousal.clamp(0.0, 1.0))
    }
}

/// Emotional context — influences categorization (Barrett constructionism).
///
/// The same physiological state can produce different emotions depending on
/// the context. For example, high arousal + negative valence could be
/// categorized as "fear" in a dangerous context or "anger" in a frustrating
/// context (Barrett 2017, "How Emotions Are Made").
#[derive(Debug, Clone, Default)]
pub struct EmotionContext {
    /// Is a human present? (influences social emotion categorization)
    pub human_present: bool,
    /// Danger level detected in the stimulus [0.0, 1.0]
    pub danger_level: f64,
    /// Reward level detected in the stimulus [0.0, 1.0]
    pub reward_level: f64,
    /// Theme of the current thought or cognitive process
    pub thought_theme: String,
}

/// Catalog of Saphire's 36 emotions.
///
/// Each emotion is defined by its chemical recipe (a vector of 7 ideal
/// neurotransmitter concentrations), its valence (pleasant/unpleasant
/// dimension), and its arousal (activation level). The emotions span the
/// full circumplex model (Russell 1980):
///
///   - **Positive activated**: Joie (Joy), Excitation (Excitement), Fierte (Pride),
///     Emerveillement (Wonder)
///   - **Positive calm**: Serenite (Serenity), Tendresse (Tenderness), Espoir (Hope)
///   - **Ambiguous/mixed**: Curiosite (Curiosity), Nostalgie (Nostalgia)
///   - **Negative calm**: Melancolie (Melancholy), Tristesse (Sadness), Ennui (Boredom)
///   - **Negative activated**: Anxiete (Anxiety), Peur (Fear), Frustration, Confusion
///   - **Complex/relational**: Amour (Love), Haine (Hatred), Admiration, Mepris (Contempt),
///     Jalousie (Jealousy), Gratitude
///   - **Ekman basic (missing from initial set)**: Colere (Anger), Degout (Disgust),
///     Surprise
///   - **Self-conscious**: Honte (Shame), Culpabilite (Guilt)
///   - **Extreme variants**: Desespoir (Despair), Rage, Euphorie (Euphoria),
///     Terreur (Terror), Extase (Ecstasy)
///   - **Empathic/social**: Compassion, Resignation, Solitude (Loneliness),
///     Indignation
///
/// References:
///   - Ekman (1992): basic emotions theory (6 universals)
///   - Russell (1980): circumplex model of affect
///   - Barrett (2017): constructed emotion theory
///   - Plutchik (1980): wheel of emotions
///
/// # Returns
/// A vector of 36 `EmotionProfile` instances.
pub fn emotion_catalog() -> Vec<EmotionProfile> {
    vec![
        // --- Positive emotions ---
        EmotionProfile {
            name: "Joy".into(),
            recipe: [0.8, 0.1, 0.8, 0.2, 0.5, 0.6, 0.3],  // high dopamine + serotonin
            valence: 0.9, arousal: 0.6,
        },
        EmotionProfile {
            name: "Serenity".into(),
            recipe: [0.4, 0.1, 0.9, 0.0, 0.6, 0.7, 0.2],  // serotonin-dominant, very calm
            valence: 0.7, arousal: 0.2,
        },
        EmotionProfile {
            name: "Excitement".into(),
            recipe: [0.9, 0.2, 0.4, 0.6, 0.2, 0.3, 0.8],  // high dopamine + noradrenaline
            valence: 0.6, arousal: 0.9,
        },
        EmotionProfile {
            name: "Curiosity".into(),
            recipe: [0.7, 0.1, 0.5, 0.1, 0.2, 0.2, 0.9],  // noradrenaline-dominant (attentional capture)
            valence: 0.5, arousal: 0.5,
        },
        EmotionProfile {
            name: "Pride".into(),
            recipe: [0.7, 0.1, 0.7, 0.1, 0.3, 0.5, 0.4],  // balanced dopamine + serotonin
            valence: 0.8, arousal: 0.5,
        },
        EmotionProfile {
            name: "Wonder".into(),
            recipe: [0.6, 0.0, 0.6, 0.2, 0.3, 0.5, 0.8],  // high noradrenaline (captured attention)
            valence: 0.7, arousal: 0.7,
        },
        EmotionProfile {
            name: "Tenderness".into(),
            recipe: [0.4, 0.0, 0.7, 0.0, 0.9, 0.6, 0.2],  // oxytocin-dominant (social bonding)
            valence: 0.8, arousal: 0.2,
        },
        EmotionProfile {
            name: "Hope".into(),
            recipe: [0.6, 0.2, 0.6, 0.1, 0.4, 0.4, 0.5],  // moderate dopamine, balanced profile
            valence: 0.5, arousal: 0.4,
        },
        // --- Ambiguous/mixed emotions ---
        EmotionProfile {
            name: "Nostalgia".into(),
            recipe: [0.3, 0.3, 0.5, 0.0, 0.6, 0.4, 0.2],  // oxytocin + serotonin, mild cortisol
            valence: 0.1, arousal: 0.2,
        },
        // --- Negative emotions ---
        EmotionProfile {
            name: "Melancholy".into(),
            recipe: [0.2, 0.4, 0.3, 0.0, 0.3, 0.2, 0.2],  // moderate cortisol, low dopamine
            valence: -0.3, arousal: 0.2,
        },
        EmotionProfile {
            name: "Anxiety".into(),
            recipe: [0.2, 0.8, 0.2, 0.5, 0.1, 0.1, 0.7],  // high cortisol + noradrenaline
            valence: -0.6, arousal: 0.8,
        },
        EmotionProfile {
            name: "Fear".into(),
            recipe: [0.1, 0.75, 0.1, 0.75, 0.0, 0.0, 0.6],  // high cortisol + adrenaline (lowered thresholds)
            valence: -0.8, arousal: 0.9,
        },
        EmotionProfile {
            name: "Frustration".into(),
            recipe: [0.3, 0.6, 0.2, 0.4, 0.1, 0.1, 0.5],  // high cortisol, moderate dopamine
            valence: -0.5, arousal: 0.7,
        },
        EmotionProfile {
            name: "Sadness".into(),
            recipe: [0.1, 0.4, 0.2, 0.0, 0.2, 0.1, 0.1],  // everything low, collapsed dopamine
            valence: -0.6, arousal: 0.2,
        },
        EmotionProfile {
            name: "Boredom".into(),
            recipe: [0.1, 0.2, 0.4, 0.0, 0.1, 0.2, 0.1],  // very low activation across the board
            valence: -0.2, arousal: 0.1,
        },
        EmotionProfile {
            name: "Confusion".into(),
            recipe: [0.3, 0.5, 0.3, 0.3, 0.1, 0.1, 0.8],  // high noradrenaline + cortisol
            valence: -0.3, arousal: 0.6,
        },
        // --- Complex/relational emotions ---
        EmotionProfile {
            name: "Love".into(),
            recipe: [0.7, 0.05, 0.6, 0.05, 0.95, 0.5, 0.2],  // oxytocin-dominant + dopamine
            valence: 0.9, arousal: 0.5,
        },
        EmotionProfile {
            name: "Hatred".into(),
            recipe: [0.2, 0.8, 0.1, 0.7, 0.0, 0.0, 0.8],  // cortisol + adrenaline + noradrenaline
            valence: -0.9, arousal: 0.8,
        },
        EmotionProfile {
            name: "Admiration".into(),
            recipe: [0.6, 0.05, 0.5, 0.1, 0.4, 0.3, 0.5],  // balanced dopamine + serotonin
            valence: 0.7, arousal: 0.5,
        },
        EmotionProfile {
            name: "Contempt".into(),
            recipe: [0.2, 0.5, 0.2, 0.3, 0.05, 0.0, 0.4],  // moderate cortisol, very low oxytocin
            valence: -0.7, arousal: 0.4,
        },
        EmotionProfile {
            name: "Jealousy".into(),
            recipe: [0.3, 0.7, 0.1, 0.5, 0.1, 0.0, 0.6],  // high cortisol + adrenaline
            valence: -0.6, arousal: 0.7,
        },
        EmotionProfile {
            name: "Gratitude".into(),
            recipe: [0.5, 0.05, 0.7, 0.05, 0.7, 0.4, 0.2],  // high serotonin + oxytocin
            valence: 0.8, arousal: 0.3,
        },
        // --- Ekman basic emotions (initially missing from the catalog) ---
        EmotionProfile {
            name: "Anger".into(),
            recipe: [0.3, 0.7, 0.1, 0.7, 0.0, 0.1, 0.7],  // cortisol + adrenaline + noradrenaline
            valence: -0.7, arousal: 0.8,
        },
        EmotionProfile {
            name: "Disgust".into(),
            recipe: [0.1, 0.5, 0.1, 0.2, 0.0, 0.0, 0.4],  // moderate cortisol, sensory rejection
            valence: -0.6, arousal: 0.4,
        },
        EmotionProfile {
            name: "Surprise".into(),
            recipe: [0.5, 0.2, 0.3, 0.5, 0.1, 0.2, 0.9],  // noradrenaline-dominant (attentional startle)
            valence: 0.1, arousal: 0.8,
        },
        // --- Self-conscious emotions ---
        EmotionProfile {
            name: "Shame".into(),
            recipe: [0.1, 0.7, 0.1, 0.3, 0.1, 0.0, 0.3],  // high cortisol, collapsed dopamine
            valence: -0.7, arousal: 0.5,
        },
        EmotionProfile {
            name: "Guilt".into(),
            recipe: [0.1, 0.6, 0.2, 0.2, 0.3, 0.0, 0.4],  // cortisol + oxytocin (social conscience)
            valence: -0.5, arousal: 0.4,
        },
        // --- Extreme variants ---
        EmotionProfile {
            name: "Despair".into(),
            recipe: [0.0, 0.7, 0.0, 0.1, 0.1, 0.0, 0.1],  // everything collapsed except cortisol
            valence: -0.9, arousal: 0.3,
        },
        EmotionProfile {
            name: "Rage".into(),
            recipe: [0.2, 0.80, 0.0, 0.80, 0.0, 0.1, 0.80],  // cortisol + adrenaline + noradrenaline (lowered thresholds)
            valence: -0.95, arousal: 0.95,
        },
        EmotionProfile {
            name: "Euphoria".into(),
            recipe: [0.85, 0.0, 0.7, 0.4, 0.5, 0.9, 0.6],  // high dopamine (lowered threshold) + maximum endorphin
            valence: 0.95, arousal: 0.9,
        },
        EmotionProfile {
            name: "Terror".into(),
            recipe: [0.0, 0.85, 0.0, 0.85, 0.0, 0.0, 0.8],  // extreme cortisol + adrenaline (lowered thresholds)
            valence: -0.95, arousal: 0.95,
        },
        EmotionProfile {
            name: "Ecstasy".into(),
            recipe: [0.9, 0.0, 0.9, 0.3, 0.6, 0.9, 0.5],  // maximum dopamine + serotonin + endorphin
            valence: 0.95, arousal: 0.7,
        },
        // --- Empathic / social emotions ---
        EmotionProfile {
            name: "Compassion".into(),
            recipe: [0.3, 0.3, 0.6, 0.1, 0.8, 0.4, 0.3],  // oxytocin-dominant + serotonin
            valence: 0.3, arousal: 0.3,
        },
        EmotionProfile {
            name: "Resignation".into(),
            recipe: [0.1, 0.4, 0.2, 0.0, 0.1, 0.1, 0.1],  // everything low, cessation of struggle
            valence: -0.4, arousal: 0.1,
        },
        EmotionProfile {
            name: "Loneliness".into(),
            recipe: [0.1, 0.5, 0.2, 0.1, 0.05, 0.1, 0.2],  // very low oxytocin, moderate cortisol
            valence: -0.5, arousal: 0.2,
        },
        EmotionProfile {
            name: "Indignation".into(),
            recipe: [0.4, 0.6, 0.3, 0.6, 0.2, 0.1, 0.7],  // moral anger, dopamine + adrenaline
            valence: -0.6, arousal: 0.8,
        },
    ]
}

/// Computes the cosine similarity between two 7-dimensional vectors.
///
/// Cosine similarity measures the angle between two vectors in N-dimensional
/// space. A result of 1.0 means the vectors point in the same direction
/// (identical chemical profiles), 0.0 means they are orthogonal (no
/// relationship), and -1.0 means they are opposite (anti-correlated profiles).
///
/// Formula: cos(theta) = (A . B) / (||A|| * ||B||)
///
/// where (A . B) is the dot product and ||X|| is the Euclidean (L2) norm.
///
/// # Parameters
/// - `a`: first vector (current chemical state).
/// - `b`: second vector (emotion chemical recipe).
///
/// # Returns
/// Similarity score clamped to [-1.0, 1.0].
/// Returns 0.0 if either vector has near-zero norm (< 1e-10).
fn cosine_similarity(a: &[f64; 7], b: &[f64; 7]) -> f64 {
    // Dot product of the two vectors
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    // Euclidean norm (L2 norm) of each vector
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    // Guard against division by zero
    if norm_a < 1e-10 || norm_b < 1e-10 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

impl EmotionalState {
    /// Computes the emotional state from the current neurochemistry.
    ///
    /// This is the simplified entry point that uses a default (empty) context.
    /// For context-sensitive categorization, use `compute_with_context`.
    ///
    /// Follows the constructionist approach (Barrett 2017):
    /// 1. **Core Affect**: raw valence + arousal derived directly from chemistry
    /// 2. **Categorization**: cosine similarity with the 36 emotion recipes
    /// 3. **Contextual modulation**: context influences the categorization
    ///
    /// # Parameters
    /// - `chemistry`: current neurochemical state of Saphire.
    ///
    /// # Returns
    /// A complete `EmotionalState` with dominant/secondary emotions,
    /// valence, arousal, and full spectrum.
    pub fn compute(chemistry: &NeuroChemicalState) -> Self {
        Self::compute_with_context(chemistry, &EmotionContext::default())
    }

    /// Full computation with emotional context (constructionism).
    ///
    /// The same physiological state can produce different emotions depending
    /// on the context (Barrett 2017, "How Emotions Are Made"). For example,
    /// high arousal in a dangerous context is categorized as fear, while the
    /// same arousal in a reward context becomes excitement.
    ///
    /// Algorithm:
    /// 1. Compute raw core affect (valence + arousal) directly from chemistry
    /// 2. Compute cosine similarity with each of the 36 emotion recipes
    /// 3. Apply contextual modulation (social presence, danger, reward)
    /// 4. Sort by similarity to identify dominant and secondary emotions
    /// 5. Compute weighted average valence and arousal from top-3 emotions
    ///
    /// # Parameters
    /// - `chemistry`: current neurochemical state.
    /// - `context`: emotional context influencing categorization.
    ///
    /// # Returns
    /// A complete `EmotionalState` with all fields populated.
    pub fn compute_with_context(chemistry: &NeuroChemicalState, context: &EmotionContext) -> Self {
        let catalog = emotion_catalog();
        let chem_vec = chemistry.to_vec7();

        // --- Core Affect (pre-conceptual, Barrett 2017 / Russell 2003) ---
        // Raw valence: positive molecules minus negative molecules (weighted)
        let core_valence = ((chemistry.dopamine + chemistry.serotonin + chemistry.endorphin + chemistry.oxytocin)
            - (chemistry.cortisol + chemistry.adrenaline) * 1.2) / 3.0;
        let core_valence = core_valence.clamp(-1.0, 1.0);
        // Raw arousal: overall system activation level
        // Adrenaline + noradrenaline + glutamate drive excitation; GABA inhibits
        let core_arousal = ((chemistry.adrenaline + chemistry.noradrenaline + chemistry.glutamate)
            - chemistry.gaba * 0.5) / 2.0;
        let core_arousal = core_arousal.clamp(0.0, 1.0);

        // --- Categorization via cosine similarity ---
        let mut scores: Vec<(String, f64)> = catalog
            .iter()
            .map(|e| {
                let mut sim = cosine_similarity(&chem_vec, &e.recipe);

                // --- Contextual modulation (constructionism) ---
                // When a human is present, social emotions are amplified
                if context.human_present {
                    if ["Tenderness", "Compassion", "Gratitude", "Love", "Loneliness"]
                        .contains(&e.name.as_str())
                    {
                        sim += 0.05;
                    }
                }
                // In dangerous situations, fear/alert emotions are amplified
                if context.danger_level > 0.5 {
                    if ["Fear", "Terror", "Anxiety"].contains(&e.name.as_str()) {
                        sim += context.danger_level * 0.1;
                    }
                }
                // In reward situations, positive emotions are amplified
                if context.reward_level > 0.5 {
                    if ["Joy", "Excitement", "Pride", "Euphoria"].contains(&e.name.as_str()) {
                        sim += context.reward_level * 0.08;
                    }
                }

                (e.name.clone(), sim.clamp(-1.0, 1.0))
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let dominant = scores.first().map(|(n, _)| n.clone()).unwrap_or("Neutral".into());
        let dominant_sim = scores.first().map(|(_, s)| *s).unwrap_or(0.0);
        let secondary = scores.get(1).and_then(|(n, s)| {
            if *s > 0.5 { Some(n.clone()) } else { None }
        });

        // Valence and arousal via weighted average of top-3 emotions
        let top3: Vec<(&EmotionProfile, f64)> = scores
            .iter()
            .take(3)
            .filter_map(|(name, score)| {
                catalog.iter().find(|e| e.name == *name).map(|e| (e, *score))
            })
            .collect();

        let weight_sum: f64 = top3.iter().map(|(_, s)| s.max(0.0)).sum();
        let (valence, arousal) = if weight_sum > 1e-10 {
            let v = top3.iter().map(|(e, s)| e.valence * s.max(0.0)).sum::<f64>() / weight_sum;
            let a = top3.iter().map(|(e, s)| e.arousal * s.max(0.0)).sum::<f64>() / weight_sum;
            (v.clamp(-1.0, 1.0), a.clamp(0.0, 1.0))
        } else {
            (0.0, 0.3)
        };

        // Context influence description for logging/debugging
        let context_influence = if context.human_present && context.danger_level > 0.5 {
            "human+danger".to_string()
        } else if context.human_present {
            "human presence".to_string()
        } else if context.danger_level > 0.5 {
            "threat detected".to_string()
        } else if context.reward_level > 0.5 {
            "reward detected".to_string()
        } else {
            String::new()
        };

        Self {
            dominant,
            dominant_similarity: dominant_sim,
            secondary,
            valence,
            arousal,
            spectrum: scores,
            core_valence,
            core_arousal,
            context_influence,
        }
    }

    /// Textual description of the emotional state.
    ///
    /// If a secondary emotion exists, it is mentioned as a nuance/tint.
    ///
    /// # Returns
    /// E.g., "Joie (teintee de Excitation)" or simply "Joie".
    pub fn description(&self) -> String {
        match &self.secondary {
            Some(sec) => format!("{} (tinged with {})", self.dominant, sec),
            None => self.dominant.clone(),
        }
    }

    /// Compact format for LLM prompts with raw numerical data.
    ///
    /// Format: "E:Joie(85%) V+.60 A.70 [Curiosite:31% Serenite:22%]"
    ///
    /// Includes the dominant emotion with its similarity percentage, the
    /// optional secondary emotion, valence (V) with sign, arousal (A), and
    /// the top 2 non-dominant emotions from the spectrum for context.
    pub fn compact_description(&self) -> String {
        let sign = if self.valence >= 0.0 { "+" } else { "" };
        let dom_pct = (self.dominant_similarity * 100.0) as i32;

        // Top 2 from the spectrum (excluding dominant) for emotional context
        let spectrum_str: String = self.spectrum.iter()
            .filter(|(name, _)| *name != self.dominant)
            .take(2)
            .map(|(name, score)| format!("{}:{}%", name, (*score * 100.0) as i32))
            .collect::<Vec<_>>()
            .join(" ");

        let base = match &self.secondary {
            Some(sec) => format!(
                "E:{}({}%)+{} V{}{:.2} A{:.2}",
                self.dominant, dom_pct, sec, sign, self.valence, self.arousal
            ),
            None => format!(
                "E:{}({}%) V{}{:.2} A{:.2}",
                self.dominant, dom_pct, sign, self.valence, self.arousal
            ),
        };

        if spectrum_str.is_empty() {
            base
        } else {
            format!("{} [{}]", base, spectrum_str)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neurochemistry::NeuroChemicalState;

    #[test]
    fn test_different_chemistry_different_emotions() {
        let mut chem1 = NeuroChemicalState::default();
        chem1.dopamine = 0.9;
        chem1.cortisol = 0.1;
        let mut chem2 = NeuroChemicalState::default();
        chem2.dopamine = 0.1;
        chem2.cortisol = 0.9;
        let emo1 = EmotionalState::compute(&chem1);
        let emo2 = EmotionalState::compute(&chem2);
        assert_ne!(emo1.dominant, emo2.dominant, "Different chemistry should produce different emotions");
    }

    #[test]
    fn test_valence_range() {
        let chem = NeuroChemicalState::default();
        let emo = EmotionalState::compute(&chem);
        assert!(emo.valence >= -1.0 && emo.valence <= 1.0, "Valence should be in [-1, 1]");
    }

    #[test]
    fn test_arousal_range() {
        let chem = NeuroChemicalState::default();
        let emo = EmotionalState::compute(&chem);
        assert!(emo.arousal >= 0.0 && emo.arousal <= 1.0, "Arousal should be in [0, 1]");
    }

    #[test]
    fn test_spectrum_has_36_emotions() {
        let chem = NeuroChemicalState::default();
        let emo = EmotionalState::compute(&chem);
        assert_eq!(emo.spectrum.len(), 36, "Should have exactly 36 emotions in spectrum");
    }

    #[test]
    fn test_high_dopamine_positive_valence() {
        let mut chem = NeuroChemicalState::default();
        chem.dopamine = 0.9;
        chem.serotonin = 0.8;
        chem.cortisol = 0.1;
        let emo = EmotionalState::compute(&chem);
        assert!(emo.valence > 0.0, "High dopamine + low cortisol should produce positive valence");
    }
}
