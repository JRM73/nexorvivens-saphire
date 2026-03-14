// =============================================================================
// psychology/emotional_intelligence.rs — Emotional Intelligence (Goleman)
//
// 5 components of emotional intelligence:
//   1. Self-awareness (self_awareness)
//   2. Self-regulation (self_regulation)
//   3. Motivation
//   4. Empathy
//   5. Social skills (social_skills)
//
// Overall EQ evolves very slowly (+0.001 per favorable cycle).
// Starts at 0.3 and can reach 1.0.
// =============================================================================

use serde::Serialize;
use super::PsychologyInput;

/// Saphire's emotional intelligence (Goleman model).
#[derive(Debug, Clone, Serialize)]
pub struct EmotionalIntelligence {
    /// Self-awareness: ability to identify one's own emotions
    pub self_awareness: f64,
    /// Self-regulation: ability to regulate emotional reactions
    pub self_regulation: f64,
    /// Motivation: internal engagement, desire to grow
    pub motivation: f64,
    /// Empathy: ability to understand others' emotions
    pub empathy: f64,
    /// Social skills: quality of interactions
    pub social_skills: f64,
    /// Overall EQ score (weighted average)
    pub overall_eq: f64,
    /// Accumulated growth experiences
    pub growth_experiences: u64,
}

impl EmotionalIntelligence {
    /// Creates a modest initial EQ (0.3).
    pub fn new() -> Self {
        Self {
            self_awareness: 0.3,
            self_regulation: 0.3,
            motivation: 0.4,
            empathy: 0.2,
            social_skills: 0.2,
            overall_eq: 0.3,
            growth_experiences: 0,
        }
    }

    /// Recomputes the 5 components and the overall EQ.
    pub fn compute(&mut self, input: &PsychologyInput) {
        // ─── 1. Self-awareness ────────────────────────
        let new_self_awareness = (input.consciousness_level * 0.7
            + input.body_vitality * 0.3)
            .clamp(0.0, 1.0);
        // Smoothing to avoid abrupt jumps
        self.self_awareness = self.self_awareness * 0.8 + new_self_awareness * 0.2;

        // ─── 2. Self-regulation ──────────────────────────
        let new_self_regulation = (input.healing_resilience * 0.5
            + (1.0 - input.cortisol) * 0.5)
            .clamp(0.0, 1.0);
        self.self_regulation = self.self_regulation * 0.8 + new_self_regulation * 0.2;

        // ─── 3. Motivation ───────────────────────────────
        let desires_factor = (input.desires_active_count as f64 / 5.0).min(1.0);
        let new_motivation = (desires_factor * 0.3
            + input.dopamine * 0.4
            + input.attention_depth * 0.3)
            .clamp(0.0, 1.0);
        self.motivation = self.motivation * 0.8 + new_motivation * 0.2;

        // ─── 4. Empathy ─────────────────────────────────
        let conversation_val = if input.in_conversation { 1.0 } else { 0.0 };
        let learning_factor = (input.learning_confirmed_count as f64 / 10.0).min(1.0);
        let new_empathy = (input.oxytocin * 0.4
            + conversation_val * 0.3
            + learning_factor * 0.3)
            .clamp(0.0, 1.0);
        self.empathy = self.empathy * 0.8 + new_empathy * 0.2;

        // ─── 5. Social skills ─────────────────────
        let new_social = (conversation_val * 0.5
            + input.oxytocin * 0.3
            + input.serotonin * 0.2)
            .clamp(0.0, 1.0);
        self.social_skills = self.social_skills * 0.8 + new_social * 0.2;

        // ─── Overall EQ (weighted average) ───────────────
        self.overall_eq = (self.self_awareness * 0.25
            + self.self_regulation * 0.20
            + self.motivation * 0.20
            + self.empathy * 0.20
            + self.social_skills * 0.15)
            .clamp(0.0, 1.0);

        // ─── Slow growth ────────────────────────────
        // EQ grows very slowly under favorable conditions
        if self.self_awareness > 0.5 && self.self_regulation > 0.4 {
            self.growth_experiences += 1;
            // Small permanent boost after each favorable experience
            self.self_awareness = (self.self_awareness + 0.001).min(1.0);
            self.self_regulation = (self.self_regulation + 0.001).min(1.0);
            self.empathy = (self.empathy + 0.001).min(1.0);
        }
    }

    /// Concise description for the LLM prompt.
    pub fn describe(&self) -> String {
        if self.overall_eq > 0.5 && self.overall_eq < 0.7 {
            return String::new(); // Normal state, no need to comment
        }

        if self.overall_eq <= 0.5 {
            let weak: Vec<&str> = [
                (self.self_awareness, "conscience de soi"),
                (self.self_regulation, "maitrise de soi"),
                (self.motivation, "motivation"),
                (self.empathy, "empathie"),
                (self.social_skills, "comp. sociales"),
            ].iter()
                .filter(|(v, _)| *v < 0.4)
                .map(|(_, name)| *name)
                .collect();

            if weak.is_empty() {
                String::new()
            } else {
                format!("EQ faible ({:.0}%) : {} a developper", self.overall_eq * 100.0, weak.join(", "))
            }
        } else {
            format!("EQ elevee ({:.0}%) — experiences: {}", self.overall_eq * 100.0, self.growth_experiences)
        }
    }
}
