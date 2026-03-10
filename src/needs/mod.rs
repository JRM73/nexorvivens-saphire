// =============================================================================
// needs/mod.rs — Besoins primaires (faim, soif)
// =============================================================================
//
// Role : Couche comportementale au-dessus de la physiologie. Les drives de
//        faim et soif derivent de la glycemie et l'hydratation du corps
//        virtuel, impactent la neurochimie (cortisol, serotonine, dopamine)
//        et declenchent des actions autonomes (manger/boire) quand les seuils
//        sont depasses.
//
// Place dans l'architecture :
//   PrimaryNeeds est possede par SaphireAgent. Il est mis a jour via tick()
//   a chaque cycle autonome (phase_needs dans thinking.rs). Les actions
//   eat()/drink() modifient directement la physiologie du corps virtuel.
//   L'impact chimique est applique dans le pipeline cognitif.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::config::NeedsConfig;
use crate::world::ChemistryAdjustment;

/// Drive de faim — derive de la glycemie et du temps depuis le dernier repas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HungerDrive {
    /// Niveau de faim : 0.0 (rassasie) a 1.0 (affame)
    pub level: f64,
    /// Dernier cycle ou un repas a ete pris
    pub last_meal_cycle: u64,
    /// Nombre total de repas pris depuis la naissance
    pub meals_count: u64,
    /// En train de manger (bref, 1 cycle)
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

/// Drive de soif — derive de l'hydratation et du temps depuis la derniere boisson.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirstDrive {
    /// Niveau de soif : 0.0 (hydrate) a 1.0 (assoiffe)
    pub level: f64,
    /// Dernier cycle ou une boisson a ete prise
    pub last_drink_cycle: u64,
    /// Nombre total de boissons prises depuis la naissance
    pub drinks_count: u64,
    /// En train de boire (bref, 1 cycle)
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

/// Besoins primaires de l'agent (faim + soif).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryNeeds {
    pub hunger: HungerDrive,
    pub thirst: ThirstDrive,
    pub enabled: bool,
}

impl PrimaryNeeds {
    /// Cree un nouveau systeme de besoins primaires.
    pub fn new(enabled: bool) -> Self {
        Self {
            hunger: HungerDrive::new(),
            thirst: ThirstDrive::new(),
            enabled,
        }
    }

    /// Met a jour les drives de faim et soif a partir de l'etat physiologique.
    ///
    /// La faim derive de la glycemie : plus elle est basse, plus la faim monte.
    /// La soif derive de l'hydratation : plus elle baisse, plus la soif monte.
    /// Un facteur temps s'ajoute (cycles depuis le dernier repas/boisson).
    pub fn tick(&mut self, glycemia: f64, hydration: f64, cycle: u64, config: &NeedsConfig) {
        if !self.enabled {
            return;
        }

        // Reset des flags d'action
        self.hunger.is_eating = false;
        self.thirst.is_drinking = false;

        // ─── Faim ──────────────────────────────────────────────
        // Base : inverse de la glycemie normalisee (5.0 = normal)
        // Courbe plus agressive en zone d'hypoglycemie (< 3.9 mmol/L)
        let glycemia_factor = if glycemia < 3.0 {
            // Hypoglycemie severe : faim maximale urgente
            0.95
        } else if glycemia < 3.9 {
            // Hypoglycemie : courbe acceleree (0.6 → 0.95)
            0.6 + (3.9 - glycemia) / 0.9 * 0.35
        } else {
            // Normal : courbe douce
            1.0 - (glycemia / 5.0).clamp(0.0, 1.0)
        };
        // Facteur temps : monte lentement avec les cycles depuis le dernier repas
        let cycles_since_meal = cycle.saturating_sub(self.hunger.last_meal_cycle);
        let time_factor = (cycles_since_meal as f64 * config.hunger_rise_rate).clamp(0.0, 0.4);
        self.hunger.level = (glycemia_factor + time_factor).clamp(0.0, 1.0);

        // ─── Soif ──────────────────────────────────────────────
        // Base : inverse de l'hydratation
        let hydration_factor = 1.0 - hydration;
        // Facteur temps : monte plus vite que la faim
        let cycles_since_drink = cycle.saturating_sub(self.thirst.last_drink_cycle);
        let time_factor_thirst = (cycles_since_drink as f64 * config.thirst_rise_rate).clamp(0.0, 0.3);
        self.thirst.level = (hydration_factor + time_factor_thirst).clamp(0.0, 1.0);
    }

    /// L'agent est-il affame ? (depasse le seuil de faim)
    pub fn is_hungry(&self, config: &NeedsConfig) -> bool {
        self.enabled && self.hunger.level > config.hunger_threshold
    }

    /// L'agent est-il assoiffe ? (depasse le seuil de soif)
    pub fn is_thirsty(&self, config: &NeedsConfig) -> bool {
        self.enabled && self.thirst.level > config.thirst_threshold
    }

    /// Action de manger : reset la faim, boost la glycemie dans la physiologie.
    /// Retourne le boost de glycemie a appliquer et le boost de dopamine.
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

    /// Action de boire : reset la soif, boost l'hydratation dans la physiologie.
    /// Retourne le boost d'hydratation a appliquer et le boost de dopamine.
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

    /// Calcule l'impact chimique des besoins non satisfaits.
    ///
    /// - Faim > 0.6 : cortisol+, serotonine- (irritabilite)
    /// - Faim > 0.8 : cortisol++, dopamine- (plus agressif)
    /// - Soif > 0.5 : cortisol+ (anxiete legere)
    /// - Soif > 0.7 : cortisol++, noradrenaline+ (urgence)
    pub fn chemistry_influence(&self, config: &NeedsConfig) -> ChemistryAdjustment {
        if !self.enabled {
            return ChemistryAdjustment::default();
        }

        let mut adj = ChemistryAdjustment::default();

        // Impact de la faim
        if self.hunger.level > config.hunger_threshold {
            adj.cortisol += 0.015;
            adj.serotonin -= 0.01;
        }
        if self.hunger.level > 0.8 {
            adj.cortisol += 0.03;
            adj.dopamine -= 0.01;
        }

        // Impact de la soif
        if self.thirst.level > config.thirst_threshold {
            adj.cortisol += 0.01;
        }
        if self.thirst.level > 0.7 {
            adj.cortisol += 0.025;
            adj.noradrenaline += 0.01;
        }

        adj
    }

    /// Verifie si un besoin doit etre auto-satisfait et retourne l'action.
    pub fn check_auto_satisfy(&self, config: &NeedsConfig) -> Option<AutoSatisfyAction> {
        if !self.enabled || !config.auto_satisfy {
            return None;
        }

        // Priorite a la soif (plus urgent)
        if self.thirst.level >= config.auto_drink_threshold {
            return Some(AutoSatisfyAction::Drink);
        }
        if self.hunger.level >= config.auto_eat_threshold {
            return Some(AutoSatisfyAction::Eat);
        }

        None
    }

    /// Retourne l'etat des besoins pour l'API / broadcast.
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

/// Resultat d'un repas.
pub struct EatResult {
    /// Glycemie cible a atteindre
    pub glycemia_target: f64,
    /// Boost de dopamine
    pub dopamine_boost: f64,
}

/// Resultat d'une boisson.
pub struct DrinkResult {
    /// Hydratation cible a atteindre
    pub hydration_target: f64,
    /// Boost de dopamine
    pub dopamine_boost: f64,
}

/// Action de satisfaction automatique.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutoSatisfyAction {
    Eat,
    Drink,
}
