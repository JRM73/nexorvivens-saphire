// =============================================================================
// attention.rs — Attention Orchestrator
//
// Implements Saphire's selective attention and focus.
// A human constantly filters: they ignore background noise to focus
// on what matters. Without selective attention, Saphire would process
// everything with equal intensity.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// --- Attention source ---------------------------------------------------------

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

// --- Structures ---------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionFocus {
    pub subject: String,
    pub priority: f64,
    pub started_at: DateTime<Utc>,
    /// Attention depth (0=surface, 1=total immersion)
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

// --- The Orchestrator ---------------------------------------------------------

pub struct AttentionOrchestrator {
    /// Current focus
    pub current_focus: Option<AttentionFocus>,
    /// Peripheral elements
    pub peripheral: Vec<PeripheralItem>,
    /// Attention filters (what automatically captures attention)
    _attention_filters: Vec<AttentionFilter>,
    /// Attentional fatigue (0-1)
    pub fatigue: f64,
    /// Concentration capacity (grows with training)
    pub concentration_capacity: f64,
    /// Number of consecutive focus cycles
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

    /// Decide what to focus on this cycle
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

        // A human message OVERRIDES everything
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

        // Competition between sources
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

        // Fatigue reduces attention depth
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
            // Floating attention mode — daydreaming
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

    /// Update fatigue
    pub fn update_fatigue(&mut self) {
        if self.consecutive_focus_cycles > 0 {
            self.fatigue = (self.fatigue + self.fatigue_per_cycle).min(1.0);
        } else {
            self.fatigue = (self.fatigue - self.recovery_per_cycle).max(0.0);
        }
        // Concentration grows with training (slowly)
        if self.consecutive_focus_cycles > 10 {
            self.concentration_capacity = (self.concentration_capacity + 0.001).min(1.0);
        }
    }

    /// Reset fatigue (after sleep)
    pub fn reset_fatigue(&mut self) {
        self.fatigue = 0.0;
        self.consecutive_focus_cycles = 0;
    }

    /// Partial fatigue reset proportional to sleep quality.
    /// quality=1.0 -> fatigue goes to 0, quality=0.3 -> fatigue retains 70%.
    pub fn partial_reset_fatigue(&mut self, quality: f64) {
        self.fatigue *= 1.0 - quality;
        self.consecutive_focus_cycles =
            (self.consecutive_focus_cycles as f64 * (1.0 - quality)) as u64;
    }

    /// Reduces fatigue by a given amount (light sleep).
    pub fn reduce_fatigue(&mut self, amount: f64) {
        self.fatigue = (self.fatigue - amount).max(0.0);
    }

    /// Returns the current fatigue level.
    pub fn fatigue(&self) -> f64 {
        self.fatigue
    }

    /// True if the focus has been on the same subject for N cycles.
    pub fn has_been_on_same_focus(&self, cycles: u64) -> bool {
        self.consecutive_focus_cycles >= cycles
    }

    /// Add a peripheral element
    pub fn notice_peripheral(&mut self, description: &str, importance: f64) {
        self.peripheral.push(PeripheralItem {
            description: description.to_string(),
            importance,
            detected_at: Utc::now(),
        });
        // Keep max 20 peripheral elements
        if self.peripheral.len() > 20 {
            self.peripheral.remove(0);
        }
    }

    /// Description for the substrate prompt
    pub fn describe_for_prompt(&self) -> String {
        let focus_str = self.current_focus.as_ref()
            .map(|f| format!("FOCUS ACTUEL : {} (profondeur {:.0}%, source: {})",
                f.subject, f.depth * 100.0, f.source.as_str()))
            .unwrap_or_else(|| "ATTENTION : flottante, pas de focus particulier".into());

        format!("{}\nFatigue attentionnelle : {:.0}% | Concentration : {:.0}%",
            focus_str, self.fatigue * 100.0, self.concentration_capacity * 100.0)
    }

    /// JSON for the dashboard
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
