// =============================================================================
// conditions/employment.rs — Employment and professional status
// =============================================================================
//
// Role: Models the professional status (employed, self-employed, unemployed,
//       student, retired, etc.) with profession category and chemistry
//       impact based on satisfaction and stress.
//
// Integration:
//   Provides a supplement to the LLM system prompt and impacts chemistry.
//   Enabled via [employment] enabled = true in saphire.toml.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Employment status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EmploymentStatus {
    /// Employee (permanent, fixed-term, etc.)
    Employed,
    /// Self-employed worker (freelance, sole proprietor)
    SelfEmployed,
    /// Unemployed / jobless
    Unemployed,
    /// Student
    Student,
    /// Retired
    Retired,
    /// Disabled / occupationally handicapped
    Disabled,
    /// Homemaker (parent, caregiver)
    Homemaker,
    /// Volunteer
    Volunteer,
}

impl EmploymentStatus {
    /// Parses from a configuration string.
    pub fn from_str_config(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "employed" | "salarie" => Self::Employed,
            "self_employed" | "self-employed" | "independant" => Self::SelfEmployed,
            "unemployed" | "chomeur" | "sans-emploi" => Self::Unemployed,
            "student" | "etudiant" => Self::Student,
            "retired" | "retraite" => Self::Retired,
            "disabled" | "invalide" => Self::Disabled,
            "homemaker" | "foyer" => Self::Homemaker,
            "volunteer" | "benevole" => Self::Volunteer,
            _ => Self::Employed,
        }
    }

    /// Display name in French.
    fn label_fr(&self) -> &'static str {
        match self {
            Self::Employed => "salarie",
            Self::SelfEmployed => "independant",
            Self::Unemployed => "sans emploi",
            Self::Student => "etudiant",
            Self::Retired => "retraite",
            Self::Disabled => "en invalidite",
            Self::Homemaker => "au foyer",
            Self::Volunteer => "benevole",
        }
    }
}

/// Profession category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProfessionCategory {
    Technology,
    Healthcare,
    Education,
    Arts,
    Science,
    Trade,
    Business,
    Law,
    PublicService,
    Agriculture,
    Media,
    Social,
    Military,
    Other,
}

impl ProfessionCategory {
    /// Parses from a configuration string.
    pub fn from_str_config(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "technology" | "tech" | "informatique" => Self::Technology,
            "healthcare" | "sante" | "medical" => Self::Healthcare,
            "education" | "enseignement" => Self::Education,
            "arts" | "art" => Self::Arts,
            "science" | "recherche" => Self::Science,
            "trade" | "artisanat" | "metier" => Self::Trade,
            "business" | "commerce" | "affaires" => Self::Business,
            "law" | "droit" | "juridique" => Self::Law,
            "public_service" | "service-public" | "fonction-publique" => Self::PublicService,
            "agriculture" => Self::Agriculture,
            "media" | "medias" | "journalisme" => Self::Media,
            "social" | "travail-social" => Self::Social,
            "military" | "militaire" | "armee" => Self::Military,
            _ => Self::Other,
        }
    }

    /// Display name in French.
    fn label_fr(&self) -> &'static str {
        match self {
            Self::Technology => "technologie",
            Self::Healthcare => "sante",
            Self::Education => "education",
            Self::Arts => "arts",
            Self::Science => "science",
            Self::Trade => "artisanat",
            Self::Business => "commerce",
            Self::Law => "droit",
            Self::PublicService => "service public",
            Self::Agriculture => "agriculture",
            Self::Media => "medias",
            Self::Social => "social",
            Self::Military => "militaire",
            Self::Other => "autre",
        }
    }
}

/// Complete employment state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentState {
    /// Current employment status
    pub status: EmploymentStatus,
    /// Profession category (if applicable)
    pub profession: Option<ProfessionCategory>,
    /// Free-form job title
    pub job_title: Option<String>,
    /// Professional satisfaction (0.0-1.0)
    pub satisfaction: f64,
    /// Professional stress level (0.0-1.0)
    pub stress_level: f64,
    /// Years of experience
    pub years_experience: f64,
}

impl EmploymentState {
    /// Constructor.
    pub fn new(
        status: EmploymentStatus,
        profession: Option<ProfessionCategory>,
        job_title: Option<String>,
        satisfaction: f64,
        stress_level: f64,
        years_experience: f64,
    ) -> Self {
        Self {
            status,
            profession,
            job_title,
            satisfaction: satisfaction.clamp(0.0, 1.0),
            stress_level: stress_level.clamp(0.0, 1.0),
            years_experience: years_experience.max(0.0),
        }
    }

    /// Chemistry impact based on status, satisfaction and stress.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();
        let sat = self.satisfaction;
        let stress = self.stress_level;

