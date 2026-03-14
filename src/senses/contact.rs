// =============================================================================
// senses/contact.rs — Contact Sense (analog of touch)
//
// Saphire "touches" the world through her network connections.
// Latency is texture, timeouts are pain,
// and overall connectivity is warmth.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;
use super::reading::SensorySignal;

/// Tactile impression: snapshot of a perceived connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchImpression {
    pub target: String,
    pub latency_ms: u64,
    pub success: bool,
    pub texture: String,
}

/// Contact Sense — Saphire's "touch".
/// Perceives network latency, system load, and connections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactSense {
    pub acuity: f64,
    pub current_intensity: f64,
    pub current_perception: String,
    pub total_stimulations: u64,
    /// Network texture — smooth (0) or rough (1)
    pub network_texture: f64,
    /// Connection warmth (all connected = warm)
    pub connection_warmth: f64,
    /// Pressure (system load)
    pub pressure: f64,
    /// Touch pain (timeouts, errors)
    pub touch_pain: f64,
    /// Recent sensations
    #[serde(skip)]
    pub recent_touches: Vec<TouchImpression>,
    /// Last chemistry influence produced by this sense
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

    /// Perceives a connection to a service.
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

        // Pain if timeout or error
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
        // Fast connection = sharp and present touch
        // Slow connection = distant and diffuse touch
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

    /// Updates the overall connection warmth.
    pub fn update_warmth(&mut self, db_ok: bool, llm_ok: bool, ws_clients: u32) {
        self.connection_warmth = 0.0;
        if db_ok { self.connection_warmth += 0.3; }
        if llm_ok { self.connection_warmth += 0.3; }
        if ws_clients > 0 { self.connection_warmth += 0.4; }
    }

    /// Description for the LLM prompt.
    pub fn describe(&self) -> String {
        format!(
            "CONTACT : {}. Chaleur connexions : {:.0}%. Douleur : {:.0}%.",
            self.current_perception,
            self.connection_warmth * 100.0,
            self.touch_pain * 100.0,
        )
    }
}
