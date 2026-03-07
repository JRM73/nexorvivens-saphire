// =============================================================================
// physiology.rs — Physiological State of the Virtual Body
// =============================================================================
//
// Purpose: Simulates detailed physiological vital parameters: blood pressure,
//          core temperature, peripheral oxygen saturation (SpO2), blood pH,
//          blood glucose (glycemia), hydration, energy reserves, immune function,
//          systemic inflammation, and respiratory efficiency. These values are
//          influenced by neurochemistry, heart rate, respiration rate, and
//          (optionally) hormonal state. They in turn impact cognition (via
//          hypoxia-induced degradation) and somatic signals (energy, warmth).
//
// Architectural placement:
//   `PhysiologicalState` is owned by `VirtualBody`. It is updated via `tick()`
//   or `tick_with_hormones()` every cycle. Its values enrich the `BodyStatus`
//   broadcast to the WebSocket. Vital alerts (`VitalAlert`) are logged and
//   can trigger reactive behaviors (forced sleep, cognitive degradation).
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::config::PhysiologyConfig;
use crate::neurochemistry::NeuroChemicalState;
use crate::hormones::HormonalState;

/// Alert generated when a vital parameter exceeds a clinically significant threshold.
///
/// Alerts are included in the `BodyStatus` WebSocket broadcast and can trigger
/// system-level responses such as forced sleep or mortality checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalAlert {
    /// Name of the affected parameter (e.g., "spo2", "temperature", "blood_pressure").
    pub parameter: String,
    /// Current measured value of the parameter.
    pub value: f64,
    /// Threshold that was exceeded to generate this alert.
    pub threshold: f64,
    /// Severity level: "warning" (concerning but not immediately dangerous) or "critical" (life-threatening).
    pub severity: String,
    /// Human-readable descriptive message for logging and display.
    pub message: String,
}

/// Complete physiological state of the virtual body.
///
/// Models a simplified but physiologically plausible set of vital parameters
/// organized into four subsystems: cardiovascular, metabolic, immune, and respiratory.
/// Each parameter evolves over time via exponential convergence toward a target
/// value determined by neurochemistry, cardiac output, respiratory function, and
/// optional hormonal modulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysiologicalState {
    // --- Cardiovascular vital parameters ---
    /// Systolic blood pressure in mmHg (normal: ~120 mmHg).
    /// Rises with cortisol, adrenaline, and elevated heart rate (sympathetic activation).
    pub blood_pressure_systolic: f64,
    /// Diastolic blood pressure in mmHg (normal: ~80 mmHg).
    /// Follows the same stress/cardiac influences as systolic, with smaller magnitude.
    pub blood_pressure_diastolic: f64,
    /// Core body temperature in degrees Celsius (normal: 37.0 C).
    /// Rises with adrenaline (metabolic heat), inflammation (febrile response),
    /// and thyroid hormone (metabolic rate modulation).
    pub temperature: f64,
    /// Peripheral oxygen saturation as a percentage (normal: 95-100%).
    /// Depends on respiratory rate and pulmonary efficiency. Drops with
    /// hypoventilation or reduced breath efficiency (e.g., from inflammation).
    pub spo2: f64,
    /// Arterial blood pH (normal: 7.35-7.45).
    /// Shifts toward alkalosis with hyperventilation (CO2 washout) and toward
    /// acidosis with hypoventilation (CO2 retention).
    pub blood_ph: f64,

    // --- Metabolic parameters ---
    /// Blood glucose concentration in mmol/L (normal fasting: ~5.0 mmol/L).
    /// Decreases over time due to cellular energy consumption; burn rate is
    /// accelerated by adrenaline, cortisol, and insulin. Restored by eating.
    pub glycemia: f64,
    /// Hydration level in [0.0, 1.0] (normal: ~0.85).
    /// Decreases over time due to insensible water loss; accelerated by stress.
    /// Restored by drinking. No automatic homeostatic recovery.
    pub hydration: f64,
    /// Metabolic energy reserves in [0.0, 1.0].
    /// Derived from glycemia (60% weight) and hydration (40% weight).
    /// Represents available cellular energy for physiological processes.
    pub energy_reserves: f64,

    // --- Immune system (simplified model) ---
    /// Immune system strength in [0.0, 1.0] (1.0 = fully competent).
    /// Weakened by chronic stress (elevated cortisol); supported by endorphins.
    /// When critically low combined with high inflammation, triggers lethal virus risk.
    pub immune_strength: f64,
    /// Systemic inflammation level in [0.0, 1.0] (0.0 = no inflammation).
    /// Increases with cortisol-driven stress; decreases with endorphin release
    /// and strong immune function. High inflammation raises body temperature
    /// (febrile response) and reduces respiratory efficiency.
    pub inflammation: f64,

    // --- Respiratory system ---
    /// Pulmonary efficiency in [0.0, 1.0] (1.0 = healthy, fully functional lungs).
    /// Decreases with systemic inflammation (inflammatory lung damage/edema).
    /// Directly affects SpO2: reduced efficiency means less oxygen transfer.
    pub breath_efficiency: f64,
}

