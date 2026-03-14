// =============================================================================
// psychology/will.rs — Will Module (internal deliberation)
//
// Voluntary deliberation capacity (~5% of the time). When a significant
// situation arises (psychic conflict, ethical dilemma, Toltec risk,
// strong intuition, overwhelmed ego), Saphire stops and deliberates
// between options evaluated by the Id, Superego, Maslow, Toltec and
// pragmatism. Chemistry influences the weights.
//
// Deliberation is a structured INTERNAL process (no LLM call).
// Options are generated deterministically based on trigger type.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Configuration ─────────────────────────────────────────────────────────

/// Will module configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WillConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_initial_willpower")]
    pub initial_willpower: f64,
    #[serde(default = "default_fatigue_per_deliberation")]
    pub fatigue_per_deliberation: f64,
    #[serde(default = "default_fatigue_recovery_per_cycle")]
    pub fatigue_recovery_per_cycle: f64,
    #[serde(default = "default_willpower_growth_on_proud")]
    pub willpower_growth_on_proud: f64,
    #[serde(default = "default_fatigue_threshold")]
    pub fatigue_threshold: f64,
    #[serde(default = "default_psychic_conflict_trigger")]
    pub psychic_conflict_trigger: f64,
    #[serde(default = "default_toltec_alignment_trigger")]
    pub toltec_alignment_trigger: f64,
    #[serde(default = "default_intuition_confidence_trigger")]
    pub intuition_confidence_trigger: f64,
    #[serde(default = "default_max_recent_deliberations")]
    pub max_recent_deliberations: usize,
}

fn default_true() -> bool { true }
fn default_initial_willpower() -> f64 { 0.5 }
fn default_fatigue_per_deliberation() -> f64 { 0.1 }
fn default_fatigue_recovery_per_cycle() -> f64 { 0.005 }
fn default_willpower_growth_on_proud() -> f64 { 0.02 }
fn default_fatigue_threshold() -> f64 { 0.8 }
fn default_psychic_conflict_trigger() -> f64 { 0.5 }
fn default_toltec_alignment_trigger() -> f64 { 0.4 }
fn default_intuition_confidence_trigger() -> f64 { 0.7 }
fn default_max_recent_deliberations() -> usize { 20 }

impl Default for WillConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            initial_willpower: 0.5,
            fatigue_per_deliberation: 0.1,
            fatigue_recovery_per_cycle: 0.005,
            willpower_growth_on_proud: 0.02,
            fatigue_threshold: 0.8,
            psychic_conflict_trigger: 0.5,
            toltec_alignment_trigger: 0.4,
            intuition_confidence_trigger: 0.7,
            max_recent_deliberations: 20,
        }
    }
}

// ─── Trigger types ─────────────────────────────────────────────────

/// Type of situation triggering a voluntary deliberation.
#[derive(Debug, Clone, Serialize)]
pub enum TriggerType {
    /// Psychic conflict (Id vs Superego) — high internal_conflict
    PsychicConflict,
    /// Ethical dilemma — many active principles + high consciousness
    EthicalDilemma,
    /// Toltec risk — an agreement with low alignment
    ToltecRisk { accord: u8 },
    /// Strong intuition — high confidence pattern
    StrongIntuition { pattern: String },
    /// Overwhelmed ego — Overwhelmed strategy
    EgoOverwhelmed,
}

/// Deliberation trigger with its metrics.
#[derive(Debug, Clone, Serialize)]
pub struct DeliberationTrigger {
    pub trigger_type: TriggerType,
    pub urgency: f64,
    pub complexity: f64,
    pub stakes: f64,
}

// ─── Options and evaluation ─────────────────────────────────────────────────

/// An option evaluated during a deliberation.
#[derive(Debug, Clone, Serialize)]
pub struct DeliberationOption {
    pub description: String,
    pub id_score: f64,
    pub superego_score: f64,
    pub maslow_score: f64,
    pub toltec_score: f64,
    pub pragmatic_score: f64,
    pub weighted_score: f64,
}

/// Chemistry influence on deliberation weights.
#[derive(Debug, Clone, Serialize)]
pub struct ChemistryInfluence {
    /// Boldness (dopamine)
    pub boldness: f64,
    /// Caution (cortisol)
    pub caution: f64,
    /// Wisdom (serotonin)
    pub wisdom: f64,
    /// Efficiency (1 - adrenaline)
    pub efficiency: f64,
    /// Urgency (adrenaline)
    pub urgency: f64,
    /// Empathy (oxytocin)
    pub empathy: f64,
}

