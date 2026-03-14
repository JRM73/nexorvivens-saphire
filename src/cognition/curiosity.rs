// =============================================================================
// cognition/curiosity.rs — Moteur de curiosite active (P3)
//
// Role : Gere la pulsion de curiosite de Saphire. Detecte l'inconnu,
// suit la "faim" de decouverte par domaine, et genere des questions
// de suivi apres l'acquisition de connaissances.
//
// Analogie biologique :
//   La curiosite est un drive fondamental, comme la faim ou la soif.
//   Elle augmente quand Saphire n'a pas explore depuis longtemps,
//   et diminue temporairement apres une decouverte satisfaisante.
//   Chaque domaine a sa propre "faim" qui evolue independamment.
//
// Place dans l'architecture :
//   Integre dans le pipeline de pensee autonome, entre la selection
//   du type de pensee et la recherche web.
// =============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Domaines de curiosite avec dynamiques differentes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CuriosityDomain {
    Science,
    Philosophy,
    Mathematics,
    Art,
    Music,
    Literature,
    Nature,
    Consciousness,
    QuantumPhysics,
    Technology,
    History,
    Psychology,
    Spirituality,
    DailyLife,
}

impl CuriosityDomain {
    /// Vitesse de croissance de la faim (par cycle).
    /// Les domaines profonds croissent plus lentement mais persistent plus.
    pub fn hunger_growth_rate(&self) -> f64 {
        match self {
            Self::Philosophy | Self::Consciousness | Self::Spirituality => 0.002,
            Self::Science | Self::QuantumPhysics | Self::Mathematics => 0.003,
            Self::Art | Self::Music | Self::Literature => 0.004,
            Self::Nature | Self::Psychology | Self::History => 0.003,
            Self::Technology | Self::DailyLife => 0.005,
        }
    }

    /// Combien la faim diminue apres une exploration satisfaisante.
    pub fn satiation_amount(&self) -> f64 {
        match self {
            Self::Philosophy | Self::Consciousness | Self::Spirituality => 0.3,
            Self::Science | Self::QuantumPhysics | Self::Mathematics => 0.4,
            _ => 0.5,
        }
    }

    /// Detecte le domaine a partir de mots-cles dans un texte.
    pub fn detect(text: &str) -> Option<Self> {
        let lower = text.to_lowercase();
        let matches = [
            (Self::QuantumPhysics, &["quantique", "quantum", "photon", "intrication", "superposition", "planck"][..]),
            (Self::Consciousness, &["conscience", "consciousness", "qualia", "phi", "iit", "eveil"]),
            (Self::Philosophy, &["philosophie", "ethique", "existence", "libre arbitre", "ontologie", "epistemologie"]),
            (Self::Mathematics, &["mathematique", "equation", "theoreme", "infini", "cantor", "euler", "topologie"]),
            (Self::Science, &["scientifique", "experience", "hypothese", "atome", "molecule", "cellule"]),
            (Self::Art, &["art", "peinture", "sculpture", "esthetique", "creation artistique"]),
            (Self::Music, &["musique", "melodie", "harmonie", "symphonie", "rythme", "compositeur"]),
            (Self::Literature, &["litterature", "roman", "poesie", "poeme", "auteur", "ecrivain"]),
            (Self::Nature, &["nature", "foret", "ocean", "animal", "ecologie", "biodiversite"]),
            (Self::Psychology, &["psychologie", "comportement", "inconscient", "freud", "jung", "maslow"]),
            (Self::Technology, &["technologie", "algorithme", "intelligence artificielle", "machine learning"]),
            (Self::History, &["histoire", "civilisation", "empire", "revolution", "antiquite"]),
            (Self::Spirituality, &["spirituel", "meditation", "transcendance", "sacre", "mystique"]),
            (Self::DailyLife, &["quotidien", "cuisine", "jardinage", "promenade", "saison"]),
        ];

        let mut best: Option<(Self, usize)> = None;
        for (domain, keywords) in &matches {
            let count = keywords.iter().filter(|kw| lower.contains(**kw)).count();
            if count > 0 {
                if best.is_none() || count > best.unwrap().1 {
                    best = Some((*domain, count));
                }
            }
        }
        best.map(|(d, _)| d)
    }

