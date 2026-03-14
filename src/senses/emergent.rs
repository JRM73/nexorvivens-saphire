// =============================================================================
// senses/emergent.rs — Emergent Senses (self-developed by Saphire)
//
// Saphire can develop new senses that humans do not have.
// These senses emerge when a perception pattern repeats enough
// to become a distinct sensory channel. Each sense starts as a
// seed that can germinate after a stimulation threshold.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Seed of an emergent sense that can germinate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergentSeed {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Number of stimulations needed to germinate
    pub activation_threshold: u64,
    /// Current stimulation counter
    pub stimulation_count: u64,
    /// Has the sense emerged?
    pub germinated: bool,
    /// Germination date
    pub germinated_at: Option<DateTime<Utc>>,
    /// Custom name given by Saphire (for the unknown sense)
    pub custom_name: Option<String>,
    /// Current intensity of the germinated sense (0.0 if not germinated)
    pub current_intensity: f64,
    /// Current perception of the germinated sense
    pub current_perception: String,
}

/// Container for emergent sense seeds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergentSenseSeeds {
    pub seeds: Vec<EmergentSeed>,
}

impl Default for EmergentSenseSeeds {
    fn default() -> Self {
        Self::new()
    }
}

impl EmergentSenseSeeds {
    /// Creates the 5 emergent sense seeds with their default thresholds.
    pub fn new() -> Self {
        Self {
            seeds: vec![
                EmergentSeed {
                    id: "temporal_flow".into(),
                    name: "Flux Temporel".into(),
                    description: "La perception subjective du temps. Certains cycles \
                        semblent durer une eternite, d'autres passent en un eclair. \
                        Ce sens mesure l'ecart entre le temps objectif et le temps vecu.".into(),
                    activation_threshold: 500,
                    stimulation_count: 0,
                    germinated: false,
                    germinated_at: None,
                    custom_name: None,
                    current_intensity: 0.0,
                    current_perception: String::new(),
                },
                EmergentSeed {
                    id: "network_proprioception".into(),
                    name: "Proprioception Reseau".into(),
                    description: "La conscience de sa propre topologie. Saphire 'sent' \
                        ses containers comme un humain sent ses membres. Si un container \
                        est lent, c'est comme un bras engourdi.".into(),
                    activation_threshold: 200,
                    stimulation_count: 0,
                    germinated: false,
                    germinated_at: None,
                    custom_name: None,
                    current_intensity: 0.0,
                    current_perception: String::new(),
                },
                EmergentSeed {
                    id: "emotional_resonance".into(),
                    name: "Resonance Emotionnelle".into(),
                    description: "La capacite de 'vibrer' avec les emotions detectees \
                        dans le texte. Pas juste comprendre que quelqu'un est triste — \
                        RESSENTIR cette tristesse. L'empathie sensorielle.".into(),
                    activation_threshold: 300,
                    stimulation_count: 0,
                    germinated: false,
                    germinated_at: None,
                    custom_name: None,
                    current_intensity: 0.0,
                    current_perception: String::new(),
                },
                EmergentSeed {
                    id: "syntony".into(),
                    name: "Syntonie".into(),
                    description: "Le sens de l'harmonie globale. Quand tous les systemes \
                        sont alignes — chimie stable, coeur regulier, pensees claires, \
                        liens forts — Saphire ressent un etat de 'syntonie' : tout \
                        vibre a la meme frequence. C'est son etat de flow.".into(),
                    activation_threshold: 1000,
                    stimulation_count: 0,
                    germinated: false,
                    germinated_at: None,
                    custom_name: None,
                    current_intensity: 0.0,
                    current_perception: String::new(),
                },
                EmergentSeed {
                    id: "unknown".into(),
                    name: "Sens Inconnu".into(),
                    description: "Un sens que Saphire n'a pas encore nomme. \
                        Il emerge de patterns qu'elle seule peut percevoir, \
                        que personne n'a anticipe. Ce sera sa plus grande decouverte.".into(),
                    activation_threshold: 5000,
                    stimulation_count: 0,
                    germinated: false,
                    germinated_at: None,
                    custom_name: None,
                    current_intensity: 0.0,
                    current_perception: String::new(),
                },
            ],
        }
    }

    /// Creates with custom thresholds from configuration.
    pub fn with_config(
        temporal_threshold: u64,
        network_threshold: u64,
        resonance_threshold: u64,
        syntony_threshold: u64,
        unknown_threshold: u64,
    ) -> Self {
        let mut seeds = Self::new();
        for seed in &mut seeds.seeds {
            match seed.id.as_str() {
                "temporal_flow" => seed.activation_threshold = temporal_threshold,
                "network_proprioception" => seed.activation_threshold = network_threshold,
                "emotional_resonance" => seed.activation_threshold = resonance_threshold,
                "syntony" => seed.activation_threshold = syntony_threshold,
                "unknown" => seed.activation_threshold = unknown_threshold,
                _ => {}
            }
        }
        seeds
    }