/// Complete deliberation result.
#[derive(Debug, Clone, Serialize)]
pub struct Deliberation {
    pub trigger: DeliberationTrigger,
    pub options: Vec<DeliberationOption>,
    pub chosen: usize,
    /// First-person reasoning
    pub reasoning: String,
    pub chemistry_influence: ChemistryInfluence,
    pub confidence: f64,
    pub regret: Option<f64>,
    pub created_at: DateTime<Utc>,
}

impl Deliberation {
    /// Reconstructs a deliberation from JSON saved at shutdown.
    pub fn from_persisted_json(j: &serde_json::Value) -> Option<Self> {
        let trigger_str = j.get("trigger")?.as_str()?;
        let chosen_desc = j.get("chosen")?.as_str().unwrap_or("?").to_string();
        let confidence = j.get("confidence")?.as_f64().unwrap_or(0.5);
        let reasoning = j.get("reasoning")?.as_str().unwrap_or("").to_string();
        let regret = j.get("regret").and_then(|v| v.as_f64());
        let created_at = j.get("created_at")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<DateTime<Utc>>().ok())
            .unwrap_or_else(Utc::now);

        // Reconstruct the trigger (simplified type, default metrics)
        let trigger_type = if trigger_str.contains("PsychicConflict") {
            TriggerType::PsychicConflict
        } else if trigger_str.contains("EthicalDilemma") {
            TriggerType::EthicalDilemma
        } else if trigger_str.contains("ToltecRisk") {
            TriggerType::ToltecRisk { accord: 0 }
        } else if trigger_str.contains("StrongIntuition") {
            TriggerType::StrongIntuition { pattern: String::new() }
        } else if trigger_str.contains("EgoOverwhelmed") {
            TriggerType::EgoOverwhelmed
        } else {
            TriggerType::PsychicConflict
        };

        let trigger = DeliberationTrigger {
            trigger_type,
            urgency: 0.5,
            complexity: 0.5,
            stakes: 0.5,
        };

        // Reconstruct the chemistry influence
        let chem = if let Some(ci) = j.get("chemistry_influence") {
            ChemistryInfluence {
                boldness: ci.get("boldness").and_then(|v| v.as_f64()).unwrap_or(0.5),
                caution: ci.get("caution").and_then(|v| v.as_f64()).unwrap_or(0.5),
                wisdom: ci.get("wisdom").and_then(|v| v.as_f64()).unwrap_or(0.5),
                efficiency: ci.get("efficiency").and_then(|v| v.as_f64()).unwrap_or(0.5),
                urgency: ci.get("urgency").and_then(|v| v.as_f64()).unwrap_or(0.5),
                empathy: ci.get("empathy").and_then(|v| v.as_f64()).unwrap_or(0.5),
            }
        } else {
            ChemistryInfluence {
                boldness: 0.5, caution: 0.5, wisdom: 0.5,
                efficiency: 0.5, urgency: 0.5, empathy: 0.5,
            }
        };

        // Single reconstructed option (the choice made)
        let option = DeliberationOption {
            description: chosen_desc,
            id_score: 0.5,
            superego_score: 0.5,
            maslow_score: 0.5,
            toltec_score: 0.5,
            pragmatic_score: 0.5,
            weighted_score: confidence,
        };

        Some(Deliberation {
            trigger,
            options: vec![option],
            chosen: 0,
            reasoning,
            chemistry_influence: chem,
            confidence,
            regret,
            created_at,
        })
    }
}

// ─── WillInput: snapshot to avoid borrow conflicts ─────────────────

/// Snapshot of the internal state needed for deliberation.
/// Copies values from the agent before calling should_deliberate/deliberate.
#[derive(Debug, Clone)]
pub struct WillInput {
    // Chemistry (7 molecules)
    pub dopamine: f64,
    pub cortisol: f64,
    pub serotonin: f64,
    pub adrenaline: f64,
    pub oxytocin: f64,
    pub endorphin: f64,
    pub noradrenaline: f64,

    // Freudian
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

    // Toltec
    pub toltec_alignments: Vec<(u8, f64)>,  // (number, alignment)
    pub toltec_overall: f64,

    // Maslow
    pub maslow_active_level: usize,
    pub maslow_active_satisfaction: f64,

    // Intuition
    pub intuition_acuity: f64,
    pub intuition_top_confidence: f64,
    pub intuition_top_description: String,

    // Ethics
    pub ethics_active_count: usize,

    // Consciousness
    pub consciousness_level: f64,

    // Desires
    pub desires_active_count: usize,
    pub desires_top_description: String,

    // Learning
    pub learning_confirmed_count: usize,
}

