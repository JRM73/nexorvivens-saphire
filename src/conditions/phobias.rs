// =============================================================================
// conditions/phobias.rs — Phobia system
// =============================================================================
//
// Purpose: Models phobias — irrational fears triggered by specific
//          keywords detected in text. Each phobia has an intensity,
//          triggers, and can be progressively desensitized
//          through repeated safe exposure.
//
// Mechanics:
//   1. Detection: scan text for trigger_keywords
//   2. Reaction: cortisol spike, adrenaline spike
//   3. Panic: if intensity is high, confused thoughts
//   4. Desensitization: repeated exposures -> intensity decreases
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// An individual phobia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phobia {
    /// Name of the phobia (e.g., "claustrophobia")
    pub name: String,
    /// Keywords that trigger the phobia (lowercase)
    pub trigger_keywords: Vec<String>,
    /// Phobia intensity (0.0 = mild, 1.0 = paralyzing)
    pub intensity: f64,
    /// Desensitization progress (0.0 = none, 1.0 = cured)
    pub desensitization: f64,
    /// Number of times triggered
    pub times_triggered: u64,
    /// Last trigger time
    pub last_triggered: Option<DateTime<Utc>>,
}

impl Phobia {
    /// Creates a new phobia.
    pub fn new(name: &str, triggers: Vec<String>, intensity: f64) -> Self {
        Self {
            name: name.to_string(),
            trigger_keywords: triggers,
            intensity: intensity.clamp(0.0, 1.0),
            desensitization: 0.0,
            times_triggered: 0,
            last_triggered: None,
        }
    }

    /// Effective intensity (reduced by desensitization).
    pub fn effective_intensity(&self) -> f64 {
        (self.intensity * (1.0 - self.desensitization)).clamp(0.0, 1.0)
    }

    /// Checks if the text contains a trigger.
    pub fn is_triggered_by(&self, text: &str) -> bool {
        let lower = text.to_lowercase();
        self.trigger_keywords.iter().any(|kw| lower.contains(kw))
    }

    /// Records a trigger event and applies desensitization.
    pub fn trigger(&mut self, desensitization_rate: f64) {
        self.times_triggered += 1;
        self.last_triggered = Some(Utc::now());

        // Progressive desensitization (each safe exposure)
        self.desensitization = (self.desensitization + desensitization_rate).min(1.0);
    }
}

/// Phobia manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhobiaManager {
    /// List of active phobias
    pub phobias: Vec<Phobia>,
    /// Desensitization rate per exposure
    pub desensitization_rate: f64,
    /// Last triggered phobia (for context)
    #[serde(skip)]
    pub last_triggered_name: Option<String>,
    /// Intensity of the last trigger
    #[serde(skip)]
    pub last_trigger_intensity: f64,
}

impl PhobiaManager {
    /// Creates an empty manager.
    pub fn new(desensitization_rate: f64) -> Self {
        Self {
            phobias: Vec::new(),
            desensitization_rate,
            last_triggered_name: None,
            last_trigger_intensity: 0.0,
        }
    }

    /// Adds a phobia.
    pub fn add(&mut self, phobia: Phobia) {
        self.phobias.push(phobia);
    }

    /// Scans text and triggers matching phobias.
    /// Returns the number of phobias triggered.
    pub fn scan_text(&mut self, text: &str) -> u32 {
        let mut triggered = 0;
        let rate = self.desensitization_rate;
        let mut strongest_name: Option<String> = None;
        let mut strongest_intensity: f64 = 0.0;

        for phobia in &mut self.phobias {
            if phobia.is_triggered_by(text) {
                phobia.trigger(rate);
                triggered += 1;
                let eff = phobia.effective_intensity();
                if eff > strongest_intensity {
                    strongest_intensity = eff;
                    strongest_name = Some(phobia.name.clone());
                }
            }
        }

        self.last_triggered_name = strongest_name;
        self.last_trigger_intensity = strongest_intensity;
        triggered
    }

    /// Chemistry impact of phobias triggered this cycle.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        if self.last_trigger_intensity < 0.05 {
            return ChemistryAdjustment::default();
        }

        let i = self.last_trigger_intensity;
        ChemistryAdjustment {
            cortisol: i * 0.30,        // Intense stress
            adrenaline: i * 0.25,      // Flight reaction
            serotonin: -i * 0.10,      // Well-being drops
            endorphin: i * 0.05,       // Analgesic response
            noradrenaline: i * 0.15,   // Vigilance
            ..Default::default()
        }
    }

    /// Resets the last trigger (at start of cycle).
    pub fn reset_cycle(&mut self) {
        self.last_triggered_name = None;
        self.last_trigger_intensity = 0.0;
    }

    /// Serializes for the API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "phobias": self.phobias.iter().map(|p| serde_json::json!({
                "name": p.name,
                "triggers": p.trigger_keywords,
                "intensity": p.intensity,
                "effective_intensity": p.effective_intensity(),
                "desensitization": p.desensitization,
                "times_triggered": p.times_triggered,
                "last_triggered": p.last_triggered,
            })).collect::<Vec<_>>(),
            "last_triggered": self.last_triggered_name,
            "last_intensity": self.last_trigger_intensity,
        })
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phobia_trigger() {
        let p = Phobia::new("claustrophobie", vec!["enferme".into(), "etroit".into()], 0.7);
        assert!(p.is_triggered_by("je suis enferme dans un espace etroit"));
        assert!(!p.is_triggered_by("le ciel est bleu"));
    }

    #[test]
    fn test_desensitization() {
        let mut p = Phobia::new("test", vec!["mot".into()], 0.8);
        let initial = p.effective_intensity();

        // 10 exposures
        for _ in 0..10 {
            p.trigger(0.05);
        }
        assert!(p.effective_intensity() < initial);
        assert!(p.desensitization > 0.0);
    }

    #[test]
    fn test_manager_scan() {
        let mut mgr = PhobiaManager::new(0.005);
        mgr.add(Phobia::new("arachnophobie", vec!["araignee".into(), "spider".into()], 0.6));
        mgr.add(Phobia::new("claustrophobie", vec!["enferme".into()], 0.8));

        let count = mgr.scan_text("il y a une araignee dans la piece");
        assert_eq!(count, 1);
        assert_eq!(mgr.last_triggered_name.as_deref(), Some("arachnophobie"));

        mgr.reset_cycle();
        let count = mgr.scan_text("rien de special");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_chemistry_influence() {
        let mut mgr = PhobiaManager::new(0.005);
        mgr.add(Phobia::new("test", vec!["peur".into()], 0.8));
        mgr.scan_text("j'ai peur");
        let adj = mgr.chemistry_influence();
        assert!(adj.cortisol > 0.0);
        assert!(adj.adrenaline > 0.0);
    }
}
