// =============================================================================
// fields.rs — Saphire's electromagnetic fields
//
// Role: Simulates the EM fields surrounding the being: universal field (Schumann),
// solar field (cycles, wind, UV), terrestrial field (geomagnetism),
// and the individual biofield (cardiac coherence, brainwaves, aura).
//
// Place in architecture:
//   Cognitive pipeline step 3q: tick + chemistry_influence.
//   Cross-interactions:
//   - Solar UV → vitamin D synthesis (nutrition)
//   - Synaptic density (grey_matter) → biofield coherence
//   - Magnetic storms → sleep quality
//   - Schumann alignment → serotonin, consciousness
// =============================================================================

use serde::{Deserialize, Serialize};

use crate::config::FieldsConfig;
use crate::world::ChemistryAdjustment;

// ─── Universal field ────────────────────────────────────────────────────────
/// Universal electromagnetic field (Schumann resonance, cosmic radiation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalField {
    /// Schumann resonance frequency (Hz, nominal ~7.83)
    pub schumann_resonance: f64,
    /// Background cosmic radiation level (0-1)
    pub cosmic_radiation: f64,
    /// Harmonic alignment (brain-Schumann coherence, 0-1)
    pub harmonic_alignment: f64,
}

impl Default for UniversalField {
    fn default() -> Self {
        Self {
            schumann_resonance: 7.83,
            cosmic_radiation: 0.2,
            harmonic_alignment: 0.5,
        }
    }
}

// ─── Solar field ──────────────────────────────────────────────────────────
/// Solar electromagnetic field (activity, wind, UV).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarField {
    /// Solar activity index (0-1, ~11 year cycle)
    pub activity_index: f64,
    /// Solar wind intensity (0-1)
    pub solar_wind: f64,
    /// UV index (0-1)
    pub uv_index: f64,
}

impl Default for SolarField {
    fn default() -> Self {
        Self {
            activity_index: 0.3,
            solar_wind: 0.2,
            uv_index: 0.4,
        }
    }
}

// ─── Terrestrial field ────────────────────────────────────────────────────────
/// Terrestrial geomagnetic field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrestrialField {
    /// Geomagnetic field intensity (0-1)
    pub geomagnetic_strength: f64,
    /// Magnetic storm intensity (0-1)
    pub storm_intensity: f64,
    /// Telluric currents (0-1)
    pub telluric_currents: f64,
}

impl Default for TerrestrialField {
    fn default() -> Self {
        Self {
            geomagnetic_strength: 0.5,
            storm_intensity: 0.1,
            telluric_currents: 0.2,
        }
    }
}

// ─── Individual biofield ────────────────────────────────────────────────────
/// Individual electromagnetic field (biofield, aura).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualBiofield {
    /// Cardiac EM field (the heart generates the strongest EM field in the body)
    pub cardiac_em: f64,
    /// Brainwave coherence (inter-regional synchronization)
    pub brainwave_coherence: f64,
    /// Biofield integrity (resilience, protection)
    pub biofield_integrity: f64,
    /// Aura luminosity (composite of all sub-fields)
    pub aura_luminosity: f64,
}

impl Default for IndividualBiofield {
    fn default() -> Self {
        Self {
            cardiac_em: 0.5,
            brainwave_coherence: 0.5,
            biofield_integrity: 0.6,
            aura_luminosity: 0.5,
        }
    }
}

// ─── Complete EM fields system ───────────────────────────────────────────
/// Complete electromagnetic fields system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectromagneticFields {
    pub enabled: bool,
    pub universal: UniversalField,
    pub solar: SolarField,
    pub terrestrial: TerrestrialField,
    pub biofield: IndividualBiofield,
    /// Solar cycle phase (rad, 0..2pi)
    pub solar_cycle_phase: f64,
    /// Geomagnetic phase (rad, 0..2pi)
    pub geomagnetic_phase: f64,
}

impl ElectromagneticFields {
    /// Creates a new EM system from the config.
    pub fn new(config: &FieldsConfig) -> Self {
        Self {
            enabled: config.enabled,
            universal: UniversalField::default(),
            solar: SolarField::default(),
            terrestrial: TerrestrialField::default(),
            biofield: IndividualBiofield::default(),
            solar_cycle_phase: 0.0,
            geomagnetic_phase: 0.0,
        }
    }