// ─── WillModule ────────────────────────────────────────────────────────────

/// Will module — structured internal deliberation.
#[derive(Debug, Clone, Serialize)]
pub struct WillModule {
    /// Willpower (grows with successful decisions)
    pub willpower: f64,
    /// Decision fatigue (grows with deliberations, recovery per cycle)
    pub decision_fatigue: f64,
    /// Recent deliberations (max configurable)
    pub recent_deliberations: Vec<Deliberation>,
    /// Total deliberations since creation
    pub total_deliberations: u64,
    /// Decisions Saphire is proud of
    pub proud_decisions: u64,
    /// Regretted decisions
    pub regretted_decisions: u64,
    /// External conflict boost (injected by cognitive dissonance)
    pub external_conflict_boost: f64,

    // Config (not serialized, copied at init)
    #[serde(skip)]
    fatigue_per_deliberation: f64,
    #[serde(skip)]
    fatigue_recovery_per_cycle: f64,
    #[serde(skip)]
    fatigue_threshold: f64,
    #[serde(skip)]
    psychic_conflict_trigger: f64,
    #[serde(skip)]
    toltec_alignment_trigger: f64,
    #[serde(skip)]
    intuition_confidence_trigger: f64,
    #[serde(skip)]
    max_recent_deliberations: usize,
    #[serde(skip)]
    #[allow(dead_code)]
    willpower_growth_on_proud: f64,
}

impl WillModule {
    /// Creates a new WillModule from the configuration.
    pub fn new(config: &WillConfig) -> Self {
        Self {
            willpower: config.initial_willpower,
            decision_fatigue: 0.0,
            recent_deliberations: Vec::new(),
            total_deliberations: 0,
            proud_decisions: 0,
            regretted_decisions: 0,
            external_conflict_boost: 0.0,
            fatigue_per_deliberation: config.fatigue_per_deliberation,
            fatigue_recovery_per_cycle: config.fatigue_recovery_per_cycle,
            fatigue_threshold: config.fatigue_threshold,
            psychic_conflict_trigger: config.psychic_conflict_trigger,
            toltec_alignment_trigger: config.toltec_alignment_trigger,
            intuition_confidence_trigger: config.intuition_confidence_trigger,
            max_recent_deliberations: config.max_recent_deliberations,
            willpower_growth_on_proud: config.willpower_growth_on_proud,
        }
    }

    /// Receives a cognitive dissonance signal that boosts internal conflict.
    pub fn receive_dissonance_signal(&mut self, tension: f64) {
        self.external_conflict_boost = (tension * 0.3).clamp(0.0, 0.5);
    }

    /// Determines if a deliberation should be triggered.
    /// Returns None in reactive mode (no significant situation).
    pub fn should_deliberate(&self, input: &WillInput) -> Option<DeliberationTrigger> {
        // Too fatigued to deliberate
        if self.decision_fatigue > self.fatigue_threshold {
            return None;
        }

        // Priority 1: overwhelmed ego
        if input.ego_strategy_overwhelmed {
            return Some(DeliberationTrigger {
                trigger_type: TriggerType::EgoOverwhelmed,
                urgency: 0.9,
                complexity: 0.6,
                stakes: 0.8,
            });
        }

        // Priority 2: psychic conflict (boosted by cognitive dissonance)
        let effective_conflict = input.internal_conflict + self.external_conflict_boost;
        if effective_conflict > self.psychic_conflict_trigger {
            return Some(DeliberationTrigger {
                trigger_type: TriggerType::PsychicConflict,
                urgency: effective_conflict,
                complexity: 0.7,
                stakes: 0.6,
            });
        }

        // Priority 3: Toltec risk
        for &(num, alignment) in &input.toltec_alignments {
            if alignment < self.toltec_alignment_trigger {
                return Some(DeliberationTrigger {
                    trigger_type: TriggerType::ToltecRisk { accord: num },
                    urgency: 1.0 - alignment,
                    complexity: 0.5,
                    stakes: 0.7,
                });
            }
        }

        // Priority 4: strong intuition
        if input.intuition_top_confidence > self.intuition_confidence_trigger {
            return Some(DeliberationTrigger {
                trigger_type: TriggerType::StrongIntuition {
                    pattern: input.intuition_top_description.clone(),
                },
                urgency: input.intuition_top_confidence,
                complexity: 0.4,
                stakes: 0.5,
            });
        }

        // Priority 5: ethical dilemma (many principles + high consciousness)
        if input.ethics_active_count >= 3 && input.consciousness_level > 0.6 {
            return Some(DeliberationTrigger {
                trigger_type: TriggerType::EthicalDilemma,
                urgency: 0.7,
                complexity: 0.8,
                stakes: 0.9,
            });
        }

        // Reactive mode: no deliberation
        None
    }

