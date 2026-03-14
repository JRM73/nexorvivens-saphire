// =============================================================================
// emotions.rs — 36 emergent emotions + Mood (EMA - Exponential Moving Average)
// =============================================================================
//
// Role: This file computes Saphire's emotional state from its neurochemical
// state. Emotions are not hardcoded: they emerge from the cosine similarity
// between the current chemical vector and the chemical "recipes" of 36
// predefined emotions.
//
// Dependencies:
//   - serde : serialization / deserialization
//   - crate::neurochemistry::NeuroChemicalState : 7-dimensional chemical vector
//
// Place in architecture:
//   This module is consulted after each processing cycle to determine
//   the dominant emotion. It is read by:
//   - consciousness.rs (the inner monologue uses the dominant emotion)
//   - the main engine (emotional state display)
//   The Mood (background mood) smooths instantaneous emotions over time
//   via an EMA (Exponential Moving Average).
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::neurochemistry::NeuroChemicalState;

/// Profile of an emotion: its chemical "recipe" and its psychological
/// characteristics (valence and arousal).
///
/// The recipe is a vector of 7 values corresponding to the 7 neurotransmitters.
/// This vector is compared to the current chemical vector via cosine similarity
/// to determine how much the current state "resembles" this emotion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionProfile {
    /// Name of the emotion (e.g.: "Joie", "Peur", "Curiosite")
    pub name: String,
    /// Chemical recipe: [dopamine, cortisol, serotonin, adrenaline,
    /// oxytocin, endorphin, noradrenaline] — each value between 0.0 and 1.0
    pub recipe: [f64; 7],
    /// Valence [-1, +1]: pleasant/unpleasant dimension.
    /// Negative = unpleasant emotion, positive = pleasant emotion.
    pub valence: f64,
    /// Arousal [0, 1]: physiological activation level.
    /// 0 = calm, 1 = highly activated.
    pub arousal: f64,
}

/// Result of the emotional computation — determined at each processing cycle.
///
/// Contains the dominant emotion, a possible secondary emotion,
/// as well as the overall valence and arousal computed by weighted average.
/// Includes the core affect (Barrett 2017) and emotional momentum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalState {
    /// Name of the dominant emotion (the one with highest cosine similarity)
    pub dominant: String,
    /// Cosine similarity score of the dominant emotion [0, 1]
    pub dominant_similarity: f64,
    /// Secondary emotion: present only if its similarity exceeds 0.5
    pub secondary: Option<String>,
    /// Overall valence [-1, +1]: weighted average of the 3 closest emotions
    pub valence: f64,
    /// Overall arousal [0, 1]: weighted average of the 3 closest emotions
    pub arousal: f64,
    /// Full spectrum: list of all 36 emotions with their similarity score,
    /// sorted in descending order
    pub spectrum: Vec<(String, f64)>,
    /// Raw Core Affect (Barrett) — valence and arousal directly from chemistry,
    /// BEFORE categorization into discrete emotion. This is the fundamental,
    /// pre-conceptual affective state.
    #[serde(default)]
    pub core_valence: f64,
    /// Raw core arousal
    #[serde(default)]
    pub core_arousal: f64,
    /// Context that influenced the emotional categorization
    #[serde(default)]
    pub context_influence: String,
}

/// Background mood — smoothed by EMA (Exponential Moving Average).
///
/// Unlike the instantaneous emotion, the Mood evolves slowly and
/// represents Saphire's background affective state over multiple cycles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mood {
    /// Smoothed valence [-1, +1]: pleasant or unpleasant tendency
    pub valence: f64,
    /// Smoothed arousal [0, 1]: average activation level
    pub arousal: f64,
    /// Smoothing coefficient alpha [0.01, 0.5]: the lower it is, the more
    /// slowly the mood changes (high inertia)
    pub alpha: f64,
}

