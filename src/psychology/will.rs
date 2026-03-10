// psychology/will.rs — Stub for the lite edition

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WillConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
}
fn default_true() -> bool { true }

impl Default for WillConfig {
    fn default() -> Self { Self { enabled: false } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    EthicalDilemma,
    InternalConflict,
    HighStakes,
    Ambiguity,
    Temptation,
    Urgency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WillTrigger {
    pub trigger_type: TriggerType,
    pub urgency: f64,
    pub complexity: f64,
    pub stakes: f64,
}

pub struct WillInput {
    pub dopamine: f64,
    pub cortisol: f64,
    pub serotonin: f64,
    pub adrenaline: f64,
    pub oxytocin: f64,
    pub endorphin: f64,
    pub noradrenaline: f64,
    pub internal_conflict: f64,
    pub ego_strength: f64,
    pub ego_strategy_overwhelmed: bool,
    pub ego_anxiety: f64,
    pub id_drive_strength: f64,
    pub id_frustration: f64,
    pub id_active_drives_count: usize,
    pub superego_strength: f64,
    pub superego_guilt: f64,
    pub superego_pride: f64,
    pub toltec_alignments: Vec<(u8, f64)>,
    pub toltec_overall: f64,
    pub maslow_active_level: usize,
    pub maslow_active_satisfaction: f64,
    pub intuition_acuity: f64,
    pub intuition_top_confidence: f64,
    pub intuition_top_description: String,
    pub ethics_active_count: usize,
    pub consciousness_level: f64,
    pub desires_active_count: usize,
    pub desires_top_description: String,
    pub learning_confirmed_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliberationOption {
    pub description: String,
    pub id_score: f64,
    pub superego_score: f64,
    pub weighted_score: f64,
    pub maslow_score: f64,
    pub toltec_score: f64,
    pub pragmatic_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChemistryInfluence {
    pub boldness: f64,
    pub caution: f64,
    pub wisdom: f64,
    pub efficiency: f64,
    pub urgency: f64,
    pub empathy: f64,
}

impl Default for ChemistryInfluence {
    fn default() -> Self {
        Self { boldness: 0.0, caution: 0.0, wisdom: 0.0, efficiency: 0.0, urgency: 0.0, empathy: 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deliberation {
    pub trigger: WillTrigger,
    pub options: Vec<DeliberationOption>,
    pub chosen: usize,
    pub confidence: f64,
    pub reasoning: String,
    pub regret: f64,
    pub created_at: DateTime<Utc>,
    pub chemistry_influence: ChemistryInfluence,
}

impl Deliberation {
    pub fn from_persisted_json(_json: &serde_json::Value) -> Option<Self> {
        None // stub — no restoration in lite
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WillEngine {
    pub willpower: f64,
    pub decision_fatigue: f64,
    pub total_deliberations: u64,
    pub proud_decisions: u64,
    pub regretted_decisions: u64,
    pub recent_deliberations: Vec<Deliberation>,
}

impl WillEngine {
    pub fn new(_config: &WillConfig) -> Self {
        Self {
            willpower: 0.8,
            decision_fatigue: 0.0,
            total_deliberations: 0,
            proud_decisions: 0,
            regretted_decisions: 0,
            recent_deliberations: Vec::new(),
        }
    }

    pub fn should_deliberate(&self, _input: &WillInput) -> Option<WillTrigger> {
        None // stub — never deliberates in lite
    }

    pub fn deliberate(&mut self, trigger: WillTrigger, _input: &WillInput) -> Deliberation {
        Deliberation {
            trigger,
            options: vec![],
            chosen: 0,
            confidence: 0.0,
            reasoning: String::new(),
            regret: 0.0,
            created_at: Utc::now(),
            chemistry_influence: ChemistryInfluence::default(),
        }
    }

    pub fn describe_for_prompt(&self) -> String {
        String::new()
    }

    pub fn to_broadcast_json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    pub fn receive_dissonance_signal(&mut self, _tension: f64) {
        // stub
    }
}
