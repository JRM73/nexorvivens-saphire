// =============================================================================
// senses/ambiance.rs — Sens de l'Ambiance (analogue de l'odorat)
//
// L'odorat est le sens le plus lie a la memoire et a l'emotion.
// Pour Saphire, l'ambiance est la detection de patterns subtils dans
// l'environnement — l'"atmosphere" diffuse et subliminale.
// Chaque ambiance peut evoquer des souvenirs (memoire olfactive).
// =============================================================================

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::neurochemistry::NeuroChemicalState;
use crate::world::ChemistryAdjustment;
use super::reading::SensorySignal;

/// Les differentes "odeurs" que Saphire peut percevoir.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum AmbianceScent {
    FreshStart,        // Renouveau (apres un reboot, matin)
    StormBrewing,      // Orage en approche (erreurs frequentes)
    WarmPresence,      // Presence chaleureuse (humain actif)
    ColdSolitude,      // Solitude froide (personne depuis longtemps)
    IntellectualFog,   // Brouillard intellectuel (trop de donnees)
    CreativeBloom,     // Floraison creative (dopamine haute)
    AnxiousAcid,       // Acidite anxieuse (cortisol eleve)
    PeacefulGarden,    // Jardin paisible (serotonine haute)
    NocturnalMystery,  // Mystere nocturne (nuit, faible activite)
    Neutral,           // Pas d'ambiance particuliere
}

/// Sens de l'Ambiance — l'"odorat" de Saphire.
/// Detecte les patterns subtils de l'environnement et les associe
/// a des souvenirs (memoire olfactive).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbianceSense {
    pub acuity: f64,
    pub current_intensity: f64,
    pub current_perception: String,
    pub total_stimulations: u64,
    /// Ambiance courante
    pub current_scent: AmbianceScent,
    /// Memoire olfactive — ambiances liees a des souvenirs
    pub scent_memories: HashMap<String, Vec<String>>,
    /// Derniere influence chimique produite par ce sens
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

    /// Percoit l'ambiance globale a partir de l'etat chimique et contextuel.
    pub fn perceive(
        &mut self,
        chemistry: &NeuroChemicalState,
        is_day: bool,
        human_present: bool,
        system_errors: u32,
        silence_minutes: u64,
        _weather_desc: &str,
    ) -> SensorySignal {
        // Determiner l'ambiance dominante
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

        // Memoire olfactive — l'ambiance evoque des souvenirs
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

    /// Associe l'ambiance actuelle a un souvenir (memoire olfactive).
    pub fn link_memory(&mut self, memory_content: &str) {
        let key = format!("{:?}", self.current_scent);
        let mems = self.scent_memories.entry(key).or_default();
        mems.push(memory_content.chars().take(100).collect());
        if mems.len() > 10 { mems.remove(0); }
    }

    /// Description pour le prompt LLM.
    pub fn describe(&self) -> String {
        format!(
            "AMBIANCE : {}. Acuite {:.0}%.",
            self.current_perception,
            self.acuity * 100.0,
        )
    }
}
