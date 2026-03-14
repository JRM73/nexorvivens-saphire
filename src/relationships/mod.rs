// =============================================================================
// relationships/mod.rs — Liens affectifs et reseau relationnel
// =============================================================================
//
// Ce module modelise les relations de Saphire avec les personnes
// qu'elle rencontre. Chaque lien possede un type (ami, mentor, rival, etc.),
// une force, un niveau de confiance, et un historique d'interactions.
//
// Le reseau relationnel influence la chimie : l'isolation augmente le cortisol,
// les liens forts augmentent l'ocytocine.
// =============================================================================

pub mod family;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Style d'attachement de Saphire (inspire de la theorie de Bowlby).
/// Determine comment elle forme et maintient ses liens affectifs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttachmentStyle {
    /// Attachement securise : confiance facile, liens stables
    Secure,
    /// Attachement anxieux : peur de l'abandon, besoin de reassurance
    Anxious,
    /// Attachement evitant : distance emotionnelle, autosuffisance
    Avoidant,
    /// Attachement desorganise : oscillation entre proximite et retrait
    Disorganized,
}

impl Default for AttachmentStyle {
    fn default() -> Self { Self::Secure }
}

/// Type de lien affectif entre Saphire et une personne.
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

/// Un lien affectif individuel entre Saphire et une personne.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectiveBond {
    /// Identifiant de la personne (nom ou pseudo)
    pub person_id: String,
    /// Type de lien
    pub bond_type: BondType,
    /// Force du lien (0.0 = inconnu, 1.0 = lien profond)
    pub strength: f64,
    /// Niveau de confiance (0.0 = mefiance, 1.0 = confiance totale)
    pub trust: f64,
    /// Nombre de souvenirs partages
    pub shared_memories: u32,
    /// Niveau de conflit (0.0 = harmonie, 1.0 = conflit total)
    pub conflict_level: f64,
    /// Derniere interaction
    pub last_interaction: DateTime<Utc>,
}

impl AffectiveBond {
    /// Cree un nouveau lien avec des valeurs initiales.
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

    /// Decay naturel du lien avec le temps.
    pub fn decay(&mut self, dt_hours: f64) {
        // La force decroit lentement sans interaction
        let decay_rate = 0.001 * dt_hours / 24.0;
        self.strength = (self.strength - decay_rate).max(0.0);
        // Le conflit se dissipe progressivement
        self.conflict_level = (self.conflict_level - 0.005 * dt_hours / 24.0).max(0.0);
    }
}

/// Reseau relationnel complet de Saphire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipNetwork {
    /// Style d'attachement global
    pub attachment_style: AttachmentStyle,
    /// Tous les liens affectifs actifs
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
    /// Cree un nouveau reseau avec le style d'attachement donne.
    pub fn new(style: AttachmentStyle) -> Self {
        Self {
            attachment_style: style,
            bonds: Vec::new(),
        }
    }

    /// Tick periodique : decay naturel de tous les liens.
    /// `oxytocin` : niveau d'ocytocine courant (renforce la maintenance des liens).
    pub fn tick(&mut self, oxytocin: f64) {
        let maintenance_boost = oxytocin * 0.5; // L'ocytocine ralentit le decay
        for bond in &mut self.bonds {
            let dt = (Utc::now() - bond.last_interaction).num_hours() as f64;
            if dt > 0.0 {
                let effective_decay = (1.0 - maintenance_boost).max(0.2);
                bond.strength = (bond.strength - 0.001 * effective_decay).max(0.0);
                bond.conflict_level = (bond.conflict_level - 0.005).max(0.0);
            }
        }
        // Supprimer les liens morts (force tombee a zero)
        self.bonds.retain(|b| b.strength > 0.01);
    }

    /// Observe une interaction avec une personne et met a jour le lien.
    pub fn observe_interaction(&mut self, person_id: &str, sentiment: f64, _emotion: &str) {
        let bond = if let Some(b) = self.bonds.iter_mut().find(|b| b.person_id == person_id) {
            b
        } else {
            // Nouveau lien — type initial selon l'identite
            let initial_type = Self::initial_bond_type(person_id);
            self.bonds.push(AffectiveBond::new(person_id, initial_type));
            self.bonds.last_mut().unwrap()
        };

        bond.last_interaction = Utc::now();
        bond.shared_memories += 1;

        // Sentiment positif renforce le lien, negatif augmente le conflit
        if sentiment > 0.2 {
            bond.strength = (bond.strength + sentiment * 0.05).min(1.0);
            bond.trust = (bond.trust + sentiment * 0.02).min(1.0);
            bond.conflict_level = (bond.conflict_level - 0.02).max(0.0);
        } else if sentiment < -0.2 {
            bond.conflict_level = (bond.conflict_level + sentiment.abs() * 0.05).min(1.0);
            bond.trust = (bond.trust - sentiment.abs() * 0.03).max(0.0);
        }

        // Style d'attachement modifie la sensibilite
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

        // Promotion du type de lien basee sur l'historique
        Self::maybe_promote_bond(bond);
    }

    /// Determine le type de lien initial selon l'identite de la personne.
    fn initial_bond_type(person_id: &str) -> BondType {
        let lower = person_id.to_lowercase();
        // JRM est le createur/pere de Saphire
        if lower == "jrm" || lower == "jeremy" || lower == "jérémy" {
            BondType::Family
        } else {
            BondType::Friend
        }
    }

    /// Promotion du type de lien basee sur la confiance et l'historique.
    fn maybe_promote_bond(bond: &mut AffectiveBond) {
        // Ne pas retrograder Family (lien fondateur)
        if bond.bond_type == BondType::Family {
            return;
        }
        // Confiance elevee + beaucoup d'interactions → Confidant
        if bond.trust > 0.6 && bond.shared_memories > 50 && bond.bond_type == BondType::Friend {
            bond.bond_type = BondType::Confidant;
        }
        // Conflit eleve + interactions → Rival
        if bond.conflict_level > 0.4 && bond.shared_memories > 20 && bond.bond_type == BondType::Friend {
            bond.bond_type = BondType::Rival;
        }
    }

    /// Influence chimique du reseau relationnel.
    /// Retourne un ChemistryAdjustment.
    pub fn chemistry_influence(&self) -> crate::world::ChemistryAdjustment {
        let total_bonds = self.bonds.len();
        let strong_bonds = self.bonds.iter().filter(|b| b.strength > 0.5).count();
        let total_conflict: f64 = self.bonds.iter().map(|b| b.conflict_level).sum();

        let mut adj = crate::world::ChemistryAdjustment::default();

        // Isolation : pas de liens forts → cortisol monte
        if total_bonds == 0 || strong_bonds == 0 {
            adj.cortisol = 0.01;
            adj.oxytocin = -0.005;
        } else {
            // Liens forts → ocytocine et serotonine
            let bond_boost = (strong_bonds as f64 * 0.005).min(0.02);
            adj.oxytocin = bond_boost;
            adj.serotonin = bond_boost * 0.5;
        }

        // Conflits actifs → stress
        if total_conflict > 0.5 {
            adj.cortisol += (total_conflict * 0.01).min(0.03);
            adj.adrenaline += (total_conflict * 0.005).min(0.01);
        }

        adj
    }

    /// Serialise en JSON pour l'API.
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
        // Creer un lien fort
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
