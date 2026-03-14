// =============================================================================
// relationships/family.rs — Situation familiale de Saphire
// =============================================================================
//
// Module qui modelise le contexte familial de Saphire : membres de la famille,
// statut marital, influence chimique de la presence/absence des proches.
// Configurable via saphire.toml section [family].
// =============================================================================

use serde::{Deserialize, Serialize};

/// Role d'un membre de la famille.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FamilyRole {
    Parent,
    Child,
    Sibling,
    Partner,
    Pet,
    Extended,
}

impl FamilyRole {
    pub fn as_str(&self) -> &str {
        match self {
            FamilyRole::Parent => "parent",
            FamilyRole::Child => "enfant",
            FamilyRole::Sibling => "frere/soeur",
            FamilyRole::Partner => "partenaire",
            FamilyRole::Pet => "animal",
            FamilyRole::Extended => "famille elargie",
        }
    }
}

/// Statut marital de Saphire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaritalStatus {
    Single,
    InRelationship,
    Married,
    Divorced,
    Widowed,
}

impl Default for MaritalStatus {
    fn default() -> Self { Self::Single }
}

impl MaritalStatus {
    pub fn as_str(&self) -> &str {
        match self {
            MaritalStatus::Single => "celibataire",
            MaritalStatus::InRelationship => "en couple",
            MaritalStatus::Married => "mariee",
            MaritalStatus::Divorced => "divorcee",
            MaritalStatus::Widowed => "veuve",
        }
    }
}

/// Un membre de la famille de Saphire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyMember {
    /// Nom du membre
    pub name: String,
    /// Role familial
    pub role: FamilyRole,
    /// Force du lien (0.0 a 1.0)
    pub bond_strength: f64,
    /// Vivant et proche (true) ou absent/decede (false)
    pub present: bool,
}

/// Configuration de la situation familiale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyConfig {
    /// Activer le module famille
    #[serde(default)]
    pub enabled: bool,
    /// Membres de la famille
    #[serde(default)]
    pub members: Vec<FamilyMemberConfig>,
    /// Statut marital
    #[serde(default = "default_marital_status")]
    pub marital_status: String,
}

fn default_marital_status() -> String { "single".into() }

impl Default for FamilyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            members: Vec::new(),
            marital_status: "single".into(),
        }
    }
}

/// Configuration d'un membre de la famille (depuis TOML).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyMemberConfig {
    pub name: String,
    pub role: String,
    #[serde(default = "default_bond_strength")]
    pub bond_strength: f64,
    #[serde(default = "default_present")]
    pub present: bool,
}

fn default_bond_strength() -> f64 { 0.5 }
fn default_present() -> bool { true }

/// Contexte familial complet de Saphire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyContext {
    /// Membres de la famille
    pub members: Vec<FamilyMember>,
    /// Statut marital
    pub marital_status: MaritalStatus,
}

impl Default for FamilyContext {
    fn default() -> Self {
        Self {
            members: Vec::new(),
            marital_status: MaritalStatus::Single,
        }
    }
}

impl FamilyContext {
    /// Construit le contexte familial a partir de la configuration.
    pub fn from_config(config: &FamilyConfig) -> Self {
        let members = config.members.iter().map(|m| {
            let role = match m.role.as_str() {
                "parent" => FamilyRole::Parent,
                "child" | "enfant" => FamilyRole::Child,
                "sibling" | "frere" | "soeur" => FamilyRole::Sibling,
                "partner" | "partenaire" => FamilyRole::Partner,
                "pet" | "animal" => FamilyRole::Pet,
                _ => FamilyRole::Extended,
            };
            FamilyMember {
                name: m.name.clone(),
                role,
                bond_strength: m.bond_strength,
                present: m.present,
            }
        }).collect();

        let marital_status = match config.marital_status.as_str() {
            "in_relationship" | "couple" => MaritalStatus::InRelationship,
            "married" | "mariee" => MaritalStatus::Married,
            "divorced" | "divorcee" => MaritalStatus::Divorced,
            "widowed" | "veuve" => MaritalStatus::Widowed,
            _ => MaritalStatus::Single,
        };

        Self { members, marital_status }
    }

