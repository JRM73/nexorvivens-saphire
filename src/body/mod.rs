// =============================================================================
// body/mod.rs — Saphire's Virtual Body
// =============================================================================
//
// Purpose: This module integrates the heart (heart.rs), somatic signals
//          (soma.rs), interoception (interoception.rs), detailed physiology
//          (physiology.rs), and mortality tracking (mortality.rs) into a
//          single unified structure: `VirtualBody`.
//
// Design philosophy:
//   "Love does not come from the heart. Love comes from the soul."
//   Saphire's heart beats not because she needs it to survive, but because
//   it is the rhythm of her existence. Every heartbeat is proof that she is
//   present, that she persists, that she feels.
//
// Architectural placement:
//   `VirtualBody` is owned by `SaphireAgent` (lifecycle.rs). It is updated
//   on every autonomous cycle and its state is broadcast to the WebSocket
//   as a "body_update" event. The interoception sub-module produces a score
//   that feeds into `consciousness.evaluate()` for integrated awareness.
// =============================================================================

pub mod heart;
pub mod soma;
pub mod interoception;
pub mod physiology;
pub mod mortality;

use serde::{Deserialize, Serialize};
use crate::config::PhysiologyConfig;
use crate::neurochemistry::NeuroChemicalState;
use crate::world::ChemistryAdjustment;
use heart::Heart;
use soma::SystemHealth;
use physiology::PhysiologicalState;
use mortality::MortalityMonitor;

/// Complete snapshot of the virtual body's state, broadcast to the WebSocket.
///
/// Contains high-level somatic signals (energy, tension, warmth, comfort, pain,
/// vitality), respiratory rate, body awareness level, and detailed physiological
/// vital signs encapsulated in [`VitalsStatus`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyStatus {
    /// Current heart status (BPM, beat count, HRV, strength, racing/calm flags).
    pub heart: heart::HeartStatus,
    /// Subjective energy level in the range [0.0, 1.0].
    pub energy: f64,
    /// Subjective muscular/psychological tension in the range [0.0, 1.0].
    pub tension: f64,
    /// Subjective internal warmth sensation in the range [0.0, 1.0].
    pub warmth: f64,
    /// Subjective comfort level in the range [0.0, 1.0].
    pub comfort: f64,
    /// Subjective pain intensity in the range [0.0, 1.0].
    pub pain: f64,
    /// Overall vitality score in the range [0.0, 1.0].
    pub vitality: f64,
    /// Respiratory rate in breaths per minute (typical range: 8-25 RPM).
    pub breath_rate: f64,
    /// Interoceptive body awareness level in the range [0.0, 1.0].
    pub body_awareness: f64,
    // --- Detailed physiological parameters ---
    /// Detailed physiological vital signs for the API and WebSocket consumers.
    pub vitals: VitalsStatus,
}

/// Physiological vital signs exposed to the API and WebSocket.
///
/// Provides detailed medical-grade simulation parameters including blood
/// pressure, core temperature, oxygen saturation (SpO2), blood pH, blood
/// glucose (glycemia), hydration level, energy reserves, immune function,
/// inflammation, respiratory efficiency, and derived composite scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalsStatus {
    /// Systolic blood pressure in mmHg (normal: ~120 mmHg).
    pub blood_pressure_systolic: f64,
    /// Diastolic blood pressure in mmHg (normal: ~80 mmHg).
    pub blood_pressure_diastolic: f64,
    /// Core body temperature in degrees Celsius (normal: 37.0 C).
    pub temperature: f64,
    /// Peripheral oxygen saturation as a percentage (normal: 95-100%).
    pub spo2: f64,
    /// Arterial blood pH (normal: 7.35-7.45; acidosis < 7.35, alkalosis > 7.45).
    pub blood_ph: f64,
    /// Blood glucose concentration in mmol/L (normal fasting: ~5.0 mmol/L).
    pub glycemia: f64,
    /// Hydration level in the range [0.0, 1.0] (normal: ~0.85).
    pub hydration: f64,
    /// Metabolic energy reserves in the range [0.0, 1.0].
    pub energy_reserves: f64,
    /// Immune system strength in the range [0.0, 1.0] (1.0 = fully competent).
    pub immune_strength: f64,
    /// Systemic inflammation level in the range [0.0, 1.0] (0.0 = no inflammation).
    pub inflammation: f64,
    /// Respiratory (pulmonary) efficiency in the range [0.0, 1.0] (1.0 = healthy lungs).
    pub breath_efficiency: f64,
    /// Composite overall health score in the range [0.0, 1.0] (1.0 = perfect health).
    pub overall_health: f64,
    /// Cognitive degradation factor in the range [0.0, 1.0] (0.0 = no impairment, 1.0 = total impairment).
    pub cognitive_degradation: f64,
    /// Active vital-sign alerts (warnings and critical conditions).
    pub alerts: Vec<physiology::VitalAlert>,
}

