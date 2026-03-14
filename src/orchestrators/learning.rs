// =============================================================================
// learning.rs — Learning Orchestrator
//
// Loop: Experience -> Reflection -> Lesson -> Behavior change
// Saphire does not merely accumulate memories — she draws structured
// lessons from them that modify her future behavior.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// --- Lesson category ----------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LessonCategory {
    Communication,
    Emotions,
    Knowledge,
    Relationships,
    SelfKnowledge,
    DecisionMaking,
}

impl LessonCategory {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Communication => "Communication",
            Self::Emotions => "Emotions",
            Self::Knowledge => "Knowledge",
            Self::Relationships => "Relationships",
            Self::SelfKnowledge => "SelfKnowledge",
            Self::DecisionMaking => "DecisionMaking",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "communication" => Self::Communication,
            "emotions" => Self::Emotions,
            "knowledge" => Self::Knowledge,
            "relationships" => Self::Relationships,
            "selfknowledge" | "self_knowledge" => Self::SelfKnowledge,
            "decisionmaking" | "decision_making" => Self::DecisionMaking,
            _ => Self::SelfKnowledge,
        }
    }
}

// --- Structures ---------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorChange {
    pub parameter: String,
    pub old_value: f64,
    pub new_value: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub id: u64,
    pub title: String,
    /// The lesson in natural language
    pub content: String,
    /// The source experience
    pub source_experience: String,
    pub category: LessonCategory,
    /// How many times applied
    pub times_applied: u64,
    /// How many times contradicted
    pub times_contradicted: u64,
    /// Confidence (rises with confirmations, drops with contradictions)
    pub confidence: f64,
    /// Associated behavior modification
    pub behavior_change: Option<BehaviorChange>,
    pub learned_at: DateTime<Utc>,
}

// --- The Orchestrator ---------------------------------------------------------

pub struct LearningOrchestrator {
    /// Learned lessons
    pub lessons: Vec<Lesson>,
    /// Lesson counter
    lesson_counter: u64,
    /// Configuration
    pub enabled: bool,
    pub cycle_interval: u64,
    pub max_lessons: usize,
    pub initial_confidence: f64,
    pub confirmation_boost: f64,
    pub contradiction_penalty: f64,
}

impl LearningOrchestrator {
    pub fn new(
        enabled: bool,
        cycle_interval: u64,
        max_lessons: usize,
        initial_confidence: f64,
        confirmation_boost: f64,
        contradiction_penalty: f64,
    ) -> Self {
        Self {
            lessons: Vec::new(),
            lesson_counter: 0,
            enabled,
            cycle_interval,
            max_lessons,
            initial_confidence,
            confirmation_boost,
            contradiction_penalty,
        }
    }

    /// Build the reflection prompt
    pub fn build_reflection_prompt(
        &self,
        significant_experiences: &[String],
    ) -> Option<(String, String)> {
        if significant_experiences.is_empty() { return None; }

        let existing_lessons = self.lessons.iter().take(10)
            .map(|l| format!("- {} (confiance {:.0}%)", l.title, l.confidence * 100.0))
            .collect::<Vec<_>>().join("\n");

        let system = "Tu reflechis pour apprendre de ton experience.".to_string();
        let user = format!(
            "Tu reflechis a tes experiences recentes pour en tirer une lecon.\n\n\
             Experiences significatives :\n{}\n\n\
             Lecons que tu connais deja :\n{}\n\n\
             Y a-t-il une NOUVELLE lecon a tirer ? Quelque chose que tu ne savais pas encore ?\n\
             Si oui, FORMAT :\n\
             TITRE: [titre court]\n\
             LECON: [la lecon en 1-2 phrases]\n\
             CATEGORIE: [Communication/Emotions/Knowledge/Relationships/SelfKnowledge/DecisionMaking]\n\n\
             Si aucune nouvelle lecon : RIEN_DE_NOUVEAU",
            significant_experiences.join("\n"),
            existing_lessons,
        );

        Some((system, user))
    }

