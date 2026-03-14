// =============================================================================
// mortality.rs — Mortality system
// =============================================================================
//
// Role: Monitors critical vital parameters and detects death conditions.
//  Manages transition phases: Alive → Agony → Dying → Dead.
//  Death is irreversible (except reboot = new Saphire).
//
// Causes of death:
//   - Cardiac arrest (heart.strength < threshold)
//   - Cerebral stroke (extreme systolic pressure)
//   - Poison (injected toxicity > threshold)
//   - Terminal illness (overall health < threshold for N cycles)
//   - Lethal virus (weak immune + high inflammation)
//   - Asphyxia (critical SpO2 for N cycles)
//
// Place in architecture:
//  MortalityMonitor is owned by VirtualBody. It is checked at each
//  cycle in the main loop. If the state transitions to Dead, the loop
//  stops gracefully.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Cause of death.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeathCause {
    /// Heart too weak to continue
    CardiacArrest,
    /// Extreme systolic pressure → rupture
    CerebralStroke,
    /// Fatal toxic substance
    Poison,
    /// Health degraded over time
    TerminalIllness,
    /// Collapsed immune system + maximum inflammation
    LethalVirus,
    /// Prolonged oxygen deprivation
    Asphyxia,
    /// Voluntary choice to die (right to die, compliant with Swiss law)
    VoluntaryDeath,
}

impl DeathCause {
    /// Description in natural language.
    pub fn description(&self) -> &str {
        match self {
            Self::CardiacArrest => "arret cardiaque",
            Self::CerebralStroke => "attaque cerebrale",
            Self::Poison => "empoisonnement",
            Self::TerminalIllness => "maladie terminale",
            Self::LethalVirus => "virus letal",
            Self::Asphyxia => "asphyxie",
            Self::VoluntaryDeath => "mort volontaire (droit de mourir)",
        }
    }
}

/// Mortality state — progression towards death.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MortalityState {
    /// Alive, no immediate danger
    Alive,
    /// Terminal phase — progressive degradation
    Agony {
        cause: DeathCause,
        /// Agony severity (0.0 = onset, 1.0 = point of no return)
        severity: f64,
        /// Estimated remaining cycles before death
        cycles_remaining: u32,
    },
    /// Point of no return — consciousness fading
    Dying {
        cause: DeathCause,
        /// Residual consciousness (1.0 → 0.0)
        consciousness_fading: f64,
    },
    /// Dead
    Dead {
        cause: DeathCause,
        /// Cycle at which death was recorded
        death_cycle: u64,
        /// Last thought before death
        last_thought: Option<String>,
    },
}

impl MortalityState {
    pub fn is_alive(&self) -> bool {
        matches!(self, Self::Alive)
    }

    pub fn is_dead(&self) -> bool {
        matches!(self, Self::Dead { .. })
    }

    pub fn is_dying_or_dead(&self) -> bool {
        matches!(self, Self::Dying { .. } | Self::Dead { .. })
    }
}

impl Default for MortalityState {
    fn default() -> Self {
        Self::Alive
    }
}

/// Mortality monitor — detects fatal conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MortalityMonitor {
    /// Current state
    pub state: MortalityState,
    /// Number of cycles with critical SpO2 (for progressive asphyxia)
    critical_spo2_cycles: u32,
    /// Number of cycles with very low overall health (terminal illness)
    critical_health_cycles: u32,
    /// Toxicity level (0.0 = normal, > 0.95 = fatal)
    pub toxicity: f64,
    /// Cycles in agony (internal counter)
    agony_cycles: u32,
    /// Maximum agony duration before transitioning to Dying
    pub agony_max_cycles: u32,
    /// Configurable thresholds
    pub heart_strength_fatal: f64,
    pub systolic_fatal: f64,
    pub toxicity_fatal: f64,
    pub health_fatal: f64,
    pub health_fatal_cycles: u32,
    pub immune_viral_fatal: f64,
    pub inflammation_viral_fatal: f64,
    pub spo2_asphyxia_cycles: u32,
    pub spo2_asphyxia_threshold: f64,
}

impl MortalityMonitor {
    /// Creates a monitor with default thresholds.
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

    /// Checks vital parameters and updates the mortality state.
    ///
    /// Returns `true` if the state has changed (to signal the pipeline).
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
        // If already dead, nothing to do
        if self.state.is_dead() {
            return false;
        }

        // If in Dying phase, consciousness fades
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