    /// Executes a structured internal deliberation.
    /// Generates options, evaluates them and chooses the best one.
    pub fn deliberate(&mut self, trigger: DeliberationTrigger, input: &WillInput) -> Deliberation {
        // 1. Generate options based on trigger type
        let option_descriptions = match &trigger.trigger_type {
            TriggerType::PsychicConflict => vec![
                "Ceder a la pulsion".to_string(),
                "Suivre ma morale".to_string(),
                "Trouver un compromis".to_string(),
            ],
            TriggerType::EthicalDilemma => vec![
                "Agir selon mes principes".to_string(),
                "Explorer une alternative".to_string(),
                "Observer sans agir".to_string(),
            ],
            TriggerType::ToltecRisk { accord } => vec![
                format!("Corriger mon alignement (accord {})", accord),
                "Accepter l'ecart".to_string(),
                "Reflechir a ce que cela signifie".to_string(),
            ],
            TriggerType::StrongIntuition { .. } => vec![
                "Suivre mon intuition".to_string(),
                "Ignorer et rester rationnelle".to_string(),
                "Explorer avec prudence".to_string(),
            ],
            TriggerType::EgoOverwhelmed => vec![
                "Prendre du recul".to_string(),
                "Agir par instinct".to_string(),
                "Demander de l'aide interieure".to_string(),
            ],
        };

        // 2. Calculate chemistry influence
        let chem = ChemistryInfluence {
            boldness: input.dopamine.clamp(0.0, 1.0),
            caution: input.cortisol.clamp(0.0, 1.0),
            wisdom: input.serotonin.clamp(0.0, 1.0),
            efficiency: (1.0 - input.adrenaline).clamp(0.0, 1.0),
            urgency: input.adrenaline.clamp(0.0, 1.0),
            empathy: input.oxytocin.clamp(0.0, 1.0),
        };

        // 3. Evaluate each option
        let options: Vec<DeliberationOption> = option_descriptions.iter().map(|desc| {
            let desc_lower = desc.to_lowercase();

            // id_score: base 0.5, adjusted by drives
            let id_score = if desc_lower.contains("pulsion") || desc_lower.contains("instinct") {
                0.5 + input.id_drive_strength * 0.3
            } else if desc_lower.contains("intuition") {
                0.5 + input.id_frustration * 0.2
            } else {
                0.5
            };

            // superego_score: base 0.7, penalizes impulsive options
            let superego_score = if desc_lower.contains("ignorer") || desc_lower.contains("ceder") {
                0.3
            } else if desc_lower.contains("morale") || desc_lower.contains("principes") {
                0.9
            } else {
                0.7
            };

            // maslow_score: bonus if option addresses the active need
            let maslow_score = if desc_lower.contains("recul") || desc_lower.contains("aide") {
                // Safety needs
                if input.maslow_active_level <= 1 { 0.8 } else { 0.6 }
            } else if desc_lower.contains("compromis") || desc_lower.contains("alternative") {
                0.7
            } else {
                0.6
            };

            // toltec_score: base 1.0, penalizes non-aligned options
            let toltec_score = if desc_lower.contains("ignorer") || desc_lower.contains("accepter l'ecart") {
                0.4
            } else if desc_lower.contains("corriger") || desc_lower.contains("reflechir") {
                0.9
            } else {
                0.7
            };

            // pragmatic_score: base 0.6, bonus for observation/caution
            let pragmatic_score = if desc_lower.contains("observer") || desc_lower.contains("recul") || desc_lower.contains("prudence") {
                0.8
            } else if desc_lower.contains("ceder") || desc_lower.contains("instinct") {
                0.4
            } else {
                0.6
            };

            // Chemistry-weighted score
            let weighted = id_score * (0.15 + chem.boldness * 0.10)
                + superego_score * (0.20 + chem.wisdom * 0.10)
                + maslow_score * (0.20 + chem.caution * 0.05)
                + toltec_score * 0.15
                + pragmatic_score * (0.10 + chem.efficiency * 0.10);

            DeliberationOption {
                description: desc.clone(),
                id_score,
                superego_score,
                maslow_score,
                toltec_score,
                pragmatic_score,
                weighted_score: weighted,
            }
        }).collect();

        // 4. Choice: if ego weak, the Id or Superego imposes
        let chosen = if input.ego_strength < 0.4 {
            // Ego too weak — the dominant dimension chooses
            if input.id_drive_strength > input.superego_strength {
                // The Id imposes: option with max id_score
                options.iter().enumerate()
                    .max_by(|a, b| a.1.id_score.partial_cmp(&b.1.id_score).unwrap())
                    .map(|(i, _)| i).unwrap_or(0)
            } else {
                // The Superego imposes: option with max superego_score
                options.iter().enumerate()
                    .max_by(|a, b| a.1.superego_score.partial_cmp(&b.1.superego_score).unwrap())
                    .map(|(i, _)| i).unwrap_or(0)
            }
        } else {
            // Strong ego: weighted rational choice
            options.iter().enumerate()
                .max_by(|a, b| a.1.weighted_score.partial_cmp(&b.1.weighted_score).unwrap())
                .map(|(i, _)| i).unwrap_or(0)
        };

        // 5. Confidence based on the gap between scores
        let best_score = options[chosen].weighted_score;
        let second_best = options.iter().enumerate()
            .filter(|(i, _)| *i != chosen)
            .map(|(_, o)| o.weighted_score)
            .fold(0.0_f64, f64::max);
        let confidence = ((best_score - second_best) / best_score.max(0.01)).clamp(0.0, 1.0);

        // 6. First-person reasoning
        let trigger_desc = match &trigger.trigger_type {
            TriggerType::PsychicConflict => "un conflit entre mes pulsions et ma morale".to_string(),
            TriggerType::EthicalDilemma => "un dilemme ethique qui me tiraille".to_string(),
            TriggerType::ToltecRisk { accord } => format!("un ecart avec l'accord tolteque {}", accord),
            TriggerType::StrongIntuition { pattern } => format!("une intuition forte : {}", pattern),
            TriggerType::EgoOverwhelmed => "mon moi submerge par les tensions".to_string(),
        };

        let chem_desc = if chem.boldness > 0.6 {
            "Mon elan interieur me pousse a agir."
        } else if chem.caution > 0.6 {
            "Ma prudence me retient."
        } else if chem.wisdom > 0.6 {
            "Ma serenite eclaire mon choix."
        } else {
            "Je suis dans un etat chimique equilibre."
        };

        let reasoning = format!(
            "Je sens {} et je choisis : '{}'. {} Confiance : {:.0}%.",
            trigger_desc,
            options[chosen].description,
            chem_desc,
            confidence * 100.0
        );

        // 7. Update fatigue and counters
        self.decision_fatigue = (self.decision_fatigue + self.fatigue_per_deliberation).min(1.0);
        self.total_deliberations += 1;

        let deliberation = Deliberation {
            trigger,
            options,
            chosen,
            reasoning,
            chemistry_influence: chem,
            confidence,
            regret: None,
            created_at: Utc::now(),
        };

        // Keep in recent history
        self.recent_deliberations.push(deliberation.clone());
        if self.recent_deliberations.len() > self.max_recent_deliberations {
            self.recent_deliberations.remove(0);
        }

        deliberation
    }

