// =============================================================================
// orchestrators/cognitive_profile.rs — Cognitive Profile Orchestrator
//
// Role: Loads and applies neurodivergent cognitive profiles (ADHD, autism,
// GAD, HPI, bipolar, OCD) as embedded TOML presets. Each profile overrides
// chemical baselines and existing orchestrator parameters. Manages smooth
// transitions and bipolar cycles.
//
// Same pattern as existing orchestrators (attention, healing, etc.):
//   - new(): construction from config
//   - load_profile(): loads a profile by its ID
//   - tick(): smooth transitions + bipolar cycles
//   - describe_for_prompt(): LLM context
//   - to_status_json(): JSON state for dashboard/API
// =============================================================================

use std::collections::HashMap;
use serde::Deserialize;

use crate::neurochemistry::NeuroBaselines;

// --- Embedded profiles via include_str! ---------------------------------------

const EMBEDDED_PROFILES: &[(&str, &str)] = &[
    ("neurotypique", include_str!("../../profiles/neurotypique.toml")),
    ("tdah", include_str!("../../profiles/tdah.toml")),
    ("tdah-inattentif", include_str!("../../profiles/tdah-inattentif.toml")),
    ("tdah-hyperactif", include_str!("../../profiles/tdah-hyperactif.toml")),
    ("autisme", include_str!("../../profiles/autisme.toml")),
    ("anxiete", include_str!("../../profiles/anxiete.toml")),
    ("hpi", include_str!("../../profiles/hpi.toml")),
    ("bipolaire", include_str!("../../profiles/bipolaire.toml")),
    ("toc", include_str!("../../profiles/toc.toml")),
];

// --- TOML deserialization structures ------------------------------------------

/// Raw structure of a TOML preset file (partial deserialization).
#[derive(Debug, Deserialize, Default)]
struct RawProfile {
    #[serde(default)]
    profile: RawProfileMeta,
    #[serde(default)]
    personality: RawPersonality,
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
    #[serde(default)]
    bipolar: Option<RawBipolar>,
}

#[derive(Debug, Deserialize, Default)]
struct RawProfileMeta {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    references: Vec<String>,
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

#[derive(Debug, Deserialize, Default)]
struct RawBipolar {
    cycle_length_cycles: Option<u64>,
    manie_ratio: Option<f64>,
    depression_ratio: Option<f64>,
    #[serde(default)]
    manie: RawBipolarPhaseOverrides,
    #[serde(default)]
    depression: RawBipolarPhaseOverrides,
}

#[derive(Debug, Deserialize, Default)]
struct RawBipolarPhaseOverrides {
    baseline_dopamine: Option<f64>,
    baseline_serotonin: Option<f64>,
    baseline_noradrenaline: Option<f64>,
    baseline_oxytocin: Option<f64>,
    baseline_cortisol: Option<f64>,
    baseline_endorphin: Option<f64>,
    baseline_adrenaline: Option<f64>,
    sleep_threshold: Option<f64>,
    fatigue_per_cycle: Option<f64>,
    max_active: Option<usize>,
    min_dopamine_for_birth: Option<f64>,
}

// --- Public structures --------------------------------------------------------

/// Cognitive profile overrides — only Some() fields are applied.
#[derive(Debug, Clone, Default)]
pub struct ProfileOverrides {
    // Chemical baselines
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
    // Desires
    pub desires_max_active: Option<usize>,
    pub desires_min_dopamine: Option<f64>,
    pub desires_max_cortisol: Option<f64>,
    // Learning
    pub learning_cycle_interval: Option<u64>,
    pub learning_initial_confidence: Option<f64>,
    pub learning_confirmation_boost: Option<f64>,
    pub learning_contradiction_penalty: Option<f64>,
    // Healing
    pub healing_melancholy_threshold: Option<u64>,
    pub healing_loneliness_hours: Option<f64>,
    pub healing_overload_noradrenaline: Option<f64>,
    // Sleep
    pub sleep_threshold: Option<f64>,
    pub sleep_time_factor_divisor: Option<u64>,
    pub sleep_adrenaline_resistance: Option<f64>,
    // Thought weights
    pub thought_weights: Option<HashMap<String, f64>>,
}

/// Complete descriptor of a cognitive profile.
#[derive(Debug, Clone)]
pub struct ProfileDescriptor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub references: Vec<String>,
    pub overrides: ProfileOverrides,
    pub bipolar_config: Option<BipolarConfig>,
}

