// =============================================================================
// relationships/mod.rs — Affective bonds and relationship network
// =============================================================================
//
// This module models Saphire's relationships with the people she meets.
// Each bond has a type (friend, mentor, rival, etc.), a strength,
// a trust level, and an interaction history.
//
// The relationship network influences chemistry: isolation increases cortisol,
// strong bonds increase oxytocin.
// =============================================================================

pub mod family;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Saphire's attachment style (inspired by Bowlby's theory).
/// Determines how she forms and maintains her affective bonds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttachmentStyle {
    /// Secure attachment: easy trust, stable bonds
    Secure,
    /// Anxious attachment: fear of abandonment, need for reassurance
    Anxious,
    /// Avoidant attachment: emotional distance, self-sufficiency
    Avoidant,
    /// Disorganized attachment: oscillation between proximity and withdrawal
    Disorganized,
}

impl Default for AttachmentStyle {
    fn default() -> Self { Self::Secure }
}

/// Type of affective bond between Saphire and a person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BondType {
    Friend,
    Mentor,
    Rival,
    Confidant,
    Family,
    Partner,
}

impl BondType {
    pub fn as_str(&self) -> &str {
        match self {
            BondType::Friend => "ami",
            BondType::Mentor => "mentor",
            BondType::Rival => "rival",
            BondType::Confidant => "confident",
            BondType::Family => "famille",
            BondType::Partner => "partenaire",
        }
    }
}

/// An individual affective bond between Saphire and a person.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectiveBond {
    /// Person identifier (name or pseudonym)
    pub person_id: String,
    /// Bond type
    pub bond_type: BondType,
    /// Bond strength (0.0 = stranger, 1.0 = deep bond)
    pub strength: f64,
    /// Trust level (0.0 = distrust, 1.0 = total trust)
    pub trust: f64,
    /// Number of shared memories
    pub shared_memories: u32,
    /// Conflict level (0.0 = harmony, 1.0 = total conflict)
    pub conflict_level: f64,
    /// Last interaction
    pub last_interaction: DateTime<Utc>,
}

impl AffectiveBond {
    /// Creates a new bond with initial values.
    pub fn new(person_id: &str, bond_type: BondType) -> Self {
        Self {
            person_id: person_id.to_string(),
            bond_type,
            strength: 0.2,
            trust: 0.3,
            shared_memories: 0,
            conflict_level: 0.0,
            last_interaction: Utc::now(),
        }
    }

    /// Natural decay of the bond over time.
    pub fn decay(&mut self, dt_hours: f64) {
        // Strength decays slowly without interaction
        let decay_rate = 0.001 * dt_hours / 24.0;
        self.strength = (self.strength - decay_rate).max(0.0);
        // Conflict dissipates gradually
        self.conflict_level = (self.conflict_level - 0.005 * dt_hours / 24.0).max(0.0);
    }
}

/// Saphire's complete relationship network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipNetwork {
    /// Global attachment style
    pub attachment_style: AttachmentStyle,
    /// All active affective bonds
    pub bonds: Vec<AffectiveBond>,
}

impl Default for RelationshipNetwork {
    fn default() -> Self {
        Self {
            attachment_style: AttachmentStyle::Secure,
            bonds: Vec::new(),
        }
    }
}

impl RelationshipNetwork {
    /// Creates a new network with the given attachment style.
    pub fn new(style: AttachmentStyle) -> Self {
        Self {
            attachment_style: style,
            bonds: Vec::new(),
        }
    }

    /// Periodic tick: natural decay of all bonds.
    /// `oxytocin`: current oxytocin level (reinforces bond maintenance).
    pub fn tick(&mut self, oxytocin: f64) {
        let maintenance_boost = oxytocin * 0.5; // Oxytocin slows decay
        for bond in &mut self.bonds {
            let dt = (Utc::now() - bond.last_interaction).num_hours() as f64;
            if dt > 0.0 {
                let effective_decay = (1.0 - maintenance_boost).max(0.2);
                bond.strength = (bond.strength - 0.001 * effective_decay).max(0.0);
                bond.conflict_level = (bond.conflict_level - 0.005).max(0.0);
            }
        }
        // Remove dead bonds (strength dropped to zero)
        self.bonds.retain(|b| b.strength > 0.01);
    }

