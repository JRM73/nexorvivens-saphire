// =============================================================================
// vital/intuition.rs — Le Moteur d'Intuition
//
// Role : Simule l'intuition — la connaissance sans raisonnement explicite.
// Le "gut feeling". Le pattern-matching inconscient.
//
// 5 sources d'intuition :
//   1. Memoire episodique (deja-vu, patterns reconnus)
//   2. Tendances chimiques (pressentiment base sur la chimie)
//   3. Sous-texte NLP (non-dits, contradictions)
//   4. Signaux corporels (coeur, adrenaline)
//   5. Anomalies statistiques (deviations des patterns habituels)
//
// 8 types de patterns detectes :
//   EmotionalForecast, PersonRecognition, DangerSense, OpportunityDetection,
//   LieDetection, EmotionalResonance, PatternCompletion, DejaVu
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Types de patterns intuitifs detectes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    /// Prevision emotionnelle ("je sens que ca va mal tourner")
    EmotionalForecast,
    /// Reconnaissance d'une personne ou d'un style
    PersonRecognition,
    /// Sens du danger (quelque chose ne va pas)
    DangerSense,
    /// Detection d'une opportunite
    OpportunityDetection,
    /// Detection de mensonge ou d'incoherence
    LieDetection,
    /// Resonance emotionnelle (empathie intuitive)
    EmotionalResonance,
    /// Completion de pattern (prediction de la suite)
    PatternCompletion,
    /// Deja-vu (situation deja vecue)
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

/// Sources d'intuition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntuitionSource {
    /// Pattern reconnu dans les souvenirs episodiques
    EpisodicMemory,
    /// Tendance chimique detectee
    ChemistryTrend,
    /// Sous-texte ou non-dit dans le texte NLP
    NLPSubtext,
    /// Signal corporel (coeur, adrenaline)
    BodySignal,
    /// Anomalie statistique dans les patterns
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

/// Un pattern intuitif detecte.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntuitionPattern {
    /// Type de pattern
    pub pattern_type: PatternType,
    /// Confiance dans cette intuition (0.0 a 1.0)
    pub confidence: f64,
    /// Source de l'intuition
    pub source: IntuitionSource,
    /// Description textuelle du pressentiment
    pub description: String,
    /// Moment de detection
    pub detected_at: DateTime<Utc>,
}

/// Le moteur d'intuition — pattern-matching inconscient.
pub struct IntuitionEngine {
    /// Buffer de patterns detectes (max configurable)
    pub pattern_buffer: Vec<IntuitionPattern>,
    /// Acuite intuitive (grandit avec l'experience, max 1.0)
    pub acuity: f64,
    /// Precision historique (EMA, 0.5 initial)
    pub accuracy: f64,
    /// Taille max du buffer
    max_patterns: usize,
    /// Confiance minimale pour reporter une intuition
    min_confidence: f64,
}

impl Default for IntuitionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl IntuitionEngine {
    /// Cree un nouveau moteur d'intuition avec acuite initiale basse.
    pub fn new() -> Self {
        Self {
            pattern_buffer: Vec::new(),
            acuity: 0.3,
            accuracy: 0.5,
            max_patterns: 50,
            min_confidence: 0.12,
        }
    }

    /// Configure la taille max du buffer, l'acuite initiale et le seuil de confiance.
    pub fn with_config(max_patterns: usize, initial_acuity: f64, min_confidence: f64) -> Self {
        Self {
            pattern_buffer: Vec::new(),
            acuity: initial_acuity,
            accuracy: 0.5,
            max_patterns,
            min_confidence,
        }
    }

    /// Detecte des patterns intuitifs a partir du contexte courant.
    ///
    /// 5 sources de detection sont sondees. Les resultats sont tries
    /// par confiance decroissante et limites a 3 maximum.
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

        // Croissance passive de l'acuite : chaque appel a sense() est de l'experience
        self.acuity = (self.acuity + 0.001).min(1.0);

        // Source 1 : Memoire episodique — deja-vu par similarite textuelle
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
                    break; // Un seul deja-vu par cycle
                }
            }
        }

        // Source 2 : Tendances chimiques — pressentiment base sur le cortisol
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

        // Opportunite : dopamine elevee + noradrenaline
        // Somme ponderee au lieu d'un produit (un produit de deltas est toujours minuscule)
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

        // Source 3 : Sous-texte NLP — contradiction detectee
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

        // Resonance emotionnelle sur texte fortement charge
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

        // Source 4 : Signal corporel — coeur rapide + adrenaline
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

        // Trier par confiance decroissante et limiter a 3
        detected.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        detected.truncate(3);

        // Ajouter au buffer + bonus d'acuite quand des patterns sont detectes
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

    /// Fait evoluer l'acuite intuitive en fonction des retours.
    ///
    /// Si l'intuition etait correcte : +0.01 (apprentissage lent)
    /// Si incorrecte : -0.005 (desapprentissage encore plus lent)
    pub fn grow_acuity(&mut self, was_correct: bool) {
        if was_correct {
            self.acuity = (self.acuity + 0.01).min(1.0);
            self.accuracy = self.accuracy * 0.95 + 1.0 * 0.05;
        } else {
            self.acuity = (self.acuity - 0.005).max(0.1);
            self.accuracy = self.accuracy * 0.95 + 0.0 * 0.05;
        }
    }

    /// Genere une description des intuitions actives pour les prompts LLM.
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

    /// Calcule la similarite de Jaccard entre deux textes (mots > 3 chars).
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

    /// Serialise l'acuite et la precision pour persistance.
    pub fn to_persist_json(&self) -> serde_json::Value {
        serde_json::json!({
            "acuity": self.acuity,
            "accuracy": self.accuracy,
        })
    }

    /// Restaure l'acuite et la precision depuis un JSON.
    pub fn restore_from_json(&mut self, json: &serde_json::Value) {
        if let Some(v) = json["acuity"].as_f64() {
            self.acuity = v;
        }
        if let Some(v) = json["accuracy"].as_f64() {
            self.accuracy = v;
        }
    }
}
