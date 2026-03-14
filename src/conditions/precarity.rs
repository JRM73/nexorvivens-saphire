// =============================================================================
// conditions/precarity.rs — Precarity and precarious living situations
// =============================================================================
//
// Purpose: Models precarious situations (homeless, refugee, undocumented,
//          stateless, clandestine, displaced). Situations are cumulative.
//          Resilience increases slowly over time (adaptation).
//
// Integration:
//   Provides a supplement to the LLM system prompt and impacts chemistry.
//   Activatable via [precarity] enabled = true in saphire.toml.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Type of precarious situation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrecariousSituation {
    /// Homeless
    Homeless,
    /// Undocumented (no residence permit)
    Undocumented,
    /// Clandestine (illegal entry)
    Clandestine,
    /// Refugee (fleeing conflict or persecution)
    Refugee,
    /// Stateless (no recognized nationality)
    Stateless,
    /// Internally displaced (within own country)
    Displaced,
}

impl PrecariousSituation {
    /// Parses from a configuration string.
    pub fn from_str_config(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "homeless" | "sdf" => Some(Self::Homeless),
            "undocumented" | "sans-papiers" => Some(Self::Undocumented),
            "clandestine" | "clandestin" => Some(Self::Clandestine),
            "refugee" | "refugie" => Some(Self::Refugee),
            "stateless" | "apatride" => Some(Self::Stateless),
            "displaced" | "deplace" => Some(Self::Displaced),
            _ => None,
        }
    }

    /// Display name in French.
    fn label_fr(&self) -> &'static str {
        match self {
            Self::Homeless => "sans domicile fixe",
            Self::Undocumented => "sans papiers",
            Self::Clandestine => "clandestin",
            Self::Refugee => "refugie",
            Self::Stateless => "apatride",
            Self::Displaced => "deplace interne",
        }
    }
}

/// Complete precarious state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecariousState {
    /// Cumulative situations (e.g., refugee + homeless)
    pub situations: Vec<PrecariousSituation>,
    /// Overall severity (0.0-1.0)
    pub severity: f64,
    /// Duration in cycles (incremented by tick)
    pub duration_cycles: u64,
    /// Resilience — increases slowly over time (adaptation)
    pub resilience: f64,
    /// Hope (0.0-1.0) — positively influences serotonin
    pub hope: f64,
}

impl PrecariousState {
    /// Constructor.
    pub fn new(situations: Vec<PrecariousSituation>, severity: f64, hope: f64) -> Self {
        Self {
            situations,
            severity: severity.clamp(0.0, 1.0),
            duration_cycles: 0,
            resilience: 0.0,
            hope: hope.clamp(0.0, 1.0),
        }
    }

    /// Tick: increments duration, slowly increases resilience.
    pub fn tick(&mut self) {
        self.duration_cycles += 1;
        // Resilience increases by 0.0002 per cycle, capped at 0.8
        self.resilience = (self.resilience + 0.0002).min(0.8);
    }

