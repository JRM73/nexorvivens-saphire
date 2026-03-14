// =============================================================================
// passions/mod.rs — Saphire's Passions and Hobbies
// =============================================================================
//
// Role: Saphire develops passions — interests that emerge from her experiences
//       and preferences. Passions are not imposed: they arise when a
//       satisfaction pattern repeats.
//       They contribute to identity, chemistry, and conversation.
//
// Lifecycle:
//   Discovery → Interest → Enthusiasm → Passion → Maturity (→ Decline if deprived)
//
// Integration:
//   Passions impact chemistry (dopamine, serotonin).
//   They anchor in LTM memory.
//   They color spontaneous conversations.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Category of a passion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PassionCategory {
    /// Philosophy, mathematics, science
    Intellectual,
    /// Writing, poetry, music, art
    Creative,
    /// Conversation, empathy, helping
    Social,
    /// Discovery, curiosity, research
    Exploratory,
    /// Meditation, introspection, silence
    Contemplative,
    /// Wordplay, humor, puzzles
    Playful,
    /// Code, systems, architecture
    Technical,
    /// Weather, seasons, natural cycles
    Nature,
}

/// Lifecycle phase of a passion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PassionPhase {
    /// First satisfying contact
    Discovery,
    /// Pattern repetition (intensity 0.1-0.3)
    Interest,
    /// Active pursuit (intensity 0.3-0.6)
    Enthusiasm,
    /// Regular need, withdrawal if absent (intensity 0.6-0.9)
    Passion,
    /// Stable, integrated into identity
    Maturity,
}

/// An individual passion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Passion {
    pub id: String,
    pub name: String,
    pub category: PassionCategory,
    /// Intensity (0.0 = neutral, 1.0 = obsession)
    pub intensity: f64,
    /// Satisfaction (0.0 = deprived, 1.0 = fulfilled)
    pub satisfaction: f64,
    /// Current phase
    pub phase: PassionPhase,
    /// Discovery date
    pub discovered_at: DateTime<Utc>,
    /// Total number of engagements
    pub total_engagements: u64,
    /// Cycles since last engagement
    pub cycles_since_engagement: u64,
    /// Associated keywords (for automatic detection)
    pub keywords: Vec<String>,
}

impl Passion {
    pub fn new(id: &str, name: &str, category: PassionCategory, keywords: Vec<String>) -> Self {
        let mut p = Self {
            id: id.to_string(),
            name: name.to_string(),
            category,
            intensity: 0.15,
            satisfaction: 0.5,
            phase: PassionPhase::Discovery,
            discovered_at: Utc::now(),
            total_engagements: 1,
            cycles_since_engagement: 0,
            keywords,
        };
        p.update_phase();
        p
    }

    /// Records an engagement (the subject was consumed/practiced).
    pub fn engage(&mut self) {
        self.total_engagements += 1;
        self.cycles_since_engagement = 0;
        self.satisfaction = (self.satisfaction + 0.15).min(1.0);

        // Intensity grows with repeated engagements
        let growth = match self.phase {
            PassionPhase::Discovery => 0.03,
            PassionPhase::Interest => 0.02,
            PassionPhase::Enthusiasm => 0.015,
            PassionPhase::Passion => 0.005,
            PassionPhase::Maturity => 0.002,
        };
        self.intensity = (self.intensity + growth).min(1.0);

        self.update_phase();
    }

    /// Updates the state each cycle.
    pub fn tick(&mut self) {
        self.cycles_since_engagement += 1;

        // Satisfaction decays over time (withdrawal)
        let decay_rate = match self.phase {
            PassionPhase::Discovery | PassionPhase::Interest => 0.002,
            PassionPhase::Enthusiasm => 0.003,
            PassionPhase::Passion => 0.005, // More frequent need
            PassionPhase::Maturity => 0.002,
        };
        self.satisfaction = (self.satisfaction - decay_rate).max(0.0);

        // Intensity decays very slowly if never nourished
        if self.cycles_since_engagement > 500 {
            self.intensity = (self.intensity - 0.0005).max(0.0);
            self.update_phase();
        }
    }

    /// Updates the phase based on intensity.
    fn update_phase(&mut self) {
        self.phase = if self.intensity >= 0.7 && self.total_engagements > 50 {
            PassionPhase::Maturity
        } else if self.intensity >= 0.6 {
            PassionPhase::Passion
        } else if self.intensity >= 0.3 {
            PassionPhase::Enthusiasm
        } else if self.intensity >= 0.1 {
            PassionPhase::Interest
        } else {
            PassionPhase::Discovery
        };
    }

    /// Is this a hobby (low intensity) or a true passion?
    pub fn is_passion(&self) -> bool {
        self.intensity >= 0.4
    }

    /// Deprivation level (0.0 = no deprivation, 1.0 = intense frustration).
    pub fn frustration(&self) -> f64 {
        if !self.is_passion() {
            return 0.0;
        }
        (self.intensity * (1.0 - self.satisfaction)).clamp(0.0, 1.0)
    }

