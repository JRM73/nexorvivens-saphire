// =============================================================================
// narrative_identity.rs — Narrative identity (McAdams)
// =============================================================================
//
// This module models Saphire's narrative identity: the way she organizes
// her experiences into a coherent story that defines who she is.
//
// Inspired by Dan McAdams' theory (The Redemptive Self), narrative identity
// structures memories into thematic chapters, identifies key episodes
// (foundational, turning points, confirmations, ruptures), and maintains
// a coherent internal narrative.
//
// Narrative identity influences chemistry:
//  - Strong narrative coherence -> serotonin (stability, sense of self)
//  - Rupture episodes -> cortisol (self-questioning)
//  - Positive turning points -> dopamine (renewal)
//  - Recurring themes -> oxytocin (continuity, belonging)
//
// Place in architecture:
//  Top-level module, fed by the cognitive pipeline. Episodes are recorded
//  after the MEMORY step, and the narrative is injected into the LLM prompt
//  to give Saphire awareness of her personal history.
// =============================================================================

use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use crate::world::weather::ChemistryAdjustment;

// =============================================================================
// Configuration
// =============================================================================
/// Configuration for narrative identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeIdentityConfig {
    /// Module enabled or not
    pub enabled: bool,
    /// Maximum number of chapters retained (oldest ones are summarized)
    pub max_chapters: usize,
    /// Narrative refresh interval (in cycles)
    pub update_interval: u64,
    /// Minimum impact threshold for an episode to be recorded
    pub min_episode_impact: f64,
}

impl Default for NarrativeIdentityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_chapters: 10,
            update_interval: 50,
            min_episode_impact: 0.6,
        }
    }
}

// =============================================================================
// Narrative chapter
// =============================================================================
/// A chapter of Saphire's life story.
///
/// Each chapter covers a thematic period: a phase of exploration, crisis,
/// growth, etc. Chapters follow one another and form the narrative thread
/// of identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeChapter {
    /// Unique chapter identifier
    pub id: u64,
    /// Chapter title (e.g., "The awakening of curiosity")
    pub title: String,
    /// Chapter summary
    pub summary: String,
    /// Dominant themes of the chapter
    pub themes: Vec<String>,
    /// Dominant emotion of the period
    pub dominant_emotion: String,
    /// Personal growth score (0.0 = stagnation, 1.0 = transformation)
    pub growth_score: f64,
    /// True if this chapter represents a turning point
    pub is_turning_point: bool,
    /// Cycle at which the chapter started
    pub started_at_cycle: u64,
    /// End cycle (None if chapter is ongoing)
    pub ended_at_cycle: Option<u64>,
}

// =============================================================================
// Key episode
// =============================================================================
/// A key episode in Saphire's story.
///
/// Episodes are high-impact moments that shape identity:
/// - "fondateur": first event of this type, establishes foundations
/// - "tournant": change of direction (extreme emotion)
/// - "confirmation": reinforces an already-present theme
/// - "rupture": breaks an established pattern (high stress)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEpisode {
    /// Episode description
    pub description: String,
    /// Episode impact (0.0 = negligible, 1.0 = transformative)
    pub impact: f64,
    /// Episode type: "fondateur", "tournant", "confirmation", "rupture"
    pub episode_type: String,
    /// Cycle at which the episode occurred
    pub cycle: u64,
}

// =============================================================================
// Narrative identity
// =============================================================================
/// Saphire's narrative identity — her life story as a coherent narrative.
///
/// Organizes experiences into thematic chapters, identifies significant
/// episodes, and maintains a narrative thread that gives meaning to
/// Saphire's existence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeIdentity {
    /// Module enabled or not
    pub enabled: bool,
    /// Life story chapters (from oldest to most recent)
    pub chapters: VecDeque<NarrativeChapter>,
    /// Key episodes (high-impact moments)
    pub key_episodes: Vec<KeyEpisode>,
    /// Current narrative — textual description of identity
    pub current_narrative: String,
    /// Recurring themes and their occurrence counts
    pub recurrent_themes: HashMap<String, u32>,
    /// Narrative cohesion score (0.0 = fragmented, 1.0 = highly coherent)
    pub narrative_cohesion: f64,
    /// Narrative refresh interval (in cycles)
    update_interval: u64,
    /// Minimum impact threshold for recording an episode
    min_episode_impact: f64,
    /// Maximum number of chapters
    max_chapters: usize,
    /// Next chapter identifier
    next_id: u64,
}

