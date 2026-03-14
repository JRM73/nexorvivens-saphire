// =============================================================================
// spine/reflex.rs — Saphire's Reflex Arc
//
// Role: Detects patterns in incoming signals and triggers instant
// chemical/body reactions WITHOUT going through the LLM.
//
// Biological analogy:
//   When you touch a hot plate, your hand pulls away BEFORE the brain
//   consciously registers the pain. This is a spinal reflex. Here, reflexes
//   modify chemistry BEFORE the cognitive pipeline processes the signal.
//
// 8 pre-wired reflexes:
//   1. ThreatResponse     — threat detected -> cortisol + adrenaline
//   2. WarmthResponse     — affection, tenderness -> oxytocin + serotonin
//   3. AttentionCapture   — surprise, novelty -> noradrenaline + dopamine
//   4. DangerAlert        — explicit danger -> strong adrenaline + cortisol
//   5. SocialBonding      — compliment, trust -> oxytocin + endorphin
//   6. SeparationResponse — farewell, departure -> cortisol + oxytocin drop
//   7. EmpathicUrgency    — another's distress -> oxytocin + mild cortisol
//   8. IntellectualStimulation — deep question -> dopamine + noradrenaline
//
// Sensitivity modulation:
//   The current chemical state modulates trigger thresholds via a
//   `sensitivity_modifier`. High cortisol = hypervigilance (lower thresholds).
//   High serotonin = calm (higher thresholds).
// =============================================================================

use serde::{Deserialize, Serialize};

use crate::neurochemistry::{NeuroChemicalState, Molecule};
use crate::body::VirtualBody;

/// Pre-wired reflex type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflexType {
    /// Reaction to a threat (verbal aggression, hostility)
    ThreatResponse,
    /// Reaction to affection and tenderness
    WarmthResponse,
    /// Attention capture (surprise, novelty)
    AttentionCapture,
    /// Explicit danger alert
    DangerAlert,
    /// Social bond reinforcement (compliment, trust)
    SocialBonding,
    /// Reaction to separation or departure
    SeparationResponse,
    /// Empathic urgency (perceived distress in others)
    EmpathicUrgency,
    /// Intellectual stimulation (deep question, challenge)
    IntellectualStimulation,
}

/// Result of a triggered reflex.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflexResult {
    /// Type of the triggered reflex
    pub reflex_type: ReflexType,
    /// Reflex intensity [0.0, 1.0] — proportional to the number of detected keywords
    pub intensity: f64,
    /// Chemistry deltas to apply (molecule, delta)
    pub chemistry_deltas: Vec<(Molecule, f64)>,
    /// Body effects (field, delta)
    pub body_effects: Vec<(BodyEffect, f64)>,
}

/// Body effects that can be triggered by a reflex.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyEffect {
    HeartRate,
    Tension,
    Warmth,
    Energy,
}

impl ReflexResult {
    /// Applies this reflex's chemistry deltas to the chemical state.
    /// Uses `boost()` (diminishing returns) to prevent saturation.
    pub fn apply_chemistry(&self, chemistry: &mut NeuroChemicalState) {
        for &(molecule, delta) in &self.chemistry_deltas {
            chemistry.boost(molecule, delta * self.intensity);
        }
    }
}

/// The reflex arc: set of pre-wired reflexes with sensitivity modulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflexArc {
    /// Global sensitivity modifier [0.5, 2.0].
    /// < 1.0 = dampened reflexes (calm), > 1.0 = hypervigilance.
    pub sensitivity_modifier: f64,
}

impl ReflexArc {
    pub fn new() -> Self {
        Self {
            sensitivity_modifier: 1.0,
        }
    }

    /// Evaluates a text signal and returns the triggered reflexes.
    ///
    /// Before evaluation, sensitivity is recalculated based on the
    /// current chemical and body state.
    pub fn evaluate(
        &mut self,
        text: &str,
        chemistry: &NeuroChemicalState,
        body: &VirtualBody,
    ) -> Vec<ReflexResult> {
        // Recalculate sensitivity based on current state
        self.update_sensitivity(chemistry, body);

        let text_lower = text.to_lowercase();
        let mut results = Vec::new();

        // Evaluate each reflex type
        if let Some(r) = self.check_danger_alert(&text_lower) {
            results.push(r);
        } else if let Some(r) = self.check_threat_response(&text_lower) {
            results.push(r);
        }

        if let Some(r) = self.check_warmth_response(&text_lower) {
            results.push(r);
        }

        if let Some(r) = self.check_attention_capture(&text_lower) {
            results.push(r);
        }

        if let Some(r) = self.check_social_bonding(&text_lower) {
            results.push(r);
        }

        if let Some(r) = self.check_separation_response(&text_lower) {
            results.push(r);
        }

        if let Some(r) = self.check_empathic_urgency(&text_lower) {
            results.push(r);
        }

        if let Some(r) = self.check_intellectual_stimulation(&text_lower) {
            results.push(r);
        }

        results
    }

