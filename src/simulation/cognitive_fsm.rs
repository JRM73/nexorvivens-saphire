// =============================================================================
// cognitive_fsm.rs — Augmented finite state machine for cognitive states
//
// Role: Models Saphire's cognitive states as an FSM (Finite State Machine).
//       Transitions are based on chemistry and feelings, not on discrete
//       events. Each state influences the pipeline.
//
// States:
//   - Eveil (Awake): normal state, standard cognition
//   - Focus: intense concentration, high noradrenaline
//   - Reverie (Daydream): free thoughts, high dopamine + serotonin
//   - Stress: high cortisol, defensive mode
//   - Flow: optimal state, balanced dopamine + noradrenaline
//   - Repos (Rest): fatigue, all transmitters low
//
// Place in the architecture:
//   Consulted in lifecycle/mod.rs every cycle to modulate the pipeline
//   behavior (thought interval, preferred type, etc.).
// =============================================================================

use serde::{Serialize, Deserialize};

/// Cognitive states of the FSM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CognitiveState {
    /// Normal state, standard cognition
    Eveil,
    /// Intense concentration (noradrenaline dominant)
    Focus,
    /// Free and creative thoughts (dopamine + serotonin)
    Reverie,
    /// Defensive mode, high stress (cortisol dominant)
    Stress,
    /// Optimal performance state (perfect balance)
    Flow,
    /// Fatigue, recovery needed
    Repos,
}

impl CognitiveState {
    /// Display name of the state.
    pub fn as_str(&self) -> &str {
        match self {
            CognitiveState::Eveil => "Éveil",
            CognitiveState::Focus => "Focus",
            CognitiveState::Reverie => "Rêverie",
            CognitiveState::Stress => "Stress",
            CognitiveState::Flow => "Flow",
            CognitiveState::Repos => "Repos",
        }
    }

    /// Thought interval multiplier for this state.
    /// < 1.0 = faster thoughts, > 1.0 = slower thoughts.
    pub fn thought_interval_multiplier(&self) -> f64 {
        match self {
            CognitiveState::Flow => 0.7,     // Faster in flow
            CognitiveState::Focus => 0.8,    // Fast in focus
            CognitiveState::Stress => 1.5,   // Slowed by stress
            CognitiveState::Repos => 2.0,    // Slow at rest
            CognitiveState::Reverie => 1.2,  // Slightly slow in daydream
            CognitiveState::Eveil => 1.0,    // Normal
        }
    }

    /// Preferred thought types in this state.
    pub fn preferred_thought_types(&self) -> &[usize] {
        match self {
            CognitiveState::Eveil => &[],          // No preference
            CognitiveState::Focus => &[1, 6, 10],  // Exploration, Curiosity, AlgorithmicReflection
            CognitiveState::Reverie => &[7, 3, 13],// Daydream, Continuation, DesireFormation
            CognitiveState::Stress => &[0, 5],      // Introspection, SelfAnalysis
            CognitiveState::Flow => &[1, 6, 7, 4], // Exploration, Curiosity, Daydream, Existential
            CognitiveState::Repos => &[2, 8],       // MemoryReflection, TemporalAwareness
        }
    }
}

/// The cognitive FSM with transition history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveFsm {
    /// Current state
    pub current_state: CognitiveState,
    /// Number of cycles in the current state
    pub cycles_in_state: u64,
    /// Recent state history (size 20)
    pub state_history: Vec<CognitiveState>,
    /// Total number of transitions
    pub total_transitions: u64,
}

impl CognitiveFsm {
    pub fn new() -> Self {
        Self {
            current_state: CognitiveState::Eveil,
            cycles_in_state: 0,
            state_history: vec![CognitiveState::Eveil],
            total_transitions: 0,
        }
    }

    /// Evaluates possible transitions from the current state
    /// based on chemistry and feelings.
    pub fn tick(
        &mut self,
        cortisol: f64,
        dopamine: f64,
        serotonin: f64,
        noradrenaline: f64,
        endorphin: f64,
    ) {
        self.cycles_in_state += 1;

        let new_state = self.evaluate_transition(
            cortisol, dopamine, serotonin, noradrenaline, endorphin,
        );

        if new_state != self.current_state {
            self.current_state = new_state;
            self.cycles_in_state = 0;
            self.total_transitions += 1;
            self.state_history.push(new_state);
            if self.state_history.len() > 20 {
                self.state_history.remove(0);
            }
        }
    }