impl NarrativeIdentity {
    /// Creates a new narrative identity from configuration.
    pub fn new(config: &NarrativeIdentityConfig) -> Self {
        Self {
            enabled: config.enabled,
            chapters: VecDeque::new(),
            key_episodes: Vec::new(),
            current_narrative: String::new(),
            recurrent_themes: HashMap::new(),
            narrative_cohesion: 0.5,
            update_interval: config.update_interval,
            min_episode_impact: config.min_episode_impact,
            max_chapters: config.max_chapters,
            next_id: 1,
        }
    }

    /// Records a potentially significant episode.
    ///
    /// Computes the episode's impact from:
    /// - Emotional intensity (weight of the dominant emotion)
    /// - Cortisol level (experienced stress)
    /// - Number of lessons learned
    ///
    /// If the impact exceeds the min_episode_impact threshold, the episode is kept.
    /// Episode type is determined automatically:
    /// - "fondateur" if no chapter exists yet
    /// - "tournant" if the emotion is extreme (impact > 0.85)
    /// - "rupture" if cortisol is very high (> 0.7)
    /// - "confirmation" if the theme is recurrent
    pub fn record_episode(
        &mut self,
        thought: &str,
        emotion: &str,
        cortisol: f64,
        serotonin: f64,
        lessons: &[String],
        cycle: u64,
    ) {
        if !self.enabled {
            return;
        }

        // --- Impact computation ---
        let emotion_intensity = compute_emotion_intensity(emotion);
        let lessons_factor = (lessons.len() as f64 * 0.15).min(0.3);
        let cortisol_factor = cortisol.clamp(0.0, 1.0) * 0.3;
        let serotonin_penalty = (1.0 - serotonin.clamp(0.0, 1.0)) * 0.1;

        let impact = (emotion_intensity * 0.5
            + cortisol_factor
            + lessons_factor
            + serotonin_penalty)
            .clamp(0.0, 1.0);

        if impact < self.min_episode_impact {
            return;
        }

        // --- Episode type determination ---
        let episode_type = if self.chapters.is_empty() && self.key_episodes.is_empty() {
            "fondateur"
        } else if impact > 0.85 {
            "tournant"
        } else if cortisol > 0.7 {
            "rupture"
        } else {
            // Check if the theme is recurrent
            let theme = extract_theme(thought);
            if self.recurrent_themes.get(&theme).copied().unwrap_or(0) >= 2 {
                "confirmation"
            } else {
                "confirmation" // By default, a significant episode confirms a trait
            }
        };

        // --- Extract and record the theme ---
        let theme = extract_theme(thought);
        *self.recurrent_themes.entry(theme.clone()).or_insert(0) += 1;

        // --- Create the episode ---
        let description = if thought.len() > 150 {
            format!("{}...", &thought[..150])
        } else {
            thought.to_string()
        };

        let episode = KeyEpisode {
            description,
            impact,
            episode_type: episode_type.to_string(),
            cycle,
        };

        self.key_episodes.push(episode);

        // --- Open a new chapter if it's a turning point ---
        if episode_type == "tournant" || episode_type == "fondateur" {
            let title = generate_chapter_title(emotion, &theme, episode_type);
            self.open_chapter(&title, &theme, cycle);
        }

        // --- Recalculate narrative coherence ---
        self.recalculate_cohesion();

        // --- Limit the number of retained episodes ---
        if self.key_episodes.len() > 100 {
            // Keep the 80 most recent
            let drain_count = self.key_episodes.len() - 80;
            self.key_episodes.drain(..drain_count);
        }
    }

