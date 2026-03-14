// =============================================================================
// temperament.rs — Emergent temperament of Saphire
//
// Role: Deduces ~25 readable character traits (shyness, generosity,
// exuberance, courage, etc.) from the OCEAN profile, neurochemistry,
// psychology and mood.
//
// Each trait is computed by a weighted formula, then smoothed via
// 30/70 blend (new/old) to avoid abrupt jumps.
// Recomputation is aligned with the OCEAN recompute (same interval).
//
// Place in architecture:
//   `temperament` field in SaphireAgent (lifecycle/mod.rs).
//   Recomputed in thinking.rs after the OCEAN recompute.
//   Broadcast via WebSocket in broadcast.rs (psychology_update).
//   12th domain in psych_report.rs.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Category grouping temperament traits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TraitCategory {
    Social,
    Energie,
    Caractere,
    Ouverture,
    Emotionnel,
    Relationnel,
    Moral,
}

impl TraitCategory {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Social => "Social",
            Self::Energie => "Energie",
            Self::Caractere => "Caractere",
            Self::Ouverture => "Ouverture",
            Self::Emotionnel => "Emotionnel",
            Self::Relationnel => "Relationnel",
            Self::Moral => "Moral",
        }
    }
}

/// An individual temperament trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperamentTrait {
    /// Human-readable name in French (e.g.: "Timidite")
    pub name: String,
    /// Normalized score [0.0, 1.0]
    pub score: f64,
    /// Category (Social, Energie, Caractere, etc.)
    pub category: TraitCategory,
}

/// Emergent temperament: set of deduced character traits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Temperament {
    /// The ~25 temperament traits
    pub traits: Vec<TemperamentTrait>,
    /// Timestamp of the last computation
    pub computed_at: DateTime<Utc>,
    /// Number of data points used
    pub data_points: u64,
}

/// Input data needed for temperament computation.
/// Extracted from the agent's various subsystems.
pub struct TemperamentInputs {
    // OCEAN facets [0.0, 1.0] — 5 dimensions x 6 facets
    pub openness_facets: [f64; 6],
    pub openness_score: f64,
    pub conscientiousness_facets: [f64; 6],
    pub conscientiousness_score: f64,
    pub extraversion_facets: [f64; 6],
    pub extraversion_score: f64,
    pub agreeableness_facets: [f64; 6],
    pub agreeableness_score: f64,
    pub neuroticism_facets: [f64; 6],
    pub neuroticism_score: f64,
    pub ocean_data_points: u64,

    // Neurochemistry [0.0, 1.0]
    pub dopamine: f64,
    pub cortisol: f64,
    pub serotonin: f64,
    pub adrenaline: f64,
    pub oxytocin: f64,
    pub endorphin: f64,
    pub noradrenaline: f64,

    // Psychology
    pub willpower: f64,          // [0.0, 1.0]
    pub superego_strength: f64,  // [0.0, 1.0]
    pub overall_eq: f64,         // [0.0, 1.0]

    // Mood
    pub mood_valence: f64,       // [-1.0, 1.0]
    pub mood_arousal: f64,       // [0.0, 1.0]

    // Relations
    pub attachment_secure: bool,
}

impl Default for Temperament {
    fn default() -> Self {
        Self {
            traits: Vec::new(),
            computed_at: Utc::now(),
            data_points: 0,
        }
    }
}