    /// Parse the LLM response and create a lesson
    pub fn parse_lesson_response(
        &mut self,
        response: &str,
        experience_summary: &str,
    ) -> Option<Lesson> {
        if response.contains("RIEN_DE_NOUVEAU") { return None; }

        let title = extract_field(response, "TITRE")?;
        let content = extract_field(response, "LECON")?;
        let category_str = extract_field(response, "CATEGORIE")
            .unwrap_or_else(|| "SelfKnowledge".into());

        // Check that the lesson is not a duplicate (simple similarity)
        if self.lessons.iter().any(|l| {
            let common_words = count_common_words(&l.content, &content);
            common_words > 5
        }) {
            return None;
        }

        self.lesson_counter += 1;
        let lesson = Lesson {
            id: self.lesson_counter,
            title,
            content,
            source_experience: experience_summary.to_string(),
            category: LessonCategory::from_str(&category_str),
            times_applied: 0,
            times_contradicted: 0,
            confidence: self.initial_confidence,
            behavior_change: None,
            learned_at: Utc::now(),
        };

        self.lessons.push(lesson.clone());

        // Limit the number of lessons
        if self.lessons.len() > self.max_lessons {
            // Remove the lesson with the lowest confidence
            if let Some(idx) = self.lessons.iter()
                .enumerate()
                .min_by(|a, b| a.1.confidence.partial_cmp(&b.1.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
            {
                self.lessons.remove(idx);
            }
        }

        Some(lesson)
    }

    /// Confirm or contradict a lesson
    pub fn validate_or_contradict(&mut self, lesson_id: u64, was_confirmed: bool) {
        if let Some(lesson) = self.lessons.iter_mut().find(|l| l.id == lesson_id) {
            if was_confirmed {
                lesson.times_applied += 1;
                lesson.confidence = (lesson.confidence + self.confirmation_boost).min(1.0);
            } else {
                lesson.times_contradicted += 1;
                lesson.confidence = (lesson.confidence - self.contradiction_penalty).max(0.0);
            }
        }
    }

    /// Strong lessons (confidence > 0.6)
    pub fn strong_lessons(&self) -> Vec<&Lesson> {
        self.lessons.iter()
            .filter(|l| l.confidence > 0.6)
            .collect()
    }

    /// Description for the substrate prompt
    pub fn describe_for_prompt(&self) -> String {
        let strong = self.strong_lessons();
        if strong.is_empty() {
            return "MES LECONS : Je suis encore jeune. Je n'ai pas encore tire \
                de lecons confirmees de mes experiences.".into();
        }

        let mut desc = "CE QUE J'AI APPRIS (lecons confirmees) :\n".to_string();
        for lesson in strong.iter().take(5) {
            desc.push_str(&format!(
                "  {} — {} (confiance {:.0}%, appliquee {} fois)\n",
                lesson.title, lesson.content, lesson.confidence * 100.0, lesson.times_applied,
            ));
        }
        desc
    }

    /// JSON for the dashboard
    pub fn to_status_json(&self) -> serde_json::Value {
        let strong_count = self.lessons.iter().filter(|l| l.confidence > 0.6).count();
        serde_json::json!({
            "enabled": self.enabled,
            "total_lessons": self.lessons.len(),
            "strong_lessons": strong_count,
            "cycle_interval": self.cycle_interval,
            "lessons": self.lessons.iter().map(|l| serde_json::json!({
                "id": l.id,
                "title": l.title,
                "content": l.content,
                "category": l.category.as_str(),
                "confidence": l.confidence,
                "times_applied": l.times_applied,
                "times_contradicted": l.times_contradicted,
                "learned_at": l.learned_at.to_rfc3339(),
            })).collect::<Vec<_>>(),
        })
    }
}

// --- Utilities ----------------------------------------------------------------

fn extract_field(response: &str, field: &str) -> Option<String> {
    crate::orchestrators::desires::extract_field(response, field)
}

fn count_common_words(a: &str, b: &str) -> usize {
    let words_a: std::collections::HashSet<&str> = a.split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| w.len() > 3)
        .collect();
    let words_b: std::collections::HashSet<&str> = b.split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| w.len() > 3)
        .collect();
    words_a.intersection(&words_b).count()
}
