// conditions/ — Stub for the lite edition
// Full conditions module (addictions, trauma, phobias, etc.) not ported.
// Only bare struct shells so the rest of the codebase compiles.

use crate::world::ChemistryAdjustment;

fn zero_adj() -> ChemistryAdjustment {
    ChemistryAdjustment {
        dopamine: 0.0, cortisol: 0.0, serotonin: 0.0,
        adrenaline: 0.0, oxytocin: 0.0, endorphin: 0.0, noradrenaline: 0.0,
    }
}

pub mod motion_sickness {
    use serde::{Serialize, Deserialize};
    use crate::world::ChemistryAdjustment;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MotionSickness { pub susceptibility: f64 }
    impl MotionSickness {
        pub fn new(susceptibility: f64) -> Self { Self { susceptibility } }
        pub fn evaluate_conflict(&mut self, _senses: &[f64]) {}
        pub fn tick(&mut self) {}
        pub fn chemistry_influence(&self) -> ChemistryAdjustment { super::zero_adj() }
    }
}

pub mod phobias {
    use serde::{Serialize, Deserialize};
    use crate::world::ChemistryAdjustment;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Phobia { pub name: String, pub triggers: Vec<String>, pub intensity: f64 }
    impl Phobia {
        pub fn new(name: &str, triggers: Vec<String>, intensity: f64) -> Self {
            Self { name: name.to_string(), triggers, intensity }
        }
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PhobiaManager { pub phobias: Vec<Phobia>, pub desensitization_rate: f64 }
    impl PhobiaManager {
        pub fn new(desensitization_rate: f64) -> Self {
            Self { phobias: Vec::new(), desensitization_rate }
        }
        pub fn add(&mut self, phobia: Phobia) { self.phobias.push(phobia); }
        pub fn reset_cycle(&mut self) {}
        pub fn scan_text(&mut self, _text: &str) -> bool { false }
        pub fn chemistry_influence(&self) -> ChemistryAdjustment { super::zero_adj() }
    }
}

pub mod eating {
    use serde::{Serialize, Deserialize};
    use crate::world::ChemistryAdjustment;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum EatingDisorderType { Anorexia, Bulimia, BingeEating }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EatingDisorder { pub disorder_type: EatingDisorderType, pub severity: f64 }
    impl EatingDisorder {
        pub fn new(dtype: EatingDisorderType, severity: f64) -> Self {
            Self { disorder_type: dtype, severity }
        }
        pub fn tick(&mut self, _hunger: f64, _cortisol: f64) {}
        pub fn chemistry_influence(&mut self) -> ChemistryAdjustment { super::zero_adj() }
    }
}

pub mod disabilities {
    use serde::{Serialize, Deserialize};
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum DisabilityType { Blind, Deaf, Paraplegic, BurnSurvivor, Mute }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum DisabilityOrigin { Congenital, Acquired }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Disability { pub dtype: DisabilityType, pub origin: DisabilityOrigin, pub severity: f64 }
    impl Disability {
        pub fn new(dtype: DisabilityType, origin: DisabilityOrigin, severity: f64) -> Self {
            Self { dtype, origin, severity }
        }
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DisabilityManager {
        pub disabilities: Vec<Disability>,
        pub adaptation_rate: f64,
        pub compensation_factor: f64,
    }
    impl DisabilityManager {
        pub fn new(adaptation_rate: f64, compensation_factor: f64) -> Self {
            Self { disabilities: Vec::new(), adaptation_rate, compensation_factor }
        }
        pub fn add(&mut self, d: Disability) { self.disabilities.push(d); }
        pub fn tick(&mut self) {}
        pub fn chronic_pain(&self) -> f64 { 0.0 }
    }
}

pub mod extreme {
    use serde::{Serialize, Deserialize};
    use crate::world::ChemistryAdjustment;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ExtremeConditionType { Military, Rescuer, DeepSeaDiver, Astronaut }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ExtremeConditionManager { pub active: Option<ExtremeConditionType> }
    impl ExtremeConditionManager {
        pub fn new() -> Self { Self { active: None } }
        pub fn activate(&mut self, ctype: ExtremeConditionType) { self.active = Some(ctype); }
        pub fn tick(&mut self, _cortisol: f64) {}
        pub fn chemistry_influence(&self) -> ChemistryAdjustment { super::zero_adj() }
    }
}

pub mod addictions {
    use serde::{Serialize, Deserialize};
    use crate::world::ChemistryAdjustment;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Addiction { pub substance: String, pub dependency_level: f64 }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AddictionManager { pub active: Vec<Addiction>, pub susceptibility: f64 }
    impl AddictionManager {
        pub fn new(susceptibility: f64) -> Self {
            Self { active: Vec::new(), susceptibility }
        }
        pub fn add(&mut self, substance: &str) {
            self.active.push(Addiction { substance: substance.to_string(), dependency_level: 0.0 });
        }
        pub fn tick(&mut self, _cycle: u64) {}
        pub fn chemistry_influence(&self) -> ChemistryAdjustment { super::zero_adj() }
    }
}

pub mod trauma {
    use serde::{Serialize, Deserialize};
    use crate::world::ChemistryAdjustment;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum TraumaType { Grief, Accident, EmotionalNeglect, ChildhoodTrauma, Torture, Hostage }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TraumaticEvent { pub trauma_type: TraumaType, pub severity: f64, pub cycle: u64, pub triggers: Vec<String> }
    impl TraumaticEvent {
        pub fn new(ttype: TraumaType, severity: f64, cycle: u64, triggers: Vec<String>) -> Self {
            Self { trauma_type: ttype, severity, cycle, triggers }
        }
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PtsdState { pub traumas: Vec<TraumaticEvent>, pub healing_rate: f64, pub dissociation_threshold: f64 }
    impl PtsdState {
        pub fn new(healing_rate: f64, dissociation_threshold: f64) -> Self {
            Self { traumas: Vec::new(), healing_rate, dissociation_threshold }
        }
        pub fn add_trauma(&mut self, event: TraumaticEvent) { self.traumas.push(event); }
        pub fn scan_for_triggers(&mut self, _text: &str) {}
        pub fn tick(&mut self, _cortisol: f64) {}
        pub fn chemistry_influence(&self) -> ChemistryAdjustment { super::zero_adj() }
    }
}

pub mod nde {
    use serde::{Serialize, Deserialize};
    use crate::world::ChemistryAdjustment;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NdeExperience { pub occurred: bool, pub in_progress: bool }
    impl NdeExperience {
        pub fn new() -> Self { Self { occurred: false, in_progress: false } }
        pub fn tick(&mut self) -> bool { false }
        pub fn chemistry_influence(&self) -> ChemistryAdjustment { super::zero_adj() }
        pub fn post_nde_baseline_shift(&self) -> ChemistryAdjustment { super::zero_adj() }
    }
}

pub mod drugs {
    use serde::{Serialize, Deserialize};
    use crate::world::ChemistryAdjustment;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DrugManager { pub active: Vec<String> }
    impl DrugManager {
        pub fn new() -> Self { Self { active: Vec::new() } }
        pub fn tick(&mut self) -> ChemistryAdjustment { super::zero_adj() }
    }
}

pub mod iq_constraint {
    use serde::{Serialize, Deserialize};
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IqConstraint { pub target_iq: u8 }
    impl IqConstraint {
        pub fn from_iq(target_iq: u8) -> Self { Self { target_iq } }
    }
}

pub mod sexuality {
    use serde::{Serialize, Deserialize};
    use crate::world::ChemistryAdjustment;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum SexualOrientation { Heterosexual, Homosexual, Bisexual, Asexual, Pansexual, Undefined }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SexualityModule {
        pub orientation: SexualOrientation,
        pub libido_baseline: f64,
        pub romantic_attachment_capacity: f64,
    }
    impl SexualityModule {
        pub fn new(orientation: SexualOrientation, libido_baseline: f64, romantic_attachment_capacity: f64) -> Self {
            Self { orientation, libido_baseline, romantic_attachment_capacity }
        }
        pub fn tick(&mut self, _testosterone: f64, _estrogen: f64, _oxytocin: f64) {}
        pub fn chemistry_influence(&mut self) -> ChemistryAdjustment { super::zero_adj() }
    }
}

pub mod degenerative {
    use serde::{Serialize, Deserialize};
    use crate::world::ChemistryAdjustment;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum DegenerativeType { Alzheimer, Parkinson, Epilepsy, Dementia, MajorDepression }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DegenerativeCondition { pub disease_type: DegenerativeType, pub progression_rate: f64 }
    impl DegenerativeCondition {
        pub fn new(dtype: DegenerativeType, progression_rate: f64) -> Self {
            Self { disease_type: dtype, progression_rate }
        }
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DegenerativeManager { pub conditions: Vec<DegenerativeCondition> }
    impl DegenerativeManager {
        pub fn new() -> Self { Self { conditions: Vec::new() } }
        pub fn add(&mut self, c: DegenerativeCondition) { self.conditions.push(c); }
        pub fn tick(&mut self) {}
        pub fn chemistry_influence(&self) -> ChemistryAdjustment { super::zero_adj() }
    }
}

pub mod medical {
    use serde::{Serialize, Deserialize};
    use crate::world::ChemistryAdjustment;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum CancerStage { StageI, StageII, StageIII, StageIV, Remission }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum MedicalConditionType { Cancer(CancerStage), HIV, Autoimmune }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MedicalCondition { pub condition_type: MedicalConditionType }
    impl MedicalCondition {
        pub fn cancer(stage: CancerStage) -> Self { Self { condition_type: MedicalConditionType::Cancer(stage) } }
        pub fn hiv() -> Self { Self { condition_type: MedicalConditionType::HIV } }
        pub fn autoimmune() -> Self { Self { condition_type: MedicalConditionType::Autoimmune } }
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MedicalManager { pub conditions: Vec<MedicalCondition> }
    impl MedicalManager {
        pub fn new() -> Self { Self { conditions: Vec::new() } }
        pub fn add(&mut self, c: MedicalCondition) { self.conditions.push(c); }
        pub fn tick(&mut self) {}
        pub fn chemistry_influence(&self) -> ChemistryAdjustment { super::zero_adj() }
    }
}

pub mod culture {
    use serde::{Serialize, Deserialize};
    use crate::world::ChemistryAdjustment;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum CommStyle { Direct, Indirect, Formal, Informal }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CulturalFramework {
        pub comm_style: CommStyle,
        pub allow_belief_evolution: bool,
        pub taboos: Vec<String>,
    }
    impl CulturalFramework {
        pub fn occidental_secular() -> Self {
            Self { comm_style: CommStyle::Direct, allow_belief_evolution: true, taboos: Vec::new() }
        }
        pub fn oriental_confucean() -> Self {
            Self { comm_style: CommStyle::Indirect, allow_belief_evolution: false, taboos: Vec::new() }
        }
        pub fn taboo_chemistry(&self, _text: &str) -> ChemistryAdjustment { super::zero_adj() }
    }
}

pub mod precarity {
    use serde::{Serialize, Deserialize};
    use crate::world::ChemistryAdjustment;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum PrecariousSituation { Homeless, Refugee, Undocumented, Unemployed }
    impl PrecariousSituation {
        pub fn from_str_config(s: &str) -> Option<Self> {
            match s {
                "homeless" => Some(Self::Homeless),
                "refugee" => Some(Self::Refugee),
                "undocumented" => Some(Self::Undocumented),
                "unemployed" => Some(Self::Unemployed),
                _ => None,
            }
        }
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PrecariousState { pub situations: Vec<PrecariousSituation>, pub severity: f64, pub hope: f64 }
    impl PrecariousState {
        pub fn new(situations: Vec<PrecariousSituation>, severity: f64, hope: f64) -> Self {
            Self { situations, severity, hope }
        }
        pub fn tick(&mut self) {}
        pub fn chemistry_influence(&mut self) -> ChemistryAdjustment { super::zero_adj() }
    }
}

pub mod employment {
    use serde::{Serialize, Deserialize};
    use crate::world::ChemistryAdjustment;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum EmploymentStatus { Employed, Unemployed, Student, Retired, SelfEmployed }
    impl EmploymentStatus {
        pub fn from_str_config(s: &str) -> Self {
            match s {
                "employed" => Self::Employed,
                "unemployed" => Self::Unemployed,
                "student" => Self::Student,
                "retired" => Self::Retired,
                "self_employed" => Self::SelfEmployed,
                _ => Self::Unemployed,
            }
        }
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ProfessionCategory { Tech, Science, Health, Education, Arts, Trade, Service, Other }
    impl ProfessionCategory {
        pub fn from_str_config(s: &str) -> Self {
            match s {
                "tech" => Self::Tech,
                "science" => Self::Science,
                "health" => Self::Health,
                "education" => Self::Education,
                "arts" => Self::Arts,
                "trade" => Self::Trade,
                "service" => Self::Service,
                _ => Self::Other,
            }
        }
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EmploymentState {
        pub status: EmploymentStatus,
        pub profession: Option<ProfessionCategory>,
        pub job_title: Option<String>,
        pub satisfaction: f64,
        pub stress_level: f64,
        pub years_experience: f64,
    }
    impl EmploymentState {
        pub fn new(
            status: EmploymentStatus,
            profession: Option<ProfessionCategory>,
            job_title: Option<String>,
            satisfaction: f64,
            stress_level: f64,
            years_experience: f64,
        ) -> Self {
            Self { status, profession, job_title, satisfaction, stress_level, years_experience }
        }
        pub fn chemistry_influence(&self) -> ChemistryAdjustment { super::zero_adj() }
    }
}
