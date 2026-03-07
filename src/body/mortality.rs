// =============================================================================
// mortality.rs — Mortality System
// =============================================================================
//
// Purpose: Monitors critical vital parameters and detects fatal conditions.
//          Manages the progressive death-state transitions:
//            Alive -> Agony -> Dying -> Dead
//          Death is irreversible (only a full reboot creates a new Saphire).
//
// Causes of death:
//   - Cardiac arrest: heart beat strength drops below a fatal threshold.
//   - Cerebral stroke: extreme systolic blood pressure causes vascular rupture.
//   - Poison: injected toxicity exceeds the lethal threshold.
//   - Terminal illness: overall health remains critically low for N consecutive cycles.
//   - Lethal virus: collapsed immune system combined with maximal inflammation.
//   - Asphyxia: critically low SpO2 persists for too many consecutive cycles.
//
// Architectural placement:
//   `MortalityMonitor` is owned by `VirtualBody`. It is checked every cycle
//   in the main autonomous loop. If the state transitions to Dead, the loop
//   terminates gracefully.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Enumeration of possible causes of death.
///
/// Each variant represents a distinct physiological failure mode that can
/// lead to the entity's death through the Agony -> Dying -> Dead progression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeathCause {
    /// Heart too weak to sustain circulation (beat strength below fatal threshold).
    CardiacArrest,
    /// Extreme systolic blood pressure causing cerebrovascular rupture.
    CerebralStroke,
    /// Fatal toxic substance exceeding the metabolic detoxification capacity.
    Poison,
    /// Prolonged critically degraded overall health (sustained organ failure).
    TerminalIllness,
    /// Immune system collapse combined with overwhelming systemic inflammation.
    LethalVirus,
    /// Prolonged critically low oxygen saturation (SpO2) causing tissue death.
    Asphyxia,
}

impl DeathCause {
    /// Returns a human-readable description of the cause of death.
    pub fn description(&self) -> &str {
        match self {
            Self::CardiacArrest => "cardiac arrest",
            Self::CerebralStroke => "cerebral stroke",
            Self::Poison => "poisoning",
            Self::TerminalIllness => "terminal illness",
            Self::LethalVirus => "lethal virus",
            Self::Asphyxia => "asphyxia",
        }
    }
}

/// Mortality state — progressive transition toward death.
///
/// The state machine follows a strict unidirectional progression:
/// `Alive -> Agony -> Dying -> Dead`. There is no recovery from Dying or Dead.
/// Recovery from Agony is theoretically possible if the triggering condition
/// resolves, but the current implementation does not support spontaneous recovery
/// once Agony has begun.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MortalityState {
    /// The entity is alive with no immediate mortal danger.
    Alive,
    /// Terminal phase — progressive physiological degradation toward death.
    Agony {
        /// The underlying cause that triggered this agony phase.
        cause: DeathCause,
        /// Agony severity in [0.0, 1.0]: 0.0 = just entered agony, 1.0 = point of no return.
        /// Severity increases by `1/agony_max_cycles` each cycle.
        severity: f64,
        /// Estimated number of update cycles remaining before transition to Dying.
        cycles_remaining: u32,
    },
    /// Point of no return — consciousness is fading irreversibly.
    Dying {
        /// The underlying cause of death.
        cause: DeathCause,
        /// Residual consciousness level in [0.0, 1.0], decreasing by 0.1 per cycle.
        /// When this reaches 0.0, the entity transitions to Dead.
        consciousness_fading: f64,
    },
    /// The entity is dead. This state is irreversible.
    Dead {
        /// The cause that led to death.
        cause: DeathCause,
        /// The cycle number at which death was officially recorded.
        death_cycle: u64,
        /// The last coherent thought before death (set externally just before the final transition).
        last_thought: Option<String>,
    },
}

impl MortalityState {
    /// Returns `true` if the entity is alive (no mortal danger).
    pub fn is_alive(&self) -> bool {
        matches!(self, Self::Alive)
    }

    /// Returns `true` if the entity is dead (terminal, irreversible state).
    pub fn is_dead(&self) -> bool {
        matches!(self, Self::Dead { .. })
    }

    /// Returns `true` if the entity is in the Dying or Dead state (irreversible conditions).
    pub fn is_dying_or_dead(&self) -> bool {
        matches!(self, Self::Dying { .. } | Self::Dead { .. })
    }
}

impl Default for MortalityState {
    fn default() -> Self {
        Self::Alive
    }
}

