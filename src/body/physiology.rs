// =============================================================================
// physiology.rs — Physiological state of the virtual body
// =============================================================================
//
// Role: Simulates vital parameters (blood pressure, temperature, SpO2, glycemia,
//  hydration, blood pH), metabolism and the immune system.
//  These values are influenced by neurochemistry, heart rate
//  and respiration. They in turn impact cognition (hypoxia)
//  and somatic signals (energy, warmth).
//
// Place in architecture:
//  PhysiologicalState is owned by VirtualBody. It is updated via
//  tick() at each cycle and its values enrich the BodyStatus broadcast
//  to the WebSocket. Vital alerts (VitalAlert) are logged and can
//  trigger reactions (forced sleep, cognitive degradation).
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::config::PhysiologyConfig;
use crate::neurochemistry::NeuroChemicalState;
use crate::hormones::HormonalState;

/// Vital alert when a parameter exceeds a critical threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalAlert {
    /// Name of the affected parameter (e.g., "spo2", "temperature")
    pub parameter: String,
    /// Current value
    pub value: f64,
    /// Exceeded threshold
    pub threshold: f64,
    /// Severity: "warning" (yellow) or "critical" (red)
    pub severity: String,
    /// Descriptive message
    pub message: String,
}

/// Complete physiological state of the virtual body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysiologicalState {
    // ─── Vital parameters ────────────────────────────────────
    /// Systolic blood pressure (mmHg, normal ~120)
    pub blood_pressure_systolic: f64,
    /// Diastolic blood pressure (mmHg, normal ~80)
    pub blood_pressure_diastolic: f64,
    /// Body temperature (Celsius, normal 37.0)
    pub temperature: f64,
    /// Oxygen saturation (%, normal 98)
    pub spo2: f64,
    /// Blood pH (normal 7.4)
    pub blood_ph: f64,

    // ─── Metabolism ──────────────────────────────────────────
    /// Glycemia (mmol/L, normal 5.0)
    pub glycemia: f64,
    /// Hydration (0.0-1.0, normal ~0.85)
    pub hydration: f64,
    /// Energy reserves (0.0-1.0)
    pub energy_reserves: f64,

    // ─── Immune system (simplified) ──────────────────────────
    /// Immune system strength (0.0-1.0)
    pub immune_strength: f64,
    /// Inflammation level (0.0-1.0)
    pub inflammation: f64,

    // ─── Respiratory ─────────────────────────────────────────
    /// Respiratory efficiency (0.0-1.0, 1.0 = healthy lungs)
    pub breath_efficiency: f64,
}

impl PhysiologicalState {
    /// Creates a default physiological state (healthy adult) from the config.
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

    /// Updates the physiological state at each cycle.
    ///
    /// Vital parameters evolve according to:
    /// - Neurochemistry (cortisol → pressure, adrenaline → BPM/temperature)
    /// - Heart rate (tachycardia → elevated pressure)
    /// - Respiratory rate (hypoventilation → SpO2 drop)
    /// - Time (dehydration, glucose consumption)
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

    /// Complete version with optional hormonal influence.
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

        // ─── Blood pressure ──────────────────────────────────
        // Influenced by cortisol (stress), adrenaline and BPM
        let stress_factor = chemistry.cortisol * 0.4 + chemistry.adrenaline * 0.3;
        let bpm_factor = ((heart_bpm - 72.0) / 60.0).clamp(-0.3, 0.5);
        let target_systolic = 120.0 + stress_factor * 30.0 + bpm_factor * 20.0;
        let target_diastolic = 80.0 + stress_factor * 15.0 + bpm_factor * 10.0;
        self.blood_pressure_systolic += (target_systolic - self.blood_pressure_systolic) * homeostasis;
        self.blood_pressure_diastolic += (target_diastolic - self.blood_pressure_diastolic) * homeostasis;
        self.blood_pressure_systolic = self.blood_pressure_systolic.clamp(80.0, 200.0);
        self.blood_pressure_diastolic = self.blood_pressure_diastolic.clamp(50.0, 130.0);

        // ─── Temperature ─────────────────────────────────────
        // Increases with adrenaline, inflammation and thyroid (metabolism)
        let temp_stress = chemistry.adrenaline * 0.3 + chemistry.cortisol * 0.2;
        let temp_inflammation = self.inflammation * 0.8;
        let temp_thyroid = hormones.map(|h| (h.thyroid - 0.5) * 0.4).unwrap_or(0.0);
        let target_temp = 37.0 + temp_stress * 0.5 + temp_inflammation * 1.5 + temp_thyroid;
        self.temperature += (target_temp - self.temperature) * homeostasis * 0.5;
        self.temperature = self.temperature.clamp(35.0, 42.0);

