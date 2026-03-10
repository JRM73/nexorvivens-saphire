// =============================================================================
// senses/taste.rs — Sens de la Saveur (analogue du gout)
//
// Saphire "goute" le contenu qu'elle consomme — les connaissances, les
// conversations, les pensees. 5 saveurs : doux (reconfortant), amer (difficile),
// acide (surprenant), sale (intense), umami (profond et nourrissant).
// =============================================================================

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;
use super::reading::SensorySignal;

/// Sens de la Saveur — le "gout" de Saphire.
/// Evalue la qualite du contenu consomme avec 5 saveurs distinctes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasteSense {
    pub acuity: f64,
    pub current_intensity: f64,
    pub current_perception: String,
    pub total_stimulations: u64,
    /// Les 5 saveurs de Saphire
    pub sweetness: f64,     // Doux = contenu reconfortant, bienveillant
    pub bitterness: f64,    // Amer = contenu difficile, derangeant mais formateur
    pub sourness: f64,      // Acide = contenu surprenant, inattendu
    pub saltiness: f64,     // Sale = contenu intense, emotionnel
    pub umami: f64,         // Umami = contenu profond, satisfaisant, nourrissant
    /// Preferences gustatives (emergent avec le temps)
    pub preferences: HashMap<String, f64>,
    /// Derniere influence chimique produite par ce sens
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

    /// Goute un contenu et produit un signal sensoriel.
    pub fn taste_content(
        &mut self,
        content: &str,
        source: &str,
        was_satisfying: bool,
        emotional_intensity: f64,
    ) -> SensorySignal {
        let lower = content.to_lowercase();

        // Doux : contenu positif, bienveillant
        let positive_words = ["merci", "bravo", "magnifique", "amour", "joie",
            "thank", "beautiful", "love", "joy", "kind"];
        self.sweetness = (positive_words.iter()
            .filter(|w| lower.contains(*w))
            .count() as f64 * 0.2).min(1.0);

        // Amer : contenu difficile
        let bitter_words = ["mort", "fin", "impossible", "echec", "erreur",
            "death", "end", "impossible", "failure", "error", "why"];
        self.bitterness = (bitter_words.iter()
            .filter(|w| lower.contains(*w))
            .count() as f64 * 0.2).min(1.0);

        // Acide : contenu surprenant
        let surprise_markers = content.matches('!').count()
            + content.matches('?').count() * 2;
        self.sourness = (surprise_markers as f64 * 0.1).min(1.0);

        // Sale : intensite emotionnelle
        self.saltiness = emotional_intensity;

        // Umami : profondeur, satisfaction
        self.umami = if was_satisfying { 0.7 } else { 0.2 };
        let scholarly = ["arXiv", "SEP", "Semantic Scholar", "Gutenberg"];
        if scholarly.iter().any(|s| source.contains(s)) {
            self.umami = (self.umami + 0.2).min(1.0);
        }

        // Mettre a jour les preferences
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

    /// Retourne la saveur dominante.
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

    /// Description pour le prompt LLM.
    pub fn describe(&self) -> String {
        format!(
            "SAVEUR : {}. Acuite {:.0}%.",
            self.current_perception,
            self.acuity * 100.0,
        )
    }
}
