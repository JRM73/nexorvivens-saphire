// =============================================================================
// cognitive_load.rs — Cognitive load (Sweller's theory)
// =============================================================================
//
// Cognitive load represents the total amount of mental resources
// mobilized by Saphire at a given moment. When the load exceeds
// a threshold, Saphire is "overloaded": her chemistry is affected
// (cortisol, noradrenaline) and she can signal this overload
// in her responses.
//
// Load sources:
//  - Active conversation (intrinsic load)
//  - Number of active desires (extraneous load)
//  - Unresolved emotional wounds (emotional load)
//  - Ambient cortisol (cumulative stress)
//
// Processing capacity can be modulated by other systems
// (fatigue, cognitive profile, etc.).
// =============================================================================

use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};

// ─── Configuration ──────────────────────────────────────────────────────────
/// Configuration for the cognitive load module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveLoadConfig {
    /// Enables or disables cognitive load tracking
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Overload threshold (0.0 - 1.0)
    #[serde(default = "default_overload_threshold")]
    pub overload_threshold: f64,

    /// Load decay per cycle
    #[serde(default = "default_load_decay")]
    pub load_decay_per_cycle: f64,

    /// Cortisol increase during overload
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

// ─── Main state ─────────────────────────────────────────────────────────
/// Cognitive load module state.
///
/// Maintains a current load value (0.0 - 1.0) computed from
/// multiple sources, along with history and overload counters.
pub struct CognitiveLoadState {
    /// Module enabled or not
    pub enabled: bool,
    /// Current cognitive load (0.0 - 1.0)
    pub current_load: f64,
    /// Breakdown by source (key = name, value = contribution)
    pub load_sources: HashMap<String, f64>,
    /// Processing capacity (1.0 = full capacity, can be modulated)
    pub processing_capacity: f64,
    /// History of the last 20 cycles (load values)
    pub load_history: VecDeque<f64>,
    /// Number of consecutive overload cycles
    pub overload_cycles: u64,
    /// Overload threshold
    pub overload_threshold: f64,
    /// Total number of detected overloads
    pub total_overloads: u64,
    /// Decay per cycle
    load_decay: f64,
    /// Cortisol generated during overload
    cortisol_on_overload: f64,
}

impl CognitiveLoadState {
    /// Creates a new cognitive load state.
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

    /// Updates the cognitive load from current sources.
    ///
    /// Formula:
    ///  load = conversation*0.2 + desires*0.05 + wounds*0.1 + cortisol*0.3
    ///
    /// The load is then divided by the processing capacity (which can
    /// be reduced by fatigue or a particular cognitive profile).
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

        // ─── Compute each source ─────────────────────────────────
        let conversation_load = if in_conversation { 0.2 } else { 0.0 };
        let desires_load = (active_desires as f64 * 0.05).min(0.3);
        let wounds_load = (active_wounds as f64 * 0.1).min(0.3);
        let cortisol_load = cortisol * 0.3;

        // ─── Record the sources ────────────────────────────────
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

        // ─── Raw load ───────────────────────────────────────────
        let raw_load = conversation_load + desires_load + wounds_load + cortisol_load;

        // ─── Adjust by processing capacity ──────────────────
        let effective_capacity = self.processing_capacity.max(0.1);
        self.current_load = (raw_load / effective_capacity).clamp(0.0, 1.0);
    }

    /// Periodic tick: decay, history, overload counters.
    pub fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        // Natural load decay
        self.current_load = (self.current_load - self.load_decay).max(0.0);

        // Push to history (keep 20 entries)
        if self.load_history.len() >= 20 {
            self.load_history.pop_front();
        }
        self.load_history.push_back(self.current_load);

        // Overload counter
        if self.is_overloaded() {
            self.overload_cycles += 1;
            if self.overload_cycles == 1 {
                self.total_overloads += 1;
            }
        } else {
            self.overload_cycles = 0;
        }
    }

    /// Returns the chemical influence of cognitive load.
    ///
    /// During overload: cortisol increases, noradrenaline increases (hypervigilance).
    /// If overload persists, serotonin decreases (exhaustion).
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
            serotonin: -0.01 * chronic_factor, // Chronic exhaustion            adrenaline: 0.0,
            oxytocin: 0.0,
            endorphin: 0.0,
            noradrenaline: self.cortisol_on_overload * overload_intensity * 0.4,
        }
    }

    /// Indicates whether Saphire is in cognitive overload.
    pub fn is_overloaded(&self) -> bool {
        self.enabled && self.current_load > self.overload_threshold
    }

    /// Indicates whether the overload is prolonged (> 5 consecutive cycles)
    /// and should be reported in the prompt.
    pub fn should_report_overload(&self) -> bool {
        self.is_overloaded() && self.overload_cycles > 5
    }

    /// Generates a textual description for the LLM prompt.
    /// Only returns content if Saphire is overloaded.
    pub fn describe_for_prompt(&self) -> String {
        if !self.enabled || !self.is_overloaded() {
            return String::new();
        }

        let mut desc = format!(
            "SURCHARGE COGNITIVE ({:.0}%, seuil {:.0}%) — ",
            self.current_load * 100.0,
            self.overload_threshold * 100.0,
        );

        // Detail the main sources
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

    /// Generates a compact summary of internal state, ALWAYS present in the prompt.
    /// Unlike describe_for_prompt() which only activates during overload,
    /// this method provides permanent proprioception to Saphire.
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

    /// Serializes the complete state to JSON for the dashboard.
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
        // Force a high load: cortisol=1.0 -> 0.3, wounds=3 -> 0.3, conv -> 0.2 = 0.8
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

        // The load is still > threshold after a single decay of 0.05
        state.tick();
        assert_eq!(state.overload_cycles, 2);
        // total_overloads only increments on the first overload cycle
        assert_eq!(state.total_overloads, 1);
    }

    #[test]
    fn test_overload_cycles_reset() {
        let mut state = make_state();
        state.current_load = 0.9;
        state.tick();
        state.tick();
        assert_eq!(state.overload_cycles, 2);

        // Drop the load below the threshold
        state.current_load = 0.3;
        state.tick();
        assert_eq!(state.overload_cycles, 0);
    }

    #[test]
    fn test_should_report_overload() {
        let mut state = make_state();
        state.current_load = 0.95;
        // Requires > 5 consecutive cycles
        for _ in 0..6 {
            state.tick();
            // Raise the load back since decay reduces it
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
        // load = 0.2 + 0.2 + 0.3 + 0.3 = 1.0 (clamp)
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
        // All sources at maximum
        state.update(true, 10, 10, 1.0);
        assert!(state.current_load <= 1.0);
    }

    #[test]
    fn test_reduced_processing_capacity() {
        let mut state = make_state();
        state.processing_capacity = 0.5;
        // raw load = 0.2, divided by 0.5 = 0.4
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
