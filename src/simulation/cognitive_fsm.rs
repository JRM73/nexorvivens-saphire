// =============================================================================
// cognitive_fsm.rs — Machine a etats finis augmentee pour les etats cognitifs
//
// Role : Modelise les etats cognitifs de Saphire comme une FSM (Finite State
//        Machine). Les transitions sont basees sur la chimie et les sentiments,
//        pas sur des evenements discrets. Chaque etat influence le pipeline.
//
// Etats :
//   - Eveil : etat normal, cognition standard
//   - Focus : concentration intense, noradrenaline elevee
//   - Reverie : pensees libres, dopamine + serotonine elevees
//   - Stress : cortisol eleve, mode defensif
//   - Flow : etat optimal, dopamine + noradrenaline equilibrees
//   - Repos : fatigue, tous les transmetteurs bas
//
// Place dans l'architecture :
//   Consulte dans lifecycle/mod.rs a chaque cycle pour moduler le
//   comportement du pipeline (intervalle de pensee, type privilegie, etc.).
// =============================================================================

use serde::{Serialize, Deserialize};

/// Etats cognitifs de la FSM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CognitiveState {
    /// Etat normal, cognition standard
    Eveil,
    /// Concentration intense (noradrenaline dominante)
    Focus,
    /// Pensees libres et creatives (dopamine + serotonine)
    Reverie,
    /// Mode defensif, stress eleve (cortisol dominant)
    Stress,
    /// Etat optimal de performance (equilibre parfait)
    Flow,
    /// Fatigue, recuperation necessaire
    Repos,
}

impl CognitiveState {
    /// Nom affichable de l'etat.
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

    /// Multiplicateur d'intervalle de pensee pour cet etat.
    /// < 1.0 = pensees plus rapides, > 1.0 = pensees plus lentes.
    pub fn thought_interval_multiplier(&self) -> f64 {
        match self {
            CognitiveState::Flow => 0.7,     // Plus rapide en flow
            CognitiveState::Focus => 0.8,    // Rapide en focus
            CognitiveState::Stress => 1.5,   // Ralenti par le stress
            CognitiveState::Repos => 2.0,    // Lent en repos
            CognitiveState::Reverie => 1.2,  // Legerement lent en reverie
            CognitiveState::Eveil => 1.0,    // Normal
        }
    }

    /// Types de pensees privilegies dans cet etat.
    pub fn preferred_thought_types(&self) -> &[usize] {
        match self {
            CognitiveState::Eveil => &[],          // Pas de preference
            CognitiveState::Focus => &[1, 6, 10],  // Exploration, Curiosite, AlgorithmicReflection
            CognitiveState::Reverie => &[7, 3, 13],// Daydream, Continuation, DesireFormation
            CognitiveState::Stress => &[0, 5],      // Introspection, SelfAnalysis
            CognitiveState::Flow => &[1, 6, 7, 4], // Exploration, Curiosite, Daydream, Existential
            CognitiveState::Repos => &[2, 8],       // MemoryReflection, TemporalAwareness
        }
    }
}

/// La FSM cognitive avec historique des transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveFsm {
    /// Etat courant
    pub current_state: CognitiveState,
    /// Nombre de cycles dans l'etat courant
    pub cycles_in_state: u64,
    /// Historique des etats recents (taille 20)
    pub state_history: Vec<CognitiveState>,
    /// Nombre total de transitions
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

    /// Evalue les transitions possibles depuis l'etat courant
    /// en fonction de la chimie et des sentiments.
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

    /// Determine le prochain etat base sur les niveaux chimiques.
    fn evaluate_transition(
        &self,
        cortisol: f64,
        dopamine: f64,
        serotonin: f64,
        noradrenaline: f64,
        endorphin: f64,
    ) -> CognitiveState {
        // Hysteresie : rester dans l'etat courant si pas de signal fort
        // (evite les oscillations rapides entre etats)
        let hysteresis = if self.cycles_in_state < 3 { 0.1 } else { 0.0 };

        // Stress : cortisol tres eleve
        if cortisol > 0.75 + hysteresis {
            return CognitiveState::Stress;
        }

        // Repos : tout est bas (fatigue)
        if dopamine < 0.25 && serotonin < 0.3 && noradrenaline < 0.25 && endorphin < 0.2 {
            return CognitiveState::Repos;
        }

        // Flow : equilibre optimal (dopamine + noradrenaline elevees, cortisol modere)
        if dopamine > 0.55 + hysteresis
            && noradrenaline > 0.45 + hysteresis
            && cortisol < 0.45
            && serotonin > 0.4
        {
            return CognitiveState::Flow;
        }

        // Focus : noradrenaline dominante
        if noradrenaline > 0.6 + hysteresis && cortisol < 0.5 {
            return CognitiveState::Focus;
        }

        // Reverie : dopamine + serotonine elevees, noradrenaline basse
        if dopamine > 0.5 + hysteresis
            && serotonin > 0.5 + hysteresis
            && noradrenaline < 0.4
            && cortisol < 0.4
        {
            return CognitiveState::Reverie;
        }

        // Defaut : Eveil
        CognitiveState::Eveil
    }

    /// Description pour le prompt LLM.
    pub fn describe_for_prompt(&self) -> String {
        format!(
            "ETAT COGNITIF : {} (depuis {} cycles)",
            self.current_state.as_str(),
            self.cycles_in_state,
        )
    }

    /// JSON pour le dashboard.
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
        // Cortisol eleve → Stress
        for _ in 0..4 { // Depasser l'hysteresie
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
        // Un seul tick stressant ne devrait pas suffire si on vient d'arriver
        fsm.tick(0.8, 0.3, 0.3, 0.3, 0.3);
        // L'etat peut changer car 0.8 > 0.75 + 0.1 = false pour le premier cycle
        // Mais 0.8 > 0.85 est false, donc reste en Eveil
        assert_eq!(fsm.current_state, CognitiveState::Eveil);
    }

    #[test]
    fn test_thought_multiplier() {
        assert!(CognitiveState::Flow.thought_interval_multiplier() < 1.0);
        assert!(CognitiveState::Repos.thought_interval_multiplier() > 1.0);
    }
}
