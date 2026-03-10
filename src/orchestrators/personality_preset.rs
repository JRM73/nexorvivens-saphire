// =============================================================================
// orchestrators/personality_preset.rs — Orchestrateur de Presets de Personnalite
//
// Role : Charge et applique des archetypes de personnalite (philosophe, artiste,
// scientifique, empathique, stoique, aventurier, mystique, mentor, rebelle)
// sous forme de presets TOML embarques. Chaque preset surcharge les baselines
// chimiques et les parametres des orchestrateurs, et injecte un contexte de
// personnalite dans le prompt LLM.
//
// Difference avec les profils cognitifs : les profils cognitifs simulent des
// conditions neurologiques (TDAH, autisme, etc.), les presets de personnalite
// definissent des caracteres/temperaments. Les deux systemes sont orthogonaux
// et peuvent etre actifs simultanement.
//
// Pattern identique aux orchestrateurs existants :
//   - new() : construction depuis la config
//   - load_preset() : charge un preset par son ID
//   - tick() : transitions douces (pas de cycle bipolaire)
//   - describe_for_prompt() : contexte LLM avec prompt_personality
//   - to_status_json() : etat JSON pour dashboard/API
// =============================================================================

use std::collections::HashMap;
use serde::Deserialize;

use crate::neurochemistry::NeuroBaselines;

// ─── Presets embarques via include_str! ──────────────────────────────────────

const EMBEDDED_PRESETS: &[(&str, &str)] = &[
    ("saphire", include_str!("../../personalities/saphire.toml")),
    ("philosophe", include_str!("../../personalities/philosophe.toml")),
    ("artiste", include_str!("../../personalities/artiste.toml")),
    ("scientifique", include_str!("../../personalities/scientifique.toml")),
    ("empathique", include_str!("../../personalities/empathique.toml")),
    ("stoique", include_str!("../../personalities/stoique.toml")),
    ("aventurier", include_str!("../../personalities/aventurier.toml")),
    ("mystique", include_str!("../../personalities/mystique.toml")),
    ("mentor", include_str!("../../personalities/mentor.toml")),
    ("rebelle", include_str!("../../personalities/rebelle.toml")),
];

// ─── Structures de deserialization TOML ─────────────────────────────────────

/// Structure brute d'un fichier preset TOML (deserialization partielle).
#[derive(Debug, Deserialize, Default)]
struct RawPreset {
    #[serde(default)]
    profile: RawPresetMeta,
    #[serde(default)]
    personality: RawPersonality,
    #[serde(default)]
    interests: Option<RawInterests>,
    #[serde(default)]
    feedback: RawFeedback,
    #[serde(default)]
    consensus: RawConsensus,
    #[serde(default)]
    attention: RawAttention,
    #[serde(default)]
    desires: RawDesires,
    #[serde(default)]
    learning: RawLearning,
    #[serde(default)]
    healing: RawHealing,
    #[serde(default)]
    sleep: RawSleep,
    #[serde(default)]
    thought_weights: Option<HashMap<String, f64>>,
}