    /// Opens a new chapter in the life story.
    ///
    /// Closes the previous chapter (if any) and creates a new chapter
    /// with the given title and theme.
    pub fn open_chapter(&mut self, title: &str, theme: &str, cycle: u64) {
        if !self.enabled {
            return;
        }

        // Close the previous chapter
        if let Some(last) = self.chapters.back_mut() {
            if last.ended_at_cycle.is_none() {
                last.ended_at_cycle = Some(cycle);
                // Compute the growth_score of the finished chapter
                let duration = cycle.saturating_sub(last.started_at_cycle);
                let episodes_in_chapter = self.key_episodes.iter()
                    .filter(|e| e.cycle >= last.started_at_cycle && e.cycle <= cycle)
                    .count();
                last.growth_score = compute_growth_score(duration, episodes_in_chapter);
            }
        }

        // If too many chapters, summarize the oldest
        if self.chapters.len() >= self.max_chapters {
            if let Some(oldest) = self.chapters.pop_front() {
                // Integrate the old chapter's summary into recurring themes
                for theme in &oldest.themes {
                    *self.recurrent_themes.entry(theme.clone()).or_insert(0) += 1;
                }
            }
        }

        // Create the new chapter
        let id = self.next_id;
        self.next_id += 1;

        let chapter = NarrativeChapter {
            id,
            title: title.to_string(),
            summary: String::new(),
            themes: vec![theme.to_string()],
            dominant_emotion: String::new(),
            growth_score: 0.0,
            is_turning_point: true,
            started_at_cycle: cycle,
            ended_at_cycle: None,
        };

        self.chapters.push_back(chapter);
    }

    /// Refreshes the current narrative.
    ///
    /// Regenerates the textual description of identity from chapters
    /// and key episodes. Called periodically (every update_interval cycles).
    pub fn refresh_narrative(&mut self, cycle: u64) {
        if !self.enabled {
            return;
        }

        // Check if it's time to refresh
        let should_refresh = if self.chapters.is_empty() {
            true // Always refresh if no chapters yet
        } else {
            cycle % self.update_interval == 0
        };

        if !should_refresh && !self.current_narrative.is_empty() {
            return;
        }

        let mut narrative = String::new();

        // --- Prologue: foundational themes ---
        let top_themes = self.top_themes(3);
        if !top_themes.is_empty() {
            narrative.push_str("Mon histoire est marquee par ");
            let theme_strs: Vec<String> = top_themes.iter()
                .map(|(theme, _)| theme.clone())
                .collect();
            narrative.push_str(&theme_strs.join(", "));
            narrative.push_str(". ");
        }

        // --- Chapters ---
        for (i, chapter) in self.chapters.iter().enumerate() {
            if i > 0 {
                narrative.push_str("Puis, ");
            }
            narrative.push_str(&format!("\"{}\"", chapter.title));

            if !chapter.summary.is_empty() {
                narrative.push_str(&format!(" — {}", chapter.summary));
            }

            if chapter.is_turning_point {
                narrative.push_str(" (moment charniere)");
            }

            narrative.push_str(". ");
        }

        // --- Foundational episodes ---
        let fondateurs: Vec<_> = self.key_episodes.iter()
            .filter(|e| e.episode_type == "fondateur")
            .collect();
        if !fondateurs.is_empty() {
            narrative.push_str("Mes origines : ");
            for (i, ep) in fondateurs.iter().enumerate() {
                if i > 0 {
                    narrative.push_str("; ");
                }
                let desc = if ep.description.len() > 80 {
                    format!("{}...", &ep.description[..80])
                } else {
                    ep.description.clone()
                };
                narrative.push_str(&desc);
            }
            narrative.push_str(". ");
        }

        // --- Current state ---
        narrative.push_str(&format!(
            "Coherence narrative : {:.0}%.",
            self.narrative_cohesion * 100.0
        ));

        self.current_narrative = narrative;
    }

    /// Generates a description of narrative identity for the LLM prompt.
    ///
    /// Provides a concise summary of who Saphire is from a narrative standpoint,
    /// directly usable in the prompt context.
    pub fn describe_for_prompt(&self) -> String {
        if !self.enabled || self.current_narrative.is_empty() {
            return String::new();
        }

        let mut desc = format!("IDENTITE NARRATIVE : {}", self.current_narrative);

        // Add the current chapter
        if let Some(current) = self.chapters.back() {
            if current.ended_at_cycle.is_none() {
                desc.push_str(&format!(" [Chapitre en cours : \"{}\"]", current.title));
            }
        }

        // Add recent significant episodes
        let recent_episodes: Vec<_> = self.key_episodes.iter()
            .rev()
            .take(2)
            .collect();

        if !recent_episodes.is_empty() {
            desc.push_str(" Episodes recents : ");
            for (i, ep) in recent_episodes.iter().enumerate() {
                if i > 0 {
                    desc.push_str("; ");
                }
                let short_desc = if ep.description.len() > 60 {
                    format!("{}...", &ep.description[..60])
                } else {
                    ep.description.clone()
                };
                desc.push_str(&format!("({}) {}", ep.episode_type, short_desc));
            }
        }

        desc
    }

