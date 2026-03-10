// neural/ — Stub for the lite edition
// Micro neural networks not ported.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroNeuralNet {
    pub learning_rate: f64,
    pub train_count: u64,
    pub last_prediction: [f64; 4],
}

impl MicroNeuralNet {
    pub fn new(learning_rate: f64) -> Self {
        Self { learning_rate, train_count: 0, last_prediction: [0.0; 4] }
    }

    pub fn predict(&self, _input: &[f64]) -> Vec<f64> { vec![0.5; 4] }

    pub fn train(&mut self, _input: &[f64], _target: &[f64]) {
        self.train_count += 1;
    }

    pub fn forward(&self, _input: &[f64]) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        (vec![0.25; 4], vec![], vec![])
    }

    pub fn build_input(
        chem_vec: &[f64], stimulus_features: &[f64], module_signals: &[f64],
        valence: f64, arousal: f64,
    ) -> Vec<f64> {
        let mut input = Vec::new();
        input.extend_from_slice(chem_vec);
        input.extend_from_slice(stimulus_features);
        input.extend_from_slice(module_signals);
        input.push(valence);
        input.push(arousal);
        input
    }

    pub fn satisfaction_to_target(satisfaction: f64) -> Vec<f64> {
        // P(Oui), P(Non), P(Peut-etre), satisfaction
        if satisfaction > 0.6 {
            vec![0.8, 0.1, 0.1, satisfaction]
        } else if satisfaction < 0.4 {
            vec![0.1, 0.8, 0.1, satisfaction]
        } else {
            vec![0.3, 0.3, 0.4, satisfaction]
        }
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
