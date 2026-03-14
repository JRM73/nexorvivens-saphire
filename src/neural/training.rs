// =============================================================================
// training.rs — Backpropagation for the micro neural network (3 layers)
// =============================================================================
//
// Role: Implements the gradient backpropagation algorithm to train
//       Saphire's MicroNeuralNet.
//       3-layer architecture: input(17) -> hidden1(24) -> hidden2(10) -> output(4)
// =============================================================================

use super::MicroNeuralNet;

impl MicroNeuralNet {
    /// Trains the network by gradient backpropagation.
    ///
    /// The algorithm proceeds in 6 steps:
    ///   1. Forward pass -> (output, hidden1, hidden2)
    ///   2. Output error = output - target
    ///   3. Update weights3/biases3 (hidden2 -> output)
    ///   4. Backpropagate to hidden2, tanh derivative, update weights2/biases2
    ///   5. Backpropagate to hidden1, tanh derivative, update weights1/biases1
    ///   6. Clamp all 3 layers
    #[allow(clippy::needless_range_loop)]
    pub fn train(&mut self, input: &[f64], target: &[f64]) {
        if input.len() < 17 || target.len() < 4 { return; }

        // Step 1: forward propagation
        let (output, hidden1, hidden2) = self.forward(input);

        // Save the last prediction for monitoring
        self.last_prediction = [output[0], output[1], output[2], output[3]];

        // Step 2: output layer error (cross-entropy + softmax derivative)
        let output_error: Vec<f64> = (0..4)
            .map(|j| output[j] - target[j])
            .collect();

        // Step 3: update weights3/biases3 (hidden2 -> output)
        for i in 0..10 {
            for j in 0..4 {
                self.weights3[i][j] -= self.learning_rate * output_error[j] * hidden2[i];
            }
        }
        for j in 0..4 {
            self.biases3[j] -= self.learning_rate * output_error[j];
        }

        // Step 4: backpropagation to hidden2
        let mut hidden2_error = [0.0; 10];
        for i in 0..10 {
            for j in 0..4 {
                hidden2_error[i] += output_error[j] * self.weights3[i][j];
            }
            // Derivative of tanh: d/dx tanh(x) = 1 - tanh(x)^2
            hidden2_error[i] *= 1.0 - hidden2[i] * hidden2[i];
        }

        // Update weights2/biases2 (hidden1 -> hidden2)
        for i in 0..24 {
            for j in 0..10 {
                self.weights2[i][j] -= self.learning_rate * hidden2_error[j] * hidden1[i];
            }
        }
        for j in 0..10 {
            self.biases2[j] -= self.learning_rate * hidden2_error[j];
        }

        // Step 5: backpropagation to hidden1
        let mut hidden1_error = [0.0; 24];
        for i in 0..24 {
            for j in 0..10 {
                hidden1_error[i] += hidden2_error[j] * self.weights2[i][j];
            }
            hidden1_error[i] *= 1.0 - hidden1[i] * hidden1[i];
        }

        // Update weights1/biases1 (input -> hidden1)
        for i in 0..17 {
            for j in 0..24 {
                self.weights1[i][j] -= self.learning_rate * hidden1_error[j] * input[i];
            }
        }
        for j in 0..24 {
            self.biases1[j] -= self.learning_rate * hidden1_error[j];
        }

        self.train_count += 1;

        // Clamp weights to prevent gradient explosion
        self.clamp_weights();
    }

    /// Creates the target vector (soft target) from a satisfaction score.
    pub fn satisfaction_to_target(satisfaction: f64) -> Vec<f64> {
        if satisfaction > 0.7 {
            vec![0.8, 0.05, 0.1, 0.05]
        } else if satisfaction > 0.5 {
            vec![0.5, 0.1, 0.3, 0.1]
        } else if satisfaction < 0.3 {
            vec![0.05, 0.8, 0.1, 0.05]
        } else {
            vec![0.1, 0.1, 0.7, 0.1]
        }
    }

    /// Clamps (bounds) all network weights to the interval [-5, 5].
    fn clamp_weights(&mut self) {
        for row in &mut self.weights1 {
            for w in row {
                *w = w.clamp(-5.0, 5.0);
            }
        }
        for row in &mut self.weights2 {
            for w in row {
                *w = w.clamp(-5.0, 5.0);
            }
        }
        for row in &mut self.weights3 {
            for w in row {
                *w = w.clamp(-5.0, 5.0);
            }
        }
    }
}