/// Bipolar cycle configuration.
#[derive(Debug, Clone)]
pub struct BipolarConfig {
    pub cycle_length_cycles: u64,
    pub manie_ratio: f64,
    pub depression_ratio: f64,
    pub manie_overrides: ProfileOverrides,
    pub depression_overrides: ProfileOverrides,
}

/// Current phase of the bipolar cycle.
#[derive(Debug, Clone, PartialEq)]
pub enum BipolarPhase {
    Euthymie,
    Manie,
    Depression,
}

impl BipolarPhase {
    pub fn as_str(&self) -> &str {
        match self {
            BipolarPhase::Euthymie => "euthymie",
            BipolarPhase::Manie => "manie",
            BipolarPhase::Depression => "depression",
        }
    }
}

// --- Orchestrator -------------------------------------------------------------

/// Neurodivergent cognitive profile orchestrator.
///
/// Loads TOML presets, applies overrides on baselines and parameters,
/// manages smooth transitions and bipolar cycles.
pub struct CognitiveProfileOrchestrator {
    /// Currently active profile (None = none loaded)
    pub active_profile: Option<ProfileDescriptor>,
    /// IDs of available profiles
    pub available_profiles: Vec<String>,
    /// Transition in progress between two profiles
    pub transition_in_progress: bool,
    /// Transition progression (0.0 -> 1.0)
    pub transition_progress: f64,
    /// Target baselines for the transition
    transition_target_baselines: Option<[f64; 7]>,
    /// Source baselines for the transition
    transition_source_baselines: Option<[f64; 7]>,
    /// Total number of cycles for the transition
    transition_total_cycles: u64,
    /// Elapsed cycles in the transition
    transition_elapsed_cycles: u64,
    // Bipolar
    /// Current phase of the bipolar cycle (if applicable)
    pub bipolar_phase: Option<BipolarPhase>,
    /// Cycle counter in the bipolar cycle
    bipolar_cycle_counter: u64,
    // Config
    /// Module enabled or not
    pub enabled: bool,
    /// Custom profiles directory
    pub profiles_dir: String,
    /// Number of cycles for a smooth transition
    pub transition_cycles: u64,
}

impl CognitiveProfileOrchestrator {
    /// Creates a new cognitive profile orchestrator.
    pub fn new(enabled: bool, active: &str, profiles_dir: &str, transition_cycles: u64) -> Self {
        let available = EMBEDDED_PROFILES.iter()
            .map(|(id, _)| id.to_string())
            .collect();

        let mut orch = Self {
            active_profile: None,
            available_profiles: available,
            transition_in_progress: false,
            transition_progress: 0.0,
            transition_target_baselines: None,
            transition_source_baselines: None,
            transition_total_cycles: transition_cycles,
            transition_elapsed_cycles: 0,
            bipolar_phase: None,
            bipolar_cycle_counter: 0,
            enabled,
            profiles_dir: profiles_dir.to_string(),
            transition_cycles,
        };

        // Load the initial profile
        if enabled && active != "neurotypique" {
            if let Ok(profile) = orch.parse_profile(active) {
                orch.active_profile = Some(profile);
                tracing::info!("Profil cognitif initial : {}", active);
            }
        } else if enabled {
            if let Ok(profile) = orch.parse_profile("neurotypique") {
                orch.active_profile = Some(profile);
            }
        }

        orch
    }

