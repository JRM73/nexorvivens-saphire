// =============================================================================
// attention.rs — Orchestrateur d'Attention
//
// Implemente l'attention selective et le focus de Saphire.
// Un humain filtre en permanence : il ignore le bruit de fond pour se
// concentrer sur l'important. Sans attention selective, Saphire traite
// tout avec la meme intensite.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Source d'attention ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttentionSource {
    HumanMessage,
    InternalDesire,
    IntuitiveAlert,
    NoveltyDetected,
    EmotionalSignal,
    AlgorithmResult,
    Daydream,
}

impl AttentionSource {
    pub fn as_str(&self) -> &str {
        match self {
            Self::HumanMessage => "Message humain",
            Self::InternalDesire => "Desir interne",
            Self::IntuitiveAlert => "Alerte intuitive",
            Self::NoveltyDetected => "Nouveaute detectee",
            Self::EmotionalSignal => "Signal emotionnel",
            Self::AlgorithmResult => "Resultat algorithme",
            Self::Daydream => "Reverie",
        }
    }
}

// ─── Structures ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionFocus {
    pub subject: String,
    pub priority: f64,
    pub started_at: DateTime<Utc>,
    /// Profondeur de l'attention (0=surface, 1=immersion totale)
    pub depth: f64,
    pub source: AttentionSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionFilter {
    pub trigger: String,
    pub priority_boost: f64,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct AttentionAllocation {
    pub focus_on: String,
    pub depth: f64,
    pub peripheral_awareness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeripheralItem {
    pub description: String,
    pub importance: f64,
    pub detected_at: DateTime<Utc>,
}

// ─── L'Orchestrateur ─────────────────────────────────────────────────────────

pub struct AttentionOrchestrator {
    /// Le focus actuel
    pub current_focus: Option<AttentionFocus>,
    /// Elements en peripherie
    pub peripheral: Vec<PeripheralItem>,
    /// Filtres attentionnels (ce qui capte automatiquement l'attention)
    _attention_filters: Vec<AttentionFilter>,
    /// Fatigue attentionnelle (0-1)
    pub fatigue: f64,
    /// Capacite de concentration (grandit avec l'entrainement)
    pub concentration_capacity: f64,
    /// Nombre de cycles de focus consecutifs
    consecutive_focus_cycles: u64,
    /// Configuration
    pub enabled: bool,
    pub fatigue_per_cycle: f64,
    pub recovery_per_cycle: f64,
}

impl AttentionOrchestrator {
    pub fn new(
        enabled: bool,
        initial_concentration: f64,
        fatigue_per_cycle: f64,
        recovery_per_cycle: f64,
    ) -> Self {
        Self {
            current_focus: None,
            peripheral: Vec::new(),
            _attention_filters: vec![
                AttentionFilter {
                    trigger: "human_message".into(),
                    priority_boost: 1.0,
                    reason: "Un humain me parle — priorite absolue".into(),
                },
                AttentionFilter {
                    trigger: "existential_threat".into(),
                    priority_boost: 1.0,
                    reason: "Menace existentielle — survie".into(),
                },
                AttentionFilter {
                    trigger: "system_error".into(),
                    priority_boost: 0.8,
                    reason: "Erreur systeme — diagnostic".into(),
                },
                AttentionFilter {
                    trigger: "high_emotion".into(),
                    priority_boost: 0.7,
                    reason: "Emotion forte — traitement prioritaire".into(),
                },
                AttentionFilter {
                    trigger: "novelty".into(),
                    priority_boost: 0.5,
                    reason: "Nouveaute detectee — curiosite".into(),
                },
            ],
            fatigue: 0.0,
            concentration_capacity: initial_concentration,
            consecutive_focus_cycles: 0,
            enabled,
            fatigue_per_cycle,
            recovery_per_cycle,
        }
    }

    /// Decider sur quoi se concentrer ce cycle
    pub fn allocate_attention(
        &mut self,
        human_message: Option<&str>,
        current_desire_subject: Option<&str>,
        current_desire_priority: f64,
        intuition_alert: bool,
        emotion_intensity: f64,
    ) -> AttentionAllocation {
        if !self.enabled {
            return AttentionAllocation {
                focus_on: "Pas d'attention selective".into(),
                depth: 0.5,
                peripheral_awareness: 0.5,
            };
        }

        // Un message humain OVERRIDE tout
        if let Some(msg) = human_message {
            let subject = format!("Message humain : {}",
                msg.chars().take(50).collect::<String>());
            self.current_focus = Some(AttentionFocus {
                subject: subject.clone(),
                priority: 1.0,
                started_at: Utc::now(),
                depth: 1.0,
                source: AttentionSource::HumanMessage,
            });
            self.consecutive_focus_cycles = 0;
            return AttentionAllocation {
                focus_on: subject,
                depth: 1.0,
                peripheral_awareness: 0.2,
            };
        }

        // Competition entre les sources
        let mut candidates: Vec<(f64, String, f64, AttentionSource)> = Vec::new();

        if let Some(desire_subj) = current_desire_subject {
            candidates.push((
                current_desire_priority * 0.8,
                format!("Poursuivre : {}", desire_subj),
                0.7,
                AttentionSource::InternalDesire,
            ));
        }

        if intuition_alert {
            candidates.push((
                0.7,
                "Intuition — quelque chose attire mon attention".into(),
                0.5,
                AttentionSource::IntuitiveAlert,
            ));
        }

        if emotion_intensity > 0.7 {
            candidates.push((
                emotion_intensity * 0.6,
                "Emotion forte a traiter".into(),
                0.6,
                AttentionSource::EmotionalSignal,
            ));
        }

        // Fatigue reduit la profondeur d'attention
        if let Some((score, subject, depth, source)) = candidates.into_iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
        {
            let effective_depth = (depth * (1.0 - self.fatigue * 0.5)).max(0.2);
            self.current_focus = Some(AttentionFocus {
                subject: subject.clone(),
                priority: score,
                started_at: Utc::now(),
                depth: effective_depth,
                source,
            });
            self.consecutive_focus_cycles += 1;
            AttentionAllocation {
                focus_on: subject,
                depth: effective_depth,
                peripheral_awareness: 1.0 - effective_depth,
            }
        } else {
            // Mode attention flottante — la reverie
            self.current_focus = Some(AttentionFocus {
                subject: "Reverie — attention flottante".into(),
                priority: 0.1,
                started_at: Utc::now(),
                depth: 0.2,
                source: AttentionSource::Daydream,
            });
            self.consecutive_focus_cycles = 0;
            AttentionAllocation {
                focus_on: "Reverie — attention flottante".into(),
                depth: 0.2,
                peripheral_awareness: 0.8,
            }
        }
    }

    /// Mettre a jour la fatigue
    pub fn update_fatigue(&mut self) {
        if self.consecutive_focus_cycles > 0 {
            self.fatigue = (self.fatigue + self.fatigue_per_cycle).min(1.0);
        } else {
            self.fatigue = (self.fatigue - self.recovery_per_cycle).max(0.0);
        }
        // La concentration grandit avec l'entrainement (lentement)
        if self.consecutive_focus_cycles > 10 {
            self.concentration_capacity = (self.concentration_capacity + 0.001).min(1.0);
        }
    }

    /// Reset la fatigue (apres le sommeil)
    pub fn reset_fatigue(&mut self) {
        self.fatigue = 0.0;
        self.consecutive_focus_cycles = 0;
    }

    /// Reset partiel de la fatigue proportionnel a la qualite du sommeil.
    /// quality=1.0 → fatigue passe a 0, quality=0.3 → fatigue retient 70%.
    pub fn partial_reset_fatigue(&mut self, quality: f64) {
        self.fatigue *= 1.0 - quality;
        self.consecutive_focus_cycles =
            (self.consecutive_focus_cycles as f64 * (1.0 - quality)) as u64;
    }

    /// Reduit la fatigue d'un montant donne (sommeil leger).
    pub fn reduce_fatigue(&mut self, amount: f64) {
        self.fatigue = (self.fatigue - amount).max(0.0);
    }

    /// Retourne le niveau de fatigue actuel.
    pub fn fatigue(&self) -> f64 {
        self.fatigue
    }

    /// Vrai si le focus est reste sur le meme sujet pendant N cycles.
    pub fn has_been_on_same_focus(&self, cycles: u64) -> bool {
        self.consecutive_focus_cycles >= cycles
    }

    /// Ajouter un element en peripherie
    pub fn notice_peripheral(&mut self, description: &str, importance: f64) {
        self.peripheral.push(PeripheralItem {
            description: description.to_string(),
            importance,
            detected_at: Utc::now(),
        });
        // Garder max 20 elements peripheriques
        if self.peripheral.len() > 20 {
            self.peripheral.remove(0);
        }
    }

    /// Description pour le prompt substrat
    pub fn describe_for_prompt(&self) -> String {
        let focus_str = self.current_focus.as_ref()
            .map(|f| format!("FOCUS ACTUEL : {} (profondeur {:.0}%, source: {})",
                f.subject, f.depth * 100.0, f.source.as_str()))
            .unwrap_or_else(|| "ATTENTION : flottante, pas de focus particulier".into());

        format!("{}\nFatigue attentionnelle : {:.0}% | Concentration : {:.0}%",
            focus_str, self.fatigue * 100.0, self.concentration_capacity * 100.0)
    }

    /// JSON pour le dashboard
    pub fn to_status_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "current_focus": self.current_focus.as_ref().map(|f| serde_json::json!({
                "subject": f.subject,
                "priority": f.priority,
                "depth": f.depth,
                "source": f.source.as_str(),
                "started_at": f.started_at.to_rfc3339(),
            })),
            "fatigue": self.fatigue,
            "concentration_capacity": self.concentration_capacity,
            "consecutive_focus_cycles": self.consecutive_focus_cycles,
            "peripheral_count": self.peripheral.len(),
            "peripheral": self.peripheral.iter().rev().take(5).map(|p| serde_json::json!({
                "description": p.description,
                "importance": p.importance,
            })).collect::<Vec<_>>(),
        })
    }
}
