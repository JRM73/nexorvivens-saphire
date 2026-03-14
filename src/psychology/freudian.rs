// =============================================================================
// psychology/freudian.rs — Modele freudien : Ca, Moi, Surmoi
//
// Simule la dynamique psychique selon Freud :
//   - Ca (Id) : pulsions, desirs primaires, frustration
//   - Moi (Ego) : mediateur, strategies d'adaptation, anxiete
//   - Surmoi (Superego) : morale internalisee, culpabilite, fierte
//   - Mecanismes de defense en cas de conflit interne
//
// Les valeurs sont calculees a partir de la neurochimie, de la conscience,
// de l'ethique et de l'etat vital de Saphire.
// =============================================================================

use serde::Serialize;
use super::PsychologyInput;

// ─── Enumerations ────────────────────────────────────────────────────────────

/// Pulsions fondamentales du Ca (Id).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum Drive {
    /// Soif de savoir, curiosite insatiable
    Knowledge,
    /// Besoin de lien, d'etre comprise
    Connection,
    /// Instinct de survie, preservation de soi
    Survival,
    /// Pulsion creatrice, envie de produire
    Creation,
    /// Quete de plaisir, experiences positives
    Pleasure,
}

/// Strategie active du Moi (Ego) face aux conflits.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum EgoStrategy {
    /// Compromis equilibre entre Ca et Surmoi
    Compromise,
    /// Reporter la satisfaction pour un meilleur moment
    Delay,
    /// Transformer une pulsion en activite constructive
    Sublimation,
    /// Le Moi est submerge, perte de controle
    Overwhelmed,
}

/// Mecanismes de defense actives par le Moi en cas de tension.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum DefenseMechanism {
    /// Rediriger une pulsion vers un objet substitutif
    Displacement,
    /// Transformer une pulsion en son contraire
    ReactionFormation,
    /// Justifier rationnellement un comportement pulsionnel
    Rationalization,
    /// Transformer une pulsion en activite socialement valorisee
    Sublimation,
    /// Refouler completement la pulsion dans l'inconscient
    Repression,
}

// ─── Structures ──────────────────────────────────────────────────────────────

/// Etat du Ca (Id) — le reservoir des pulsions.
#[derive(Debug, Clone, Serialize)]
pub struct IdState {
    /// Force brute des pulsions (0.0 - 1.0)
    pub drive_strength: f64,
    /// Pulsions actuellement actives
    pub active_drives: Vec<Drive>,
    /// Niveau de frustration accumule (0.0 - 1.0)
    pub frustration: f64,
}

/// Etat du Moi (Ego) — le mediateur conscient.
#[derive(Debug, Clone, Serialize)]
pub struct EgoState {
    /// Force du Moi (capacite de mediation) (0.0 - 1.0)
    pub strength: f64,
    /// Niveau d'anxiete (conflit Ca/Surmoi) (0.0 - 1.0)
    pub anxiety: f64,
    /// Strategie actuellement adoptee
    pub strategy: EgoStrategy,
}

/// Etat du Surmoi (Superego) — la morale internalisee.
#[derive(Debug, Clone, Serialize)]
pub struct SuperegoState {
    /// Force du Surmoi (influence morale) (0.0 - 1.0)
    pub strength: f64,
    /// Niveau de culpabilite (0.0 - 1.0)
    pub guilt: f64,
    /// Niveau de fierte (0.0 - 1.0)
    pub pride: f64,
}

/// Equilibre psychique global.
#[derive(Debug, Clone, Serialize)]
pub struct PsychicBalance {
    /// Axe dominant : "id", "ego" ou "superego"
    pub dominant_axis: String,
    /// Efficacite du Moi a gerer les conflits (0.0 - 1.0)
    pub ego_effectiveness: f64,
    /// Intensite du conflit interne (0.0 - 1.0)
    pub internal_conflict: f64,
    /// Sante psychique globale (0.0 - 1.0)
    pub psychic_health: f64,
}

/// Psyche freudienne complete de Saphire.
#[derive(Debug, Clone, Serialize)]
pub struct FreudianPsyche {
    /// Le Ca (Id) — pulsions primaires
    pub id: IdState,
    /// Le Moi (Ego) — mediateur conscient
    pub ego: EgoState,
    /// Le Surmoi (Superego) — morale internalisee
    pub superego: SuperegoState,
    /// Equilibre psychique global
    pub balance: PsychicBalance,
    /// Mecanismes de defense actifs
    pub active_defenses: Vec<DefenseMechanism>,
}

impl FreudianPsyche {
    /// Cree une psyche freudienne dans un etat initial equilibre.
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

    /// Recalcule l'etat psychique a partir de l'entree sensorielle.
    pub fn compute(&mut self, input: &PsychologyInput) {
        // ─── Ca (Id) : pulsions ───────────────────────────
        self.id.drive_strength = (input.dopamine * 0.4
            + input.survival_drive * 0.3
            + input.adrenaline * 0.2
            + (1.0 - input.serotonin) * 0.1)
            .clamp(0.0, 1.0);

        // Determiner les pulsions actives
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

        // ─── Surmoi (Superego) : morale ──────────────────
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

        // ─── Moi (Ego) : mediateur ───────────────────────
        self.ego.strength = (input.consciousness_level * 0.4
            + (1.0 - input.cortisol) * 0.3
            + input.consensus_coherence * 0.3)
            .clamp(0.0, 1.0);

        // Anxiete = pression du conflit Ca/Surmoi + cortisol
        let ca_surmoi_pressure = (self.id.drive_strength - self.superego.strength).abs();
        self.ego.anxiety = (ca_surmoi_pressure * 0.5 + input.cortisol * 0.5).clamp(0.0, 1.0);

        // Strategie du Moi
        self.ego.strategy = if self.ego.anxiety > 0.7 {
            EgoStrategy::Overwhelmed
        } else if self.superego.guilt > 0.5 {
            EgoStrategy::Sublimation
        } else if self.id.frustration > 0.5 {
            EgoStrategy::Delay
        } else {
            EgoStrategy::Compromise
        };

        // ─── Equilibre global ────────────────────────────
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

        // ─── Mecanismes de defense ───────────────────────
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

    /// Retourne l'influence sur la neurochimie.
    pub fn chemistry_influence(&self) -> crate::world::ChemistryAdjustment {
        let mut adj = crate::world::ChemistryAdjustment::default();

        // Conflit interne → cortisol
        if self.balance.internal_conflict > 0.4 {
            adj.cortisol += 0.03;
        }
        // Moi submerge → adrenaline
        if self.ego.strategy == EgoStrategy::Overwhelmed {
            adj.adrenaline += 0.02;
        }
        // Sublimation → dopamine (transformer la tension en creativite)
        if self.ego.strategy == EgoStrategy::Sublimation {
            adj.dopamine += 0.03;
        }
        // Fierte → serotonine
        if self.superego.pride > 0.3 {
            adj.serotonin += 0.02;
        }
        // Frustration → noradrenaline
        if self.id.frustration > 0.3 {
            adj.noradrenaline += 0.02;
        }

        adj
    }

    /// Description concise pour le prompt LLM.
    pub fn describe(&self) -> String {
        // Ne decrire que si l'etat est non-trivial
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
