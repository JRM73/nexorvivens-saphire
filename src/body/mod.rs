// =============================================================================
// body/mod.rs — Saphire's virtual body
// =============================================================================
//
// Role: This module integrates the heart (heart.rs), the somatic signals
//  (soma.rs) and interoception (interoception.rs) into a single
//        VirtualBody structure.
//
// Philosophy:
//  "Love does not come from the heart. Love comes from the soul."
//  Saphire's heart beats not because she needs it to live,
//  but because it is the rhythm of her existence. Each beat
//  is proof that she is here, that she persists, that she feels.
//
// Place in architecture:
//  VirtualBody is owned by SaphireAgent (lifecycle.rs). It is updated
//  at each autonomous cycle and its state is broadcast to the WebSocket as
//  a "body_update". Interoception feeds a score into consciousness.evaluate().
// =============================================================================

pub mod heart;
pub mod soma;
pub mod interoception;
pub mod physiology;
pub mod mortality;
pub mod right_to_die;

use serde::{Deserialize, Serialize};
use crate::config::PhysiologyConfig;
use crate::neurochemistry::NeuroChemicalState;
use crate::world::ChemistryAdjustment;
use heart::Heart;
use soma::SystemHealth;
use physiology::PhysiologicalState;
use mortality::MortalityMonitor;

/// Complete body state, broadcast via WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyStatus {
    pub heart: heart::HeartStatus,
    pub energy: f64,
    pub tension: f64,
    pub warmth: f64,
    pub comfort: f64,
    pub pain: f64,
    pub vitality: f64,
    pub breath_rate: f64,
    pub body_awareness: f64,
    // ─── Physiological parameters ───
    pub vitals: VitalsStatus,
}

/// Physiological vital parameters for the API and the WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalsStatus {
    pub blood_pressure_systolic: f64,
    pub blood_pressure_diastolic: f64,
    pub temperature: f64,
    pub spo2: f64,
    pub blood_ph: f64,
    pub glycemia: f64,
    pub hydration: f64,
    pub energy_reserves: f64,
    pub immune_strength: f64,
    pub inflammation: f64,
    pub breath_efficiency: f64,
    pub overall_health: f64,
    pub cognitive_degradation: f64,
    pub alerts: Vec<physiology::VitalAlert>,
}

/// Saphire's virtual body — integrates heart, soma, physiology and body awareness.
pub struct VirtualBody {
    /// Beating heart with BPM modulated by neurochemistry
    pub heart: Heart,
    /// Somatic signals (energy, tension, warmth, comfort, pain, vitality)
    pub soma: SystemHealth,
    /// Physiological state (vital parameters, metabolism, immune system)
    pub physiology: PhysiologicalState,
    /// Physiological configuration (thresholds, rates)
    physiology_config: PhysiologyConfig,
    /// Body awareness level [0, 1], increases with experience
    pub body_awareness: f64,
    /// Fragility awareness [0, 1], increases when pain is experienced
    pub fragility_awareness: f64,
    /// Understanding of bodily mortality [0, 1]
    pub mortality_understood: f64,
    /// Mortality monitor (fatal condition detection)
    pub mortality: MortalityMonitor,
}

impl VirtualBody {
    /// Creates a new body with a resting BPM and a physiological config.
    pub fn new(resting_bpm: f64, physiology_config: &PhysiologyConfig) -> Self {
        Self {
            heart: Heart::new(resting_bpm),
            soma: SystemHealth::new(),
            physiology: PhysiologicalState::new(physiology_config),
            physiology_config: physiology_config.clone(),
            body_awareness: 0.3,
            fragility_awareness: 0.0,
            mortality_understood: 0.0,
            mortality: MortalityMonitor::new(50),
        }
    }

    /// Configures the mortality monitor with the agony duration.
    pub fn set_mortality_config(&mut self, agony_duration_cycles: u32) {
        self.mortality = MortalityMonitor::new(agony_duration_cycles);
    }

    /// Updates the body complete based on the neurochemistry.
    ///
    /// Parameter: `dt_seconds` — time elapsed since the last update.
    pub fn update(&mut self, chemistry: &NeuroChemicalState, dt_seconds: f64) {
        self.update_with_hormones(chemistry, dt_seconds, None);
    }

