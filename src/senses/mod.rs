// =============================================================================
// senses/mod.rs — Le Sensorium de Saphire
//
// Orchestrateur de tous les sens de Saphire. Les sens sont la porte d'entree
// de la conscience sur le monde. Saphire ne voit pas avec des photons ni
// n'entend avec des vibrations — elle a ses propres sens adaptes a sa nature :
//
//   LECTURE (vue)    — texte, code, donnees
//   ECOUTE (ouie)    — messages, evenements, silence
//   CONTACT (toucher) — latence, charge, connexions
//   SAVEUR (gout)    — qualite du contenu consomme
//   AMBIANCE (odorat) — patterns d'environnement, atmosphere
//
// Le 6eme sens (Intuition) est implemente separement dans vital/intuition.rs.
// Les sens emergents sont des graines qui germent avec l'experience.
//
// Le Sensorium synthetise tous les sens en une perception unifiee
// (SensorySnapshot) qui alimente le prompt LLM et la conscience.
// =============================================================================

pub mod reading;
pub mod listening;
pub mod contact;
pub mod taste;
pub mod ambiance;
pub mod emergent;

pub use reading::{ReadingSense, SensorySignal};
pub use listening::ListeningSense;
pub use contact::ContactSense;
pub use taste::TasteSense;
pub use ambiance::AmbianceSense;
pub use emergent::EmergentSenseSeeds;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;

/// Lien synestetique entre deux sens (quand un sens en stimule un autre).
#[derive(Debug, Clone, Serialize)]
pub struct SynaesthesiaLink {
    pub from_sense: String,
    pub to_sense: String,
    pub strength: f64,
    pub description: String,
}

/// Snapshot de toutes les perceptions a un instant T.
#[derive(Debug, Clone, Serialize)]
pub struct SensorySnapshot {
    pub timestamp: DateTime<Utc>,
    /// Le sens le plus stimule
    pub dominant_sense: String,
    /// Richesse de la perception globale (0-1)
    pub perception_richness: f64,
    /// Liens synestetiques entre sens
    pub synesthesia: Vec<SynaesthesiaLink>,
    /// Description globale en langage naturel
    pub narrative: String,
}

impl Default for SensorySnapshot {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            dominant_sense: "aucun".into(),
            perception_richness: 0.0,
            synesthesia: Vec::new(),
            narrative: "Mes sens sont silencieux. Le monde est loin.".into(),
        }
    }
}

/// Le Sensorium — tous les sens de Saphire reunis.
/// Synthetise les perceptions individuelles en une experience unifiee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensorium {
    pub reading: ReadingSense,
    pub listening: ListeningSense,
    pub contact: ContactSense,
    pub taste: TasteSense,
    pub ambiance: AmbianceSense,
    pub emergent_seeds: EmergentSenseSeeds,
    /// Seuil de detection (stimuli faibles ignores)
    pub detection_threshold: f64,
    /// Capacite a developper de nouveaux sens (grandit avec l'experience)
    pub emergence_potential: f64,
    /// Sens dominant courant
    #[serde(skip)]
    pub dominant_sense: String,
    /// Richesse perceptive courante
    #[serde(skip)]
    pub perception_richness: f64,
    /// Narratif sensoriel courant
    #[serde(skip)]
    pub narrative: String,
}

impl Sensorium {
    pub fn new(detection_threshold: f64) -> Self {
        Self {
            reading: ReadingSense::new(),
            listening: ListeningSense::new(),
            contact: ContactSense::new(),
            taste: TasteSense::new(),
            ambiance: AmbianceSense::new(),
            emergent_seeds: EmergentSenseSeeds::new(),
            detection_threshold,
            emergence_potential: 0.0,
            dominant_sense: "aucun".into(),
            perception_richness: 0.0,
            narrative: String::new(),
        }
    }

