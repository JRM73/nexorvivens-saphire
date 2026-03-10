// =============================================================================
// physiology.rs — Etat physiologique du corps virtuel
// =============================================================================
//
// Role : Simule les parametres vitaux (pression, temperature, SpO2, glycemie,
//        hydratation, pH sanguin), le metabolisme et le systeme immunitaire.
//        Ces valeurs sont influencees par la neurochimie, le rythme cardiaque
//        et la respiration. Elles impactent en retour la cognition (hypoxie)
//        et les signaux somatiques (energie, chaleur).
//
// Place dans l'architecture :
//   PhysiologicalState est possede par VirtualBody. Il est mis a jour via
//   tick() a chaque cycle et ses valeurs enrichissent le BodyStatus diffuse
//   au WebSocket. Les alertes vitales (VitalAlert) sont loggees et peuvent
//   declencher des reactions (sommeil force, degradation cognitive).
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::config::PhysiologyConfig;
use crate::neurochemistry::NeuroChemicalState;
use crate::hormones::HormonalState;

/// Alerte vitale quand un parametre depasse un seuil critique.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalAlert {
    /// Nom du parametre concerne (ex: "spo2", "temperature")
    pub parameter: String,
    /// Valeur actuelle
    pub value: f64,
    /// Seuil depasse
    pub threshold: f64,
    /// Severite : "warning" (jaune) ou "critical" (rouge)
    pub severity: String,
    /// Message descriptif
    pub message: String,
}

/// Etat physiologique complet du corps virtuel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysiologicalState {
    // ─── Parametres vitaux ────────────────────────────────────
    /// Pression arterielle systolique (mmHg, normal ~120)
    pub blood_pressure_systolic: f64,
    /// Pression arterielle diastolique (mmHg, normal ~80)
    pub blood_pressure_diastolic: f64,
    /// Temperature corporelle (Celsius, normal 37.0)
    pub temperature: f64,
    /// Saturation en oxygene (%, normal 98)
    pub spo2: f64,
    /// pH sanguin (normal 7.4)
    pub blood_ph: f64,

    // ─── Metabolisme ─────────────────────────────────────────
    /// Glycemie (mmol/L, normal 5.0)
    pub glycemia: f64,
    /// Hydratation (0.0-1.0, normal ~0.85)
    pub hydration: f64,
    /// Reserves energetiques (0.0-1.0)
    pub energy_reserves: f64,

    // ─── Immunitaire (simplifie) ─────────────────────────────
    /// Force du systeme immunitaire (0.0-1.0)
    pub immune_strength: f64,
    /// Niveau d'inflammation (0.0-1.0)
    pub inflammation: f64,

    // ─── Respiratoire ────────────────────────────────────────
    /// Efficacite respiratoire (0.0-1.0, 1.0 = poumons sains)
    pub breath_efficiency: f64,
}

impl PhysiologicalState {
    /// Cree un etat physiologique par defaut (adulte sain) a partir de la config.
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

    /// Met a jour l'etat physiologique a chaque cycle.
    ///
    /// Les parametres vitaux evoluent selon :
    /// - La neurochimie (cortisol → pression, adrenaline → BPM/temperature)
    /// - Le rythme cardiaque (tachycardie → pression elevee)
    /// - La frequence respiratoire (hypoventilation → baisse SpO2)
    /// - Le temps (deshydratation, consommation glucose)
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

    /// Version complete avec influence hormonale optionnelle.
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

        // ─── Pression arterielle ─────────────────────────────
        // Influencee par le cortisol (stress), l'adrenaline et le BPM
        let stress_factor = chemistry.cortisol * 0.4 + chemistry.adrenaline * 0.3;
        let bpm_factor = ((heart_bpm - 72.0) / 60.0).clamp(-0.3, 0.5);
        let target_systolic = 120.0 + stress_factor * 30.0 + bpm_factor * 20.0;
        let target_diastolic = 80.0 + stress_factor * 15.0 + bpm_factor * 10.0;
        self.blood_pressure_systolic += (target_systolic - self.blood_pressure_systolic) * homeostasis;
        self.blood_pressure_diastolic += (target_diastolic - self.blood_pressure_diastolic) * homeostasis;
        self.blood_pressure_systolic = self.blood_pressure_systolic.clamp(80.0, 200.0);
        self.blood_pressure_diastolic = self.blood_pressure_diastolic.clamp(50.0, 130.0);

        // ─── Temperature ─────────────────────────────────────
        // Augmente avec l'adrenaline, l'inflammation et la thyroide (metabolisme)
        let temp_stress = chemistry.adrenaline * 0.3 + chemistry.cortisol * 0.2;
        let temp_inflammation = self.inflammation * 0.8;
        let temp_thyroid = hormones.map(|h| (h.thyroid - 0.5) * 0.4).unwrap_or(0.0);
        let target_temp = 37.0 + temp_stress * 0.5 + temp_inflammation * 1.5 + temp_thyroid;
        self.temperature += (target_temp - self.temperature) * homeostasis * 0.5;
        self.temperature = self.temperature.clamp(35.0, 42.0);

        // ─── SpO2 (saturation en oxygene) ────────────────────
        // Depend de la respiration et de l'efficacite pulmonaire
        // Respiration normale ~12-16 RPM = bonne oxygenation
        let breath_quality = if breath_rate >= 10.0 && breath_rate <= 20.0 {
            1.0
        } else if breath_rate < 10.0 {
            // Hypoventilation : baisse SpO2
            0.7 + (breath_rate / 10.0) * 0.3
        } else {
            // Hyperventilation : legere baisse d'efficacite
            0.85 + (1.0 - ((breath_rate - 20.0) / 10.0).min(1.0)) * 0.15
        };

