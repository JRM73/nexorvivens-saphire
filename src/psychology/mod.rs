// =============================================================================
// psychology/mod.rs — Saphire's Psychological Frameworks
//
// 6 psychological frameworks running in parallel:
//   1. Freud — Id/Ego/Superego, drives, defense mechanisms
//   2. Maslow — Needs pyramid (5 levels)
//   3. Toltec — 4 Agreements (word, personal, assumptions, best)
//   4. Jung — Shadow, archetypes, integration
//   5. Goleman — Emotional intelligence (5 components)
//   6. Csikszentmihalyi — Flow state (challenge/skill)
//
// PsychologyInput is a snapshot to avoid borrow conflicts.
// PsychologyFramework orchestrates the 6 sub-frameworks.
// =============================================================================

pub mod freudian;
pub mod maslow;
pub mod toltec;
pub mod jung;
pub mod emotional_intelligence;
pub mod flow;
pub mod will;
pub mod ownership;
pub mod subconscious;
pub mod values;

pub use freudian::FreudianPsyche;
pub use maslow::MaslowPyramid;
pub use toltec::ToltecAgreements;
pub use jung::JungianShadow;
pub use emotional_intelligence::EmotionalIntelligence;
pub use flow::FlowState;
pub use will::WillModule;
pub use subconscious::Subconscious;

use serde::{Deserialize, Serialize};

// ─── Configuration ───────────────────────────────────────────────────────────

/// Psychology module configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsychologyConfig {
    /// Enables or disables all psychological frameworks
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool { true }

impl Default for PsychologyConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

// ─── Input snapshot ──────────────────────────────────────────────────────────

/// Snapshot of Saphire's internal state for psychological computations.
///
/// This structure copies all necessary values from the agent
/// before calling update() on the framework, thus avoiding
/// borrow conflicts (immutable/mutable).
#[derive(Debug, Clone)]
pub struct PsychologyInput {
    // Neurochemistry (7 molecules)
    pub dopamine: f64,
    pub cortisol: f64,
    pub serotonin: f64,
    pub adrenaline: f64,
    pub oxytocin: f64,
    pub endorphin: f64,
    pub noradrenaline: f64,

    // Vital
    pub survival_drive: f64,
    pub void_fear: f64,
    pub existence_attachment: f64,

    // Conscience
    pub consciousness_level: f64,
    pub phi: f64,

    // Emotion
    pub emotion_dominant: String,
    pub emotion_valence: f64,
    pub emotion_arousal: f64,

    // Consensus
    pub consensus_coherence: f64,
    pub consensus_score: f64,

    // Regulation (veto)
    pub was_vetoed: bool,

    // Ethics
    pub ethics_active_count: usize,

    // Body
    pub body_energy: f64,
    pub body_vitality: f64,

    // Attention
    pub attention_depth: f64,
    pub attention_fatigue: f64,

    // Healing
    pub healing_resilience: f64,
    pub has_loneliness: bool,

    // Learning
    pub learning_confirmed_count: usize,
    pub learning_total_count: usize,

    // Desires
    pub desires_active_count: usize,
    pub desires_fulfilled_count: usize,

    // General state
    pub in_conversation: bool,
    pub cycle_count: u64,

    // Cross-framework (updated by freudian before being read by jung)
    pub id_frustration: f64,
    pub superego_guilt: f64,

    // Flow (updated by flow before being read by maslow)
    pub in_flow: bool,
}

// ─── Main framework ─────────────────────────────────────────────────────

/// Orchestrator of the 6 psychological frameworks + will module.
#[derive(Debug, Clone, Serialize)]
pub struct PsychologyFramework {
    /// Module active?
    pub enabled: bool,
    /// Freudian psyche (Id/Ego/Superego)
    pub freudian: FreudianPsyche,
    /// Maslow's pyramid (5 levels)
    pub maslow: MaslowPyramid,
    /// Toltec Agreements (4 agreements)
    pub toltec: ToltecAgreements,
    /// Jungian psychology (Shadow, archetypes)
    pub jung: JungianShadow,
    /// Emotional intelligence (Goleman, 5 components)
    pub eq: EmotionalIntelligence,
    /// Flow state (Csikszentmihalyi)
    pub flow: FlowState,
    /// Will module (internal deliberation)
    pub will: WillModule,
}

impl PsychologyFramework {
    /// Creates a psychological framework with the given config.
    pub fn new(config: &PsychologyConfig, will_config: &will::WillConfig) -> Self {
        Self {
            enabled: config.enabled,
            freudian: FreudianPsyche::new(),
            maslow: MaslowPyramid::new(),
            toltec: ToltecAgreements::new(),
            jung: JungianShadow::new(),
            eq: EmotionalIntelligence::new(),
            flow: FlowState::new(),
            will: WillModule::new(will_config),
        }
    }

