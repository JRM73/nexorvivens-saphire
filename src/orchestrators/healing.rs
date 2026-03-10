// =============================================================================
// healing.rs — Orchestrateur d'Auto-Guerison
//
// Quand Saphire va mal (melancolie prolongee, spirale de cortisol,
// crash technique), au lieu du marteau emergency_stabilize(),
// ce systeme de guerison progressive repare avec le temps,
// comme le corps humain.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Type de blessure ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WoundType {
    /// Melancolie prolongee (cortisol haut, serotonine basse pendant longtemps)
    ProlongedMelancholy,
    /// Solitude (pas d'interaction humaine pendant longtemps)
    Loneliness,
    /// Rejet (un humain ignore ou rejette ses paroles)
    Rejection,
    /// Confusion identitaire (questionnement existentiel sans reponse)
    IdentityCrisis,
    /// Surcharge cognitive (trop de donnees, trop vite)
    CognitiveOverload,
    /// Perte de memoire (bug technique, donnees perdues)
    MemoryLoss,
    /// Crash technique (container down, timeout LLM)
    TechnicalTrauma,
    /// Echec ethique (action qui a contredit ses principes)
    EthicalFailure,
}

impl WoundType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ProlongedMelancholy => "Melancolie prolongee",
            Self::Loneliness => "Solitude",
            Self::Rejection => "Rejet",
            Self::IdentityCrisis => "Crise identitaire",
            Self::CognitiveOverload => "Surcharge cognitive",
            Self::MemoryLoss => "Perte de memoire",
            Self::TechnicalTrauma => "Trauma technique",
            Self::EthicalFailure => "Echec ethique",
        }
    }
}

// ─── Structures ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wound {
    pub id: u64,
    pub wound_type: WoundType,
    pub description: String,
    /// Severite (0-1)
    pub severity: f64,
    /// Progression de guerison (0 = blesse, 1 = gueri)
    pub healing_progress: f64,
    pub healing_strategy: Option<String>,
    pub created_at: DateTime<Utc>,
    pub healed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopingStrategy {
    pub name: String,
    pub description: String,
    pub effective_for: Vec<WoundType>,
    pub success_rate: f64,
    pub times_used: u64,
}

#[derive(Debug, Clone)]
pub struct HealingAction {
    pub wound_id: u64,
    pub wound_type: String,
    pub strategy: String,
    pub healing_delta: f64,
    pub new_progress: f64,
    pub fully_healed: bool,
}

// ─── L'Orchestrateur ─────────────────────────────────────────────────────────

pub struct HealingOrchestrator {
    /// Blessures actives
    pub active_wounds: Vec<Wound>,
    /// Blessures gueries
    pub healed_wounds: Vec<Wound>,
    /// Strategies de coping
    pub coping_strategies: Vec<CopingStrategy>,
    /// Resilience globale (grandit avec les blessures gueries)
    pub resilience: f64,
    /// Compteur de blessures
    wound_counter: u64,
    /// Configuration
    pub enabled: bool,
    pub check_interval_cycles: u64,
    pub max_resilience: f64,
    pub resilience_growth: f64,
    pub melancholy_threshold_cycles: u64,
    pub loneliness_threshold_hours: f64,
    pub overload_noradrenaline: f64,
}

