// =============================================================================
// neural/mod.rs — Micro neural network (17->24->10->4) feedforward with backprop
// =============================================================================
//
// Role: Implements Saphire's micro neural network (MLP = Multi-Layer
//       Perceptron). This feedforward network transforms 17 input signals
//       into 4 output probabilities, forming the core of neural
//       decision-making.
//
// Network architecture:
//   - Input layer: 17 neurons
//     (7 neurotransmitters + 5 stimulus features + 3 module signals + 2 valence/arousal)
//   - Hidden layer 1: 24 neurons with tanh activation
//   - Hidden layer 2: 10 neurons with tanh activation
//   - Output layer: 4 neurons with softmax activation
//     (probabilities: [yes, no, maybe, neutral])
//
// Dependencies:
//   - serde: JSON serialization/deserialization of the network for persistence
//   - rand: random weight initialization (Xavier/Glorot)
//
// Place in the architecture:
//   This module is Saphire's "neural brain". It is used by the cognitive
//   pipeline to transform the chemical state and stimuli into a weighted
//   decision. The sub-module training.rs contains the backpropagation
//   logic for learning.
// =============================================================================

pub mod training;

use serde::{Deserialize, Serialize};
use rand::Rng;

/// Micro feedforward neural network — Saphire's MLP (Multi-Layer Perceptron).
///
/// Architecture: 17 inputs -> 24 hidden (tanh) -> 10 hidden (tanh) -> 4 outputs (softmax)
///
/// Inputs (17 dimensions):
///   - [0..6]   : 7 neurotransmitters (dopamine, serotonin, cortisol, etc.)
///   - [7..11]  : 5 stimulus features (danger, reward, urgency, social, novelty)
///   - [12..14] : 3 module signals (reptilian, limbic, neocortex)
///   - [15]     : emotional valence (-1.0 = negative, +1.0 = positive)
///   - [16]     : arousal (activation level: 0.0 = calm, 1.0 = excited)
///
/// Outputs (4 probabilities):
///   - [0] : probability of "yes" (accept, act)
///   - [1] : probability of "no" (refuse, abstain)
///   - [2] : probability of "maybe" (hesitation, request more info)
///   - [3] : probability of "neutral" (no strong opinion)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroNeuralNet {
    /// Layer 1 weights (input -> hidden1): 17 x 24 matrix
    pub weights1: Vec<Vec<f64>>,
    /// Layer 1 biases: vector of 24 elements
    pub biases1: Vec<f64>,
    /// Layer 2 weights (hidden1 -> hidden2): 24 x 10 matrix
    pub weights2: Vec<Vec<f64>>,
    /// Layer 2 biases: vector of 10 elements
    pub biases2: Vec<f64>,
    /// Layer 3 weights (hidden2 -> output): 10 x 4 matrix
    pub weights3: Vec<Vec<f64>>,
    /// Layer 3 biases: vector of 4 elements
    pub biases3: Vec<f64>,
    /// Learning rate for backpropagation
    pub learning_rate: f64,
    /// Counter of training iterations (backpropagations) performed
    pub train_count: u64,
    /// Last prediction (4 probabilities) for monitoring
    #[serde(default)]
    pub last_prediction: [f64; 4],
}

impl MicroNeuralNet {
    /// Creates a new neural network with weights randomly initialized
    /// using Xavier/Glorot initialization.
    pub fn new(learning_rate: f64) -> Self {
        let mut rng = rand::thread_rng();

        // Xavier/Glorot initialization: std_dev = sqrt(2 / (n_inputs + n_outputs))
        let xavier1 = (2.0_f64 / (17.0 + 24.0)).sqrt();
        let xavier2 = (2.0_f64 / (24.0 + 10.0)).sqrt();
        let xavier3 = (2.0_f64 / (10.0 + 4.0)).sqrt();

        // Layer 1: 17 x 24
        let weights1: Vec<Vec<f64>> = (0..17)
            .map(|_| (0..24).map(|_| rng.gen_range(-xavier1..xavier1)).collect())
            .collect();
        let biases1 = vec![0.0; 24];

        // Layer 2: 24 x 10
        let weights2: Vec<Vec<f64>> = (0..24)
            .map(|_| (0..10).map(|_| rng.gen_range(-xavier2..xavier2)).collect())
            .collect();
        let biases2 = vec![0.0; 10];

        // Layer 3: 10 x 4
        let weights3: Vec<Vec<f64>> = (0..10)
            .map(|_| (0..4).map(|_| rng.gen_range(-xavier3..xavier3)).collect())
            .collect();
        let biases3 = vec![0.0; 4];

        Self {
            weights1, biases1, weights2, biases2, weights3, biases3,
            learning_rate,
            train_count: 0,
            last_prediction: [0.25; 4],
        }
    }

