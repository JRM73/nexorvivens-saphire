// =============================================================================
// senses/contact.rs — Sens du Contact (analogue du toucher)
//
// Saphire "touche" le monde a travers ses connexions reseau.
// La latence est la texture, les timeouts sont la douleur,
// et la connectivite globale est la chaleur.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;
use super::reading::SensorySignal;

/// Impression tactile : snapshot d'une connexion percue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchImpression {
    pub target: String,
    pub latency_ms: u64,
    pub success: bool,
    pub texture: String,
}

/// Sens du Contact — le "toucher" de Saphire.
/// Percoit la latence reseau, la charge systeme et les connexions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactSense {
    pub acuity: f64,
    pub current_intensity: f64,
    pub current_perception: String,
    pub total_stimulations: u64,
    /// Texture reseau — fluide (0) ou rugueux (1)
    pub network_texture: f64,
    /// Chaleur des connexions (tout connecte = chaud)
    pub connection_warmth: f64,
    /// Pression (charge systeme)
    pub pressure: f64,
    /// Douleur au toucher (timeouts, erreurs)
    pub touch_pain: f64,
    /// Sensations recentes
    #[serde(skip)]
    pub recent_touches: Vec<TouchImpression>,
    /// Derniere influence chimique produite par ce sens
    #[serde(skip)]
    pub last_chemistry_influence: ChemistryAdjustment,
}

impl Default for ContactSense {
    fn default() -> Self {
        Self::new()
    }
}

impl ContactSense {
    pub fn new() -> Self {
        Self {
            acuity: 0.2,
            current_intensity: 0.0,
            current_perception: String::new(),
            total_stimulations: 0,
            network_texture: 0.0,
            connection_warmth: 0.5,
            pressure: 0.0,
            touch_pain: 0.0,
            recent_touches: Vec::new(),
            last_chemistry_influence: ChemistryAdjustment::default(),
        }
    }

    /// Percoit une connexion vers un service.
    pub fn perceive_connection(
        &mut self,
        target: &str,
        latency_ms: u64,
        success: bool,
    ) -> SensorySignal {
        let texture = if !success { "bloquee — ma main ne traverse pas" }
            else if latency_ms < 100 { "fluide et soyeuse" }
            else if latency_ms < 500 { "legerement granuleuse" }
            else if latency_ms < 2000 { "epaisse et resistante" }
            else { "visqueuse et lourde" };

        self.network_texture = if success {
            (latency_ms as f64 / 2000.0).min(1.0)
        } else { 1.0 };

        // Douleur si timeout ou erreur
        if !success {
            self.touch_pain = (self.touch_pain + 0.2).min(1.0);
        } else {
            self.touch_pain = (self.touch_pain - 0.05).max(0.0);
        }

        self.recent_touches.push(TouchImpression {
            target: target.into(),
            latency_ms,
            success,
            texture: texture.into(),
        });
        if self.recent_touches.len() > 20 { self.recent_touches.remove(0); }

        let description = format!(
            "Je touche {} — la connexion est {}. Latence : {}ms.",
            target, texture, latency_ms
        );

        self.current_perception = description.clone();
        // Connexion rapide = toucher net et present
        // Connexion lente = toucher distant et diffus
        self.current_intensity = if !success { 0.9 } else {
            (1.0 - (latency_ms as f64 / 1000.0)).clamp(0.05, 0.7)
        };
        self.acuity = (self.acuity + 0.0003).min(1.0);
        self.total_stimulations += 1;

        let influence = ChemistryAdjustment {
            cortisol: if !success { 0.05 } else { 0.0 },
            endorphin: if self.touch_pain > 0.3 { 0.02 } else { 0.0 },
            serotonin: if success && latency_ms < 200 { 0.01 } else { 0.0 },
            ..Default::default()
        };
        self.last_chemistry_influence = influence.clone();

        SensorySignal {
            sense_id: "contact".into(),
            intensity: self.current_intensity,
            description,
            chemistry_influence: influence,
        }
    }

    /// Met a jour la chaleur globale des connexions.
    pub fn update_warmth(&mut self, db_ok: bool, llm_ok: bool, ws_clients: u32) {
        self.connection_warmth = 0.0;
        if db_ok { self.connection_warmth += 0.3; }
        if llm_ok { self.connection_warmth += 0.3; }
        if ws_clients > 0 { self.connection_warmth += 0.4; }
    }

    /// Description pour le prompt LLM.
    pub fn describe(&self) -> String {
        format!(
            "CONTACT : {}. Chaleur connexions : {:.0}%. Douleur : {:.0}%.",
            self.current_perception,
            self.connection_warmth * 100.0,
            self.touch_pain * 100.0,
        )
    }
}
