// psychology/freudian.rs — Stub for the lite edition

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EgoStrategy {
    Rational,
    Overwhelmed,
    Defensive,
    Adaptive,
}

impl Default for EgoStrategy {
    fn default() -> Self { EgoStrategy::Rational }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Id {
    pub drive_strength: f64,
    pub frustration: f64,
    pub active_drives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Ego {
    pub strength: f64,
    pub anxiety: f64,
    pub strategy: EgoStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SuperEgo {
    pub strength: f64,
    pub guilt: f64,
    pub pride: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PsychicBalance {
    pub internal_conflict: f64,
    pub psychic_health: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreudianFramework {
    pub id: Id,
    pub ego: Ego,
    pub superego: SuperEgo,
    pub balance: PsychicBalance,
    pub active_defenses: Vec<String>,
}

impl Default for FreudianFramework {
    fn default() -> Self {
        Self {
            id: Id { drive_strength: 0.5, frustration: 0.0, active_drives: Vec::new() },
            ego: Ego { strength: 0.7, anxiety: 0.2, strategy: EgoStrategy::Rational },
            superego: SuperEgo { strength: 0.6, guilt: 0.1, pride: 0.3 },
            balance: PsychicBalance { internal_conflict: 0.1, psychic_health: 0.8 },
            active_defenses: Vec::new(),
        }
    }
}