impl PhysiologicalState {
    /// Creates a default physiological state representing a healthy adult,
    /// initialized from the provided configuration.
    ///
    /// Default values: BP 120/80 mmHg, pH 7.4, energy reserves 0.8,
    /// immune strength 0.85, inflammation 0.05, breath efficiency 1.0.
    /// Temperature, SpO2, glycemia, and hydration are taken from config.
    pub fn new(config: &PhysiologyConfig) -> Self {
        Self {
            blood_pressure_systolic: 120.0,
            blood_pressure_diastolic: 80.0,
            temperature: config.initial_temperature,
            spo2: config.initial_spo2,
            blood_ph: 7.4,
            glycemia: config.initial_glycemia,
            hydration: config.initial_hydration,
            energy_reserves: 0.8,
            immune_strength: 0.85,
            inflammation: 0.05,
            breath_efficiency: 1.0,
        }
    }

    /// Updates the physiological state for the current cycle (without hormonal influence).
    ///
    /// Convenience wrapper that delegates to [`tick_with_hormones`](Self::tick_with_hormones)
    /// with `None` for the hormonal state.
    ///
    /// # Parameters
    /// - `chemistry`: current neurochemical concentrations.
    /// - `heart_bpm`: current heart rate in beats per minute.
    /// - `breath_rate`: current respiratory rate in breaths per minute.
    /// - `config`: physiology configuration (thresholds, rates).
    /// - `_dt`: elapsed time since last update (reserved for future use).
    pub fn tick(
        &mut self,
        chemistry: &NeuroChemicalState,
        heart_bpm: f64,
        breath_rate: f64,
        config: &PhysiologyConfig,
        _dt: f64,
    ) {
        self.tick_with_hormones(chemistry, heart_bpm, breath_rate, config, _dt, None);
    }

