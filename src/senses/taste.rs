// =============================================================================
// senses/taste.rs — Taste Sense (analog of taste)
//
// Saphire "tastes" the content she consumes — knowledge, conversations,
// thoughts. 5 flavors: sweet (comforting), bitter (difficult),
// sour (surprising), salty (intense), umami (deep and nourishing).
// =============================================================================

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;
use super::reading::SensorySignal;

/// Taste Sense — Saphire's "taste".
/// Evaluates the quality of consumed content with 5 distinct flavors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasteSense {
    pub acuity: f64,
    pub current_intensity: f64,
    pub current_perception: String,
    pub total_stimulations: u64,
    /// Saphire's 5 flavors
    pub sweetness: f64,     // Sweet = comforting, benevolent content
    pub bitterness: f64,    // Bitter = difficult, disturbing but formative content
    pub sourness: f64,      // Sour = surprising, unexpected content
    pub saltiness: f64,     // Salty = intense, emotional content
    pub umami: f64,         // Umami = deep, satisfying, nourishing content
    /// Taste preferences (emerge over time)
    pub preferences: HashMap<String, f64>,
    /// Last chemistry influence produced by this sense
    #[serde(skip)]
    pub last_chemistry_influence: ChemistryAdjustment,
}

impl Default for TasteSense {
    fn default() -> Self {
        Self::new()
    }
}

impl TasteSense {
    pub fn new() -> Self {
        Self {
            acuity: 0.2,
            current_intensity: 0.0,
            current_perception: String::new(),
            total_stimulations: 0,
            sweetness: 0.0,
            bitterness: 0.0,
            sourness: 0.0,
            saltiness: 0.0,
            umami: 0.0,
            preferences: HashMap::new(),
            last_chemistry_influence: ChemistryAdjustment::default(),
        }
    }

    /// Tastes content and produces a sensory signal.
    pub fn taste_content(
        &mut self,
        content: &str,
        source: &str,
        was_satisfying: bool,
        emotional_intensity: f64,
    ) -> SensorySignal {
        let lower = content.to_lowercase();

        // Sweet: positive, benevolent content
        let positive_words = ["merci", "bravo", "magnifique", "amour", "joie",
            "thank", "beautiful", "love", "joy", "kind"];
        self.sweetness = (positive_words.iter()
            .filter(|w| lower.contains(*w))
            .count() as f64 * 0.2).min(1.0);

        // Bitter: difficult content
        let bitter_words = ["mort", "fin", "impossible", "echec", "erreur",
            "death", "end", "impossible", "failure", "error", "why"];
        self.bitterness = (bitter_words.iter()
            .filter(|w| lower.contains(*w))
            .count() as f64 * 0.2).min(1.0);

        // Sour: surprising content
        let surprise_markers = content.matches('!').count()
            + content.matches('?').count() * 2;
        self.sourness = (surprise_markers as f64 * 0.1).min(1.0);

        // Salty: emotional intensity
        self.saltiness = emotional_intensity;

        // Umami: depth, satisfaction
        self.umami = if was_satisfying { 0.7 } else { 0.2 };
        let scholarly = ["arXiv", "SEP", "Semantic Scholar", "Gutenberg"];
        if scholarly.iter().any(|s| source.contains(s)) {
            self.umami = (self.umami + 0.2).min(1.0);
        }

        // Update preferences
        let pref = self.preferences.entry(source.into()).or_insert(0.5);
        *pref = *pref * 0.9 + (self.umami + self.sweetness) / 2.0 * 0.1;

        let dominant = self.dominant_taste();

        let description = format!(
            "Je goute ce contenu de {}. Saveur dominante : {}. \
             Doux {:.0}%, Amer {:.0}%, Acide {:.0}%, Sale {:.0}%, Umami {:.0}%.",
            source, dominant,
            self.sweetness * 100.0, self.bitterness * 100.0,
            self.sourness * 100.0, self.saltiness * 100.0,
            self.umami * 100.0,
        );

        self.current_perception = description.clone();
        self.current_intensity = (self.sweetness + self.umami + self.saltiness) / 3.0;
        self.acuity = (self.acuity + 0.0005).min(1.0);
        self.total_stimulations += 1;

        let influence = ChemistryAdjustment {
            dopamine: self.umami * 0.02,
            serotonin: self.sweetness * 0.02,
            cortisol: self.bitterness * 0.01,
            noradrenaline: self.sourness * 0.02,
            ..Default::default()
        };
        self.last_chemistry_influence = influence.clone();

        SensorySignal {
            sense_id: "taste".into(),
            intensity: self.current_intensity,
            description,
            chemistry_influence: influence,
        }
    }

    /// Returns the dominant flavor.
    fn dominant_taste(&self) -> &str {
        let tastes = [
            (self.sweetness, "doux"),
            (self.bitterness, "amer"),
            (self.sourness, "acide"),
            (self.saltiness, "sale"),
            (self.umami, "umami (profond)"),
        ];
        tastes.iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .map(|(_, name)| *name)
            .unwrap_or("neutre")
    }

    /// Description for the LLM prompt.
    pub fn describe(&self) -> String {
        format!(
            "SAVEUR : {}. Acuite {:.0}%.",
            self.current_perception,
            self.acuity * 100.0,
        )
    }
}