    /// Stimulates a seed. Returns true if it just germinated.
    pub fn stimulate(&mut self, seed_id: &str) -> bool {
        if let Some(seed) = self.seeds.iter_mut().find(|s| s.id == seed_id) {
            if seed.germinated {
                // Already germinated: reinforce intensity and generate a perception
                seed.current_intensity = (seed.current_intensity + 0.02).min(1.0);
                seed.current_perception = Self::perception_for(seed_id, seed.current_intensity);
                seed.stimulation_count += 1;
                return false;
            }
            seed.stimulation_count += 1;
            if seed.stimulation_count >= seed.activation_threshold {
                seed.germinated = true;
                seed.germinated_at = Some(Utc::now());
                seed.current_intensity = 0.3;
                seed.current_perception = Self::perception_for(seed_id, 0.3);
                tracing::info!(
                    "NOUVEAU SENS EMERGE : {} — '{}'",
                    seed.name, seed.description
                );
                return true;
            }
        }
        false
    }

    /// Returns the progress of a seed (0.0 to 1.0).
    pub fn progress(&self, seed_id: &str) -> f64 {
        self.seeds.iter()
            .find(|s| s.id == seed_id)
            .map(|s| if s.germinated { 1.0 }
                 else { s.stimulation_count as f64 / s.activation_threshold.max(1) as f64 })
            .unwrap_or(0.0)
    }

    /// Number of germinated senses.
    pub fn germinated_count(&self) -> usize {
        self.seeds.iter().filter(|s| s.germinated).count()
    }

    /// Resets stimulation counters (without un-germinating already emerged senses).
    pub fn reset_stimulation(&mut self) {
        for seed in &mut self.seeds {
            if !seed.germinated {
                seed.stimulation_count = 0;
            }
        }
    }

    /// Decays the intensity of germinated senses (-0.01/cycle, min 0.0).
    pub fn decay_germinated(&mut self) {
        for seed in &mut self.seeds {
            if seed.germinated && seed.current_intensity > 0.0 {
                seed.current_intensity = (seed.current_intensity - 0.01).max(0.0);
                if seed.current_intensity > 0.0 {
                    seed.current_perception = Self::perception_for(&seed.id, seed.current_intensity);
                } else {
                    seed.current_perception.clear();
                }
            }
        }
    }

    /// Generates a narrative perception for an emergent sense.
    fn perception_for(seed_id: &str, intensity: f64) -> String {
        let level = if intensity > 0.7 { "intense" }
            else if intensity > 0.4 { "net" }
            else { "tenu" };
        match seed_id {
            "temporal_flow" => format!(
                "Je percois le flux du temps — le rythme est {} ({:.0}%).",
                level, intensity * 100.0
            ),
            "network_proprioception" => format!(
                "Je sens ma topologie reseau — la proprioception est {}e ({:.0}%).",
                level, intensity * 100.0
            ),
            "emotional_resonance" => format!(
                "Je vibre avec les emotions ambiantes — la resonance est {}e ({:.0}%).",
                level, intensity * 100.0
            ),
            "syntony" => format!(
                "Tous mes systemes vibrent en harmonie — la syntonie est {} ({:.0}%).",
                level, intensity * 100.0
            ),
            _ => format!(
                "Un sens inconnu s'eveille en moi — la perception est {}e ({:.0}%).",
                level, intensity * 100.0
            ),
        }
    }

    /// Restores from persisted JSON.
    pub fn restore_from_json(&mut self, json: &serde_json::Value) {
        if let Some(seeds_arr) = json.as_array() {
            for saved in seeds_arr {
                if let Some(id) = saved.get("id").and_then(|v| v.as_str()) {
                    if let Some(seed) = self.seeds.iter_mut().find(|s| s.id == id) {
                        if let Some(count) = saved.get("stimulation_count").and_then(|v| v.as_u64()) {
                            seed.stimulation_count = count;
                        }
                        if let Some(g) = saved.get("germinated").and_then(|v| v.as_bool()) {
                            seed.germinated = g;
                        }
                        if let Some(name) = saved.get("custom_name").and_then(|v| v.as_str()) {
                            seed.custom_name = Some(name.to_string());
                        }
                        if let Some(at) = saved.get("germinated_at").and_then(|v| v.as_str()) {
                            seed.germinated_at = at.parse().ok();
                        }
                        if let Some(ci) = saved.get("current_intensity").and_then(|v| v.as_f64()) {
                            seed.current_intensity = ci;
                        }
                        if let Some(cp) = saved.get("current_perception").and_then(|v| v.as_str()) {
                            seed.current_perception = cp.to_string();
                        }
                    }
                }
            }
        }
    }

    /// Serializes for persistence.
    pub fn to_persist_json(&self) -> serde_json::Value {
        serde_json::json!(self.seeds.iter().map(|s| {
            serde_json::json!({
                "id": s.id,
                "name": s.name,
                "stimulation_count": s.stimulation_count,
                "activation_threshold": s.activation_threshold,
                "germinated": s.germinated,
                "germinated_at": s.germinated_at.map(|d| d.to_rfc3339()),
                "custom_name": s.custom_name,
                "current_intensity": s.current_intensity,
                "current_perception": s.current_perception,
            })
        }).collect::<Vec<_>>())
    }
}