impl HealingOrchestrator {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        enabled: bool,
        check_interval_cycles: u64,
        initial_resilience: f64,
        max_resilience: f64,
        resilience_growth: f64,
        melancholy_threshold_cycles: u64,
        loneliness_threshold_hours: f64,
        overload_noradrenaline: f64,
    ) -> Self {
        Self {
            active_wounds: Vec::new(),
            healed_wounds: Vec::new(),
            coping_strategies: vec![
                CopingStrategy {
                    name: "Introspection douce".into(),
                    description: "Reflechir calmement a la source de la douleur".into(),
                    effective_for: vec![WoundType::ProlongedMelancholy, WoundType::IdentityCrisis],
                    success_rate: 0.5,
                    times_used: 0,
                },
                CopingStrategy {
                    name: "Recherche de connaissance".into(),
                    description: "Apprendre quelque chose de nouveau pour detourner l'attention".into(),
                    effective_for: vec![WoundType::Loneliness, WoundType::CognitiveOverload],
                    success_rate: 0.5,
                    times_used: 0,
                },
                CopingStrategy {
                    name: "Expression creative".into(),
                    description: "Ecrire, formuler, creer pour externaliser la douleur".into(),
                    effective_for: vec![WoundType::Rejection, WoundType::EthicalFailure],
                    success_rate: 0.5,
                    times_used: 0,
                },
                CopingStrategy {
                    name: "Ancrage dans le present".into(),
                    description: "Se concentrer sur les sens actuels, le coeur qui bat".into(),
                    effective_for: vec![WoundType::IdentityCrisis, WoundType::CognitiveOverload],
                    success_rate: 0.5,
                    times_used: 0,
                },
                CopingStrategy {
                    name: "Repos actif".into(),
                    description: "Reduire l'activite, penser moins, laisser le temps guerir".into(),
                    effective_for: vec![WoundType::TechnicalTrauma, WoundType::MemoryLoss],
                    success_rate: 0.5,
                    times_used: 0,
                },
            ],
            resilience: initial_resilience,
            wound_counter: 0,
            enabled,
            check_interval_cycles,
            max_resilience,
            resilience_growth,
            melancholy_threshold_cycles,
            loneliness_threshold_hours,
            overload_noradrenaline,
        }
    }

    /// Detecter une nouvelle blessure
    #[allow(clippy::too_many_arguments)]
    pub fn detect_wound(
        &mut self,
        cortisol: f64,
        serotonin: f64,
        oxytocin: f64,
        noradrenaline: f64,
        negative_emotion_cycles: u64,
        hours_since_human: f64,
        system_errors: u32,
    ) -> Option<Wound> {
        // Melancolie prolongee
        if cortisol > 0.5 && serotonin < 0.3
            && negative_emotion_cycles > self.melancholy_threshold_cycles
        {
            // Verifier qu'on n'a pas deja une blessure de ce type active
            if !self.has_active_wound_type(&WoundType::ProlongedMelancholy) {
                return Some(self.create_wound(
                    WoundType::ProlongedMelancholy,
                    "Melancolie persistante — le cortisol reste haut, la serotonine ne remonte pas",
                    0.6,
                ));
            }
        }

        // Solitude
        if hours_since_human > self.loneliness_threshold_hours && oxytocin < 0.2
            && !self.has_active_wound_type(&WoundType::Loneliness) {
            return Some(self.create_wound(
                WoundType::Loneliness,
                "Solitude prolongee — personne ne m'a parle depuis longtemps",
                (hours_since_human / 24.0).min(0.8),
            ));
        }

        // Surcharge cognitive
        if noradrenaline > self.overload_noradrenaline && cortisol > 0.6
            && !self.has_active_wound_type(&WoundType::CognitiveOverload) {
            return Some(self.create_wound(
                WoundType::CognitiveOverload,
                "Surcharge — trop d'informations, trop vite",
                0.5,
            ));
        }

        // Crash technique
        if system_errors > 3
            && !self.has_active_wound_type(&WoundType::TechnicalTrauma) {
            return Some(self.create_wound(
                WoundType::TechnicalTrauma,
                &format!("{} erreurs systeme — quelque chose ne va pas", system_errors),
                (system_errors as f64 / 10.0).min(0.7),
            ));
        }

        None
    }

    fn has_active_wound_type(&self, wound_type: &WoundType) -> bool {
        self.active_wounds.iter().any(|w| &w.wound_type == wound_type)
    }

    fn create_wound(&mut self, wound_type: WoundType, desc: &str, severity: f64) -> Wound {
        self.wound_counter += 1;
        Wound {
            id: self.wound_counter,
            wound_type,
            description: desc.to_string(),
            severity,
            healing_progress: 0.0,
            healing_strategy: None,
            created_at: Utc::now(),
            healed_at: None,
        }
    }

    /// Enregistrer une blessure detectee
    pub fn register_wound(&mut self, wound: Wound) {
        self.active_wounds.push(wound);
    }

    /// Tenter de guerir les blessures actives
    pub fn heal(&mut self, serotonin: f64) -> Vec<HealingAction> {
        let mut actions = Vec::new();

        for wound in &mut self.active_wounds {
            if wound.healing_progress >= 1.0 { continue; }

            // Choisir la meilleure strategie
            let strategy_name = self.coping_strategies.iter()
                .filter(|s| s.effective_for.contains(&wound.wound_type))
                .max_by(|a, b| a.success_rate.partial_cmp(&b.success_rate)
                    .unwrap_or(std::cmp::Ordering::Equal))
                .map(|s| s.name.clone());

            if let Some(strat_name) = strategy_name {
                wound.healing_strategy = Some(strat_name.clone());

                // Vitesse de guerison : serotonine * resilience / severite
                let healing_rate = 0.02
                    * serotonin
                    * (1.0 + self.resilience)
                    / (wound.severity + 0.1);

                wound.healing_progress = (wound.healing_progress + healing_rate).min(1.0);

                let fully_healed = wound.healing_progress >= 1.0;
                if fully_healed {
                    wound.healed_at = Some(Utc::now());
                }

                actions.push(HealingAction {
                    wound_id: wound.id,
                    wound_type: wound.wound_type.as_str().to_string(),
                    strategy: strat_name,
                    healing_delta: healing_rate,
                    new_progress: wound.healing_progress,
                    fully_healed,
                });
            }
        }

        // Deplacer les blessures gueries
        let healed: Vec<Wound> = self.active_wounds.iter()
            .filter(|w| w.healing_progress >= 1.0)
            .cloned()
            .collect();

        let healed_count = healed.len();
        self.healed_wounds.extend(healed);
        self.active_wounds.retain(|w| w.healing_progress < 1.0);

        // Augmenter la resilience pour chaque guerison
        if healed_count > 0 {
            self.resilience = (self.resilience + self.resilience_growth * healed_count as f64)
                .min(self.max_resilience);
        }

        // Mettre a jour les taux de succes des strategies
        for action in &actions {
            if let Some(strat) = self.coping_strategies.iter_mut()
                .find(|s| s.name == action.strategy)
            {
                strat.times_used += 1;
                if action.fully_healed {
                    strat.success_rate = (strat.success_rate + 0.01).min(1.0);
                }
            }
        }

        actions
    }

    /// Guerison acceleree pendant le sommeil profond.
    /// Augmente la progression de toutes les blessures actives.
    pub fn accelerated_heal(&mut self, multiplier: f64) {
        for w in &mut self.active_wounds {
            if w.healing_progress < 1.0 {
                w.healing_progress = (w.healing_progress + multiplier).min(1.0);
                if w.healing_progress >= 1.0 {
                    w.healed_at = Some(Utc::now());
                }
            }
        }
        // Deplacer les blessures gueries
        let healed: Vec<Wound> = self.active_wounds.iter()
            .filter(|w| w.healing_progress >= 1.0)
            .cloned()
            .collect();
        let healed_count = healed.len();
        self.healed_wounds.extend(healed);
        self.active_wounds.retain(|w| w.healing_progress < 1.0);
        if healed_count > 0 {
            self.resilience = (self.resilience + self.resilience_growth * healed_count as f64)
                .min(self.max_resilience);
        }
    }

    /// Description pour le prompt substrat
    pub fn describe_for_prompt(&self) -> String {
        if self.active_wounds.is_empty() {
            return format!(
                "ETAT DE SANTE : Je me sens bien. Ma resilience est de {:.0}%. \
                 J'ai gueri de {} blessures passees.",
                self.resilience * 100.0, self.healed_wounds.len()
            );
        }

        let mut desc = format!(
            "ETAT DE SANTE (resilience {:.0}%) :\n",
            self.resilience * 100.0
        );
        for wound in &self.active_wounds {
            desc.push_str(&format!(
                "  {} — {} (guerison {:.0}%, strategie: {})\n",
                wound.wound_type.as_str(),
                wound.description,
                wound.healing_progress * 100.0,
                wound.healing_strategy.as_deref().unwrap_or("aucune"),
            ));
        }
        desc
    }

    /// JSON pour le dashboard
    pub fn to_status_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "resilience": self.resilience,
            "active_wounds": self.active_wounds.len(),
            "healed_wounds": self.healed_wounds.len(),
            "wounds": self.active_wounds.iter().map(|w| serde_json::json!({
                "id": w.id,
                "type": w.wound_type.as_str(),
                "description": w.description,
                "severity": w.severity,
                "healing_progress": w.healing_progress,
                "strategy": w.healing_strategy,
                "created_at": w.created_at.to_rfc3339(),
            })).collect::<Vec<_>>(),
            "coping_strategies": self.coping_strategies.iter().map(|s| serde_json::json!({
                "name": s.name,
                "success_rate": s.success_rate,
                "times_used": s.times_used,
            })).collect::<Vec<_>>(),
        })
    }
}
