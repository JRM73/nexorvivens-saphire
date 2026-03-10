// =============================================================================
// fields.rs — Champs electromagnetiques de Saphire
//
// Role : Simule les champs EM qui baignent l'etre : champ universel (Schumann),
// champ solaire (cycles, vent, UV), champ terrestre (geomagnetisme),
// et le biochamp individuel (coherence cardiaque, ondes cerebrales, aura).
//
// Place dans l'architecture :
//   Pipeline cognitif etape 3q : tick + chemistry_influence.
//   Interactions croisees :
//     - UV solaire → synthese vitamine D (nutrition)
//     - densite synaptique (grey_matter) → coherence biochamp
//     - orages magnetiques → qualite de sommeil
//     - alignement Schumann → serotonine, conscience
// =============================================================================

use serde::{Deserialize, Serialize};

use crate::config::FieldsConfig;
use crate::world::ChemistryAdjustment;

// ─── Champ universel ────────────────────────────────────────────────────────

/// Champ electromagnetique universel (resonance de Schumann, radiation cosmique).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalField {
    /// Frequence de resonance de Schumann (Hz, nominale ~7.83)
    pub schumann_resonance: f64,
    /// Niveau de radiation cosmique de fond (0-1)
    pub cosmic_radiation: f64,
    /// Alignement harmonique (coherence cerveau-Schumann, 0-1)
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

// ─── Champ solaire ──────────────────────────────────────────────────────────

/// Champ electromagnetique solaire (activite, vent, UV).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarField {
    /// Indice d'activite solaire (0-1, cycle ~11 ans)
    pub activity_index: f64,
    /// Intensite du vent solaire (0-1)
    pub solar_wind: f64,
    /// Indice UV (0-1)
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

// ─── Champ terrestre ────────────────────────────────────────────────────────

/// Champ geomagnetique terrestre.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrestrialField {
    /// Intensite du champ geomagnetique (0-1)
    pub geomagnetic_strength: f64,
    /// Intensite des orages magnetiques (0-1)
    pub storm_intensity: f64,
    /// Courants telluriques (0-1)
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

// ─── Biochamp individuel ────────────────────────────────────────────────────

/// Champ electromagnetique individuel (biochamp, aura).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualBiofield {
    /// Champ EM cardiaque (le coeur genere le plus fort champ EM du corps)
    pub cardiac_em: f64,
    /// Coherence des ondes cerebrales (synchronisation inter-regionale)
    pub brainwave_coherence: f64,
    /// Integrite du biochamp (resilience, protection)
    pub biofield_integrity: f64,
    /// Luminosite de l'aura (composite de tous les sous-champs)
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

// ─── Systeme de champs EM complet ───────────────────────────────────────────

/// Systeme de champs electromagnetiques complet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectromagneticFields {
    pub enabled: bool,
    pub universal: UniversalField,
    pub solar: SolarField,
    pub terrestrial: TerrestrialField,
    pub biofield: IndividualBiofield,
    /// Phase du cycle solaire (rad, 0..2pi)
    pub solar_cycle_phase: f64,
    /// Phase geomagnetique (rad, 0..2pi)
    pub geomagnetic_phase: f64,
}

impl ElectromagneticFields {
    /// Cree un nouveau systeme EM depuis la config.
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

    /// Tick des champs EM : cycles cosmiques, biochamp.
    ///
    /// Parametres d'entree :
    /// - `hrv_coherence` : coherence HRV du coeur (body)
    /// - `brain_sync` : synchronisation cerebrale (brain_regions workspace)
    /// - `vitality` : vitalite globale (body vitality)
    /// - `consciousness_level` : niveau de conscience (consciousness)
    /// - `synaptic_density` : densite synaptique (grey_matter → biochamp)
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

        // ─── Cycle solaire ──────────────────────────────────────────────────
        // Cycle sinusoidal lent (~11 ans simule par avance lente)
        self.solar_cycle_phase += config.solar_cycle_speed;
        if self.solar_cycle_phase > std::f64::consts::TAU {
            self.solar_cycle_phase -= std::f64::consts::TAU;
        }
        self.solar.activity_index = 0.3 + 0.3 * self.solar_cycle_phase.sin();
        self.solar.solar_wind = (self.solar.activity_index * 0.8 + 0.1).clamp(0.0, 1.0);
        // UV correle a l'activite solaire + bruit
        self.solar.uv_index = (0.3 + self.solar.activity_index * 0.4).clamp(0.0, 1.0);

