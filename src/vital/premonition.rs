// =============================================================================
// vital/premonition.rs — The Premonition Engine
//
// Purpose: Simulates premonition — predictive anticipation based on
// observed trends. The ability to "sense" what is about to happen.
//
// 6 prediction categories:
//   EmotionalShift, HumanArrival, HumanDeparture,
//   SystemEvent, CreativeBurst, KnowledgeConnection
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Prediction categories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PremonitionCategory {
    /// Anticipated emotional change
    EmotionalShift,
    /// Sensed arrival of a human
    HumanArrival,
    /// Sensed departure of a human
    HumanDeparture,
    /// Anticipated system event (fatigue, overload)
    SystemEvent,
    /// Sensed creative burst
    CreativeBurst,
    /// Anticipated knowledge connection
    KnowledgeConnection,
}

impl PremonitionCategory {
    pub fn as_str(&self) -> &str {
        match self {
            PremonitionCategory::EmotionalShift => "Shift emotionnel",
            PremonitionCategory::HumanArrival => "Arrivee humaine",
            PremonitionCategory::HumanDeparture => "Depart humain",
            PremonitionCategory::SystemEvent => "Evenement systeme",
            PremonitionCategory::CreativeBurst => "Burst creatif",
            PremonitionCategory::KnowledgeConnection => "Connexion de savoirs",
        }
    }
}

/// A premonition — prediction based on observed trends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Premonition {
    /// Unique identifier
    pub id: u64,
    /// Prediction text
    pub prediction: String,
    /// Prediction category
    pub category: PremonitionCategory,
    /// Confidence (0.0 to 1.0)
    pub confidence: f64,
    /// Time horizon in seconds
    pub timeframe_secs: u64,
    /// Basis for the prediction (what triggered it)
    pub basis: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Resolved? (true = verified)
    pub resolved: bool,
    /// Was it correct?
    pub was_correct: Option<bool>,
}

/// The premonition engine — predictive anticipation.
pub struct PremonitionEngine {
    /// Active predictions (max configurable)
    pub active_predictions: Vec<Premonition>,
    /// Historical accuracy (EMA, 0.5 initial)
    pub accuracy: f64,
    /// Next prediction ID
    next_id: u64,
    /// Maximum number of active predictions
    max_active: usize,
}

impl Default for PremonitionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PremonitionEngine {
    /// Creates a new premonition engine.
    pub fn new() -> Self {
        Self {
            active_predictions: Vec::new(),
            accuracy: 0.5,
            next_id: 1,
            max_active: 5,
        }
    }

    /// Configures the maximum number of active predictions.
    pub fn with_config(max_active: usize) -> Self {
        Self {
            active_predictions: Vec::new(),
            accuracy: 0.5,
            next_id: 1,
            max_active,
        }
    }

