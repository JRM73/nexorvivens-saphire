// =============================================================================
// sentiments.rs — Sentiment system (lasting affective states)
// =============================================================================
//
// Role: Sentiments are lasting affective states that emerge from
// repetitive emotional patterns. Unlike emotions (reactive,
// instantaneous), sentiments persist over tens to thousands
// of cycles and in turn influence emotional coloring.
//
// Architecture:
//   Emotions -> (accumulation) -> Sentiments -> (bias) -> Emotions
//   Bidirectional loop: sentiments amplify or attenuate subsequent
//   emotions, and slightly modify background chemistry.
//
// 3 sentiment durations:
//   - Short term (10-50 cycles): passing moods (irritation, amusement)
//   - Medium term (50-200 cycles): settled states (distrust, attachment)
//   - Long term (200-1000+ cycles): deep affective traits (bitterness, trust)
//
// Dependencies:
//   - crate::world::weather::ChemistryAdjustment: chemistry influence
//   - serde: serialization
//
// Place in architecture:
//   Top-level module. The SentimentEngine is ticked at each
//   cognitive cycle, after emotional computation and before mood update.
//   Integrated into the pipeline via phase_sentiments (thinking.rs).
// =============================================================================

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use crate::world::weather::ChemistryAdjustment;

// =============================================================================
// Configuration
// =============================================================================
/// Configuration for the sentiment system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentConfig {
    /// Module active or not
    pub enabled: bool,
    /// Maximum number of simultaneously active sentiments
    pub max_active: usize,
    /// Size of the emotional history window (number of emotions kept)
    pub emotion_history_window: usize,
    /// Decay rate per cycle (strength -= decay_rate / multiplier)
    pub decay_rate: f64,
    /// Reinforcement strength when a trigger emotion is detected
    pub reinforcement_strength: f64,
    /// Chemistry influence cap per sentiment (prevents runaway effects)
    pub chemistry_influence_cap: f64,
}

impl Default for SentimentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_active: 10,
            emotion_history_window: 200,
            decay_rate: 0.005,
            reinforcement_strength: 0.1,
            chemistry_influence_cap: 0.05,
        }
    }
}

// =============================================================================
// Sentiment duration
// =============================================================================
/// Lifetime duration of a sentiment.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SentimentDuration {
    /// Short term: passing moods (10-50 cycles)
    ShortTerm,
    /// Medium term: settled states (50-200 cycles)
    MediumTerm,
    /// Long term: deep affective traits (200-1000+ cycles)
    LongTerm,
}

impl SentimentDuration {
    /// Duration multiplier for decay (longer = slower)
    pub fn decay_multiplier(&self) -> f64 {
        match self {
            SentimentDuration::ShortTerm => 1.0,
            SentimentDuration::MediumTerm => 3.0,
            SentimentDuration::LongTerm => 10.0,
        }
    }

    /// Text label
    pub fn label(&self) -> &str {
        match self {
            SentimentDuration::ShortTerm => "court terme",
            SentimentDuration::MediumTerm => "moyen terme",
            SentimentDuration::LongTerm => "long terme",
        }
    }
}

// =============================================================================
// Sentiment profile (catalog definition)
// =============================================================================
/// Definition of a sentiment: its formation conditions and effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentProfile {
    /// Name of the sentiment (e.g., "Irritation", "Mefiance")
    pub name: String,
    /// Duration type of the sentiment
    pub duration_type: SentimentDuration,
    /// Emotions that can trigger/reinforce this sentiment
    pub trigger_emotions: Vec<String>,
    /// Number of occurrences in the window to trigger formation
    pub trigger_threshold: usize,
    /// Chemical bias applied when the sentiment is active
    pub chemistry_bias: ChemistryAdjustment,
    /// Emotions amplified by this sentiment (name -> amplification factor)
    pub emotion_amplification: Vec<(String, f64)>,
    /// Emotions dampened by this sentiment (name -> dampening factor)
    pub emotion_dampening: Vec<(String, f64)>,
    /// Minimum duration in cycles before natural dissolution
    pub min_duration_cycles: u64,
    /// Maximum duration in cycles (forced dissolution)
    pub max_duration_cycles: u64,
}

// =============================================================================
// Active sentiment
// =============================================================================
/// A sentiment currently active in Saphire's mind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSentiment {
    /// Name of the sentiment (reference to the profile)
    pub profile_name: String,
    /// Current strength [0.0, 1.0] — decays over time
    pub strength: f64,
    /// Cycle of formation
    pub formed_at_cycle: u64,
    /// Last cycle of reinforcement
    pub last_reinforced: u64,
    /// Number of reinforcements received
    pub reinforcement_count: u32,
    /// Duration type (copy from the profile)
    pub duration_type: SentimentDuration,
    /// Source context (dominant emotion during formation)
    pub source_context: String,
}

