// =============================================================================
// conditions/phobias.rs — Systeme de phobies
// =============================================================================
//
// Role : Modelise les phobies — peurs irrationnelles declenchees par des
//        mots-cles specifiques detectes dans le texte. Chaque phobie a une
//        intensite, des declencheurs, et peut etre desensibilisee
//        progressivement par exposition repetee sans danger.
//
// Mecanique :
//   1. Detection : scan du texte pour les trigger_keywords
//   2. Reaction : cortisol spike, adrenaline spike
//   3. Panique : si intensite elevee, pensees confuses
//   4. Desensibilisation : expositions repetees → intensite diminue
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Une phobie individuelle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phobia {
    /// Nom de la phobie (ex: "claustrophobie")
    pub name: String,
    /// Mots-cles qui declenchent la phobie (minuscules)
    pub trigger_keywords: Vec<String>,
    /// Intensite de la phobie (0.0 = legere, 1.0 = paralysante)
    pub intensity: f64,
    /// Progres de desensibilisation (0.0 = aucun, 1.0 = guerie)
    pub desensitization: f64,
    /// Nombre de fois declenchee
    pub times_triggered: u64,
    /// Dernier declenchement
    pub last_triggered: Option<DateTime<Utc>>,
}

impl Phobia {
    /// Cree une nouvelle phobie.
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

    /// Intensite effective (reduite par la desensibilisation).
    pub fn effective_intensity(&self) -> f64 {
        (self.intensity * (1.0 - self.desensitization)).clamp(0.0, 1.0)
    }

    /// Verifie si le texte contient un declencheur.
    pub fn is_triggered_by(&self, text: &str) -> bool {
        let lower = text.to_lowercase();
        self.trigger_keywords.iter().any(|kw| lower.contains(kw))
    }

    /// Enregistre un declenchement et applique la desensibilisation.
    pub fn trigger(&mut self, desensitization_rate: f64) {
        self.times_triggered += 1;
        self.last_triggered = Some(Utc::now());

        // Desensibilisation progressive (chaque exposition sans danger reel)
        self.desensitization = (self.desensitization + desensitization_rate).min(1.0);
    }
}

/// Gestionnaire de phobies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhobiaManager {
    /// Liste des phobies actives
    pub phobias: Vec<Phobia>,
    /// Taux de desensibilisation par exposition
    pub desensitization_rate: f64,
    /// Derniere phobie declenchee (pour le contexte)
    #[serde(skip)]
    pub last_triggered_name: Option<String>,
    /// Intensite du dernier declenchement
    #[serde(skip)]
    pub last_trigger_intensity: f64,
}

impl PhobiaManager {
    /// Cree un gestionnaire vide.
    pub fn new(desensitization_rate: f64) -> Self {
        Self {
            phobias: Vec::new(),
            desensitization_rate,
            last_triggered_name: None,
            last_trigger_intensity: 0.0,
        }
    }

    /// Ajoute une phobie.
    pub fn add(&mut self, phobia: Phobia) {
        self.phobias.push(phobia);
    }

    /// Scan un texte et declenche les phobies correspondantes.
    /// Retourne le nombre de phobies declenchees.
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

    /// Impact chimique des phobies declenchees ce cycle.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        if self.last_trigger_intensity < 0.05 {
            return ChemistryAdjustment::default();
        }

        let i = self.last_trigger_intensity;
        ChemistryAdjustment {
            cortisol: i * 0.30,        // Stress intense
            adrenaline: i * 0.25,      // Reaction de fuite
            serotonin: -i * 0.10,      // Chute bien-etre
            endorphin: i * 0.05,       // Reponse analgesique
            noradrenaline: i * 0.15,   // Vigilance
            ..Default::default()
        }
    }

    /// Reset le dernier declenchement (en debut de cycle).
    pub fn reset_cycle(&mut self) {
        self.last_triggered_name = None;
        self.last_trigger_intensity = 0.0;
    }

    /// Serialise pour l'API.
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

        // 10 expositions
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
