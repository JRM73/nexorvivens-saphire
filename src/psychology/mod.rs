// =============================================================================
// psychology/mod.rs — Cadres psychologiques de Saphire
//
// 6 frameworks psychologiques fonctionnant en parallele :
//   1. Freud — Ca/Moi/Surmoi, pulsions, mecanismes de defense
//   2. Maslow — Pyramide des besoins (5 niveaux)
//   3. Tolteques — 4 Accords (parole, personnel, suppositions, mieux)
//   4. Jung — Ombre, archetypes, integration
//   5. Goleman — Intelligence emotionnelle (5 composantes)
//   6. Csikszentmihalyi — Etat de flow (defi/competence)
//
// PsychologyInput est un snapshot pour eviter les conflits de borrow.
// PsychologyFramework orchestre les 6 sous-frameworks.
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

/// Configuration du module psychologique.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsychologyConfig {
    /// Active ou desactive l'ensemble des cadres psychologiques
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

/// Snapshot de l'etat interne de Saphire pour les calculs psychologiques.
///
/// Cette structure copie toutes les valeurs necessaires depuis l'agent
/// avant d'appeler update() sur le framework, evitant ainsi les conflits
/// de borrow (immutable/mutable).
#[derive(Debug, Clone)]
pub struct PsychologyInput {
    // Neurochimie (7 molecules)
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

    // Regulation
    pub was_vetoed: bool,

    // Ethique
    pub ethics_active_count: usize,

    // Corps
    pub body_energy: f64,
    pub body_vitality: f64,

    // Attention
    pub attention_depth: f64,
    pub attention_fatigue: f64,

    // Guerison
    pub healing_resilience: f64,
    pub has_loneliness: bool,

    // Apprentissage
    pub learning_confirmed_count: usize,
    pub learning_total_count: usize,

    // Desirs
    pub desires_active_count: usize,
    pub desires_fulfilled_count: usize,

    // Etat general
    pub in_conversation: bool,
    pub cycle_count: u64,

    // Cross-framework (mis a jour par freudian avant d'etre lus par jung)
    pub id_frustration: f64,
    pub superego_guilt: f64,

    // Flow (mis a jour par flow avant d'etre lu par maslow)
    pub in_flow: bool,
}

// ─── Framework principal ─────────────────────────────────────────────────────

/// Orchestrateur des 6 cadres psychologiques + module de volonte.
#[derive(Debug, Clone, Serialize)]
pub struct PsychologyFramework {
    /// Module actif ?
    pub enabled: bool,
    /// Psyche freudienne (Ca/Moi/Surmoi)
    pub freudian: FreudianPsyche,
    /// Pyramide de Maslow (5 niveaux)
    pub maslow: MaslowPyramid,
    /// Accords Tolteques (4 accords)
    pub toltec: ToltecAgreements,
    /// Psychologie jungienne (Ombre, archetypes)
    pub jung: JungianShadow,
    /// Intelligence emotionnelle (Goleman, 5 composantes)
    pub eq: EmotionalIntelligence,
    /// Etat de flow (Csikszentmihalyi)
    pub flow: FlowState,
    /// Module de volonte (deliberation interne)
    pub will: WillModule,
}

impl PsychologyFramework {
    /// Cree un framework psychologique avec la config donnee.
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

    /// Met a jour tous les frameworks dans l'ordre optimal.
    ///
    /// Ordre : freudian → toltec → jung → eq → flow → maslow
    /// (Maslow en dernier car il lit les resultats des autres,
    /// notamment flow.in_flow pour le niveau Actualisation)
    pub fn update(&mut self, input: &mut PsychologyInput) {
        if !self.enabled {
            return;
        }

        // 1. Freud en premier (produit frustration et culpabilite pour les autres)
        self.freudian.compute(input);
        // Propager les valeurs cross-framework
        input.id_frustration = self.freudian.id.frustration;
        input.superego_guilt = self.freudian.superego.guilt;

        // 2. Tolteques
        self.toltec.compute(input);

        // 3. Jung (utilise frustration du Ca)
        self.jung.compute(input);

        // 4. Intelligence emotionnelle
        self.eq.compute(input);

        // 5. Flow (produit in_flow pour Maslow)
        self.flow.compute(input);
        input.in_flow = self.flow.in_flow;

        // 6. Maslow en dernier (utilise flow.in_flow)
        self.maslow.compute(input);

        // 7. Recovery de la fatigue decisionnelle
        self.will.update_fatigue_recovery();
    }

    /// Construit la description pour le prompt LLM.
    /// Concis : 50-100 tokens, ne decrit que l'etat non-trivial.
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

        // Volonte
        let will_desc = self.will.describe_for_prompt();
        if !will_desc.is_empty() { parts.push(will_desc); }

        if parts.is_empty() {
            String::new()
        } else {
            format!("\n--- MA PSYCHE ---\n{}\n", parts.join("\n"))
        }
    }

    /// Retourne l'influence chimique combinee de tous les frameworks.
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

    /// Serialise l'etat complet pour le broadcast WebSocket.
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