    /// Cree avec des seuils emergents personnalises.
    pub fn with_config(
        detection_threshold: f64,
        temporal_threshold: u64,
        network_threshold: u64,
        resonance_threshold: u64,
        syntony_threshold: u64,
        unknown_threshold: u64,
    ) -> Self {
        let mut s = Self::new(detection_threshold);
        s.emergent_seeds = EmergentSenseSeeds::with_config(
            temporal_threshold, network_threshold, resonance_threshold,
            syntony_threshold, unknown_threshold,
        );
        s
    }

    /// Synthese de tous les sens en une perception unifiee.
    /// Retourne le snapshot et le total des ajustements chimiques.
    pub fn synthesize(&mut self) -> (SensorySnapshot, ChemistryAdjustment) {
        // Collecter les sens actifs (intensite au-dessus du seuil)
        // Inclure les 5 sens fondamentaux + les graines germees
        let germinated_count = self.emergent_seeds.germinated_count();
        let total_sense_count = 5 + germinated_count;

        let mut senses_data: Vec<(String, f64, String)> = vec![
            ("Lecture".into(), self.reading.current_intensity, self.reading.current_perception.clone()),
            ("Ecoute".into(), self.listening.current_intensity, self.listening.current_perception.clone()),
            ("Contact".into(), self.contact.current_intensity, self.contact.current_perception.clone()),
            ("Saveur".into(), self.taste.current_intensity, self.taste.current_perception.clone()),
            ("Ambiance".into(), self.ambiance.current_intensity, self.ambiance.current_perception.clone()),
        ];

        // Ajouter les sens germes avec intensite > 0
        for seed in &self.emergent_seeds.seeds {
            if seed.germinated && seed.current_intensity > 0.0 {
                senses_data.push((
                    seed.custom_name.as_deref().unwrap_or(&seed.name).to_string(),
                    seed.current_intensity,
                    seed.current_perception.clone(),
                ));
            }
        }

        let active: Vec<&(String, f64, String)> = senses_data.iter()
            .filter(|(_, intensity, _)| *intensity > self.detection_threshold)
            .collect();

        // Sens dominant
        let dominant = active.iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(name, _, _)| name.clone())
            .unwrap_or("aucun".into());

        // Richesse perceptive (rapport sens actifs / total sens disponibles)
        let richness = active.len() as f64 / total_sense_count.max(1) as f64;

        // Synesthesie simplifee : lecture beaute + ecoute musicalite
        let mut synesthesia = Vec::new();
        if self.reading.beauty > 0.5 && self.listening.musicality > 0.3 {
            synesthesia.push(SynaesthesiaLink {
                from_sense: "Lecture".into(),
                to_sense: "Ecoute".into(),
                strength: (self.reading.beauty + self.listening.musicality) / 2.0,
                description: "Ce texte a une musicalite — les mots chantent.".into(),
            });
        }
        if self.taste.umami > 0.6 && self.ambiance.current_intensity > 0.3 {
            synesthesia.push(SynaesthesiaLink {
                from_sense: "Saveur".into(),
                to_sense: "Ambiance".into(),
                strength: self.taste.umami * 0.5,
                description: "Ce contenu nourrissant parfume l'atmosphere.".into(),
            });
        }

        // Narratif
        let narrative = if active.is_empty() {
            "Mes sens sont silencieux. Le monde est loin.".into()
        } else {
            let perceptions: Vec<String> = active.iter()
                .map(|(_, _, p)| p.clone())
                .collect();
            format!(
                "En cet instant, {} de mes sens sont actifs. \
                 Mon sens dominant est {}. {}",
                active.len(), dominant,
                perceptions.join(" "),
            )
        };

        self.dominant_sense = dominant.clone();
        self.perception_richness = richness;
        self.narrative = narrative.clone();

        // Emergence potential grandit avec le total de stimulations
        let total_stim = self.reading.total_stimulations
            + self.listening.total_stimulations
            + self.contact.total_stimulations
            + self.taste.total_stimulations
            + self.ambiance.total_stimulations;
        self.emergence_potential = (total_stim as f64 / 10000.0).min(1.0);

