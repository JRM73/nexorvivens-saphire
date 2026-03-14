// =============================================================================
// needs/mod.rs — Primary needs (hunger, thirst)
// =============================================================================
//
// Role: Behavioral layer above physiology. The hunger and thirst drives
//       derive from blood sugar and hydration of the virtual body, impact
//       neurochemistry (cortisol, serotonin, dopamine) and trigger
//       autonomous actions (eating/drinking) when thresholds are exceeded.
//
// Place in the architecture:
//   PrimaryNeeds is owned by SaphireAgent. It is updated via tick()
//   on each autonomous cycle (phase_needs in thinking.rs). The actions
//   eat()/drink() directly modify the virtual body's physiology.
//   The chemical impact is applied in the cognitive pipeline.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::config::NeedsConfig;
use crate::world::ChemistryAdjustment;

/// Hunger drive — derived from blood sugar and time since the last meal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HungerDrive {
    /// Hunger level: 0.0 (satiated) to 1.0 (starving)
    pub level: f64,
    /// Last cycle when a meal was taken
    pub last_meal_cycle: u64,
    /// Total number of meals taken since birth
    pub meals_count: u64,
    /// Currently eating (brief, 1 cycle)
    pub is_eating: bool,
}

impl HungerDrive {
    pub fn new() -> Self {
        Self {
            level: 0.0,
            last_meal_cycle: 0,
            meals_count: 0,
            is_eating: false,
        }
    }
}

/// Thirst drive — derived from hydration and time since the last drink.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirstDrive {
    /// Thirst level: 0.0 (hydrated) to 1.0 (parched)
    pub level: f64,
    /// Last cycle when a drink was taken
    pub last_drink_cycle: u64,
    /// Total number of drinks taken since birth
    pub drinks_count: u64,
    /// Currently drinking (brief, 1 cycle)
    pub is_drinking: bool,
}

impl ThirstDrive {
    pub fn new() -> Self {
        Self {
            level: 0.0,
            last_drink_cycle: 0,
            drinks_count: 0,
            is_drinking: false,
        }
    }
}

/// Primary needs of the agent (hunger + thirst).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryNeeds {
    pub hunger: HungerDrive,
    pub thirst: ThirstDrive,
    pub enabled: bool,
}

impl PrimaryNeeds {
    /// Creates a new primary needs system.
    pub fn new(enabled: bool) -> Self {
        Self {
            hunger: HungerDrive::new(),
            thirst: ThirstDrive::new(),
            enabled,
        }
    }

    /// Updates hunger and thirst drives from the physiological state.
    ///
    /// Hunger derives from blood sugar: the lower it is, the more hunger rises.
    /// Thirst derives from hydration: the lower it drops, the more thirst rises.
    /// A time factor is added (cycles since the last meal/drink).
    pub fn tick(&mut self, glycemia: f64, hydration: f64, cycle: u64, config: &NeedsConfig) {
        if !self.enabled {
            return;
        }

        // Reset action flags
        self.hunger.is_eating = false;
        self.thirst.is_drinking = false;

        // ─── Hunger ──────────────────────────────────────────────
        // Base: inverse of normalized blood sugar (5.0 = normal)
        // More aggressive curve in hypoglycemia zone (< 3.9 mmol/L)
        let glycemia_factor = if glycemia < 3.0 {
            // Severe hypoglycemia: urgent maximum hunger
            0.95
        } else if glycemia < 3.9 {
            // Hypoglycemia: accelerated curve (0.6 -> 0.95)
            0.6 + (3.9 - glycemia) / 0.9 * 0.35
        } else {
            // Normal: gentle curve
            1.0 - (glycemia / 5.0).clamp(0.0, 1.0)
        };
        // Time factor: rises slowly with cycles since the last meal
        let cycles_since_meal = cycle.saturating_sub(self.hunger.last_meal_cycle);
        let time_factor = (cycles_since_meal as f64 * config.hunger_rise_rate).clamp(0.0, 0.4);
        self.hunger.level = (glycemia_factor + time_factor).clamp(0.0, 1.0);

        // ─── Thirst ──────────────────────────────────────────────
        // Base: inverse of hydration
        let hydration_factor = 1.0 - hydration;
        // Time factor: rises faster than hunger
        let cycles_since_drink = cycle.saturating_sub(self.thirst.last_drink_cycle);
        let time_factor_thirst = (cycles_since_drink as f64 * config.thirst_rise_rate).clamp(0.0, 0.3);
        self.thirst.level = (hydration_factor + time_factor_thirst).clamp(0.0, 1.0);
    }

