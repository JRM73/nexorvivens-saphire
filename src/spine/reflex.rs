// =============================================================================
// spine/reflex.rs — Arc reflexe de Saphire
//
// Role : Detecte des patterns dans les signaux entrants et declenche des
// reactions chimiques/corporelles instantanees, SANS passer par le LLM.
//
// Analogie biologique :
//   Quand on touche une plaque chaude, la main se retire AVANT que le cerveau
//   ne conscientise la douleur. C'est un reflexe spinal. Ici, les reflexes
//   modifient la chimie AVANT que le pipeline cognitif ne traite le signal.
//
// 8 reflexes pre-cables :
//   1. ThreatResponse     — menace detectee → cortisol + adrenaline
//   2. WarmthResponse     — affection, tendresse → ocytocine + serotonine
//   3. AttentionCapture   — surprise, nouveaute → noradrenaline + dopamine
//   4. DangerAlert        — danger explicite → adrenaline forte + cortisol
//   5. SocialBonding      — compliment, confiance → ocytocine + endorphine
//   6. SeparationResponse — adieu, depart → cortisol + ocytocine baisse
//   7. EmpathicUrgency    — detresse d'autrui → ocytocine + cortisol leger
//   8. IntellectualStimulation — question profonde → dopamine + noradrenaline
//
// Modulation par sensibilite :
//   L'etat chimique actuel module les seuils de declenchement via un
//   `sensitivity_modifier`. Cortisol eleve = hypervigilance (seuils abaisses).
//   Serotonine elevee = calme (seuils releves).
// =============================================================================

use serde::{Deserialize, Serialize};

use crate::neurochemistry::{NeuroChemicalState, Molecule};
use crate::body::VirtualBody;

/// Type de reflexe pre-cable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflexType {
    /// Reaction a une menace (agression verbale, hostilite)
    ThreatResponse,
    /// Reaction a l'affection et la tendresse
    WarmthResponse,
    /// Capture d'attention (surprise, nouveaute)
    AttentionCapture,
    /// Alerte de danger explicite
    DangerAlert,
    /// Renforcement du lien social (compliment, confiance)
    SocialBonding,
    /// Reaction a la separation ou au depart
    SeparationResponse,
    /// Urgence empathique (detresse percue chez autrui)
    EmpathicUrgency,
    /// Stimulation intellectuelle (question profonde, defi)
    IntellectualStimulation,
}

/// Resultat d'un reflexe declenche.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflexResult {
    /// Type du reflexe declenche
    pub reflex_type: ReflexType,
    /// Intensite du reflexe [0.0, 1.0] — proportionnelle au nombre de mots-cles detectes
    pub intensity: f64,
    /// Deltas chimiques a appliquer (molecule, delta)
    pub chemistry_deltas: Vec<(Molecule, f64)>,
    /// Effets corporels (champ, delta)
    pub body_effects: Vec<(BodyEffect, f64)>,
}

/// Effets corporels pouvant etre declenches par un reflexe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyEffect {
    HeartRate,
    Tension,
    Warmth,
    Energy,
}

impl ReflexResult {
    /// Applique les deltas chimiques de ce reflexe sur l'etat chimique.
    /// Utilise `boost()` (rendements decroissants) pour eviter les saturations.
    pub fn apply_chemistry(&self, chemistry: &mut NeuroChemicalState) {
        for &(molecule, delta) in &self.chemistry_deltas {
            chemistry.boost(molecule, delta * self.intensity);
        }
    }
}

/// L'arc reflexe : ensemble de reflexes pre-cables avec modulation de sensibilite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflexArc {
    /// Modificateur de sensibilite global [0.5, 2.0].
    /// < 1.0 = reflexes emousses (calme), > 1.0 = hypervigilance.
    pub sensitivity_modifier: f64,
}

impl ReflexArc {
    pub fn new() -> Self {
        Self {
            sensitivity_modifier: 1.0,
        }
    }

    /// Evalue un signal textuel et retourne les reflexes declenches.
    ///
    /// Avant l'evaluation, la sensibilite est recalculee en fonction
    /// de l'etat chimique et corporel actuel.
    pub fn evaluate(
        &mut self,
        text: &str,
        chemistry: &NeuroChemicalState,
        body: &VirtualBody,
    ) -> Vec<ReflexResult> {
        // Recalculer la sensibilite en fonction de l'etat actuel
        self.update_sensitivity(chemistry, body);

        let text_lower = text.to_lowercase();
        let mut results = Vec::new();

        // Evaluer chaque type de reflexe
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

    /// Recalcule le modificateur de sensibilite en fonction de l'etat chimique.
    ///
    /// Cortisol eleve → hypervigilance (sensibilite augmente)
    /// Serotonine elevee → calme (sensibilite diminue)
    /// Adrenaline elevee → alerte maximale
    /// GABA eleve → inhibition (sensibilite diminue)
    fn update_sensitivity(&mut self, chemistry: &NeuroChemicalState, _body: &VirtualBody) {
        let mut modifier = 1.0;

        // Cortisol : stress = hypervigilance (+0.4 max a cortisol=1.0)
        modifier += (chemistry.cortisol - 0.3) * 0.8;

        // Serotonine : calme = reflexes emousses (-0.3 max a serotonine=1.0)
        modifier -= (chemistry.serotonin - 0.5) * 0.6;

        // Adrenaline : alerte (+0.3 max)
        modifier += (chemistry.adrenaline - 0.2) * 0.5;

        // GABA : inhibition (-0.2 max a gaba=1.0)
        modifier -= (chemistry.gaba - 0.5) * 0.4;

        self.sensitivity_modifier = modifier.clamp(0.5, 2.0);
    }

    /// Seuil effectif : seuil de base divise par la sensibilite.
    /// Plus la sensibilite est haute, plus le seuil est bas (hypervigilance).
    fn effective_threshold(&self, base_threshold: f64) -> f64 {
        (base_threshold / self.sensitivity_modifier).clamp(0.1, 1.0)
    }

    // ─── Reflexes individuels ──────────────────────────────────────────────

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

    // ─── Utilitaires ────────────────────────────────────────────────────

    /// Compte les mots-cles presents dans le texte et retourne un score [0.0, 1.0].
    /// Le score est base sur le nombre absolu de mots-cles trouves :
    ///   1 mot-cle → 0.15, 2 → 0.30, 3 → 0.45, etc. (plafonné à 1.0)
    /// Cela evite que les listes longues de mots-cles diluent le score.
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

        // Etat calme : sensibilite normale
        arc.update_sensitivity(&chemistry, &body);
        let normal_sensitivity = arc.sensitivity_modifier;

        // Etat stresse : sensibilite augmentee
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

        // Etat tres calme : sensibilite reduite
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