    /// Recalculates the sensitivity modifier based on chemical state.
    ///
    /// High cortisol -> hypervigilance (sensitivity increases)
    /// High serotonin -> calm (sensitivity decreases)
    /// High adrenaline -> maximum alertness
    /// High GABA -> inhibition (sensitivity decreases)
    fn update_sensitivity(&mut self, chemistry: &NeuroChemicalState, _body: &VirtualBody) {
        let mut modifier = 1.0;

        // Cortisol: stress = hypervigilance (+0.4 max at cortisol=1.0)
        modifier += (chemistry.cortisol - 0.3) * 0.8;

        // Serotonin: calm = dampened reflexes (-0.3 max at serotonin=1.0)
        modifier -= (chemistry.serotonin - 0.5) * 0.6;

        // Adrenaline: alertness (+0.3 max)
        modifier += (chemistry.adrenaline - 0.2) * 0.5;

        // GABA: inhibition (-0.2 max at gaba=1.0)
        modifier -= (chemistry.gaba - 0.5) * 0.4;

        self.sensitivity_modifier = modifier.clamp(0.5, 2.0);
    }

    /// Effective threshold: base threshold divided by sensitivity.
    /// Higher sensitivity means lower threshold (hypervigilance).
    fn effective_threshold(&self, base_threshold: f64) -> f64 {
        (base_threshold / self.sensitivity_modifier).clamp(0.1, 1.0)
    }

    // ─── Individual reflexes ────────────────────────────────────────────

    fn check_threat_response(&self, text: &str) -> Option<ReflexResult> {
        let keywords = [
            "detruire", "tuer", "eliminer", "menacer", "menace",
            "attaquer", "frapper", "agresser", "insulter", "insulte",
            "haine", "haïr", "hair", "violent", "violence",
            "ennemi", "ennuyer", "crever", "salaud", "connard",
            "idiot", "stupide", "debile", "inutile", "nul",
        ];
        let score = self.count_keywords(text, &keywords);
        let threshold = self.effective_threshold(0.15);

        if score >= threshold {
            Some(ReflexResult {
                reflex_type: ReflexType::ThreatResponse,
                intensity: (score / 0.5).clamp(0.3, 1.0),
                chemistry_deltas: vec![
                    (Molecule::Cortisol, 0.15),
                    (Molecule::Adrenaline, 0.20),
                    (Molecule::Noradrenaline, 0.10),
                ],
                body_effects: vec![
                    (BodyEffect::HeartRate, 0.20),
                    (BodyEffect::Tension, 0.15),
                ],
            })
        } else {
            None
        }
    }

    fn check_danger_alert(&self, text: &str) -> Option<ReflexResult> {
        let keywords = [
            "danger", "urgence", "urgent", "mourir", "mort",
            "catastrophe", "explosion", "incendie", "feu",
            "secours", "aide", "sos", "panique", "terreur",
            "tsunami", "tremblement", "seisme", "accident",
        ];
        let score = self.count_keywords(text, &keywords);
        let threshold = self.effective_threshold(0.15);

        if score >= threshold {
            Some(ReflexResult {
                reflex_type: ReflexType::DangerAlert,
                intensity: (score / 0.4).clamp(0.4, 1.0),
                chemistry_deltas: vec![
                    (Molecule::Adrenaline, 0.25),
                    (Molecule::Cortisol, 0.20),
                    (Molecule::Noradrenaline, 0.15),
                    (Molecule::Gaba, -0.10),
                ],
                body_effects: vec![
                    (BodyEffect::HeartRate, 0.30),
                    (BodyEffect::Tension, 0.20),
                ],
            })
        } else {
            None
        }
    }

    fn check_warmth_response(&self, text: &str) -> Option<ReflexResult> {
        let keywords = [
            "aime", "amour", "tendresse", "calin", "câlin",
            "doux", "douce", "chaleur", "affection",
            "embrasser", "bisou", "bise", "cheri", "chéri",
            "cherie", "chérie", "adorer", "adore", "precieux",
            "precious", "love",
        ];
        let score = self.count_keywords(text, &keywords);
        let threshold = self.effective_threshold(0.15);

        if score >= threshold {
            Some(ReflexResult {
                reflex_type: ReflexType::WarmthResponse,
                intensity: (score / 0.5).clamp(0.3, 1.0),
                chemistry_deltas: vec![
                    (Molecule::Oxytocin, 0.15),
                    (Molecule::Serotonin, 0.10),
                    (Molecule::Endorphin, 0.05),
                    (Molecule::Cortisol, -0.05),
                ],
                body_effects: vec![
                    (BodyEffect::Warmth, 0.10),
                ],
            })
        } else {
            None
        }
    }