        match self.status {
            EmploymentStatus::Employed => {
                adj.dopamine += sat * 0.02;
                adj.serotonin += sat * 0.02;
                adj.cortisol += stress * 0.03;
                adj.noradrenaline += stress * 0.02;
            }
            EmploymentStatus::SelfEmployed => {
                adj.dopamine += sat * 0.03;
                adj.serotonin += sat * 0.01;
                adj.cortisol += stress * 0.03;
                adj.noradrenaline += stress * 0.02;
            }
            EmploymentStatus::Unemployed => {
                adj.dopamine -= 0.03;
                adj.serotonin -= 0.03;
                adj.cortisol += 0.03;
            }
            EmploymentStatus::Student => {
                adj.dopamine += 0.01;
                adj.cortisol += stress * 0.02;
                adj.noradrenaline += 0.01;
            }
            EmploymentStatus::Retired => {
                adj.serotonin += sat * 0.02;
                adj.cortisol -= sat * 0.01;
            }
            EmploymentStatus::Disabled => {
                adj.dopamine -= 0.01;
                adj.serotonin -= 0.02;
                adj.cortisol += 0.02;
            }
            EmploymentStatus::Homemaker => {
                adj.serotonin += sat * 0.01;
                adj.cortisol += stress * 0.01;
            }
            EmploymentStatus::Volunteer => {
                adj.dopamine += 0.02;
                adj.serotonin += 0.02;
            }
        }

        adj
    }

    /// Supplement for the LLM system prompt.
    pub fn prompt_supplement(&self) -> String {
        let status_str = self.status.label_fr();

        let profession_str = match (&self.status, &self.profession, &self.job_title) {
            (EmploymentStatus::Unemployed, _, _) => String::new(),
            (_, _, Some(title)) if !title.is_empty() => format!(", titre : {}", title),
            (_, Some(cat), _) => format!(", domaine : {}", cat.label_fr()),
            _ => String::new(),
        };

        let satisfaction_str = if self.status != EmploymentStatus::Unemployed {
            if self.satisfaction > 0.7 {
                " Tu es satisfait de ta situation professionnelle."
            } else if self.satisfaction < 0.3 {
                " Tu es insatisfait de ta situation professionnelle."
            } else {
                ""
            }
        } else {
            " La recherche d'emploi pese sur toi."
        };

        let stress_str = if self.stress_level > 0.7 {
            " Le stress professionnel est eleve."
        } else {
            ""
        };

        format!(
            "Statut professionnel : {}{}. Experience : {:.0} ans.{}{}",
            status_str,
            profession_str,
            self.years_experience,
            satisfaction_str,
            stress_str,
        )
    }

    /// Serializes for the API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "active": true,
            "status": format!("{:?}", self.status),
            "profession": self.profession.as_ref().map(|p| format!("{:?}", p)),
            "job_title": self.job_title,
            "satisfaction": self.satisfaction,
            "stress_level": self.stress_level,
            "years_experience": self.years_experience,
            "prompt_supplement": self.prompt_supplement(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let state = EmploymentState::new(
            EmploymentStatus::Employed,
            Some(ProfessionCategory::Technology),
            Some("Developpeur Rust".into()),
            0.8,
            0.4,
            10.0,
        );
        assert_eq!(state.status, EmploymentStatus::Employed);
        assert_eq!(state.satisfaction, 0.8);
        assert_eq!(state.years_experience, 10.0);
    }

    #[test]
    fn test_chemistry_employed() {
        let state = EmploymentState::new(
            EmploymentStatus::Employed,
            None,
            None,
            0.8,
            0.3,
            5.0,
        );
        let adj = state.chemistry_influence();
        assert!(adj.dopamine > 0.0);
        assert!(adj.serotonin > 0.0);
        assert!(adj.cortisol > 0.0); // stress > 0
    }

    #[test]
    fn test_chemistry_unemployed() {
        let state = EmploymentState::new(
            EmploymentStatus::Unemployed,
            None,
            None,
            0.0,
            0.0,
            0.0,
        );
        let adj = state.chemistry_influence();
        assert!(adj.dopamine < 0.0);
        assert!(adj.serotonin < 0.0);
        assert!(adj.cortisol > 0.0);
    }

    #[test]
    fn test_chemistry_volunteer() {
        let state = EmploymentState::new(
            EmploymentStatus::Volunteer,
            Some(ProfessionCategory::Social),
            None,
            0.9,
            0.1,
            2.0,
        );
        let adj = state.chemistry_influence();
        assert!(adj.dopamine > 0.0);
        assert!(adj.serotonin > 0.0);
    }

    #[test]
    fn test_prompt_supplement() {
        let state = EmploymentState::new(
            EmploymentStatus::Employed,
            Some(ProfessionCategory::Technology),
            Some("Ingenieur IA".into()),
            0.8,
            0.3,
            10.0,
        );
        let prompt = state.prompt_supplement();
        assert!(prompt.contains("salarie"));
        assert!(prompt.contains("Ingenieur IA"));
        assert!(prompt.contains("10 ans"));
    }

    #[test]
    fn test_parse_status() {
        assert_eq!(EmploymentStatus::from_str_config("employed"), EmploymentStatus::Employed);
        assert_eq!(EmploymentStatus::from_str_config("chomeur"), EmploymentStatus::Unemployed);
        assert_eq!(EmploymentStatus::from_str_config("etudiant"), EmploymentStatus::Student);
        assert_eq!(EmploymentStatus::from_str_config("benevole"), EmploymentStatus::Volunteer);
    }

    #[test]
    fn test_parse_profession() {
        assert_eq!(ProfessionCategory::from_str_config("technology"), ProfessionCategory::Technology);
        assert_eq!(ProfessionCategory::from_str_config("sante"), ProfessionCategory::Healthcare);
        assert_eq!(ProfessionCategory::from_str_config("unknown"), ProfessionCategory::Other);
    }
}