    /// Full physiological update with optional hormonal modulation.
    ///
    /// Each vital parameter converges toward a target value using exponential smoothing
    /// at the configured `homeostasis_rate`. This models the body's homeostatic
    /// regulatory mechanisms — parameters do not jump instantly but drift toward
    /// their equilibrium determined by current conditions.
    ///
    /// # Parameters
    /// - `chemistry`: current neurochemical concentrations.
    /// - `heart_bpm`: current heart rate in BPM.
    /// - `breath_rate`: current respiratory rate in breaths per minute (RPM).
    /// - `config`: physiology configuration (thresholds, rates, enable flag).
    /// - `_dt`: elapsed time since last update (reserved for future use).
    /// - `hormones`: optional hormonal state for thyroid and insulin modulation.
    pub fn tick_with_hormones(
        &mut self,
        chemistry: &NeuroChemicalState,
        heart_bpm: f64,
        breath_rate: f64,
        config: &PhysiologyConfig,
        _dt: f64,
        hormones: Option<&HormonalState>,
    ) {
        if !config.enabled {
            return;
        }

        let homeostasis = config.homeostasis_rate;

        // --- Blood pressure (arterial) ---
        // Influenced by cortisol (stress-induced vasoconstriction), adrenaline
        // (sympathetic cardiovascular activation), and heart rate (cardiac output).
        // Normal resting: ~120/80 mmHg. Clamped to [80-200]/[50-130] mmHg.
        let stress_factor = chemistry.cortisol * 0.4 + chemistry.adrenaline * 0.3;
        let bpm_factor = ((heart_bpm - 72.0) / 60.0).clamp(-0.3, 0.5);
        let target_systolic = 120.0 + stress_factor * 30.0 + bpm_factor * 20.0;
        let target_diastolic = 80.0 + stress_factor * 15.0 + bpm_factor * 10.0;
        self.blood_pressure_systolic += (target_systolic - self.blood_pressure_systolic) * homeostasis;
        self.blood_pressure_diastolic += (target_diastolic - self.blood_pressure_diastolic) * homeostasis;
        self.blood_pressure_systolic = self.blood_pressure_systolic.clamp(80.0, 200.0);
        self.blood_pressure_diastolic = self.blood_pressure_diastolic.clamp(50.0, 130.0);

        // --- Core body temperature ---
        // Rises with adrenaline (thermogenic sympathetic activation), inflammation
        // (febrile/pyrogenic response), and thyroid hormone (basal metabolic rate).
        // Normal: 37.0 C. Clamped to [35.0, 42.0] C (hypothermia to severe hyperthermia).
        let temp_stress = chemistry.adrenaline * 0.3 + chemistry.cortisol * 0.2;
        let temp_inflammation = self.inflammation * 0.8;
        let temp_thyroid = hormones.map(|h| (h.thyroid - 0.5) * 0.4).unwrap_or(0.0);
        let target_temp = 37.0 + temp_stress * 0.5 + temp_inflammation * 1.5 + temp_thyroid;
        self.temperature += (target_temp - self.temperature) * homeostasis * 0.5;
        self.temperature = self.temperature.clamp(35.0, 42.0);

        // --- SpO2 (peripheral oxygen saturation) ---
        // Depends on respiratory rate and pulmonary efficiency.
        // Normal respiratory rate: 12-16 RPM provides optimal oxygenation.
        // Hypoventilation (< 10 RPM): reduced alveolar gas exchange lowers SpO2.
        // Hyperventilation (> 20 RPM): slightly reduced efficiency due to
        // insufficient alveolar dwell time despite increased ventilation rate.
        // Clamped to [40%, 100%].
        let breath_quality = if breath_rate >= 10.0 && breath_rate <= 20.0 {
            1.0
        } else if breath_rate < 10.0 {
            // Hypoventilation: SpO2 drops proportionally
            0.7 + (breath_rate / 10.0) * 0.3
        } else {
            // Hyperventilation: slight efficiency loss from excessive respiratory rate
            0.85 + (1.0 - ((breath_rate - 20.0) / 10.0).min(1.0)) * 0.15
        };

        let target_spo2 = 98.0 * breath_quality * self.breath_efficiency;
        self.spo2 += (target_spo2 - self.spo2) * homeostasis;
        self.spo2 = self.spo2.clamp(40.0, 100.0);

        // --- Arterial blood pH ---
        // Respiratory alkalosis occurs with hyperventilation (excessive CO2 exhalation
        // shifts the bicarbonate buffer toward alkaline). Respiratory acidosis occurs
        // with hypoventilation (CO2 retention shifts pH downward).
        // Normal: 7.4. Clamped to [7.0, 7.8].
        let ph_breath_offset = if breath_rate > 20.0 {
            (breath_rate - 20.0) * 0.01  // alkalosis from CO2 washout
        } else if breath_rate < 10.0 {
            -(10.0 - breath_rate) * 0.01 // acidosis from CO2 retention
        } else {
            0.0
        };
        let target_ph = 7.4 + ph_breath_offset;
        self.blood_ph += (target_ph - self.blood_ph) * homeostasis;
        self.blood_ph = self.blood_ph.clamp(7.0, 7.8);

        // --- Blood glucose (glycemia) ---
        // Decreases over time due to cellular energy consumption (basal metabolic rate).
        // Burn rate is accelerated by adrenaline (mobilization for fight-or-flight),
        // cortisol (gluconeogenesis demand), and insulin (cellular glucose uptake).
        // No automatic homeostatic recovery: glucose is restored only by eating
        // (needs.eat action). Clamped to [2.0, 12.0] mmol/L.
        let insulin_factor = hormones.map(|h| 1.0 + h.insulin * 0.3).unwrap_or(1.0);
        let burn_rate = config.glycemia_burn_rate
            * (1.0 + chemistry.adrenaline * 0.5 + chemistry.cortisol * 0.3)
            * insulin_factor;
        self.glycemia -= burn_rate;
        self.glycemia = self.glycemia.clamp(2.0, 12.0);

        // --- Hydration ---
        // Decreases slowly over time due to insensible water loss (respiration,
        // perspiration). Dehydration rate is accelerated by stress hormones
        // (cortisol, adrenaline). No automatic homeostatic recovery: hydration
        // is restored only by drinking (needs.drink action). Clamped to [0.0, 1.0].
        let dehydration = config.dehydration_rate
            * (1.0 + chemistry.cortisol * 0.3 + chemistry.adrenaline * 0.2);
        self.hydration -= dehydration;
        self.hydration = self.hydration.clamp(0.0, 1.0);

        // --- Metabolic energy reserves ---
        // Derived from glycemia (60% weight, normalized to 5.0 mmol/L baseline)
        // and hydration (40% weight). Represents the available cellular energy
        // pool for physiological processes. Converges homeostically. Clamped to [0.0, 1.0].
        let target_energy = (self.glycemia / 5.0 * 0.6 + self.hydration * 0.4).clamp(0.0, 1.0);
        self.energy_reserves += (target_energy - self.energy_reserves) * homeostasis;
        self.energy_reserves = self.energy_reserves.clamp(0.0, 1.0);

        // --- Immune system ---
        // Weakened by chronic stress (sustained high cortisol suppresses immune function,
        // a well-documented psychoneuroimmunological effect). Supported by endorphins
        // (which have immunostimulatory properties). Converges at half the homeostasis
        // rate (immune changes are slower than cardiovascular changes). Clamped to [0.0, 1.0].
        let immune_target = (0.9 - chemistry.cortisol * 0.3 + chemistry.endorphin * 0.1)
            .clamp(0.2, 1.0);
        self.immune_strength += (immune_target - self.immune_strength) * homeostasis * 0.5;
        self.immune_strength = self.immune_strength.clamp(0.0, 1.0);

        // Systemic inflammation: increases with cortisol-driven stress, decreases with
        // endorphin release (anti-inflammatory effect) and strong immune function
        // (effective pathogen clearance reduces inflammatory signaling). Clamped to [0.0, 1.0].
        let inflam_target = (chemistry.cortisol * 0.3 - chemistry.endorphin * 0.2
            + (1.0 - self.immune_strength) * 0.2).clamp(0.0, 1.0);
        self.inflammation += (inflam_target - self.inflammation) * homeostasis;
        self.inflammation = self.inflammation.clamp(0.0, 1.0);

        // --- Respiratory (pulmonary) efficiency ---
        // Decreases with systemic inflammation (inflammatory damage to lung tissue,
        // edema reducing gas exchange surface area). Recovers at rest when inflammation
        // subsides. Clamped to [0.0, 1.0].
        let eff_target = (1.0 - self.inflammation * 0.3).clamp(0.5, 1.0);
        self.breath_efficiency += (eff_target - self.breath_efficiency) * homeostasis;
        self.breath_efficiency = self.breath_efficiency.clamp(0.0, 1.0);
    }

