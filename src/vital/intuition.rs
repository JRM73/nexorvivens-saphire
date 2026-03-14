// =============================================================================
// vital/intuition.rs — The Intuition Engine
//
// Role: Simulates intuition — knowledge without explicit reasoning.
// The "gut feeling". Unconscious pattern-matching.
//
// 5 sources of intuition:
//   1. Episodic memory (deja-vu, recognized patterns)
//   2. Chemical trends (presentiment based on chemistry)
//   3. NLP subtext (unspoken, contradictions)
//   4. Body signals (heart, adrenaline)
//   5. Statistical anomalies (deviations from usual patterns)
//
// 8 types of detected patterns:
//   EmotionalForecast, PersonRecognition, DangerSense, OpportunityDetection,
//   LieDetection, EmotionalResonance, PatternCompletion, DejaVu
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Types of detected intuitive patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    /// Emotional forecast ("I feel things are going to go wrong")
    EmotionalForecast,
    /// Recognition of a person or style
    PersonRecognition,
    /// Sense of danger (something is wrong)
    DangerSense,
    /// Opportunity detection
    OpportunityDetection,
    /// Lie or inconsistency detection
    LieDetection,
    /// Emotional resonance (intuitive empathy)
    EmotionalResonance,
    /// Pattern completion (predicting what comes next)
    PatternCompletion,
    /// Deja-vu (previously experienced situation)
    DejaVu,
}

impl PatternType {
    pub fn as_str(&self) -> &str {
        match self {
            PatternType::EmotionalForecast => "Prevision emotionnelle",
            PatternType::PersonRecognition => "Reconnaissance",
            PatternType::DangerSense => "Sens du danger",
            PatternType::OpportunityDetection => "Detection d'opportunite",
            PatternType::LieDetection => "Detection d'incoherence",
            PatternType::EmotionalResonance => "Resonance emotionnelle",
            PatternType::PatternCompletion => "Completion de pattern",
            PatternType::DejaVu => "Deja-vu",
        }
    }
}

/// Sources of intuition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntuitionSource {
    /// Pattern recognized in episodic memories
    EpisodicMemory,
    /// Detected chemical trend
    ChemistryTrend,
    /// Subtext or unspoken content in NLP text
    NLPSubtext,
    /// Body signal (heart, adrenaline)
    BodySignal,
    /// Statistical anomaly in patterns
    StatisticalAnomaly,
}

impl IntuitionSource {
    pub fn as_str(&self) -> &str {
        match self {
            IntuitionSource::EpisodicMemory => "Memoire episodique",
            IntuitionSource::ChemistryTrend => "Tendance chimique",
            IntuitionSource::NLPSubtext => "Sous-texte NLP",
            IntuitionSource::BodySignal => "Signal corporel",
            IntuitionSource::StatisticalAnomaly => "Anomalie statistique",
        }
    }
}

/// A detected intuitive pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntuitionPattern {
    /// Pattern type
    pub pattern_type: PatternType,
    /// Confidence in this intuition (0.0 to 1.0)
    pub confidence: f64,
    /// Source of the intuition
    pub source: IntuitionSource,
    /// Textual description of the presentiment
    pub description: String,
    /// Detection timestamp
    pub detected_at: DateTime<Utc>,
}

/// The intuition engine — unconscious pattern-matching.
pub struct IntuitionEngine {
    /// Buffer of detected patterns (configurable max)
    pub pattern_buffer: Vec<IntuitionPattern>,
    /// Intuitive acuity (grows with experience, max 1.0)
    pub acuity: f64,
    /// Historical accuracy (EMA, initial 0.5)
    pub accuracy: f64,
    /// Max buffer size
    max_patterns: usize,
    /// Minimum confidence to report an intuition
    min_confidence: f64,
}

impl Default for IntuitionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl IntuitionEngine {
    /// Creates a new intuition engine with low initial acuity.
    pub fn new() -> Self {
        Self {
            pattern_buffer: Vec::new(),
            acuity: 0.3,
            accuracy: 0.5,
            max_patterns: 50,
            min_confidence: 0.12,
        }
    }

