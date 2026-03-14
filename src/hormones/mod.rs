// =============================================================================
// hormones/mod.rs — Saphire's hormonal system (slow cycles)
//
// Purpose: Models 8 hormones that drive slow dynamics complementary to the
// 7 fast neurotransmitters. Includes neuroreceptors (sensitivity, tolerance,
// saturation) and circadian/ultradian cycles.
//
// Architecture:
//   HormonalSystem groups HormonalState + ReceptorSystem + circadian phase.
//   Called once per cognitive cycle via tick().
// =============================================================================

pub mod receptors;
pub mod cycles;
pub mod interactions;

use serde::{Deserialize, Serialize};
use crate::neurochemistry::NeuroChemicalState;
use crate::config::HormonesConfig;

pub use receptors::ReceptorSystem;

/// Hormonal state: 8 hormones normalized between 0.0 and 1.0.
///
/// Some hormones have a dual role (hormone + NT):
///   - cortisol_h: hormonal version (circadian cycle, chronic stress)
///   - epinephrine: hormonal version (episodic peaks, fight-or-flight)
///   - oxytocin_h: hormonal version (slow attachment)
/// The NT versions remain in NeuroChemicalState for fast reactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HormonalState {
    /// Hormonal cortisol: chronic stress, circadian cycle (morning peak)
    pub cortisol_h: f64,
    /// Melatonin: sleep regulation (night peak)
    pub melatonin: f64,
    /// Epinephrine (hormonal adrenaline): episodic fight-or-flight
    pub epinephrine: f64,
    /// Testosterone: motivation, dominance, libido
    pub testosterone: f64,
    /// Estrogen: emotional regulation, mood
    pub estrogen: f64,
    /// Hormonal oxytocin: attachment, slow social bonding
    pub oxytocin_h: f64,
    /// Insulin: blood sugar/energy regulation
    pub insulin: f64,
    /// Thyroid (T3/T4): metabolism, thought speed
    pub thyroid: f64,
}

impl HormonalState {
    /// Creates a hormonal state from configuration.
    pub fn from_config(config: &HormonesConfig) -> Self {
        Self {
            cortisol_h: 0.3,
            melatonin: 0.2,
            epinephrine: 0.2,
            testosterone: config.initial_testosterone,
            estrogen: config.initial_estrogen,
            oxytocin_h: 0.4,
            insulin: config.initial_insulin,
            thyroid: config.initial_thyroid,
        }
    }

    /// Clamps all values between 0.0 and 1.0.
    pub fn clamp_all(&mut self) {
        self.cortisol_h = self.cortisol_h.clamp(0.0, 1.0);
        self.melatonin = self.melatonin.clamp(0.0, 1.0);
        self.epinephrine = self.epinephrine.clamp(0.0, 1.0);
        self.testosterone = self.testosterone.clamp(0.0, 1.0);
        self.estrogen = self.estrogen.clamp(0.0, 1.0);
        self.oxytocin_h = self.oxytocin_h.clamp(0.0, 1.0);
        self.insulin = self.insulin.clamp(0.0, 1.0);
        self.thyroid = self.thyroid.clamp(0.0, 1.0);
    }

    /// Formats the hormonal state for display.
    pub fn display_string(&self) -> String {
        format!(
            "Cort_H:{:.2} Mela:{:.2} Epin:{:.2} Test:{:.2} Estr:{:.2} Ocyt_H:{:.2} Insu:{:.2} Thyr:{:.2}",
            self.cortisol_h, self.melatonin, self.epinephrine, self.testosterone,
            self.estrogen, self.oxytocin_h, self.insulin, self.thyroid
        )
    }
}

impl Default for HormonalState {
    fn default() -> Self {
        Self {
            cortisol_h: 0.3,
            melatonin: 0.2,
            epinephrine: 0.2,
            testosterone: 0.50,
            estrogen: 0.50,
            oxytocin_h: 0.4,
            insulin: 0.50,
            thyroid: 0.60,
        }
    }
}

/// Complete hormonal system: state + receptors + circadian phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HormonalSystem {
    /// Current hormonal state (8 hormones)
    pub state: HormonalState,
    /// Neuroreceptor system (sensitivity, tolerance, saturation)
    pub receptors: ReceptorSystem,
    /// Circadian phase (0.0 = midnight, 0.5 = noon, 1.0 = midnight)
    pub circadian_phase: f64,
    /// Cycle counter since start
    pub cycle_counter: u64,
    /// Enables or disables the hormonal system
    pub enabled: bool,
}

impl HormonalSystem {
    /// Creates a new hormonal system from configuration.
    pub fn new(config: &HormonesConfig) -> Self {
        Self {
            state: HormonalState::from_config(config),
            receptors: ReceptorSystem::new(config),
            circadian_phase: 0.25,  // Start at 6am
            cycle_counter: 0,
            enabled: config.enabled,
        }
    }