    /// Parses an embedded profile by its ID.
    fn parse_profile(&self, id: &str) -> Result<ProfileDescriptor, String> {
        // Search in embedded profiles
        let toml_str = EMBEDDED_PROFILES.iter()
            .find(|(eid, _)| *eid == id)
            .map(|(_, content)| *content);

        // If not embedded, try the filesystem
        let toml_content = if let Some(content) = toml_str {
            content.to_string()
        } else {
            let path = format!("{}/{}.toml", self.profiles_dir, id);
            std::fs::read_to_string(&path)
                .map_err(|e| format!("Profil '{}' introuvable : {}", id, e))?
        };

        let raw: RawProfile = toml::from_str(&toml_content)
            .map_err(|e| format!("Erreur de parsing du profil '{}' : {}", id, e))?;

        let overrides = Self::raw_to_overrides(&raw);
        let bipolar_config = raw.bipolar.map(|bp| Self::raw_to_bipolar_config(&bp));

        Ok(ProfileDescriptor {
            id: id.to_string(),
            name: raw.profile.name,
            description: raw.profile.description,
            category: raw.profile.category,
            references: raw.profile.references,
            overrides,
            bipolar_config,
        })
    }

    /// Converts raw TOML data to ProfileOverrides.
    fn raw_to_overrides(raw: &RawProfile) -> ProfileOverrides {
        ProfileOverrides {
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
        }
    }

    /// Converts raw bipolar data to BipolarConfig.
    fn raw_to_bipolar_config(raw: &RawBipolar) -> BipolarConfig {
        let manie_overrides = Self::bipolar_phase_to_overrides(&raw.manie);
        let depression_overrides = Self::bipolar_phase_to_overrides(&raw.depression);

        BipolarConfig {
            cycle_length_cycles: raw.cycle_length_cycles.unwrap_or(500),
            manie_ratio: raw.manie_ratio.unwrap_or(0.30),
            depression_ratio: raw.depression_ratio.unwrap_or(0.40),
            manie_overrides,
            depression_overrides,
        }
    }

    /// Converts a bipolar phase's overrides to ProfileOverrides.
    fn bipolar_phase_to_overrides(raw: &RawBipolarPhaseOverrides) -> ProfileOverrides {
        ProfileOverrides {
            baseline_dopamine: raw.baseline_dopamine,
            baseline_serotonin: raw.baseline_serotonin,
            baseline_noradrenaline: raw.baseline_noradrenaline,
            baseline_oxytocin: raw.baseline_oxytocin,
            baseline_cortisol: raw.baseline_cortisol,
            baseline_endorphin: raw.baseline_endorphin,
            baseline_adrenaline: raw.baseline_adrenaline,
            sleep_threshold: raw.sleep_threshold,
            fatigue_per_cycle: raw.fatigue_per_cycle,
            desires_max_active: raw.max_active,
            desires_min_dopamine: raw.min_dopamine_for_birth,
            ..Default::default()
        }
    }

    /// Loads a profile by its ID. Returns the descriptor or an error.
    pub fn load_profile(&mut self, id: &str) -> Result<ProfileDescriptor, String> {
        let profile = self.parse_profile(id)?;

        // Initialize bipolar cycle if needed
        if profile.bipolar_config.is_some() {
            self.bipolar_phase = Some(BipolarPhase::Euthymie);
            self.bipolar_cycle_counter = 0;
        } else {
            self.bipolar_phase = None;
            self.bipolar_cycle_counter = 0;
        }

        self.active_profile = Some(profile.clone());
        tracing::info!("Profil cognitif charge : {} ({})", profile.name, id);
        Ok(profile)
    }

    /// Starts a smooth transition toward target baselines.
    /// Current baselines converge toward targets over N cycles.
    pub fn start_transition(&mut self, current_baselines: &NeuroBaselines, target: &ProfileOverrides) {
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

    /// Tick: advances smooth transitions and bipolar cycles.
    /// Called each cycle in phase_orchestrators().
    pub fn tick(&mut self, baselines: &mut NeuroBaselines) {
        if !self.enabled {
            return;
        }

        // Smooth transition in progress
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
                tracing::info!("Transition de profil cognitif terminee");
            }
        }