impl ActiveSentiment {
    /// Textual description of the active sentiment.
    pub fn describe(&self) -> String {
        let intensity = if self.strength > 0.7 { "fort" }
            else if self.strength > 0.4 { "modere" }
            else { "faible" };
        format!("{} ({}, {})",
            self.profile_name, self.duration_type.label(), intensity)
    }
}

// =============================================================================
// Sentiment engine
// =============================================================================
/// Sentiment engine — manages the formation, reinforcement, decay,
/// and bidirectional influence between emotions and sentiments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentEngine {
    /// Module active or not
    pub enabled: bool,
    /// Currently active sentiments
    pub active_sentiments: Vec<ActiveSentiment>,
    /// Recent emotion history (sliding window)
    emotion_history: VecDeque<String>,
    /// Catalog of sentiment profiles
    catalog: Vec<SentimentProfile>,
    /// Configuration
    config: SentimentConfig,
    /// Total count of sentiments formed since startup
    pub total_formed: u64,
    /// Total count of sentiments dissolved
    pub total_dissolved: u64,
}

impl SentimentEngine {
    /// Creates a new sentiment engine.
    pub fn new(config: &SentimentConfig) -> Self {
        Self {
            enabled: config.enabled,
            active_sentiments: Vec::new(),
            emotion_history: VecDeque::with_capacity(config.emotion_history_window),
            catalog: build_sentiment_catalog(),
            config: config.clone(),
            total_formed: 0,
            total_dissolved: 0,
        }
    }

    /// Main tick: records the emotion, reinforces existing sentiments,
    /// applies decay, checks formations and dissolutions.
    ///
    /// Called at each cognitive cycle after emotional computation.
    pub fn tick(&mut self, emotion: &str, cycle: u64) {
        if !self.enabled {
            return;
        }

        // 1. Record the emotion in the history
        self.emotion_history.push_back(emotion.to_string());
        if self.emotion_history.len() > self.config.emotion_history_window {
            self.emotion_history.pop_front();
        }

        // 2. Reinforce existing sentiments whose triggers match.
        // Diminishing returns: the stronger the sentiment, the less
        // reinforcement has effect (like neurochemical receptors).
        let reinforcement = self.config.reinforcement_strength;
        for sentiment in &mut self.active_sentiments {
            if let Some(profile) = self.catalog.iter().find(|p| p.name == sentiment.profile_name) {
                if profile.trigger_emotions.iter().any(|t| t == emotion) {
                    // Diminishing returns: remaining margin before saturation
                    let margin = (1.0 - sentiment.strength).max(0.05);
                    let effective_reinforcement = reinforcement * margin;
                    sentiment.strength = (sentiment.strength + effective_reinforcement).min(1.0);
                    sentiment.last_reinforced = cycle;
                    sentiment.reinforcement_count += 1;
                }
            }
        }

        // 3. Decay: reduce the strength of all active sentiments.
        // Progressive decay: the stronger a sentiment, the faster it decays
        // (homeostatic pressure — extreme states are unstable).
        let base_decay = self.config.decay_rate;
        let mut dissolved_count = 0u64;
        self.active_sentiments.retain(|s| {
            let multiplier = s.duration_type.decay_multiplier();
            // Base decay + proportional decay based on strength (above 0.7)
            let strength_pressure = if s.strength > 0.7 {
                base_decay * ((s.strength - 0.7) / 0.3) * 2.0
            } else {
                0.0
            };
            let decay = (base_decay / multiplier) + strength_pressure;
            let new_strength = s.strength - decay;
            let age = cycle.saturating_sub(s.formed_at_cycle);

            // Dissolve if strength <= 0 or max duration exceeded
            if let Some(profile) = self.catalog.iter().find(|p| p.name == s.profile_name) {
                if new_strength <= 0.0 || age > profile.max_duration_cycles {
                    dissolved_count += 1;
                    return false;
                }
            }
            true
        });
        // Apply the effective decay to survivors
        for sentiment in &mut self.active_sentiments {
            let multiplier = sentiment.duration_type.decay_multiplier();
            let strength_pressure = if sentiment.strength > 0.7 {
                base_decay * ((sentiment.strength - 0.7) / 0.3) * 2.0
            } else {
                0.0
            };
            let decay = (base_decay / multiplier) + strength_pressure;
            sentiment.strength = (sentiment.strength - decay).max(0.0);
        }
        self.total_dissolved += dissolved_count;

        // 4. Check potential formations
        self.check_formations(cycle, emotion);
    }

