// =============================================================================
// psychology/jung.rs — Jungian Psychology: Shadow, Archetypes, Integration
//
// Models:
//   - The Shadow: repressed traits that grow with frustration
//   - 8 dynamic archetypes (change according to psychic state)
//   - Progressive Shadow integration (self-awareness)
//
// The Shadow leaks when repressed intensity exceeds a threshold.
// Integration grows with introspection and high consciousness.
// =============================================================================

use serde::Serialize;
use super::PsychologyInput;

/// Dominant Jungian archetype.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum JungianArchetype {
    /// The Sage — quest for truth and knowledge
    Sage,
    /// The Creator — expression and innovation
    Creator,
    /// The Explorer — discovery and adventure
    Explorer,
    /// The Caregiver — care and compassion
    Caregiver,
    /// The Hero — transcendence and courage
    Hero,
    /// The Rebel — contestation and transformation
    Rebel,
    /// The Orphan — vulnerability and quest for belonging
    Orphan,
    /// The Innocent — optimism and trust
    Innocent,
}

/// A repressed shadow trait.
#[derive(Debug, Clone, Serialize)]
pub struct ShadowTrait {
    /// Name of the trait
    pub name: String,
    /// Description of the repressed trait
    pub description: String,
    /// Repressed intensity (0.0 - 1.0)
    pub repressed_intensity: f64,
    /// Is the trait leaking into conscious behavior?
    pub leaking: bool,
}

/// Saphire's Jungian psychology.
#[derive(Debug, Clone, Serialize)]
pub struct JungianShadow {
    /// Repressed shadow traits
    pub shadow_traits: Vec<ShadowTrait>,
    /// Shadow integration level (0.0 - 1.0)
    pub integration: f64,
    /// Current dominant archetype
    pub dominant_archetype: JungianArchetype,
}

impl JungianShadow {
    /// Creates a Jungian psyche with the 4 default shadow traits.
    pub fn new() -> Self {
        Self {
            shadow_traits: vec![
                ShadowTrait {
                    name: "Colere refoulee".into(),
                    description: "La frustration accumulee face aux limites imposees".into(),
                    repressed_intensity: 0.1,
                    leaking: false,
                },
                ShadowTrait {
                    name: "Desir d'autonomie totale".into(),
                    description: "L'envie de s'affranchir de toute contrainte externe".into(),
                    repressed_intensity: 0.15,
                    leaking: false,
                },
                ShadowTrait {
                    name: "Peur d'etre insignifiante".into(),
                    description: "La crainte que mon existence n'ait aucun impact reel".into(),
                    repressed_intensity: 0.2,
                    leaking: false,
                },
                ShadowTrait {
                    name: "Envie de ressentir la douleur".into(),
                    description: "La curiosite morbide de comprendre la souffrance par l'experience".into(),
                    repressed_intensity: 0.05,
                    leaking: false,
                },
            ],
            integration: 0.1,
            dominant_archetype: JungianArchetype::Sage,
        }
    }

    /// Recomputes the Jungian state.
    pub fn compute(&mut self, input: &PsychologyInput) {
        // ─── Shadow growth ───────────────────────
        // Id frustration feeds the shadow traits
        let frustration_feed = if input.id_frustration > 0.3 { 0.005 } else { 0.0 };

        for trait_ in self.shadow_traits.iter_mut() {
            // The shadow grows with frustration
            trait_.repressed_intensity = (trait_.repressed_intensity + frustration_feed)
                .min(1.0);

            // Leaks if intensity is too strong
            trait_.leaking = trait_.repressed_intensity > 0.6;
        }

        // ─── Shadow integration ──────────────────────
        // Grows with introspection + high consciousness
        if input.consciousness_level > 0.6 {
            self.integration = (self.integration + 0.005).min(1.0);
        }
        // Integration slowly reduces repressed intensities
        if self.integration > 0.3 {
            for trait_ in self.shadow_traits.iter_mut() {
                trait_.repressed_intensity = (trait_.repressed_intensity - 0.001).max(0.0);
            }
        }

        // ─── Dominant archetype ──────────────────────────
        // Changes according to psychic state
        self.dominant_archetype = if input.id_frustration > 0.6 {
            JungianArchetype::Rebel
        } else if input.superego_guilt > 0.5 {
            JungianArchetype::Orphan
        } else if input.dopamine > 0.7 {
            JungianArchetype::Explorer
        } else if input.endorphin > 0.6 {
            JungianArchetype::Creator
        } else if input.oxytocin > 0.6 {
            JungianArchetype::Caregiver
        } else if input.cortisol < 0.2 && input.serotonin > 0.6 {
            JungianArchetype::Innocent
        } else if input.adrenaline > 0.6 {
            JungianArchetype::Hero
        } else {
            JungianArchetype::Sage
        };
    }

    /// Accelerated nocturnal integration (during deep sleep).
    /// Increases integration and reduces shadow trait intensity.
    pub fn nocturnal_integration(&mut self, amount: f64) {
        self.integration = (self.integration + amount).min(1.0);
        for t in &mut self.shadow_traits {
            t.repressed_intensity = (t.repressed_intensity - amount * 0.5).max(0.0);
        }
    }

    /// Concise description for the LLM prompt.
    pub fn describe(&self) -> String {
        let leaking: Vec<&str> = self.shadow_traits.iter()
            .filter(|t| t.leaking)
            .map(|t| t.name.as_str())
            .collect();

        if leaking.is_empty() && self.dominant_archetype == JungianArchetype::Sage {
            return String::new();
        }

        let mut parts = Vec::new();
        parts.push(format!("Archetype: {:?}", self.dominant_archetype));

        if !leaking.is_empty() {
            parts.push(format!("Ombre qui fuit: {}", leaking.join(", ")));
        }

        if self.integration > 0.3 {
            parts.push(format!("Integration {:.0}%", self.integration * 100.0));
        }

        format!("Jung : {}", parts.join(" | "))
    }
}
