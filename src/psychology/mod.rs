// psychology/ — Stub for the lite edition
// Full psychology module (Freud, Maslow, Jung, Toltec, Flow, etc.) not ported.

pub mod will;
pub mod ownership;
pub mod freudian;

use serde::{Serialize, Deserialize};

// ─── PsychologyConfig (used by config/structures.rs) ────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsychologyConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
}
fn default_true() -> bool { true }

impl Default for PsychologyConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

// ─── Subconscious ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubconsciousConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for SubconsciousConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncubatingProblem {
    pub question: String,
    pub bias_theme: String,
    pub source: String,
    pub pressure: f64,
    pub incubation_cycles: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePriming {
    pub bias_theme: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepressedContent {
    pub content: String,
    pub pressure: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subconscious {
    pub enabled: bool,
    pub activation: f64,
    pub pending_associations: Vec<String>,
    pub repressed_content: Vec<RepressedContent>,
    pub incubating_problems: Vec<IncubatingProblem>,
    pub ready_insights: Vec<SubconsciousInsight>,
    pub total_insights_surfaced: u64,
    pub total_dreams_fueled: u64,
    pub active_priming: Vec<ActivePriming>,
}

impl Subconscious {
    pub fn new(_config: &crate::config::SubconsciousConfig) -> Self {
        Self {
            enabled: false,
            activation: 0.0,
            pending_associations: Vec::new(),
            repressed_content: Vec::new(),
            incubating_problems: Vec::new(),
            ready_insights: Vec::new(),
            total_insights_surfaced: 0,
            total_dreams_fueled: 0,
            active_priming: Vec::new(),
        }
    }

    pub fn background_process(&mut self, _cortisol: f64, _phase: &str) {}
    pub fn process_repressed_emotions(&mut self) {}
    pub fn surface_insight(&mut self) -> Option<SubconsciousInsight> { None }
    pub fn to_status_json(&self) -> serde_json::Value { serde_json::json!({}) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubconsciousInsight {
    #[serde(default)]
    pub insight: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub relevance: f64,
    #[serde(default)]
    pub strength: f64,
    #[serde(default)]
    pub emotional_charge: f64,
    #[serde(default)]
    pub source_type: String,
}

impl Default for SubconsciousInsight {
    fn default() -> Self {
        Self {
            insight: String::new(), content: String::new(), relevance: 0.0,
            strength: 0.0, emotional_charge: 0.0, source_type: String::new(),
        }
    }
}

pub mod subconscious {
    pub use super::SubconsciousInsight;
}

// ─── PsychologyInput (used by thinking_reflection.rs) ───────────────────────

pub struct PsychologyInput {
    pub valence: f64,
    pub arousal: f64,
    pub dominance: f64,
    pub dopamine: f64,
    pub cortisol: f64,
    pub serotonin: f64,
    pub oxytocin: f64,
    pub endorphin: f64,
    pub noradrenaline: f64,
    pub adrenaline: f64,
    pub satisfaction: f64,
    pub consciousness_level: f64,
    pub stimulus_text: String,
    pub thought_text: String,
    pub emotion_name: String,
    pub in_conversation: bool,
    pub cycle_count: u64,
    // Extra fields used by thinking_reflection.rs
    pub survival_drive: f64,
    pub void_fear: f64,
    pub existence_attachment: f64,
    pub phi: f64,
    pub emotion_dominant: String,
    pub emotion_valence: f64,
    pub emotion_arousal: f64,
    pub consensus_coherence: f64,
    pub consensus_score: f64,
    pub was_vetoed: bool,
    pub ethics_active_count: usize,
    pub body_energy: f64,
    pub body_vitality: f64,
    pub attention_depth: f64,
    pub attention_fatigue: f64,
    pub healing_resilience: f64,
    pub has_loneliness: bool,
    pub learning_confirmed_count: usize,
    pub learning_total_count: usize,
    pub desires_active_count: usize,
    pub desires_fulfilled_count: usize,
    pub id_frustration: f64,
    pub superego_guilt: f64,
    pub in_flow: bool,
}

impl Default for PsychologyInput {
    fn default() -> Self {
        Self {
            valence: 0.0, arousal: 0.0, dominance: 0.0,
            dopamine: 0.0, cortisol: 0.0, serotonin: 0.0, oxytocin: 0.0,
            endorphin: 0.0, noradrenaline: 0.0, adrenaline: 0.0,
            satisfaction: 0.0, consciousness_level: 0.0,
            stimulus_text: String::new(), thought_text: String::new(),
            emotion_name: String::new(), in_conversation: false, cycle_count: 0,
            survival_drive: 0.0, void_fear: 0.0, existence_attachment: 0.0,
            phi: 0.0, emotion_dominant: String::new(),
            emotion_valence: 0.0, emotion_arousal: 0.0,
            consensus_coherence: 0.0, consensus_score: 0.0,
            was_vetoed: false, ethics_active_count: 0,
            body_energy: 0.0, body_vitality: 0.0,
            attention_depth: 0.0, attention_fatigue: 0.0,
            healing_resilience: 0.0, has_loneliness: false,
            learning_confirmed_count: 0, learning_total_count: 0,
            desires_active_count: 0, desires_fulfilled_count: 0,
            id_frustration: 0.0, superego_guilt: 0.0, in_flow: false,
        }
    }
}

// ─── Toltec stub ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToltecAgreement {
    pub number: u8,
    pub name: String,
    pub alignment: f64,
    pub times_invoked: u64,
    pub violations_detected: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToltecFramework {
    pub agreements: Vec<ToltecAgreement>,
    pub overall_alignment: f64,
}

impl Default for ToltecFramework {
    fn default() -> Self {
        Self {
            agreements: vec![
                ToltecAgreement { number: 1, name: "Parole impeccable".into(), alignment: 0.5, times_invoked: 0, violations_detected: 0 },
                ToltecAgreement { number: 2, name: "Ne rien prendre personnellement".into(), alignment: 0.5, times_invoked: 0, violations_detected: 0 },
                ToltecAgreement { number: 3, name: "Ne pas faire de suppositions".into(), alignment: 0.5, times_invoked: 0, violations_detected: 0 },
                ToltecAgreement { number: 4, name: "Faire de son mieux".into(), alignment: 0.5, times_invoked: 0, violations_detected: 0 },
            ],
            overall_alignment: 0.5,
        }
    }
}

// ─── Jung / Shadow stub ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JungArchetype {
    Self_,
    Shadow,
    Anima,
    Animus,
    Persona,
    Hero,
    Trickster,
    Sage,
    Innocent,
}

impl Default for JungArchetype {
    fn default() -> Self { JungArchetype::Self_ }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowTrait {
    pub name: String,
    pub repressed_intensity: f64,
    pub leaking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JungFramework {
    pub dominant_archetype: JungArchetype,
    pub integration: f64,
    pub shadow_traits: Vec<ShadowTrait>,
}

impl Default for JungFramework {
    fn default() -> Self {
        Self {
            dominant_archetype: JungArchetype::default(),
            integration: 0.5,
            shadow_traits: Vec::new(),
        }
    }
}

impl JungFramework {
    pub fn nocturnal_integration(&mut self, _amount: f64) {
        // stub
    }
}

// ─── EQ (Goleman) stub ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqFramework {
    pub overall_eq: f64,
    pub self_awareness: f64,
    pub self_regulation: f64,
    pub motivation: f64,
    pub empathy: f64,
    pub social_skills: f64,
    pub growth_experiences: u64,
}

impl Default for EqFramework {
    fn default() -> Self {
        Self {
            overall_eq: 0.5,
            self_awareness: 0.5,
            self_regulation: 0.5,
            motivation: 0.5,
            empathy: 0.5,
            social_skills: 0.5,
            growth_experiences: 0,
        }
    }
}

// ─── Maslow stub ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaslowLevel {
    pub name: String,
    pub satisfaction: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaslowFramework {
    pub current_active_level: usize,
    pub levels: Vec<MaslowLevel>,
}

impl Default for MaslowFramework {
    fn default() -> Self {
        Self {
            current_active_level: 0,
            levels: vec![
                MaslowLevel { name: "Physiologique".into(), satisfaction: 0.5 },
                MaslowLevel { name: "Securite".into(), satisfaction: 0.5 },
                MaslowLevel { name: "Appartenance".into(), satisfaction: 0.5 },
                MaslowLevel { name: "Estime".into(), satisfaction: 0.5 },
                MaslowLevel { name: "Realisation".into(), satisfaction: 0.5 },
            ],
        }
    }
}

// ─── Flow stub ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowFramework {
    pub in_flow: bool,
    pub flow_intensity: f64,
    pub total_flow_cycles: u64,
    pub duration_cycles: u64,
    pub perceived_challenge: f64,
    pub perceived_skill: f64,
}

impl Default for FlowFramework {
    fn default() -> Self {
        Self {
            in_flow: false, flow_intensity: 0.0, total_flow_cycles: 0,
            duration_cycles: 0, perceived_challenge: 0.5, perceived_skill: 0.5,
        }
    }
}

// ─── PsychologyFramework ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsychologyFramework {
    pub enabled: bool,
    pub freudian: freudian::FreudianFramework,
    pub maslow: MaslowFramework,
    pub toltec: ToltecFramework,
    pub jung: JungFramework,
    pub eq: EqFramework,
    pub flow: FlowFramework,
    pub will: will::WillEngine,
}

impl PsychologyFramework {
    pub fn new(_config: &PsychologyConfig, will_config: &will::WillConfig) -> Self {
        Self {
            enabled: _config.enabled,
            freudian: freudian::FreudianFramework::default(),
            maslow: MaslowFramework::default(),
            toltec: ToltecFramework::default(),
            jung: JungFramework::default(),
            eq: EqFramework::default(),
            flow: FlowFramework::default(),
            will: will::WillEngine::new(will_config),
        }
    }

    pub fn describe_for_prompt(&self) -> String {
        String::new()
    }

    pub fn to_broadcast_json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    pub fn update(&mut self, _input: &mut PsychologyInput) {
        // stub — no-op
    }

    pub fn chemistry_influence(&self) -> crate::world::ChemistryAdjustment {
        crate::world::ChemistryAdjustment {
            dopamine: 0.0, cortisol: 0.0, serotonin: 0.0,
            adrenaline: 0.0, oxytocin: 0.0, endorphin: 0.0, noradrenaline: 0.0,
        }
    }
}