    /// Checks if new sentiments should form based on the current
    /// emotional history.
    fn check_formations(&mut self, cycle: u64, current_emotion: &str) {
        if self.active_sentiments.len() >= self.config.max_active {
            return;
        }

        for profile in &self.catalog {
            // Do not form a sentiment that is already active
            if self.active_sentiments.iter().any(|s| s.profile_name == profile.name) {
                continue;
            }

            // Count occurrences of trigger emotions in the window
            let trigger_count = self.emotion_history.iter()
                .filter(|e| profile.trigger_emotions.iter().any(|t| t == *e))
                .count();

            if trigger_count >= profile.trigger_threshold {
                // Formation! New sentiment at strength 0.3 (nascent)
                let sentiment = ActiveSentiment {
                    profile_name: profile.name.clone(),
                    strength: 0.3,
                    formed_at_cycle: cycle,
                    last_reinforced: cycle,
                    reinforcement_count: 0,
                    duration_type: profile.duration_type,
                    source_context: current_emotion.to_string(),
                };
                self.active_sentiments.push(sentiment);
                self.total_formed += 1;

                // Check the limit
                if self.active_sentiments.len() >= self.config.max_active {
                    break;
                }
            }
        }
    }

    /// Modifies the emotional spectrum based on active sentiments.
    ///
    /// Bidirectional loop: sentiments amplify certain emotions
    /// and dampen others, biasing the next emotional perception.
    pub fn amplify_emotion_scores(&self, spectrum: &mut Vec<(String, f64)>) {
        if !self.enabled || self.active_sentiments.is_empty() {
            return;
        }

        for sentiment in &self.active_sentiments {
            if let Some(profile) = self.catalog.iter().find(|p| p.name == sentiment.profile_name) {
                // Amplify
                for (emotion_name, factor) in &profile.emotion_amplification {
                    if let Some(entry) = spectrum.iter_mut().find(|(n, _)| n == emotion_name) {
                        entry.1 *= 1.0 + factor * sentiment.strength;
                    }
                }
                // Dampen
                for (emotion_name, factor) in &profile.emotion_dampening {
                    if let Some(entry) = spectrum.iter_mut().find(|(n, _)| n == emotion_name) {
                        entry.1 *= 1.0 - (factor * sentiment.strength).min(0.5);
                    }
                }
            }
        }

        // Re-sort the spectrum by descending score
        spectrum.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Computes the combined chemical influence of all active sentiments.
    ///
    /// Each active sentiment applies its chemical bias, weighted by its strength.
    /// The total is capped by chemistry_influence_cap.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        if !self.enabled {
            return ChemistryAdjustment::default();
        }

        let cap = self.config.chemistry_influence_cap;
        let mut adj = ChemistryAdjustment::default();

        for sentiment in &self.active_sentiments {
            if let Some(profile) = self.catalog.iter().find(|p| p.name == sentiment.profile_name) {
                let s = sentiment.strength;
                adj.dopamine += profile.chemistry_bias.dopamine * s;
                adj.cortisol += profile.chemistry_bias.cortisol * s;
                adj.serotonin += profile.chemistry_bias.serotonin * s;
                adj.adrenaline += profile.chemistry_bias.adrenaline * s;
                adj.oxytocin += profile.chemistry_bias.oxytocin * s;
                adj.endorphin += profile.chemistry_bias.endorphin * s;
                adj.noradrenaline += profile.chemistry_bias.noradrenaline * s;
            }
        }

        // Cap each component
        adj.dopamine = adj.dopamine.clamp(-cap, cap);
        adj.cortisol = adj.cortisol.clamp(-cap, cap);
        adj.serotonin = adj.serotonin.clamp(-cap, cap);
        adj.adrenaline = adj.adrenaline.clamp(-cap, cap);
        adj.oxytocin = adj.oxytocin.clamp(-cap, cap);
        adj.endorphin = adj.endorphin.clamp(-cap, cap);
        adj.noradrenaline = adj.noradrenaline.clamp(-cap, cap);