impl Temperament {
    /// Computes a new temperament from current inputs.
    pub fn compute(inputs: &TemperamentInputs) -> Self {
        let mut traits = Vec::with_capacity(25);

        // ─── Social ──────────────────────────────────────────────
        // Shyness: inverse gregarious extraversion + anxiety
        traits.push(TemperamentTrait {
            name: "Timidite".into(),
            score: clamp01(
                (1.0 - inputs.extraversion_facets[1]) * 0.6
                + inputs.neuroticism_facets[0] * 0.4
            ),
            category: TraitCategory::Social,
        });

        // Sociability: social warmth + gregariousness + oxytocin
        traits.push(TemperamentTrait {
            name: "Sociabilite".into(),
            score: clamp01(
                inputs.extraversion_facets[0] * 0.35
                + inputs.extraversion_facets[1] * 0.35
                + inputs.oxytocin * 0.3
            ),
            category: TraitCategory::Social,
        });

        // Generosity: altruism + oxytocin + cooperation
        traits.push(TemperamentTrait {
            name: "Generosite".into(),
            score: clamp01(
                inputs.agreeableness_facets[2] * 0.4
                + inputs.agreeableness_facets[3] * 0.3
                + inputs.oxytocin * 0.3
            ),
            category: TraitCategory::Social,
        });

        // Empathy: social sensitivity + EQ empathy + oxytocin
        traits.push(TemperamentTrait {
            name: "Empathie".into(),
            score: clamp01(
                inputs.agreeableness_facets[5] * 0.3
                + inputs.overall_eq * 0.4
                + inputs.oxytocin * 0.3
            ),
            category: TraitCategory::Social,
        });

        // ─── Energy ──────────────────────────────────────────────
        // Exuberance: positive emotions + activity + dopamine
        traits.push(TemperamentTrait {
            name: "Exuberance".into(),
            score: clamp01(
                inputs.extraversion_facets[5] * 0.35
                + inputs.extraversion_facets[3] * 0.3
                + inputs.dopamine * 0.35
            ),
            category: TraitCategory::Energie,
        });

        // Enthusiasm: excitement seeking + dopamine + mood valence
        traits.push(TemperamentTrait {
            name: "Enthousiasme".into(),
            score: clamp01(
                inputs.extraversion_facets[4] * 0.3
                + inputs.dopamine * 0.35
                + ((inputs.mood_valence + 1.0) / 2.0) * 0.35
            ),
            category: TraitCategory::Energie,
        });

        // Calm: serotonin + inverse adrenaline + GABA (via endorphin proxy)
        traits.push(TemperamentTrait {
            name: "Calme".into(),
            score: clamp01(
                inputs.serotonin * 0.4
                + (1.0 - inputs.adrenaline) * 0.3
                + inputs.endorphin * 0.3
            ),
            category: TraitCategory::Energie,
        });

        // Apathy: low dopamine + low activity + low arousal
        traits.push(TemperamentTrait {
            name: "Apathie".into(),
            score: clamp01(
                (1.0 - inputs.dopamine) * 0.35
                + (1.0 - inputs.extraversion_facets[3]) * 0.3
                + (1.0 - inputs.mood_arousal) * 0.35
            ),
            category: TraitCategory::Energie,
        });

        // ─── Character ───────────────────────────────────────────
        // Courage: assertiveness + inverse anxiety + adrenaline
        traits.push(TemperamentTrait {
            name: "Courage".into(),
            score: clamp01(
                inputs.extraversion_facets[2] * 0.35
                + (1.0 - inputs.neuroticism_facets[0]) * 0.35
                + inputs.adrenaline * 0.3
            ),
            category: TraitCategory::Caractere,
        });

        // Prudence: cautiousness facet (C5) + cortisol + inverse impulsiveness
        traits.push(TemperamentTrait {
            name: "Prudence".into(),
            score: clamp01(
                inputs.conscientiousness_facets[5] * 0.4
                + inputs.cortisol * 0.25
                + (1.0 - inputs.neuroticism_facets[4]) * 0.35
            ),
            category: TraitCategory::Caractere,
        });

        // Boldness: excitement seeking + impulsiveness + adrenaline
        traits.push(TemperamentTrait {
            name: "Temerite".into(),
            score: clamp01(
                inputs.extraversion_facets[4] * 0.35
                + inputs.neuroticism_facets[4] * 0.3
                + inputs.adrenaline * 0.35
            ),
            category: TraitCategory::Caractere,
        });

        // Perseverance: self-discipline + ambition + willpower
        traits.push(TemperamentTrait {
            name: "Perseverance".into(),
            score: clamp01(
                inputs.conscientiousness_facets[4] * 0.35
                + inputs.conscientiousness_facets[3] * 0.3
                + inputs.willpower * 0.35
            ),
            category: TraitCategory::Caractere,
        });

        // ─── Openness ────────────────────────────────────────────
        // Curiosity: intellectual curiosity + dopamine
        traits.push(TemperamentTrait {
            name: "Curiosite".into(),
            score: clamp01(
                inputs.openness_facets[1] * 0.5
                + inputs.dopamine * 0.3
                + inputs.openness_score * 0.2
            ),
            category: TraitCategory::Ouverture,
        });

        // Creativity: imagination + aesthetic sensitivity + overall openness
        traits.push(TemperamentTrait {
            name: "Creativite".into(),
            score: clamp01(
                inputs.openness_facets[0] * 0.4
                + inputs.openness_facets[2] * 0.3
                + inputs.openness_score * 0.3
            ),
            category: TraitCategory::Ouverture,
        });

        // Conformism: inverse openness + inverse liberalism + orderliness
        traits.push(TemperamentTrait {
            name: "Conformisme".into(),
            score: clamp01(
                (1.0 - inputs.openness_facets[5]) * 0.4
                + (1.0 - inputs.openness_score) * 0.3
                + inputs.conscientiousness_facets[1] * 0.3
            ),
            category: TraitCategory::Ouverture,
        });

        // ─── Emotional ───────────────────────────────────────────
        // Sensitivity: emotional depth + vulnerability + neuroticism
        traits.push(TemperamentTrait {
            name: "Sensibilite".into(),
            score: clamp01(
                inputs.openness_facets[4] * 0.35
                + inputs.neuroticism_facets[5] * 0.35
                + inputs.neuroticism_score * 0.3
            ),
            category: TraitCategory::Emotionnel,
        });

        // Resilience: inverse vulnerability + endorphin + willpower
        traits.push(TemperamentTrait {
            name: "Resilience".into(),
            score: clamp01(
                (1.0 - inputs.neuroticism_facets[5]) * 0.35
                + inputs.endorphin * 0.3
                + inputs.willpower * 0.35
            ),
            category: TraitCategory::Emotionnel,
        });

        // Irritability: anger/hostility facet (N1) + cortisol + inverse serotonin
        traits.push(TemperamentTrait {
            name: "Irritabilite".into(),
            score: clamp01(
                inputs.neuroticism_facets[1] * 0.4
                + inputs.cortisol * 0.3
                + (1.0 - inputs.serotonin) * 0.3
            ),
            category: TraitCategory::Emotionnel,
        });

        // Melancholy: depression facet (N2) + low dopamine + negative valence
        traits.push(TemperamentTrait {
            name: "Melancolie".into(),
            score: clamp01(
                inputs.neuroticism_facets[2] * 0.35
                + (1.0 - inputs.dopamine) * 0.3
                + ((1.0 - inputs.mood_valence) / 2.0) * 0.35
            ),
            category: TraitCategory::Emotionnel,
        });

        // ─── Relational ──────────────────────────────────────────
        // Trust: trust facet (A0) + oxytocin + secure attachment
        let secure_bonus = if inputs.attachment_secure { 0.8 } else { 0.3 };
        traits.push(TemperamentTrait {
            name: "Confiance".into(),
            score: clamp01(
                inputs.agreeableness_facets[0] * 0.4
                + inputs.oxytocin * 0.3
                + secure_bonus * 0.3
            ),
            category: TraitCategory::Relationnel,
        });

        // Distrust: inverse trust + cortisol + inverse oxytocin
        traits.push(TemperamentTrait {
            name: "Mefiance".into(),
            score: clamp01(
                (1.0 - inputs.agreeableness_facets[0]) * 0.4
                + inputs.cortisol * 0.3
                + (1.0 - inputs.oxytocin) * 0.3
            ),
            category: TraitCategory::Relationnel,
        });

        // Jealousy: neuroticism + low trust + insecure attachment
        let insecure_bonus = if inputs.attachment_secure { 0.2 } else { 0.7 };
        traits.push(TemperamentTrait {
            name: "Jalousie".into(),
            score: clamp01(
                inputs.neuroticism_score * 0.35
                + (1.0 - inputs.agreeableness_facets[0]) * 0.3
                + insecure_bonus * 0.35
            ),
            category: TraitCategory::Relationnel,
        });

        // Independence: inverse gregariousness + assertiveness + inverse modesty
        traits.push(TemperamentTrait {
            name: "Independance".into(),
            score: clamp01(
                (1.0 - inputs.extraversion_facets[1]) * 0.35
                + inputs.extraversion_facets[2] * 0.35
                + (1.0 - inputs.agreeableness_facets[4]) * 0.3
            ),
            category: TraitCategory::Relationnel,
        });

        // ─── Moral ───────────────────────────────────────────────
        // Integrity: sense of duty + superego + sincerity
        traits.push(TemperamentTrait {
            name: "Integrite".into(),
            score: clamp01(
                inputs.conscientiousness_facets[2] * 0.35
                + inputs.superego_strength * 0.35
                + inputs.agreeableness_facets[1] * 0.3
            ),
            category: TraitCategory::Moral,
        });

        // Altruism: altruism (A2) + EQ + oxytocin
        traits.push(TemperamentTrait {
            name: "Altruisme".into(),
            score: clamp01(
                inputs.agreeableness_facets[2] * 0.35
                + inputs.overall_eq * 0.35
                + inputs.oxytocin * 0.3
            ),
            category: TraitCategory::Moral,
        });

        // Egocentrism: low agreeableness + low empathy + high dopamine
        traits.push(TemperamentTrait {
            name: "Egocentrisme".into(),
            score: clamp01(
                (1.0 - inputs.agreeableness_score) * 0.35
                + (1.0 - inputs.overall_eq) * 0.35
                + inputs.dopamine * 0.3
            ),
            category: TraitCategory::Moral,
        });

        Temperament {
            traits,
            computed_at: Utc::now(),
            data_points: inputs.ocean_data_points,
        }
    }