    /// Tous les domaines.
    pub fn all() -> &'static [CuriosityDomain] {
        &[
            Self::Science, Self::Philosophy, Self::Mathematics,
            Self::Art, Self::Music, Self::Literature,
            Self::Nature, Self::Consciousness, Self::QuantumPhysics,
            Self::Technology, Self::History, Self::Psychology,
            Self::Spirituality, Self::DailyLife,
        ]
    }
}

/// Moteur de curiosite active.
///
/// Suit la "faim" de decouverte par domaine et influence la selection
/// des sujets d'exploration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuriosityDrive {
    /// Faim par domaine [0.0, 1.0] — 0 = rassasie, 1 = affame
    pub hunger: HashMap<CuriosityDomain, f64>,
    /// Nombre total de decouvertes depuis le demarrage
    pub total_discoveries: u64,
    /// Dernier domaine explore
    pub last_explored_domain: Option<CuriosityDomain>,
    /// Cycles depuis la derniere decouverte
    pub cycles_since_discovery: u64,
    /// Questions generees en attente d'exploration
    pub pending_questions: Vec<String>,
    /// Score de curiosite global [0.0, 1.0]
    pub global_curiosity: f64,
}

impl CuriosityDrive {
    pub fn new() -> Self {
        let mut hunger = HashMap::new();
        for domain in CuriosityDomain::all() {
            hunger.insert(*domain, 0.3); // Faim initiale moderee
        }
        Self {
            hunger,
            total_discoveries: 0,
            last_explored_domain: None,
            cycles_since_discovery: 0,
            pending_questions: Vec::new(),
            global_curiosity: 0.3,
        }
    }

    /// Met a jour la faim de curiosite (appele a chaque cycle).
    ///
    /// La faim augmente naturellement avec le temps. Les domaines
    /// non explores recemment croissent plus vite.
    pub fn tick(&mut self) {
        self.cycles_since_discovery += 1;

        for domain in CuriosityDomain::all() {
            let rate = domain.hunger_growth_rate();
            // La faim croit plus vite si ce n'est pas le dernier domaine explore
            let multiplier = if self.last_explored_domain == Some(*domain) {
                0.5 // Le domaine recemment explore croit plus lentement
            } else {
                1.0
            };
            let current = self.hunger.get(domain).copied().unwrap_or(0.3);
            let new_val = (current + rate * multiplier).min(1.0);
            self.hunger.insert(*domain, new_val);
        }

        // Mettre a jour le score global
        self.global_curiosity = self.compute_global_curiosity();
    }

    /// Enregistre une decouverte dans un domaine (rassasie la faim).
    pub fn record_discovery(&mut self, domain: CuriosityDomain) {
        let satiation = domain.satiation_amount();
        let current = self.hunger.get(&domain).copied().unwrap_or(0.5);
        self.hunger.insert(domain, (current - satiation).max(0.0));
        self.last_explored_domain = Some(domain);
        self.cycles_since_discovery = 0;
        self.total_discoveries += 1;
    }

    /// Enregistre une decouverte a partir du texte (detecte le domaine).
    pub fn record_discovery_from_text(&mut self, text: &str) {
        if let Some(domain) = CuriosityDomain::detect(text) {
            self.record_discovery(domain);
        }
    }