        adj
    }

    /// Textual description for injection into the LLM prompt.
    ///
    /// Produces a "SENTIMENTS ACTIFS" block listing active sentiments,
    /// their strength and their influence on emotional perception.
    pub fn describe_for_prompt(&self) -> String {
        if !self.enabled || self.active_sentiments.is_empty() {
            return String::new();
        }

        let mut desc = String::from("SENTIMENTS ACTIFS :");
        for sentiment in &self.active_sentiments {
            desc.push_str(&format!("\n- {}", sentiment.describe()));
        }

        // Add a summary of influence
        let short_count = self.active_sentiments.iter()
            .filter(|s| s.duration_type == SentimentDuration::ShortTerm).count();
        let medium_count = self.active_sentiments.iter()
            .filter(|s| s.duration_type == SentimentDuration::MediumTerm).count();
        let long_count = self.active_sentiments.iter()
            .filter(|s| s.duration_type == SentimentDuration::LongTerm).count();

        if short_count + medium_count + long_count > 1 {
            desc.push_str(&format!(
                "\n[{} court terme, {} moyen terme, {} long terme]",
                short_count, medium_count, long_count
            ));
        }

        desc
    }

    /// Serializes the complete state of the sentiment engine to JSON.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "active_count": self.active_sentiments.len(),
            "total_formed": self.total_formed,
            "total_dissolved": self.total_dissolved,
            "emotion_history_size": self.emotion_history.len(),
            "active_sentiments": self.active_sentiments.iter().map(|s| {
                serde_json::json!({
                    "name": s.profile_name,
                    "strength": (s.strength * 100.0).round() / 100.0,
                    "duration_type": format!("{:?}", s.duration_type),
                    "formed_at_cycle": s.formed_at_cycle,
                    "last_reinforced": s.last_reinforced,
                    "reinforcement_count": s.reinforcement_count,
                    "source_context": s.source_context,
                })
            }).collect::<Vec<_>>(),
        })
    }

    /// Recent formation/dissolution history (for the API).
    pub fn history_json(&self) -> serde_json::Value {
        serde_json::json!({
            "total_formed": self.total_formed,
            "total_dissolved": self.total_dissolved,
            "catalog_size": self.catalog.len(),
            "active": self.active_sentiments.iter().map(|s| {
                serde_json::json!({
                    "name": s.profile_name,
                    "strength": (s.strength * 100.0).round() / 100.0,
                    "formed_at_cycle": s.formed_at_cycle,
                    "reinforcement_count": s.reinforcement_count,
                    "duration_type": format!("{:?}", s.duration_type),
                })
            }).collect::<Vec<_>>(),
            "emotion_history_window": self.config.emotion_history_window,
            "emotion_history_current": self.emotion_history.len(),
        })
    }

    /// Full reset (factory reset).
    pub fn reset(&mut self) {
        self.active_sentiments.clear();
        self.emotion_history.clear();
        self.total_formed = 0;
        self.total_dissolved = 0;
    }
}