    /// Determines the next state based on chemical levels.
    fn evaluate_transition(
        &self,
        cortisol: f64,
        dopamine: f64,
        serotonin: f64,
        noradrenaline: f64,
        endorphin: f64,
    ) -> CognitiveState {
        // Hysteresis: stay in current state if no strong signal
        // (prevents rapid oscillations between states)
        let hysteresis = if self.cycles_in_state < 3 { 0.1 } else { 0.0 };

        // Stress: very high cortisol
        if cortisol > 0.75 + hysteresis {
            return CognitiveState::Stress;
        }

        // Rest: everything is low (fatigue)
        if dopamine < 0.25 && serotonin < 0.3 && noradrenaline < 0.25 && endorphin < 0.2 {
            return CognitiveState::Repos;
        }

        // Flow: optimal balance (high dopamine + noradrenaline, moderate cortisol)
        if dopamine > 0.55 + hysteresis
            && noradrenaline > 0.45 + hysteresis
            && cortisol < 0.45
            && serotonin > 0.4
        {
            return CognitiveState::Flow;
        }

        // Focus: noradrenaline dominant
        if noradrenaline > 0.6 + hysteresis && cortisol < 0.5 {
            return CognitiveState::Focus;
        }

        // Daydream: high dopamine + serotonin, low noradrenaline
        if dopamine > 0.5 + hysteresis
            && serotonin > 0.5 + hysteresis
            && noradrenaline < 0.4
            && cortisol < 0.4
        {
            return CognitiveState::Reverie;
        }

        // Default: Awake
        CognitiveState::Eveil
    }

    /// Description for the LLM prompt.
    pub fn describe_for_prompt(&self) -> String {
        format!(
            "ETAT COGNITIF : {} (depuis {} cycles)",
            self.current_state.as_str(),
            self.cycles_in_state,
        )
    }

    /// JSON for the dashboard.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "current_state": self.current_state.as_str(),
            "cycles_in_state": self.cycles_in_state,
            "total_transitions": self.total_transitions,
            "multiplier": self.current_state.thought_interval_multiplier(),
            "history": self.state_history.iter()
                .rev().take(10)
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
        })
    }
}

impl Default for CognitiveFsm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let fsm = CognitiveFsm::new();
        assert_eq!(fsm.current_state, CognitiveState::Eveil);
    }

    #[test]
    fn test_stress_transition() {
        let mut fsm = CognitiveFsm::new();
        // High cortisol -> Stress
        for _ in 0..4 { // Overcome hysteresis
            fsm.tick(0.9, 0.3, 0.3, 0.3, 0.3);
        }
        assert_eq!(fsm.current_state, CognitiveState::Stress);
    }

    #[test]
    fn test_flow_transition() {
        let mut fsm = CognitiveFsm::new();
        for _ in 0..4 {
            fsm.tick(0.3, 0.7, 0.6, 0.6, 0.5);
        }
        assert_eq!(fsm.current_state, CognitiveState::Flow);
    }

    #[test]
    fn test_repos_transition() {
        let mut fsm = CognitiveFsm::new();
        for _ in 0..4 {
            fsm.tick(0.2, 0.1, 0.1, 0.1, 0.1);
        }
        assert_eq!(fsm.current_state, CognitiveState::Repos);
    }

    #[test]
    fn test_hysteresis() {
        let mut fsm = CognitiveFsm::new();
        // A single stressful tick should not be enough if just arrived
        fsm.tick(0.8, 0.3, 0.3, 0.3, 0.3);
        // State may change because 0.8 > 0.75 + 0.1 = false for the first cycle
        // But 0.8 > 0.85 is false, so stays in Eveil
        assert_eq!(fsm.current_state, CognitiveState::Eveil);
    }

    #[test]
    fn test_thought_multiplier() {
        assert!(CognitiveState::Flow.thought_interval_multiplier() < 1.0);
        assert!(CognitiveState::Repos.thought_interval_multiplier() > 1.0);
    }
}