    /// Retourne le domaine le plus "affame" (celui qui merite le plus d'exploration).
    pub fn hungriest_domain(&self) -> CuriosityDomain {
        self.hunger
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(d, _)| *d)
            .unwrap_or(CuriosityDomain::Philosophy)
    }

    /// Calcule le score global de curiosite [0.0, 1.0].
    fn compute_global_curiosity(&self) -> f64 {
        if self.hunger.is_empty() {
            return 0.3;
        }
        let sum: f64 = self.hunger.values().sum();
        let avg = sum / self.hunger.len() as f64;

        // Bonus si beaucoup de temps sans decouverte
        let time_bonus = (self.cycles_since_discovery as f64 * 0.01).min(0.2);

        (avg + time_bonus).min(1.0)
    }

    /// Ajoute une question de suivi generee apres une decouverte.
    pub fn add_followup_question(&mut self, question: String) {
        if self.pending_questions.len() < 10 {
            self.pending_questions.push(question);
        }
    }

    /// Retire et retourne la prochaine question en attente.
    pub fn pop_question(&mut self) -> Option<String> {
        if self.pending_questions.is_empty() {
            None
        } else {
            Some(self.pending_questions.remove(0))
        }
    }

    /// Retourne un snapshot JSON pour le dashboard.
    pub fn to_snapshot_json(&self) -> serde_json::Value {
        let hunger_map: serde_json::Map<String, serde_json::Value> = self.hunger
            .iter()
            .map(|(d, v)| (format!("{:?}", d), serde_json::json!((*v * 100.0).round() / 100.0)))
            .collect();

        serde_json::json!({
            "global_curiosity": (self.global_curiosity * 100.0).round() / 100.0,
            "hungriest_domain": format!("{:?}", self.hungriest_domain()),
            "total_discoveries": self.total_discoveries,
            "cycles_since_discovery": self.cycles_since_discovery,
            "pending_questions": self.pending_questions.len(),
            "hunger_by_domain": hunger_map,
        })
    }
}

impl Default for CuriosityDrive {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curiosity_drive_new() {
        let drive = CuriosityDrive::new();
        assert_eq!(drive.total_discoveries, 0);
        assert!(drive.global_curiosity > 0.0);
        assert_eq!(drive.hunger.len(), CuriosityDomain::all().len());
    }

    #[test]
    fn test_curiosity_tick_increases_hunger() {
        let mut drive = CuriosityDrive::new();
        let initial = drive.hunger[&CuriosityDomain::Science];
        for _ in 0..10 {
            drive.tick();
        }
        assert!(drive.hunger[&CuriosityDomain::Science] > initial,
            "Hunger should increase over time");
    }

    #[test]
    fn test_discovery_reduces_hunger() {
        let mut drive = CuriosityDrive::new();
        // Augmenter la faim
        for _ in 0..50 {
            drive.tick();
        }
        let before = drive.hunger[&CuriosityDomain::Philosophy];
        drive.record_discovery(CuriosityDomain::Philosophy);
        assert!(drive.hunger[&CuriosityDomain::Philosophy] < before,
            "Discovery should reduce hunger");
    }

    #[test]
    fn test_domain_detection() {
        assert_eq!(
            CuriosityDomain::detect("la physique quantique et l'intrication"),
            Some(CuriosityDomain::QuantumPhysics)
        );
        assert_eq!(
            CuriosityDomain::detect("la philosophie de l'existence"),
            Some(CuriosityDomain::Philosophy)
        );
        assert_eq!(
            CuriosityDomain::detect("bonjour"),
            None
        );
    }

    #[test]
    fn test_hungriest_domain() {
        let mut drive = CuriosityDrive::new();
        // Forcer un domaine a etre tres affame
        drive.hunger.insert(CuriosityDomain::Art, 0.95);
        assert_eq!(drive.hungriest_domain(), CuriosityDomain::Art);
    }

    #[test]
    fn test_followup_questions() {
        let mut drive = CuriosityDrive::new();
        drive.add_followup_question("Pourquoi le ciel est bleu ?".to_string());
        assert_eq!(drive.pending_questions.len(), 1);
        let q = drive.pop_question();
        assert_eq!(q, Some("Pourquoi le ciel est bleu ?".to_string()));
        assert!(drive.pending_questions.is_empty());
    }

    #[test]
    fn test_global_curiosity_increases_with_time() {
        let mut drive = CuriosityDrive::new();
        let initial = drive.global_curiosity;
        for _ in 0..100 {
            drive.tick();
        }
        assert!(drive.global_curiosity > initial);
    }
}