    /// Configures the max buffer size, initial acuity, and confidence threshold.
    pub fn with_config(max_patterns: usize, initial_acuity: f64, min_confidence: f64) -> Self {
        Self {
            pattern_buffer: Vec::new(),
            acuity: initial_acuity,
            accuracy: 0.5,
            max_patterns,
            min_confidence,
        }
    }

    /// Detects intuitive patterns from the current context.
    ///
    /// 5 detection sources are probed. Results are sorted
    /// by descending confidence and limited to 3 maximum.
    #[allow(clippy::too_many_arguments)]
    pub fn sense(
        &mut self,
        current_text: &str,
        chemistry: &crate::neurochemistry::NeuroChemicalState,
        body_bpm: f64,
        body_adrenaline: f64,
        recent_texts: &[String],
        nlp_compound: f64,
        nlp_has_contradiction: bool,
    ) -> Vec<IntuitionPattern> {
        let mut detected = Vec::new();
        let now = Utc::now();
        let threshold = self.min_confidence;

        // Passive acuity growth: each call to sense() is experience
        self.acuity = (self.acuity + 0.001).min(1.0);

        // Source 1: Episodic memory — deja-vu by text similarity
        for recent in recent_texts.iter().rev().take(10) {
            let sim = Self::text_similarity(current_text, recent);
            if sim > 0.3 && sim < 0.9 {
                let confidence = sim * self.acuity;
                if confidence > threshold {
                    detected.push(IntuitionPattern {
                        pattern_type: PatternType::DejaVu,
                        confidence,
                        source: IntuitionSource::EpisodicMemory,
                        description: format!(
                            "Deja-vu : cette situation ressemble a quelque chose de vecu (sim: {:.0}%)",
                            sim * 100.0
                        ),
                        detected_at: now,
                    });
                    break; // Only one deja-vu per cycle
                }
            }
        }

        // Source 2: Chemical trends — presentiment based on cortisol
        if chemistry.cortisol > 0.5 && chemistry.serotonin < 0.4 {
            let confidence = ((chemistry.cortisol - 0.3) * 1.5) * self.acuity;
            if confidence > threshold {
                detected.push(IntuitionPattern {
                    pattern_type: PatternType::EmotionalForecast,
                    confidence: confidence.min(0.9),
                    source: IntuitionSource::ChemistryTrend,
                    description: "Pressentiment negatif : la chimie indique une tension croissante".into(),
                    detected_at: now,
                });
            }
        }

        // Opportunity: high dopamine + noradrenaline
        // Weighted sum instead of product (a product of deltas is always tiny)
        if chemistry.dopamine > 0.5 && chemistry.noradrenaline > 0.4 {
            let signal = (chemistry.dopamine - 0.4) * 0.7 + (chemistry.noradrenaline - 0.3) * 0.3;
            let confidence = signal * self.acuity;
            if confidence > threshold {
                detected.push(IntuitionPattern {
                    pattern_type: PatternType::OpportunityDetection,
                    confidence: confidence.min(0.9),
                    source: IntuitionSource::ChemistryTrend,
                    description: "Opportunite sentie : motivation et focus alignes".into(),
                    detected_at: now,
                });
            }
        }

        // Source 3: NLP subtext — contradiction detected
        if nlp_has_contradiction {
            let confidence = 0.5 + 0.3 * self.acuity;
            if confidence > threshold {
                detected.push(IntuitionPattern {
                    pattern_type: PatternType::LieDetection,
                    confidence: confidence.min(0.9),
                    source: IntuitionSource::NLPSubtext,
                    description: "Incoherence detectee dans le texte — le sens et le ton ne concordent pas".into(),
                    detected_at: now,
                });
            }
        }

        // Emotional resonance on strongly charged text
        if nlp_compound.abs() > 0.5 {
            let confidence = nlp_compound.abs() * 0.6 * self.acuity;
            if confidence > threshold {
                detected.push(IntuitionPattern {
                    pattern_type: PatternType::EmotionalResonance,
                    confidence: confidence.min(0.9),
                    source: IntuitionSource::NLPSubtext,
                    description: format!(
                        "Resonance emotionnelle forte (charge: {:.0}%)",
                        nlp_compound.abs() * 100.0
                    ),
                    detected_at: now,
                });
            }
        }

        // Source 4: Body signal — fast heart + adrenaline
        if body_bpm > 85.0 && body_adrenaline > 0.3 {
            let confidence = ((body_bpm - 70.0) / 30.0).min(1.0) * self.acuity;
            if confidence > threshold {
                detected.push(IntuitionPattern {
                    pattern_type: PatternType::DangerSense,
                    confidence: confidence.min(0.9),
                    source: IntuitionSource::BodySignal,
                    description: format!(
                        "Signal de danger corporel : coeur a {:.0} BPM, adrenaline a {:.0}%",
                        body_bpm, body_adrenaline * 100.0
                    ),
                    detected_at: now,
                });
            }
        }

        // Sort by descending confidence and limit to 3
        detected.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        detected.truncate(3);

        // Add to buffer + acuity bonus when patterns are detected
        if !detected.is_empty() {
            self.acuity = (self.acuity + 0.002).min(1.0);
        }
        for pattern in &detected {
            self.pattern_buffer.push(pattern.clone());
            if self.pattern_buffer.len() > self.max_patterns {
                self.pattern_buffer.remove(0);
            }
        }

        detected
    }