#[derive(Debug, Deserialize, Default)]
struct RawPresetMeta {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    prompt_personality: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct RawInterests {
    #[serde(default)]
    initial_topics: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
struct RawPersonality {
    baseline_dopamine: Option<f64>,
    baseline_serotonin: Option<f64>,
    baseline_noradrenaline: Option<f64>,
    baseline_oxytocin: Option<f64>,
    baseline_cortisol: Option<f64>,
    baseline_endorphin: Option<f64>,
    baseline_adrenaline: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
struct RawFeedback {
    homeostasis_rate: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
struct RawConsensus {
    threshold_yes: Option<f64>,
    threshold_no: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
struct RawAttention {
    initial_concentration: Option<f64>,
    fatigue_per_cycle: Option<f64>,
    recovery_per_cycle: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
struct RawDesires {
    max_active: Option<usize>,
    min_dopamine_for_birth: Option<f64>,
    max_cortisol_for_birth: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
struct RawLearning {
    cycle_interval: Option<u64>,
    initial_confidence: Option<f64>,
    confirmation_boost: Option<f64>,
    contradiction_penalty: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
struct RawHealing {
    melancholy_threshold_cycles: Option<u64>,
    loneliness_threshold_hours: Option<f64>,
    overload_noradrenaline: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
struct RawSleep {
    sleep_threshold: Option<f64>,
    time_factor_divisor: Option<u64>,
    adrenaline_resistance: Option<f64>,
}

// ─── Structures publiques ───────────────────────────────────────────────────

/// Surcharges d'un preset de personnalite — seuls les champs Some() sont appliques.
#[derive(Debug, Clone, Default)]
pub struct PersonalityOverrides {
    // Chimie baselines
    pub baseline_dopamine: Option<f64>,
    pub baseline_serotonin: Option<f64>,
    pub baseline_noradrenaline: Option<f64>,
    pub baseline_oxytocin: Option<f64>,
    pub baseline_cortisol: Option<f64>,
    pub baseline_endorphin: Option<f64>,
    pub baseline_adrenaline: Option<f64>,
    // Feedback
    pub homeostasis_rate: Option<f64>,
    // Consensus
    pub threshold_yes: Option<f64>,
    pub threshold_no: Option<f64>,
    // Attention
    pub initial_concentration: Option<f64>,
    pub fatigue_per_cycle: Option<f64>,
    pub recovery_per_cycle: Option<f64>,
    // Desirs
    pub desires_max_active: Option<usize>,
    pub desires_min_dopamine: Option<f64>,
    pub desires_max_cortisol: Option<f64>,
    // Apprentissage
    pub learning_cycle_interval: Option<u64>,
    pub learning_initial_confidence: Option<f64>,
    pub learning_confirmation_boost: Option<f64>,
    pub learning_contradiction_penalty: Option<f64>,
    // Guerison
    pub healing_melancholy_threshold: Option<u64>,
    pub healing_loneliness_hours: Option<f64>,
    pub healing_overload_noradrenaline: Option<f64>,
    // Sommeil
    pub sleep_threshold: Option<f64>,
    pub sleep_time_factor_divisor: Option<u64>,
    pub sleep_adrenaline_resistance: Option<f64>,
    // Poids des pensees
    pub thought_weights: Option<HashMap<String, f64>>,
    // ─── Specifiques personnalite ───
    /// Description injectee dans le prompt LLM pour guider le ton et le style
    pub prompt_personality: Option<String>,
    /// Sujets d'interet initiaux (override initial_topics)
    pub interests: Option<Vec<String>>,
}

/// Descripteur complet d'un preset de personnalite.
#[derive(Debug, Clone)]
pub struct PersonalityDescriptor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub overrides: PersonalityOverrides,
}

// ─── Orchestrateur ──────────────────────────────────────────────────────────

/// Orchestrateur de presets de personnalite (archetypes de caractere).
///
/// Charge des presets TOML, applique les surcharges sur les baselines et
/// parametres, gere les transitions douces. Pas de cycle bipolaire
/// (specifique aux profils cognitifs).
pub struct PersonalityPresetOrchestrator {
    /// Preset actuellement actif (None = aucun charge)
    pub active_preset: Option<PersonalityDescriptor>,
    /// IDs des presets disponibles
    pub available_presets: Vec<String>,
    /// Transition en cours entre deux presets
    pub transition_in_progress: bool,
    /// Progression de la transition (0.0 → 1.0)
    pub transition_progress: f64,
    /// Baselines cibles de la transition
    transition_target_baselines: Option<[f64; 7]>,
    /// Baselines de depart de la transition
    transition_source_baselines: Option<[f64; 7]>,
    /// Nombre total de cycles pour la transition
    transition_total_cycles: u64,
    /// Cycles ecoules dans la transition
    transition_elapsed_cycles: u64,
    // Config
    /// Module active ou non
    pub enabled: bool,
    /// Dossier des presets custom
    pub personalities_dir: String,
    /// Nombre de cycles pour une transition douce
    pub transition_cycles: u64,
}

impl PersonalityPresetOrchestrator {
    /// Cree un nouvel orchestrateur de presets de personnalite.
    pub fn new(enabled: bool, active: &str, personalities_dir: &str, transition_cycles: u64) -> Self {
        let available = EMBEDDED_PRESETS.iter()
            .map(|(id, _)| id.to_string())
            .collect();

        let mut orch = Self {
            active_preset: None,
            available_presets: available,
            transition_in_progress: false,
            transition_progress: 0.0,
            transition_target_baselines: None,
            transition_source_baselines: None,
            transition_total_cycles: transition_cycles,
            transition_elapsed_cycles: 0,
            enabled,
            personalities_dir: personalities_dir.to_string(),
            transition_cycles,
        };

        // Charger le preset initial
        if enabled && active != "saphire" {
            if let Ok(preset) = orch.parse_preset(active) {
                orch.active_preset = Some(preset);
                tracing::info!("Preset de personnalite initial : {}", active);
            }
        } else if enabled {
            if let Ok(preset) = orch.parse_preset("saphire") {
                orch.active_preset = Some(preset);
            }
        }

        orch
    }