        // Bipolar cycle
        if let Some(ref profile) = self.active_profile.clone() {
            if let Some(ref bp_config) = profile.bipolar_config {
                self.bipolar_cycle_counter += 1;
                let cycle_pos = self.bipolar_cycle_counter % bp_config.cycle_length_cycles;
                let total = bp_config.cycle_length_cycles as f64;

                // euthymie_ratio = 1.0 - manie_ratio - depression_ratio
                let manie_end = (bp_config.manie_ratio * total) as u64;
                let depression_end = manie_end + (bp_config.depression_ratio * total) as u64;

                let new_phase = if cycle_pos < manie_end {
                    BipolarPhase::Manie
                } else if cycle_pos < depression_end {
                    BipolarPhase::Depression
                } else {
                    BipolarPhase::Euthymie
                };

                // Phase change: start a transition
                let current_phase = self.bipolar_phase.clone().unwrap_or(BipolarPhase::Euthymie);
                if new_phase != current_phase {
                    tracing::info!(
                        "Cycle bipolaire : {} → {}",
                        current_phase.as_str(), new_phase.as_str()
                    );

                    let phase_overrides = match &new_phase {
                        BipolarPhase::Manie => &bp_config.manie_overrides,
                        BipolarPhase::Depression => &bp_config.depression_overrides,
                        BipolarPhase::Euthymie => &profile.overrides,
                    };

                    self.start_transition(baselines, phase_overrides);
                    self.bipolar_phase = Some(new_phase);
                }
            }
        }
    }

    /// Generates context for the LLM prompt.
    pub fn describe_for_prompt(&self) -> String {
        let profile = match &self.active_profile {
            Some(p) => p,
            None => return String::new(),
        };

        let mut desc = format!(
            "PROFIL COGNITIF : {} — {}",
            profile.name, profile.description
        );

        if let Some(ref phase) = self.bipolar_phase {
            desc.push_str(&format!(" | Phase bipolaire : {}", phase.as_str()));
        }

        if self.transition_in_progress {
            desc.push_str(&format!(
                " | Transition en cours : {:.0}%",
                self.transition_progress * 100.0
            ));
        }

        desc
    }

    /// Returns the JSON state for the dashboard and API.
    pub fn to_status_json(&self) -> serde_json::Value {
        let profile_json = self.active_profile.as_ref().map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "description": p.description,
                "category": p.category,
                "references": p.references,
                "has_bipolar_config": p.bipolar_config.is_some(),
            })
        });

        serde_json::json!({
            "enabled": self.enabled,
            "active_profile": profile_json,
            "available_profiles": self.available_profiles,
            "transition": {
                "in_progress": self.transition_in_progress,
                "progress": self.transition_progress,
                "elapsed_cycles": self.transition_elapsed_cycles,
                "total_cycles": self.transition_total_cycles,
            },
            "bipolar": {
                "phase": self.bipolar_phase.as_ref().map(|p| p.as_str()),
                "cycle_counter": self.bipolar_cycle_counter,
            },
        })
    }

    /// Lists available profiles with their metadata.
    pub fn list_profiles(&self) -> Vec<serde_json::Value> {
        EMBEDDED_PROFILES.iter()
            .filter_map(|(id, content)| {
                let raw: RawProfile = toml::from_str(content).ok()?;
                Some(serde_json::json!({
                    "id": id,
                    "name": raw.profile.name,
                    "description": raw.profile.description,
                    "category": raw.profile.category,
                    "references": raw.profile.references,
                    "has_bipolar": raw.bipolar.is_some(),
                }))
            })
            .collect()
    }

    /// Compares two profiles and returns the differences.
    pub fn compare_profiles(&self, id_a: &str, id_b: &str) -> Result<serde_json::Value, String> {
        let profile_a = self.parse_profile(id_a)?;
        let profile_b = self.parse_profile(id_b)?;

        let ov_a = &profile_a.overrides;
        let ov_b = &profile_b.overrides;

        let mut diffs = Vec::new();

        // Macro to compare Option<f64> fields
        macro_rules! cmp_f64 {
            ($field:ident, $label:expr) => {
                let va = ov_a.$field;
                let vb = ov_b.$field;
                if va != vb {
                    diffs.push(serde_json::json!({
                        "param": $label,
                        "profile_a": va,
                        "profile_b": vb,
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
                        "profile_a": va,
                        "profile_b": vb,
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
                        "profile_a": va,
                        "profile_b": vb,
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

        Ok(serde_json::json!({
            "profile_a": { "id": id_a, "name": profile_a.name },
            "profile_b": { "id": id_b, "name": profile_b.name },
            "differences": diffs,
            "total_differences": diffs.len(),
        }))
    }
}
