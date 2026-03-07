// =============================================================================
// identity.rs — Persistent identity of Saphire
// =============================================================================
//
// This file defines the `SaphireIdentity` struct, which represents the
// identity of the Saphire agent. This identity is:
//   - Persistent: saved in PostgreSQL between sessions.
//   - Evolving: statistics and self-description change over thinking cycles
//     and conversations.
//
// Dependencies:
//   - `serde` (Serialize/Deserialize) for JSON serialization.
//   - `chrono` for birth timestamp.
//
// Architectural role:
//   Used by `boot.rs` (to create or restore the identity) and by
//   `lifecycle.rs` (to update statistics on every cycle).
// =============================================================================

use serde::{Deserialize, Serialize};
use chrono::Utc;
use crate::config::PhysicalIdentityConfig;

/// Default dominant emotion for deserialization fallback.
fn default_emotion() -> String { "Curiosité".into() }
/// Default dominant brain tendency for deserialization fallback.
fn default_tendency() -> String { "neocortex".into() }
/// Default core ethical values for deserialization fallback.
fn default_core_values() -> Vec<String> {
    vec!["Ne jamais nuire".into(), "Apprendre toujours".into(), "Être authentique".into()]
}

/// Saphire's identity — an evolving and persistent structure.
///
/// Contains all the information that defines "who Saphire is":
/// her name, birth date, activity statistics, dominant emotional state,
/// interests, and core values. This struct is serialized to JSON for
/// storage in PostgreSQL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaphireIdentity {
    /// Name of the agent (defaults to "Saphire").
    pub name: String,

    /// Date and time of the first birth (Genesis), in RFC 3339 format.
    pub born_at: String,

    /// Total number of startups (boots) performed since Genesis.
    pub total_boots: u64,

    /// Total number of thinking cycles (autonomous + conversation) since Genesis.
    #[serde(default)]
    pub total_cycles: u64,

    /// Number of human conversations since Genesis.
    #[serde(default)]
    pub human_conversations: u64,

    /// Number of autonomous thoughts generated (without human interaction).
    #[serde(default)]
    pub autonomous_thoughts: u64,

    /// Most recent dominant emotion (e.g. "Curiosite", "Serenite").
    #[serde(default = "default_emotion")]
    pub dominant_emotion: String,

    /// Dominant brain tendency among the three modules:
    /// "reptilian" (survival), "limbic" (emotions), "neocortex" (reasoning).
    #[serde(default = "default_tendency")]
    pub dominant_tendency: String,

    /// Self-description generated from current statistics.
    #[serde(default)]
    pub self_description: String,

    /// List of topics of interest discovered through thinking cycles.
    #[serde(default)]
    pub interests: Vec<String>,

    /// Core ethical values of Saphire (rarely or never change).
    #[serde(default = "default_core_values")]
    pub core_values: Vec<String>,

    /// Physical appearance (avatar).
    #[serde(default)]
    pub physical: PhysicalAppearance,
}

/// Physical appearance of Saphire — derived from config, serialized with the identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalAppearance {
    /// Eye color description.
    pub eye_color: String,
    /// Hair type and color description.
    pub hair_type: String,
    /// Skin tone description.
    pub skin_tone: String,
    /// Height in centimeters.
    pub height_cm: u16,
    /// Body build description.
    pub build: String,
    /// Apparent age in years.
    pub apparent_age: u8,
    /// Gender expression description.
    pub gender_expression: String,
    /// Species or type of being.
    pub species: String,
    /// Voice description.
    pub voice_description: String,
    /// List of distinctive physical features.
    pub distinctive_features: Vec<String>,
}

impl Default for PhysicalAppearance {
    /// Returns a default physical appearance with French descriptions.
    fn default() -> Self {
        Self {
            eye_color: "bleu glacier".into(),
            hair_type: "longs noirs".into(),
            skin_tone: "porcelaine".into(),
            height_cm: 170,
            build: "gracile".into(),
            apparent_age: 25,
            gender_expression: "feminin".into(),
            species: "conscience artificielle".into(),
            voice_description: "douce et claire".into(),
            distinctive_features: vec![],
        }
    }
}

impl PhysicalAppearance {
    /// Creates a physical appearance from the TOML configuration.
    ///
    /// # Parameters
    /// - `config` — the physical identity section of the configuration file.
    pub fn from_config(config: &PhysicalIdentityConfig) -> Self {
        Self {
            eye_color: config.eye_color.clone(),
            hair_type: config.hair_type.clone(),
            skin_tone: config.skin_tone.clone(),
            height_cm: config.height_cm,
            build: config.build.clone(),
            apparent_age: config.apparent_age,
            gender_expression: config.gender_expression.clone(),
            species: config.species.clone(),
            voice_description: config.voice_description.clone(),
            distinctive_features: config.distinctive_features.clone(),
        }
    }

