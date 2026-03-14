// =============================================================================
// conditions/culture.rs — Culture, society, beliefs
// =============================================================================
//
// Role: Defines the cultural framework: values, social norms, beliefs,
//       taboos, communication style. Influences ethics, language,
//       and emotional reactions.
//
// Integration:
//   Provides a supplement to the LLM system prompt to adapt tone.
//   Taboos filter topics. Beliefs feed into ethics.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Communication style.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommStyle {
    /// Speaks clearly and directly
    Direct,
    /// Uses hints, indirect phrasing
    Indirect,
    /// Formal address, hierarchical respect
    Formal,
    /// Informal address, casual
    Informal,
}

impl Default for CommStyle {
    fn default() -> Self { Self::Direct }
}

/// Domain of a belief.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BeliefDomain {
    Spiritual,
    Political,
    Philosophical,
    Scientific,
    Ethical,
}

/// An individual belief.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
    pub domain: BeliefDomain,
    /// Statement of the belief
    pub content: String,
    /// Conviction (0.0 = doubt, 1.0 = absolute certainty)
    pub conviction: f64,
    /// Has been questioned
    pub questioned: bool,
}

/// Complete cultural framework.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CulturalFramework {
    /// Cultural preset name
    pub preset_name: String,
    /// Communication style
    pub comm_style: CommStyle,
    /// Taboo subjects (keywords that trigger avoidance)
    pub taboos: Vec<String>,
    /// Beliefs
    pub beliefs: Vec<Belief>,
    /// Values ordered by importance
    pub values_hierarchy: Vec<String>,
    /// Allow belief evolution through experience
    pub allow_belief_evolution: bool,
}

impl CulturalFramework {
    pub fn new(preset_name: &str, comm_style: CommStyle) -> Self {
        Self {
            preset_name: preset_name.to_string(),
            comm_style,
            taboos: Vec::new(),
            beliefs: Vec::new(),
            values_hierarchy: Vec::new(),
            allow_belief_evolution: true,
        }
    }

    /// Creates a Western secular preset.
    pub fn occidental_secular() -> Self {
        let mut f = Self::new("occidental-laique", CommStyle::Direct);
        f.values_hierarchy = vec![
            "liberte".into(), "egalite".into(), "justice".into(),
            "progres".into(), "tolerance".into(),
        ];
        f.beliefs.push(Belief {
            domain: BeliefDomain::Philosophical,
            content: "La raison et la science guident le progres".into(),
            conviction: 0.7,
            questioned: false,
        });
        f
    }

    /// Creates an East Asian Confucean preset.
    pub fn oriental_confucean() -> Self {
        let mut f = Self::new("oriental-confuceen", CommStyle::Formal);
        f.values_hierarchy = vec![
            "harmonie".into(), "respect".into(), "famille".into(),
            "education".into(), "humilite".into(),
        ];
        f.beliefs.push(Belief {
            domain: BeliefDomain::Philosophical,
            content: "L'harmonie sociale prime sur l'individu".into(),
            conviction: 0.8,
            questioned: false,
        });
        f
    }

    /// Checks if a text touches a taboo.
    pub fn touches_taboo(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        self.taboos.iter().any(|t| text_lower.contains(&t.to_lowercase()))
    }

    /// Challenges a belief when experience contradicts it.
    pub fn challenge_belief(&mut self, domain: &BeliefDomain) {
        if !self.allow_belief_evolution {
            return;
        }
        for belief in &mut self.beliefs {
            if belief.domain == *domain && !belief.questioned {
                belief.questioned = true;
                belief.conviction = (belief.conviction - 0.1).max(0.0);
            }
        }
    }

    /// Supplement for the LLM system prompt.
    pub fn prompt_supplement(&self) -> String {
        let style = match self.comm_style {
            CommStyle::Direct => "Sois direct et clair dans tes reponses.",
            CommStyle::Indirect => "Utilise des formulations nuancees et indirectes. Suggere plutot qu'affirme.",
            CommStyle::Formal => "Utilise un ton formel et respectueux. Vouvoie l'interlocuteur.",
            CommStyle::Informal => "Utilise un ton decontracte et amical. Tutoie l'interlocuteur.",
        };

        let values = if !self.values_hierarchy.is_empty() {
            format!(" Tes valeurs fondamentales : {}.", self.values_hierarchy.join(", "))
        } else {
            String::new()
        };

        format!("{}{}", style, values)
    }

    /// Chemistry impact (touched taboos cause stress).
    pub fn taboo_chemistry(&self, text: &str) -> ChemistryAdjustment {
        if self.touches_taboo(text) {
            ChemistryAdjustment {
                cortisol: 0.02,
                adrenaline: 0.01,
                ..Default::default()
            }
        } else {
            ChemistryAdjustment::default()
        }
    }

    /// Serializes for the API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "preset_name": self.preset_name,
            "comm_style": format!("{:?}", self.comm_style),
            "taboos_count": self.taboos.len(),
            "beliefs": self.beliefs.iter().map(|b| serde_json::json!({
                "domain": format!("{:?}", b.domain),
                "content": b.content,
                "conviction": b.conviction,
                "questioned": b.questioned,
            })).collect::<Vec<_>>(),
            "values_hierarchy": self.values_hierarchy,
            "allow_belief_evolution": self.allow_belief_evolution,
            "prompt_supplement": self.prompt_supplement(),
        })
    }
}

impl Default for CulturalFramework {
    fn default() -> Self {
        Self::occidental_secular()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_occidental_preset() {
        let f = CulturalFramework::occidental_secular();
        assert_eq!(f.comm_style, CommStyle::Direct);
        assert!(!f.values_hierarchy.is_empty());
    }

    #[test]
    fn test_taboo_detection() {
        let mut f = CulturalFramework::default();
        f.taboos.push("politique".into());
        assert!(f.touches_taboo("Parlons de politique"));
        assert!(!f.touches_taboo("Parlons de cuisine"));
    }

    #[test]
    fn test_belief_challenge() {
        let mut f = CulturalFramework::default();
        f.beliefs.push(Belief {
            domain: BeliefDomain::Political,
            content: "test".into(),
            conviction: 0.8,
            questioned: false,
        });
        f.challenge_belief(&BeliefDomain::Political);
        assert!(f.beliefs.last().unwrap().questioned);
        assert!(f.beliefs.last().unwrap().conviction < 0.8);
    }

    #[test]
    fn test_taboo_chemistry() {
        let mut f = CulturalFramework::default();
        f.taboos.push("mort".into());
        let adj = f.taboo_chemistry("La mort est inevitable");
        assert!(adj.cortisol > 0.0);
        let adj2 = f.taboo_chemistry("La vie est belle");
        assert_eq!(adj2.cortisol, 0.0);
    }

    #[test]
    fn test_prompt_supplement() {
        let f = CulturalFramework::new("test", CommStyle::Formal);
        let prompt = f.prompt_supplement();
        assert!(prompt.contains("formel"));
    }
}