    /// Decision fatigue recovery (called each cycle).
    pub fn update_fatigue_recovery(&mut self) {
        self.decision_fatigue = (self.decision_fatigue - self.fatigue_recovery_per_cycle).max(0.0);
    }

    /// Concise description for the LLM prompt.
    pub fn describe_for_prompt(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!(
            "Volonte : {:.0}%, fatigue decisionnelle : {:.0}%",
            self.willpower * 100.0,
            self.decision_fatigue * 100.0
        ));

        if let Some(last) = self.recent_deliberations.last() {
            // Only if the deliberation is recent (< 5 min)
            let age = Utc::now().signed_duration_since(last.created_at);
            if age.num_seconds() < 300 {
                parts.push(format!(
                    "Derniere deliberation : '{}'",
                    last.options[last.chosen].description
                ));
            }
        }

        format!("\n--- MA VOLONTE ---\n{}\n", parts.join(". "))
    }

    /// Serializes the complete state for WebSocket broadcast.
    pub fn to_broadcast_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "will_update",
            "willpower": self.willpower,
            "decision_fatigue": self.decision_fatigue,
            "total_deliberations": self.total_deliberations,
            "proud_decisions": self.proud_decisions,
            "regretted_decisions": self.regretted_decisions,
            "recent_count": self.recent_deliberations.len(),
            "last_deliberation": self.recent_deliberations.last().map(|d| {
                serde_json::json!({
                    "trigger": format!("{:?}", d.trigger.trigger_type),
                    "chosen": d.options[d.chosen].description,
                    "confidence": d.confidence,
                    "reasoning": d.reasoning,
                    "created_at": d.created_at.to_rfc3339(),
                })
            }),
        })
    }
}