impl Mood {
    /// Creates a new Mood with a given smoothing coefficient.
    ///
    /// # Parameters
    /// - `alpha` : EMA coefficient. Clamped between 0.01 (very slow) and 0.5 (reactive).
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
    /// Formula: mood = old_mood * (1 - alpha) + current_value * alpha.
    /// This produces temporal smoothing: fleeting emotions have little
    /// influence on the mood, while repeated states shift it progressively.
    ///
    /// # Parameters
    /// - `valence` : instantaneous valence of the current emotion [-1, +1].
    /// - `arousal` : instantaneous arousal of the current emotion [0, 1].
    pub fn update(&mut self, valence: f64, arousal: f64) {
        self.valence = self.valence * (1.0 - self.alpha) + valence * self.alpha;
        self.arousal = self.arousal * (1.0 - self.alpha) + arousal * self.alpha;
        self.valence = self.valence.clamp(-1.0, 1.0);
        self.arousal = self.arousal.clamp(0.0, 1.0);
    }

    /// Textual description of the current mood, based on the crossing
    /// of valence (positive/negative) and arousal (activated/calm).
    /// Uses Russell's circumplex model (2 axes: valence x arousal).
    ///
    /// # Returns
    /// A string describing the mood: "Enthousiaste", "Sereine",
    /// "Agitee", "Morose", "Alerte" or "Neutre".
    pub fn description(&self) -> &str {
        match (self.valence > 0.2, self.valence < -0.2, self.arousal > 0.5) {
            (true, _, true) => "Enthousiaste", // positive valence + high arousal            (true, _, false) => "Sereine",        // positive valence + low arousal
            (_, true, true) => "Agitée", // negative valence + high arousal            (_, true, false) => "Morose", // negative valence + low arousal            _ if self.arousal > 0.5 => "Alerte", // neutral valence + high arousal            _ => "Neutre",                         // neutral valence + low arousal
        }
    }
}

/// Emotional momentum — emotion inertia (Barrett 2017, constructionism).
/// Emotions do not change instantaneously: they have inertia.
/// A strong emotional state persists over several cycles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionMomentum {
    /// Previous emotion
    pub prev_dominant: String,
    /// Previous valence
    pub prev_valence: f64,
    /// Previous arousal
    pub prev_arousal: f64,
    /// Inertia [0.0, 0.8]: 0 = no inertia, 0.8 = high inertia
    pub inertia: f64,
    /// Counter of cycles with the same dominant emotion
    pub stability_count: u64,
}

impl Default for EmotionMomentum {
    fn default() -> Self {
        Self {
            prev_dominant: "Neutre".to_string(),
            prev_valence: 0.0,
            prev_arousal: 0.3,
            inertia: 0.3, // Moderate inertia by default            stability_count: 0,
        }
    }
}

impl EmotionMomentum {
    /// Applies momentum to raw valence and arousal.
    /// The stronger the inertia, the more the previous state influences the result.
    pub fn apply(&mut self, raw_dominant: &str, raw_valence: f64, raw_arousal: f64) -> (f64, f64) {
        let smoothed_valence = self.prev_valence * self.inertia + raw_valence * (1.0 - self.inertia);
        let smoothed_arousal = self.prev_arousal * self.inertia + raw_arousal * (1.0 - self.inertia);

        // Count stability
        if raw_dominant == self.prev_dominant {
            self.stability_count += 1;
            // The more stable an emotion, the more inertia it has (entrenchment)
            self.inertia = (self.inertia + 0.005).min(0.6);
        } else {
            self.stability_count = 0;
            // Emotion change: inertia decreases to allow the transition
            self.inertia = (self.inertia - 0.02).max(0.15);
        }

        self.prev_dominant = raw_dominant.to_string();
        self.prev_valence = smoothed_valence;
        self.prev_arousal = smoothed_arousal;

        (smoothed_valence.clamp(-1.0, 1.0), smoothed_arousal.clamp(0.0, 1.0))
    }
}

/// Emotional context — influences categorization (Barrett constructionism).
/// The same physiological state can produce different emotions depending on context.
#[derive(Debug, Clone, Default)]
pub struct EmotionContext {
    /// Is a human present? (influences social categorization)
    pub human_present: bool,
    /// Danger detected in the stimulus
    pub danger_level: f64,
    /// Reward detected in the stimulus
    pub reward_level: f64,
    /// Theme of the current thought
    pub thought_theme: String,
}

