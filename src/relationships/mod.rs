// relationships/ — Stub for the lite edition

use serde::{Serialize, Deserialize};

pub mod family {
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FamilyConfig {
        #[serde(default)]
        pub enabled: bool,
    }

    impl Default for FamilyConfig {
        fn default() -> Self { Self { enabled: false } }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FamilyContext;

    impl FamilyContext {
        pub fn from_config(_config: &FamilyConfig) -> Self { FamilyContext }
        pub fn to_json(&self) -> serde_json::Value { serde_json::json!({}) }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AttachmentStyle {
    Secure,
    Anxious,
    Avoidant,
    Disorganized,
}

impl Default for AttachmentStyle {
    fn default() -> Self { AttachmentStyle::Secure }
}

impl std::fmt::Display for AttachmentStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BondType {
    Friend,
    Mentor,
    Rival,
    Family,
    Stranger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipBond {
    pub person_id: String,
    pub bond_type: BondType,
    pub strength: f64,
    pub trust: f64,
    pub conflict_level: f64,
    pub shared_memories: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipNetwork {
    pub attachment_style: AttachmentStyle,
    pub bonds: Vec<RelationshipBond>,
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
    pub fn observe_interaction(&mut self, _username: &str, _sentiment: f64, _emotion: &str) {
        // stub
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::json!({}))
    }
}