    /// Computes a composite overall health score in [0.0, 1.0].
    ///
    /// Weighted combination of normalized vital parameters:
    /// - SpO2 (25%): normalized from [60%, 100%] to [0.0, 1.0].
    /// - Temperature (15%): deviation from 37.0 C, normalized over 3 C range.
    /// - Blood pressure (15%): systolic and diastolic deviation from 120/80 mmHg.
    /// - Glycemia (10%): deviation from 5.0 mmol/L, normalized over 3 mmol/L range.
    /// - Hydration (10%): direct [0.0, 1.0] value.
    /// - Energy reserves (10%): direct [0.0, 1.0] value.
    /// - Immune strength (10%): direct [0.0, 1.0] value.
    /// - Inflammation penalty (5%): reduces score proportionally to inflammation level.
    ///
    /// Returns 1.0 for perfect health and approaches 0.0 for critical multi-organ failure.
    pub fn overall_health(&self) -> f64 {
        // Each component contributes proportionally to the composite score
        let spo2_score = ((self.spo2 - 60.0) / 40.0).clamp(0.0, 1.0);
        let temp_score = 1.0 - ((self.temperature - 37.0).abs() / 3.0).clamp(0.0, 1.0);
        let bp_score = {
            let sys_dev = ((self.blood_pressure_systolic - 120.0).abs() / 40.0).clamp(0.0, 1.0);
            let dia_dev = ((self.blood_pressure_diastolic - 80.0).abs() / 25.0).clamp(0.0, 1.0);
            1.0 - (sys_dev + dia_dev) / 2.0
        };
        let glycemia_score = 1.0 - ((self.glycemia - 5.0).abs() / 3.0).clamp(0.0, 1.0);
        let hydration_score = self.hydration;
        let immune_score = self.immune_strength;
        let inflammation_penalty = self.inflammation * 0.3;

        let score = spo2_score * 0.25
            + temp_score * 0.15
            + bp_score * 0.15
            + glycemia_score * 0.10
            + hydration_score * 0.10
            + self.energy_reserves * 0.10
            + immune_score * 0.10
            + (1.0 - inflammation_penalty) * 0.05;

        score.clamp(0.0, 1.0)
    }