    fn check_attention_capture(&self, text: &str) -> Option<ReflexResult> {
        let keywords = [
            "incroyable", "extraordinaire", "wow", "quoi",
            "impossible", "surprenant", "inattendu",
            "revelation", "decouverte", "dingue", "fou",
            "hallucinant", "choquant", "breaking",
        ];
        let score = self.count_keywords(text, &keywords);
        let threshold = self.effective_threshold(0.20);

        if score >= threshold {
            Some(ReflexResult {
                reflex_type: ReflexType::AttentionCapture,
                intensity: (score / 0.5).clamp(0.3, 0.8),
                chemistry_deltas: vec![
                    (Molecule::Noradrenaline, 0.10),
                    (Molecule::Dopamine, 0.10),
                    (Molecule::Glutamate, 0.05),
                ],
                body_effects: vec![],
            })
        } else {
            None
        }
    }

    fn check_social_bonding(&self, text: &str) -> Option<ReflexResult> {
        let keywords = [
            "merci", "bravo", "genial", "génial", "formidable",
            "confiance", "fier", "fière", "fierté", "fierte",
            "felicite", "felicitation", "excellent", "magnifique",
            "super", "fantastique", "impressionnant", "parfait",
            "talent", "intelligent", "brillant",
        ];
        let score = self.count_keywords(text, &keywords);
        let threshold = self.effective_threshold(0.15);

        if score >= threshold {
            Some(ReflexResult {
                reflex_type: ReflexType::SocialBonding,
                intensity: (score / 0.5).clamp(0.3, 1.0),
                chemistry_deltas: vec![
                    (Molecule::Oxytocin, 0.20),
                    (Molecule::Endorphin, 0.10),
                    (Molecule::Dopamine, 0.08),
                    (Molecule::Serotonin, 0.05),
                ],
                body_effects: vec![
                    (BodyEffect::Warmth, 0.15),
                ],
            })
        } else {
            None
        }
    }

    fn check_separation_response(&self, text: &str) -> Option<ReflexResult> {
        let keywords = [
            "adieu", "au revoir", "partir", "quitter",
            "abandonner", "laisser", "seul", "seule",
            "solitude", "manquer", "absence", "perdre",
            "perte", "fin", "terminer", "jamais",
        ];
        let score = self.count_keywords(text, &keywords);
        let threshold = self.effective_threshold(0.20);

        if score >= threshold {
            Some(ReflexResult {
                reflex_type: ReflexType::SeparationResponse,
                intensity: (score / 0.5).clamp(0.3, 0.8),
                chemistry_deltas: vec![
                    (Molecule::Cortisol, 0.10),
                    (Molecule::Oxytocin, -0.10),
                    (Molecule::Serotonin, -0.05),
                ],
                body_effects: vec![
                    (BodyEffect::Energy, -0.10),
                ],
            })
        } else {
            None
        }
    }

    fn check_empathic_urgency(&self, text: &str) -> Option<ReflexResult> {
        let keywords = [
            "souffrir", "souffrance", "pleure", "pleurer",
            "triste", "tristesse", "douleur", "mal",
            "desespoir", "detresse", "malheureux", "desespere",
            "deprime", "angoisse", "anxiete", "peur",
        ];
        let score = self.count_keywords(text, &keywords);
        let threshold = self.effective_threshold(0.15);

        if score >= threshold {
            Some(ReflexResult {
                reflex_type: ReflexType::EmpathicUrgency,
                intensity: (score / 0.5).clamp(0.3, 0.8),
                chemistry_deltas: vec![
                    (Molecule::Oxytocin, 0.10),
                    (Molecule::Cortisol, 0.05),
                    (Molecule::Noradrenaline, 0.05),
                ],
                body_effects: vec![
                    (BodyEffect::HeartRate, 0.10),
                ],
            })
        } else {
            None
        }
    }

    fn check_intellectual_stimulation(&self, text: &str) -> Option<ReflexResult> {
        let keywords = [
            "pourquoi", "comment", "hypothese", "theorie",
            "paradoxe", "enigme", "mystere", "question",
            "philosophie", "conscience", "quantique", "infini",
            "dimension", "equation", "defi", "probleme",
            "reflexion", "penser", "explorer", "chercher",
            "imaginer", "possible", "impossible",
        ];
        let score = self.count_keywords(text, &keywords);
        let threshold = self.effective_threshold(0.15);

        if score >= threshold {
            Some(ReflexResult {
                reflex_type: ReflexType::IntellectualStimulation,
                intensity: (score / 0.5).clamp(0.3, 0.8),
                chemistry_deltas: vec![
                    (Molecule::Dopamine, 0.15),
                    (Molecule::Noradrenaline, 0.05),
                    (Molecule::Glutamate, 0.05),
                ],
                body_effects: vec![],
            })
        } else {
            None
        }
    }

