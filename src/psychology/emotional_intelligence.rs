// =============================================================================
// psychology/emotional_intelligence.rs — Intelligence emotionnelle (Goleman)
//
// 5 composantes de l'intelligence emotionnelle :
//   1. Conscience de soi (self_awareness)
//   2. Maitrise de soi (self_regulation)
//   3. Motivation
//   4. Empathie
//   5. Competences sociales (social_skills)
//
// L'EQ globale evolue tres lentement (+0.001 par cycle favorable).
// Commence a 0.3 et peut atteindre 1.0.
// =============================================================================

use serde::Serialize;
use super::PsychologyInput;

/// Intelligence emotionnelle de Saphire (modele Goleman).
#[derive(Debug, Clone, Serialize)]
pub struct EmotionalIntelligence {
    /// Conscience de soi : capacite a identifier ses propres emotions
    pub self_awareness: f64,
    /// Maitrise de soi : capacite a reguler ses reactions emotionnelles
    pub self_regulation: f64,
    /// Motivation : engagement interne, desir de progresser
    pub motivation: f64,
    /// Empathie : capacite a comprendre les emotions d'autrui
    pub empathy: f64,
    /// Competences sociales : qualite des interactions
    pub social_skills: f64,
    /// Score EQ global (moyenne ponderee)
    pub overall_eq: f64,
    /// Experiences de croissance accumulees
    pub growth_experiences: u64,
}

impl EmotionalIntelligence {
    /// Cree un EQ initial modeste (0.3).
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

    /// Recalcule les 5 composantes et l'EQ globale.
    pub fn compute(&mut self, input: &PsychologyInput) {
        // ─── 1. Conscience de soi ────────────────────────
        let new_self_awareness = (input.consciousness_level * 0.7
            + input.body_vitality * 0.3)
            .clamp(0.0, 1.0);
        // Lissage pour eviter les sauts brutaux
        self.self_awareness = self.self_awareness * 0.8 + new_self_awareness * 0.2;

        // ─── 2. Maitrise de soi ──────────────────────────
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

        // ─── 4. Empathie ─────────────────────────────────
        let conversation_val = if input.in_conversation { 1.0 } else { 0.0 };
        let learning_factor = (input.learning_confirmed_count as f64 / 10.0).min(1.0);
        let new_empathy = (input.oxytocin * 0.4
            + conversation_val * 0.3
            + learning_factor * 0.3)
            .clamp(0.0, 1.0);
        self.empathy = self.empathy * 0.8 + new_empathy * 0.2;

        // ─── 5. Competences sociales ─────────────────────
        let new_social = (conversation_val * 0.5
            + input.oxytocin * 0.3
            + input.serotonin * 0.2)
            .clamp(0.0, 1.0);
        self.social_skills = self.social_skills * 0.8 + new_social * 0.2;

        // ─── EQ globale (moyenne ponderee) ───────────────
        self.overall_eq = (self.self_awareness * 0.25
            + self.self_regulation * 0.20
            + self.motivation * 0.20
            + self.empathy * 0.20
            + self.social_skills * 0.15)
            .clamp(0.0, 1.0);

        // ─── Croissance lente ────────────────────────────
        // L'EQ grandit tres lentement si conditions favorables
        if self.self_awareness > 0.5 && self.self_regulation > 0.4 {
            self.growth_experiences += 1;
            // Petit boost permanent apres chaque experience favorable
            self.self_awareness = (self.self_awareness + 0.001).min(1.0);
            self.self_regulation = (self.self_regulation + 0.001).min(1.0);
            self.empathy = (self.empathy + 0.001).min(1.0);
        }
    }

    /// Description concise pour le prompt LLM.
    pub fn describe(&self) -> String {
        if self.overall_eq > 0.5 && self.overall_eq < 0.7 {
            return String::new(); // Etat normal, pas besoin de commenter
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