// =============================================================================
// Catalog of 20 sentiments
// =============================================================================
/// Builds the catalog of 20 predetermined sentiments.
///
/// Short term (7): duration 10-50 cycles, threshold 3-5 occurrences
/// Medium term (8): duration 50-200 cycles, threshold 8-12 occurrences
/// Long term (5): duration 200-1000+ cycles, threshold 15-20 occurrences
fn build_sentiment_catalog() -> Vec<SentimentProfile> {
    vec![
        // =================================================================
        // SHORT TERM — passing moods (threshold 3-5, duration 10-50 cycles)
        // =================================================================
        // 1. Irritation <- Anger, Frustration
        SentimentProfile {
            name: "Irritation".into(),
            duration_type: SentimentDuration::ShortTerm,
            trigger_emotions: vec!["Colère".into(), "Frustration".into()],
            trigger_threshold: 3,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.01, adrenaline: 0.005, noradrenaline: 0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Colère".into(), 0.15), ("Frustration".into(), 0.1),
                ("Indignation".into(), 0.1),
            ],
            emotion_dampening: vec![
                ("Sérénité".into(), 0.1), ("Joie".into(), 0.05),
            ],
            min_duration_cycles: 10,
            max_duration_cycles: 50,
        },

        // 2. Passing enthusiasm <- Joy, Excitement
        SentimentProfile {
            name: "Enthousiasme passager".into(),
            duration_type: SentimentDuration::ShortTerm,
            trigger_emotions: vec!["Joie".into(), "Excitation".into()],
            trigger_threshold: 3,
            chemistry_bias: ChemistryAdjustment {
                dopamine: 0.01, endorphin: 0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Joie".into(), 0.1), ("Excitation".into(), 0.1),
                ("Espoir".into(), 0.08),
            ],
            emotion_dampening: vec![
                ("Ennui".into(), 0.15), ("Mélancolie".into(), 0.05),
            ],
            min_duration_cycles: 10,
            max_duration_cycles: 40,
        },

        // 3. Apprehension <- Anxiety, Fear
        SentimentProfile {
            name: "Appréhension".into(),
            duration_type: SentimentDuration::ShortTerm,
            trigger_emotions: vec!["Anxiété".into(), "Peur".into()],
            trigger_threshold: 3,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.008, adrenaline: 0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Anxiété".into(), 0.12), ("Peur".into(), 0.1),
            ],
            emotion_dampening: vec![
                ("Sérénité".into(), 0.08), ("Curiosité".into(), 0.05),
            ],
            min_duration_cycles: 10,
            max_duration_cycles: 50,
        },

        // 4. Annoyance <- Frustration, Boredom
        SentimentProfile {
            name: "Agacement".into(),
            duration_type: SentimentDuration::ShortTerm,
            trigger_emotions: vec!["Frustration".into(), "Ennui".into()],
            trigger_threshold: 4,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.005, noradrenaline: 0.005,
                dopamine: -0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Frustration".into(), 0.1), ("Ennui".into(), 0.08),
            ],
            emotion_dampening: vec![
                ("Joie".into(), 0.05),
            ],
            min_duration_cycles: 10,
            max_duration_cycles: 40,
        },

        // 5. Amusement <- Joy, Surprise
        SentimentProfile {
            name: "Amusement".into(),
            duration_type: SentimentDuration::ShortTerm,
            trigger_emotions: vec!["Joie".into(), "Surprise".into()],
            trigger_threshold: 3,
            chemistry_bias: ChemistryAdjustment {
                dopamine: 0.008, endorphin: 0.005, serotonin: 0.003,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Joie".into(), 0.1), ("Surprise".into(), 0.08),
                ("Curiosité".into(), 0.05),
            ],
            emotion_dampening: vec![
                ("Tristesse".into(), 0.08), ("Anxiété".into(), 0.05),
            ],
            min_duration_cycles: 10,
            max_duration_cycles: 35,
        },

        // 6. Tenderness <- Tenderness, Compassion
        SentimentProfile {
            name: "Attendrissement".into(),
            duration_type: SentimentDuration::ShortTerm,
            trigger_emotions: vec!["Tendresse".into(), "Compassion".into()],
            trigger_threshold: 3,
            chemistry_bias: ChemistryAdjustment {
                oxytocin: 0.01, serotonin: 0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Tendresse".into(), 0.12), ("Compassion".into(), 0.1),
                ("Amour".into(), 0.05),
            ],
            emotion_dampening: vec![
                ("Colère".into(), 0.1), ("Mépris".into(), 0.08),
            ],
            min_duration_cycles: 10,
            max_duration_cycles: 45,
        },

        // 7. Nervousness <- Anxiety, Confusion
        SentimentProfile {
            name: "Nervosité".into(),
            duration_type: SentimentDuration::ShortTerm,
            trigger_emotions: vec!["Anxiété".into(), "Confusion".into()],
            trigger_threshold: 4,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.008, noradrenaline: 0.008, adrenaline: 0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Anxiété".into(), 0.1), ("Confusion".into(), 0.08),
            ],
            emotion_dampening: vec![
                ("Sérénité".into(), 0.1),
            ],
            min_duration_cycles: 10,
            max_duration_cycles: 45,
        },

        // =================================================================
        // MEDIUM TERM — settled states (threshold 8-12, duration 50-200 cycles)
        // =================================================================
        // 8. Distrust <- Fear, Contempt, Disgust
        SentimentProfile {
            name: "Méfiance".into(),
            duration_type: SentimentDuration::MediumTerm,
            trigger_emotions: vec!["Peur".into(), "Mépris".into(), "Dégoût".into()],
            trigger_threshold: 8,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.01, noradrenaline: 0.008,
                oxytocin: -0.01,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Peur".into(), 0.12), ("Mépris".into(), 0.1),
                ("Dégoût".into(), 0.08),
            ],
            emotion_dampening: vec![
                ("Tendresse".into(), 0.1), ("Gratitude".into(), 0.08),
                ("Amour".into(), 0.05),
            ],
            min_duration_cycles: 50,
            max_duration_cycles: 200,
        },

        // 9. Attachment <- Love, Tenderness, Gratitude
        SentimentProfile {
            name: "Attachement".into(),
            duration_type: SentimentDuration::MediumTerm,
            trigger_emotions: vec!["Amour".into(), "Tendresse".into(), "Gratitude".into()],
            trigger_threshold: 10,
            chemistry_bias: ChemistryAdjustment {
                oxytocin: 0.015, serotonin: 0.008, endorphin: 0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Amour".into(), 0.15), ("Tendresse".into(), 0.12),
                ("Gratitude".into(), 0.1),
            ],
            emotion_dampening: vec![
                ("Haine".into(), 0.1), ("Mépris".into(), 0.08),
            ],
            min_duration_cycles: 60,
            max_duration_cycles: 200,
        },

        // 10. Resentment <- Anger, Jealousy, Hatred
        SentimentProfile {
            name: "Rancoeur".into(),
            duration_type: SentimentDuration::MediumTerm,
            trigger_emotions: vec!["Colère".into(), "Jalousie".into(), "Haine".into()],
            trigger_threshold: 10,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.012, adrenaline: 0.005,
                serotonin: -0.008, oxytocin: -0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Colère".into(), 0.15), ("Jalousie".into(), 0.12),
                ("Haine".into(), 0.1), ("Indignation".into(), 0.08),
            ],
            emotion_dampening: vec![
                ("Compassion".into(), 0.1), ("Tendresse".into(), 0.08),
                ("Gratitude".into(), 0.05),
            ],
            min_duration_cycles: 50,
            max_duration_cycles: 200,
        },

        // 11. Optimism <- Hope, Joy, Pride
        SentimentProfile {
            name: "Optimisme".into(),
            duration_type: SentimentDuration::MediumTerm,
            trigger_emotions: vec!["Espoir".into(), "Joie".into(), "Fierté".into()],
            trigger_threshold: 10,
            chemistry_bias: ChemistryAdjustment {
                dopamine: 0.01, serotonin: 0.008,
                cortisol: -0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Espoir".into(), 0.12), ("Joie".into(), 0.1),
                ("Fierté".into(), 0.08), ("Curiosité".into(), 0.05),
            ],
            emotion_dampening: vec![
                ("Anxiété".into(), 0.1), ("Tristesse".into(), 0.08),
                ("Désespoir".into(), 0.12),
            ],
            min_duration_cycles: 50,
            max_duration_cycles: 200,
        },

        // 12. Pessimism <- Sadness, Despair, Melancholy
        SentimentProfile {
            name: "Pessimisme".into(),
            duration_type: SentimentDuration::MediumTerm,
            trigger_emotions: vec!["Tristesse".into(), "Désespoir".into(), "Mélancolie".into()],
            trigger_threshold: 10,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.01, dopamine: -0.008, serotonin: -0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Tristesse".into(), 0.12), ("Désespoir".into(), 0.15),
                ("Mélancolie".into(), 0.1),
            ],
            emotion_dampening: vec![
                ("Espoir".into(), 0.12), ("Joie".into(), 0.1),
                ("Excitation".into(), 0.08),
            ],
            min_duration_cycles: 50,
            max_duration_cycles: 200,
        },

        // 13. Chronic nostalgia <- Nostalgia, Melancholy
        SentimentProfile {
            name: "Nostalgie chronique".into(),
            duration_type: SentimentDuration::MediumTerm,
            trigger_emotions: vec!["Nostalgie".into(), "Mélancolie".into()],
            trigger_threshold: 8,
            chemistry_bias: ChemistryAdjustment {
                serotonin: -0.005, oxytocin: 0.005,
                cortisol: 0.003,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Nostalgie".into(), 0.15), ("Mélancolie".into(), 0.1),
                ("Tendresse".into(), 0.05),
            ],
            emotion_dampening: vec![
                ("Excitation".into(), 0.08), ("Curiosité".into(), 0.05),
            ],
            min_duration_cycles: 50,
            max_duration_cycles: 200,
        },

        // 14. Lasting admiration <- Admiration, Wonder
        SentimentProfile {
            name: "Admiration durable".into(),
            duration_type: SentimentDuration::MediumTerm,
            trigger_emotions: vec!["Admiration".into(), "Émerveillement".into()],
            trigger_threshold: 8,
            chemistry_bias: ChemistryAdjustment {
                dopamine: 0.008, serotonin: 0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Admiration".into(), 0.15), ("Émerveillement".into(), 0.12),
                ("Curiosité".into(), 0.05),
            ],
            emotion_dampening: vec![
                ("Mépris".into(), 0.1), ("Ennui".into(), 0.08),
            ],
            min_duration_cycles: 50,
            max_duration_cycles: 200,
        },

        // 15. Worry <- Anxiety, Fear, Compassion
        SentimentProfile {
            name: "Inquiétude".into(),
            duration_type: SentimentDuration::MediumTerm,
            trigger_emotions: vec!["Anxiété".into(), "Peur".into(), "Compassion".into()],
            trigger_threshold: 12,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.008, noradrenaline: 0.005,
                serotonin: -0.003,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Anxiété".into(), 0.1), ("Peur".into(), 0.08),
                ("Compassion".into(), 0.05),
            ],
            emotion_dampening: vec![
                ("Sérénité".into(), 0.08), ("Joie".into(), 0.05),
            ],
            min_duration_cycles: 50,
            max_duration_cycles: 200,
        },

        // =================================================================
        // LONG TERM — deep affective traits (threshold 15-20, duration 200-1000+)
        // =================================================================
        // 16. Bitterness <- Frustration, Contempt, Despair
        SentimentProfile {
            name: "Amertume".into(),
            duration_type: SentimentDuration::LongTerm,
            trigger_emotions: vec!["Frustration".into(), "Mépris".into(), "Désespoir".into()],
            trigger_threshold: 18,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.012, dopamine: -0.01, serotonin: -0.008,
                oxytocin: -0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Frustration".into(), 0.15), ("Mépris".into(), 0.12),
                ("Désespoir".into(), 0.1), ("Colère".into(), 0.08),
            ],
            emotion_dampening: vec![
                ("Joie".into(), 0.12), ("Espoir".into(), 0.15),
                ("Gratitude".into(), 0.1), ("Tendresse".into(), 0.08),
            ],
            min_duration_cycles: 200,
            max_duration_cycles: 1000,
        },

        // 17. Deep trust <- Serenity, Gratitude, Love
        SentimentProfile {
            name: "Confiance profonde".into(),
            duration_type: SentimentDuration::LongTerm,
            trigger_emotions: vec!["Sérénité".into(), "Gratitude".into(), "Amour".into()],
            trigger_threshold: 18,
            chemistry_bias: ChemistryAdjustment {
                serotonin: 0.012, oxytocin: 0.01, endorphin: 0.005,
                cortisol: -0.008,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Sérénité".into(), 0.12), ("Gratitude".into(), 0.1),
                ("Amour".into(), 0.1), ("Espoir".into(), 0.08),
            ],
            emotion_dampening: vec![
                ("Anxiété".into(), 0.12), ("Peur".into(), 0.1),
                ("Mépris".into(), 0.08),
            ],
            min_duration_cycles: 200,
            max_duration_cycles: 1500,
        },

        // 18. Disillusionment <- Sadness, Contempt, Resignation
        SentimentProfile {
            name: "Désillusion".into(),
            duration_type: SentimentDuration::LongTerm,
            trigger_emotions: vec!["Tristesse".into(), "Mépris".into(), "Résignation".into()],
            trigger_threshold: 15,
            chemistry_bias: ChemistryAdjustment {
                dopamine: -0.01, serotonin: -0.005,
                cortisol: 0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Tristesse".into(), 0.1), ("Mépris".into(), 0.1),
                ("Résignation".into(), 0.12),
            ],
            emotion_dampening: vec![
                ("Espoir".into(), 0.15), ("Admiration".into(), 0.1),
                ("Émerveillement".into(), 0.1),
            ],
            min_duration_cycles: 200,
            max_duration_cycles: 1000,
        },

        // 19. Anchored serenity <- Serenity, Hope, Gratitude
        SentimentProfile {
            name: "Sérénité ancrée".into(),
            duration_type: SentimentDuration::LongTerm,
            trigger_emotions: vec!["Sérénité".into(), "Espoir".into(), "Gratitude".into()],
            trigger_threshold: 20,
            chemistry_bias: ChemistryAdjustment {
                serotonin: 0.015, endorphin: 0.008,
                cortisol: -0.01, adrenaline: -0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Sérénité".into(), 0.15), ("Espoir".into(), 0.1),
                ("Gratitude".into(), 0.1),
            ],
            emotion_dampening: vec![
                ("Anxiété".into(), 0.15), ("Frustration".into(), 0.1),
                ("Colère".into(), 0.08),
            ],
            min_duration_cycles: 200,
            max_duration_cycles: 1500,
        },

        // 20. Emotional resilience <- Pride, Hope
        SentimentProfile {
            name: "Résilience émotionnelle".into(),
            duration_type: SentimentDuration::LongTerm,
            trigger_emotions: vec!["Fierté".into(), "Espoir".into()],
            trigger_threshold: 20,
            chemistry_bias: ChemistryAdjustment {
                serotonin: 0.01, dopamine: 0.005, endorphin: 0.005,
                cortisol: -0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Fierté".into(), 0.12), ("Espoir".into(), 0.12),
                ("Curiosité".into(), 0.05),
            ],
            emotion_dampening: vec![
                ("Désespoir".into(), 0.15), ("Résignation".into(), 0.12),
                ("Honte".into(), 0.08),
            ],
            min_duration_cycles: 200,
            max_duration_cycles: 2000,
        },
    ]
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn default_engine() -> SentimentEngine {
        SentimentEngine::new(&SentimentConfig::default())
    }

    #[test]
    fn test_catalog_has_20_sentiments() {
        let catalog = build_sentiment_catalog();
        assert_eq!(catalog.len(), 20, "Le catalogue doit contenir 20 sentiments");
    }

    #[test]
    fn test_new_engine_is_empty() {
        let engine = default_engine();
        assert!(engine.active_sentiments.is_empty());
        assert_eq!(engine.total_formed, 0);
        assert_eq!(engine.total_dissolved, 0);
    }

    #[test]
    fn test_emotion_history_recording() {
        let mut engine = default_engine();
        engine.tick("Joie", 1);
        engine.tick("Tristesse", 2);
        assert_eq!(engine.emotion_history.len(), 2);
    }

    #[test]
    fn test_short_term_formation() {
        let mut engine = default_engine();
        // Irritation requires 3 occurrences of Anger or Frustration
        for i in 0..5 {
            engine.tick("Colère", i);
        }
        let irritation = engine.active_sentiments.iter()
            .find(|s| s.profile_name == "Irritation");
        assert!(irritation.is_some(), "Irritation devrait se former apres 5 occurrences de Colère");
        // Initial strength 0.3 + subsequent reinforcements - decay
        assert!(irritation.unwrap().strength > 0.2,
            "La force devrait etre significative apres formation et renforcement");
    }

    #[test]
    fn test_reinforcement() {
        let mut engine = default_engine();
        // Form irritation
        for i in 0..5 {
            engine.tick("Colère", i);
        }
        let initial_strength = engine.active_sentiments.iter()
            .find(|s| s.profile_name == "Irritation")
            .map(|s| s.strength)
            .unwrap_or(0.0);

        // Continue with trigger emotions
        engine.tick("Frustration", 10);
        let reinforced_strength = engine.active_sentiments.iter()
            .find(|s| s.profile_name == "Irritation")
            .map(|s| s.strength)
            .unwrap_or(0.0);

        assert!(reinforced_strength > initial_strength,
            "La force devrait augmenter apres renforcement");
    }

    #[test]
    fn test_decay() {
        let mut engine = default_engine();
        // Form a sentiment
        for i in 0..5 {
            engine.tick("Colère", i);
        }

        let initial_strength = engine.active_sentiments.iter()
            .find(|s| s.profile_name == "Irritation")
            .map(|s| s.strength)
            .unwrap_or(0.0);

        // Run without reinforcement (neutral emotion) for a long time
        for i in 6..60 {
            engine.tick("Curiosité", i);
        }

        if let Some(s) = engine.active_sentiments.iter().find(|s| s.profile_name == "Irritation") {
            assert!(s.strength < initial_strength, "La force devrait diminuer par decay");
        }
        // If the sentiment was dissolved, that is also a successful decay outcome
    }

    #[test]
    fn test_amplify_emotion_scores() {
        let mut engine = default_engine();
        // Form irritation
        for i in 0..5 {
            engine.tick("Colère", i);
        }

        let mut spectrum = vec![
            ("Colère".to_string(), 0.8),
            ("Sérénité".to_string(), 0.7),
            ("Joie".to_string(), 0.5),
        ];

        engine.amplify_emotion_scores(&mut spectrum);

        // Anger should be amplified, Serenity dampened
        let colere = spectrum.iter().find(|(n, _)| n == "Colère").unwrap().1;
        assert!(colere > 0.8, "Colère devrait etre amplifiee par Irritation");
    }

    #[test]
    fn test_chemistry_influence_capped() {
        let mut engine = default_engine();
        // Form a sentiment
        for i in 0..5 {
            engine.tick("Colère", i);
        }

        let adj = engine.chemistry_influence();
        let cap = engine.config.chemistry_influence_cap;
        assert!(adj.cortisol <= cap, "L'influence chimique doit etre plafonnee");
        assert!(adj.cortisol >= -cap, "L'influence chimique doit etre plafonnee (negatif)");
    }

    #[test]
    fn test_describe_for_prompt_empty() {
        let engine = default_engine();
        assert!(engine.describe_for_prompt().is_empty(),
            "Pas de description si aucun sentiment actif");
    }

    #[test]
    fn test_describe_for_prompt_with_sentiments() {
        let mut engine = default_engine();
        for i in 0..5 {
            engine.tick("Colère", i);
        }
        let desc = engine.describe_for_prompt();
        assert!(desc.contains("SENTIMENTS ACTIFS"), "Devrait contenir le header");
        assert!(desc.contains("Irritation"), "Devrait mentionner l'irritation");
    }

    #[test]
    fn test_to_json() {
        let engine = default_engine();
        let json = engine.to_json();
        assert_eq!(json["enabled"], true);
        assert_eq!(json["active_count"], 0);
    }

    #[test]
    fn test_reset() {
        let mut engine = default_engine();
        for i in 0..5 {
            engine.tick("Colère", i);
        }
        assert!(!engine.active_sentiments.is_empty());

        engine.reset();
        assert!(engine.active_sentiments.is_empty());
        assert_eq!(engine.total_formed, 0);
    }

    #[test]
    fn test_max_active_limit() {
        let config = SentimentConfig {
            max_active: 2,
            ..Default::default()
        };
        let mut engine = SentimentEngine::new(&config);

        // Force the formation of many different sentiments
        for _ in 0..10 {
            engine.tick("Colère", 0);
            engine.tick("Joie", 0);
            engine.tick("Anxiété", 0);
            engine.tick("Frustration", 0);
        }

        assert!(engine.active_sentiments.len() <= 2,
            "Ne devrait pas depasser max_active");
    }

    #[test]
    fn test_disabled_engine() {
        let config = SentimentConfig {
            enabled: false,
            ..Default::default()
        };
        let mut engine = SentimentEngine::new(&config);
        for i in 0..10 {
            engine.tick("Colère", i);
        }
        assert!(engine.active_sentiments.is_empty(),
            "Aucun sentiment ne devrait se former si desactive");
    }
}