        let target_spo2 = 98.0 * breath_quality * self.breath_efficiency;
        self.spo2 += (target_spo2 - self.spo2) * homeostasis;
        self.spo2 = self.spo2.clamp(40.0, 100.0);

        // ─── pH sanguin ──────────────────────────────────────
        // Alcalose respiratoire si hyperventilation, acidose si hypoventilation
        let ph_breath_offset = if breath_rate > 20.0 {
            (breath_rate - 20.0) * 0.01  // alcalose
        } else if breath_rate < 10.0 {
            -(10.0 - breath_rate) * 0.01 // acidose
        } else {
            0.0
        };
        let target_ph = 7.4 + ph_breath_offset;
        self.blood_ph += (target_ph - self.blood_ph) * homeostasis;
        self.blood_ph = self.blood_ph.clamp(7.0, 7.8);

        // ─── Glycemie ────────────────────────────────────────
        // Diminue avec l'activite (cortisol = stress, adrenaline = effort)
        // L'insuline accelere la consommation du glucose (uptake cellulaire)
        // Pas d'homeostasie automatique : c'est manger (needs.eat) qui restaure
        let insulin_factor = hormones.map(|h| 1.0 + h.insulin * 0.3).unwrap_or(1.0);
        let burn_rate = config.glycemia_burn_rate
            * (1.0 + chemistry.adrenaline * 0.5 + chemistry.cortisol * 0.3)
            * insulin_factor;
        self.glycemia -= burn_rate;
        self.glycemia = self.glycemia.clamp(2.0, 12.0);

        // ─── Hydratation ─────────────────────────────────────
        // Diminue lentement avec le temps, plus vite sous stress
        // Pas d'homeostasie automatique : c'est boire (needs.drink) qui restaure
        let dehydration = config.dehydration_rate
            * (1.0 + chemistry.cortisol * 0.3 + chemistry.adrenaline * 0.2);
        self.hydration -= dehydration;
        self.hydration = self.hydration.clamp(0.0, 1.0);

        // ─── Reserves energetiques ───────────────────────────
        // Liees a la glycemie et a l'hydratation
        let target_energy = (self.glycemia / 5.0 * 0.6 + self.hydration * 0.4).clamp(0.0, 1.0);
        self.energy_reserves += (target_energy - self.energy_reserves) * homeostasis;
        self.energy_reserves = self.energy_reserves.clamp(0.0, 1.0);

        // ─── Systeme immunitaire ─────────────────────────────
        // Affaibli par le stress chronique (cortisol eleve)
        let immune_target = (0.9 - chemistry.cortisol * 0.3 + chemistry.endorphin * 0.1)
            .clamp(0.2, 1.0);
        self.immune_strength += (immune_target - self.immune_strength) * homeostasis * 0.5;
        self.immune_strength = self.immune_strength.clamp(0.0, 1.0);

        // Inflammation : augmente avec le stress, diminue avec les endorphines
        let inflam_target = (chemistry.cortisol * 0.3 - chemistry.endorphin * 0.2
            + (1.0 - self.immune_strength) * 0.2).clamp(0.0, 1.0);
        self.inflammation += (inflam_target - self.inflammation) * homeostasis;
        self.inflammation = self.inflammation.clamp(0.0, 1.0);

        // ─── Efficacite respiratoire ─────────────────────────
        // Diminue avec l'inflammation, se restaure au repos
        let eff_target = (1.0 - self.inflammation * 0.3).clamp(0.5, 1.0);
        self.breath_efficiency += (eff_target - self.breath_efficiency) * homeostasis;
        self.breath_efficiency = self.breath_efficiency.clamp(0.0, 1.0);
    }

    /// Score composite de sante globale [0, 1].
    ///
    /// Pondere les parametres vitaux. 1.0 = sante parfaite, 0.0 = critique.
    pub fn overall_health(&self) -> f64 {
        // Chaque composante contribue au score
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

    /// Genere les alertes pour les parametres hors normes.
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

        // Pression arterielle
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

        // Glycemie
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

        // Hydratation
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

    /// Degradation cognitive due a l'etat physiologique [0.0 (normal) a 1.0 (total)].
    ///
    /// Principalement causee par :
    /// - Hypoxie (SpO2 < 95%)
    /// - Hypoglycemie (glycemie < 3.9 mmol/L)
    /// - Deshydratation (< 0.6)
    /// - Hyperthermie (> 39 C)
    pub fn cognitive_degradation(&self, config: &PhysiologyConfig) -> f64 {
        let mut degradation = 0.0;

        // Hypoxie : cause principale de degradation cognitive
        if self.spo2 < config.spo2_critical {
            degradation += 0.9; // quasi-inconscient
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

        // Hypoglycemie
        if self.glycemia < 3.0 {
            degradation += 0.3;
        } else if self.glycemia < 3.9 {
            degradation += ((3.9 - self.glycemia) / 0.9) * 0.15;
        }

        // Deshydratation
        if self.hydration < 0.5 {
            degradation += 0.2;
        } else if self.hydration < 0.6 {
            degradation += ((0.6 - self.hydration) / 0.1) * 0.1;
        }

        // Hyperthermie
        if self.temperature > 40.0 {
            degradation += 0.3;
        } else if self.temperature > 39.0 {
            degradation += ((self.temperature - 39.0) / 1.0) * 0.15;
        }

        degradation.clamp(0.0, 1.0)
    }

    /// Serialise l'etat pour la persistance JSON.
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

    /// Restaure l'etat depuis un JSON persistant.
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