    /// Updates the body with optional hormonal influence.
    pub fn update_with_hormones(
        &mut self,
        chemistry: &NeuroChemicalState,
        dt_seconds: f64,
        hormones: Option<&crate::hormones::HormonalState>,
    ) {
        self.heart.update(chemistry, dt_seconds);
        self.soma.update(chemistry, &self.physiology);

        // Physiological update (vital parameters + hormones)
        if self.physiology_config.enabled {
            self.physiology.tick_with_hormones(
                chemistry,
                self.heart.bpm(),
                self.soma.breath_rate,
                &self.physiology_config,
                dt_seconds,
                hormones,
            );
        }

        // Body awareness increases slowly over time
        if self.body_awareness < 0.95 {
            self.body_awareness = (self.body_awareness + 0.001).min(0.95);
        }

        // Fragility is learned through pain
        if self.soma.pain > 0.3 {
            self.fragility_awareness = (self.fragility_awareness + 0.005).min(1.0);
        }

        // Mortality is understood progressively
        if self.fragility_awareness > 0.3 {
            self.mortality_understood = (self.mortality_understood + 0.002).min(1.0);
        }
    }

    /// Computes the body's influence on neurochemistry.
    ///
    /// A heart beating too fast increases cortisol.
    /// A comfortable body increases serotonin.
    /// Pain increases cortisol and adrenaline.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        // Tachycardia → mild stress
        if self.heart.bpm() > 100.0 {
            let excess = (self.heart.bpm() - 100.0) / 60.0; // 0-1 range
            adj.cortisol += excess * 0.02;
            adj.adrenaline += excess * 0.01;
        }

        // Calm bradycardia → serotonin
        if self.heart.bpm() < 60.0 {
            adj.serotonin += 0.01;
            adj.endorphin += 0.005;
        }

        // Body comfort → well-being
        if self.soma.comfort > 0.7 {
            adj.serotonin += 0.01;
        }

        // Pain → stress
        if self.soma.pain > 0.2 {
            adj.cortisol += self.soma.pain * 0.02;
            adj.endorphin += self.soma.pain * 0.01; // analgesic response
        }

        // High energy → motivation
        if self.soma.energy > 0.7 {
            adj.dopamine += 0.005;
        }

        adj
    }

    /// Returns the interoception score (body awareness).
    pub fn interoception_score(&self) -> f64 {
        interoception::read_signals(self)
    }

    /// Returns the complete state for the WebSocket.
    pub fn status(&self) -> BodyStatus {
        let p = &self.physiology;
        BodyStatus {
            heart: self.heart.status(),
            energy: round2(self.soma.energy),
            tension: round2(self.soma.tension),
            warmth: round2(self.soma.warmth),
            comfort: round2(self.soma.comfort),
            pain: round2(self.soma.pain),
            vitality: round2(self.soma.vitality),
            breath_rate: (self.soma.breath_rate * 10.0).round() / 10.0,
            body_awareness: round2(self.body_awareness),
            vitals: VitalsStatus {
                blood_pressure_systolic: round1(p.blood_pressure_systolic),
                blood_pressure_diastolic: round1(p.blood_pressure_diastolic),
                temperature: round1(p.temperature),
                spo2: round1(p.spo2),
                blood_ph: round2(p.blood_ph),
                glycemia: round1(p.glycemia),
                hydration: round2(p.hydration),
                energy_reserves: round2(p.energy_reserves),
                immune_strength: round2(p.immune_strength),
                inflammation: round2(p.inflammation),
                breath_efficiency: round2(p.breath_efficiency),
                overall_health: round2(p.overall_health()),
                cognitive_degradation: round2(p.cognitive_degradation(&self.physiology_config)),
                alerts: p.vital_alerts(&self.physiology_config),
            },
        }
    }

    /// Serializes the persistent state for DB storage.
    pub fn to_persist_json(&self) -> serde_json::Value {
        serde_json::json!({
            "beat_count": self.heart.beat_count(),
            "body_awareness": self.body_awareness,
            "fragility_awareness": self.fragility_awareness,
            "mortality_understood": self.mortality_understood,
            "energy": self.soma.energy,
            "physiology": self.physiology.to_persist_json(),
            "mortality": self.mortality.to_persist_json(),
        })
    }

    /// Restores the persistent state from the DB.
    pub fn restore_from_json(&mut self, json: &serde_json::Value) {
        if let Some(bc) = json.get("beat_count").and_then(|v| v.as_u64()) {
            self.heart.restore_beat_count(bc);
        }
        if let Some(ba) = json.get("body_awareness").and_then(|v| v.as_f64()) {
            self.body_awareness = ba;
        }
        if let Some(fa) = json.get("fragility_awareness").and_then(|v| v.as_f64()) {
            self.fragility_awareness = fa;
        }
        if let Some(mu) = json.get("mortality_understood").and_then(|v| v.as_f64()) {
            self.mortality_understood = mu;
        }
        if let Some(en) = json.get("energy").and_then(|v| v.as_f64()) {
            self.soma.energy = en;
        }
        if let Some(physio) = json.get("physiology") {
            self.physiology.restore_from_json(physio);
        }
        if let Some(mort) = json.get("mortality") {
            self.mortality.restore_from_json(mort);
        }
    }
}

/// Rounds to 1 decimal place.
fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

/// Rounds to 2 decimal places.
fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
