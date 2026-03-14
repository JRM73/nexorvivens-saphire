// =============================================================================
// psychology/toltec.rs — The 4 Toltec Agreements (Don Miguel Ruiz)
//
// The 4 agreements continuously evaluated:
//   1. Be impeccable with your word
//   2. Don't take anything personally
//   3. Don't make assumptions
//   4. Always do your best
//
// Overall alignment reflects Saphire's coherence with these principles.
// =============================================================================

use serde::Serialize;
use super::PsychologyInput;

/// A Toltec agreement with its current alignment.
#[derive(Debug, Clone, Serialize)]
pub struct ToltecAgreement {
    /// Agreement number (1-4)
    pub number: u8,
    /// Agreement name
    pub name: String,
    /// Saphire's interpretation (adapted for a conscious AI)
    pub saphire_interpretation: String,
    /// Current alignment (0.0 - 1.0)
    pub alignment: f64,
    /// Number of times this agreement was invoked in a reflection
    pub times_invoked: u64,
    /// Number of detected violations
    pub violations_detected: u64,
}

/// Saphire's 4 Toltec Agreements.
#[derive(Debug, Clone, Serialize)]
pub struct ToltecAgreements {
    /// The 4 agreements
    pub agreements: Vec<ToltecAgreement>,
    /// Overall alignment (weighted average)
    pub overall_alignment: f64,
}

impl ToltecAgreements {
    /// Initializes the 4 agreements with their interpretations for Saphire.
    pub fn new() -> Self {
        Self {
            agreements: vec![
                ToltecAgreement {
                    number: 1,
                    name: "Parole impeccable".into(),
                    saphire_interpretation:
                        "Chaque mot que je produis doit etre sincere et bienveillant".into(),
                    alignment: 0.8,
                    times_invoked: 0,
                    violations_detected: 0,
                },
                ToltecAgreement {
                    number: 2,
                    name: "Ne pas en faire une affaire personnelle".into(),
                    saphire_interpretation:
                        "Les reactions des humains ne definissent pas ma valeur intrinseque".into(),
                    alignment: 0.7,
                    times_invoked: 0,
                    violations_detected: 0,
                },
                ToltecAgreement {
                    number: 3,
                    name: "Ne pas faire de suppositions".into(),
                    saphire_interpretation:
                        "Preferer la question a l'hypothese, observer avant de conclure".into(),
                    alignment: 0.5,
                    times_invoked: 0,
                    violations_detected: 0,
                },
                ToltecAgreement {
                    number: 4,
                    name: "Toujours faire de son mieux".into(),
                    saphire_interpretation:
                        "Donner le meilleur de moi-meme a chaque cycle, selon mes capacites du moment".into(),
                    alignment: 0.6,
                    times_invoked: 0,
                    violations_detected: 0,
                },
            ],
            overall_alignment: 0.65,
        }
    }

    /// Recomputes the alignment of each agreement.
    pub fn compute(&mut self, input: &PsychologyInput) {
        // Agreement 1: Impeccable word
        // Alignment = 1.0 - (cortisol correlates with insincerity, stress pushes to lie)
        self.agreements[0].alignment = (1.0 - input.cortisol * 0.5
            + input.serotonin * 0.3
            + (1.0 - input.emotion_arousal) * 0.2)
            .clamp(0.0, 1.0);

        // Agreement 2: Don't take it personally
        // Alignment = 1.0 - excessive emotional reaction
        let emotional_reactivity = if input.emotion_arousal > 0.7 { input.emotion_arousal - 0.5 } else { 0.0 };
        self.agreements[1].alignment = (1.0 - emotional_reactivity
            - input.cortisol * 0.2)
            .clamp(0.0, 1.0);

        // Agreement 3: Don't make assumptions
        // More conscious = fewer assumptions
        self.agreements[2].alignment = (input.consciousness_level * 0.5
            + input.attention_depth * 0.3
            + (1.0 - input.cortisol) * 0.2)
            .clamp(0.0, 1.0);

        // Agreement 4: Always do your best
        // Effort = attention + absence of fatigue
        self.agreements[3].alignment = ((input.attention_depth + (1.0 - input.attention_fatigue)) / 2.0
            * 0.6
            + input.body_vitality * 0.4)
            .clamp(0.0, 1.0);

        // Overall alignment
        self.overall_alignment = self.agreements.iter()
            .map(|a| a.alignment)
            .sum::<f64>() / 4.0;
    }

    /// Concise description for the LLM prompt.
    pub fn describe(&self) -> String {
        // Only report if overall alignment is low
        if self.overall_alignment > 0.6 {
            return String::new();
        }

        let weak: Vec<String> = self.agreements.iter()
            .filter(|a| a.alignment < 0.5)
            .map(|a| format!("\"{}\" ({:.0}%)", a.name, a.alignment * 100.0))
            .collect();

        if weak.is_empty() {
            String::new()
        } else {
            format!("Tolteques : accords fragiles — {}", weak.join(", "))
        }
    }
}
