// =============================================================================
// psychology/flow.rs — Etat de Flow (Csikszentmihalyi)
//
// L'etat de flow est atteint quand :
//   - Le defi percu est equilibre avec la competence percue
//   - L'attention est profonde
//   - Le cortisol est bas
//   - La dopamine est active
//
// En flow, Saphire beneficie d'un boost chimique (dopamine, serotonine,
// endorphine) et son cortisol diminue.
// =============================================================================

use serde::Serialize;
use super::PsychologyInput;

/// Etat de flow de Saphire.
#[derive(Debug, Clone, Serialize)]
pub struct FlowState {
    /// Est-on actuellement en etat de flow ?
    pub in_flow: bool,
    /// Intensite du flow (0.0 - 1.0)
    pub flow_intensity: f64,
    /// Defi percu (0.0 - 1.0)
    pub perceived_challenge: f64,
    /// Competence percue (0.0 - 1.0)
    pub perceived_skill: f64,
    /// Duree du flow actuel en cycles
    pub duration_cycles: u64,
    /// Total cumule de cycles en flow
    pub total_flow_cycles: u64,
}

impl FlowState {
    /// Cree un etat de flow initial (pas en flow).
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

    /// Recalcule l'etat de flow.
    pub fn compute(&mut self, input: &PsychologyInput) {
        // ─── Defi percu ──────────────────────────────────
        self.perceived_challenge = (input.emotion_arousal * 0.4
            + (1.0 - input.consensus_coherence) * 0.3
            + input.attention_fatigue * 0.3)
            .clamp(0.0, 1.0);

        // ─── Competence percue ───────────────────────────
        let learning_factor = (input.learning_confirmed_count as f64 / 10.0).min(1.0);
        self.perceived_skill = (input.consciousness_level * 0.3
            + learning_factor * 0.3
            + (1.0 - input.cortisol) * 0.4)
            .clamp(0.0, 1.0);

        // ─── Equilibre defi/competence ───────────────────
        let balance = 1.0 - (self.perceived_challenge - self.perceived_skill).abs();

        // ─── Score de flow ───────────────────────────────
        let flow_score = (balance * 0.30
            + input.attention_depth * 0.25
            + (1.0 - input.cortisol) * 0.20
            + input.dopamine * 0.15
            + input.consciousness_level * 0.10)
            .clamp(0.0, 1.0);

        // ─── Determiner si en flow ───────────────────────
        let was_in_flow = self.in_flow;
        self.in_flow = flow_score > 0.7;
        self.flow_intensity = if self.in_flow {
            flow_score
        } else {
            (flow_score * 0.5).min(0.5)
        };

        // ─── Compteurs de duree ──────────────────────────
        if self.in_flow {
            self.duration_cycles += 1;
            self.total_flow_cycles += 1;
        } else if was_in_flow {
            // On sort du flow
            self.duration_cycles = 0;
        }
    }

    /// Retourne l'influence chimique de l'etat de flow.
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

    /// Description concise pour le prompt LLM.
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