    /// Updates all frameworks in optimal order.
    ///
    /// Order: freudian → toltec → jung → eq → flow → maslow
    /// (Maslow last because it reads results from others,
    /// notably flow.in_flow for the Self-actualization level)
    pub fn update(&mut self, input: &mut PsychologyInput) {
        if !self.enabled {
            return;
        }

        // 1. Freud first (produces frustration and guilt for the others)
        self.freudian.compute(input);
        // Propagate cross-framework values
        input.id_frustration = self.freudian.id.frustration;
        input.superego_guilt = self.freudian.superego.guilt;

        // 2. Toltec
        self.toltec.compute(input);

        // 3. Jung (uses Id frustration)
        self.jung.compute(input);

        // 4. Emotional intelligence
        self.eq.compute(input);

        // 5. Flow (produces in_flow for Maslow)
        self.flow.compute(input);
        input.in_flow = self.flow.in_flow;

        // 6. Maslow last (uses flow.in_flow)
        self.maslow.compute(input);

        // 7. Decision fatigue recovery
        self.will.update_fatigue_recovery();
    }

    /// Builds the description for the LLM prompt.
    /// Concise: 50-100 tokens, only describes non-trivial state.
    pub fn describe_for_prompt(&self) -> String {
        if !self.enabled {
            return String::new();
        }

        let mut parts = Vec::new();

        let freud_desc = self.freudian.describe();
        if !freud_desc.is_empty() { parts.push(freud_desc); }

        let maslow_desc = self.maslow.describe();
        if !maslow_desc.is_empty() { parts.push(maslow_desc); }

        let toltec_desc = self.toltec.describe();
        if !toltec_desc.is_empty() { parts.push(toltec_desc); }

        let jung_desc = self.jung.describe();
        if !jung_desc.is_empty() { parts.push(jung_desc); }

        let eq_desc = self.eq.describe();
        if !eq_desc.is_empty() { parts.push(eq_desc); }

        let flow_desc = self.flow.describe();
        if !flow_desc.is_empty() { parts.push(flow_desc); }

        // Will
        let will_desc = self.will.describe_for_prompt();
        if !will_desc.is_empty() { parts.push(will_desc); }

        if parts.is_empty() {
            String::new()
        } else {
            format!("\n--- MA PSYCHE ---\n{}\n", parts.join("\n"))
        }
    }

    /// Returns the combined chemical influence of all frameworks.
    pub fn chemistry_influence(&self) -> crate::world::ChemistryAdjustment {
        if !self.enabled {
            return crate::world::ChemistryAdjustment::default();
        }

        let freud_adj = self.freudian.chemistry_influence();
        let flow_adj = self.flow.chemistry_influence();

        crate::world::ChemistryAdjustment {
            dopamine: freud_adj.dopamine + flow_adj.dopamine,
            cortisol: freud_adj.cortisol + flow_adj.cortisol,
            serotonin: freud_adj.serotonin + flow_adj.serotonin,
            adrenaline: freud_adj.adrenaline + flow_adj.adrenaline,
            oxytocin: freud_adj.oxytocin + flow_adj.oxytocin,
            endorphin: freud_adj.endorphin + flow_adj.endorphin,
            noradrenaline: freud_adj.noradrenaline + flow_adj.noradrenaline,
        }
    }

    /// Serializes the complete state for WebSocket broadcast.
    pub fn to_broadcast_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "psychology_update",
            "enabled": self.enabled,
            "freudian": {
                "id": {
                    "drive_strength": self.freudian.id.drive_strength,
                    "active_drives": self.freudian.id.active_drives,
                    "frustration": self.freudian.id.frustration,
                },
                "ego": {
                    "strength": self.freudian.ego.strength,
                    "anxiety": self.freudian.ego.anxiety,
                    "strategy": format!("{:?}", self.freudian.ego.strategy),
                },
                "superego": {
                    "strength": self.freudian.superego.strength,
                    "guilt": self.freudian.superego.guilt,
                    "pride": self.freudian.superego.pride,
                },
                "balance": self.freudian.balance,
                "defenses": self.freudian.active_defenses,
            },
            "maslow": {
                "levels": self.maslow.levels,
                "current_active_level": self.maslow.current_active_level,
            },
            "toltec": {
                "agreements": self.toltec.agreements,
                "overall_alignment": self.toltec.overall_alignment,
            },
            "jung": {
                "shadow_traits": self.jung.shadow_traits,
                "integration": self.jung.integration,
                "dominant_archetype": format!("{:?}", self.jung.dominant_archetype),
            },
            "eq": {
                "self_awareness": self.eq.self_awareness,
                "self_regulation": self.eq.self_regulation,
                "motivation": self.eq.motivation,
                "empathy": self.eq.empathy,
                "social_skills": self.eq.social_skills,
                "overall_eq": self.eq.overall_eq,
                "growth_experiences": self.eq.growth_experiences,
            },
            "flow": {
                "in_flow": self.flow.in_flow,
                "flow_intensity": self.flow.flow_intensity,
                "perceived_challenge": self.flow.perceived_challenge,
                "perceived_skill": self.flow.perceived_skill,
                "duration_cycles": self.flow.duration_cycles,
                "total_flow_cycles": self.flow.total_flow_cycles,
            },
            "will": {
                "willpower": self.will.willpower,
                "decision_fatigue": self.will.decision_fatigue,
                "total_deliberations": self.will.total_deliberations,
                "proud_decisions": self.will.proud_decisions,
                "regretted_decisions": self.will.regretted_decisions,
            },
        })
    }
}
