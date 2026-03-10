// =============================================================================
// cognitive_load.rs — Charge cognitive (theorie de Sweller)
// =============================================================================
//
// La charge cognitive represente la quantite totale de ressources mentales
// mobilisees par Saphire a un instant donne. Quand la charge depasse
// un seuil, Saphire est "surchargee" : sa chimie en est affectee
// (cortisol, noradrenaline) et elle peut signaler cette surcharge
// dans ses reponses.
//
// Sources de charge :
//   - Conversation active (charge intrinseque)
//   - Nombre de desirs actifs (charge etrangere)
//   - Blessures emotionnelles non resolues (charge emotionnelle)
//   - Cortisol ambiant (stress cumulatif)
//
// La capacite de traitement peut etre modulee par d'autres systemes
// (fatigue, profil cognitif, etc.).
// =============================================================================

use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};

// ─── Configuration ──────────────────────────────────────────────────────────

/// Configuration du module de charge cognitive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveLoadConfig {
    /// Active ou desactive le suivi de charge cognitive
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Seuil de surcharge (0.0 - 1.0)
    #[serde(default = "default_overload_threshold")]
    pub overload_threshold: f64,

    /// Decay de la charge par cycle
    #[serde(default = "default_load_decay")]
    pub load_decay_per_cycle: f64,

    /// Augmentation du cortisol en cas de surcharge
    #[serde(default = "default_cortisol_on_overload")]
    pub cortisol_on_overload: f64,
}

fn default_true() -> bool { true }
fn default_overload_threshold() -> f64 { 0.75 }
fn default_load_decay() -> f64 { 0.05 }
fn default_cortisol_on_overload() -> f64 { 0.06 }

impl Default for CognitiveLoadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            overload_threshold: 0.75,
            load_decay_per_cycle: 0.05,
            cortisol_on_overload: 0.06,
        }
    }
}

// ─── Etat principal ─────────────────────────────────────────────────────────

/// Etat du module de charge cognitive.
///
/// Maintient une valeur de charge courante (0.0 - 1.0) calculee a partir
/// de plusieurs sources, ainsi qu'un historique et des compteurs de surcharge.
pub struct CognitiveLoadState {
    /// Module actif ou non
    pub enabled: bool,
    /// Charge cognitive courante (0.0 - 1.0)
    pub current_load: f64,
    /// Decomposition par source (cle = nom, valeur = contribution)
    pub load_sources: HashMap<String, f64>,
    /// Capacite de traitement (1.0 = pleine capacite, peut etre modulee)
    pub processing_capacity: f64,
    /// Historique des 20 derniers cycles (valeurs de charge)
    pub load_history: VecDeque<f64>,
    /// Nombre de cycles consecutifs en surcharge
    pub overload_cycles: u64,
    /// Seuil de surcharge
    pub overload_threshold: f64,
    /// Nombre total de surcharges detectees
    pub total_overloads: u64,
    /// Decay par cycle
    load_decay: f64,
    /// Cortisol genere en surcharge
    cortisol_on_overload: f64,
}

impl CognitiveLoadState {
    /// Cree un nouvel etat de charge cognitive.
    pub fn new(config: &CognitiveLoadConfig) -> Self {
        Self {
            enabled: config.enabled,
            current_load: 0.0,
            load_sources: HashMap::new(),
            processing_capacity: 1.0,
            load_history: VecDeque::with_capacity(20),
            overload_cycles: 0,
            overload_threshold: config.overload_threshold,
            total_overloads: 0,
            load_decay: config.load_decay_per_cycle,
            cortisol_on_overload: config.cortisol_on_overload,
        }
    }

    /// Met a jour la charge cognitive a partir des sources courantes.
    ///
    /// Formule :
    ///   charge = conversation*0.2 + desirs*0.05 + blessures*0.1 + cortisol*0.3
    ///
    /// La charge est ensuite divisee par la capacite de traitement (qui peut
    /// etre reduite par la fatigue ou un profil cognitif particulier).
    pub fn update(
        &mut self,
        in_conversation: bool,
        active_desires: usize,
        active_wounds: usize,
        cortisol: f64,
    ) {
        if !self.enabled {
            return;
        }

        // ─── Calculer chaque source ─────────────────────────────────
        let conversation_load = if in_conversation { 0.2 } else { 0.0 };
        let desires_load = (active_desires as f64 * 0.05).min(0.3);
        let wounds_load = (active_wounds as f64 * 0.1).min(0.3);
        let cortisol_load = cortisol * 0.3;

        // ─── Enregistrer les sources ────────────────────────────────
        self.load_sources.clear();
        if conversation_load > 0.0 {
            self.load_sources.insert("conversation".to_string(), conversation_load);
        }
        if desires_load > 0.0 {
            self.load_sources.insert("desirs".to_string(), desires_load);
        }
        if wounds_load > 0.0 {
            self.load_sources.insert("blessures".to_string(), wounds_load);
        }
        if cortisol_load > 0.0 {
            self.load_sources.insert("cortisol".to_string(), cortisol_load);
        }

        // ─── Charge brute ───────────────────────────────────────────
        let raw_load = conversation_load + desires_load + wounds_load + cortisol_load;

        // ─── Ajuster par la capacite de traitement ──────────────────
        let effective_capacity = self.processing_capacity.max(0.1);
        self.current_load = (raw_load / effective_capacity).clamp(0.0, 1.0);
    }