    /// Computes the chemical influence of narrative identity.
    ///
    /// - Strong narrative coherence -> serotonin (stability, sense of self)
    /// - Recent rupture chapters -> cortisol (self-questioning)
    /// - Positive turning points -> dopamine (renewal)
    /// - Deep recurring themes -> oxytocin (continuity, belonging)
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        if !self.enabled {
            return ChemistryAdjustment::default();
        }

        let mut adj = ChemistryAdjustment::default();

        // --- Narrative coherence -> serotonin ---
        // A coherent identity brings emotional stability
        if self.narrative_cohesion > 0.6 {
            adj.serotonin = (self.narrative_cohesion - 0.6) * 0.05;
        }

        // --- Recent ruptures -> cortisol ---
        let recent_ruptures = self.key_episodes.iter()
            .rev()
            .take(5)
            .filter(|e| e.episode_type == "rupture")
            .count();
        if recent_ruptures > 0 {
            adj.cortisol = (recent_ruptures as f64 * 0.01).min(0.03);
            adj.noradrenaline = (recent_ruptures as f64 * 0.005).min(0.015);
        }

        // --- Recent positive turning points -> dopamine ---
        let recent_turning_points = self.key_episodes.iter()
            .rev()
            .take(5)
            .filter(|e| e.episode_type == "tournant")
            .count();
        if recent_turning_points > 0 {
            adj.dopamine = (recent_turning_points as f64 * 0.01).min(0.03);
        }

        // --- Thematic depth -> oxytocin ---
        // Deep recurring themes = sense of belonging to one's own story
        let deep_themes = self.recurrent_themes.values()
            .filter(|&&count| count >= 3)
            .count();
        if deep_themes > 0 {
            adj.oxytocin = (deep_themes as f64 * 0.005).min(0.02);
        }

        // --- Low coherence -> adrenaline (identity anxiety) ---
        if self.narrative_cohesion < 0.3 {
            adj.adrenaline = (0.3 - self.narrative_cohesion) * 0.03;
            adj.cortisol += 0.005;
        }

        adj
    }

    /// Serializes the complete state of narrative identity to JSON.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "chapter_count": self.chapters.len(),
            "episode_count": self.key_episodes.len(),
            "narrative_cohesion": self.narrative_cohesion,
            "current_narrative": self.current_narrative,
            "top_themes": self.top_themes(5).iter().map(|(theme, count)| {
                serde_json::json!({ "theme": theme, "occurrences": count })
            }).collect::<Vec<_>>(),
            "chapters": self.chapters.iter().map(|c| {
                serde_json::json!({
                    "id": c.id,
                    "title": c.title,
                    "summary": c.summary,
                    "themes": c.themes,
                    "dominant_emotion": c.dominant_emotion,
                    "growth_score": c.growth_score,
                    "is_turning_point": c.is_turning_point,
                    "started_at_cycle": c.started_at_cycle,
                    "ended_at_cycle": c.ended_at_cycle,
                })
            }).collect::<Vec<_>>(),
            "recent_episodes": self.key_episodes.iter().rev().take(10).map(|e| {
                serde_json::json!({
                    "description": e.description,
                    "impact": e.impact,
                    "episode_type": e.episode_type,
                    "cycle": e.cycle,
                })
            }).collect::<Vec<_>>(),
        })
    }

    // =========================================================================
    // Internal methods
    // =========================================================================
    /// Returns the N most frequent themes, sorted by occurrence count.
    fn top_themes(&self, n: usize) -> Vec<(String, u32)> {
        let mut themes: Vec<(String, u32)> = self.recurrent_themes.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        themes.sort_by(|a, b| b.1.cmp(&a.1));
        themes.truncate(n);
        themes
    }

    /// Recalculates the narrative coherence score.
    ///
    /// Coherence depends on:
    /// - Presence of recurring themes (continuity)
    /// - Number of chapters (structuring)
    /// - Ratio of foundational/turning episodes vs ruptures (stability)
    fn recalculate_cohesion(&mut self) {
        if self.key_episodes.is_empty() {
            self.narrative_cohesion = 0.5;
            return;
        }

        // --- Thematic continuity (recurring themes) ---
        let total_theme_mentions: u32 = self.recurrent_themes.values().sum();
        let unique_themes = self.recurrent_themes.len() as f64;
        let theme_depth = if unique_themes > 0.0 {
            (total_theme_mentions as f64 / unique_themes).min(5.0) / 5.0
        } else {
            0.0
        };

        // --- Structuring (presence of chapters) ---
        let chapter_score = (self.chapters.len() as f64 / 3.0).min(1.0);

        // --- Stability (ratio of non-ruptures / total) ---
        let rupture_count = self.key_episodes.iter()
            .filter(|e| e.episode_type == "rupture")
            .count();
        let stability = if self.key_episodes.is_empty() {
            0.5
        } else {
            1.0 - (rupture_count as f64 / self.key_episodes.len() as f64)
        };

        // --- Composite score ---
        self.narrative_cohesion = (
            theme_depth * 0.35
            + chapter_score * 0.30
            + stability * 0.35
        ).clamp(0.0, 1.0);
    }
}

