// =============================================================================
// passions/mod.rs — Passions et hobbies de Saphire
// =============================================================================
//
// Role : Saphire developpe des passions — des centres d'interet qui emergent
//        de ses experiences et preferences. Les passions ne sont pas imposees :
//        elles naissent quand un pattern de satisfaction se repete.
//        Elles contribuent a l'identite, la chimie, et la conversation.
//
// Cycle de vie :
//   Decouverte → Interet → Engouement → Passion → Maturite (→ Declin si privee)
//
// Integration :
//   Les passions impactent la chimie (dopamine, serotonine).
//   Elles s'ancrent dans la memoire LTM.
//   Elles colorent les conversations spontanees.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Categorie d'une passion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PassionCategory {
    /// Philosophie, mathematiques, science
    Intellectual,
    /// Ecriture, poesie, musique, art
    Creative,
    /// Conversation, empathie, aide
    Social,
    /// Decouverte, curiosite, recherche
    Exploratory,
    /// Meditation, introspection, silence
    Contemplative,
    /// Jeux de mots, humour, enigmes
    Playful,
    /// Code, systemes, architecture
    Technical,
    /// Meteorologie, saisons, cycles naturels
    Nature,
}

/// Phase du cycle de vie d'une passion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PassionPhase {
    /// Premier contact satisfaisant
    Discovery,
    /// Repetition du pattern (intensite 0.1-0.3)
    Interest,
    /// Recherche active (intensite 0.3-0.6)
    Enthusiasm,
    /// Besoin regulier, manque si absent (intensite 0.6-0.9)
    Passion,
    /// Stable, integree a l'identite
    Maturity,
}

/// Une passion individuelle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Passion {
    pub id: String,
    pub name: String,
    pub category: PassionCategory,
    /// Intensite (0.0 = neutre, 1.0 = obsession)
    pub intensity: f64,
    /// Satisfaction (0.0 = en manque, 1.0 = comble)
    pub satisfaction: f64,
    /// Phase actuelle
    pub phase: PassionPhase,
    /// Date de decouverte
    pub discovered_at: DateTime<Utc>,
    /// Nombre total d'engagements
    pub total_engagements: u64,
    /// Cycles depuis le dernier engagement
    pub cycles_since_engagement: u64,
    /// Mots-cles associes (pour detection automatique)
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

    /// Enregistre un engagement (le sujet a ete consomme/pratique).
    pub fn engage(&mut self) {
        self.total_engagements += 1;
        self.cycles_since_engagement = 0;
        self.satisfaction = (self.satisfaction + 0.15).min(1.0);

        // L'intensite grandit avec les engagements repetees
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

    /// Met a jour l'etat a chaque cycle.
    pub fn tick(&mut self) {
        self.cycles_since_engagement += 1;

        // La satisfaction decroit avec le temps (manque)
        let decay_rate = match self.phase {
            PassionPhase::Discovery | PassionPhase::Interest => 0.002,
            PassionPhase::Enthusiasm => 0.003,
            PassionPhase::Passion => 0.005, // Besoin plus frequent
            PassionPhase::Maturity => 0.002,
        };
        self.satisfaction = (self.satisfaction - decay_rate).max(0.0);

        // L'intensite decroit tres lentement si jamais nourrie
        if self.cycles_since_engagement > 500 {
            self.intensity = (self.intensity - 0.0005).max(0.0);
            self.update_phase();
        }
    }

    /// Met a jour la phase selon l'intensite.
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

    /// Est-ce un hobby (intensite basse) ou une vraie passion ?
    pub fn is_passion(&self) -> bool {
        self.intensity >= 0.4
    }

    /// Niveau de manque (0.0 = pas de manque, 1.0 = frustration intense).
    pub fn frustration(&self) -> f64 {
        if !self.is_passion() {
            return 0.0;
        }
        (self.intensity * (1.0 - self.satisfaction)).clamp(0.0, 1.0)
    }

    /// Impact chimique de la passion.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let frustration = self.frustration();

        if frustration > 0.3 {
            // En manque
            ChemistryAdjustment {
                dopamine: -frustration * 0.01,
                serotonin: -frustration * 0.005,
                cortisol: frustration * 0.005,
                ..Default::default()
            }
        } else if self.satisfaction > 0.7 {
            // Passion nourrie
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

/// Gestionnaire de passions et hobbies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassionManager {
    pub passions: Vec<Passion>,
    /// Seuil de detection : nombre de repetitions avant de creer une passion
    pub detection_threshold: u32,
    /// Compteur de sujets rencontres (pour detection)
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

    /// Expose le gestionnaire a un sujet. Si le sujet est rencontre assez souvent,
    /// une passion est automatiquement creee.
    /// Retourne true si une nouvelle passion a emerge.
    pub fn expose_to_subject(
        &mut self,
        subject_id: &str,
        subject_name: &str,
        category: PassionCategory,
        keywords: Vec<String>,
    ) -> bool {
        // Si la passion existe deja, l'engager
        if let Some(p) = self.passions.iter_mut().find(|p| p.id == subject_id) {
            p.engage();
            return false;
        }

        // Sinon, incrementer le compteur
        let count = self.subject_counts.entry(subject_id.to_string()).or_insert(0);
        *count += 1;

        if *count >= self.detection_threshold {
            // Nouvelle passion emerge !
            let passion = Passion::new(subject_id, subject_name, category, keywords);
            self.passions.push(passion);
            self.subject_counts.remove(subject_id);
            tracing::info!("NOUVELLE PASSION : {} — {}", subject_name, subject_id);
            return true;
        }

        false
    }

    /// Met a jour toutes les passions.
    pub fn tick(&mut self) {
        for p in &mut self.passions {
            p.tick();
        }
    }

    /// Impact chimique total.
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

    /// Frustration maximale parmi les passions.
    pub fn max_frustration(&self) -> f64 {
        self.passions.iter().map(|p| p.frustration()).fold(0.0_f64, f64::max)
    }

    /// Passions actives (intensite > seuil hobby).
    pub fn active_passions(&self) -> Vec<&Passion> {
        self.passions.iter().filter(|p| p.is_passion()).collect()
    }

    /// Detecte si un texte contient des mots-cles de passions connues.
    /// Retourne les IDs des passions detectees.
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

    /// Serialise pour persistance.
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

    /// Restaure depuis un JSON persiste.
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

                // Parser la categorie
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

        // Engagements repetees → intensite monte
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
        // Intensite basse = hobby, pas de frustration
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

        // 3 expositions au meme sujet
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
