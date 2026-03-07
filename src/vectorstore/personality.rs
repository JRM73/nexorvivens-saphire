// personality.rs — Emergent personality from vector memory
//
// This module computes an emergent personality for Saphire based on
// statistical analysis of the emotions associated with her memories.
//
// The principle is that personality is not statically defined but
// emerges dynamically from emotional history: if Saphire has
// experienced many joyful moments, she will be characterized by her
// optimism; if she has explored a lot, by her curiosity, etc.
//
// The 3-step process:
//   1. Count the frequency of each emotion in the memories.
//   2. Derive composite personality traits from the raw emotional
//      frequencies.
//   3. Generate a textual description based on the dominant trait.
//
// Dependencies:
//   - serde: serialization/deserialization for the API and persistence.
//   - HashMap (std): storage of trait-name -> score associations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Saphire's emergent personality, dynamically computed from the
/// emotional history of her memories.
///
/// Traits are expressed as normalized scores between 0.0 and 1.0,
/// representing the proportion or intensity of each characteristic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergentPersonality {
    /// Associative table of traits (trait name -> score between 0.0 and 1.0).
    /// Contains both raw emotional frequencies (e.g., "Joie" -> 0.4) and
    /// composite personality traits (e.g., "Optimisme" -> 0.6).
    pub traits: HashMap<String, f64>,
    /// Automatically generated textual description, summarizing Saphire's
    /// dominant personality in a human-readable way.
    pub description: String,
    /// Total number of memories analyzed to compute this personality.
    /// The higher this number, the more reliable the profile.
    pub memory_count: u64,
}

impl EmergentPersonality {
    /// Computes the emergent personality from a list of emotions.
    ///
    /// Each emotion in the list corresponds to a memory. The algorithm
    /// counts occurrences of each emotion, normalizes them into
    /// proportions, then derives 5 composite personality traits:
    ///
    /// | Trait       | Formula                            |
    /// |-------------|------------------------------------|
    /// | Optimisme   | min(Joie + Serenite, 1.0)          |
    /// | Curiosite   | frequency of "Curiosite"           |
    /// | Empathie    | frequency of "Tendresse"           |
    /// | Anxiete     | frequency of "Anxiete"             |
    /// | Stabilite   | clamp(Serenite - Anxiete, 0, 1)    |
    ///
    /// # Parameters
    /// - `emotions`: slice of strings, each being the name of an emotion
    ///   (e.g., "Joie", "Curiosite", "Serenite", etc.).
    ///
    /// # Returns
    /// An EmergentPersonality instance containing the computed traits,
    /// a textual description, and the number of analyzed memories.
    pub fn compute(emotions: &[String]) -> Self {
        // Step 1: Count occurrences of each emotion.
        let mut emotion_counts: HashMap<String, u64> = HashMap::new();
        for emotion in emotions {
            *emotion_counts.entry(emotion.clone()).or_insert(0) += 1;
        }

        // Normalize counters into proportions (relative frequencies).
        // Use max(1) to avoid division by zero if the list is empty.
        let total = emotions.len().max(1) as f64;
        let mut traits: HashMap<String, f64> = emotion_counts.into_iter()
            .map(|(k, v)| (k, v as f64 / total))
            .collect();

        // Step 2: Derive composite personality traits from the raw
        // emotional frequencies. Extract key emotion frequencies first
        // (0.0 if absent).
        let joy = traits.get("Joie").copied().unwrap_or(0.0);
        let curiosity = traits.get("Curiosité").copied().unwrap_or(0.0);
        let anxiety = traits.get("Anxiété").copied().unwrap_or(0.0);
        let serenity = traits.get("Sérénité").copied().unwrap_or(0.0);
        let tenderness = traits.get("Tendresse").copied().unwrap_or(0.0);
        let compassion = traits.get("Compassion").copied().unwrap_or(0.0);
        let anger = traits.get("Colère").copied().unwrap_or(0.0);
        let despair = traits.get("Désespoir").copied().unwrap_or(0.0);
        let pride = traits.get("Fierté").copied().unwrap_or(0.0);
        let hope = traits.get("Espoir").copied().unwrap_or(0.0);

        // Build composite traits according to the combination formulas.
        let mut personality_traits = HashMap::new();
        // Optimism = sum of joy and serenity, capped at 1.0.
        personality_traits.insert("Optimisme".to_string(), (joy + serenity).min(1.0));
        // Curiosity = directly the frequency of the "Curiosite" emotion.
        personality_traits.insert("Curiosité".to_string(), curiosity);
        // Empathy = derived from tenderness and compassion.
        personality_traits.insert("Empathie".to_string(), (tenderness + compassion * 0.8).min(1.0));
        // Altruism = compassion + tenderness, capped at 1.0.
        personality_traits.insert("Altruisme".to_string(), (compassion + tenderness * 0.5).min(1.0));
        // Anxiety = directly the frequency of the "Anxiete" emotion.
        personality_traits.insert("Anxiété".to_string(), anxiety);
        // Stability = difference between serenity and anxiety, clamped to [0, 1].
        // A serene and non-anxious Saphire is considered stable.
        personality_traits.insert("Stabilité".to_string(), (serenity - anxiety).clamp(0.0, 1.0));
        // Resilience = hope + pride - despair, clamped to [0, 1].
        personality_traits.insert("Résilience".to_string(), (hope + pride * 0.5 - despair).clamp(0.0, 1.0));
        // Combativeness = channeled anger, capped.
        personality_traits.insert("Combativité".to_string(), (anger * 0.6).min(1.0));

        // Step 3: Identify the dominant trait and generate a description.
        // The dominant trait is the one with the highest score.
        let dominant_trait = personality_traits.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(k, _)| k.clone())
            .unwrap_or("Neutre".to_string());

        let description = format!(
            "Saphire est principalement caractérisée par son {}. \
             Basé sur {} souvenirs analysés.",
            dominant_trait.to_lowercase(),
            emotions.len()
        );

        // Merge raw emotional traits and composite personality traits
        // into a single table. Composite traits overwrite raw emotions
        // with the same name (e.g., "Curiosite" is replaced by the
        // composite "Curiosite" personality trait).
        for (k, v) in personality_traits {
            traits.insert(k, v);
        }

        EmergentPersonality {
            traits,
            description,
            memory_count: emotions.len() as u64,
        }
    }
}