    /// Performs a complete hormonal system cycle:
    /// 1. Advance the circadian phase
    /// 2. Apply hormonal cycles (circadian, ultradian)
    /// 3. Update receptors (tolerance, sensitivity)
    /// 4. Apply hormone -> NT interactions
    /// 5. Receive NT -> hormone feedback
    pub fn tick(&mut self, chemistry: &mut NeuroChemicalState, config: &HormonesConfig) {
        if !self.enabled {
            return;
        }

        self.cycle_counter += 1;

        // 1. Advance the circadian phase
        cycles::tick_circadian(&mut self.state, &mut self.circadian_phase, config);

        // 2. Ultradian cycles (testosterone)
        cycles::tick_ultradian(&mut self.state, self.cycle_counter);

        // 3. Update receptors
        self.receptors.tick(chemistry, config);

        // 4. Hormones -> neurotransmitters
        interactions::apply_hormones_to_chemistry(&self.state, &self.receptors, chemistry);

        // 5. Neurotransmitters -> hormones
        interactions::update_hormones_from_chemistry(&mut self.state, chemistry);

        // Clamp all values
        self.state.clamp_all();
        chemistry.clamp_all();
    }

    /// Returns a JSON snapshot of the hormonal state for WebSocket broadcast.
    pub fn to_snapshot_json(&self) -> serde_json::Value {
        serde_json::json!({
            "cortisol_h": self.state.cortisol_h,
            "melatonin": self.state.melatonin,
            "epinephrine": self.state.epinephrine,
            "testosterone": self.state.testosterone,
            "estrogen": self.state.estrogen,
            "oxytocin_h": self.state.oxytocin_h,
            "insulin": self.state.insulin,
            "thyroid": self.state.thyroid,
            "circadian_phase": self.circadian_phase,
            "circadian_time": circadian_time_label(self.circadian_phase),
            "cycle_counter": self.cycle_counter,
            "receptors": self.receptors.to_snapshot_json(),
        })
    }

    /// Describes the hormonal state for LLM context (prompt).
    pub fn describe_for_prompt(&self) -> String {
        if !self.enabled {
            return String::new();
        }

        let time = circadian_time_label(self.circadian_phase);
        let mut parts = Vec::new();

        parts.push(format!("[HORMONES — Phase circadienne: {} ({:.0}%)]",
            time, self.circadian_phase * 100.0));

        // Report out-of-range hormones
        if self.state.melatonin > 0.6 {
            parts.push("Melatonine elevee — somnolence naturelle".into());
        }
        if self.state.cortisol_h > 0.7 {
            parts.push("Cortisol hormonal eleve — stress chronique".into());
        }
        if self.state.testosterone > 0.7 {
            parts.push("Testosterone elevee — motivation/dominance accrue".into());
        }
        if self.state.insulin < 0.3 {
            parts.push("Insuline basse — risque hypoglycemie, irritabilite".into());
        }
        if self.state.thyroid < 0.4 {
            parts.push("Thyroide basse — ralentissement metabolique".into());
        }
        if self.state.thyroid > 0.8 {
            parts.push("Thyroide elevee — acceleration metabolique".into());
        }

        // Desensitized receptors
        let desensitized = self.receptors.describe_desensitized();
        if !desensitized.is_empty() {
            parts.push(format!("Recepteurs desensibilises: {}", desensitized));
        }

        parts.join("\n")
    }
}

impl Default for HormonalSystem {
    fn default() -> Self {
        Self {
            state: HormonalState::default(),
            receptors: ReceptorSystem::default(),
            circadian_phase: 0.25,
            cycle_counter: 0,
            enabled: false,
        }
    }
}

/// Converts a circadian phase (0.0-1.0) to a time-of-day label.
pub fn circadian_time_label(phase: f64) -> &'static str {
    match phase {
        p if p < 0.125 => "Nuit profonde (0h-3h)",
        p if p < 0.25 => "Fin de nuit (3h-6h)",
        p if p < 0.375 => "Matin (6h-9h)",
        p if p < 0.5 => "Matinee (9h-12h)",
        p if p < 0.625 => "Debut d'apres-midi (12h-15h)",
        p if p < 0.75 => "Apres-midi (15h-18h)",
        p if p < 0.875 => "Soiree (18h-21h)",
        _ => "Nuit (21h-0h)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hormonal_state_default_in_range() {
        let state = HormonalState::default();
        assert!(state.cortisol_h >= 0.0 && state.cortisol_h <= 1.0);
        assert!(state.melatonin >= 0.0 && state.melatonin <= 1.0);
        assert!(state.testosterone >= 0.0 && state.testosterone <= 1.0);
        assert!(state.thyroid >= 0.0 && state.thyroid <= 1.0);
    }

    #[test]
    fn test_clamp_all() {
        let mut state = HormonalState::default();
        state.cortisol_h = 1.5;
        state.melatonin = -0.1;
        state.clamp_all();
        assert_eq!(state.cortisol_h, 1.0);
        assert_eq!(state.melatonin, 0.0);
    }

    #[test]
    fn test_circadian_time_label() {
        assert_eq!(circadian_time_label(0.0), "Nuit profonde (0h-3h)");
        assert_eq!(circadian_time_label(0.5), "Debut d'apres-midi (12h-15h)");
        assert_eq!(circadian_time_label(0.9), "Nuit (21h-0h)");
    }

    #[test]
    fn test_hormonal_system_disabled_noop() {
        let config = HormonesConfig::default();
        let mut system = HormonalSystem::new(&config);
        system.enabled = false;
        let mut chem = NeuroChemicalState::default();
        let before = chem.dopamine;
        system.tick(&mut chem, &config);
        // When disabled, nothing changes
        assert_eq!(chem.dopamine, before);
        assert_eq!(system.cycle_counter, 0);
    }
}