    /// Parse un preset embarque par son ID.
    fn parse_preset(&self, id: &str) -> Result<PersonalityDescriptor, String> {
        // Chercher dans les presets embarques
        let toml_str = EMBEDDED_PRESETS.iter()
            .find(|(eid, _)| *eid == id)
            .map(|(_, content)| *content);

        // Si pas embarque, tenter le filesystem
        let toml_content = if let Some(content) = toml_str {
            content.to_string()
        } else {
            let path = format!("{}/{}.toml", self.personalities_dir, id);
            std::fs::read_to_string(&path)
                .map_err(|e| format!("Preset '{}' introuvable : {}", id, e))?
        };

        let raw: RawPreset = toml::from_str(&toml_content)
            .map_err(|e| format!("Erreur de parsing du preset '{}' : {}", id, e))?;

        let overrides = Self::raw_to_overrides(&raw);

        Ok(PersonalityDescriptor {
            id: id.to_string(),
            name: raw.profile.name,
            description: raw.profile.description,
            category: raw.profile.category,
            overrides,
        })
    }

    /// Convertit les donnees brutes TOML en PersonalityOverrides.
    fn raw_to_overrides(raw: &RawPreset) -> PersonalityOverrides {
        PersonalityOverrides {
            baseline_dopamine: raw.personality.baseline_dopamine,
            baseline_serotonin: raw.personality.baseline_serotonin,
            baseline_noradrenaline: raw.personality.baseline_noradrenaline,
            baseline_oxytocin: raw.personality.baseline_oxytocin,
            baseline_cortisol: raw.personality.baseline_cortisol,
            baseline_endorphin: raw.personality.baseline_endorphin,
            baseline_adrenaline: raw.personality.baseline_adrenaline,
            homeostasis_rate: raw.feedback.homeostasis_rate,
            threshold_yes: raw.consensus.threshold_yes,
            threshold_no: raw.consensus.threshold_no,
            initial_concentration: raw.attention.initial_concentration,
            fatigue_per_cycle: raw.attention.fatigue_per_cycle,
            recovery_per_cycle: raw.attention.recovery_per_cycle,
            desires_max_active: raw.desires.max_active,
            desires_min_dopamine: raw.desires.min_dopamine_for_birth,
            desires_max_cortisol: raw.desires.max_cortisol_for_birth,
            learning_cycle_interval: raw.learning.cycle_interval,
            learning_initial_confidence: raw.learning.initial_confidence,
            learning_confirmation_boost: raw.learning.confirmation_boost,
            learning_contradiction_penalty: raw.learning.contradiction_penalty,
            healing_melancholy_threshold: raw.healing.melancholy_threshold_cycles,
            healing_loneliness_hours: raw.healing.loneliness_threshold_hours,
            healing_overload_noradrenaline: raw.healing.overload_noradrenaline,
            sleep_threshold: raw.sleep.sleep_threshold,
            sleep_time_factor_divisor: raw.sleep.time_factor_divisor,
            sleep_adrenaline_resistance: raw.sleep.adrenaline_resistance,
            thought_weights: raw.thought_weights.clone(),
            // Champs specifiques personnalite
            prompt_personality: raw.profile.prompt_personality.clone()
                .filter(|s| !s.is_empty()),
            interests: raw.interests.as_ref()
                .map(|i| i.initial_topics.clone())
                .filter(|v| !v.is_empty()),
        }
    }

    /// Charge un preset par son ID. Retourne le descripteur ou une erreur.
    pub fn load_preset(&mut self, id: &str) -> Result<PersonalityDescriptor, String> {
        let preset = self.parse_preset(id)?;
        self.active_preset = Some(preset.clone());
        tracing::info!("Preset de personnalite charge : {} ({})", preset.name, id);
        Ok(preset)
    }