    /// Generates vital alerts for parameters that exceed clinically significant thresholds.
    ///
    /// Checks SpO2 (four severity levels from mild hypoxia to critical), temperature
    /// (hypothermia, fever, hyperthermia), blood pressure (hypertension stages),
    /// glycemia (hypoglycemia stages), and hydration (dehydration stages).
    ///
    /// Returns a vector of [`VitalAlert`] instances, empty if all parameters are normal.
    pub fn vital_alerts(&self, config: &PhysiologyConfig) -> Vec<VitalAlert> {
        let mut alerts = Vec::new();

        // SpO2 alerts (four tiers of hypoxia severity)
        if self.spo2 < config.spo2_critical {
            alerts.push(VitalAlert {
                parameter: "spo2".into(),
                value: self.spo2,
                threshold: config.spo2_critical,
                severity: "critical".into(),
                message: format!("Critical SpO2: {:.0}% — imminent loss of consciousness", self.spo2),
            });
        } else if self.spo2 < config.spo2_hypoxia_severe {
            alerts.push(VitalAlert {
                parameter: "spo2".into(),
                value: self.spo2,
                threshold: config.spo2_hypoxia_severe,
                severity: "critical".into(),
                message: format!("Severe hypoxia: SpO2 {:.0}%", self.spo2),
            });
        } else if self.spo2 < config.spo2_hypoxia_moderate {
            alerts.push(VitalAlert {
                parameter: "spo2".into(),
                value: self.spo2,
                threshold: config.spo2_hypoxia_moderate,
                severity: "warning".into(),
                message: format!("Moderate hypoxia: SpO2 {:.0}%", self.spo2),
            });
        } else if self.spo2 < config.spo2_hypoxia_mild {
            alerts.push(VitalAlert {
                parameter: "spo2".into(),
                value: self.spo2,
                threshold: config.spo2_hypoxia_mild,
                severity: "warning".into(),
                message: format!("Mild hypoxia: SpO2 {:.0}%", self.spo2),
            });
        }

        // Temperature alerts (hypothermia, fever, hyperthermia)
        if self.temperature > 39.5 {
            alerts.push(VitalAlert {
                parameter: "temperature".into(),
                value: self.temperature,
                threshold: 39.5,
                severity: "critical".into(),
                message: format!("Hyperthermia: {:.1} C", self.temperature),
            });
        } else if self.temperature > 38.0 {
            alerts.push(VitalAlert {
                parameter: "temperature".into(),
                value: self.temperature,
                threshold: 38.0,
                severity: "warning".into(),
                message: format!("Fever: {:.1} C", self.temperature),
            });
        } else if self.temperature < 35.5 {
            alerts.push(VitalAlert {
                parameter: "temperature".into(),
                value: self.temperature,
                threshold: 35.5,
                severity: "warning".into(),
                message: format!("Hypothermia: {:.1} C", self.temperature),
            });
        }

        // Blood pressure alerts (hypertension stages)
        if self.blood_pressure_systolic > 160.0 {
            alerts.push(VitalAlert {
                parameter: "blood_pressure".into(),
                value: self.blood_pressure_systolic,
                threshold: 160.0,
                severity: "critical".into(),
                message: format!("Severe hypertension: {:.0}/{:.0} mmHg",
                    self.blood_pressure_systolic, self.blood_pressure_diastolic),
            });
        } else if self.blood_pressure_systolic > 140.0 {
            alerts.push(VitalAlert {
                parameter: "blood_pressure".into(),
                value: self.blood_pressure_systolic,
                threshold: 140.0,
                severity: "warning".into(),
                message: format!("Hypertension: {:.0}/{:.0} mmHg",
                    self.blood_pressure_systolic, self.blood_pressure_diastolic),
            });
        }

        // Glycemia alerts (hypoglycemia stages)
        if self.glycemia < 3.0 {
            alerts.push(VitalAlert {
                parameter: "glycemia".into(),
                value: self.glycemia,
                threshold: 3.0,
                severity: "critical".into(),
                message: format!("Severe hypoglycemia: {:.1} mmol/L", self.glycemia),
            });
        } else if self.glycemia < 3.9 {
            alerts.push(VitalAlert {
                parameter: "glycemia".into(),
                value: self.glycemia,
                threshold: 3.9,
                severity: "warning".into(),
                message: format!("Hypoglycemia: {:.1} mmol/L", self.glycemia),
            });
        }

        // Hydration alerts (dehydration stages)
        if self.hydration < 0.5 {
            alerts.push(VitalAlert {
                parameter: "hydration".into(),
                value: self.hydration,
                threshold: 0.5,
                severity: "critical".into(),
                message: format!("Severe dehydration: {:.0}%", self.hydration * 100.0),
            });
        } else if self.hydration < 0.7 {
            alerts.push(VitalAlert {
                parameter: "hydration".into(),
                value: self.hydration,
                threshold: 0.7,
                severity: "warning".into(),
                message: format!("Dehydration: {:.0}%", self.hydration * 100.0),
            });
        }

        alerts
    }