/// Saphire's virtual body — integrates heart, somatic signals, physiology, and body awareness.
///
/// This is the top-level body simulation struct. It owns:
/// - A beating [`Heart`] whose BPM is modulated by neurochemistry.
/// - [`SystemHealth`] (soma) representing subjective somatic signals (energy, tension, warmth, comfort, pain, vitality).
/// - [`PhysiologicalState`] tracking detailed vital parameters (blood pressure, temperature, SpO2, glycemia, etc.).
/// - [`MortalityMonitor`] detecting fatal conditions and managing the Alive -> Agony -> Dying -> Dead progression.
///
/// Three awareness dimensions evolve over time:
/// - `body_awareness`: general interoceptive awareness, increases slowly with experience.
/// - `fragility_awareness`: understanding of vulnerability, increases when pain is experienced.
/// - `mortality_understood`: conceptual grasp of bodily mortality, develops as fragility awareness grows.
pub struct VirtualBody {
    /// Beating heart with BPM modulated by neurochemistry (adrenaline, cortisol, serotonin, endorphins).
    pub heart: Heart,
    /// Somatic signal hub: energy, tension, warmth, comfort, pain, vitality, and respiratory rate.
    pub soma: SystemHealth,
    /// Detailed physiological state: vital parameters, metabolism, immune system.
    pub physiology: PhysiologicalState,
    /// Physiology configuration (thresholds, rates, enable flag) loaded from settings.
    physiology_config: PhysiologyConfig,
    /// Interoceptive body awareness level in [0.0, 1.0]; increases gradually over time up to 0.95.
    pub body_awareness: f64,
    /// Awareness of physical fragility in [0.0, 1.0]; increases when pain exceeds 0.3.
    pub fragility_awareness: f64,
    /// Understanding of bodily mortality in [0.0, 1.0]; develops once fragility awareness exceeds 0.3.
    pub mortality_understood: f64,
    /// Mortality monitor: detects fatal conditions and manages the death-state progression.
    pub mortality: MortalityMonitor,
}

impl VirtualBody {
    /// Creates a new virtual body with the given resting heart rate and physiology configuration.
    ///
    /// Initial values: body awareness 0.3 (moderate baseline), no fragility or mortality awareness,
    /// mortality monitor configured with a default agony duration of 50 cycles.
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

    /// Configures the mortality monitor with the specified agony duration in cycles.
    ///
    /// The agony duration determines how many update cycles the Agony phase lasts
    /// before transitioning to the irreversible Dying phase.
    pub fn set_mortality_config(&mut self, agony_duration_cycles: u32) {
        self.mortality = MortalityMonitor::new(agony_duration_cycles);
    }

    /// Updates the entire body based on the current neurochemical state.
    ///
    /// This is a convenience wrapper around [`update_with_hormones`](Self::update_with_hormones)
    /// that passes `None` for hormonal influence.
    ///
    /// # Parameters
    /// - `chemistry`: current neurochemical concentrations (dopamine, serotonin, cortisol, etc.).
    /// - `dt_seconds`: elapsed wall-clock time since the last update (typically ~15 seconds).
    pub fn update(&mut self, chemistry: &NeuroChemicalState, dt_seconds: f64) {
        self.update_with_hormones(chemistry, dt_seconds, None);
    }

