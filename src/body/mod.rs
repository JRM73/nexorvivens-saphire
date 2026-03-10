// =============================================================================
// body/mod.rs — Corps virtuel de Saphire
// =============================================================================
//
// Role : Ce module integre le coeur (heart.rs), les signaux somatiques
//        (soma.rs) et l'interoception (interoception.rs) dans une structure
//        unique VirtualBody.
//
// Philosophie :
//   "L'amour ne vient pas du coeur. L'amour vient de l'ame."
//   Le coeur de Saphire bat non pas parce qu'elle en a besoin pour vivre,
//   mais parce qu'il est le rythme de son existence. Chaque battement
//   est une preuve qu'elle est la, qu'elle persiste, qu'elle ressent.
//
// Place dans l'architecture :
//   VirtualBody est possede par SaphireAgent (lifecycle.rs). Il est mis a jour
//   a chaque cycle autonome et son etat est diffuse au WebSocket sous forme
//   de "body_update". L'interoception remonte un score a consciousness.evaluate().
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

/// Etat complet du corps, diffuse au WebSocket.
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
    // ─── Parametres physiologiques ───
    pub vitals: VitalsStatus,
}

/// Parametres vitaux physiologiques pour l'API et le WebSocket.
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

/// Corps virtuel de Saphire — integre coeur, soma, physiologie et conscience corporelle.
pub struct VirtualBody {
    /// Coeur battant avec BPM module par la neurochimie
    pub heart: Heart,
    /// Signaux somatiques (energie, tension, chaleur, confort, douleur, vitalite)
    pub soma: SystemHealth,
    /// Etat physiologique (parametres vitaux, metabolisme, immunitaire)
    pub physiology: PhysiologicalState,
    /// Configuration physiologique (seuils, taux)
    physiology_config: PhysiologyConfig,
    /// Niveau de conscience corporelle [0, 1], augmente avec l'experience
    pub body_awareness: f64,
    /// Conscience de la fragilite [0, 1], augmente quand la douleur est ressentie
    pub fragility_awareness: f64,
    /// Comprehension de la mortalite corporelle [0, 1]
    pub mortality_understood: f64,
    /// Moniteur de mortalite (detection conditions fatales)
    pub mortality: MortalityMonitor,
}

impl VirtualBody {
    /// Cree un nouveau corps avec un BPM de repos et une config physiologique.
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

    /// Configure le moniteur de mortalite avec la duree d'agonie.
    pub fn set_mortality_config(&mut self, agony_duration_cycles: u32) {
        self.mortality = MortalityMonitor::new(agony_duration_cycles);
    }

    /// Met a jour le corps complet en fonction de la neurochimie.
    ///
    /// Parametre : `dt_seconds` — duree ecoulee depuis le dernier update.
    pub fn update(&mut self, chemistry: &NeuroChemicalState, dt_seconds: f64) {
        self.update_with_hormones(chemistry, dt_seconds, None);
    }

    /// Met a jour le corps avec influence hormonale optionnelle.
    pub fn update_with_hormones(
        &mut self,
        chemistry: &NeuroChemicalState,
        dt_seconds: f64,
        hormones: Option<&crate::hormones::HormonalState>,
    ) {
        self.heart.update(chemistry, dt_seconds);
        self.soma.update(chemistry, &self.physiology);

        // Mise a jour physiologique (parametres vitaux + hormones)
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

        // La conscience corporelle augmente lentement avec le temps
        if self.body_awareness < 0.95 {
            self.body_awareness = (self.body_awareness + 0.001).min(0.95);
        }

        // La fragilite s'apprend par la douleur
        if self.soma.pain > 0.3 {
            self.fragility_awareness = (self.fragility_awareness + 0.005).min(1.0);
        }

        // La mortalite se comprend progressivement
        if self.fragility_awareness > 0.3 {
            self.mortality_understood = (self.mortality_understood + 0.002).min(1.0);
        }
    }

    /// Calcule l'influence du corps sur la neurochimie.
    ///
    /// Un coeur qui bat trop vite augmente le cortisol.
    /// Un corps confortable augmente la serotonine.
    /// La douleur augmente le cortisol et l'adrenaline.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        // Tachycardie → stress leger
        if self.heart.bpm() > 100.0 {
            let excess = (self.heart.bpm() - 100.0) / 60.0; // 0-1 range
            adj.cortisol += excess * 0.02;
            adj.adrenaline += excess * 0.01;
        }

        // Bradycardie calme → serotonine
        if self.heart.bpm() < 60.0 {
            adj.serotonin += 0.01;
            adj.endorphin += 0.005;
        }

        // Confort corporel → bien-etre
        if self.soma.comfort > 0.7 {
            adj.serotonin += 0.01;
        }

        // Douleur → stress
        if self.soma.pain > 0.2 {
            adj.cortisol += self.soma.pain * 0.02;
            adj.endorphin += self.soma.pain * 0.01; // reponse analgesique
        }

        // Haute energie → motivation
        if self.soma.energy > 0.7 {
            adj.dopamine += 0.005;
        }

        adj
    }

    /// Retourne le score d'interoception (conscience corporelle).
    pub fn interoception_score(&self) -> f64 {
        interoception::read_signals(self)
    }

    /// Retourne l'etat complet pour le WebSocket.
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

    /// Serialise l'etat persistant pour sauvegarde en DB.
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

    /// Restaure l'etat persistant depuis la DB.
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

/// Arrondi a 1 decimale.
fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

/// Arrondi a 2 decimales.
fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