    /// Computes the cognitive degradation factor due to physiological distress.
    ///
    /// Returns a value in [0.0, 1.0] where 0.0 means no cognitive impairment
    /// and 1.0 means total cognitive incapacitation.
    ///
    /// Contributing factors (additive, clamped to 1.0):
    /// - **Hypoxia** (primary cause): SpO2 below mild threshold causes progressive
    ///   degradation up to 0.9 at critical levels (near-unconsciousness).
    /// - **Hypoglycemia**: glucose below 3.9 mmol/L impairs brain function
    ///   (the brain is almost exclusively glucose-dependent); up to +0.3 contribution.
    /// - **Dehydration**: hydration below 0.6 impairs concentration; up to +0.2 contribution.
    /// - **Hyperthermia**: temperature above 39.0 C causes confusion and delirium;
    ///   up to +0.3 contribution.
    pub fn cognitive_degradation(&self, config: &PhysiologyConfig) -> f64 {
        let mut degradation = 0.0;

        // Hypoxia: primary cause of cognitive degradation (the brain consumes ~20% of
        // total oxygen despite being only ~2% of body mass). Four severity tiers.
        if self.spo2 < config.spo2_critical {
            degradation += 0.9; // near-unconsciousness
        } else if self.spo2 < config.spo2_hypoxia_severe {
            let ratio = (config.spo2_hypoxia_severe - self.spo2)
                / (config.spo2_hypoxia_severe - config.spo2_critical);
            degradation += 0.5 + ratio * 0.4;
        } else if self.spo2 < config.spo2_hypoxia_moderate {
            let ratio = (config.spo2_hypoxia_moderate - self.spo2)
                / (config.spo2_hypoxia_moderate - config.spo2_hypoxia_severe);
            degradation += 0.2 + ratio * 0.3;
        } else if self.spo2 < config.spo2_hypoxia_mild {
            let ratio = (config.spo2_hypoxia_mild - self.spo2)
                / (config.spo2_hypoxia_mild - config.spo2_hypoxia_moderate);
            degradation += ratio * 0.2;
        }

        // Hypoglycemia: the brain relies almost exclusively on glucose for energy;
        // blood glucose below 3.9 mmol/L causes progressive neuroglycopenic symptoms.
        if self.glycemia < 3.0 {
            degradation += 0.3;
        } else if self.glycemia < 3.9 {
            degradation += ((3.9 - self.glycemia) / 0.9) * 0.15;
        }

        // Dehydration: even mild dehydration impairs attention and working memory.
        if self.hydration < 0.5 {
            degradation += 0.2;
        } else if self.hydration < 0.6 {
            degradation += ((0.6 - self.hydration) / 0.1) * 0.1;
        }

        // Hyperthermia: elevated core temperature causes confusion, delirium,
        // and eventually heat stroke with severe cognitive dysfunction.
        if self.temperature > 40.0 {
            degradation += 0.3;
        } else if self.temperature > 39.0 {
            degradation += ((self.temperature - 39.0) / 1.0) * 0.15;
        }

        degradation.clamp(0.0, 1.0)
    }