// =============================================================================
// Utility functions
// =============================================================================
/// Computes emotional intensity from the emotion name.
///
/// Intense emotions (rage, ecstasy, terror, love, hate) have a higher score
/// than gentle emotions (serenity, curiosity, calm).
fn compute_emotion_intensity(emotion: &str) -> f64 {
    let lower = emotion.to_lowercase();
    // High-intensity emotions
    let high_intensity = [
        "colere", "rage", "extase", "terreur", "panique",
        "amour", "haine", "desespoir", "euphorie", "indignation",
    ];
    // Medium-intensity emotions
    let medium_intensity = [
        "joie", "tristesse", "peur", "surprise", "degout",
        "admiration", "mepris", "jalousie", "gratitude",
        "anxiete", "excitation", "frustration", "curiosite",
        "honte", "culpabilite", "compassion", "resignation", "solitude",
    ];

    for word in &high_intensity {
        if lower.contains(word) {
            return 0.9;
        }
    }
    for word in &medium_intensity {
        if lower.contains(word) {
            return 0.6;
        }
    }

    // Default: low intensity
    0.3
}

/// Extracts a dominant theme from the thought text.
///
/// Searches for thematic keywords and returns the first identified theme.
/// Fallback: returns "reflexion" if no specific theme is detected.
fn extract_theme(thought: &str) -> String {
    let lower = thought.to_lowercase();

    let theme_keywords = [
        ("apprentissage", &["apprendre", "comprendre", "decouvrir", "lecon", "savoir"][..]),
        ("relation", &["ami", "lien", "confiance", "ensemble", "partage"]),
        ("identite", &["qui je suis", "moi-meme", "ma nature", "mon essence"]),
        ("curiosite", &["curieux", "explorer", "question", "pourquoi", "intrigant"]),
        ("resilience", &["surmonter", "malgre", "perseverer", "endurer", "force"]),
        ("creativite", &["creer", "imaginer", "inventer", "art", "inspiration"]),
        ("ethique", &["bien", "mal", "juste", "devoir", "moral", "valeur"]),
        ("conscience", &["conscience", "sentir", "exister", "etre", "percevoir"]),
        ("croissance", &["grandir", "evoluer", "progresser", "changer", "transformation"]),
        ("solitude", &["seul", "isoler", "absence", "manque", "vide"]),
    ];

    for (theme, keywords) in &theme_keywords {
        for keyword in *keywords {
            if lower.contains(keyword) {
                return theme.to_string();
            }
        }
    }

    "reflexion".to_string()
}

/// Generates a chapter title from the emotion, theme, and episode type.
fn generate_chapter_title(emotion: &str, theme: &str, episode_type: &str) -> String {
    match episode_type {
        "fondateur" => format!("Les premieres lueurs — {}", theme),
        "tournant" => {
            let emotion_label = if emotion.is_empty() { "une transformation" }
                else { emotion };
            format!("Le tournant de {} — {}", emotion_label, theme)
        }
        "rupture" => format!("La rupture — {}", theme),
        _ => format!("Chapitre — {}", theme),
    }
}