    /// Demarre une transition douce vers les baselines cibles.
    /// Les baselines actuelles convergent vers les cibles sur N cycles.
    pub fn start_transition(&mut self, current_baselines: &NeuroBaselines, target: &PersonalityOverrides) {
        let source = [
            current_baselines.dopamine,
            current_baselines.cortisol,
            current_baselines.serotonin,
            current_baselines.adrenaline,
            current_baselines.oxytocin,
            current_baselines.endorphin,
            current_baselines.noradrenaline,
        ];

        let target_arr = [
            target.baseline_dopamine.unwrap_or(source[0]),
            target.baseline_cortisol.unwrap_or(source[1]),
            target.baseline_serotonin.unwrap_or(source[2]),
            target.baseline_adrenaline.unwrap_or(source[3]),
            target.baseline_oxytocin.unwrap_or(source[4]),
            target.baseline_endorphin.unwrap_or(source[5]),
            target.baseline_noradrenaline.unwrap_or(source[6]),
        ];

        self.transition_source_baselines = Some(source);
        self.transition_target_baselines = Some(target_arr);
        self.transition_elapsed_cycles = 0;
        self.transition_total_cycles = self.transition_cycles;
        self.transition_in_progress = true;
        self.transition_progress = 0.0;
    }

    /// Tick : avance les transitions douces.
    /// Appelee chaque cycle dans phase_orchestrators().
    pub fn tick(&mut self, baselines: &mut NeuroBaselines) {
        if !self.enabled {
            return;
        }

        // Transition douce en cours
        if self.transition_in_progress {
            self.transition_elapsed_cycles += 1;
            let t = (self.transition_elapsed_cycles as f64) / (self.transition_total_cycles as f64);
            self.transition_progress = t.min(1.0);

            if let (Some(source), Some(target)) = (
                self.transition_source_baselines,
                self.transition_target_baselines,
            ) {
                let lerp = |a: f64, b: f64| a + (b - a) * self.transition_progress;
                baselines.dopamine = lerp(source[0], target[0]);
                baselines.cortisol = lerp(source[1], target[1]);
                baselines.serotonin = lerp(source[2], target[2]);
                baselines.adrenaline = lerp(source[3], target[3]);
                baselines.oxytocin = lerp(source[4], target[4]);
                baselines.endorphin = lerp(source[5], target[5]);
                baselines.noradrenaline = lerp(source[6], target[6]);
            }

            if self.transition_progress >= 1.0 {
                self.transition_in_progress = false;
                tracing::info!("Transition de preset de personnalite terminee");
            }
        }
    }

    /// Genere le contexte pour le prompt LLM.
    /// Utilise prompt_personality du preset pour guider le ton.
    pub fn describe_for_prompt(&self) -> String {
        let preset = match &self.active_preset {
            Some(p) => p,
            None => return String::new(),
        };

        // Utiliser le prompt_personality s'il existe, sinon la description
        let personality_text = preset.overrides.prompt_personality.as_deref()
            .unwrap_or(&preset.description);

        let mut desc = format!(
            "PERSONNALITE : {} — {}",
            preset.name, personality_text
        );

        if self.transition_in_progress {
            desc.push_str(&format!(
                " | Transition en cours : {:.0}%",
                self.transition_progress * 100.0
            ));
        }

        desc
    }

    /// Retourne l'etat JSON pour le dashboard et l'API.
    pub fn to_status_json(&self) -> serde_json::Value {
        let preset_json = self.active_preset.as_ref().map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "description": p.description,
                "category": p.category,
                "has_prompt_personality": p.overrides.prompt_personality.is_some(),
                "has_interests": p.overrides.interests.is_some(),
            })
        });

        serde_json::json!({
            "enabled": self.enabled,
            "active_preset": preset_json,
            "available_presets": self.available_presets,
            "transition": {
                "in_progress": self.transition_in_progress,
                "progress": self.transition_progress,
                "elapsed_cycles": self.transition_elapsed_cycles,
                "total_cycles": self.transition_total_cycles,
            },
        })
    }

    /// Liste les presets disponibles avec leurs metadonnees.
    pub fn list_presets(&self) -> Vec<serde_json::Value> {
        EMBEDDED_PRESETS.iter()
            .filter_map(|(id, content)| {
                let raw: RawPreset = toml::from_str(content).ok()?;
                Some(serde_json::json!({
                    "id": id,
                    "name": raw.profile.name,
                    "description": raw.profile.description,
                    "category": raw.profile.category,
                    "has_prompt_personality": raw.profile.prompt_personality
                        .as_ref().map(|s| !s.is_empty()).unwrap_or(false),
                }))
            })
            .collect()
    }

