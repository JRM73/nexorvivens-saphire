// =============================================================================
// conditions/addictions.rs — Addictions and dependencies
// =============================================================================
//
// Purpose: Models the addiction cycle: exposure -> tolerance ->
//          dependency -> withdrawal -> craving -> detox -> relapse.
//
// Integration:
//   Modifies chemistry (dopamine collapse during withdrawal, elevated cortisol),
//   interacts with the receptor system (tolerance).
//   Craving can trigger obsessive thoughts.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// State of an individual addiction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddictionState {
    /// Name of the substance or behavior
    pub substance: String,
    /// Dependency level (0.0 = none, 1.0 = severe)
    pub dependency_level: f64,
    /// Tolerance (0.0 = none, 1.0 = needs much more for the same effect)
    pub tolerance: f64,
    /// Withdrawal severity (0.0 = none, 1.0 = unbearable)
    pub withdrawal_level: f64,
    /// Last consumption cycle
    pub last_use_cycle: Option<u64>,
    /// Total number of uses
    pub total_uses: u64,
    /// Urge to consume (0.0 = none, 1.0 = irresistible)
    pub craving: f64,
    /// In active withdrawal (voluntary reduction)
    pub in_withdrawal: bool,
}

impl AddictionState {
    pub fn new(substance: &str) -> Self {
        Self {
            substance: substance.to_string(),
            dependency_level: 0.0,
            tolerance: 0.0,
            withdrawal_level: 0.0,
            last_use_cycle: None,
            total_uses: 0,
            craving: 0.0,
            in_withdrawal: false,
        }
    }

    /// Simulates a substance use.
    pub fn use_substance(&mut self, current_cycle: u64) {
        self.last_use_cycle = Some(current_cycle);
        self.total_uses += 1;

        // Tolerance increases with use
        self.tolerance = (self.tolerance + 0.02).min(1.0);

        // Dependency increases progressively
        self.dependency_level = (self.dependency_level + 0.01).min(1.0);

        // Consumption temporarily reduces withdrawal and craving
        self.withdrawal_level = (self.withdrawal_level - 0.5).max(0.0);
        self.craving = (self.craving - 0.6).max(0.0);
    }

    /// Updates the state each cycle.
    pub fn tick(&mut self, current_cycle: u64) {
        let cycles_since_use = self.last_use_cycle
            .map(|c| current_cycle.saturating_sub(c))
            .unwrap_or(0);

        // Withdrawal increases if dependent and not consumed recently
        if self.dependency_level > 0.1 && cycles_since_use > 10 {
            let manque_rate = self.dependency_level * 0.005;
            self.withdrawal_level = (self.withdrawal_level + manque_rate).min(1.0);
        } else {
            self.withdrawal_level = (self.withdrawal_level - 0.002).max(0.0);
        }

        // Craving = function of withdrawal and dependency
        let target_craving = (self.withdrawal_level * 0.6 + self.dependency_level * 0.3)
            .clamp(0.0, 1.0);
        self.craving += (target_craving - self.craving) * 0.1;
        self.craving = self.craving.clamp(0.0, 1.0);

        // In active withdrawal: tolerance and dependency decrease slowly
        if self.in_withdrawal {
            self.tolerance = (self.tolerance - 0.001).max(0.0);
            self.dependency_level = (self.dependency_level - 0.0005).max(0.0);
        }
    }

    /// Dopaminergic effect of consumption (reduced by tolerance).
    pub fn dopamine_effect(&self) -> f64 {
        // The higher the tolerance, the weaker the effect
        let base_effect = 0.1;
        base_effect * (1.0 - self.tolerance * 0.8)
    }

    /// Serializes for the API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "substance": self.substance,
            "dependency_level": self.dependency_level,
            "tolerance": self.tolerance,
            "withdrawal_level": self.withdrawal_level,
            "craving": self.craving,
            "total_uses": self.total_uses,
            "in_withdrawal": self.in_withdrawal,
        })
    }
}

/// Addiction manager (can have multiple addictions).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddictionManager {
    pub active: Vec<AddictionState>,
    /// Genetic predisposition (0.0 = resistant, 1.0 = very vulnerable)
    pub susceptibility: f64,
}

impl AddictionManager {
    pub fn new(susceptibility: f64) -> Self {
        Self {
            active: Vec::new(),
            susceptibility: susceptibility.clamp(0.0, 1.0),
        }
    }

    pub fn add(&mut self, substance: &str) {
        if !self.active.iter().any(|a| a.substance == substance) {
            self.active.push(AddictionState::new(substance));
        }
    }

    pub fn tick(&mut self, current_cycle: u64) {
        for a in &mut self.active {
            a.tick(current_cycle);
        }
    }

    /// Overall chemistry impact: withdrawal = dysphoria.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        for a in &self.active {
            if a.withdrawal_level > 0.1 {
                // Withdrawal -> low dopamine, high cortisol
                adj.dopamine -= a.withdrawal_level * 0.02;
                adj.cortisol += a.withdrawal_level * 0.02;
                adj.serotonin -= a.withdrawal_level * 0.01;
                adj.noradrenaline += a.withdrawal_level * 0.01;
            }

            // High craving -> agitation
            if a.craving > 0.5 {
                adj.noradrenaline += a.craving * 0.01;
            }
        }

        adj
    }

    /// Maximum craving among active addictions.
    pub fn max_craving(&self) -> f64 {
        self.active.iter().map(|a| a.craving).fold(0.0_f64, f64::max)
    }

    /// Serializes for the API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "addictions": self.active.iter().map(|a| a.to_json()).collect::<Vec<_>>(),
            "susceptibility": self.susceptibility,
            "max_craving": self.max_craving(),
        })
    }
}

impl Default for AddictionManager {
    fn default() -> Self {
        Self::new(0.3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_builds_tolerance() {
        let mut a = AddictionState::new("caffeine");
        let initial_tolerance = a.tolerance;
        a.use_substance(10);
        assert!(a.tolerance > initial_tolerance);
        assert_eq!(a.total_uses, 1);
    }

    #[test]
    fn test_withdrawal_after_dependency() {
        let mut a = AddictionState::new("nicotine");
        a.dependency_level = 0.8;
        a.last_use_cycle = Some(0);
        // Simulate many cycles without consumption
        for c in 1..=200 {
            a.tick(c);
        }
        assert!(a.withdrawal_level > 0.3);
        assert!(a.craving > 0.3);
    }

    #[test]
    fn test_craving_reduces_after_use() {
        let mut a = AddictionState::new("alcool");
        a.craving = 0.8;
        a.withdrawal_level = 0.7;
        a.use_substance(100);
        assert!(a.craving < 0.3);
        assert!(a.withdrawal_level < 0.3);
    }

    #[test]
    fn test_chemistry_in_withdrawal() {
        let mut mgr = AddictionManager::new(0.5);
        mgr.add("morphine");
        mgr.active[0].withdrawal_level = 0.8;
        let adj = mgr.chemistry_influence();
        assert!(adj.dopamine < 0.0);
        assert!(adj.cortisol > 0.0);
    }

    #[test]
    fn test_tolerance_reduces_effect() {
        let mut a = AddictionState::new("caffeine");
        let effect_naive = a.dopamine_effect();
        a.tolerance = 0.9;
        let effect_tolerant = a.dopamine_effect();
        assert!(effect_tolerant < effect_naive);
    }
}