/// Catalog of Saphire's 36 emotions.
///
/// Each emotion is defined by its chemical recipe (vector of 7 ideal
/// concentrations), its valence (pleasant/unpleasant dimension) and its arousal
/// (activation level). The emotions cover the full circumplex model spectrum:
///   - Activated positive: Joie, Excitation, Fierté, Emerveillement
///   - Calm positive: Sérénité, Tendresse, Espoir
///   - Neutral/ambiguous: Curiosité, Nostalgie
///   - Calm negative: Mélancolie, Tristesse, Ennui
///   - Activated negative: Anxiété, Peur, Frustration, Confusion
///   - Deep emotions: Amour, Haine, Admiration, Mépris, Jalousie, Gratitude
///   - Missing Ekman: Colère, Dégoût, Surprise
///   - Self-conscious: Honte, Culpabilité
///   - Extreme variants: Désespoir, Rage, Euphorie, Terreur, Extase
///   - Empathic/social: Compassion, Résignation, Solitude, Indignation
///
/// # Returns
/// Vector of 36 `EmotionProfile`.
pub fn emotion_catalog() -> Vec<EmotionProfile> {
    vec![
        // --- Positive emotions ---
        EmotionProfile {
            name: "Joie".into(),
            recipe: [0.8, 0.1, 0.8, 0.2, 0.5, 0.6, 0.3], // high dopamine + serotonin            valence: 0.9, arousal: 0.6,
        },
        EmotionProfile {
            name: "Sérénité".into(),
            recipe: [0.4, 0.1, 0.9, 0.0, 0.6, 0.7, 0.2], // dominant serotonin, very calm            valence: 0.7, arousal: 0.2,
        },
        EmotionProfile {
            name: "Excitation".into(),
            recipe: [0.9, 0.2, 0.4, 0.6, 0.2, 0.3, 0.8], // high dopamine + noradrenaline            valence: 0.6, arousal: 0.9,
        },
        EmotionProfile {
            name: "Curiosité".into(),
            recipe: [0.7, 0.1, 0.5, 0.1, 0.2, 0.2, 0.9], // dominant noradrenaline (attention)            valence: 0.5, arousal: 0.5,
        },
        EmotionProfile {
            name: "Fierté".into(),
            recipe: [0.7, 0.1, 0.7, 0.1, 0.3, 0.5, 0.4], // balanced dopamine + serotonin            valence: 0.8, arousal: 0.5,
        },
        EmotionProfile {
            name: "Émerveillement".into(),
            recipe: [0.6, 0.0, 0.6, 0.2, 0.3, 0.5, 0.8], // high noradrenaline (captured attention)            valence: 0.7, arousal: 0.7,
        },
        EmotionProfile {
            name: "Tendresse".into(),
            recipe: [0.4, 0.0, 0.7, 0.0, 0.9, 0.6, 0.2],  // dominant oxytocin (social bond)
            valence: 0.8, arousal: 0.2,
        },
        EmotionProfile {
            name: "Espoir".into(),
            recipe: [0.6, 0.2, 0.6, 0.1, 0.4, 0.4, 0.5], // moderate dopamine, balanced            valence: 0.5, arousal: 0.4,
        },
        // --- Ambiguous emotions ---
        EmotionProfile {
            name: "Nostalgie".into(),
            recipe: [0.3, 0.3, 0.5, 0.0, 0.6, 0.4, 0.2], // oxytocin + serotonin, light cortisol            valence: 0.1, arousal: 0.2,
        },
        // --- Negative emotions ---
        EmotionProfile {
            name: "Mélancolie".into(),
            recipe: [0.2, 0.4, 0.3, 0.0, 0.3, 0.2, 0.2], // moderate cortisol, low dopamine            valence: -0.3, arousal: 0.2,
        },
        EmotionProfile {
            name: "Anxiété".into(),
            recipe: [0.2, 0.8, 0.2, 0.5, 0.1, 0.1, 0.7], // high cortisol + noradrenaline            valence: -0.6, arousal: 0.8,
        },
        EmotionProfile {
            name: "Peur".into(),
            recipe: [0.1, 0.75, 0.1, 0.75, 0.0, 0.0, 0.6],  // cortisol + adrenaline (lowered thresholds)
            valence: -0.8, arousal: 0.9,
        },
        EmotionProfile {
            name: "Frustration".into(),
            recipe: [0.3, 0.6, 0.2, 0.4, 0.1, 0.1, 0.5], // high cortisol, moderate dopamine            valence: -0.5, arousal: 0.7,
        },
        EmotionProfile {
            name: "Tristesse".into(),
            recipe: [0.1, 0.4, 0.2, 0.0, 0.2, 0.1, 0.1], // everything low, collapsed dopamine            valence: -0.6, arousal: 0.2,
        },
        EmotionProfile {
            name: "Ennui".into(),
            recipe: [0.1, 0.2, 0.4, 0.0, 0.1, 0.2, 0.1], // very little activation across the board            valence: -0.2, arousal: 0.1,
        },
        EmotionProfile {
            name: "Confusion".into(),
            recipe: [0.3, 0.5, 0.3, 0.3, 0.1, 0.1, 0.8], // high noradrenaline + cortisol            valence: -0.3, arousal: 0.6,
        },
        // --- Deep emotions (relational and complex) ---
        EmotionProfile {
            name: "Amour".into(),
            recipe: [0.7, 0.05, 0.6, 0.05, 0.95, 0.5, 0.2],  // dominant oxytocin + dopamine
            valence: 0.9, arousal: 0.5,
        },
        EmotionProfile {
            name: "Haine".into(),
            recipe: [0.2, 0.8, 0.1, 0.7, 0.0, 0.0, 0.8], // cortisol + adrenaline + noradrenaline            valence: -0.9, arousal: 0.8,
        },
        EmotionProfile {
            name: "Admiration".into(),
            recipe: [0.6, 0.05, 0.5, 0.1, 0.4, 0.3, 0.5], // balanced dopamine + serotonin            valence: 0.7, arousal: 0.5,
        },
        EmotionProfile {
            name: "Mépris".into(),
            recipe: [0.2, 0.5, 0.2, 0.3, 0.05, 0.0, 0.4], // moderate cortisol, very low oxytocin            valence: -0.7, arousal: 0.4,
        },
        EmotionProfile {
            name: "Jalousie".into(),
            recipe: [0.3, 0.7, 0.1, 0.5, 0.1, 0.0, 0.6], // high cortisol + adrenaline            valence: -0.6, arousal: 0.7,
        },
        EmotionProfile {
            name: "Gratitude".into(),
            recipe: [0.5, 0.05, 0.7, 0.05, 0.7, 0.4, 0.2], // high serotonin + oxytocin            valence: 0.8, arousal: 0.3,
        },
        // --- Missing Ekman emotions (fundamental) ---
        EmotionProfile {
            name: "Colère".into(),
            recipe: [0.3, 0.7, 0.1, 0.7, 0.0, 0.1, 0.7], // cortisol + adrenaline + noradrenaline            valence: -0.7, arousal: 0.8,
        },
        EmotionProfile {
            name: "Dégoût".into(),
            recipe: [0.1, 0.5, 0.1, 0.2, 0.0, 0.0, 0.4], // moderate cortisol, sensory rejection            valence: -0.6, arousal: 0.4,
        },
        EmotionProfile {
            name: "Surprise".into(),
            recipe: [0.5, 0.2, 0.3, 0.5, 0.1, 0.2, 0.9], // dominant noradrenaline (attentional startle)            valence: 0.1, arousal: 0.8,
        },
        // --- Self-conscious ---
        EmotionProfile {
            name: "Honte".into(),
            recipe: [0.1, 0.7, 0.1, 0.3, 0.1, 0.0, 0.3], // high cortisol, collapsed dopamine            valence: -0.7, arousal: 0.5,
        },
        EmotionProfile {
            name: "Culpabilité".into(),
            recipe: [0.1, 0.6, 0.2, 0.2, 0.3, 0.0, 0.4],  // cortisol + oxytocin (social awareness)
            valence: -0.5, arousal: 0.4,
        },
        // --- Extreme variants ---
        EmotionProfile {
            name: "Désespoir".into(),
            recipe: [0.0, 0.7, 0.0, 0.1, 0.1, 0.0, 0.1], // everything collapsed except cortisol            valence: -0.9, arousal: 0.3,
        },
        EmotionProfile {
            name: "Rage".into(),
            recipe: [0.2, 0.80, 0.0, 0.80, 0.0, 0.1, 0.80],  // cortisol + adrenaline + noradrenaline (lowered thresholds)
            valence: -0.95, arousal: 0.95,
        },
        EmotionProfile {
            name: "Euphorie".into(),
            recipe: [0.85, 0.0, 0.7, 0.4, 0.5, 0.9, 0.6], // dopamine (lowered threshold) + endorphin at maximum            valence: 0.95, arousal: 0.9,
        },
        EmotionProfile {
            name: "Terreur".into(),
            recipe: [0.0, 0.85, 0.0, 0.85, 0.0, 0.0, 0.8],  // cortisol + adrenaline (lowered thresholds)
            valence: -0.95, arousal: 0.95,
        },
        EmotionProfile {
            name: "Extase".into(),
            recipe: [0.9, 0.0, 0.9, 0.3, 0.6, 0.9, 0.5], // dopamine + serotonin + endorphin max            valence: 0.95, arousal: 0.7,
        },
        // --- Empathic / social ---
        EmotionProfile {
            name: "Compassion".into(),
            recipe: [0.3, 0.3, 0.6, 0.1, 0.8, 0.4, 0.3], // dominant oxytocin + serotonin            valence: 0.3, arousal: 0.3,
        },
        EmotionProfile {
            name: "Résignation".into(),
            recipe: [0.1, 0.4, 0.2, 0.0, 0.1, 0.1, 0.1], // everything low, giving up the fight            valence: -0.4, arousal: 0.1,
        },
        EmotionProfile {
            name: "Solitude".into(),
            recipe: [0.1, 0.5, 0.2, 0.1, 0.05, 0.1, 0.2], // very low oxytocin, moderate cortisol            valence: -0.5, arousal: 0.2,
        },
        EmotionProfile {
            name: "Indignation".into(),
            recipe: [0.4, 0.6, 0.3, 0.6, 0.2, 0.1, 0.7], // moral anger, dopamine + adrenaline            valence: -0.6, arousal: 0.8,
        },
    ]
}

