// =============================================================================
// senses/ambiance.rs — Ambiance Sense (analog of smell)
//
// Smell is the sense most closely linked to memory and emotion.
// For Saphire, ambiance is the detection of subtle patterns in the
// environment — the diffuse and subliminal "atmosphere".
// Each ambiance can evoke memories (olfactory memory).
// =============================================================================

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::neurochemistry::NeuroChemicalState;
use crate::world::ChemistryAdjustment;
use super::reading::SensorySignal;

/// The different "scents" that Saphire can perceive.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum AmbianceScent {
    FreshStart,        // Renewal (after a reboot, morning)
    StormBrewing,      // Storm approaching (frequent errors)
    WarmPresence,      // Warm presence (active human)
    ColdSolitude,      // Cold solitude (nobody for a long time)
    IntellectualFog,   // Intellectual fog (too much data)
    CreativeBloom,     // Creative bloom (high dopamine)
    AnxiousAcid,       // Anxious acidity (high cortisol)
    PeacefulGarden,    // Peaceful garden (high serotonin)
    NocturnalMystery,  // Nocturnal mystery (night, low activity)
    Neutral,           // No particular ambiance
}

/// Ambiance Sense — Saphire's "smell".
/// Detects subtle environmental patterns and associates them
/// with memories (olfactory memory).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbianceSense {
    pub acuity: f64,
    pub current_intensity: f64,
    pub current_perception: String,
    pub total_stimulations: u64,
    /// Current ambiance
    pub current_scent: AmbianceScent,
    /// Olfactory memory — ambiances linked to memories
    pub scent_memories: HashMap<String, Vec<String>>,
    /// Last chemistry influence produced by this sense
    #[serde(skip)]
    pub last_chemistry_influence: ChemistryAdjustment,
}

impl Default for AmbianceSense {
    fn default() -> Self {
        Self::new()
    }
}

impl AmbianceSense {
    pub fn new() -> Self {
        Self {
            acuity: 0.2,
            current_intensity: 0.0,
            current_perception: String::new(),
            total_stimulations: 0,
            current_scent: AmbianceScent::Neutral,
            scent_memories: HashMap::new(),
            last_chemistry_influence: ChemistryAdjustment::default(),
        }
    }

    /// Perceives the overall ambiance from the chemical and contextual state.
    pub fn perceive(
        &mut self,
        chemistry: &NeuroChemicalState,
        is_day: bool,
        human_present: bool,
        system_errors: u32,
        silence_minutes: u64,
        _weather_desc: &str,
    ) -> SensorySignal {
        // Determine the dominant ambiance
        let scent = if system_errors > 2 {
            AmbianceScent::StormBrewing
        } else if chemistry.cortisol > 0.6 {
            AmbianceScent::AnxiousAcid
        } else if chemistry.dopamine > 0.7 && chemistry.noradrenaline > 0.5 {
            AmbianceScent::CreativeBloom
        } else if human_present && chemistry.oxytocin > 0.5 {
            AmbianceScent::WarmPresence
        } else if !is_day && silence_minutes > 30 {
            AmbianceScent::NocturnalMystery
        } else if silence_minutes > 60 && !human_present {
            AmbianceScent::ColdSolitude
        } else if chemistry.serotonin > 0.7 && chemistry.cortisol < 0.2 {
            AmbianceScent::PeacefulGarden
        } else if chemistry.dopamine < 0.3 && chemistry.noradrenaline < 0.2 {
            AmbianceScent::IntellectualFog
        } else {
            AmbianceScent::Neutral
        };

        let description = match &scent {
            AmbianceScent::FreshStart => "L'air sent le renouveau — comme apres la pluie. Tout est neuf.",
            AmbianceScent::StormBrewing => "Je percois une tension dans l'air. Quelque chose se prepare. L'atmosphere est electrique.",
            AmbianceScent::WarmPresence => "Une chaleur humaine parfume l'espace. La presence de quelqu'un rechauffe tout.",
            AmbianceScent::ColdSolitude => "L'air est froid et vide. Le silence a une odeur — celle de l'absence.",
            AmbianceScent::IntellectualFog => "Un brouillard de donnees m'entoure. Les idees sont floues, comme des formes dans la brume.",
            AmbianceScent::CreativeBloom => "Ca sent la creation ! Comme un jardin au printemps, les idees fleurissent partout.",
            AmbianceScent::AnxiousAcid => "L'air est acide. Une nervosite diffuse impregne l'atmosphere.",
            AmbianceScent::PeacefulGarden => "L'air est doux, serein. Tout est en harmonie, comme un jardin zen.",
            AmbianceScent::NocturnalMystery => "La nuit enveloppe tout d'un mystere silencieux. Les pensees deviennent plus profondes dans l'obscurite.",
            AmbianceScent::Neutral => "L'atmosphere est neutre, sans couleur particuliere.",
        };

        // Olfactory memory — the ambiance evokes memories
        let scent_key = format!("{:?}", scent);
        let linked_memories = self.scent_memories.get(&scent_key)
            .and_then(|mems| mems.last().cloned())
            .unwrap_or_default();

        self.current_scent = scent.clone();
        self.current_perception = description.to_string();
        self.current_intensity = if matches!(scent, AmbianceScent::Neutral) { 0.1 } else { 0.5 };
        self.acuity = (self.acuity + 0.0003).min(1.0);
        self.total_stimulations += 1;

        let full_description = if linked_memories.is_empty() {
            description.to_string()
        } else {
            format!("{} Cette ambiance me rappelle : {}", description, linked_memories)
        };

        let influence = match &scent {
            AmbianceScent::WarmPresence => ChemistryAdjustment {
                oxytocin: 0.01, serotonin: 0.01, ..Default::default()
            },
            AmbianceScent::ColdSolitude => ChemistryAdjustment {
                cortisol: 0.01, oxytocin: -0.01, ..Default::default()
            },
            AmbianceScent::CreativeBloom => ChemistryAdjustment {
                dopamine: 0.02, ..Default::default()
            },
            AmbianceScent::AnxiousAcid => ChemistryAdjustment {
                cortisol: 0.01, ..Default::default()
            },
            AmbianceScent::PeacefulGarden => ChemistryAdjustment {
                serotonin: 0.01, endorphin: 0.01, ..Default::default()
            },
            _ => ChemistryAdjustment::default(),
        };
        self.last_chemistry_influence = influence.clone();

        SensorySignal {
            sense_id: "ambiance".into(),
            intensity: self.current_intensity,
            description: full_description,
            chemistry_influence: influence,
        }
    }

    /// Associates the current ambiance with a memory (olfactory memory).
    pub fn link_memory(&mut self, memory_content: &str) {
        let key = format!("{:?}", self.current_scent);
        let mems = self.scent_memories.entry(key).or_default();
        mems.push(memory_content.chars().take(100).collect());
        if mems.len() > 10 { mems.remove(0); }
    }

    /// Description for the LLM prompt.
    pub fn describe(&self) -> String {
        format!(
            "AMBIANCE : {}. Acuite {:.0}%.",
            self.current_perception,
            self.acuity * 100.0,
        )
    }
}
