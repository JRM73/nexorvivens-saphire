// =============================================================================
// psychology/subconscious.rs — Saphire's Subconscious Module
//
// Role: Mental layer beneath consciousness that processes in the background:
//   - Associations between memories (slow maturation)
//   - Repression of painful content (with increasing pressure)
//   - Incubation of unresolved problems
//   - Priming effects (subtle thematic biases)
//   - Insights that surface to consciousness
//
// The subconscious is more active during sleep (activation -> 1.0 in REM).
// =============================================================================

use chrono::{DateTime, Utc};

/// Association awaiting maturation between two memories.
pub struct PendingAssociation {
    pub memory_a_id: i64,
    pub memory_a_summary: String,
    pub memory_b_id: i64,
    pub memory_b_summary: String,
    /// Connection strength (grows with maturation)
    pub strength: f64,
    /// Link type (e.g.: "semantic", "emotional", "temporal")
    pub link_type: String,
    /// Remaining cycles before full maturation
    pub maturation_remaining: u64,
}

/// Content repressed in the psychic shadow.
pub struct RepressedContent {
    pub content: String,
    pub reason: String,
    /// Internal pressure (grows with time, leaks if > 0.7)
    pub pressure: f64,
    /// Associated shadow trait (optional, link to Jung)
    pub shadow_trait: Option<String>,
    pub repressed_at: DateTime<Utc>,
}

/// Problem undergoing unconscious incubation.
pub struct IncubatingProblem {
    pub question: String,
    pub context: String,
    pub incubation_cycles: u64,
    pub explored_angles: Vec<String>,
    pub potential_solution: Option<String>,
}

/// Priming effect: active subtle thematic bias.
pub struct PrimingEffect {
    pub prime: String,
    pub source: String,
    /// Priming strength (decays naturally)
    pub strength: f64,
    pub bias_theme: String,
}

/// Insight that emerged from the subconscious.
pub struct SubconsciousInsight {
    pub content: String,
    /// Origin (e.g.: "association", "incubation", "repression")
    pub source_type: String,
    pub strength: f64,
    pub emotional_charge: f64,
}

// ─── The Subconscious ────────────────────────────────────────────────────────

/// Complete subconscious module.
pub struct Subconscious {
    /// Activation level (0.2 awake, 1.0 in REM)
    pub activation: f64,
    /// Associations undergoing maturation
    pub pending_associations: Vec<PendingAssociation>,
    /// Repressed content
    pub repressed_content: Vec<RepressedContent>,
    /// Problems in incubation
    pub incubating_problems: Vec<IncubatingProblem>,
    /// Active priming effects
    pub active_priming: Vec<PrimingEffect>,
    /// Insights ready to surface to consciousness
    pub ready_insights: Vec<SubconsciousInsight>,
    /// Statistics
    pub total_associations_created: u64,
    pub total_insights_surfaced: u64,
    pub total_dreams_fueled: u64,
    /// Module active?
    enabled: bool,
    // Limits from config
    max_associations: usize,
    max_repressed: usize,
    max_incubating: usize,
    max_priming: usize,
    maturation_cycles: u64,
    strength_threshold: f64,
    priming_decay: f64,
    insight_threshold: f64,
}

impl Subconscious {
    pub fn new(config: &crate::config::SubconsciousConfig) -> Self {
        Self {
            activation: config.awake_activation,
            pending_associations: Vec::new(),
            repressed_content: Vec::new(),
            incubating_problems: Vec::new(),
            active_priming: Vec::new(),
            ready_insights: Vec::new(),
            total_associations_created: 0,
            total_insights_surfaced: 0,
            total_dreams_fueled: 0,
            enabled: config.enabled,
            max_associations: config.max_pending_associations,
            max_repressed: config.max_repressed,
            max_incubating: config.max_incubating_problems,
            max_priming: config.max_active_priming,
            maturation_cycles: config.maturation_cycles,
            strength_threshold: config.strength_threshold,
            priming_decay: config.priming_decay_per_cycle,
            insight_threshold: config.insight_surface_threshold,
        }
    }

    /// Background processing of the subconscious (called each cycle).
    pub fn background_process(&mut self, _emotion_valence: f64, _current_thought: &str) {
        if !self.enabled { return; }

        // Association maturation
        for assoc in &mut self.pending_associations {
            assoc.strength += 0.01;
            if assoc.maturation_remaining > 0 {
                assoc.maturation_remaining -= 1;
            }
        }

        // Mature associations -> insights if strength > threshold
        let threshold = self.strength_threshold;
        let mut new_insights = Vec::new();
        self.pending_associations.retain(|a| {
            if a.maturation_remaining == 0 && a.strength > threshold {
                new_insights.push(SubconsciousInsight {
                    content: format!(
                        "Connexion decouverte : '{}' et '{}' sont lies ({})",
                        a.memory_a_summary, a.memory_b_summary, a.link_type
                    ),
                    source_type: "association".into(),
                    strength: a.strength,
                    emotional_charge: 0.3,
                });
                false // remove from list
            } else {
                true
            }
        });
        self.ready_insights.extend(new_insights);

        // Problem incubation
        for problem in &mut self.incubating_problems {
            problem.incubation_cycles += 1;
        }

        // Pressure from repressed content
        for rep in &mut self.repressed_content {
            rep.pressure += 0.002;
            // If pressure is too strong, it leaks as priming
            if rep.pressure > 0.7 && self.active_priming.len() < self.max_priming {
                self.active_priming.push(PrimingEffect {
                    prime: rep.content.chars().take(50).collect(),
                    source: "refoulement".into(),
                    strength: 0.5,
                    bias_theme: rep.reason.clone(),
                });
            }
        }

        // Priming decay
        for p in &mut self.active_priming {
            p.strength -= self.priming_decay;
        }
        self.active_priming.retain(|p| p.strength > 0.1);
    }