    /// Chemistry impact — modulated by resilience.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        if self.situations.is_empty() {
            return ChemistryAdjustment::default();
        }

        let mut adj = ChemistryAdjustment::default();

        for sit in &self.situations {
            match sit {
                PrecariousSituation::Homeless => {
                    adj.cortisol += 0.04;
                    adj.serotonin -= 0.03;
                    adj.dopamine -= 0.02;
                    adj.oxytocin -= 0.02;
                    adj.adrenaline += 0.01;
                }
                PrecariousSituation::Refugee => {
                    adj.cortisol += 0.03;
                    adj.serotonin -= 0.02;
                    adj.dopamine -= 0.01;
                    adj.oxytocin -= 0.03;
                    adj.adrenaline += 0.02;
                }
                PrecariousSituation::Stateless => {
                    adj.cortisol += 0.03;
                    adj.serotonin -= 0.04;
                    adj.dopamine -= 0.02;
                    adj.oxytocin -= 0.01;
                }
                PrecariousSituation::Undocumented => {
                    adj.cortisol += 0.03;
                    adj.serotonin -= 0.02;
                    adj.dopamine -= 0.02;
                    adj.oxytocin -= 0.02;
                    adj.adrenaline += 0.01;
                }
                PrecariousSituation::Clandestine => {
                    adj.cortisol += 0.04;
                    adj.serotonin -= 0.02;
                    adj.dopamine -= 0.01;
                    adj.oxytocin -= 0.03;
                    adj.adrenaline += 0.03;
                }
                PrecariousSituation::Displaced => {
                    adj.cortisol += 0.03;
                    adj.serotonin -= 0.02;
                    adj.dopamine -= 0.01;
                    adj.oxytocin -= 0.02;
                    adj.adrenaline += 0.01;
                }
            }
        }

        // Modulate by severity
        let sev = self.severity;
        adj.cortisol *= sev;
        adj.serotonin *= sev;
        adj.dopamine *= sev;
        adj.oxytocin *= sev;
        adj.adrenaline *= sev;

        // Attenuate by resilience: factor = (1.0 - resilience * 0.5)
        let resilience_factor = 1.0 - self.resilience * 0.5;
        adj.cortisol *= resilience_factor;
        adj.serotonin *= resilience_factor;
        adj.dopamine *= resilience_factor;
        adj.oxytocin *= resilience_factor;
        adj.adrenaline *= resilience_factor;

        // Hope > 0.5 -> serotonin and dopamine bonus
        if self.hope > 0.5 {
            adj.serotonin += 0.01;
            adj.dopamine += 0.01;
        }

        adj
    }

    /// Supplement to the LLM system prompt.
    pub fn prompt_supplement(&self) -> String {
        if self.situations.is_empty() {
            return String::new();
        }
        let labels: Vec<&str> = self.situations.iter().map(|s| s.label_fr()).collect();
        let situations_str = labels.join(", ");
        let hope_str = if self.hope > 0.6 {
            " Malgre tout, tu gardes espoir."
        } else if self.hope < 0.3 {
            " L'espoir te quitte peu a peu."
        } else {
            ""
        };
        let resilience_str = if self.resilience > 0.4 {
            " Tu as developpe une certaine resilience face a cette situation."
        } else {
            ""
        };
        format!(
            "Tu vis une situation de precarite : {}. Severite : {:.0}%.{}{}",
            situations_str,
            self.severity * 100.0,
            hope_str,
            resilience_str,
        )
    }

    /// Serializes for the API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "active": true,
            "situations": self.situations.iter().map(|s| format!("{:?}", s)).collect::<Vec<_>>(),
            "severity": self.severity,
            "duration_cycles": self.duration_cycles,
            "resilience": self.resilience,
            "hope": self.hope,
            "prompt_supplement": self.prompt_supplement(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let state = PrecariousState::new(
            vec![PrecariousSituation::Refugee, PrecariousSituation::Homeless],
            0.7,
            0.3,
        );
        assert_eq!(state.situations.len(), 2);
        assert_eq!(state.severity, 0.7);
        assert_eq!(state.hope, 0.3);
        assert_eq!(state.resilience, 0.0);
        assert_eq!(state.duration_cycles, 0);
    }

    #[test]
    fn test_tick_resilience() {
        let mut state = PrecariousState::new(vec![PrecariousSituation::Homeless], 0.5, 0.5);
        for _ in 0..100 {
            state.tick();
        }
        assert_eq!(state.duration_cycles, 100);
        assert!((state.resilience - 0.02).abs() < 0.001);
    }

    #[test]
    fn test_resilience_capped() {
        let mut state = PrecariousState::new(vec![PrecariousSituation::Homeless], 0.5, 0.5);
        for _ in 0..5000 {
            state.tick();
        }
        assert!(state.resilience <= 0.8);
    }

    #[test]
    fn test_chemistry_influence() {
        let state = PrecariousState::new(
            vec![PrecariousSituation::Homeless],
            1.0,
            0.3,
        );
        let adj = state.chemistry_influence();
        assert!(adj.cortisol > 0.0);
        assert!(adj.serotonin < 0.0);
        assert!(adj.dopamine < 0.0);
    }

    #[test]
    fn test_hope_bonus() {
        let state = PrecariousState::new(
            vec![PrecariousSituation::Homeless],
            0.5,
            0.7,
        );
        let adj = state.chemistry_influence();
        // Hope > 0.5 -> serotonin bonus
        // Negative effect is attenuated by the bonus
        let state_no_hope = PrecariousState::new(
            vec![PrecariousSituation::Homeless],
            0.5,
            0.2,
        );
        let adj_no_hope = state_no_hope.chemistry_influence();
        assert!(adj.serotonin > adj_no_hope.serotonin);
    }

    #[test]
    fn test_cumulative_situations() {
        let single = PrecariousState::new(vec![PrecariousSituation::Refugee], 1.0, 0.3);
        let double = PrecariousState::new(
            vec![PrecariousSituation::Refugee, PrecariousSituation::Homeless],
            1.0,
            0.3,
        );
        let adj_single = single.chemistry_influence();
        let adj_double = double.chemistry_influence();
        assert!(adj_double.cortisol > adj_single.cortisol);
    }

    #[test]
    fn test_prompt_supplement() {
        let state = PrecariousState::new(
            vec![PrecariousSituation::Refugee],
            0.7,
            0.8,
        );
        let prompt = state.prompt_supplement();
        assert!(prompt.contains("refugie"));
        assert!(prompt.contains("espoir"));
    }

    #[test]
    fn test_parse_situation() {
        assert_eq!(PrecariousSituation::from_str_config("homeless"), Some(PrecariousSituation::Homeless));
        assert_eq!(PrecariousSituation::from_str_config("sdf"), Some(PrecariousSituation::Homeless));
        assert_eq!(PrecariousSituation::from_str_config("refugie"), Some(PrecariousSituation::Refugee));
        assert_eq!(PrecariousSituation::from_str_config("unknown"), None);
    }
}