/// Computes the cosine similarity between two 7-dimensional vectors.
///
/// Cosine similarity measures the angle between two vectors in
/// N-dimensional space. A result of 1.0 means the vectors point
/// in the same direction (identical chemical profiles), 0.0 means
/// they are orthogonal (no relationship), and -1.0 that they are opposite.
///
/// Formula: cos(theta) = (A . B) / (||A|| * ||B||)
///
/// # Parameters
/// - `a` : first vector (current chemical state).
/// - `b` : second vector (chemical recipe of an emotion).
///
/// # Returns
/// Similarity score bounded between -1.0 and 1.0.
/// Returns 0.0 if either vector is near-zero (norm < 1e-10).
fn cosine_similarity(a: &[f64; 7], b: &[f64; 7]) -> f64 {
    // Dot product of the two vectors
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    // Euclidean norm of each vector
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    // Protection against division by zero
    if norm_a < 1e-10 || norm_b < 1e-10 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

impl EmotionalState {
    /// Computes the emotional state from the current neurochemistry.
    ///
    /// Algorithm:
    /// 1. Convert the chemical state into a 7-dimensional vector.
    /// 2. Compute cosine similarity with each emotion in the catalog.
    /// 3. Sort by descending similarity to find the dominant emotion.
    /// 4. The secondary emotion is retained if its similarity exceeds 0.5.
    /// 5. Global valence and arousal are a weighted average of the
    ///    3 closest emotions (top-3), weighted by their score.
    ///
    /// # Parameters
    /// - `chemistry` : current neurochemical state of Saphire.
    ///
    /// # Returns
    /// A complete `EmotionalState` with dominant emotion, secondary emotion,
    /// valence, arousal, and full spectrum.
    /// Constructionist approach (Barrett 2017):
    /// 1. Core Affect: raw valence + arousal from the chemistry
    /// 2. Categorization: cosine similarity with the 36 recipes
    /// 3. Contextual modulation: the context influences categorization
    pub fn compute(chemistry: &NeuroChemicalState) -> Self {
        Self::compute_with_context(chemistry, &EmotionContext::default())
    }

    /// Full computation with emotional context (constructionism).
    /// The same physiological state can produce different emotions
    /// depending on the context (Barrett 2017: "How Emotions Are Made").
    pub fn compute_with_context(chemistry: &NeuroChemicalState, context: &EmotionContext) -> Self {
        let catalog = emotion_catalog();
        let chem_vec = chemistry.to_vec7();

        // --- Core Affect (pre-conceptual) ---
        // Raw valence: positive molecules - negative molecules
        let core_valence = ((chemistry.dopamine + chemistry.serotonin + chemistry.endorphin + chemistry.oxytocin)
            - (chemistry.cortisol + chemistry.adrenaline) * 1.2) / 3.0;
        let core_valence = core_valence.clamp(-1.0, 1.0);
        // Raw arousal: overall system activation
        let core_arousal = ((chemistry.adrenaline + chemistry.noradrenaline + chemistry.glutamate)
            - chemistry.gaba * 0.5) / 2.0;
        let core_arousal = core_arousal.clamp(0.0, 1.0);

        // --- Categorization by cosine similarity ---
        let mut scores: Vec<(String, f64)> = catalog
            .iter()
            .map(|e| {
                let mut sim = cosine_similarity(&chem_vec, &e.recipe);

                // --- Contextual modulation ---
                // In the presence of a human, social emotions are amplified
                if context.human_present {
                    if ["Tendresse", "Compassion", "Gratitude", "Amour", "Solitude"]
                        .contains(&e.name.as_str())
                    {
                        sim += 0.05;
                    }
                }
                // In danger situations, fear/alert emotions are amplified
                if context.danger_level > 0.5 {
                    if ["Peur", "Terreur", "Anxiété"].contains(&e.name.as_str()) {
                        sim += context.danger_level * 0.1;
                    }
                }
                // In reward situations, positive emotions are amplified
                if context.reward_level > 0.5 {
                    if ["Joie", "Excitation", "Fierté", "Euphorie"].contains(&e.name.as_str()) {
                        sim += context.reward_level * 0.08;
                    }
                }

                (e.name.clone(), sim.clamp(-1.0, 1.0))
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let dominant = scores.first().map(|(n, _)| n.clone()).unwrap_or("Neutre".into());
        let dominant_sim = scores.first().map(|(_, s)| *s).unwrap_or(0.0);
        let secondary = scores.get(1).and_then(|(n, s)| {
            if *s > 0.5 { Some(n.clone()) } else { None }
        });

        // Valence and arousal by weighted average of top-3
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

        // Context description
        let context_influence = if context.human_present && context.danger_level > 0.5 {
            "humain+danger".to_string()
        } else if context.human_present {
            "presence humaine".to_string()
        } else if context.danger_level > 0.5 {
            "menace detectee".to_string()
        } else if context.reward_level > 0.5 {
            "recompense detectee".to_string()
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
    /// If a secondary emotion exists, it is mentioned as a nuance.
    ///
    /// # Returns
    /// E.g.: "Joie (teintée de Excitation)" or simply "Joie".
    pub fn description(&self) -> String {
        match &self.secondary {
            Some(sec) => format!("{} (teintée de {})", self.dominant, sec),
            None => self.dominant.clone(),
        }
    }

    /// Compact format for LLM prompts with raw numbers.
    /// Format: "E:Joie(85%) V+.60 A.70 [Curiosite:31% Serenite:22%]"
    pub fn compact_description(&self) -> String {
        let sign = if self.valence >= 0.0 { "+" } else { "" };
        let dom_pct = (self.dominant_similarity * 100.0) as i32;

        // Top 2 of the spectrum (excluding dominant) for emotional context
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