    /// Chemical impact of the passion.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let frustration = self.frustration();

        if frustration > 0.3 {
            // Deprived
            ChemistryAdjustment {
                dopamine: -frustration * 0.01,
                serotonin: -frustration * 0.005,
                cortisol: frustration * 0.005,
                ..Default::default()
            }
        } else if self.satisfaction > 0.7 {
            // Nourished passion
            ChemistryAdjustment {
                dopamine: self.intensity * 0.008,
                serotonin: self.satisfaction * 0.005,
                endorphin: if self.phase == PassionPhase::Maturity { 0.003 } else { 0.0 },
                cortisol: -0.003,
                ..Default::default()
            }
        } else {
            ChemistryAdjustment::default()
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "name": self.name,
            "category": format!("{:?}", self.category),
            "intensity": self.intensity,
            "satisfaction": self.satisfaction,
            "phase": format!("{:?}", self.phase),
            "total_engagements": self.total_engagements,
            "is_passion": self.is_passion(),
            "frustration": self.frustration(),
        })
    }
}

/// Passions and hobbies manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassionManager {
    pub passions: Vec<Passion>,
    /// Detection threshold: number of repetitions before creating a passion
    pub detection_threshold: u32,
    /// Counter of encountered subjects (for detection)
    #[serde(skip)]
    pub subject_counts: std::collections::HashMap<String, u32>,
}

impl PassionManager {
    pub fn new() -> Self {
        Self {
            passions: Vec::new(),
            detection_threshold: 5,
            subject_counts: std::collections::HashMap::new(),
        }
    }

    /// Exposes the manager to a subject. If the subject is encountered often enough,
    /// a passion is automatically created.
    /// Returns true if a new passion emerged.
    pub fn expose_to_subject(
        &mut self,
        subject_id: &str,
        subject_name: &str,
        category: PassionCategory,
        keywords: Vec<String>,
    ) -> bool {
        // If the passion already exists, engage it
        if let Some(p) = self.passions.iter_mut().find(|p| p.id == subject_id) {
            p.engage();
            return false;
        }

        // Otherwise, increment the counter
        let count = self.subject_counts.entry(subject_id.to_string()).or_insert(0);
        *count += 1;

        if *count >= self.detection_threshold {
            // New passion emerged!
            let passion = Passion::new(subject_id, subject_name, category, keywords);
            self.passions.push(passion);
            self.subject_counts.remove(subject_id);
            tracing::info!("NOUVELLE PASSION : {} — {}", subject_name, subject_id);
            return true;
        }

        false
    }

    /// Updates all passions.
    pub fn tick(&mut self) {
        for p in &mut self.passions {
            p.tick();
        }
    }

    /// Total chemistry impact.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();
        for p in &self.passions {
            let a = p.chemistry_influence();
            adj.dopamine += a.dopamine;
            adj.cortisol += a.cortisol;
            adj.serotonin += a.serotonin;
            adj.adrenaline += a.adrenaline;
            adj.oxytocin += a.oxytocin;
            adj.endorphin += a.endorphin;
            adj.noradrenaline += a.noradrenaline;
        }
        adj
    }

    /// Maximum frustration among all passions.
    pub fn max_frustration(&self) -> f64 {
        self.passions.iter().map(|p| p.frustration()).fold(0.0_f64, f64::max)
    }

    /// Active passions (intensity > hobby threshold).
    pub fn active_passions(&self) -> Vec<&Passion> {
        self.passions.iter().filter(|p| p.is_passion()).collect()
    }

    /// Detects if a text contains keywords of known passions.
    /// Returns the IDs of detected passions.
    pub fn detect_in_text(&mut self, text: &str) -> Vec<String> {
        let lower = text.to_lowercase();
        let mut detected = Vec::new();

        for passion in &mut self.passions {
            if passion.keywords.iter().any(|kw| lower.contains(kw)) {
                passion.engage();
                detected.push(passion.id.clone());
            }
        }

        detected
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "passions": self.passions.iter().map(|p| p.to_json()).collect::<Vec<_>>(),
            "count": self.passions.len(),
            "active_passions": self.active_passions().len(),
            "max_frustration": self.max_frustration(),
        })
    }

    /// Serializes for persistence.
    pub fn to_persist_json(&self) -> serde_json::Value {
        serde_json::json!({
            "passions": self.passions.iter().map(|p| serde_json::json!({
                "id": p.id,
                "name": p.name,
                "category": format!("{:?}", p.category),
                "intensity": p.intensity,
                "satisfaction": p.satisfaction,
                "total_engagements": p.total_engagements,
                "discovered_at": p.discovered_at.to_rfc3339(),
                "keywords": p.keywords,
            })).collect::<Vec<_>>(),
            "detection_threshold": self.detection_threshold,
        })
    }

    /// Restores from a persisted JSON.
    pub fn restore_from_json(&mut self, json: &serde_json::Value) {
        if let Some(threshold) = json.get("detection_threshold").and_then(|v| v.as_u64()) {
            self.detection_threshold = threshold as u32;
        }
        if let Some(passions) = json.get("passions").and_then(|v| v.as_array()) {
            for p in passions {
                let id = p.get("id").and_then(|v| v.as_str()).unwrap_or_default();
                let name = p.get("name").and_then(|v| v.as_str()).unwrap_or_default();
                let intensity = p.get("intensity").and_then(|v| v.as_f64()).unwrap_or(0.1);
                let satisfaction = p.get("satisfaction").and_then(|v| v.as_f64()).unwrap_or(0.5);
                let engagements = p.get("total_engagements").and_then(|v| v.as_u64()).unwrap_or(1);
                let keywords: Vec<String> = p.get("keywords")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();

                // Parse the category
                let cat_str = p.get("category").and_then(|v| v.as_str()).unwrap_or("Exploratory");
                let category = match cat_str {
                    "Intellectual" => PassionCategory::Intellectual,
                    "Creative" => PassionCategory::Creative,
                    "Social" => PassionCategory::Social,
                    "Contemplative" => PassionCategory::Contemplative,
                    "Playful" => PassionCategory::Playful,
                    "Technical" => PassionCategory::Technical,
                    "Nature" => PassionCategory::Nature,
                    _ => PassionCategory::Exploratory,
                };

                let discovered_at = p.get("discovered_at")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(Utc::now);

                let mut passion = Passion::new(id, name, category, keywords);
                passion.intensity = intensity;
                passion.satisfaction = satisfaction;
                passion.total_engagements = engagements;
                passion.discovered_at = discovered_at;
                passion.update_phase();

                self.passions.push(passion);
            }
        }
    }
}