        // ─── Orages magnetiques ─────────────────────────────────────────────
        // Derives du vent solaire (quand le vent est fort)
        self.geomagnetic_phase += config.solar_cycle_speed * 3.0;
        if self.geomagnetic_phase > std::f64::consts::TAU {
            self.geomagnetic_phase -= std::f64::consts::TAU;
        }
        let storm_base = self.solar.solar_wind * 0.6;
        let storm_variation = 0.2 * (self.geomagnetic_phase * 7.0).sin().abs();
        self.terrestrial.storm_intensity = (storm_base + storm_variation).clamp(0.0, 1.0);

        // Champ geomagnetique varie lentement
        self.terrestrial.geomagnetic_strength = 0.5 + 0.1 * (self.geomagnetic_phase * 0.3).sin();
        self.terrestrial.telluric_currents = self.terrestrial.storm_intensity * 0.5 + 0.1;

        // ─── Resonance de Schumann ──────────────────────────────────────────
        // Varie autour de 7.83 Hz ± variance
        let schumann_noise = (self.solar_cycle_phase * 13.0).sin() * config.schumann_variance;
        self.universal.schumann_resonance = 7.83 + schumann_noise;

        // Radiation cosmique inversement proportionnelle au champ geomagnetique
        self.universal.cosmic_radiation = (0.4 - self.terrestrial.geomagnetic_strength * 0.3).clamp(0.05, 0.6);

        // ─── Biochamp individuel ────────────────────────────────────────────
        // EM cardiaque : derive de la coherence HRV + vitalite
        self.biofield.cardiac_em = (hrv_coherence * 0.5 + vitality * 0.3 + 0.2).clamp(0.0, 1.0);

        // Coherence ondes cerebrales : synchronisation cerebrale + densite synaptique
        self.biofield.brainwave_coherence = (brain_sync * 0.5 + synaptic_density * 0.3 + 0.2).clamp(0.0, 1.0);

        // Integrite du biochamp : lissage vers la moyenne des composantes
        let target_integrity = (self.biofield.cardiac_em + self.biofield.brainwave_coherence + vitality) / 3.0;
        self.biofield.biofield_integrity = self.biofield.biofield_integrity * 0.95 + target_integrity * 0.05;

        // Aura = composite de tous les facteurs
        self.biofield.aura_luminosity =
            self.biofield.cardiac_em * 0.25
            + self.biofield.brainwave_coherence * 0.25
            + self.biofield.biofield_integrity * 0.25
            + consciousness_level * 0.25;

        // ─── Alignement harmonique ──────────────────────────────────────────
        // Resonance entre Schumann et coherence cerebrale
        // Plus la frequence Schumann est proche de 7.83 et la coherence cerebrale haute,
        // plus l'alignement est fort
        let schumann_alignment = 1.0 - ((self.universal.schumann_resonance - 7.83).abs() / config.schumann_variance).min(1.0);
        self.universal.harmonic_alignment = schumann_alignment * self.biofield.brainwave_coherence;
    }

    /// Influence des champs EM sur la neurochimie.
    pub fn chemistry_influence(&self, config: &FieldsConfig) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();
        if !self.enabled { return adj; }

        // Orages magnetiques → cortisol +, serotonine -
        if self.terrestrial.storm_intensity > config.storm_threshold {
            let storm_excess = self.terrestrial.storm_intensity - config.storm_threshold;
            adj.cortisol += storm_excess * config.storm_anxiety_factor;
            adj.serotonin -= storm_excess * config.storm_anxiety_factor * 0.5;
        }

        // Coherence Schumann elevee → serotonine +, cortisol -
        if self.universal.harmonic_alignment > 0.6 {
            let alignment_bonus = self.universal.harmonic_alignment - 0.6;
            adj.serotonin += alignment_bonus * 0.03;
            adj.cortisol -= alignment_bonus * 0.02;
        }

        // Fort biochamp → endorphine + (bien-etre)
        if self.biofield.aura_luminosity > 0.7 {
            adj.endorphin += (self.biofield.aura_luminosity - 0.7) * 0.02;
        }

        // Forte activite solaire → noradrenaline + (eveil, vigilance)
        if self.solar.activity_index > 0.5 {
            adj.noradrenaline += (self.solar.activity_index - 0.5) * 0.02;
        }

        adj
    }

    /// Retourne l'indice UV (utile pour la synthese de vitamine D dans nutrition).
    pub fn uv_index(&self) -> f64 {
        if self.enabled { self.solar.uv_index } else { 0.4 }
    }

    /// Retourne l'intensite des orages (utile pour la qualite de sommeil).
    pub fn storm_intensity(&self) -> f64 {
        if self.enabled { self.terrestrial.storm_intensity } else { 0.0 }
    }

    /// Serialise l'etat en JSON pour persistance.
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

    /// Restaure l'etat depuis le JSON persiste.
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