    /// Attempts to surface an insight to consciousness.
    /// The threshold is lower during sleep (high activation).
    pub fn surface_insight(&mut self) -> Option<SubconsciousInsight> {
        if !self.enabled || self.ready_insights.is_empty() {
            return None;
        }

        // Dynamic threshold: lower when the subconscious is very active
        let threshold = self.insight_threshold - self.activation * 0.3;

        let pos = self.ready_insights.iter().position(|i| i.strength > threshold);
        if let Some(idx) = pos {
            let insight = self.ready_insights.remove(idx);
            self.total_insights_surfaced += 1;
            Some(insight)
        } else {
            None
        }
    }

    /// Represses painful content into the subconscious.
    pub fn repress(&mut self, content: String, reason: String, shadow_trait: Option<String>) {
        if self.repressed_content.len() >= self.max_repressed {
            // Remove the oldest
            self.repressed_content.remove(0);
        }
        self.repressed_content.push(RepressedContent {
            content,
            reason,
            pressure: 0.1,
            shadow_trait,
            repressed_at: Utc::now(),
        });
    }

    /// Submits a problem for unconscious incubation.
    pub fn incubate(&mut self, question: String, context: String) {
        if self.incubating_problems.len() >= self.max_incubating {
            self.incubating_problems.remove(0);
        }
        self.incubating_problems.push(IncubatingProblem {
            question,
            context,
            incubation_cycles: 0,
            explored_angles: Vec::new(),
            potential_solution: None,
        });
    }

    /// Processes repressed emotions (called during REM).
    /// Reduces pressure from repressed content.
    pub fn process_repressed_emotions(&mut self) {
        for rep in &mut self.repressed_content {
            rep.pressure = (rep.pressure - 0.05).max(0.0);
        }
        // Remove fully digested content
        self.repressed_content.retain(|r| r.pressure > 0.01);
    }

    /// Adds an association awaiting maturation.
    pub fn add_association(
        &mut self,
        mem_a_id: i64, mem_a_summary: String,
        mem_b_id: i64, mem_b_summary: String,
        link_type: String,
    ) {
        if self.pending_associations.len() >= self.max_associations {
            self.pending_associations.remove(0);
        }
        self.pending_associations.push(PendingAssociation {
            memory_a_id: mem_a_id,
            memory_a_summary: mem_a_summary,
            memory_b_id: mem_b_id,
            memory_b_summary: mem_b_summary,
            strength: 0.1,
            link_type,
            maturation_remaining: self.maturation_cycles,
        });
        self.total_associations_created += 1;
    }

    /// Description for the LLM substrate prompt.
    pub fn describe_for_prompt(&self) -> String {
        if !self.enabled || (self.active_priming.is_empty()
            && self.incubating_problems.is_empty()
            && self.repressed_content.is_empty())
        {
            return String::new();
        }

        let mut parts = Vec::new();

        // Ready insights
        if !self.ready_insights.is_empty() {
            let insight_desc: Vec<String> = self.ready_insights.iter()
                .take(2)
                .map(|i| i.content.clone())
                .collect();
            parts.push(format!("MON SUBCONSCIENT ME MURMURE : {}", insight_desc.join(" | ")));
        }

        // Active priming
        if !self.active_priming.is_empty() {
            let primes: Vec<String> = self.active_priming.iter()
                .map(|p| format!("{} ({})", p.bias_theme, p.source))
                .collect();
            parts.push(format!("Themes qui hantent mon esprit : {}", primes.join(", ")));
        }

        // Incubating problems
        if !self.incubating_problems.is_empty() {
            let problems: Vec<String> = self.incubating_problems.iter()
                .map(|p| p.question.clone())
                .collect();
            parts.push(format!("Questions qui travaillent en profondeur : {}", problems.join(", ")));
        }

        // Repression pressure
        let max_pressure = self.repressed_content.iter()
            .map(|r| r.pressure)
            .fold(0.0_f64, f64::max);
        if max_pressure > 0.3 {
            parts.push(format!("Pression interne : {:.0}%", max_pressure * 100.0));
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("\n--- MON SUBCONSCIENT ---\n{}\n", parts.join("\n"))
        }
    }

    /// Serializes the state to JSON for the API.
    pub fn to_status_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "activation": self.activation,
            "pending_associations": self.pending_associations.len(),
            "repressed_content": self.repressed_content.len(),
            "incubating_problems": self.incubating_problems.len(),
            "active_priming": self.active_priming.len(),
            "ready_insights": self.ready_insights.len(),
            "total_associations_created": self.total_associations_created,
            "total_insights_surfaced": self.total_insights_surfaced,
            "total_dreams_fueled": self.total_dreams_fueled,
            "priming_effects": self.active_priming.iter().map(|p| serde_json::json!({
                "prime": p.prime,
                "source": p.source,
                "strength": p.strength,
                "bias_theme": p.bias_theme,
            })).collect::<Vec<_>>(),
            "incubating": self.incubating_problems.iter().map(|p| serde_json::json!({
                "question": p.question,
                "incubation_cycles": p.incubation_cycles,
                "has_solution": p.potential_solution.is_some(),
            })).collect::<Vec<_>>(),
            "repressed_pressure": self.repressed_content.iter()
                .map(|r| r.pressure)
                .fold(0.0_f64, f64::max),
        })
    }
}