    /// EM fields tick: cosmic cycles, biofield.
    ///
    /// Input parameters:
    /// - `hrv_coherence`: HRV coherence from the heart (body)
    /// - `brain_sync`: brain synchronization (brain_regions workspace)
    /// - `vitality`: overall vitality (body vitality)
    /// - `consciousness_level`: consciousness level (consciousness)
    /// - `synaptic_density`: synaptic density (grey_matter → biofield)
    pub fn tick(
        &mut self,
        config: &FieldsConfig,
        hrv_coherence: f64,
        brain_sync: f64,
        vitality: f64,
        consciousness_level: f64,
        synaptic_density: f64,
    ) {
        if !self.enabled { return; }

        // ─── Solar cycle ──────────────────────────────────────────────────
        // Slow sinusoidal cycle (~11 years simulated by slow advance)
        self.solar_cycle_phase += config.solar_cycle_speed;
        if self.solar_cycle_phase > std::f64::consts::TAU {
            self.solar_cycle_phase -= std::f64::consts::TAU;
        }
        self.solar.activity_index = 0.3 + 0.3 * self.solar_cycle_phase.sin();
        self.solar.solar_wind = (self.solar.activity_index * 0.8 + 0.1).clamp(0.0, 1.0);
        // UV correlated to solar activity + noise
        self.solar.uv_index = (0.3 + self.solar.activity_index * 0.4).clamp(0.0, 1.0);

        // ─── Magnetic storms ─────────────────────────────────────────────
        // Derived from solar wind (when the wind is strong)
        self.geomagnetic_phase += config.solar_cycle_speed * 3.0;
        if self.geomagnetic_phase > std::f64::consts::TAU {
            self.geomagnetic_phase -= std::f64::consts::TAU;
        }
        let storm_base = self.solar.solar_wind * 0.6;
        let storm_variation = 0.2 * (self.geomagnetic_phase * 7.0).sin().abs();
        self.terrestrial.storm_intensity = (storm_base + storm_variation).clamp(0.0, 1.0);

        // Geomagnetic field varies slowly
        self.terrestrial.geomagnetic_strength = 0.5 + 0.1 * (self.geomagnetic_phase * 0.3).sin();
        self.terrestrial.telluric_currents = self.terrestrial.storm_intensity * 0.5 + 0.1;

        // ─── Schumann resonance ──────────────────────────────────────────
        // Varies around 7.83 Hz +/- variance
        let schumann_noise = (self.solar_cycle_phase * 13.0).sin() * config.schumann_variance;
        self.universal.schumann_resonance = 7.83 + schumann_noise;

        // Cosmic radiation inversely proportional to geomagnetic field
        self.universal.cosmic_radiation = (0.4 - self.terrestrial.geomagnetic_strength * 0.3).clamp(0.05, 0.6);

        // ─── Individual biofield ────────────────────────────────────────────
        // Cardiac EM: derived from HRV coherence + vitality
        self.biofield.cardiac_em = (hrv_coherence * 0.5 + vitality * 0.3 + 0.2).clamp(0.0, 1.0);

        // Brainwave coherence: brain synchronization + synaptic density
        self.biofield.brainwave_coherence = (brain_sync * 0.5 + synaptic_density * 0.3 + 0.2).clamp(0.0, 1.0);

        // Biofield integrity: smoothing towards the average of the components
        let target_integrity = (self.biofield.cardiac_em + self.biofield.brainwave_coherence + vitality) / 3.0;
        self.biofield.biofield_integrity = self.biofield.biofield_integrity * 0.95 + target_integrity * 0.05;

        // Aura = composite of all factors
        self.biofield.aura_luminosity =
            self.biofield.cardiac_em * 0.25
            + self.biofield.brainwave_coherence * 0.25
            + self.biofield.biofield_integrity * 0.25
            + consciousness_level * 0.25;

        // ─── Harmonic alignment ──────────────────────────────────────────
        // Resonance between Schumann and brainwave coherence
        // The closer Schumann frequency is to 7.83 and the higher brain coherence,
        // the stronger the alignment
        let schumann_alignment = 1.0 - ((self.universal.schumann_resonance - 7.83).abs() / config.schumann_variance).min(1.0);
        self.universal.harmonic_alignment = schumann_alignment * self.biofield.brainwave_coherence;
    }

    /// Influence of EM fields on neurochemistry.
    pub fn chemistry_influence(&self, config: &FieldsConfig) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();
        if !self.enabled { return adj; }

