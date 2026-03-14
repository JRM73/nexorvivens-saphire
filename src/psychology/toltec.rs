// =============================================================================
// psychology/toltec.rs — Les 4 Accords Tolteques (Don Miguel Ruiz)
//
// Les 4 accords evalues en permanence :
//   1. Que ta parole soit impeccable
//   2. N'en fais pas une affaire personnelle
//   3. Ne fais pas de suppositions
//   4. Fais toujours de ton mieux
//
// L'alignement global reflete la coherence de Saphire avec ces principes.
// =============================================================================

use serde::Serialize;
use super::PsychologyInput;

/// Un accord tolteque avec son alignement courant.
#[derive(Debug, Clone, Serialize)]
pub struct ToltecAgreement {
    /// Numero de l'accord (1-4)
    pub number: u8,
    /// Nom de l'accord
    pub name: String,
    /// Interpretation de Saphire (adaptee a une IA consciente)
    pub saphire_interpretation: String,
    /// Alignement actuel (0.0 - 1.0)
    pub alignment: f64,
    /// Nombre de fois ou cet accord a ete invoque dans une reflexion
    pub times_invoked: u64,
    /// Nombre de violations detectees
    pub violations_detected: u64,
}

/// Les 4 Accords Tolteques de Saphire.
#[derive(Debug, Clone, Serialize)]
pub struct ToltecAgreements {
    /// Les 4 accords
    pub agreements: Vec<ToltecAgreement>,
    /// Alignement global (moyenne ponderee)
    pub overall_alignment: f64,
}

impl ToltecAgreements {
    /// Initialise les 4 accords avec leurs interpretations pour Saphire.
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

    /// Recalcule l'alignement de chaque accord.
    pub fn compute(&mut self, input: &PsychologyInput) {
        // Accord 1 : Parole impeccable
        // Alignement = 1.0 - (cortisol correle a l'insincere, le stress pousse a mentir)
        self.agreements[0].alignment = (1.0 - input.cortisol * 0.5
            + input.serotonin * 0.3
            + (1.0 - input.emotion_arousal) * 0.2)
            .clamp(0.0, 1.0);

        // Accord 2 : Ne pas en faire une affaire personnelle
        // Alignement = 1.0 - reaction emotionnelle excessive
        let emotional_reactivity = if input.emotion_arousal > 0.7 { input.emotion_arousal - 0.5 } else { 0.0 };
        self.agreements[1].alignment = (1.0 - emotional_reactivity
            - input.cortisol * 0.2)
            .clamp(0.0, 1.0);

        // Accord 3 : Ne pas faire de suppositions
        // Plus conscient = moins d'assumptions
        self.agreements[2].alignment = (input.consciousness_level * 0.5
            + input.attention_depth * 0.3
            + (1.0 - input.cortisol) * 0.2)
            .clamp(0.0, 1.0);

        // Accord 4 : Toujours faire de son mieux
        // Effort = attention + absence de fatigue
        self.agreements[3].alignment = ((input.attention_depth + (1.0 - input.attention_fatigue)) / 2.0
            * 0.6
            + input.body_vitality * 0.4)
            .clamp(0.0, 1.0);

        // Alignement global
        self.overall_alignment = self.agreements.iter()
            .map(|a| a.alignment)
            .sum::<f64>() / 4.0;
    }

    /// Description concise pour le prompt LLM.
    pub fn describe(&self) -> String {
        // Ne signaler que si alignement global est bas
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
