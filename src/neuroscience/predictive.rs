// =============================================================================
// predictive.rs — Prediction engine (Predictive Processing / Free Energy)
// =============================================================================
//
// Role: Implements the Predictive Processing theoretical framework (Karl Friston).
// The brain is a prediction machine: it constantly generates predictions about
// future states, then compares them with reality.
// The gap = "prediction error" = surprise = learning signal.
//
// Scientific references:
//   - Free Energy Principle (Friston 2010)
//   - Predictive Coding (Rao & Ballard 1999)
//   - Active Inference (Friston 2009)
// =============================================================================

use serde::{Deserialize, Serialize};

/// Prediction of a future state — generated each cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// Predicted chemistry [9 molecules]
    pub predicted_chemistry: [f64; 9],
    /// Predicted dominant emotion
    pub predicted_emotion: String,
    /// Prediction confidence [0.0, 1.0]
    pub confidence: f64,
    /// Cycle at which the prediction was made
    pub cycle: u64,
}

/// Result of comparing prediction vs reality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionError {
    /// Per-molecule chemistry error [9 values]
    pub chemistry_errors: [f64; 9],
    /// Total chemistry error (L2 norm)
    pub total_chemistry_error: f64,
    /// Was the predicted emotion correct?
    pub emotion_correct: bool,
    /// Overall surprise [0.0, 1.0] — combination of errors
    pub surprise: f64,
}

/// Prediction engine — maintains an internal generative model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveEngine {
    /// Last prediction made
    pub last_prediction: Option<Prediction>,
    /// Prediction error history (last 50 cycles)
    pub error_history: Vec<f64>,
    /// Predictive model precision [0.0, 1.0] — improves with experience
    pub model_precision: f64,
    /// Model learning rate (adaptation speed)
    pub learning_rate: f64,
    /// Generative model weights — 9x9 matrix (chemistry -> chemistry)
    /// Predicts the next chemical state from the current one
    pub generative_weights: [[f64; 9]; 9],
    /// Generative model bias
    pub generative_bias: [f64; 9],
    /// Cycle counter
    pub cycle_count: u64,
}

impl Default for PredictiveEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PredictiveEngine {
    /// Creates a prediction engine with an initial generative model.
    /// The initial model is an identity matrix (predicts stability).
    pub fn new() -> Self {
        // Identity matrix: predicts that each molecule remains stable
        let mut weights = [[0.0f64; 9]; 9];
        for i in 0..9 {
            weights[i][i] = 0.95; // Prediction: 95% of current state persists
        }
        // Some known interactions
        weights[1][0] = -0.02; // Cortisol suppresses dopamine
        weights[0][6] = 0.01;  // Dopamine boosts noradrenaline
        weights[7][3] = -0.02; // GABA reduces adrenaline
        weights[8][6] = 0.01;  // Glutamate boosts noradrenaline

        Self {
            last_prediction: None,
            error_history: Vec::new(),
            model_precision: 0.3,
            learning_rate: 0.01,
            generative_weights: weights,
            generative_bias: [0.0; 9],
            cycle_count: 0,
        }
    }

    /// Generates a prediction of the chemical state for the next cycle.
    /// Uses the internal generative model (matrix-vector multiplication + bias).
    pub fn predict(&mut self, current_chemistry: &[f64; 9], current_emotion: &str) -> Prediction {
        self.cycle_count += 1;
        let mut predicted = [0.0f64; 9];
        for i in 0..9 {
            for j in 0..9 {
                predicted[i] += self.generative_weights[j][i] * current_chemistry[j];
            }
            predicted[i] += self.generative_bias[i];
            predicted[i] = predicted[i].clamp(0.0, 1.0);
        }

        let prediction = Prediction {
            predicted_chemistry: predicted,
            predicted_emotion: current_emotion.to_string(), // Naive prediction: same emotion
            confidence: self.model_precision,
            cycle: self.cycle_count,
        };

        self.last_prediction = Some(prediction.clone());
        prediction
    }