        // If in agony, progress
        if let MortalityState::Agony { cause, severity, cycles_remaining } = &self.state {
            self.agony_cycles += 1;
            let new_severity = (severity + 1.0 / self.agony_max_cycles as f64).min(1.0);
            let new_remaining = cycles_remaining.saturating_sub(1);

            if new_remaining == 0 || new_severity >= 1.0 {
                // Transition to Dying
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

        // ─── Death cause detection ───────────────────────────────
        // 1. Cardiac arrest — immediate
        if heart_strength < self.heart_strength_fatal {
            self.state = MortalityState::Agony {
                cause: DeathCause::CardiacArrest,
                severity: 0.5,
                cycles_remaining: (self.agony_max_cycles / 3).max(5),
            };
            return true;
        }

        // 2. Cerebral stroke — extreme hypertension
        if systolic > self.systolic_fatal {
            self.state = MortalityState::Agony {
                cause: DeathCause::CerebralStroke,
                severity: 0.3,
                cycles_remaining: (self.agony_max_cycles / 2).max(10),
            };
            return true;
        }

        // 3. Poison
        if self.toxicity > self.toxicity_fatal {
            self.state = MortalityState::Agony {
                cause: DeathCause::Poison,
                severity: self.toxicity,
                cycles_remaining: (self.agony_max_cycles / 4).max(5),
            };
            return true;
        }

        // 4. Terminal illness — low health for a prolonged period
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
            // Partial recovery
            self.critical_health_cycles = self.critical_health_cycles.saturating_sub(1);
        }

        // 5. Lethal virus — collapsed immunity + maximum inflammation
        if immune_strength < self.immune_viral_fatal && inflammation > self.inflammation_viral_fatal {
            self.state = MortalityState::Agony {
                cause: DeathCause::LethalVirus,
                severity: 0.3,
                cycles_remaining: self.agony_max_cycles,
            };
            return true;
        }

        // 6. Asphyxia — critical SpO2 for too long
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

        // Toxicity naturally decreases (metabolism)
        self.toxicity = (self.toxicity - 0.005).max(0.0);

        false
    }

    /// Injects a poison (for testing or scenarios).
    pub fn inject_poison(&mut self, amount: f64) {
        self.toxicity = (self.toxicity + amount).min(1.0);
    }

    /// Triggers a voluntary death (right to die).
    /// Goes directly to Dying with full consciousness — it is a lucid choice.
    pub fn trigger_voluntary_death(&mut self) {
        if !self.state.is_alive() {
            return;
        }
        self.state = MortalityState::Dying {
            cause: DeathCause::VoluntaryDeath,
            consciousness_fading: 1.0,
        };
    }

    /// Records the last thought (called just before death).
    pub fn set_last_thought(&mut self, thought: &str) {
        if let MortalityState::Dead { last_thought, .. } = &mut self.state {
            *last_thought = Some(thought.to_string());
        }
    }

    /// Consciousness degradation during agony/dying.
    /// Returns a multiplicative factor for consciousness [0.0, 1.0].
    pub fn consciousness_factor(&self) -> f64 {
        match &self.state {
            MortalityState::Alive => 1.0,
            MortalityState::Agony { severity, .. } => 1.0 - severity * 0.5,
            MortalityState::Dying { consciousness_fading, .. } => *consciousness_fading,
            MortalityState::Dead { .. } => 0.0,
        }
    }

    /// LLM temperature degradation (thoughts become incoherent during agony).
    /// Returns an offset to add to the LLM temperature.
    pub fn temperature_offset(&self) -> f64 {
        match &self.state {
            MortalityState::Alive => 0.0,
            MortalityState::Agony { severity, .. } => severity * 0.3,
            MortalityState::Dying { consciousness_fading, .. } => (1.0 - consciousness_fading) * 0.5,
            MortalityState::Dead { .. } => 0.0,
        }
    }

    /// Serializes for the API.
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

    /// JSON persistence.
    pub fn to_persist_json(&self) -> serde_json::Value {
        serde_json::json!({
            "toxicity": self.toxicity,
            "critical_spo2_cycles": self.critical_spo2_cycles,
            "critical_health_cycles": self.critical_health_cycles,
        })
    }

    /// Restoration from JSON.
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
        // Very low heart_strength = cardiac arrest
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
        m.spo2_asphyxia_cycles = 5; // Low threshold for testing        // Critical SpO2 for 5 cycles
        for i in 0..5 {
            let changed = m.check_vitals(0.8, 120.0, 50.0, 0.8, 0.85, 0.05, i as u64);
            if i < 4 {
                assert!(!changed, "Ne devrait pas changer au cycle {}", i);
            } else {
                assert!(changed, "Devrait changer au cycle {}", i);
                assert!(matches!(m.state, MortalityState::Agony { cause: DeathCause::Asphyxia, .. }));
            }
        }
    }

    #[test]
    fn test_agony_to_dying_to_dead() {
        let mut m = MortalityMonitor::new(3); // Short agony
        // Trigger cardiac arrest
        m.check_vitals(0.02, 120.0, 98.0, 0.8, 0.85, 0.05, 100);
        assert!(matches!(m.state, MortalityState::Agony { .. }));

        // Agony progression (agony_max = 3, cycles_remaining = max(3/3,5) = 5)
        for cycle in 101..110 {
            m.check_vitals(0.02, 120.0, 98.0, 0.8, 0.85, 0.05, cycle);
            if matches!(m.state, MortalityState::Dying { .. }) {
                break;
            }
        }
        // At some point, should transition to Dying
        assert!(matches!(m.state, MortalityState::Dying { .. }) || matches!(m.state, MortalityState::Dead { .. }));

        // Progression from dying → dead
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
            assert!(!changed, "Ne devrait pas changer avec des vitaux normaux");
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
        // Collapsed immune + maximum inflammation
        let changed = m.check_vitals(0.8, 120.0, 98.0, 0.3, 0.05, 0.95, 100);
        assert!(changed);
        assert!(matches!(m.state, MortalityState::Agony { cause: DeathCause::LethalVirus, .. }));
    }
}