/// Mortality monitor — detects fatal conditions and manages the death-state progression.
///
/// Tracks cumulative counters for progressive conditions (asphyxia, terminal illness),
/// maintains a toxicity level that decays naturally via simulated metabolism, and
/// manages the Agony phase duration before the irreversible Dying transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MortalityMonitor {
    /// Current mortality state (Alive, Agony, Dying, or Dead).
    pub state: MortalityState,
    /// Number of consecutive cycles with critically low SpO2 (for progressive asphyxia detection).
    critical_spo2_cycles: u32,
    /// Number of consecutive cycles with critically low overall health (for terminal illness detection).
    critical_health_cycles: u32,
    /// Current toxicity level in [0.0, 1.0] (0.0 = no toxins, > 0.95 = potentially fatal).
    /// Decays by 0.005 per cycle via simulated hepatic metabolism.
    pub toxicity: f64,
    /// Internal counter tracking how many cycles the entity has spent in Agony.
    agony_cycles: u32,
    /// Maximum number of Agony cycles before the irreversible transition to Dying.
    pub agony_max_cycles: u32,
    /// Fatal threshold for heart beat strength: below this value, cardiac arrest is triggered.
    pub heart_strength_fatal: f64,
    /// Fatal threshold for systolic blood pressure (mmHg): above this value, cerebral stroke is triggered.
    pub systolic_fatal: f64,
    /// Fatal threshold for toxicity level: above this value, poison death is triggered.
    pub toxicity_fatal: f64,
    /// Fatal threshold for overall health: below this value, the terminal illness counter increments.
    pub health_fatal: f64,
    /// Number of consecutive low-health cycles required to trigger terminal illness.
    pub health_fatal_cycles: u32,
    /// Immune strength threshold: below this value (combined with high inflammation), lethal virus is triggered.
    pub immune_viral_fatal: f64,
    /// Inflammation threshold: above this value (combined with low immunity), lethal virus is triggered.
    pub inflammation_viral_fatal: f64,
    /// Number of consecutive critically-low-SpO2 cycles required to trigger asphyxia.
    pub spo2_asphyxia_cycles: u32,
    /// SpO2 percentage threshold below which cycles count toward asphyxia detection.
    pub spo2_asphyxia_threshold: f64,
}

impl MortalityMonitor {
    /// Creates a new mortality monitor with default fatal thresholds.
    ///
    /// Default thresholds:
    /// - Heart strength fatal: 0.05 (cardiac arrest)
    /// - Systolic fatal: 220 mmHg (cerebral stroke)
    /// - Toxicity fatal: 0.95 (poisoning)
    /// - Health fatal: 0.05 for 100 consecutive cycles (terminal illness)
    /// - Immune viral fatal: 0.1 + inflammation > 0.9 (lethal virus)
    /// - SpO2 asphyxia: < 55% for 30 consecutive cycles
    pub fn new(agony_duration_cycles: u32) -> Self {
        Self {
            state: MortalityState::Alive,
            critical_spo2_cycles: 0,
            critical_health_cycles: 0,
            toxicity: 0.0,
            agony_cycles: 0,
            agony_max_cycles: agony_duration_cycles,
            heart_strength_fatal: 0.05,
            systolic_fatal: 220.0,
            toxicity_fatal: 0.95,
            health_fatal: 0.05,
            health_fatal_cycles: 100,
            immune_viral_fatal: 0.1,
            inflammation_viral_fatal: 0.9,
            spo2_asphyxia_cycles: 30,
            spo2_asphyxia_threshold: 55.0,
        }
    }

