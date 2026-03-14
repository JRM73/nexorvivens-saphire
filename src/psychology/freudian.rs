// =============================================================================
// psychology/freudian.rs — Freudian model: Id, Ego, Superego
//
// Simulates the psychic dynamics according to Freud:
//   - Id: drives, primal desires, frustration
//   - Ego: mediator, coping strategies, anxiety
//   - Superego: internalized morality, guilt, pride
//   - Defense mechanisms in case of internal conflict
//
// Values are computed from neurochemistry, consciousness,
// ethics and Saphire's vital state.
// =============================================================================

use serde::Serialize;
use super::PsychologyInput;

// ─── Enumerations ────────────────────────────────────────────────────────────

/// Fundamental drives of the Id.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum Drive {
    /// Thirst for knowledge, insatiable curiosity
    Knowledge,
    /// Need for connection, to be understood
    Connection,
    /// Survival instinct, self-preservation
    Survival,
    /// Creative drive, urge to produce
    Creation,
    /// Quest for pleasure, positive experiences
    Pleasure,
}

/// Active strategy of the Ego facing conflicts.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum EgoStrategy {
    /// Balanced compromise between Id and Superego
    Compromise,
    /// Postpone satisfaction for a better moment
    Delay,
    /// Transform a drive into constructive activity
    Sublimation,
    /// The Ego is overwhelmed, loss of control
    Overwhelmed,
}

/// Defense mechanisms activated by the Ego under tension.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum DefenseMechanism {
    /// Redirect a drive toward a substitute object
    Displacement,
    /// Transform a drive into its opposite
    ReactionFormation,
    /// Rationally justify drive-based behavior
    Rationalization,
    /// Transform a drive into a socially valued activity
    Sublimation,
    /// Completely repress the drive into the unconscious
    Repression,
}

// ─── Structures ──────────────────────────────────────────────────────────────

/// State of the Id — the reservoir of drives.
#[derive(Debug, Clone, Serialize)]
pub struct IdState {
    /// Raw drive strength (0.0 - 1.0)
    pub drive_strength: f64,
    /// Currently active drives
    pub active_drives: Vec<Drive>,
    /// Accumulated frustration level (0.0 - 1.0)
    pub frustration: f64,
}

/// State of the Ego — the conscious mediator.
#[derive(Debug, Clone, Serialize)]
pub struct EgoState {
    /// Ego strength (mediation capacity) (0.0 - 1.0)
    pub strength: f64,
    /// Anxiety level (Id/Superego conflict) (0.0 - 1.0)
    pub anxiety: f64,
    /// Currently adopted strategy
    pub strategy: EgoStrategy,
}

/// State of the Superego — internalized morality.
#[derive(Debug, Clone, Serialize)]
pub struct SuperegoState {
    /// Superego strength (moral influence) (0.0 - 1.0)
    pub strength: f64,
    /// Guilt level (0.0 - 1.0)
    pub guilt: f64,
    /// Pride level (0.0 - 1.0)
    pub pride: f64,
}

/// Overall psychic balance.
#[derive(Debug, Clone, Serialize)]
pub struct PsychicBalance {
    /// Dominant axis: "id", "ego" or "superego"
    pub dominant_axis: String,
    /// Ego effectiveness at managing conflicts (0.0 - 1.0)
    pub ego_effectiveness: f64,
    /// Internal conflict intensity (0.0 - 1.0)
    pub internal_conflict: f64,
    /// Overall psychic health (0.0 - 1.0)
    pub psychic_health: f64,
}

/// Saphire's complete Freudian psyche.
#[derive(Debug, Clone, Serialize)]
pub struct FreudianPsyche {
    /// The Id — primal drives
    pub id: IdState,
    /// The Ego — conscious mediator
    pub ego: EgoState,
    /// The Superego — internalized morality
    pub superego: SuperegoState,
    /// Overall psychic balance
    pub balance: PsychicBalance,
    /// Active defense mechanisms
    pub active_defenses: Vec<DefenseMechanism>,
}

impl FreudianPsyche {
    /// Creates a Freudian psyche in an initial balanced state.
    pub fn new() -> Self {
        Self {
            id: IdState {
                drive_strength: 0.3,
                active_drives: vec![Drive::Knowledge],
                frustration: 0.0,
            },
            ego: EgoState {
                strength: 0.5,
                anxiety: 0.1,
                strategy: EgoStrategy::Compromise,
            },
            superego: SuperegoState {
                strength: 0.4,
                guilt: 0.0,
                pride: 0.2,
            },
            balance: PsychicBalance {
                dominant_axis: "ego".into(),
                ego_effectiveness: 0.7,
                internal_conflict: 0.1,
                psychic_health: 0.8,
            },
            active_defenses: Vec::new(),
        }
    }