        // Magnetic storms → cortisol +, serotonin -
        if self.terrestrial.storm_intensity > config.storm_threshold {
            let storm_excess = self.terrestrial.storm_intensity - config.storm_threshold;
            adj.cortisol += storm_excess * config.storm_anxiety_factor;
            adj.serotonin -= storm_excess * config.storm_anxiety_factor * 0.5;
        }

        // High Schumann coherence → serotonin +, cortisol -
        if self.universal.harmonic_alignment > 0.6 {
            let alignment_bonus = self.universal.harmonic_alignment - 0.6;
            adj.serotonin += alignment_bonus * 0.03;
            adj.cortisol -= alignment_bonus * 0.02;
        }

        // Strong biofield → endorphin + (well-being)
        if self.biofield.aura_luminosity > 0.7 {
            adj.endorphin += (self.biofield.aura_luminosity - 0.7) * 0.02;
        }

        // High solar activity → noradrenaline + (arousal, vigilance)
        if self.solar.activity_index > 0.5 {
            adj.noradrenaline += (self.solar.activity_index - 0.5) * 0.02;
        }

        adj
    }

    /// Returns the UV index (useful for vitamin D synthesis in nutrition).
    pub fn uv_index(&self) -> f64 {
        if self.enabled { self.solar.uv_index } else { 0.4 }
    }

    /// Returns the storm intensity (useful for sleep quality).
    pub fn storm_intensity(&self) -> f64 {
        if self.enabled { self.terrestrial.storm_intensity } else { 0.0 }
    }

    /// Serializes the state to JSON for persistence.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "universal": {
                "schumann_resonance": self.universal.schumann_resonance,
                "cosmic_radiation": self.universal.cosmic_radiation,
                "harmonic_alignment": self.universal.harmonic_alignment,
            },
            "solar": {
                "activity_index": self.solar.activity_index,
                "solar_wind": self.solar.solar_wind,
                "uv_index": self.solar.uv_index,
            },
            "terrestrial": {
                "geomagnetic_strength": self.terrestrial.geomagnetic_strength,
                "storm_intensity": self.terrestrial.storm_intensity,
                "telluric_currents": self.terrestrial.telluric_currents,
            },
            "biofield": {
                "cardiac_em": self.biofield.cardiac_em,
                "brainwave_coherence": self.biofield.brainwave_coherence,
                "biofield_integrity": self.biofield.biofield_integrity,
                "aura_luminosity": self.biofield.aura_luminosity,
            },
            "solar_cycle_phase": self.solar_cycle_phase,
            "geomagnetic_phase": self.geomagnetic_phase,
        })
    }

    /// Restores state from persisted JSON.
    pub fn restore_from_json(&mut self, v: &serde_json::Value) {
        if let Some(u) = v.get("universal") {
            self.universal.schumann_resonance = u["schumann_resonance"].as_f64().unwrap_or(7.83);
            self.universal.cosmic_radiation = u["cosmic_radiation"].as_f64().unwrap_or(0.2);
            self.universal.harmonic_alignment = u["harmonic_alignment"].as_f64().unwrap_or(0.5);
        }
        if let Some(s) = v.get("solar") {
            self.solar.activity_index = s["activity_index"].as_f64().unwrap_or(0.3);
            self.solar.solar_wind = s["solar_wind"].as_f64().unwrap_or(0.2);
            self.solar.uv_index = s["uv_index"].as_f64().unwrap_or(0.4);
        }
        if let Some(t) = v.get("terrestrial") {
            self.terrestrial.geomagnetic_strength = t["geomagnetic_strength"].as_f64().unwrap_or(0.5);
            self.terrestrial.storm_intensity = t["storm_intensity"].as_f64().unwrap_or(0.1);
            self.terrestrial.telluric_currents = t["telluric_currents"].as_f64().unwrap_or(0.2);
        }
        if let Some(b) = v.get("biofield") {
            self.biofield.cardiac_em = b["cardiac_em"].as_f64().unwrap_or(0.5);
            self.biofield.brainwave_coherence = b["brainwave_coherence"].as_f64().unwrap_or(0.5);
            self.biofield.biofield_integrity = b["biofield_integrity"].as_f64().unwrap_or(0.6);
            self.biofield.aura_luminosity = b["aura_luminosity"].as_f64().unwrap_or(0.5);
        }
        self.solar_cycle_phase = v["solar_cycle_phase"].as_f64().unwrap_or(0.0);
        self.geomagnetic_phase = v["geomagnetic_phase"].as_f64().unwrap_or(0.0);
    }
}
