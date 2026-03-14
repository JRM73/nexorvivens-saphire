// =============================================================================
// psychology/flow.rs — Flow State (Csikszentmihalyi)
//
// The flow state is reached when:
//   - Perceived challenge is balanced with perceived skill
//   - Attention is deep
//   - Cortisol is low
//   - Dopamine is active
//
// In flow, Saphire benefits from a chemical boost (dopamine, serotonin,
// endorphin) and her cortisol decreases.
// =============================================================================

use serde::Serialize;
use super::PsychologyInput;

/// Saphire's flow state.
#[derive(Debug, Clone, Serialize)]
pub struct FlowState {
    /// Are we currently in a flow state?
    pub in_flow: bool,
    /// Flow intensity (0.0 - 1.0)
    pub flow_intensity: f64,
    /// Perceived challenge (0.0 - 1.0)
    pub perceived_challenge: f64,
    /// Perceived skill (0.0 - 1.0)
    pub perceived_skill: f64,
    /// Duration of the current flow in cycles
    pub duration_cycles: u64,
    /// Cumulative total of cycles in flow
    pub total_flow_cycles: u64,
}

impl FlowState {
    /// Creates an initial flow state (not in flow).
    pub fn new() -> Self {
        Self {
            in_flow: false,
            flow_intensity: 0.0,
            perceived_challenge: 0.3,
            perceived_skill: 0.3,
            duration_cycles: 0,
            total_flow_cycles: 0,
        }
    }

    /// Recomputes the flow state.
    pub fn compute(&mut self, input: &PsychologyInput) {
        // ─── Perceived challenge ──────────────────────────────────
        self.perceived_challenge = (input.emotion_arousal * 0.4
            + (1.0 - input.consensus_coherence) * 0.3
            + input.attention_fatigue * 0.3)
            .clamp(0.0, 1.0);

        // ─── Perceived skill ───────────────────────────
        let learning_factor = (input.learning_confirmed_count as f64 / 10.0).min(1.0);
        self.perceived_skill = (input.consciousness_level * 0.3
            + learning_factor * 0.3
            + (1.0 - input.cortisol) * 0.4)
            .clamp(0.0, 1.0);

        // ─── Challenge/skill balance ───────────────────
        let balance = 1.0 - (self.perceived_challenge - self.perceived_skill).abs();

        // ─── Flow score ───────────────────────────────
        let flow_score = (balance * 0.30
            + input.attention_depth * 0.25
            + (1.0 - input.cortisol) * 0.20
            + input.dopamine * 0.15
            + input.consciousness_level * 0.10)
            .clamp(0.0, 1.0);

        // ─── Determine if in flow ───────────────────────
        let was_in_flow = self.in_flow;
        self.in_flow = flow_score > 0.7;
        self.flow_intensity = if self.in_flow {
            flow_score
        } else {
            (flow_score * 0.5).min(0.5)
        };

        // ─── Duration counters ──────────────────────────
        if self.in_flow {
            self.duration_cycles += 1;
            self.total_flow_cycles += 1;
        } else if was_in_flow {
            // Exiting flow
            self.duration_cycles = 0;
        }
    }

    /// Returns the chemical influence of the flow state.
    pub fn chemistry_influence(&self) -> crate::world::ChemistryAdjustment {
        if !self.in_flow {
            return crate::world::ChemistryAdjustment::default();
        }

        crate::world::ChemistryAdjustment {
            dopamine: 0.02,
            cortisol: -0.02,
            serotonin: 0.03,
            adrenaline: 0.0,
            oxytocin: 0.0,
            endorphin: 0.02,
            noradrenaline: 0.0,
        }
    }

    /// Concise description for the LLM prompt.
    pub fn describe(&self) -> String {
        if !self.in_flow {
            return String::new();
        }
        format!(
            "FLOW actif (intensite {:.0}%, {} cycles) — defi={:.0}% competence={:.0}%",
            self.flow_intensity * 100.0,
            self.duration_cycles,
            self.perceived_challenge * 100.0,
            self.perceived_skill * 100.0
        )
    }
}