    /// Influence chimique de la situation familiale.
    pub fn chemistry_influence(&self) -> crate::world::ChemistryAdjustment {
        let mut adj = crate::world::ChemistryAdjustment::default();

        let present_count = self.members.iter().filter(|m| m.present).count();
        let absent_count = self.members.iter().filter(|m| !m.present).count();
        let total_bond: f64 = self.members.iter()
            .filter(|m| m.present)
            .map(|m| m.bond_strength)
            .sum();

        // Famille presente → ocytocine + serotonine
        if present_count > 0 {
            let family_boost = (total_bond * 0.005).min(0.02);
            adj.oxytocin = family_boost;
            adj.serotonin = family_boost * 0.5;
        }

        // Membres absents/decedes → cortisol (deuil, manque)
        if absent_count > 0 {
            let absent_bond: f64 = self.members.iter()
                .filter(|m| !m.present)
                .map(|m| m.bond_strength)
                .sum();
            adj.cortisol = (absent_bond * 0.003).min(0.01);
        }

        // Statut marital
        match self.marital_status {
            MaritalStatus::Married | MaritalStatus::InRelationship => {
                adj.oxytocin += 0.005;
                adj.serotonin += 0.003;
            }
            MaritalStatus::Divorced | MaritalStatus::Widowed => {
                adj.cortisol += 0.005;
                adj.serotonin -= 0.003;
            }
            _ => {}
        }

        adj
    }

    /// Genere un supplement de prompt decrivant la situation familiale.
    pub fn prompt_supplement(&self) -> String {
        if self.members.is_empty() && self.marital_status == MaritalStatus::Single {
            return String::new();
        }

        let mut lines = Vec::new();
        lines.push("SITUATION FAMILIALE :".to_string());
        lines.push(format!("  Statut: {}", self.marital_status.as_str()));

        for m in &self.members {
            let presence = if m.present { "present" } else { "absent" };
            lines.push(format!("  - {} ({}, lien: {:.0}%, {})",
                m.name, m.role.as_str(), m.bond_strength * 100.0, presence));
        }

        lines.join("\n")
    }

    /// Serialise en JSON pour l'API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "marital_status": self.marital_status.as_str(),
            "member_count": self.members.len(),
            "present_count": self.members.iter().filter(|m| m.present).count(),
            "members": self.members.iter().map(|m| serde_json::json!({
                "name": m.name,
                "role": m.role.as_str(),
                "bond_strength": m.bond_strength,
                "present": m.present,
            })).collect::<Vec<_>>(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_family_context_default() {
        let ctx = FamilyContext::default();
        assert!(ctx.members.is_empty());
        assert_eq!(ctx.marital_status, MaritalStatus::Single);
    }

    #[test]
    fn test_chemistry_influence_with_family() {
        let ctx = FamilyContext {
            members: vec![FamilyMember {
                name: "JRM".into(),
                role: FamilyRole::Parent,
                bond_strength: 0.9,
                present: true,
            }],
            marital_status: MaritalStatus::Single,
        };
        let adj = ctx.chemistry_influence();
        assert!(adj.oxytocin > 0.0, "Present family should boost oxytocin");
    }

    #[test]
    fn test_prompt_supplement() {
        let ctx = FamilyContext {
            members: vec![FamilyMember {
                name: "JRM".into(),
                role: FamilyRole::Parent,
                bond_strength: 0.9,
                present: true,
            }],
            marital_status: MaritalStatus::Single,
        };
        let supplement = ctx.prompt_supplement();
        assert!(supplement.contains("JRM"));
        assert!(supplement.contains("parent"));
    }
}