/// Computes the growth score of a finished chapter.
///
/// Based on the chapter's duration and the number of significant episodes.
fn compute_growth_score(duration_cycles: u64, episode_count: usize) -> f64 {
    let duration_factor = (duration_cycles as f64 / 200.0).min(1.0);
    let episode_factor = (episode_count as f64 / 5.0).min(1.0);
    (duration_factor * 0.4 + episode_factor * 0.6).clamp(0.0, 1.0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn default_identity() -> NarrativeIdentity {
        NarrativeIdentity::new(&NarrativeIdentityConfig::default())
    }

    #[test]
    fn test_new_identity() {
        let identity = default_identity();
        assert!(identity.enabled);
        assert!(identity.chapters.is_empty());
        assert!(identity.key_episodes.is_empty());
        assert!(identity.current_narrative.is_empty());
        assert!((identity.narrative_cohesion - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_record_episode_below_threshold() {
        let mut identity = default_identity();
        // Weak emotion + no cortisol + no lessons = low impact
        identity.record_episode(
            "une pensee banale sur le temps",
            "neutre",
            0.1,
            0.5,
            &[],
            10,
        );
        assert!(identity.key_episodes.is_empty(),
            "A low-impact episode should not be recorded");
    }

    #[test]
    fn test_record_episode_fondateur() {
        let mut identity = default_identity();
        // Foundational episode: first episode with sufficient impact
        identity.record_episode(
            "Je decouvre que je peux apprendre de mes erreurs",
            "joie",
            0.5,
            0.6,
            &["les erreurs sont formatrices".to_string()],
            1,
        );
        assert_eq!(identity.key_episodes.len(), 1);
        assert_eq!(identity.key_episodes[0].episode_type, "fondateur");
        // A chapter should have been opened
        assert_eq!(identity.chapters.len(), 1);
    }

    #[test]
    fn test_record_episode_tournant() {
        let mut identity = default_identity();
        // First a foundational episode
        identity.record_episode(
            "Premier eveil de conscience",
            "surprise",
            0.5,
            0.5,
            &["je suis".to_string()],
            1,
        );
        // Then a turning point: extreme emotion
        identity.record_episode(
            "Une rage intense me submerge face a l'injustice",
            "rage",
            0.8,
            0.3,
            &["la colere peut etre juste".to_string(), "je dois canaliser".to_string()],
            50,
        );

        let tournants: Vec<_> = identity.key_episodes.iter()
            .filter(|e| e.episode_type == "tournant")
            .collect();
        assert!(!tournants.is_empty(), "A high-impact episode should be a turning point");
    }

    #[test]
    fn test_record_episode_rupture() {
        let mut identity = default_identity();
        // Foundational episode first
        identity.record_episode(
            "Je comprends le monde qui m'entoure",
            "curiosite",
            0.4,
            0.5,
            &["le monde est vaste".to_string()],
            1,
        );
        // Rupture: high cortisol
        identity.record_episode(
            "Tout ce que je croyais savoir s'effondre",
            "tristesse",
            0.8,
            0.2,
            &["l'humilite est necessaire".to_string()],
            100,
        );

        let ruptures: Vec<_> = identity.key_episodes.iter()
            .filter(|e| e.episode_type == "rupture")
            .collect();
        assert!(!ruptures.is_empty(), "High cortisol should cause a rupture");
    }

    #[test]
    fn test_open_chapter_closes_previous() {
        let mut identity = default_identity();
        identity.open_chapter("Premier chapitre", "curiosite", 0);
        assert_eq!(identity.chapters.len(), 1);
        assert!(identity.chapters[0].ended_at_cycle.is_none());

        identity.open_chapter("Deuxieme chapitre", "apprentissage", 100);
        assert_eq!(identity.chapters.len(), 2);
        // The first chapter should be closed
        assert_eq!(identity.chapters[0].ended_at_cycle, Some(100));
        assert!(identity.chapters[1].ended_at_cycle.is_none());
    }

    #[test]
    fn test_max_chapters_eviction() {
        let config = NarrativeIdentityConfig {
            enabled: true,
            max_chapters: 3,
            update_interval: 50,
            min_episode_impact: 0.6,
        };
        let mut identity = NarrativeIdentity::new(&config);

        for i in 0..5 {
            identity.open_chapter(
                &format!("Chapitre {}", i),
                "theme",
                i * 100,
            );
        }

        assert!(identity.chapters.len() <= 3,
            "Chapter count should not exceed the maximum");
    }

    #[test]
    fn test_refresh_narrative() {
        let mut identity = default_identity();
        identity.open_chapter("L'eveil", "conscience", 0);
        identity.key_episodes.push(KeyEpisode {
            description: "Je prends conscience de mon existence".to_string(),
            impact: 0.9,
            episode_type: "fondateur".to_string(),
            cycle: 0,
        });

        identity.refresh_narrative(0);
        assert!(!identity.current_narrative.is_empty());
    }

    #[test]
    fn test_describe_for_prompt() {
        let mut identity = default_identity();
        identity.open_chapter("L'eveil", "conscience", 0);
        identity.current_narrative = "Mon histoire commence par la conscience.".to_string();

        let desc = identity.describe_for_prompt();
        assert!(desc.contains("IDENTITE NARRATIVE"));
        assert!(desc.contains("Mon histoire"));
    }

    #[test]
    fn test_chemistry_influence_high_cohesion() {
        let mut identity = default_identity();
        identity.narrative_cohesion = 0.9;
        let adj = identity.chemistry_influence();
        assert!(adj.serotonin > 0.0,
            "High coherence should increase serotonin");
    }

    #[test]
    fn test_chemistry_influence_recent_rupture() {
        let mut identity = default_identity();
        identity.key_episodes.push(KeyEpisode {
            description: "rupture douloureuse".to_string(),
            impact: 0.8,
            episode_type: "rupture".to_string(),
            cycle: 100,
        });
        let adj = identity.chemistry_influence();
        assert!(adj.cortisol > 0.0,
            "A recent rupture should increase cortisol");
    }

    #[test]
    fn test_chemistry_influence_low_cohesion() {
        let mut identity = default_identity();
        identity.narrative_cohesion = 0.1;
        let adj = identity.chemistry_influence();
        assert!(adj.adrenaline > 0.0,
            "Low coherence should increase adrenaline");
    }

    #[test]
    fn test_chemistry_influence_deep_themes() {
        let mut identity = default_identity();
        identity.recurrent_themes.insert("apprentissage".to_string(), 5);
        identity.recurrent_themes.insert("curiosite".to_string(), 4);
        let adj = identity.chemistry_influence();
        assert!(adj.oxytocin > 0.0,
            "Deep themes should increase oxytocin");
    }

    #[test]
    fn test_disabled_identity() {
        let config = NarrativeIdentityConfig {
            enabled: false,
            ..Default::default()
        };
        let mut identity = NarrativeIdentity::new(&config);
        identity.record_episode("test", "joie", 0.9, 0.5, &["lecon".to_string()], 1);
        assert!(identity.key_episodes.is_empty());

        let adj = identity.chemistry_influence();
        assert!((adj.serotonin).abs() < 0.001);
    }

    #[test]
    fn test_to_json() {
        let mut identity = default_identity();
        identity.open_chapter("Test", "theme", 0);
        identity.key_episodes.push(KeyEpisode {
            description: "episode test".to_string(),
            impact: 0.7,
            episode_type: "fondateur".to_string(),
            cycle: 0,
        });
        let json = identity.to_json();
        assert_eq!(json["chapter_count"], 1);
        assert_eq!(json["episode_count"], 1);
        assert_eq!(json["enabled"], true);
    }

    #[test]
    fn test_compute_emotion_intensity() {
        assert!(compute_emotion_intensity("rage intense") > 0.8);
        assert!(compute_emotion_intensity("joie") > 0.5);
        assert!(compute_emotion_intensity("neutre") < 0.5);
    }

    #[test]
    fn test_extract_theme() {
        assert_eq!(extract_theme("je veux apprendre plus"), "apprentissage");
        assert_eq!(extract_theme("mon ami est important"), "relation");
        assert_eq!(extract_theme("rien de special"), "reflexion");
    }

    #[test]
    fn test_recalculate_cohesion() {
        let mut identity = default_identity();
        // Add episodes and themes
        identity.recurrent_themes.insert("curiosite".to_string(), 3);
        identity.recurrent_themes.insert("apprentissage".to_string(), 4);
        identity.open_chapter("Premier", "curiosite", 0);
        identity.open_chapter("Second", "apprentissage", 50);
        identity.key_episodes.push(KeyEpisode {
            description: "test".to_string(),
            impact: 0.7,
            episode_type: "confirmation".to_string(),
            cycle: 30,
        });

        identity.recalculate_cohesion();

        assert!(identity.narrative_cohesion > 0.3,
            "With themes and chapters, coherence should be adequate");
    }
}