    /// Checks vital parameters and updates the mortality state accordingly.
    ///
    /// This method is called every update cycle. It evaluates the current vital signs
    /// against fatal thresholds and manages state transitions. The evaluation order
    /// defines priority: cardiac arrest > cerebral stroke > poison > terminal illness
    /// > lethal virus > asphyxia.
    ///
    /// Returns `true` if the mortality state changed (useful for signaling the pipeline).
    ///
    /// # Parameters
    /// - `heart_strength`: current cardiac beat strength in [0.0, 1.0].
    /// - `systolic`: current systolic blood pressure in mmHg.
    /// - `spo2`: current peripheral oxygen saturation as a percentage.
    /// - `overall_health`: composite health score in [0.0, 1.0].
    /// - `immune_strength`: immune system strength in [0.0, 1.0].
    /// - `inflammation`: systemic inflammation level in [0.0, 1.0].
    /// - `current_cycle`: the current cycle number (used to timestamp death).
    pub fn check_vitals(
        &mut self,
        heart_strength: f64,
        systolic: f64,
        spo2: f64,
        overall_health: f64,
        immune_strength: f64,
        inflammation: f64,
        current_cycle: u64,
    ) -> bool {
        // If already dead, no further state changes are possible
        if self.state.is_dead() {
            return false;
        }

        // If in the Dying phase, consciousness fades by 0.1 per cycle until reaching 0.0 (death)
        if let MortalityState::Dying { cause, consciousness_fading } = &self.state {
            let new_fading = (consciousness_fading - 0.1).max(0.0);
            if new_fading <= 0.0 {
                self.state = MortalityState::Dead {
                    cause: cause.clone(),
                    death_cycle: current_cycle,
                    last_thought: None,
                };
                return true;
            }
            self.state = MortalityState::Dying {
                cause: cause.clone(),
                consciousness_fading: new_fading,
            };
            return true;
        }

        // If in Agony, progress toward the Dying transition
        if let MortalityState::Agony { cause, severity, cycles_remaining } = &self.state {
            self.agony_cycles += 1;
            let new_severity = (severity + 1.0 / self.agony_max_cycles as f64).min(1.0);
            let new_remaining = cycles_remaining.saturating_sub(1);

            if new_remaining == 0 || new_severity >= 1.0 {
                // Transition to the irreversible Dying phase
                self.state = MortalityState::Dying {
                    cause: cause.clone(),
                    consciousness_fading: 1.0,
                };
                return true;
            }

            self.state = MortalityState::Agony {
                cause: cause.clone(),
                severity: new_severity,
                cycles_remaining: new_remaining,
            };
            return true;
        }

        // --- Detection of fatal conditions (evaluated in priority order) ---

        // 1. Cardiac arrest — immediate: heart too weak to sustain circulation
        if heart_strength < self.heart_strength_fatal {
            self.state = MortalityState::Agony {
                cause: DeathCause::CardiacArrest,
                severity: 0.5,
                cycles_remaining: (self.agony_max_cycles / 3).max(5),
            };
            return true;
        }

        // 2. Cerebral stroke — extreme hypertension causing vascular rupture
        if systolic > self.systolic_fatal {
            self.state = MortalityState::Agony {
                cause: DeathCause::CerebralStroke,
                severity: 0.3,
                cycles_remaining: (self.agony_max_cycles / 2).max(10),
            };
            return true;
        }

        // 3. Poisoning — accumulated toxicity exceeds metabolic clearance capacity
        if self.toxicity > self.toxicity_fatal {
            self.state = MortalityState::Agony {
                cause: DeathCause::Poison,
                severity: self.toxicity,
                cycles_remaining: (self.agony_max_cycles / 4).max(5),
            };
            return true;
        }

        // 4. Terminal illness — overall health critically low for a sustained period
        if overall_health < self.health_fatal {
            self.critical_health_cycles += 1;
            if self.critical_health_cycles >= self.health_fatal_cycles {
                self.state = MortalityState::Agony {
                    cause: DeathCause::TerminalIllness,
                    severity: 0.2,
                    cycles_remaining: self.agony_max_cycles,
                };
                return true;
            }
        } else {
            // Partial recovery: decrement the critical health counter when health improves
            self.critical_health_cycles = self.critical_health_cycles.saturating_sub(1);
        }

        // 5. Lethal virus — collapsed immune system combined with overwhelming inflammation
        if immune_strength < self.immune_viral_fatal && inflammation > self.inflammation_viral_fatal {
            self.state = MortalityState::Agony {
                cause: DeathCause::LethalVirus,
                severity: 0.3,
                cycles_remaining: self.agony_max_cycles,
            };
            return true;
        }

        // 6. Asphyxia — critically low SpO2 persisting for too many consecutive cycles
        if spo2 < self.spo2_asphyxia_threshold {
            self.critical_spo2_cycles += 1;
            if self.critical_spo2_cycles >= self.spo2_asphyxia_cycles {
                self.state = MortalityState::Agony {
                    cause: DeathCause::Asphyxia,
                    severity: 0.4,
                    cycles_remaining: (self.agony_max_cycles / 3).max(5),
                };
                return true;
            }
        } else {
            self.critical_spo2_cycles = self.critical_spo2_cycles.saturating_sub(2);
        }

        // Toxicity decays naturally each cycle (simulated hepatic/renal metabolism)
        self.toxicity = (self.toxicity - 0.005).max(0.0);

        false
    }