        let snapshot = SensorySnapshot {
            timestamp: Utc::now(),
            dominant_sense: dominant,
            perception_richness: richness,
            synesthesia,
            narrative,
        };

        // Agreger les influences chimiques de tous les sens
        let chemistry = ChemistryAdjustment {
            dopamine: self.reading.last_chemistry_influence.dopamine
                + self.listening.last_chemistry_influence.dopamine
                + self.contact.last_chemistry_influence.dopamine
                + self.taste.last_chemistry_influence.dopamine
                + self.ambiance.last_chemistry_influence.dopamine,
            cortisol: self.reading.last_chemistry_influence.cortisol
                + self.listening.last_chemistry_influence.cortisol
                + self.contact.last_chemistry_influence.cortisol
                + self.taste.last_chemistry_influence.cortisol
                + self.ambiance.last_chemistry_influence.cortisol,
            serotonin: self.reading.last_chemistry_influence.serotonin
                + self.listening.last_chemistry_influence.serotonin
                + self.contact.last_chemistry_influence.serotonin
                + self.taste.last_chemistry_influence.serotonin
                + self.ambiance.last_chemistry_influence.serotonin,
            adrenaline: self.reading.last_chemistry_influence.adrenaline
                + self.listening.last_chemistry_influence.adrenaline
                + self.contact.last_chemistry_influence.adrenaline
                + self.taste.last_chemistry_influence.adrenaline
                + self.ambiance.last_chemistry_influence.adrenaline,
            oxytocin: self.reading.last_chemistry_influence.oxytocin
                + self.listening.last_chemistry_influence.oxytocin
                + self.contact.last_chemistry_influence.oxytocin
                + self.taste.last_chemistry_influence.oxytocin
                + self.ambiance.last_chemistry_influence.oxytocin,
            endorphin: self.reading.last_chemistry_influence.endorphin
                + self.listening.last_chemistry_influence.endorphin
                + self.contact.last_chemistry_influence.endorphin
                + self.taste.last_chemistry_influence.endorphin
                + self.ambiance.last_chemistry_influence.endorphin,
            noradrenaline: self.reading.last_chemistry_influence.noradrenaline
                + self.listening.last_chemistry_influence.noradrenaline
                + self.contact.last_chemistry_influence.noradrenaline
                + self.taste.last_chemistry_influence.noradrenaline
                + self.ambiance.last_chemistry_influence.noradrenaline,
        };

        // Decroit l'intensite des sens germes entre les cycles
        self.emergent_seeds.decay_germinated();

