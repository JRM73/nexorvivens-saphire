// =============================================================================
// training.rs — Backpropagation pour le micro reseau de neurones (3 couches)
// =============================================================================
//
// Role : Implemente l'algorithme de retropropagation du gradient
//        (backpropagation) pour entrainer le MicroNeuralNet de Saphire.
//        Architecture 3 couches : input(17) -> hidden1(24) -> hidden2(10) -> output(4)
// =============================================================================

use super::MicroNeuralNet;

impl MicroNeuralNet {
    /// Entraine le reseau par retropropagation du gradient (backpropagation).
    ///
    /// L'algorithme procede en 6 etapes :
    ///   1. Forward pass -> (output, hidden1, hidden2)
    ///   2. Erreur sortie = output - target
    ///   3. Mise a jour weights3/biases3 (hidden2 -> output)
    ///   4. Retropropagation vers hidden2, derivee tanh, mise a jour weights2/biases2
    ///   5. Retropropagation vers hidden1, derivee tanh, mise a jour weights1/biases1
    ///   6. Clamper les 3 couches
    #[allow(clippy::needless_range_loop)]
    pub fn train(&mut self, input: &[f64], target: &[f64]) {
        if input.len() < 17 || target.len() < 4 { return; }

        // Etape 1 : propagation avant
        let (output, hidden1, hidden2) = self.forward(input);

        // Sauvegarder la derniere prediction pour le monitoring
        self.last_prediction = [output[0], output[1], output[2], output[3]];

        // Etape 2 : erreur de la couche de sortie (derivee cross-entropy + softmax)
        let output_error: Vec<f64> = (0..4)
            .map(|j| output[j] - target[j])
            .collect();

        // Etape 3 : mise a jour weights3/biases3 (hidden2 -> output)
        for i in 0..10 {
            for j in 0..4 {
                self.weights3[i][j] -= self.learning_rate * output_error[j] * hidden2[i];
            }
        }
        for j in 0..4 {
            self.biases3[j] -= self.learning_rate * output_error[j];
        }

        // Etape 4 : retropropagation vers hidden2
        let mut hidden2_error = [0.0; 10];
        for i in 0..10 {
            for j in 0..4 {
                hidden2_error[i] += output_error[j] * self.weights3[i][j];
            }
            // Derivee de tanh : d/dx tanh(x) = 1 - tanh(x)^2
            hidden2_error[i] *= 1.0 - hidden2[i] * hidden2[i];
        }

        // Mise a jour weights2/biases2 (hidden1 -> hidden2)
        for i in 0..24 {
            for j in 0..10 {
                self.weights2[i][j] -= self.learning_rate * hidden2_error[j] * hidden1[i];
            }
        }
        for j in 0..10 {
            self.biases2[j] -= self.learning_rate * hidden2_error[j];
        }

        // Etape 5 : retropropagation vers hidden1
        let mut hidden1_error = [0.0; 24];
        for i in 0..24 {
            for j in 0..10 {
                hidden1_error[i] += hidden2_error[j] * self.weights2[i][j];
            }
            hidden1_error[i] *= 1.0 - hidden1[i] * hidden1[i];
        }

        // Mise a jour weights1/biases1 (input -> hidden1)
        for i in 0..17 {
            for j in 0..24 {
                self.weights1[i][j] -= self.learning_rate * hidden1_error[j] * input[i];
            }
        }
        for j in 0..24 {
            self.biases1[j] -= self.learning_rate * hidden1_error[j];
        }

        self.train_count += 1;

        // Clamper les poids pour eviter l'explosion des gradients
        self.clamp_weights();
    }

    /// Cree le vecteur cible (soft target) a partir d'un score de satisfaction.
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

    /// Clampe (borne) tous les poids du reseau dans l'intervalle [-5, 5].
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