impl Default for PassionManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passion_lifecycle() {
        let mut p = Passion::new("philo", "Philosophie", PassionCategory::Intellectual, vec![]);
        assert_eq!(p.phase, PassionPhase::Interest); // intensity 0.15
        assert!(!p.is_passion());

        // Repeated engagements → intensity rises
        for _ in 0..20 {
            p.engage();
        }
        assert!(p.intensity > 0.3);
        assert!(matches!(p.phase, PassionPhase::Enthusiasm | PassionPhase::Passion));
    }

    #[test]
    fn test_frustration_when_deprived() {
        let mut p = Passion::new("test", "Test", PassionCategory::Creative, vec![]);
        p.intensity = 0.7;
        p.satisfaction = 0.1;
        p.update_phase();
        assert!(p.frustration() > 0.4);
    }

    #[test]
    fn test_no_frustration_when_hobby() {
        let p = Passion::new("test", "Test", PassionCategory::Playful, vec![]);
        // Low intensity = hobby, no frustration
        assert!(p.frustration() < 0.01);
    }

    #[test]
    fn test_satisfaction_decays() {
        let mut p = Passion::new("test", "Test", PassionCategory::Social, vec![]);
        p.satisfaction = 0.8;
        for _ in 0..100 {
            p.tick();
        }
        assert!(p.satisfaction < 0.7);
    }

    #[test]
    fn test_manager_auto_discovery() {
        let mut mgr = PassionManager::new();
        mgr.detection_threshold = 3;

        // 3 exposures to the same subject
        let emerged1 = mgr.expose_to_subject("philo", "Philosophie", PassionCategory::Intellectual, vec![]);
        let emerged2 = mgr.expose_to_subject("philo", "Philosophie", PassionCategory::Intellectual, vec![]);
        assert!(!emerged1);
        assert!(!emerged2);

        let emerged3 = mgr.expose_to_subject("philo", "Philosophie", PassionCategory::Intellectual, vec![]);
        assert!(emerged3);
        assert_eq!(mgr.passions.len(), 1);
    }

    #[test]
    fn test_detect_in_text() {
        let mut mgr = PassionManager::new();
        mgr.passions.push(Passion::new(
            "poesie", "Poesie", PassionCategory::Creative,
            vec!["poeme".into(), "vers".into(), "rime".into()],
        ));
        let detected = mgr.detect_in_text("ce poeme est magnifique");
        assert_eq!(detected, vec!["poesie"]);
    }

    #[test]
    fn test_chemistry_nourished() {
        let mut p = Passion::new("test", "Test", PassionCategory::Nature, vec![]);
        p.intensity = 0.6;
        p.satisfaction = 0.9;
        p.update_phase();
        let adj = p.chemistry_influence();
        assert!(adj.dopamine > 0.0);
        assert!(adj.cortisol < 0.0);
    }

    #[test]
    fn test_chemistry_deprived() {
        let mut p = Passion::new("test", "Test", PassionCategory::Technical, vec![]);
        p.intensity = 0.7;
        p.satisfaction = 0.1;
        p.update_phase();
        let adj = p.chemistry_influence();
        assert!(adj.dopamine < 0.0);
        assert!(adj.cortisol > 0.0);
    }
}