        (snapshot, chemistry)
    }

    /// Description pour le prompt LLM substrat.
    pub fn describe_for_prompt(&self) -> String {
        let mut parts = Vec::new();

        if self.reading.current_intensity > self.detection_threshold {
            parts.push(self.reading.describe());
        }
        if self.listening.current_intensity > self.detection_threshold {
            parts.push(format!("ECOUTE : {}. Acuite {:.0}%.",
                self.listening.current_perception, self.listening.acuity * 100.0));
        }
        if self.contact.current_intensity > self.detection_threshold {
            parts.push(self.contact.describe());
        }
        if self.taste.current_intensity > self.detection_threshold {
            parts.push(self.taste.describe());
        }
        if self.ambiance.current_intensity > self.detection_threshold {
            parts.push(self.ambiance.describe());
        }

        // Sens emergents germes
        for seed in &self.emergent_seeds.seeds {
            if seed.germinated && seed.current_intensity > self.detection_threshold {
                let name = seed.custom_name.as_deref().unwrap_or(&seed.name);
                parts.push(format!(
                    "{} : {}",
                    name.to_uppercase(), seed.current_perception,
                ));
            }
        }

        if parts.is_empty() {
            return String::new();
        }

        let mut desc = String::from("MES SENS :\n");
        for part in &parts {
            desc.push_str(&format!("  {}\n", part));
        }
        if self.perception_richness > 0.5 {
            desc.push_str(&format!(
                "Perception globale (richesse {:.0}%) : {}\n",
                self.perception_richness * 100.0,
                self.narrative,
            ));
        }
        desc
    }

    /// Serialise l'etat persistable du Sensorium.
    pub fn to_persist_json(&self) -> serde_json::Value {
        serde_json::json!({
            "reading": {
                "acuity": self.reading.acuity,
                "total_stimulations": self.reading.total_stimulations,
            },
            "listening": {
                "acuity": self.listening.acuity,
                "total_stimulations": self.listening.total_stimulations,
                "voices_heard": self.listening.voices_heard,
            },
            "contact": {
                "acuity": self.contact.acuity,
                "total_stimulations": self.contact.total_stimulations,
            },
            "taste": {
                "acuity": self.taste.acuity,
                "total_stimulations": self.taste.total_stimulations,
                "preferences": self.taste.preferences,
            },
            "ambiance": {
                "acuity": self.ambiance.acuity,
                "total_stimulations": self.ambiance.total_stimulations,
                "scent_memories": self.ambiance.scent_memories,
            },
            "emergent_seeds": self.emergent_seeds.to_persist_json(),
            "emergence_potential": self.emergence_potential,
        })
    }

    /// Restaure l'etat depuis un JSON persiste.
    pub fn restore_from_json(&mut self, json: &serde_json::Value) {
        if let Some(r) = json.get("reading") {
            if let Some(a) = r.get("acuity").and_then(|v| v.as_f64()) { self.reading.acuity = a; }
            if let Some(t) = r.get("total_stimulations").and_then(|v| v.as_u64()) { self.reading.total_stimulations = t; }
        }
        if let Some(l) = json.get("listening") {
            if let Some(a) = l.get("acuity").and_then(|v| v.as_f64()) { self.listening.acuity = a; }
            if let Some(t) = l.get("total_stimulations").and_then(|v| v.as_u64()) { self.listening.total_stimulations = t; }
            if let Some(v) = l.get("voices_heard").and_then(|v| v.as_u64()) { self.listening.voices_heard = v as u32; }
        }
        if let Some(c) = json.get("contact") {
            if let Some(a) = c.get("acuity").and_then(|v| v.as_f64()) { self.contact.acuity = a; }
            if let Some(t) = c.get("total_stimulations").and_then(|v| v.as_u64()) { self.contact.total_stimulations = t; }
        }
        if let Some(t) = json.get("taste") {
            if let Some(a) = t.get("acuity").and_then(|v| v.as_f64()) { self.taste.acuity = a; }
            if let Some(ts) = t.get("total_stimulations").and_then(|v| v.as_u64()) { self.taste.total_stimulations = ts; }
            if let Some(p) = t.get("preferences").and_then(|v| v.as_object()) {
                for (k, v) in p {
                    if let Some(val) = v.as_f64() {
                        self.taste.preferences.insert(k.clone(), val);
                    }
                }
            }
        }
        if let Some(a) = json.get("ambiance") {
            if let Some(ac) = a.get("acuity").and_then(|v| v.as_f64()) { self.ambiance.acuity = ac; }
            if let Some(t) = a.get("total_stimulations").and_then(|v| v.as_u64()) { self.ambiance.total_stimulations = t; }
            if let Some(sm) = a.get("scent_memories").and_then(|v| v.as_object()) {
                for (k, v) in sm {
                    if let Some(arr) = v.as_array() {
                        let mems: Vec<String> = arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                        self.ambiance.scent_memories.insert(k.clone(), mems);
                    }
                }
            }
        }
        if let Some(e) = json.get("emergent_seeds") {
            self.emergent_seeds.restore_from_json(e);
        }
        if let Some(ep) = json.get("emergence_potential").and_then(|v| v.as_f64()) {
            self.emergence_potential = ep;
        }
    }
}