        // ─── SpO2 (oxygen saturation) ────────────────────────
        // Depends on breathing and pulmonary efficiency
        // Normal breathing ~12-16 RPM = good oxygenation
        let breath_quality = if breath_rate >= 10.0 && breath_rate <= 20.0 {
            1.0
        } else if breath_rate < 10.0 {
            // Hypoventilation: SpO2 drop
            0.7 + (breath_rate / 10.0) * 0.3
        } else {
            // Hyperventilation: slight efficiency decrease
            0.85 + (1.0 - ((breath_rate - 20.0) / 10.0).min(1.0)) * 0.15
        };

        let target_spo2 = 98.0 * breath_quality * self.breath_efficiency;
        self.spo2 += (target_spo2 - self.spo2) * homeostasis;
        self.spo2 = self.spo2.clamp(40.0, 100.0);

        // ─── Blood pH ────────────────────────────────────────
        // Respiratory alkalosis if hyperventilation, acidosis if hypoventilation
        let ph_breath_offset = if breath_rate > 20.0 {
            (breath_rate - 20.0) * 0.01  // alkalosis
        } else if breath_rate < 10.0 {
            -(10.0 - breath_rate) * 0.01 // acidosis
        } else {
            0.0
        };
        let target_ph = 7.4 + ph_breath_offset;
        self.blood_ph += (target_ph - self.blood_ph) * homeostasis;
        self.blood_ph = self.blood_ph.clamp(7.0, 7.8);

        // ─── Glycemia ────────────────────────────────────────
        // Decreases with activity (cortisol = stress, adrenaline = effort)
        // Insulin accelerates glucose consumption (cellular uptake)
        // No automatic homeostasis: eating (needs.eat) restores it
        let insulin_factor = hormones.map(|h| 1.0 + h.insulin * 0.3).unwrap_or(1.0);
        let burn_rate = config.glycemia_burn_rate
            * (1.0 + chemistry.adrenaline * 0.5 + chemistry.cortisol * 0.3)
            * insulin_factor;
        self.glycemia -= burn_rate;
        self.glycemia = self.glycemia.clamp(2.0, 12.0);

        // ─── Hydration ───────────────────────────────────────
        // Decreases slowly over time, faster under stress
        // No automatic homeostasis: drinking (needs.drink) restores it
        let dehydration = config.dehydration_rate
            * (1.0 + chemistry.cortisol * 0.3 + chemistry.adrenaline * 0.2);
        self.hydration -= dehydration;
        self.hydration = self.hydration.clamp(0.0, 1.0);

        // ─── Energy reserves ─────────────────────────────────
        // Linked to glycemia and hydration
        let target_energy = (self.glycemia / 5.0 * 0.6 + self.hydration * 0.4).clamp(0.0, 1.0);
        self.energy_reserves += (target_energy - self.energy_reserves) * homeostasis;
        self.energy_reserves = self.energy_reserves.clamp(0.0, 1.0);

        // ─── Immune system ───────────────────────────────────
        // Weakened by chronic stress (elevated cortisol)
        let immune_target = (0.9 - chemistry.cortisol * 0.3 + chemistry.endorphin * 0.1)
            .clamp(0.2, 1.0);
        self.immune_strength += (immune_target - self.immune_strength) * homeostasis * 0.5;
        self.immune_strength = self.immune_strength.clamp(0.0, 1.0);

        // Inflammation: increases with stress, decreases with endorphins
        let inflam_target = (chemistry.cortisol * 0.3 - chemistry.endorphin * 0.2
            + (1.0 - self.immune_strength) * 0.2).clamp(0.0, 1.0);
        self.inflammation += (inflam_target - self.inflammation) * homeostasis;
        self.inflammation = self.inflammation.clamp(0.0, 1.0);