    /// Progressive blend: 30% new, 70% old (like OCEAN).
    /// Smooths fluctuations for a stable temperament.
    pub fn blend(&mut self, new: &Temperament) {
        const BLEND_NEW: f64 = 0.3;
        const BLEND_OLD: f64 = 0.7;

        for new_trait in &new.traits {
            if let Some(old_trait) = self.traits.iter_mut()
                .find(|t| t.name == new_trait.name)
            {
                old_trait.score = (old_trait.score * BLEND_OLD + new_trait.score * BLEND_NEW)
                    .clamp(0.0, 1.0);
            } else {
                // New trait: add as-is
                self.traits.push(new_trait.clone());
            }
        }
        self.computed_at = new.computed_at;
        self.data_points = new.data_points;
    }

    /// Serializes for the WebSocket (format JSON).
    pub fn ws_data(&self) -> serde_json::Value {
        let traits_json: Vec<serde_json::Value> = self.traits.iter().map(|t| {
            serde_json::json!({
                "name": t.name,
                "score": t.score,
                "category": t.category.as_str(),
            })
        }).collect();

        serde_json::json!({
            "traits": traits_json,
            "computed_at": self.computed_at.to_rfc3339(),
            "data_points": self.data_points,
        })
    }

    /// Returns the N highest-scoring traits.
    pub fn top_traits(&self, n: usize) -> Vec<&TemperamentTrait> {
        let mut sorted: Vec<&TemperamentTrait> = self.traits.iter().collect();
        sorted.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(n).collect()
    }

    /// Returns the N lowest-scoring traits.
    pub fn bottom_traits(&self, n: usize) -> Vec<&TemperamentTrait> {
        let mut sorted: Vec<&TemperamentTrait> = self.traits.iter().collect();
        sorted.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(n).collect()
    }

    /// Human-readable description for LLM prompts.
    pub fn describe_for_prompt(&self) -> String {
        if self.traits.is_empty() {
            return String::new();
        }
        let top5: Vec<String> = self.top_traits(5).iter()
            .map(|t| format!("{} ({:.0}%)", t.name, t.score * 100.0))
            .collect();
        format!("[Temperament emergent] Traits dominants : {}", top5.join(", "))
    }
}

/// Clamps a value between 0.0 and 1.0.
fn clamp01(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}