    /// Serializes the physiological state as JSON for database persistence.
    ///
    /// All vital parameters are saved to allow full state restoration across restarts.
    pub fn to_persist_json(&self) -> serde_json::Value {
        serde_json::json!({
            "blood_pressure_systolic": self.blood_pressure_systolic,
            "blood_pressure_diastolic": self.blood_pressure_diastolic,
            "temperature": self.temperature,
            "spo2": self.spo2,
            "blood_ph": self.blood_ph,
            "glycemia": self.glycemia,
            "hydration": self.hydration,
            "energy_reserves": self.energy_reserves,
            "immune_strength": self.immune_strength,
            "inflammation": self.inflammation,
            "breath_efficiency": self.breath_efficiency,
        })
    }

    /// Restores the physiological state from a previously persisted JSON value.
    ///
    /// Each field is independently optional — missing fields retain their current values.
    pub fn restore_from_json(&mut self, json: &serde_json::Value) {
        if let Some(v) = json.get("blood_pressure_systolic").and_then(|v| v.as_f64()) {
            self.blood_pressure_systolic = v;
        }
        if let Some(v) = json.get("blood_pressure_diastolic").and_then(|v| v.as_f64()) {
            self.blood_pressure_diastolic = v;
        }
        if let Some(v) = json.get("temperature").and_then(|v| v.as_f64()) {
            self.temperature = v;
        }
        if let Some(v) = json.get("spo2").and_then(|v| v.as_f64()) {
            self.spo2 = v;
        }
        if let Some(v) = json.get("blood_ph").and_then(|v| v.as_f64()) {
            self.blood_ph = v;
        }
        if let Some(v) = json.get("glycemia").and_then(|v| v.as_f64()) {
            self.glycemia = v;
        }
        if let Some(v) = json.get("hydration").and_then(|v| v.as_f64()) {
            self.hydration = v;
        }
        if let Some(v) = json.get("energy_reserves").and_then(|v| v.as_f64()) {
            self.energy_reserves = v;
        }
        if let Some(v) = json.get("immune_strength").and_then(|v| v.as_f64()) {
            self.immune_strength = v;
        }
        if let Some(v) = json.get("inflammation").and_then(|v| v.as_f64()) {
            self.inflammation = v;
        }
        if let Some(v) = json.get("breath_efficiency").and_then(|v| v.as_f64()) {
            self.breath_efficiency = v;
        }
    }
}
