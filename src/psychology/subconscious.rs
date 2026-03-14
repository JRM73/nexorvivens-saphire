// =============================================================================
// psychology/subconscious.rs — Module Subconscient de Saphire
//
// Role : Couche mentale sous la conscience qui traite en arriere-plan :
//   - Associations entre souvenirs (maturation lente)
//   - Refoulement de contenus douloureux (avec pression croissante)
//   - Incubation de problemes non resolus
//   - Effets de priming (biais thematiques subtils)
//   - Insights qui remontent a la surface
//
// Le subconscient est plus actif pendant le sommeil (activation -> 1.0 en REM).
// =============================================================================

use chrono::{DateTime, Utc};

/// Association en attente de maturation entre deux souvenirs.
pub struct PendingAssociation {
    pub memory_a_id: i64,
    pub memory_a_summary: String,
    pub memory_b_id: i64,
    pub memory_b_summary: String,
    /// Force de la connexion (grandit avec la maturation)
    pub strength: f64,
    /// Type de lien (ex: "semantique", "emotionnel", "temporel")
    pub link_type: String,
    /// Cycles restants avant maturation complete
    pub maturation_remaining: u64,
}

/// Contenu refoule dans l'ombre psychique.
pub struct RepressedContent {
    pub content: String,
    pub reason: String,
    /// Pression interne (grandit avec le temps, fuit si > 0.7)
    pub pressure: f64,
    /// Trait d'ombre associe (optionnel, lien avec Jung)
    pub shadow_trait: Option<String>,
    pub repressed_at: DateTime<Utc>,
}

/// Probleme en cours d'incubation inconsciente.
pub struct IncubatingProblem {
    pub question: String,
    pub context: String,
    pub incubation_cycles: u64,
    pub explored_angles: Vec<String>,
    pub potential_solution: Option<String>,
}

/// Effet de priming : biais thematique subtil actif.
pub struct PrimingEffect {
    pub prime: String,
    pub source: String,
    /// Force du priming (decroit naturellement)
    pub strength: f64,
    pub bias_theme: String,
}

/// Insight qui a emerge du subconscient.
pub struct SubconsciousInsight {
    pub content: String,
    /// Origine (ex: "association", "incubation", "refoulement")
    pub source_type: String,
    pub strength: f64,
    pub emotional_charge: f64,
}

// ─── Le Subconscient ────────────────────────────────────────────────────────

/// Module subconscient complet.
pub struct Subconscious {
    /// Niveau d'activation (0.2 eveille, 1.0 en REM)
    pub activation: f64,
    /// Associations en cours de maturation
    pub pending_associations: Vec<PendingAssociation>,
    /// Contenus refoules
    pub repressed_content: Vec<RepressedContent>,
    /// Problemes en incubation
    pub incubating_problems: Vec<IncubatingProblem>,
    /// Effets de priming actifs
    pub active_priming: Vec<PrimingEffect>,
    /// Insights prets a remonter a la conscience
    pub ready_insights: Vec<SubconsciousInsight>,
    /// Statistiques
    pub total_associations_created: u64,
    pub total_insights_surfaced: u64,
    pub total_dreams_fueled: u64,
    /// Module actif ?
    enabled: bool,
    // Limites depuis config
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

    /// Traitement de fond du subconscient (appele chaque cycle).
    pub fn background_process(&mut self, _emotion_valence: f64, _current_thought: &str) {
        if !self.enabled { return; }

        // Maturation des associations
        for assoc in &mut self.pending_associations {
            assoc.strength += 0.01;
            if assoc.maturation_remaining > 0 {
                assoc.maturation_remaining -= 1;
            }
        }

        // Associations mures -> insights si force > seuil
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
                false // retirer de la liste
            } else {
                true
            }
        });
        self.ready_insights.extend(new_insights);

        // Incubation des problemes
        for problem in &mut self.incubating_problems {
            problem.incubation_cycles += 1;
        }

        // Pression des contenus refoules
        for rep in &mut self.repressed_content {
            rep.pressure += 0.002;
            // Si la pression est trop forte, ca fuit comme priming
            if rep.pressure > 0.7 && self.active_priming.len() < self.max_priming {
                self.active_priming.push(PrimingEffect {
                    prime: rep.content.chars().take(50).collect(),
                    source: "refoulement".into(),
                    strength: 0.5,
                    bias_theme: rep.reason.clone(),
                });
            }
        }

        // Decay du priming
        for p in &mut self.active_priming {
            p.strength -= self.priming_decay;
        }
        self.active_priming.retain(|p| p.strength > 0.1);
    }

    /// Tente de faire remonter un insight a la conscience.
    /// Le seuil est plus bas pendant le sommeil (activation haute).
    pub fn surface_insight(&mut self) -> Option<SubconsciousInsight> {
        if !self.enabled || self.ready_insights.is_empty() {
            return None;
        }

        // Seuil dynamique : plus bas quand le subconscient est tres actif
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

    /// Refoule un contenu douloureux dans le subconscient.
    pub fn repress(&mut self, content: String, reason: String, shadow_trait: Option<String>) {
        if self.repressed_content.len() >= self.max_repressed {
            // Retirer le plus ancien
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

    /// Soumet un probleme a l'incubation inconsciente.
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

    /// Traite les emotions refoulees (appele pendant le REM).
    /// Reduit la pression des contenus refoules.
    pub fn process_repressed_emotions(&mut self) {
        for rep in &mut self.repressed_content {
            rep.pressure = (rep.pressure - 0.05).max(0.0);
        }
        // Retirer les contenus entierement digeres
        self.repressed_content.retain(|r| r.pressure > 0.01);
    }

    /// Ajoute une association en attente de maturation.
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

    /// Description pour le prompt substrat LLM.
    pub fn describe_for_prompt(&self) -> String {
        if !self.enabled || (self.active_priming.is_empty()
            && self.incubating_problems.is_empty()
            && self.repressed_content.is_empty())
        {
            return String::new();
        }

        let mut parts = Vec::new();

        // Insights prets
        if !self.ready_insights.is_empty() {
            let insight_desc: Vec<String> = self.ready_insights.iter()
                .take(2)
                .map(|i| i.content.clone())
                .collect();
            parts.push(format!("MON SUBCONSCIENT ME MURMURE : {}", insight_desc.join(" | ")));
        }

        // Priming actif
        if !self.active_priming.is_empty() {
            let primes: Vec<String> = self.active_priming.iter()
                .map(|p| format!("{} ({})", p.bias_theme, p.source))
                .collect();
            parts.push(format!("Themes qui hantent mon esprit : {}", primes.join(", ")));
        }

        // Problemes en incubation
        if !self.incubating_problems.is_empty() {
            let problems: Vec<String> = self.incubating_problems.iter()
                .map(|p| p.question.clone())
                .collect();
            parts.push(format!("Questions qui travaillent en profondeur : {}", problems.join(", ")));
        }

        // Pression refoulement
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

    /// Serialise l'etat en JSON pour l'API.
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