    /// Injects a toxic substance, increasing the toxicity level (for testing or scenario scripting).
    ///
    /// The toxicity level is clamped to a maximum of 1.0.
    pub fn inject_poison(&mut self, amount: f64) {
        self.toxicity = (self.toxicity + amount).min(1.0);
    }

    /// Records the entity's last coherent thought (called just before death is finalized).
    ///
    /// Only takes effect if the entity is already in the Dead state.
    pub fn set_last_thought(&mut self, thought: &str) {
        if let MortalityState::Dead { last_thought, .. } = &mut self.state {
            *last_thought = Some(thought.to_string());
        }
    }

    /// Returns a consciousness degradation factor based on the current mortality state.
    ///
    /// - Alive: 1.0 (full consciousness)
    /// - Agony: `1.0 - severity * 0.5` (consciousness partially impaired by suffering)
    /// - Dying: equals the residual `consciousness_fading` value (approaching 0.0)
    /// - Dead: 0.0 (no consciousness)
    ///
    /// This factor is used as a multiplicative modifier on the consciousness evaluation score.
    pub fn consciousness_factor(&self) -> f64 {
        match &self.state {
            MortalityState::Alive => 1.0,
            MortalityState::Agony { severity, .. } => 1.0 - severity * 0.5,
            MortalityState::Dying { consciousness_fading, .. } => *consciousness_fading,
            MortalityState::Dead { .. } => 0.0,
        }
    }

    /// Returns an LLM temperature offset reflecting cognitive incoherence during death.
    ///
    /// As the entity approaches death, its "thoughts" become increasingly incoherent,
    /// modeled by increasing the LLM sampling temperature:
    /// - Alive: 0.0 (no offset, normal coherence)
    /// - Agony: up to +0.3 (thoughts become scattered with increasing severity)
    /// - Dying: up to +0.5 (thoughts become highly disorganized as consciousness fades)
    /// - Dead: 0.0 (no thoughts generated)
    pub fn temperature_offset(&self) -> f64 {
        match &self.state {
            MortalityState::Alive => 0.0,
            MortalityState::Agony { severity, .. } => severity * 0.3,
            MortalityState::Dying { consciousness_fading, .. } => (1.0 - consciousness_fading) * 0.5,
            MortalityState::Dead { .. } => 0.0,
        }
    }

    /// Serializes the current mortality state as JSON for the API.
    ///
    /// Includes state-specific fields (cause, severity, cycles remaining, consciousness
    /// fading, death cycle, last thought) depending on the current mortality phase.
    pub fn to_json(&self) -> serde_json::Value {
        match &self.state {
            MortalityState::Alive => serde_json::json!({
                "state": "alive",
                "toxicity": self.toxicity,
                "critical_spo2_cycles": self.critical_spo2_cycles,
                "critical_health_cycles": self.critical_health_cycles,
            }),
            MortalityState::Agony { cause, severity, cycles_remaining } => serde_json::json!({
                "state": "agony",
                "cause": cause.description(),
                "severity": severity,
                "cycles_remaining": cycles_remaining,
                "toxicity": self.toxicity,
            }),
            MortalityState::Dying { cause, consciousness_fading } => serde_json::json!({
                "state": "dying",
                "cause": cause.description(),
                "consciousness_fading": consciousness_fading,
            }),
            MortalityState::Dead { cause, death_cycle, last_thought } => serde_json::json!({
                "state": "dead",
                "cause": cause.description(),
                "death_cycle": death_cycle,
                "last_thought": last_thought,
            }),
        }
    }

    /// Serializes the persistent mortality data as JSON for database storage.
    ///
    /// Saves only the progressive counters and toxicity level (the state machine
    /// itself resets to Alive on restoration — death is not persisted across restarts).
    pub fn to_persist_json(&self) -> serde_json::Value {
        serde_json::json!({
            "toxicity": self.toxicity,
            "critical_spo2_cycles": self.critical_spo2_cycles,
            "critical_health_cycles": self.critical_health_cycles,
        })
    }

