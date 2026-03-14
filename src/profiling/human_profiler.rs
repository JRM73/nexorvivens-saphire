// =============================================================================
// human_profiler.rs — HumanProfiler: psychological profiling of interlocutors
//
// Role: Analyzes human interlocutors' messages to build and update their
//       OCEAN psychological profile (Openness / Conscientiousness /
//       Extraversion / Agreeableness / Neuroticism) and their
//       communication style.
//
// How it works:
//   For each human message, the profiler updates:
//     1. Communication style (verbosity, formality, emotionality, etc.)
//     2. OCEAN dimensions (inferred from content and style)
//     3. Emotional patterns (sentiment polarity history)
//     4. Rapport score (relationship quality)
//
//   Exponential smoothing (80% old + 20% new) is used for communication
//   style, and (90% old + 10% new) for OCEAN dimensions, to avoid
//   abrupt fluctuations.
//
// Dependencies:
//   - std::collections::HashMap: storage of profiles and emotional patterns
//   - chrono: interaction timestamps
//   - serde: serialization for persistence
//   - crate::nlp: NlpResult (NLP analysis result of the message)
//   - crate::nlp::intent: Intent (detected intent of the message)
//   - super::ocean: OceanProfile
//
// Place in architecture:
//   Called by the main cognitive loop when a human message is received.
//   The resulting human profile is consumed by the adaptation module to
//   generate style instructions adapted to the interlocutor.
// =============================================================================

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::nlp::NlpResult;
use crate::nlp::intent::Intent;
use super::ocean::OceanProfile;

/// Psychological profile of a human interlocutor built through observation.
///
/// Accumulates data over interactions to progressively refine
/// the psychological portrait and communication style of the human.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanProfile {
    /// Name or identifier of the profiled human
    pub name: String,
    /// OCEAN (Big Five) profile estimated through message observation
    pub ocean: OceanProfile,
    /// Observed communication style (verbosity, formality, etc.)
    pub communication_style: CommunicationStyle,
    /// Total number of recorded interactions with this human
    pub interaction_count: u64,
    /// Timestamp of the first interaction
    pub first_seen: DateTime<Utc>,
    /// Timestamp of the last interaction
    pub last_seen: DateTime<Utc>,
    /// List of preferred topics detected in conversations
    pub preferred_topics: Vec<String>,
    /// Emotional pattern history: key = polarity label
    /// ("positif", "negatif", "neutre"), value = occurrence count
    pub emotional_patterns: HashMap<String, u32>,
    /// Rapport score in [0.0, 1.0]: estimated relationship quality.
    /// Increases when interactions are positive, stagnates or decreases otherwise.
    pub rapport_score: f64,
}

impl HumanProfile {
    /// Creates a new human profile with default values.
    ///
    /// The OCEAN profile is initialized to neutral (0.5), communication style
    /// is at median value, and rapport is at 0.5.
    ///
    /// Parameters:
    ///   - name: the name or identifier of the human
    ///
    /// Returns: an initialized HumanProfile
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ocean: OceanProfile::default(),
            communication_style: CommunicationStyle::default(),
            interaction_count: 0,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            preferred_topics: Vec::new(),
            emotional_patterns: HashMap::new(),
            rapport_score: 0.5,
        }
    }
}

/// Observed communication style of a human.
///
/// Each dimension is a value in [0.0, 1.0] updated by exponential
/// smoothing (80% old + 20% new) with each message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStyle {
    /// Verbosity: tendency to write long messages (0 = very concise, 1 = very verbose).
    /// Computed from word count per message, normalized to 50 words = 1.0.
    pub verbosity: f64,
    /// Formality: language register (0 = very informal, 1 = very formal).
    /// Estimated by the ratio of formal markers / (formal + informal markers).
    pub formality: f64,
    /// Emotionality: tendency to express emotions in messages.
    /// Based on the absolute value of the sentiment compound score.
    pub emotionality: f64,
    /// Directness: tendency to give orders or direct instructions.
    /// High if the message starts with an imperative verb or if the intent is Command.
    pub directness: f64,
    /// Questioning rate: proportion of messages containing questions.
    /// 1.0 if the message contains '?', 0.0 otherwise.
    pub questioning_rate: f64,
    /// Preferred language of the interlocutor ("fr" for French, "en" for English).
    pub preferred_language: String,
}