    /// Tick periodique : decay, historique, compteurs de surcharge.
    pub fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        // Decay naturel de la charge
        self.current_load = (self.current_load - self.load_decay).max(0.0);

        // Pousser dans l'historique (garder 20 entrees)
        if self.load_history.len() >= 20 {
            self.load_history.pop_front();
        }
        self.load_history.push_back(self.current_load);

        // Compteur de surcharge
        if self.is_overloaded() {
            self.overload_cycles += 1;
            if self.overload_cycles == 1 {
                self.total_overloads += 1;
            }
        } else {
            self.overload_cycles = 0;
        }
    }

    /// Retourne l'influence chimique de la charge cognitive.
    ///
    /// En surcharge : cortisol augmente, noradrenaline augmente (hypervigilance).
    /// Si la surcharge dure longtemps, la serotonine baisse (epuisement).
    pub fn chemistry_influence(&self) -> crate::world::ChemistryAdjustment {
        if !self.enabled || !self.is_overloaded() {
            return crate::world::ChemistryAdjustment::default();
        }

        let overload_intensity = (self.current_load - self.overload_threshold)
            / (1.0 - self.overload_threshold).max(0.01);

        let chronic_factor = (self.overload_cycles as f64 / 10.0).min(1.0);

        crate::world::ChemistryAdjustment {
            dopamine: 0.0,
            cortisol: self.cortisol_on_overload * overload_intensity,
            serotonin: -0.01 * chronic_factor, // Epuisement chronique
            adrenaline: 0.0,
            oxytocin: 0.0,
            endorphin: 0.0,
            noradrenaline: self.cortisol_on_overload * overload_intensity * 0.4,
        }
    }

    /// Indique si Saphire est en surcharge cognitive.
    pub fn is_overloaded(&self) -> bool {
        self.enabled && self.current_load > self.overload_threshold
    }

    /// Indique si la surcharge est prolongee (> 5 cycles consecutifs)
    /// et merite d'etre signalee dans le prompt.
    pub fn should_report_overload(&self) -> bool {
        self.is_overloaded() && self.overload_cycles > 5
    }

    /// Genere une description textuelle pour le prompt LLM.
    /// Ne retourne du contenu que si Saphire est en surcharge.
    pub fn describe_for_prompt(&self) -> String {
        if !self.enabled || !self.is_overloaded() {
            return String::new();
        }

        let mut desc = format!(
            "SURCHARGE COGNITIVE ({:.0}%, seuil {:.0}%) — ",
            self.current_load * 100.0,
            self.overload_threshold * 100.0,
        );

        // Detailler les sources principales
        let mut sources: Vec<(&str, &f64)> = self.load_sources.iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect();
        sources.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        let source_strs: Vec<String> = sources.iter()
            .take(3)
            .map(|(name, val)| format!("{} ({:.0}%)", name, *val * 100.0))
            .collect();
        desc.push_str(&format!("sources: {}", source_strs.join(", ")));

        if self.should_report_overload() {
            desc.push_str(&format!(
                " | SURCHARGE PROLONGEE ({} cycles)",
                self.overload_cycles,
            ));
        }

        desc
    }

    /// Genere un resume compact de l'etat interne, TOUJOURS present dans le prompt.
    /// Contrairement a describe_for_prompt() qui ne s'active qu'en surcharge,
    /// cette methode fournit une proprioception permanente a Saphire.
    pub fn proprioception_prompt(
        &self,
        umami: f64,
        exploration_c: f64,
        is_stagnating: bool,
    ) -> String {
        if !self.enabled {
            return String::new();
        }

        let capacity_label = if self.current_load < 0.3 {
            "legere"
        } else if self.current_load < 0.6 {
            "normale"
        } else if self.current_load < self.overload_threshold {
            "elevee"
        } else {
            "SURCHARGE"
        };

        let stagnation_label = if is_stagnating { "oui" } else { "non" };

        format!(
            "PROPRIOCEPTION: charge {:.0}% | capacite {} | umami {:.2} | exploration C={:.1} | stagnation: {}",
            self.current_load * 100.0,
            capacity_label,
            umami,
            exploration_c,
            stagnation_label,
        )
    }

    /// Serialise l'etat complet en JSON pour le dashboard.
    pub fn to_json(&self) -> serde_json::Value {
        let avg_load = if self.load_history.is_empty() {
            0.0
        } else {
            self.load_history.iter().sum::<f64>() / self.load_history.len() as f64
        };

        serde_json::json!({
            "enabled": self.enabled,
            "current_load": self.current_load,
            "overload_threshold": self.overload_threshold,
            "is_overloaded": self.is_overloaded(),
            "overload_cycles": self.overload_cycles,
            "total_overloads": self.total_overloads,
            "processing_capacity": self.processing_capacity,
            "average_load": avg_load,
            "load_sources": self.load_sources,
            "load_history": self.load_history.iter().collect::<Vec<_>>(),
        })
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> CognitiveLoadState {
        CognitiveLoadState::new(&CognitiveLoadConfig::default())
    }

    #[test]
    fn test_initial_state() {
        let state = make_state();
        assert_eq!(state.current_load, 0.0);
        assert!(!state.is_overloaded());
        assert!(!state.should_report_overload());
    }

    #[test]
    fn test_update_in_conversation() {
        let mut state = make_state();
        state.update(true, 0, 0, 0.0);
        assert!((state.current_load - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_update_multiple_sources() {
        let mut state = make_state();
        // conversation=0.2, desires=2*0.05=0.1, wounds=1*0.1=0.1, cortisol=0.5*0.3=0.15
        // total = 0.55
        state.update(true, 2, 1, 0.5);
        assert!((state.current_load - 0.55).abs() < 0.01);
    }

    #[test]
    fn test_overload_detection() {
        let mut state = make_state();
        // Force une charge elevee : cortisol=1.0 → 0.3, wounds=3 → 0.3, conv → 0.2 = 0.8
        state.update(true, 0, 3, 1.0);
        assert!(state.is_overloaded());
    }

    #[test]
    fn test_tick_decays_load() {
        let mut state = make_state();
        state.current_load = 0.5;
        state.tick();
        assert!(state.current_load < 0.5);
        assert_eq!(state.load_history.len(), 1);
    }

    #[test]
    fn test_tick_counts_overload_cycles() {
        let mut state = make_state();
        state.current_load = 0.9;
        state.tick();
        assert_eq!(state.overload_cycles, 1);
        assert_eq!(state.total_overloads, 1);

        // La charge est toujours > seuil apres un seul decay de 0.05
        state.tick();
        assert_eq!(state.overload_cycles, 2);
        // total_overloads ne s'incremente qu'au premier cycle de surcharge
        assert_eq!(state.total_overloads, 1);
    }

    #[test]
    fn test_overload_cycles_reset() {
        let mut state = make_state();
        state.current_load = 0.9;
        state.tick();
        state.tick();
        assert_eq!(state.overload_cycles, 2);

        // Faire tomber la charge sous le seuil
        state.current_load = 0.3;
        state.tick();
        assert_eq!(state.overload_cycles, 0);
    }

    #[test]
    fn test_should_report_overload() {
        let mut state = make_state();
        state.current_load = 0.95;
        // Il faut > 5 cycles consecutifs
        for _ in 0..6 {
            state.tick();
            // Remonter la charge car le decay la reduit
            state.current_load = 0.95;
        }
        assert!(state.should_report_overload());
    }

    #[test]
    fn test_chemistry_influence_not_overloaded() {
        let state = make_state();
        let adj = state.chemistry_influence();
        assert_eq!(adj.cortisol, 0.0);
        assert_eq!(adj.noradrenaline, 0.0);
    }

    #[test]
    fn test_chemistry_influence_overloaded() {
        let mut state = make_state();
        state.current_load = 0.9;
        let adj = state.chemistry_influence();
        assert!(adj.cortisol > 0.0, "Overload should produce cortisol");
        assert!(adj.noradrenaline > 0.0, "Overload should produce noradrenaline");
    }

    #[test]
    fn test_describe_empty_when_not_overloaded() {
        let state = make_state();
        assert!(state.describe_for_prompt().is_empty());
    }

    #[test]
    fn test_describe_when_overloaded() {
        let mut state = make_state();
        state.update(true, 4, 3, 1.0);
        // charge = 0.2 + 0.2 + 0.3 + 0.3 = 1.0 (clamp)
        let desc = state.describe_for_prompt();
        assert!(!desc.is_empty());
        assert!(desc.contains("SURCHARGE COGNITIVE"));
    }

    #[test]
    fn test_to_json() {
        let state = make_state();
        let json = state.to_json();
        assert_eq!(json["enabled"], true);
        assert_eq!(json["current_load"], 0.0);
        assert_eq!(json["is_overloaded"], false);
    }

    #[test]
    fn test_load_clamped_to_one() {
        let mut state = make_state();
        // Toutes les sources au maximum
        state.update(true, 10, 10, 1.0);
        assert!(state.current_load <= 1.0);
    }

    #[test]
    fn test_reduced_processing_capacity() {
        let mut state = make_state();
        state.processing_capacity = 0.5;
        // charge brute = 0.2, divisee par 0.5 = 0.4
        state.update(true, 0, 0, 0.0);
        assert!((state.current_load - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_history_capped_at_20() {
        let mut state = make_state();
        for i in 0..25 {
            state.current_load = i as f64 * 0.04;
            state.tick();
        }
        assert_eq!(state.load_history.len(), 20);
    }
}