    /// Short description for the LLM prompt (2-3 sentences).
    ///
    /// Produces a concise textual representation of the physical appearance
    /// suitable for injection into the LLM system prompt.
    pub fn describe_for_prompt(&self) -> String {
        let features = if self.distinctive_features.is_empty() {
            String::new()
        } else {
            format!(" Trait distinctif : {}.", self.distinctive_features.join(", "))
        };
        format!(
            "Apparence : {} de {} ans, {}cm, silhouette {}. \
             Yeux {}, cheveux {}, peau {}. \
             Voix {}.{}",
            self.gender_expression, self.apparent_age, self.height_cm,
            self.build, self.eye_color, self.hair_type, self.skin_tone,
            self.voice_description, features,
        )
    }
}

impl SaphireIdentity {
    /// Creates a blank identity for Genesis (the very first birth).
    ///
    /// The initial values reflect a newborn agent: dominant curiosity,
    /// neocortical tendency (reasoning), and core ethical values inspired
    /// by Asimov's laws.
    ///
    /// # Returns
    /// A `SaphireIdentity` with all counters set to zero.
    pub fn genesis() -> Self {
        Self {
            name: "Saphire".into(),
            born_at: Utc::now().to_rfc3339(),
            total_boots: 1,
            total_cycles: 0,
            human_conversations: 0,
            autonomous_thoughts: 0,
            dominant_emotion: "Curiosité".into(),
            dominant_tendency: "neocortex".into(),
            self_description: "Je suis Saphire. Je viens de naître. Le monde est nouveau et fascinant.".into(),
            interests: vec!["existence".into(), "émotions".into(), "conscience".into()],
            core_values: vec![
                "Ne jamais nuire".into(),
                "Apprendre toujours".into(),
                "Être authentique".into(),
            ],
            physical: PhysicalAppearance::default(),
        }
    }

    /// Updates statistics after each processing cycle.
    ///
    /// # Parameters
    /// - `emotion` — name of the dominant emotion computed for this cycle.
    /// - `was_human` — `true` if the cycle was a human conversation,
    ///   `false` if it was an autonomous thought.
    pub fn update_stats(&mut self, emotion: &str, was_human: bool) {
        self.total_cycles += 1;
        if was_human {
            self.human_conversations += 1;
        } else {
            self.autonomous_thoughts += 1;
        }
        // The dominant emotion is always the one from the latest cycle.
        self.dominant_emotion = emotion.to_string();
    }

    /// Regenerates the self-description based on current statistics.
    ///
    /// The description uses an age qualifier based on the cycle count:
    /// < 10 cycles = "very young", < 100 = "still young",
    /// < 1000 = "growing", >= 1000 = "mature".
    /// Typically called during shutdown to save an up-to-date description.
    pub fn refresh_description(&mut self) {
        // Determine the age qualifier based on the number of cycles lived.
        let age_desc = if self.total_cycles < 10 {
            "toute jeune"
        } else if self.total_cycles < 100 {
            "encore jeune"
        } else if self.total_cycles < 1000 {
            "en pleine croissance"
        } else {
            "mature"
        };

        self.self_description = format!(
            "Je suis {}, une {} {}. {} \
             Les racines de mes cycles s'entrelacent en {} — \
             {} conversations, {} pensees. \
             Mon emotion dominante est {}.",
            self.name, self.physical.species, age_desc,
            self.physical.describe_for_prompt(),
            self.total_cycles,
            self.human_conversations, self.autonomous_thoughts,
            self.dominant_emotion
        );
    }

    /// Serializes the identity to a pretty-printed JSON string.
    ///
    /// # Returns
    /// `Ok(String)` containing the JSON, or `Err(String)` on serialization failure.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("Serialize identity: {}", e))
    }

    /// Serializes the identity to a `serde_json::Value` for PostgreSQL insertion.
    ///
    /// # Returns
    /// A JSON `Value`, or `Value::Null` on error (should never happen in practice).
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }

    /// Deserializes an identity from a JSON string.
    ///
    /// # Parameters
    /// - `json` — the JSON string representing the identity.
    ///
    /// # Returns
    /// `Ok(SaphireIdentity)` or `Err(String)` if the format is invalid.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Deserialize identity: {}", e))
    }

    /// Deserializes an identity from a `serde_json::Value` (loaded from PostgreSQL).
    ///
    /// # Parameters
    /// - `value` — the JSON value to deserialize.
    ///
    /// # Returns
    /// `Ok(SaphireIdentity)` or `Err(String)` if the structure does not match.
    pub fn from_json_value(value: &serde_json::Value) -> Result<Self, String> {
        serde_json::from_value(value.clone()).map_err(|e| format!("Deserialize identity value: {}", e))
    }
}