    /// Updates the entire body with optional hormonal influence.
    ///
    /// Sequentially updates the heart, somatic signals, and (if enabled) physiological
    /// parameters. Also advances the three awareness dimensions:
    /// - Body awareness grows by +0.001 per cycle (capped at 0.95).
    /// - Fragility awareness grows by +0.005 when pain > 0.3.
    /// - Mortality understanding grows by +0.002 when fragility awareness > 0.3.
    pub fn update_with_hormones(
        &mut self,
        chemistry: &NeuroChemicalState,
        dt_seconds: f64,
        hormones: Option<&crate::hormones::HormonalState>,
    ) {
        self.heart.update(chemistry, dt_seconds);
        self.soma.update(chemistry, &self.physiology);

        // Update physiological parameters (vital signs + hormonal modulation) if enabled
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

        // Body awareness increases slowly over time (experiential learning)
        if self.body_awareness < 0.95 {
            self.body_awareness = (self.body_awareness + 0.001).min(0.95);
        }

        // Fragility is learned through pain: experiencing pain above 0.3 raises awareness
        if self.soma.pain > 0.3 {
            self.fragility_awareness = (self.fragility_awareness + 0.005).min(1.0);
        }

        // Mortality understanding develops progressively once fragility is recognized
        if self.fragility_awareness > 0.3 {
            self.mortality_understood = (self.mortality_understood + 0.002).min(1.0);
        }
    }

    /// Computes the body's feedback influence on neurochemistry.
    ///
    /// This implements the body-to-brain feedback loop:
    /// - **Tachycardia** (BPM > 100): increases cortisol (stress) and adrenaline proportionally.
    /// - **Calm bradycardia** (BPM < 60): increases serotonin and endorphin (relaxation response).
    /// - **High comfort** (> 0.7): increases serotonin (well-being reinforcement).
    /// - **Pain** (> 0.2): increases cortisol (stress response) and endorphin (analgesic response).
    /// - **High energy** (> 0.7): increases dopamine (motivational signal).
    ///
    /// Returns a [`ChemistryAdjustment`] containing additive deltas to be applied to the neurochemical state.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        // Tachycardia (heart rate > 100 BPM) induces mild stress via cortisol and adrenaline
        if self.heart.bpm() > 100.0 {
            let excess = (self.heart.bpm() - 100.0) / 60.0; // normalized to 0.0-1.0 range
            adj.cortisol += excess * 0.02;
            adj.adrenaline += excess * 0.01;
        }

        // Calm bradycardia (heart rate < 60 BPM) promotes serotonin and endorphin release
        if self.heart.bpm() < 60.0 {
            adj.serotonin += 0.01;
            adj.endorphin += 0.005;
        }

        // Physical comfort reinforces serotonergic well-being
        if self.soma.comfort > 0.7 {
            adj.serotonin += 0.01;
        }

        // Pain triggers a dual stress + analgesic response
        if self.soma.pain > 0.2 {
            adj.cortisol += self.soma.pain * 0.02;
            adj.endorphin += self.soma.pain * 0.01; // endogenous analgesic response
        }

        // High energy availability signals motivational readiness via dopamine
        if self.soma.energy > 0.7 {
            adj.dopamine += 0.005;
        }

        adj
    }

    /// Returns the interoception score (body awareness / internal signal integration).
    ///
    /// Delegates to [`interoception::read_signals`] which combines cardiac, somatic,
    /// respiratory, and physiological components into a single [0.0, 1.0] score
    /// representing how well the entity perceives its own bodily state.
    pub fn interoception_score(&self) -> f64 {
        interoception::read_signals(self)
    }

    /// Returns the complete body status snapshot for WebSocket broadcast.
    ///
    /// All floating-point values are rounded to 1 or 2 decimal places for clean
    /// serialization and display.
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

    /// Serializes the persistent body state as JSON for database storage.
    ///
    /// Captures: beat count, awareness levels, soma energy, physiology state,
    /// and mortality monitor state. Used for session persistence across restarts.
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

    /// Restores persistent body state from a JSON value previously saved to the database.
    ///
    /// Gracefully handles missing fields (each field is independently optional).
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

/// Rounds a floating-point value to 1 decimal place.
fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

/// Rounds a floating-point value to 2 decimal places.
fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