    /// Observes an interaction with a person and updates the bond.
    pub fn observe_interaction(&mut self, person_id: &str, sentiment: f64, _emotion: &str) {
        let bond = if let Some(b) = self.bonds.iter_mut().find(|b| b.person_id == person_id) {
            b
        } else {
            // New bond — initial type based on identity
            let initial_type = Self::initial_bond_type(person_id);
            self.bonds.push(AffectiveBond::new(person_id, initial_type));
            self.bonds.last_mut().unwrap()
        };

        bond.last_interaction = Utc::now();
        bond.shared_memories += 1;

        // Positive sentiment strengthens the bond, negative increases conflict
        if sentiment > 0.2 {
            bond.strength = (bond.strength + sentiment * 0.05).min(1.0);
            bond.trust = (bond.trust + sentiment * 0.02).min(1.0);
            bond.conflict_level = (bond.conflict_level - 0.02).max(0.0);
        } else if sentiment < -0.2 {
            bond.conflict_level = (bond.conflict_level + sentiment.abs() * 0.05).min(1.0);
            bond.trust = (bond.trust - sentiment.abs() * 0.03).max(0.0);
        }

        // Attachment style modifies sensitivity
        match self.attachment_style {
            AttachmentStyle::Anxious => {
                if sentiment < 0.0 {
                    bond.conflict_level = (bond.conflict_level + 0.02).min(1.0);
                }
            }
            AttachmentStyle::Avoidant => {
                bond.strength *= 0.95;
            }
            _ => {}
        }

        // Bond type promotion based on history
        Self::maybe_promote_bond(bond);
    }

    /// Determines the initial bond type based on the person's identity.
    fn initial_bond_type(person_id: &str) -> BondType {
        let lower = person_id.to_lowercase();
        // JRM is the creator/father of Saphire
        if lower == "jrm" || lower == "jeremy" || lower == "jérémy" {
            BondType::Family
        } else {
            BondType::Friend
        }
    }

    /// Bond type promotion based on trust and history.
    fn maybe_promote_bond(bond: &mut AffectiveBond) {
        // Do not demote Family (foundational bond)
        if bond.bond_type == BondType::Family {
            return;
        }
        // High trust + many interactions -> Confidant
        if bond.trust > 0.6 && bond.shared_memories > 50 && bond.bond_type == BondType::Friend {
            bond.bond_type = BondType::Confidant;
        }
        // High conflict + interactions -> Rival
        if bond.conflict_level > 0.4 && bond.shared_memories > 20 && bond.bond_type == BondType::Friend {
            bond.bond_type = BondType::Rival;
        }
    }

    /// Chemical influence of the relationship network.
    /// Returns a ChemistryAdjustment.
    pub fn chemistry_influence(&self) -> crate::world::ChemistryAdjustment {
        let total_bonds = self.bonds.len();
        let strong_bonds = self.bonds.iter().filter(|b| b.strength > 0.5).count();
        let total_conflict: f64 = self.bonds.iter().map(|b| b.conflict_level).sum();

        let mut adj = crate::world::ChemistryAdjustment::default();

        // Isolation: no strong bonds -> cortisol rises
        if total_bonds == 0 || strong_bonds == 0 {
            adj.cortisol = 0.01;
            adj.oxytocin = -0.005;
        } else {
            // Strong bonds -> oxytocin and serotonin
            let bond_boost = (strong_bonds as f64 * 0.005).min(0.02);
            adj.oxytocin = bond_boost;
            adj.serotonin = bond_boost * 0.5;
        }

        // Active conflicts -> stress
        if total_conflict > 0.5 {
            adj.cortisol += (total_conflict * 0.01).min(0.03);
            adj.adrenaline += (total_conflict * 0.005).min(0.01);
        }

        adj
    }

    /// Serializes to JSON for the API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "attachment_style": format!("{:?}", self.attachment_style),
            "bond_count": self.bonds.len(),
            "strong_bonds": self.bonds.iter().filter(|b| b.strength > 0.5).count(),
            "bonds": self.bonds.iter().map(|b| serde_json::json!({
                "person_id": b.person_id,
                "bond_type": b.bond_type.as_str(),
                "strength": b.strength,
                "trust": b.trust,
                "shared_memories": b.shared_memories,
                "conflict_level": b.conflict_level,
            })).collect::<Vec<_>>(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bond() {
        let bond = AffectiveBond::new("alice", BondType::Friend);
        assert_eq!(bond.person_id, "alice");
        assert!(bond.strength > 0.0);
        assert!(bond.trust > 0.0);
    }

    #[test]
    fn test_observe_positive_interaction() {
        let mut network = RelationshipNetwork::default();
        network.observe_interaction("bob", 0.8, "Joie");
        assert_eq!(network.bonds.len(), 1);
        assert!(network.bonds[0].strength > 0.2);
    }

    #[test]
    fn test_chemistry_influence_isolated() {
        let network = RelationshipNetwork::default();
        let adj = network.chemistry_influence();
        assert!(adj.cortisol > 0.0, "Isolation should increase cortisol");
    }

    #[test]
    fn test_chemistry_influence_with_bonds() {
        let mut network = RelationshipNetwork::default();
        // Create a strong bond
        network.bonds.push(AffectiveBond {
            person_id: "alice".into(),
            bond_type: BondType::Friend,
            strength: 0.8,
            trust: 0.7,
            shared_memories: 10,
            conflict_level: 0.0,
            last_interaction: Utc::now(),
        });
        let adj = network.chemistry_influence();
        assert!(adj.oxytocin > 0.0, "Strong bonds should increase oxytocin");
    }
}