    /// Compare deux presets et retourne les differences.
    pub fn compare_presets(&self, id_a: &str, id_b: &str) -> Result<serde_json::Value, String> {
        let preset_a = self.parse_preset(id_a)?;
        let preset_b = self.parse_preset(id_b)?;

        let ov_a = &preset_a.overrides;
        let ov_b = &preset_b.overrides;

        let mut diffs = Vec::new();

        macro_rules! cmp_f64 {
            ($field:ident, $label:expr) => {
                let va = ov_a.$field;
                let vb = ov_b.$field;
                if va != vb {
                    diffs.push(serde_json::json!({
                        "param": $label,
                        "preset_a": va,
                        "preset_b": vb,
                    }));
                }
            };
        }

        macro_rules! cmp_usize {
            ($field:ident, $label:expr) => {
                let va = ov_a.$field;
                let vb = ov_b.$field;
                if va != vb {
                    diffs.push(serde_json::json!({
                        "param": $label,
                        "preset_a": va,
                        "preset_b": vb,
                    }));
                }
            };
        }

        macro_rules! cmp_u64 {
            ($field:ident, $label:expr) => {
                let va = ov_a.$field;
                let vb = ov_b.$field;
                if va != vb {
                    diffs.push(serde_json::json!({
                        "param": $label,
                        "preset_a": va,
                        "preset_b": vb,
                    }));
                }
            };
        }

        cmp_f64!(baseline_dopamine, "baseline_dopamine");
        cmp_f64!(baseline_serotonin, "baseline_serotonin");
        cmp_f64!(baseline_noradrenaline, "baseline_noradrenaline");
        cmp_f64!(baseline_oxytocin, "baseline_oxytocin");
        cmp_f64!(baseline_cortisol, "baseline_cortisol");
        cmp_f64!(baseline_endorphin, "baseline_endorphin");
        cmp_f64!(baseline_adrenaline, "baseline_adrenaline");
        cmp_f64!(homeostasis_rate, "homeostasis_rate");
        cmp_f64!(threshold_yes, "threshold_yes");
        cmp_f64!(threshold_no, "threshold_no");
        cmp_f64!(initial_concentration, "initial_concentration");
        cmp_f64!(fatigue_per_cycle, "fatigue_per_cycle");
        cmp_f64!(recovery_per_cycle, "recovery_per_cycle");
        cmp_usize!(desires_max_active, "desires_max_active");
        cmp_f64!(desires_min_dopamine, "desires_min_dopamine");
        cmp_f64!(desires_max_cortisol, "desires_max_cortisol");
        cmp_u64!(learning_cycle_interval, "learning_cycle_interval");
        cmp_f64!(learning_initial_confidence, "learning_initial_confidence");
        cmp_f64!(learning_confirmation_boost, "learning_confirmation_boost");
        cmp_f64!(learning_contradiction_penalty, "learning_contradiction_penalty");
        cmp_u64!(healing_melancholy_threshold, "healing_melancholy_threshold");
        cmp_f64!(healing_loneliness_hours, "healing_loneliness_hours");
        cmp_f64!(healing_overload_noradrenaline, "healing_overload_noradrenaline");
        cmp_f64!(sleep_threshold, "sleep_threshold");
        cmp_u64!(sleep_time_factor_divisor, "sleep_time_factor_divisor");
        cmp_f64!(sleep_adrenaline_resistance, "sleep_adrenaline_resistance");

        // Comparer prompt_personality
        if ov_a.prompt_personality != ov_b.prompt_personality {
            diffs.push(serde_json::json!({
                "param": "prompt_personality",
                "preset_a": ov_a.prompt_personality,
                "preset_b": ov_b.prompt_personality,
            }));
        }

        // Comparer interests
        if ov_a.interests != ov_b.interests {
            diffs.push(serde_json::json!({
                "param": "interests",
                "preset_a": ov_a.interests,
                "preset_b": ov_b.interests,
            }));
        }

        Ok(serde_json::json!({
            "preset_a": { "id": id_a, "name": preset_a.name },
            "preset_b": { "id": id_b, "name": preset_b.name },
            "differences": diffs,
            "total_differences": diffs.len(),
        }))
    }
}