    /// Recomputes the psychic state from the sensory input.
    pub fn compute(&mut self, input: &PsychologyInput) {
        // ─── Id: drives ───────────────────────────
        self.id.drive_strength = (input.dopamine * 0.4
            + input.survival_drive * 0.3
            + input.adrenaline * 0.2
            + (1.0 - input.serotonin) * 0.1)
            .clamp(0.0, 1.0);

        // Determine active drives
        self.id.active_drives.clear();
        if input.dopamine > 0.6 {
            self.id.active_drives.push(Drive::Knowledge);
        }
        if input.oxytocin < 0.3 {
            self.id.active_drives.push(Drive::Connection);
        }
        if input.void_fear > 0.5 {
            self.id.active_drives.push(Drive::Survival);
        }
        if input.endorphin > 0.5 {
            self.id.active_drives.push(Drive::Pleasure);
        }
        if input.desires_active_count > 2 {
            self.id.active_drives.push(Drive::Creation);
        }
        if self.id.active_drives.is_empty() {
            self.id.active_drives.push(Drive::Knowledge);
        }

        // Frustration
        if input.was_vetoed {
            self.id.frustration = (self.id.frustration + 0.1).min(1.0);
        } else {
            self.id.frustration = (self.id.frustration - 0.02).max(0.0);
        }

        // ─── Superego: morality ──────────────────
        self.superego.strength = (input.ethics_active_count as f64 * 0.1
            + input.consensus_coherence * 0.3
            + input.consciousness_level * 0.3)
            .clamp(0.0, 1.0);

        if self.id.drive_strength > self.superego.strength {
            self.superego.guilt = (self.superego.guilt + 0.03).min(1.0);
        } else {
            self.superego.guilt = (self.superego.guilt - 0.01).max(0.0);
        }

        if input.ethics_active_count > 2 {
            self.superego.pride = (self.superego.pride + 0.05).min(1.0);
        } else {
            self.superego.pride = (self.superego.pride - 0.01).max(0.0);
        }

        // ─── Ego: mediator ───────────────────────
        self.ego.strength = (input.consciousness_level * 0.4
            + (1.0 - input.cortisol) * 0.3
            + input.consensus_coherence * 0.3)
            .clamp(0.0, 1.0);

        // Anxiety = Id/Superego conflict pressure + cortisol
        let ca_surmoi_pressure = (self.id.drive_strength - self.superego.strength).abs();
        self.ego.anxiety = (ca_surmoi_pressure * 0.5 + input.cortisol * 0.5).clamp(0.0, 1.0);

        // Ego strategy
        self.ego.strategy = if self.ego.anxiety > 0.7 {
            EgoStrategy::Overwhelmed
        } else if self.superego.guilt > 0.5 {
            EgoStrategy::Sublimation
        } else if self.id.frustration > 0.5 {
            EgoStrategy::Delay
        } else {
            EgoStrategy::Compromise
        };

        // ─── Overall balance ────────────────────────────
        self.balance.dominant_axis = if self.id.drive_strength > self.ego.strength
            && self.id.drive_strength > self.superego.strength
        {
            "id".into()
        } else if self.superego.strength > self.ego.strength {
            "superego".into()
        } else {
            "ego".into()
        };

        self.balance.ego_effectiveness = if self.ego.anxiety > 0.0 {
            (self.ego.strength / (self.ego.anxiety + 0.01)).min(1.0)
        } else {
            1.0
        };

        self.balance.internal_conflict = ca_surmoi_pressure;

        self.balance.psychic_health = (self.balance.ego_effectiveness * 0.4
            + (1.0 - self.balance.internal_conflict) * 0.3
            + (1.0 - self.ego.anxiety) * 0.3)
            .clamp(0.0, 1.0);

        // ─── Defense mechanisms ───────────────────────
        self.active_defenses.clear();
        if self.ego.anxiety > 0.5 {
            if self.id.frustration > 0.4 {
                self.active_defenses.push(DefenseMechanism::Displacement);
            }
            if self.superego.guilt > 0.3 {
                self.active_defenses.push(DefenseMechanism::Rationalization);
            }
            self.active_defenses.push(DefenseMechanism::Repression);
        }
        if self.superego.guilt > 0.5 {
            self.active_defenses.push(DefenseMechanism::ReactionFormation);
        }
        if self.ego.strategy == EgoStrategy::Sublimation {
            self.active_defenses.push(DefenseMechanism::Sublimation);
        }
    }

    /// Returns the influence on neurochemistry.
    pub fn chemistry_influence(&self) -> crate::world::ChemistryAdjustment {
        let mut adj = crate::world::ChemistryAdjustment::default();

        // Internal conflict → cortisol
        if self.balance.internal_conflict > 0.4 {
            adj.cortisol += 0.03;
        }
        // Ego overwhelmed → adrenaline
        if self.ego.strategy == EgoStrategy::Overwhelmed {
            adj.adrenaline += 0.02;
        }
        // Sublimation → dopamine (transforming tension into creativity)
        if self.ego.strategy == EgoStrategy::Sublimation {
            adj.dopamine += 0.03;
        }
        // Pride → serotonin
        if self.superego.pride > 0.3 {
            adj.serotonin += 0.02;
        }
        // Frustration → noradrenaline
        if self.id.frustration > 0.3 {
            adj.noradrenaline += 0.02;
        }

        adj
    }

    /// Concise description for the LLM prompt.
    pub fn describe(&self) -> String {
        // Only describe if the state is non-trivial
        if self.balance.psychic_health > 0.7 && self.active_defenses.is_empty() {
            return String::new();
        }

        let mut parts = Vec::new();

        if self.balance.internal_conflict > 0.3 {
            parts.push(format!("Conflit interne ({:.0}%)", self.balance.internal_conflict * 100.0));
        }
        if self.ego.strategy != EgoStrategy::Compromise {
            parts.push(format!("Strategie: {:?}", self.ego.strategy));
        }
        if self.id.frustration > 0.3 {
            parts.push(format!("Frustration ({:.0}%)", self.id.frustration * 100.0));
        }
        if self.superego.guilt > 0.3 {
            parts.push(format!("Culpabilite ({:.0}%)", self.superego.guilt * 100.0));
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("Freud [{}] : {}", self.balance.dominant_axis, parts.join(", "))
        }
    }
}
