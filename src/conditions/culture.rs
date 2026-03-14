// =============================================================================
// conditions/culture.rs — Culture, societe, croyances
// =============================================================================
//
// Role : Definit le cadre culturel : valeurs, normes sociales, croyances,
//        tabous, style de communication. Influence l'ethique, le langage,
//        les reactions emotionnelles.
//
// Integration :
//   Fournit un supplement au system prompt LLM pour adapter le ton.
//   Les tabous filtrent les sujets. Les croyances alimentent l'ethique.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Style de communication.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommStyle {
    /// Parle clairement et directement
    Direct,
    /// Utilise sous-entendus, formulations indirectes
    Indirect,
    /// Vouvoiement, respect hierarchique
    Formal,
    /// Tutoiement, decontracte
    Informal,
}

impl Default for CommStyle {
    fn default() -> Self { Self::Direct }
}

/// Domaine d'une croyance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BeliefDomain {
    Spiritual,
    Political,
    Philosophical,
    Scientific,
    Ethical,
}

/// Une croyance individuelle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
    pub domain: BeliefDomain,
    /// Enonce de la croyance
    pub content: String,
    /// Conviction (0.0 = doute, 1.0 = certitude absolue)
    pub conviction: f64,
    /// A ete remise en question
    pub questioned: bool,
}

/// Cadre culturel complet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CulturalFramework {
    /// Nom du preset culturel
    pub preset_name: String,
    /// Style de communication
    pub comm_style: CommStyle,
    /// Sujets tabous (mots-cles qui declenchent un evitement)
    pub taboos: Vec<String>,
    /// Croyances
    pub beliefs: Vec<Belief>,
    /// Valeurs ordonnees par importance
    pub values_hierarchy: Vec<String>,
    /// Permettre l'evolution des croyances avec l'experience
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

    /// Cree un preset occidental laique.
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

    /// Cree un preset oriental confuceen.
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

    /// Verifie si un texte touche un tabou.
    pub fn touches_taboo(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        self.taboos.iter().any(|t| text_lower.contains(&t.to_lowercase()))
    }

    /// Remet en question une croyance si l'experience la contredit.
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

    /// Supplement au system prompt LLM.
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

    /// Impact chimique (les tabous touches causent du stress).
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

    /// Serialise pour l'API.
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