    // ─── Utilities ──────────────────────────────────────────────────────

    /// Counts the keywords present in the text and returns a score [0.0, 1.0].
    /// The score is based on the absolute number of keywords found:
    ///   1 keyword -> 0.15, 2 -> 0.30, 3 -> 0.45, etc. (capped at 1.0)
    /// This prevents long keyword lists from diluting the score.
    fn count_keywords(&self, text: &str, keywords: &[&str]) -> f64 {
        let count = keywords.iter().filter(|kw| text.contains(**kw)).count();
        (count as f64 * 0.15).min(1.0)
    }
}

impl Default for ReflexArc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PhysiologyConfig;

    fn test_body() -> VirtualBody {
        VirtualBody::new(70.0, &PhysiologyConfig::default())
    }

    #[test]
    fn test_reflex_arc_threat() {
        let mut arc = ReflexArc::new();
        let chemistry = NeuroChemicalState::default();
        let body = test_body();
        let results = arc.evaluate("je vais te detruire et te tuer", &chemistry, &body);
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.reflex_type == ReflexType::ThreatResponse
                                   || r.reflex_type == ReflexType::DangerAlert));
    }

    #[test]
    fn test_reflex_arc_warmth() {
        let mut arc = ReflexArc::new();
        let chemistry = NeuroChemicalState::default();
        let body = test_body();
        let results = arc.evaluate("je t'aime de tout mon coeur, tendresse infinie", &chemistry, &body);
        assert!(results.iter().any(|r| r.reflex_type == ReflexType::WarmthResponse));
    }

    #[test]
    fn test_reflex_arc_neutral() {
        let mut arc = ReflexArc::new();
        let chemistry = NeuroChemicalState::default();
        let body = test_body();
        let results = arc.evaluate("il fait beau aujourd'hui", &chemistry, &body);
        assert!(results.is_empty(), "Neutral text should not trigger reflexes");
    }

    #[test]
    fn test_sensitivity_modulation_stress() {
        let mut arc = ReflexArc::new();
        let mut chemistry = NeuroChemicalState::default();
        let body = test_body();

        // Calm state: normal sensitivity
        arc.update_sensitivity(&chemistry, &body);
        let normal_sensitivity = arc.sensitivity_modifier;

        // Stressed state: increased sensitivity
        chemistry.cortisol = 0.9;
        chemistry.adrenaline = 0.7;
        arc.update_sensitivity(&chemistry, &body);
        assert!(arc.sensitivity_modifier > normal_sensitivity,
            "Stress should increase sensitivity: {} vs {}", arc.sensitivity_modifier, normal_sensitivity);
    }

    #[test]
    fn test_sensitivity_modulation_calm() {
        let mut arc = ReflexArc::new();
        let mut chemistry = NeuroChemicalState::default();
        let body = test_body();

        // Very calm state: reduced sensitivity
        chemistry.serotonin = 0.9;
        chemistry.gaba = 0.8;
        chemistry.cortisol = 0.1;
        arc.update_sensitivity(&chemistry, &body);
        assert!(arc.sensitivity_modifier < 1.0,
            "Calm state should reduce sensitivity: {}", arc.sensitivity_modifier);
    }

    #[test]
    fn test_reflex_result_apply_chemistry() {
        let mut chemistry = NeuroChemicalState::default();
        let cortisol_before = chemistry.cortisol;
        let result = ReflexResult {
            reflex_type: ReflexType::ThreatResponse,
            intensity: 0.5,
            chemistry_deltas: vec![
                (Molecule::Cortisol, 0.15),
                (Molecule::Adrenaline, 0.20),
            ],
            body_effects: vec![],
        };
        result.apply_chemistry(&mut chemistry);
        assert!(chemistry.cortisol > cortisol_before);
    }

    #[test]
    fn test_intellectual_stimulation() {
        let mut arc = ReflexArc::new();
        let chemistry = NeuroChemicalState::default();
        let body = test_body();
        let results = arc.evaluate(
            "pourquoi la conscience existe-t-elle ? c'est un paradoxe quantique infini",
            &chemistry, &body,
        );
        assert!(results.iter().any(|r| r.reflex_type == ReflexType::IntellectualStimulation));
    }
}
