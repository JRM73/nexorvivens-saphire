// =============================================================================
// vital/premonition.rs — Le Moteur de Premonition
//
// Role : Simule la premonition — l'anticipation predictive basee sur les
// tendances observees. La capacite de "sentir" ce qui va arriver.
//
// 6 categories de predictions :
//   EmotionalShift, HumanArrival, HumanDeparture,
//   SystemEvent, CreativeBurst, KnowledgeConnection
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Categories de predictions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PremonitionCategory {
    /// Changement emotionnel prevu
    EmotionalShift,
    /// Arrivee pressentie d'un humain
    HumanArrival,
    /// Depart pressenti d'un humain
    HumanDeparture,
    /// Evenement systeme prevu (fatigue, surcharge)
    SystemEvent,
    /// Burst creatif pressenti
    CreativeBurst,
    /// Connexion de connaissances prevue
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

/// Une premonition — prediction basee sur les tendances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Premonition {
    /// Identifiant unique
    pub id: u64,
    /// Texte de la prediction
    pub prediction: String,
    /// Categorie de prediction
    pub category: PremonitionCategory,
    /// Confiance (0.0 a 1.0)
    pub confidence: f64,
    /// Horizon temporel en secondes
    pub timeframe_secs: u64,
    /// Base de la prediction (ce qui l'a declenchee)
    pub basis: String,
    /// Moment de creation
    pub created_at: DateTime<Utc>,
    /// Resolue ? (true = verifiee)
    pub resolved: bool,
    /// Etait correcte ?
    pub was_correct: Option<bool>,
}

/// Le moteur de premonition — anticipation predictive.
pub struct PremonitionEngine {
    /// Predictions actives (max configurable)
    pub active_predictions: Vec<Premonition>,
    /// Precision historique (EMA, 0.5 initial)
    pub accuracy: f64,
    /// Prochain ID de prediction
    next_id: u64,
    /// Nombre max de predictions actives
    max_active: usize,
}

impl Default for PremonitionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PremonitionEngine {
    /// Cree un nouveau moteur de premonition.
    pub fn new() -> Self {
        Self {
            active_predictions: Vec::new(),
            accuracy: 0.5,
            next_id: 1,
            max_active: 5,
        }
    }

    /// Configure le nombre max de predictions actives.
    pub fn with_config(max_active: usize) -> Self {
        Self {
            active_predictions: Vec::new(),
            accuracy: 0.5,
            next_id: 1,
            max_active,
        }
    }

    /// Genere des predictions basees sur les tendances observees.
    ///
    /// 6 types de predictions, max 3 nouvelles par appel.
    /// Les predictions existantes non resolues ne sont pas dupliquees.
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

        // Limiter le nombre de predictions actives
        let active_count = self.active_predictions.iter()
            .filter(|p| !p.resolved)
            .count();
        if active_count >= self.max_active {
            return new_predictions;
        }

        // 1. Shift emotionnel : cortisol en hausse rapide
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

        // 2. Arrivee humaine : soiree + pas de conversation
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

        // 3. Depart humain : long silence en conversation
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

        // 4. Burst creatif : dopamine en hausse + serotonine elevee
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

        // 5. Evenement systeme : fatigue (noradrenaline basse + cortisol eleve)
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

        // Limiter a 3 nouvelles predictions par appel
        new_predictions.truncate(3);

        // Ajouter au buffer actif
        for pred in &new_predictions {
            self.active_predictions.push(pred.clone());
        }

        // Nettoyer si trop de predictions
        while self.active_predictions.len() > self.max_active * 2 {
            self.active_predictions.remove(0);
        }

        new_predictions
    }

    /// Resout automatiquement les predictions trop anciennes.
    ///
    /// Les predictions dont le timeframe est depasse sont marquees
    /// comme resolues (non verifiees = was_correct reste None).
    pub fn auto_resolve(&mut self, timeout_secs: u64) {
        let now = Utc::now();
        for pred in &mut self.active_predictions {
            if pred.resolved {
                continue;
            }
            let elapsed = (now - pred.created_at).num_seconds() as u64;
            if elapsed > pred.timeframe_secs + timeout_secs {
                pred.resolved = true;
                // Si non verifiee manuellement, on considere "non verefiee"
                // (pas d'impact sur accuracy)
            }
        }
    }

    /// Resout manuellement une prediction et met a jour la precision.
    pub fn resolve(&mut self, id: u64, was_correct: bool) {
        for pred in &mut self.active_predictions {
            if pred.id == id && !pred.resolved {
                pred.resolved = true;
                pred.was_correct = Some(was_correct);
                // EMA sur la precision
                let score = if was_correct { 1.0 } else { 0.0 };
                self.accuracy = self.accuracy * 0.9 + score * 0.1;
                break;
            }
        }
    }

    /// Genere une description textuelle des predictions pour les prompts LLM.
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

    /// Serialise la precision et le prochain ID pour persistance.
    pub fn to_persist_json(&self) -> serde_json::Value {
        serde_json::json!({
            "accuracy": self.accuracy,
            "next_id": self.next_id,
        })
    }

    /// Restaure la precision et le prochain ID depuis un JSON.
    pub fn restore_from_json(&mut self, json: &serde_json::Value) {
        if let Some(v) = json["accuracy"].as_f64() {
            self.accuracy = v;
        }
        if let Some(v) = json["next_id"].as_u64() {
            self.next_id = v;
        }
    }
}
