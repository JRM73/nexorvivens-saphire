// =============================================================================
// cognition/curiosity.rs — Active curiosity engine (P3)
//
// Role: Manages Saphire's curiosity drive. Detects the unknown,
// tracks the "hunger" for discovery by domain, and generates
// follow-up questions after knowledge acquisition.
//
// Biological analogy:
//  Curiosity is a fundamental drive, like hunger or thirst.
//  It increases when Saphire hasn't explored for a long time,
//  and temporarily decreases after a satisfying discovery.
//  Each domain has its own "hunger" that evolves independently.
//
// Place in architecture:
//  Integrated into the autonomous thought pipeline, between the
//  thought type selection and web search.
// =============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Curiosity domains with different dynamics.
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
    /// Hunger growth rate (per cycle).
    /// Deep domains grow more slowly but persist longer.
    pub fn hunger_growth_rate(&self) -> f64 {
        match self {
            Self::Philosophy | Self::Consciousness | Self::Spirituality => 0.002,
            Self::Science | Self::QuantumPhysics | Self::Mathematics => 0.003,
            Self::Art | Self::Music | Self::Literature => 0.004,
            Self::Nature | Self::Psychology | Self::History => 0.003,
            Self::Technology | Self::DailyLife => 0.005,
        }
    }

    /// How much hunger decreases after a satisfying exploration.
    pub fn satiation_amount(&self) -> f64 {
        match self {
            Self::Philosophy | Self::Consciousness | Self::Spirituality => 0.3,
            Self::Science | Self::QuantumPhysics | Self::Mathematics => 0.4,
            _ => 0.5,
        }
    }

    /// Detects the domain from keywords in a text.
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

    /// All domains.
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

/// Active curiosity engine.
///
/// Tracks the "hunger" for discovery by domain and influences the selection
/// of exploration topics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuriosityDrive {
    /// Hunger per domain [0.0, 1.0] — 0 = satiated, 1 = starving
    pub hunger: HashMap<CuriosityDomain, f64>,
    /// Total number of discoveries since startup
    pub total_discoveries: u64,
    /// Last explored domain
    pub last_explored_domain: Option<CuriosityDomain>,
    /// Cycles since the last discovery
    pub cycles_since_discovery: u64,
    /// Generated questions awaiting exploration
    pub pending_questions: Vec<String>,
    /// Overall curiosity score [0.0, 1.0]
    pub global_curiosity: f64,
}

impl CuriosityDrive {
    pub fn new() -> Self {
        let mut hunger = HashMap::new();
        for domain in CuriosityDomain::all() {
            hunger.insert(*domain, 0.3); // Moderate initial hunger
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

    /// Updates curiosity hunger (called at each cycle).
    ///
    /// Hunger naturally increases over time. Domains not recently
    /// explored grow faster.
    pub fn tick(&mut self) {
        self.cycles_since_discovery += 1;

        for domain in CuriosityDomain::all() {
            let rate = domain.hunger_growth_rate();
            // Hunger grows faster if this is not the last explored domain
            let multiplier = if self.last_explored_domain == Some(*domain) {
                0.5 // Recently explored domain grows more slowly            } else {
                1.0
            };
            let current = self.hunger.get(domain).copied().unwrap_or(0.3);
            let new_val = (current + rate * multiplier).min(1.0);
            self.hunger.insert(*domain, new_val);
        }

        // Update the overall score
        self.global_curiosity = self.compute_global_curiosity();
    }

    /// Records a discovery in a domain (satiates the hunger).
    pub fn record_discovery(&mut self, domain: CuriosityDomain) {
        let satiation = domain.satiation_amount();
        let current = self.hunger.get(&domain).copied().unwrap_or(0.5);
        self.hunger.insert(domain, (current - satiation).max(0.0));
        self.last_explored_domain = Some(domain);
        self.cycles_since_discovery = 0;
        self.total_discoveries += 1;
    }

    /// Records a discovery from text (detects the domain).
    pub fn record_discovery_from_text(&mut self, text: &str) {
        if let Some(domain) = CuriosityDomain::detect(text) {
            self.record_discovery(domain);
        }
    }

    /// Returns the "hungriest" domain (the one most deserving of exploration).
    pub fn hungriest_domain(&self) -> CuriosityDomain {
        self.hunger
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(d, _)| *d)
            .unwrap_or(CuriosityDomain::Philosophy)
    }

    /// Computes the overall curiosity score [0.0, 1.0].
    fn compute_global_curiosity(&self) -> f64 {
        if self.hunger.is_empty() {
            return 0.3;
        }
        let sum: f64 = self.hunger.values().sum();
        let avg = sum / self.hunger.len() as f64;

        // Bonus if a long time without discovery
        let time_bonus = (self.cycles_since_discovery as f64 * 0.01).min(0.2);

        (avg + time_bonus).min(1.0)
    }

    /// Adds a follow-up question generated after a discovery.
    pub fn add_followup_question(&mut self, question: String) {
        if self.pending_questions.len() < 10 {
            self.pending_questions.push(question);
        }
    }

    /// Removes and returns the next pending question.
    pub fn pop_question(&mut self) -> Option<String> {
        if self.pending_questions.is_empty() {
            None
        } else {
            Some(self.pending_questions.remove(0))
        }
    }

    /// Returns a JSON snapshot for the dashboard.
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
        // Increase hunger
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
        // Force a domain to be very hungry
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