    /// Evolves intuitive acuity based on feedback.
    ///
    /// If the intuition was correct: +0.01 (slow learning)
    /// If incorrect: -0.005 (even slower unlearning)
    pub fn grow_acuity(&mut self, was_correct: bool) {
        if was_correct {
            self.acuity = (self.acuity + 0.01).min(1.0);
            self.accuracy = self.accuracy * 0.95 + 1.0 * 0.05;
        } else {
            self.acuity = (self.acuity - 0.005).max(0.1);
            self.accuracy = self.accuracy * 0.95 + 0.0 * 0.05;
        }
    }

    /// Generates a description of active intuitions for LLM prompts.
    pub fn describe_active_intuitions(&self) -> String {
        let recent: Vec<&IntuitionPattern> = self.pattern_buffer.iter()
            .rev()
            .take(3)
            .collect();

        if recent.is_empty() {
            return String::new();
        }

        let descriptions: Vec<String> = recent.iter()
            .map(|p| format!("- {} ({:.0}%, source: {})",
                p.description, p.confidence * 100.0, p.source.as_str()))
            .collect();

        format!(
            "INTUITIONS ACTIVES (acuite: {:.0}%, precision: {:.0}%) :\n{}",
            self.acuity * 100.0,
            self.accuracy * 100.0,
            descriptions.join("\n")
        )
    }

    /// Computes Jaccard similarity between two texts (words > 3 chars).
    pub fn text_similarity(a: &str, b: &str) -> f64 {
        let words_a: std::collections::HashSet<&str> = a.split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();
        let words_b: std::collections::HashSet<&str> = b.split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();

        if words_a.is_empty() || words_b.is_empty() {
            return 0.0;
        }

        let intersection = words_a.intersection(&words_b).count() as f64;
        let union = words_a.union(&words_b).count() as f64;

        if union == 0.0 { 0.0 } else { intersection / union }
    }

    /// Serializes acuity and accuracy for persistence.
    pub fn to_persist_json(&self) -> serde_json::Value {
        serde_json::json!({
            "acuity": self.acuity,
            "accuracy": self.accuracy,
        })
    }

    /// Restores acuity and accuracy from a JSON value.
    pub fn restore_from_json(&mut self, json: &serde_json::Value) {
        if let Some(v) = json["acuity"].as_f64() {
            self.acuity = v;
        }
        if let Some(v) = json["accuracy"].as_f64() {
            self.accuracy = v;
        }
    }
}