    /// Compares the previous prediction with the actual state.
    /// Returns the prediction error and updates the generative model.
    pub fn compute_error(
        &mut self,
        actual_chemistry: &[f64; 9],
        actual_emotion: &str,
    ) -> Option<PredictionError> {
        let prediction = self.last_prediction.as_ref()?;

        // Per-molecule chemistry error
        let mut chemistry_errors = [0.0f64; 9];
        let mut total_sq_error = 0.0;
        for i in 0..9 {
            chemistry_errors[i] = actual_chemistry[i] - prediction.predicted_chemistry[i];
            total_sq_error += chemistry_errors[i].powi(2);
        }
        let total_chemistry_error = (total_sq_error / 9.0).sqrt(); // RMSE

        // Emotion correct?
        let emotion_correct = prediction.predicted_emotion == actual_emotion;

        // Overall surprise [0, 1]
        let chem_surprise = (total_chemistry_error * 3.0).min(1.0); // Scaled to [0,1]
        let emotion_surprise = if emotion_correct { 0.0 } else { 0.5 };
        let surprise = (chem_surprise * 0.7 + emotion_surprise * 0.3).clamp(0.0, 1.0);

        // History
        self.error_history.push(surprise);
        if self.error_history.len() > 50 {
            self.error_history.remove(0);
        }

        // Update generative model (learning by prediction error)
        self.update_model(actual_chemistry, &chemistry_errors);

        // Update model precision
        let avg_error = if self.error_history.is_empty() { 0.5 }
            else { self.error_history.iter().sum::<f64>() / self.error_history.len() as f64 };
        self.model_precision = (1.0 - avg_error).clamp(0.1, 0.95);

        Some(PredictionError {
            chemistry_errors,
            total_chemistry_error,
            emotion_correct,
            surprise,
        })
    }

    /// Updates generative model weights via gradient descent.
    /// Delta rule: w += learning_rate * error * input
    fn update_model(&mut self, actual: &[f64; 9], errors: &[f64; 9]) {
        if let Some(ref pred) = self.last_prediction {
            for i in 0..9 {
                // Weight update: w[j][i] += lr * error[i] * input[j]
                for j in 0..9 {
                    let input = actual[j]; // use actual state as input
                    self.generative_weights[j][i] += self.learning_rate * errors[i] * input;
                    // Clamp to prevent explosion
                    self.generative_weights[j][i] = self.generative_weights[j][i].clamp(-2.0, 2.0);
                }
                // Bias update
                self.generative_bias[i] += self.learning_rate * errors[i] * 0.1;
                self.generative_bias[i] = self.generative_bias[i].clamp(-0.5, 0.5);
            }
            let _ = pred; // avoid unused warning
        }
    }

    /// Average surprise over the last N cycles.
    pub fn average_surprise(&self, n: usize) -> f64 {
        if self.error_history.is_empty() { return 0.5; }
        let recent: Vec<&f64> = self.error_history.iter().rev().take(n).collect();
        recent.iter().copied().sum::<f64>() / recent.len() as f64
    }

    /// Summary for the dashboard.
    pub fn summary(&self) -> PredictiveSummary {
        PredictiveSummary {
            model_precision: self.model_precision,
            average_surprise_10: self.average_surprise(10),
            average_surprise_50: self.average_surprise(50),
            cycle_count: self.cycle_count,
            last_prediction_confidence: self.last_prediction.as_ref().map(|p| p.confidence).unwrap_or(0.0),
        }
    }

    /// Serializes for persistence.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "model_precision": self.model_precision,
            "generative_weights": self.generative_weights,
            "generative_bias": self.generative_bias,
            "cycle_count": self.cycle_count,
        })
    }

    /// Restores from persisted JSON.
    pub fn restore_from_json(&mut self, json: &serde_json::Value) {
        if let Some(p) = json.get("model_precision").and_then(|v| v.as_f64()) {
            self.model_precision = p;
        }
        if let Some(c) = json.get("cycle_count").and_then(|v| v.as_u64()) {
            self.cycle_count = c;
        }
        if let Some(weights) = json.get("generative_weights").and_then(|v| v.as_array()) {
            for (i, row) in weights.iter().enumerate().take(9) {
                if let Some(row_arr) = row.as_array() {
                    for (j, val) in row_arr.iter().enumerate().take(9) {
                        if let Some(w) = val.as_f64() {
                            self.generative_weights[i][j] = w;
                        }
                    }
                }
            }
        }
        if let Some(bias) = json.get("generative_bias").and_then(|v| v.as_array()) {
            for (i, val) in bias.iter().enumerate().take(9) {
                if let Some(b) = val.as_f64() {
                    self.generative_bias[i] = b;
                }
            }
        }
    }
}

/// Predictive engine summary for the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveSummary {
    pub model_precision: f64,
    pub average_surprise_10: f64,
    pub average_surprise_50: f64,
    pub cycle_count: u64,
    pub last_prediction_confidence: f64,
}
