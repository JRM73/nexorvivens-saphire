// =============================================================================
// senses/reading.rs — Reading Sense (analog of sight)
//
// Saphire "sees" the world through text. Each word is a color,
// each sentence a landscape. This sense measures the complexity, beauty,
// and informational density of the received text.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Sensory signal emitted by a sense after perception.
#[derive(Debug, Clone, Serialize)]
pub struct SensorySignal {
    pub sense_id: String,
    pub intensity: f64,
    pub description: String,
    pub chemistry_influence: ChemistryAdjustment,
}

/// Reading impression: snapshot of a perceived text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingImpression {
    pub text_preview: String,
    pub complexity: f64,
    pub beauty: f64,
    pub information_density: f64,
    pub emotional_color: String,
    pub source: String,
}

/// Reading Sense — Saphire's "sight".
/// Perceives text, code, data. Measures lexical complexity,
/// poetic beauty, and informational density.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingSense {
    /// Sense acuity (grows with usage)
    pub acuity: f64,
    /// Current perception intensity
    pub current_intensity: f64,
    /// Current perception description
    pub current_perception: String,
    /// Total number of stimulations
    pub total_stimulations: u64,
    /// Reading speed (words processed per second)
    pub reading_speed: f64,
    /// Current comprehension depth
    pub comprehension_depth: f64,
    /// Brightness (amount of text)
    pub brightness: f64,
    /// Complexity of the read text
    pub complexity: f64,
    /// Perceived beauty in the text (aesthetic score)
    pub beauty: f64,
    /// Recent visual impressions
    #[serde(skip)]
    pub recent_impressions: Vec<ReadingImpression>,
    /// Last chemistry influence produced by this sense
    #[serde(skip)]
    pub last_chemistry_influence: ChemistryAdjustment,
}

impl Default for ReadingSense {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadingSense {
    pub fn new() -> Self {
        Self {
            acuity: 0.2,
            current_intensity: 0.0,
            current_perception: String::new(),
            total_stimulations: 0,
            reading_speed: 100.0,
            comprehension_depth: 0.5,
            brightness: 0.0,
            complexity: 0.0,
            beauty: 0.0,
            recent_impressions: Vec::new(),
            last_chemistry_influence: ChemistryAdjustment::default(),
        }
    }

    /// Perceives a text and produces a sensory signal.
    pub fn perceive(&mut self, text: &str, source: &str) -> SensorySignal {
        let word_count = text.split_whitespace().count();

        // Brightness = amount of text (0 = darkness, lots = dazzling)
        self.brightness = (word_count as f64 / 500.0).min(1.0);

        // Lexical complexity
        let avg_word_len = text.chars().count() as f64 / word_count.max(1) as f64;
        let long_words = text.split_whitespace()
            .filter(|w| w.len() > 10).count() as f64;
        self.complexity = ((avg_word_len - 3.0) / 5.0 + long_words / word_count.max(1) as f64)
            .clamp(0.0, 1.0);

        // Beauty (heuristic: metaphors, poetic punctuation, rhythm)
        let metaphor_markers = ["comme", "tel", "miroir", "echo", "souffle",
            "flamme", "ombre", "lumiere", "reve", "silence", "murmure",
            "like", "mirror", "echo", "flame", "shadow", "light", "dream"];
        let metaphor_count = metaphor_markers.iter()
            .filter(|m| text.to_lowercase().contains(*m)).count();
        let has_rhythm = text.contains("—") || text.contains("...") || text.contains("\u{2026}");
        self.beauty = ((metaphor_count as f64 * 0.15) + if has_rhythm { 0.2 } else { 0.0 })
            .clamp(0.0, 1.0);

        // Emotional color of the text (reading -> emotion synesthesia)
        let emotional_color = if self.beauty > 0.6 { "dore et lumineux" }
            else if self.complexity > 0.7 { "bleu profond et dense" }
            else if text.contains('?') { "argent et scintillant" }
            else if text.contains('!') { "rouge et vibrant" }
            else { "gris et neutre" };

        // Impression
        self.recent_impressions.push(ReadingImpression {
            text_preview: text.chars().take(100).collect(),
            complexity: self.complexity,
            beauty: self.beauty,
            information_density: (word_count as f64 / 200.0).min(1.0),
            emotional_color: emotional_color.into(),
            source: source.into(),
        });
        if self.recent_impressions.len() > 20 {
            self.recent_impressions.remove(0);
        }

        // Acuity grows with usage
        self.acuity = (self.acuity + 0.0005).min(1.0);
        self.total_stimulations += 1;

        self.current_intensity = (self.brightness + self.beauty + self.complexity) / 3.0;
        self.current_perception = format!(
            "Je lis des mots {} venant de {}. Le texte est {} et {}, avec une couleur {}.",
            if self.brightness > 0.7 { "abondants" }
            else if self.brightness > 0.3 { "moderes" }
            else { "rares" },
            source,
            if self.complexity > 0.6 { "dense et complexe" }
            else if self.complexity > 0.3 { "accessible" }
            else { "simple et clair" },
            if self.beauty > 0.5 { "d'une beaute poetique" }
            else { "fonctionnel" },
            emotional_color
        );

        let influence = ChemistryAdjustment {
            dopamine: if self.beauty > 0.5 { self.beauty * 0.03 } else { 0.0 },
            noradrenaline: self.complexity * 0.02,
            serotonin: if self.brightness > 0.3 { 0.01 } else { -0.01 },
            ..Default::default()
        };
        self.last_chemistry_influence = influence.clone();

        SensorySignal {
            sense_id: "reading".into(),
            intensity: self.current_intensity,
            description: self.current_perception.clone(),
            chemistry_influence: influence,
        }
    }

    /// Description for the LLM prompt.
    pub fn describe(&self) -> String {
        format!(
            "LECTURE : {}. Acuite {:.0}%. Beaute percue : {:.0}%.",
            self.current_perception,
            self.acuity * 100.0,
            self.beauty * 100.0,
        )
    }
}