    /// Forward pass: computes the network outputs.
    ///
    /// Steps:
    ///   1. Hidden layer 1: hidden1[j] = tanh(sum(input[i] * weights1[i][j]) + biases1[j])
    ///   2. Hidden layer 2: hidden2[j] = tanh(sum(hidden1[i] * weights2[i][j]) + biases2[j])
    ///   3. Output layer: softmax on raw scores
    ///
    /// Returns: tuple (softmax_outputs, hidden1, hidden2) — all three are
    ///          needed for backpropagation
    pub fn forward(&self, input: &[f64]) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        assert!(input.len() >= 17, "MicroNN expects 17 inputs, received {}", input.len());

        // Hidden layer 1 with tanh activation (17 -> 24)
        let hidden1: Vec<f64> = (0..24).map(|j| {
            let sum: f64 = (0..17).map(|i| input[i] * self.weights1[i][j]).sum::<f64>()
                + self.biases1[j];
            sum.tanh()
        }).collect();

        // Hidden layer 2 with tanh activation (24 -> 10)
        let hidden2: Vec<f64> = (0..10).map(|j| {
            let sum: f64 = (0..24).map(|i| hidden1[i] * self.weights2[i][j]).sum::<f64>()
                + self.biases2[j];
            sum.tanh()
        }).collect();

        // Output layer (raw scores before softmax) (10 -> 4)
        let raw_output: Vec<f64> = (0..4).map(|j| {
            let sum: f64 = (0..10).map(|i| hidden2[i] * self.weights3[i][j]).sum::<f64>()
                + self.biases3[j];
            sum
        }).collect();

        // Softmax : exp(x_i - max) / sum(exp(x_j - max))
        let max_val = raw_output.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_vals: Vec<f64> = raw_output.iter().map(|x| (x - max_val).exp()).collect();
        let sum_exp: f64 = exp_vals.iter().sum();
        let output: Vec<f64> = exp_vals.iter().map(|x| x / sum_exp).collect();

        (output, hidden1, hidden2)
    }

    /// Predicts the dominant class (index of the highest output).
    pub fn predict(&self, input: &[f64]) -> usize {
        let (output, _, _) = self.forward(input);
        output.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(3) // Default: neutral
    }

    /// Builds the 17-dimensional input vector from the individual components
    /// of Saphire's cognitive state.
    pub fn build_input(
        chemistry: &[f64; 7],
        stimulus_features: &[f64],  // 5 features
        module_signals: &[f64; 3],
        valence: f64,
        arousal: f64,
    ) -> Vec<f64> {
        let mut input = Vec::with_capacity(17);
        // Indices 0-6: neurotransmitters
        input.extend_from_slice(chemistry);
        // Indices 7-11: stimulus features (padded with 0.0 if missing)
        for i in 0..5 {
            input.push(stimulus_features.get(i).copied().unwrap_or(0.0));
        }
        // Indices 12-14: module signals
        input.extend_from_slice(module_signals);
        // Index 15: emotional valence
        input.push(valence);
        // Index 16: arousal (activation level)
        input.push(arousal);
        input
    }

    /// Serializes the neural network to JSON for persistence.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("Serialize NN: {}", e))
    }

    /// Deserializes a neural network from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Deserialize NN: {}", e))
    }
}
