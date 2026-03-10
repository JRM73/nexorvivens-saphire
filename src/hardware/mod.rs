// hardware/ — Stub for the lite edition

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub gpu: Option<String>,
    pub cpu: String,
    pub ram_gb: f64,
    pub has_ollama: bool,
}

impl Default for HardwareProfile {
    fn default() -> Self {
        Self {
            gpu: None,
            cpu: "unknown".into(),
            ram_gb: 0.0,
            has_ollama: false,
        }
    }
}

pub struct HardwareRecommendation {
    pub warnings: Vec<String>,
}

impl HardwareProfile {
    pub fn detect(_ollama_url: &str) -> Self {
        Self::default()
    }

    pub fn log_summary(&self) {
        tracing::info!("Hardware: CPU={}, GPU={:?}, RAM={:.1}GB",
            self.cpu, self.gpu, self.ram_gb);
    }

    pub fn recommend(&self, _model: &str) -> HardwareRecommendation {
        HardwareRecommendation { warnings: Vec::new() }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::json!({}))
    }
}
