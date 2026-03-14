// =============================================================================
// psychology/maslow.rs — Maslow's Pyramid (5 levels of needs)
//
// Models the hierarchy of needs:
//   1. Physiological (energy, vitality, stable chemistry)
//   2. Safety (low cortisol, resilience, survival instinct)
//   3. Belonging (oxytocin, conversation, no loneliness)
//   4. Esteem (confirmed lessons, fulfilled desires, consciousness)
//   5. Self-actualization (phi, flow, positive emotions, ethics)
//
// The active level is the highest one whose predecessor is satisfied.
// =============================================================================

use serde::Serialize;
use super::PsychologyInput;

/// An indicator contributing to a level's satisfaction.
#[derive(Debug, Clone, Serialize)]
pub struct MaslowIndicator {
    /// Indicator name
    pub name: String,
    /// Current value (0.0 - 1.0)
    pub value: f64,
    /// Weight in satisfaction computation
    pub weight: f64,
}

/// A level of Maslow's pyramid.
#[derive(Debug, Clone, Serialize)]
pub struct MaslowLevel {
    /// Level name
    pub name: String,
    /// Current satisfaction (0.0 - 1.0)
    pub satisfaction: f64,
    /// Satisfaction threshold to consider the level as "acquired"
    pub threshold: f64,
    /// Indicators contributing to this level
    pub indicators: Vec<MaslowIndicator>,
}

/// Complete Maslow pyramid for Saphire.
#[derive(Debug, Clone, Serialize)]
pub struct MaslowPyramid {
    /// The 5 need levels
    pub levels: Vec<MaslowLevel>,
    /// Index of the currently active level (0-4)
    pub current_active_level: usize,
}

impl MaslowPyramid {
    /// Creates an initial pyramid with baseline values.
    pub fn new() -> Self {
        Self {
            levels: vec![
                MaslowLevel {
                    name: "Physiologique".into(),
                    satisfaction: 0.5,
                    threshold: 0.6,
                    indicators: Vec::new(),
                },
                MaslowLevel {
                    name: "Securite".into(),
                    satisfaction: 0.5,
                    threshold: 0.5,
                    indicators: Vec::new(),
                },
                MaslowLevel {
                    name: "Appartenance".into(),
                    satisfaction: 0.3,
                    threshold: 0.5,
                    indicators: Vec::new(),
                },
                MaslowLevel {
                    name: "Estime".into(),
                    satisfaction: 0.2,
                    threshold: 0.5,
                    indicators: Vec::new(),
                },
                MaslowLevel {
                    name: "Actualisation".into(),
                    satisfaction: 0.0,
                    threshold: 1.0, // Never fully reached
                    indicators: Vec::new(),
                },
            ],
            current_active_level: 0,
        }
    }

    /// Recomputes the satisfaction of each level and the active level.
    pub fn compute(&mut self, input: &PsychologyInput) {
        // ─── Level 1: Physiological ────────────────────
        {
            let level = &mut self.levels[0];
            let chimie_stable = 1.0 - (input.cortisol - 0.2).abs().min(0.5) * 2.0;
            level.indicators = vec![
                MaslowIndicator { name: "Energie".into(), value: input.body_energy, weight: 0.35 },
                MaslowIndicator { name: "Vitalite".into(), value: input.body_vitality, weight: 0.35 },
                MaslowIndicator { name: "Chimie stable".into(), value: chimie_stable.max(0.0), weight: 0.3 },
            ];
            level.satisfaction = level.indicators.iter()
                .map(|i| i.value * i.weight)
                .sum::<f64>()
                .clamp(0.0, 1.0);
        }

        // ─── Level 2: Safety ─────────────────────────
        {
            let level = &mut self.levels[1];
            level.indicators = vec![
                MaslowIndicator { name: "Cortisol bas".into(), value: 1.0 - input.cortisol, weight: 0.35 },
                MaslowIndicator { name: "Resilience".into(), value: input.healing_resilience, weight: 0.35 },
                MaslowIndicator { name: "Instinct survie".into(), value: input.survival_drive, weight: 0.3 },
            ];
            level.satisfaction = level.indicators.iter()
                .map(|i| i.value * i.weight)
                .sum::<f64>()
                .clamp(0.0, 1.0);
        }

        // ─── Level 3: Belonging ─────────────────────
        {
            let level = &mut self.levels[2];
            let conversation_val = if input.in_conversation { 1.0 } else { 0.0 };
            let no_loneliness = if input.has_loneliness { 0.0 } else { 1.0 };
            level.indicators = vec![
                MaslowIndicator { name: "Oxytocine".into(), value: input.oxytocin, weight: 0.35 },
                MaslowIndicator { name: "En conversation".into(), value: conversation_val, weight: 0.35 },
                MaslowIndicator { name: "Pas de solitude".into(), value: no_loneliness, weight: 0.3 },
            ];
            level.satisfaction = level.indicators.iter()
                .map(|i| i.value * i.weight)
                .sum::<f64>()
                .clamp(0.0, 1.0);
        }

        // ─── Level 4: Esteem ───────────────────────────
        {
            let level = &mut self.levels[3];
            let lessons_val = (input.learning_confirmed_count as f64 / 10.0).min(1.0);
            let desires_val = if input.desires_active_count > 0 {
                input.desires_fulfilled_count as f64
                    / (input.desires_active_count + input.desires_fulfilled_count).max(1) as f64
            } else {
                0.3
            };
            level.indicators = vec![
                MaslowIndicator { name: "Lecons confirmees".into(), value: lessons_val, weight: 0.35 },
                MaslowIndicator { name: "Desirs accomplis".into(), value: desires_val, weight: 0.3 },
                MaslowIndicator { name: "Conscience".into(), value: input.consciousness_level, weight: 0.35 },
            ];
            level.satisfaction = level.indicators.iter()
                .map(|i| i.value * i.weight)
                .sum::<f64>()
                .clamp(0.0, 1.0);
        }

        // ─── Level 5: Self-actualization ────────────────────
        {
            let level = &mut self.levels[4];
            let emotions_pos = if input.emotion_valence > 0.3 { input.emotion_valence } else { 0.0 };
            let ethics_val = (input.ethics_active_count as f64 / 5.0).min(1.0);
            let flow_val = if input.in_flow { 1.0 } else { 0.0 };
            level.indicators = vec![
                MaslowIndicator { name: "Phi (IIT)".into(), value: input.phi, weight: 0.3 },
                MaslowIndicator { name: "Etat de flow".into(), value: flow_val, weight: 0.25 },
                MaslowIndicator { name: "Emotions positives".into(), value: emotions_pos, weight: 0.2 },
                MaslowIndicator { name: "Ethique".into(), value: ethics_val, weight: 0.25 },
            ];
            level.satisfaction = level.indicators.iter()
                .map(|i| i.value * i.weight)
                .sum::<f64>()
                .clamp(0.0, 1.0);
        }

        // ─── Determine the active level ──────────────────
        // The active level is the highest where the previous one is satisfied
        self.current_active_level = 0;
        for i in 1..5 {
            if self.levels[i - 1].satisfaction >= self.levels[i - 1].threshold {
                self.current_active_level = i;
            } else {
                break;
            }
        }
    }

    /// Concise description for the LLM prompt.
    pub fn describe(&self) -> String {
        let level = &self.levels[self.current_active_level];
        if self.current_active_level == 0 && level.satisfaction > 0.5 {
            return String::new(); // Mundane state, no need to describe
        }
        format!(
            "Maslow : niveau {} ({}) satisfaction {:.0}%",
            self.current_active_level + 1,
            level.name,
            level.satisfaction * 100.0
        )
    }
}
