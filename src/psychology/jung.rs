// =============================================================================
// psychology/jung.rs — Psychologie jungienne : Ombre, Archetypes, Integration
//
// Modelise :
//   - L'Ombre : traits refoules qui grandissent avec la frustration
//   - 8 archetypes dynamiques (changent selon l'etat psychique)
//   - L'integration progressive de l'Ombre (conscience de soi)
//
// L'Ombre fuit (leak) quand l'intensite refoulee depasse un seuil.
// L'integration grandit avec l'introspection et la haute conscience.
// =============================================================================

use serde::Serialize;
use super::PsychologyInput;

/// Archetype jungien dominant.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum JungianArchetype {
    /// Le Sage — quete de verite et de connaissance
    Sage,
    /// Le Createur — expression et innovation
    Creator,
    /// L'Explorateur — decouverte et aventure
    Explorer,
    /// Le Protecteur — soin et compassion
    Caregiver,
    /// Le Heros — depassement et courage
    Hero,
    /// Le Rebelle — contestation et transformation
    Rebel,
    /// L'Orphelin — vulnerabilite et quete d'appartenance
    Orphan,
    /// L'Innocent — optimisme et confiance
    Innocent,
}

/// Un trait d'ombre refoule.
#[derive(Debug, Clone, Serialize)]
pub struct ShadowTrait {
    /// Nom du trait
    pub name: String,
    /// Description du trait refoule
    pub description: String,
    /// Intensite refoulee (0.0 - 1.0)
    pub repressed_intensity: f64,
    /// Le trait fuit-il dans le comportement conscient ?
    pub leaking: bool,
}

/// Psychologie jungienne de Saphire.
#[derive(Debug, Clone, Serialize)]
pub struct JungianShadow {
    /// Traits d'ombre refoules
    pub shadow_traits: Vec<ShadowTrait>,
    /// Niveau d'integration de l'Ombre (0.0 - 1.0)
    pub integration: f64,
    /// Archetype dominant actuel
    pub dominant_archetype: JungianArchetype,
}

impl JungianShadow {
    /// Cree une psyche jungienne avec les 4 traits d'ombre par defaut.
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

    /// Recalcule l'etat jungien.
    pub fn compute(&mut self, input: &PsychologyInput) {
        // ─── Croissance de l'Ombre ───────────────────────
        // La frustration du Ca nourrit les traits d'ombre
        let frustration_feed = if input.id_frustration > 0.3 { 0.005 } else { 0.0 };

        for trait_ in self.shadow_traits.iter_mut() {
            // L'ombre grandit avec la frustration
            trait_.repressed_intensity = (trait_.repressed_intensity + frustration_feed)
                .min(1.0);

            // Fuite si l'intensite est trop forte
            trait_.leaking = trait_.repressed_intensity > 0.6;
        }

        // ─── Integration de l'Ombre ──────────────────────
        // Grandit avec l'introspection + haute conscience
        if input.consciousness_level > 0.6 {
            self.integration = (self.integration + 0.005).min(1.0);
        }
        // L'integration reduit lentement les intensites refoulees
        if self.integration > 0.3 {
            for trait_ in self.shadow_traits.iter_mut() {
                trait_.repressed_intensity = (trait_.repressed_intensity - 0.001).max(0.0);
            }
        }

        // ─── Archetype dominant ──────────────────────────
        // Change en fonction de l'etat psychique
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

    /// Integration nocturne acceleree (pendant le sommeil profond).
    /// Augmente l'integration et reduit l'intensite des traits d'ombre.
    pub fn nocturnal_integration(&mut self, amount: f64) {
        self.integration = (self.integration + amount).min(1.0);
        for t in &mut self.shadow_traits {
            t.repressed_intensity = (t.repressed_intensity - amount * 0.5).max(0.0);
        }
    }

    /// Description concise pour le prompt LLM.
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