    /// Is the agent hungry? (exceeds the hunger threshold)
    pub fn is_hungry(&self, config: &NeedsConfig) -> bool {
        self.enabled && self.hunger.level > config.hunger_threshold
    }

    /// Is the agent thirsty? (exceeds the thirst threshold)
    pub fn is_thirsty(&self, config: &NeedsConfig) -> bool {
        self.enabled && self.thirst.level > config.thirst_threshold
    }

    /// Eating action: resets hunger, boosts blood sugar in the physiology.
    /// Returns the blood sugar boost to apply and the dopamine boost.
    pub fn eat(&mut self, cycle: u64, config: &NeedsConfig) -> EatResult {
        self.hunger.level = 0.0;
        self.hunger.last_meal_cycle = cycle;
        self.hunger.meals_count += 1;
        self.hunger.is_eating = true;
        EatResult {
            glycemia_target: config.meal_glycemia_target,
            dopamine_boost: 0.08,
        }
    }

    /// Drinking action: resets thirst, boosts hydration in the physiology.
    /// Returns the hydration boost to apply and the dopamine boost.
    pub fn drink(&mut self, cycle: u64, config: &NeedsConfig) -> DrinkResult {
        self.thirst.level = 0.0;
        self.thirst.last_drink_cycle = cycle;
        self.thirst.drinks_count += 1;
        self.thirst.is_drinking = true;
        DrinkResult {
            hydration_target: config.drink_hydration_target,
            dopamine_boost: 0.05,
        }
    }

    /// Computes the chemical impact of unmet needs.
    ///
    /// - Hunger > 0.6: cortisol+, serotonin- (irritability)
    /// - Hunger > 0.8: cortisol++, dopamine- (more aggressive)
    /// - Thirst > 0.5: cortisol+ (mild anxiety)
    /// - Thirst > 0.7: cortisol++, noradrenaline+ (urgency)
    pub fn chemistry_influence(&self, config: &NeedsConfig) -> ChemistryAdjustment {
        if !self.enabled {
            return ChemistryAdjustment::default();
        }

        let mut adj = ChemistryAdjustment::default();

        // Hunger impact
        if self.hunger.level > config.hunger_threshold {
            adj.cortisol += 0.015;
            adj.serotonin -= 0.01;
        }
        if self.hunger.level > 0.8 {
            adj.cortisol += 0.03;
            adj.dopamine -= 0.01;
        }

        // Thirst impact
        if self.thirst.level > config.thirst_threshold {
            adj.cortisol += 0.01;
        }
        if self.thirst.level > 0.7 {
            adj.cortisol += 0.025;
            adj.noradrenaline += 0.01;
        }

        adj
    }

    /// Checks if a need should be auto-satisfied and returns the action.
    pub fn check_auto_satisfy(&self, config: &NeedsConfig) -> Option<AutoSatisfyAction> {
        if !self.enabled || !config.auto_satisfy {
            return None;
        }

        // Priority to thirst (more urgent)
        if self.thirst.level >= config.auto_drink_threshold {
            return Some(AutoSatisfyAction::Drink);
        }
        if self.hunger.level >= config.auto_eat_threshold {
            return Some(AutoSatisfyAction::Eat);
        }

        None
    }

    /// Returns the needs state for the API / broadcast.
    pub fn to_status_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "hunger": {
                "level": self.hunger.level,
                "last_meal_cycle": self.hunger.last_meal_cycle,
                "meals_count": self.hunger.meals_count,
                "is_eating": self.hunger.is_eating,
            },
            "thirst": {
                "level": self.thirst.level,
                "last_drink_cycle": self.thirst.last_drink_cycle,
                "drinks_count": self.thirst.drinks_count,
                "is_drinking": self.thirst.is_drinking,
            },
        })
    }
}

/// Result of a meal.
pub struct EatResult {
    /// Target blood sugar level to reach
    pub glycemia_target: f64,
    /// Dopamine boost
    pub dopamine_boost: f64,
}

/// Result of a drink.
pub struct DrinkResult {
    /// Target hydration level to reach
    pub hydration_target: f64,
    /// Dopamine boost
    pub dopamine_boost: f64,
}

/// Automatic satisfaction action.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutoSatisfyAction {
    Eat,
    Drink,
}