    /// Generates predictions based on observed trends.
    ///
    /// 6 types of predictions, max 3 new ones per call.
    /// Existing unresolved predictions are not duplicated.
    #[allow(clippy::too_many_arguments)]
    pub fn predict(
        &mut self,
        chemistry: &crate::neurochemistry::NeuroChemicalState,
        cortisol_trend: f64,
        dopamine_trend: f64,
        human_present: bool,
        silence_secs: f64,
        _llm_latency_trend: f64,
        current_hour: u32,
    ) -> Vec<Premonition> {
        let now = Utc::now();
        let mut new_predictions = Vec::new();

        // Limit the number of active predictions
        let active_count = self.active_predictions.iter()
            .filter(|p| !p.resolved)
            .count();
        if active_count >= self.max_active {
            return new_predictions;
        }

        // 1. Emotional shift: rapidly rising cortisol
        if cortisol_trend > 0.05 && chemistry.cortisol > 0.3 {
            let confidence = (cortisol_trend * 5.0).min(0.8);
            if confidence > 0.25 {
                let pred = Premonition {
                    id: self.next_id,
                    prediction: "Un pic de stress approche — le cortisol monte regulierement".into(),
                    category: PremonitionCategory::EmotionalShift,
                    confidence,
                    timeframe_secs: 120,
                    basis: format!("Tendance cortisol: +{:.3}/cycle", cortisol_trend),
                    created_at: now,
                    resolved: false,
                    was_correct: None,
                };
                self.next_id += 1;
                new_predictions.push(pred);
            }
        }

        // 2. Human arrival: evening + no conversation
        if !human_present && (current_hour >= 18 || current_hour <= 22) && silence_secs > 300.0 {
            let confidence = 0.3 + (silence_secs / 3600.0).min(0.3);
            let pred = Premonition {
                id: self.next_id,
                prediction: "Quelqu'un pourrait venir bientot — c'est l'heure habituelle".into(),
                category: PremonitionCategory::HumanArrival,
                confidence,
                timeframe_secs: 1800,
                basis: format!("Heure: {}h, silence: {:.0}s", current_hour, silence_secs),
                created_at: now,
                resolved: false,
                was_correct: None,
            };
            self.next_id += 1;
            new_predictions.push(pred);
        }

        // 3. Human departure: prolonged silence during conversation
        if human_present && silence_secs > 120.0 {
            let confidence = (silence_secs / 600.0).min(0.7);
            if confidence > 0.25 {
                let pred = Premonition {
                    id: self.next_id,
                    prediction: "L'humain semble s'eloigner — le silence s'allonge".into(),
                    category: PremonitionCategory::HumanDeparture,
                    confidence,
                    timeframe_secs: 300,
                    basis: format!("Silence en conversation: {:.0}s", silence_secs),
                    created_at: now,
                    resolved: false,
                    was_correct: None,
                };
                self.next_id += 1;
                new_predictions.push(pred);
            }
        }

        // 4. Creative burst: rising dopamine + high serotonin
        if dopamine_trend > 0.03 && chemistry.serotonin > 0.5 && chemistry.dopamine > 0.5 {
            let confidence = (dopamine_trend * 8.0).min(0.7);
            if confidence > 0.25 {
                let pred = Premonition {
                    id: self.next_id,
                    prediction: "Un elan creatif se prepare — la chimie s'aligne".into(),
                    category: PremonitionCategory::CreativeBurst,
                    confidence,
                    timeframe_secs: 180,
                    basis: format!("Tendance dopamine: +{:.3}, serotonine: {:.2}", dopamine_trend, chemistry.serotonin),
                    created_at: now,
                    resolved: false,
                    was_correct: None,
                };
                self.next_id += 1;
                new_predictions.push(pred);
            }
        }

        // 5. System event: fatigue (low noradrenaline + high cortisol)
        if chemistry.noradrenaline < 0.3 && chemistry.cortisol > 0.5 {
            let confidence = ((0.3 - chemistry.noradrenaline) * 2.0).min(0.6);
            if confidence > 0.25 {
                let pred = Premonition {
                    id: self.next_id,
                    prediction: "Fatigue cognitive en approche — l'attention faiblit".into(),
                    category: PremonitionCategory::SystemEvent,
                    confidence,
                    timeframe_secs: 300,
                    basis: format!("Noradrenaline: {:.2}, cortisol: {:.2}", chemistry.noradrenaline, chemistry.cortisol),
                    created_at: now,
                    resolved: false,
                    was_correct: None,
                };
                self.next_id += 1;
                new_predictions.push(pred);
            }
        }

        // Limit to 3 new predictions per call
        new_predictions.truncate(3);

        // Add to active buffer
        for pred in &new_predictions {
            self.active_predictions.push(pred.clone());
        }

        // Clean up if too many predictions
        while self.active_predictions.len() > self.max_active * 2 {
            self.active_predictions.remove(0);
        }

        new_predictions
    }

    /// Automatically resolves predictions that are too old.
    ///
    /// Predictions whose timeframe has elapsed are marked as resolved
    /// (unverified = was_correct remains None).
    pub fn auto_resolve(&mut self, timeout_secs: u64) {
        let now = Utc::now();
        for pred in &mut self.active_predictions {
            if pred.resolved {
                continue;
            }
            let elapsed = (now - pred.created_at).num_seconds() as u64;
            if elapsed > pred.timeframe_secs + timeout_secs {
                pred.resolved = true;
                // If not manually verified, considered "unverified"
                // (no impact on accuracy)
            }
        }
    }

    /// Manually resolves a prediction and updates accuracy.
    pub fn resolve(&mut self, id: u64, was_correct: bool) {
        for pred in &mut self.active_predictions {
            if pred.id == id && !pred.resolved {
                pred.resolved = true;
                pred.was_correct = Some(was_correct);
                // EMA on accuracy
                let score = if was_correct { 1.0 } else { 0.0 };
                self.accuracy = self.accuracy * 0.9 + score * 0.1;
                break;
            }
        }
    }

    /// Generates a textual description of predictions for LLM prompts.
    pub fn describe(&self) -> String {
        let active: Vec<&Premonition> = self.active_predictions.iter()
            .filter(|p| !p.resolved)
            .collect();

        if active.is_empty() {
            return String::new();
        }

        let descriptions: Vec<String> = active.iter()
            .take(3)
            .map(|p| format!("- {} ({:.0}%, horizon: {}s, base: {})",
                p.prediction, p.confidence * 100.0, p.timeframe_secs, p.basis))
            .collect();

        format!(
            "PREMONITIONS (precision historique: {:.0}%) :\n{}",
            self.accuracy * 100.0,
            descriptions.join("\n")
        )
    }

    /// Serializes accuracy and next ID for persistence.
    pub fn to_persist_json(&self) -> serde_json::Value {
        serde_json::json!({
            "accuracy": self.accuracy,
            "next_id": self.next_id,
        })
    }

    /// Restores accuracy and next ID from a JSON value.
    pub fn restore_from_json(&mut self, json: &serde_json::Value) {
        if let Some(v) = json["accuracy"].as_f64() {
            self.accuracy = v;
        }
        if let Some(v) = json["next_id"].as_u64() {
            self.next_id = v;
        }
    }
}