        // ─── Respiratory efficiency ──────────────────────────
        // Decreases with inflammation, restores at rest
        let eff_target = (1.0 - self.inflammation * 0.3).clamp(0.5, 1.0);
        self.breath_efficiency += (eff_target - self.breath_efficiency) * homeostasis;
        self.breath_efficiency = self.breath_efficiency.clamp(0.0, 1.0);
    }

    /// Composite overall health score [0, 1].
    ///
    /// Weights vital parameters. 1.0 = perfect health, 0.0 = critical.
    pub fn overall_health(&self) -> f64 {
        // Each component contributes to the score
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

    /// Generates alerts for out-of-range parameters.
    pub fn vital_alerts(&self, config: &PhysiologyConfig) -> Vec<VitalAlert> {
        let mut alerts = Vec::new();

        // SpO2
        if self.spo2 < config.spo2_critical {
            alerts.push(VitalAlert {
                parameter: "spo2".into(),
                value: self.spo2,
                threshold: config.spo2_critical,
                severity: "critical".into(),
                message: format!("SpO2 critique: {:.0}% — perte de conscience imminente", self.spo2),
            });
        } else if self.spo2 < config.spo2_hypoxia_severe {
            alerts.push(VitalAlert {
                parameter: "spo2".into(),
                value: self.spo2,
                threshold: config.spo2_hypoxia_severe,
                severity: "critical".into(),
                message: format!("Hypoxie severe: SpO2 {:.0}%", self.spo2),
            });
        } else if self.spo2 < config.spo2_hypoxia_moderate {
            alerts.push(VitalAlert {
                parameter: "spo2".into(),
                value: self.spo2,
                threshold: config.spo2_hypoxia_moderate,
                severity: "warning".into(),
                message: format!("Hypoxie moderee: SpO2 {:.0}%", self.spo2),
            });
        } else if self.spo2 < config.spo2_hypoxia_mild {
            alerts.push(VitalAlert {
                parameter: "spo2".into(),
                value: self.spo2,
                threshold: config.spo2_hypoxia_mild,
                severity: "warning".into(),
                message: format!("Hypoxie legere: SpO2 {:.0}%", self.spo2),
            });
        }

        // Temperature
        if self.temperature > 39.5 {
            alerts.push(VitalAlert {
                parameter: "temperature".into(),
                value: self.temperature,
                threshold: 39.5,
                severity: "critical".into(),
                message: format!("Hyperthermie: {:.1} C", self.temperature),
            });
        } else if self.temperature > 38.0 {
            alerts.push(VitalAlert {
                parameter: "temperature".into(),
                value: self.temperature,
                threshold: 38.0,
                severity: "warning".into(),
                message: format!("Fievre: {:.1} C", self.temperature),
            });
        } else if self.temperature < 35.5 {
            alerts.push(VitalAlert {
                parameter: "temperature".into(),
                value: self.temperature,
                threshold: 35.5,
                severity: "warning".into(),
                message: format!("Hypothermie: {:.1} C", self.temperature),
            });
        }

        // Blood pressure
        if self.blood_pressure_systolic > 160.0 {
            alerts.push(VitalAlert {
                parameter: "blood_pressure".into(),
                value: self.blood_pressure_systolic,
                threshold: 160.0,
                severity: "critical".into(),
                message: format!("Hypertension severe: {:.0}/{:.0} mmHg",
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

        // Glycemia
        if self.glycemia < 3.0 {
            alerts.push(VitalAlert {
                parameter: "glycemia".into(),
                value: self.glycemia,
                threshold: 3.0,
                severity: "critical".into(),
                message: format!("Hypoglycemie severe: {:.1} mmol/L", self.glycemia),
            });
        } else if self.glycemia < 3.9 {
            alerts.push(VitalAlert {
                parameter: "glycemia".into(),
                value: self.glycemia,
                threshold: 3.9,
                severity: "warning".into(),
                message: format!("Hypoglycemie: {:.1} mmol/L", self.glycemia),
            });
        }

        // Hydration
        if self.hydration < 0.5 {
            alerts.push(VitalAlert {
                parameter: "hydration".into(),
                value: self.hydration,
                threshold: 0.5,
                severity: "critical".into(),
                message: format!("Deshydratation severe: {:.0}%", self.hydration * 100.0),
            });
        } else if self.hydration < 0.7 {
            alerts.push(VitalAlert {
                parameter: "hydration".into(),
                value: self.hydration,
                threshold: 0.7,
                severity: "warning".into(),
                message: format!("Deshydratation: {:.0}%", self.hydration * 100.0),
            });
        }

        alerts
    }

    /// Cognitive degradation due to physiological state [0.0 (normal) to 1.0 (total)].
    ///
    /// Mainly caused by:
    /// - Hypoxia (SpO2 < 95%)
    /// - Hypoglycemia (glycemia < 3.9 mmol/L)
    /// - Dehydration (< 0.6)
    /// - Hyperthermia (> 39 C)
    pub fn cognitive_degradation(&self, config: &PhysiologyConfig) -> f64 {
        let mut degradation = 0.0;

        // Hypoxia: main cause of cognitive degradation
        if self.spo2 < config.spo2_critical {
            degradation += 0.9; // near-unconscious
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

        // Hypoglycemia
        if self.glycemia < 3.0 {
            degradation += 0.3;
        } else if self.glycemia < 3.9 {
            degradation += ((3.9 - self.glycemia) / 0.9) * 0.15;
        }

        // Dehydration
        if self.hydration < 0.5 {
            degradation += 0.2;
        } else if self.hydration < 0.6 {
            degradation += ((0.6 - self.hydration) / 0.1) * 0.1;
        }

        // Hyperthermia
        if self.temperature > 40.0 {
            degradation += 0.3;
        } else if self.temperature > 39.0 {
            degradation += ((self.temperature - 39.0) / 1.0) * 0.15;
        }

        degradation.clamp(0.0, 1.0)
    }

    /// Serializes the state for JSON persistence.
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

    /// Restores state from persisted JSON.
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