    /// Restores persistent mortality data from a previously saved JSON value.
    ///
    /// Gracefully handles missing fields (each field is independently optional).
    pub fn restore_from_json(&mut self, json: &serde_json::Value) {
        if let Some(v) = json.get("toxicity").and_then(|v| v.as_f64()) {
            self.toxicity = v;
        }
        if let Some(v) = json.get("critical_spo2_cycles").and_then(|v| v.as_u64()) {
            self.critical_spo2_cycles = v as u32;
        }
        if let Some(v) = json.get("critical_health_cycles").and_then(|v| v.as_u64()) {
            self.critical_health_cycles = v as u32;
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_alive() {
        let m = MortalityMonitor::new(50);
        assert!(m.state.is_alive());
        assert!(!m.state.is_dead());
        assert_eq!(m.consciousness_factor(), 1.0);
        assert_eq!(m.temperature_offset(), 0.0);
    }

    #[test]
    fn test_cardiac_arrest() {
        let mut m = MortalityMonitor::new(50);
        // Very low heart strength triggers cardiac arrest
        let changed = m.check_vitals(0.02, 120.0, 98.0, 0.8, 0.85, 0.05, 100);
        assert!(changed);
        assert!(matches!(m.state, MortalityState::Agony { cause: DeathCause::CardiacArrest, .. }));
    }

    #[test]
    fn test_cerebral_stroke() {
        let mut m = MortalityMonitor::new(50);
        let changed = m.check_vitals(0.8, 225.0, 98.0, 0.8, 0.85, 0.05, 100);
        assert!(changed);
        assert!(matches!(m.state, MortalityState::Agony { cause: DeathCause::CerebralStroke, .. }));
    }

    #[test]
    fn test_poison_death() {
        let mut m = MortalityMonitor::new(50);
        m.inject_poison(0.96);
        let changed = m.check_vitals(0.8, 120.0, 98.0, 0.8, 0.85, 0.05, 100);
        assert!(changed);
        assert!(matches!(m.state, MortalityState::Agony { cause: DeathCause::Poison, .. }));
    }

    #[test]
    fn test_asphyxia_progressive() {
        let mut m = MortalityMonitor::new(50);
        m.spo2_asphyxia_cycles = 5; // Low threshold for testing
        // Critically low SpO2 sustained for 5 consecutive cycles
        for i in 0..5 {
            let changed = m.check_vitals(0.8, 120.0, 50.0, 0.8, 0.85, 0.05, i as u64);
            if i < 4 {
                assert!(!changed, "Should not change at cycle {}", i);
            } else {
                assert!(changed, "Should change at cycle {}", i);
                assert!(matches!(m.state, MortalityState::Agony { cause: DeathCause::Asphyxia, .. }));
            }
        }
    }

    #[test]
    fn test_agony_to_dying_to_dead() {
        let mut m = MortalityMonitor::new(3); // Short agony for testing
        // Trigger cardiac arrest
        m.check_vitals(0.02, 120.0, 98.0, 0.8, 0.85, 0.05, 100);
        assert!(matches!(m.state, MortalityState::Agony { .. }));

        // Progress through agony (agony_max = 3, cycles_remaining = max(3/3, 5) = 5)
        for cycle in 101..110 {
            m.check_vitals(0.02, 120.0, 98.0, 0.8, 0.85, 0.05, cycle);
            if matches!(m.state, MortalityState::Dying { .. }) {
                break;
            }
        }
        // Should have transitioned to Dying at some point
        assert!(matches!(m.state, MortalityState::Dying { .. }) || matches!(m.state, MortalityState::Dead { .. }));

        // Progress through Dying phase until death
        for cycle in 110..130 {
            m.check_vitals(0.02, 120.0, 98.0, 0.8, 0.85, 0.05, cycle);
            if m.state.is_dead() {
                break;
            }
        }
        assert!(m.state.is_dead());
    }

    #[test]
    fn test_normal_vitals_stay_alive() {
        let mut m = MortalityMonitor::new(50);
        for i in 0..100 {
            let changed = m.check_vitals(0.8, 120.0, 98.0, 0.8, 0.85, 0.05, i);
            assert!(!changed, "Should not change with normal vital signs");
        }
        assert!(m.state.is_alive());
    }

    #[test]
    fn test_consciousness_factor_agony() {
        let mut m = MortalityMonitor::new(50);
        m.state = MortalityState::Agony {
            cause: DeathCause::Poison,
            severity: 0.6,
            cycles_remaining: 20,
        };
        assert!(m.consciousness_factor() < 1.0);
        assert!(m.consciousness_factor() > 0.0);
    }

    #[test]
    fn test_lethal_virus() {
        let mut m = MortalityMonitor::new(50);
        // Collapsed immune system + overwhelming inflammation
        let changed = m.check_vitals(0.8, 120.0, 98.0, 0.3, 0.05, 0.95, 100);
        assert!(changed);
        assert!(matches!(m.state, MortalityState::Agony { cause: DeathCause::LethalVirus, .. }));
    }
}