impl Default for CommunicationStyle {
    /// Default communication style: all dimensions at 0.5 (neutral),
    /// preferred language = French.
    fn default() -> Self {
        Self {
            verbosity: 0.5,
            formality: 0.5,
            emotionality: 0.5,
            directness: 0.5,
            questioning_rate: 0.5,
            preferred_language: "fr".into(),
        }
    }
}

/// Profiler for human interlocutors interacting with Saphire.
///
/// Maintains a dictionary of profiles indexed by human identifier.
/// Each new message is analyzed to update the corresponding profile.
pub struct HumanProfiler {
    /// Dictionary of human profiles: key = identifier, value = profile
    profiles: HashMap<String, HumanProfile>,
}

impl Default for HumanProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl HumanProfiler {
    /// Creates a new HumanProfiler with an empty dictionary.
    ///
    /// Returns: a HumanProfiler instance ready to observe
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }

    /// Returns the profile of a human identified by their ID.
    ///
    /// Parameters:
    ///   - human_id: the human's identifier
    ///
    /// Returns: Some(&HumanProfile) if the profile exists, None otherwise
    pub fn get_profile(&self, human_id: &str) -> Option<&HumanProfile> {
        self.profiles.get(human_id)
    }

    /// Returns the active profile (the most recently seen human).
    ///
    /// Useful for getting the current interlocutor's profile without knowing
    /// their identifier.
    ///
    /// Returns: Some(&HumanProfile) of the most recent profile, None if no profile
    pub fn current_profile(&self) -> Option<&HumanProfile> {
        // Select the profile with the most recent last_seen
        self.profiles.values()
            .max_by_key(|p| p.last_seen)
    }

    /// Loads profiles from an external source (database).
    ///
    /// Parameters:
    ///   - profiles: list of (identifier, profile) tuples to load
    pub fn load_profiles(&mut self, profiles: Vec<(String, HumanProfile)>) {
        for (id, profile) in profiles {
            self.profiles.insert(id, profile);
        }
    }

    /// Returns a reference to the complete profile dictionary.
    ///
    /// Useful for periodic database saving.
    ///
    /// Returns: reference to the HashMap of all profiles
    pub fn all_profiles(&self) -> &HashMap<String, HumanProfile> {
        &self.profiles
    }

    /// Analyzes a human message and updates the interlocutor's profile.
    ///
    /// Processing is done in 4 steps:
    ///   1. Update communication style (verbosity, formality, etc.)
    ///   2. Estimate OCEAN dimensions by inference
    ///   3. Update emotional patterns
    ///   4. Update rapport score
    ///
    /// Parameters:
    ///   - human_id: the human's identifier
    ///   - message: the raw text of the message
    ///   - nlp_result: the NLP analysis result of the message
    pub fn observe_message(
        &mut self,
        human_id: &str,
        message: &str,
        nlp_result: &NlpResult,
    ) {
        // Get or create the human's profile
        let profile = self.profiles.entry(human_id.to_string())
            .or_insert_with(|| HumanProfile::new(human_id));

        profile.interaction_count += 1;
        profile.last_seen = Utc::now();

        // === Step 1: Estimate communication style ===
        let word_count = message.split_whitespace().count() as f64;
        let style = &mut profile.communication_style;

        // Verbosity: word count normalized to 50 (50 words = maximum verbosity)
        // Exponential smoothing: 80% old + 20% new
        let msg_verbosity = (word_count / 50.0).min(1.0);
        style.verbosity = style.verbosity * 0.8 + msg_verbosity * 0.2;

        // Formality: ratio of formal markers / (formal + informal)
        // Formal markers: formal address, polite expressions
        // Informal markers: casual address, slang, abbreviations
        let formal_markers = ["vous", "veuillez", "cordialement", "merci de",
                              "pourriez-vous", "je vous prie", "would you"];
        let informal_markers = ["tu", "salut", "cool", "mdr", "lol", "hey",
                                "ouais", "trop", "grave"];
        let lower = message.to_lowercase();
        let formal_count = formal_markers.iter()
            .filter(|m| lower.contains(*m)).count() as f64;
        let informal_count = informal_markers.iter()
            .filter(|m| lower.contains(*m)).count() as f64;
        let msg_formality = if formal_count + informal_count > 0.0 {
            formal_count / (formal_count + informal_count)
        } else { 0.5 };
        style.formality = style.formality * 0.8 + msg_formality * 0.2;

        // Emotionality: based on the absolute value of the sentiment compound score
        let msg_emotionality = nlp_result.sentiment.compound.abs();
        style.emotionality = style.emotionality * 0.8 + msg_emotionality * 0.2;

        // Directness: high if the message starts with an imperative verb
        // or if the detected NLP intent is a Command
        let directive_markers = ["fais", "cree", "montre", "donne", "change",
                                 "arrete", "do", "make", "show", "give"];
        let is_directive = directive_markers.iter()
            .any(|m| lower.starts_with(m))
            || nlp_result.intent.primary_intent == Intent::Command;
        let msg_directness = if is_directive { 0.9 } else { 0.4 };
        style.directness = style.directness * 0.8 + msg_directness * 0.2;

        // Questioning rate: 1.0 if the message contains '?', 0.0 otherwise
        let has_question = message.contains('?');
        let msg_questioning = if has_question { 1.0 } else { 0.0 };
        style.questioning_rate = style.questioning_rate * 0.8 + msg_questioning * 0.2;

        // Preferred language: updated if the language is detected
        style.preferred_language = match nlp_result.language {
            crate::nlp::preprocessor::Language::French => "fr".into(),
            crate::nlp::preprocessor::Language::English => "en".into(),
            crate::nlp::preprocessor::Language::Unknown => style.preferred_language.clone(),
        };

        // === Step 2: Estimate the human's OCEAN dimensions ===
        // Estimates use slower smoothing (90% old + 10% new)
        // because the OCEAN profile must remain stable over time.
        let ocean = &mut profile.ocean;

        // Openness: increases if the message is philosophical or questioning,
        // as curiosity and reflection are markers of openness.
        if nlp_result.intent.primary_intent == Intent::Philosophical
           || nlp_result.intent.primary_intent == Intent::Question {
            ocean.openness.score = (ocean.openness.score * 0.9 + 0.8 * 0.1).min(1.0);
        }

        // Extraversion: increases if the message is long (> 30 words) or contains '!',
        // as expressiveness and enthusiasm are markers of extraversion.
        if word_count > 30.0 || message.contains('!') {
            ocean.extraversion.score = (ocean.extraversion.score * 0.9 + 0.7 * 0.1).min(1.0);
        }

        // Agreeableness: increases if the message contains warm words
        // (thanks, compliments, trust).
        let warm_words = ["merci", "bravo", "super", "bien", "confiance",
                          "j'apprecie", "thanks", "great", "love"];
        if warm_words.iter().any(|w| lower.contains(w)) {
            ocean.agreeableness.score = (ocean.agreeableness.score * 0.9 + 0.8 * 0.1).min(1.0);
        }

        // Neuroticism: increases if the message stimulus has high
        // urgency (> 0.5) or significant danger (> 0.3), as anxiety
        // and stress are markers of neuroticism.
        if nlp_result.stimulus.urgency > 0.5 || nlp_result.stimulus.danger > 0.3 {
            ocean.neuroticism.score = (ocean.neuroticism.score * 0.9 + 0.6 * 0.1).min(1.0);
        }

        // === Step 3: Emotional patterns ===
        // Classifies the message sentiment into 3 categories and increments the counter
        let polarity_label = if nlp_result.sentiment.compound > 0.2 { "positif" }
            else if nlp_result.sentiment.compound < -0.2 { "negatif" }
            else { "neutre" };
        let emotion_entry = profile.emotional_patterns
            .entry(polarity_label.to_string()).or_insert(0);
        *emotion_entry += 1;

        // === Step 4: Rapport score ===
        // Rapport increases slightly (+0.02) with each positive interaction.
        // It is capped at 1.0. Negative interactions do not decrease rapport
        // (relational stability).
        if nlp_result.sentiment.compound > 0.0 {
            profile.rapport_score = (profile.rapport_score + 0.02).min(1.0);
        }
    }
}
