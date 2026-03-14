// =============================================================================
// neural/mod.rs — Micro reseau de neurones (17->24->10->4) feedforward avec backprop
// =============================================================================
//
// Role : Implemente le micro-reseau de neurones (MLP = Multi-Layer Perceptron
//        = Perceptron Multi-Couches) de Saphire. Ce reseau feedforward
//        transforme 17 signaux d'entree en 4 probabilites de sortie,
//        constituant le coeur de la prise de decision neuronale.
//
// Architecture du reseau :
//   - Couche d'entree : 17 neurones
//     (7 neurotransmetteurs + 5 features stimulus + 3 signaux modules + 2 valence/arousal)
//   - Couche cachee 1 : 24 neurones avec activation tanh
//   - Couche cachee 2 : 10 neurones avec activation tanh
//   - Couche de sortie : 4 neurones avec activation softmax
//     (probabilites : [oui, non, peut-etre, neutre])
//
// Dependances :
//   - serde : serialisation/deserialisation JSON du reseau pour persistance
//   - rand : initialisation aleatoire des poids (Xavier/Glorot)
//
// Place dans l'architecture :
//   Ce module est le « cerveau neuronal » de Saphire. Il est utilise par
//   le pipeline cognitif pour transformer l'etat chimique et les stimuli
//   en une decision ponderee. Le sous-module training.rs contient la logique
//   de backpropagation pour l'apprentissage.
// =============================================================================

pub mod training;

use serde::{Deserialize, Serialize};
use rand::Rng;

/// Micro reseau de neurones feedforward — le MLP (Multi-Layer Perceptron)
/// de Saphire.
///
/// Architecture : 17 entrees -> 24 cachees (tanh) -> 10 cachees (tanh) -> 4 sorties (softmax)
///
/// Entrees (17 dimensions) :
///   - [0..6]   : 7 neurotransmetteurs (dopamine, serotonine, cortisol, etc.)
///   - [7..11]  : 5 features du stimulus (danger, recompense, urgence, social, nouveaute)
///   - [12..14] : 3 signaux des modules (reptilien, limbique, neocortex)
///   - [15]     : valence emotionnelle (-1.0 = negatif, +1.0 = positif)
///   - [16]     : arousal (niveau d'activation : 0.0 = calme, 1.0 = excite)
///
/// Sorties (4 probabilites) :
///   - [0] : probabilite de « oui » (accepter, agir)
///   - [1] : probabilite de « non » (refuser, s'abstenir)
///   - [2] : probabilite de « peut-etre » (hesitation, demander plus d'info)
///   - [3] : probabilite de « neutre » (pas d'opinion forte)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroNeuralNet {
    /// Poids de la couche 1 (entree -> cachee1) : matrice 17 x 24
    pub weights1: Vec<Vec<f64>>,
    /// Biais de la couche 1 : vecteur de 24 elements
    pub biases1: Vec<f64>,
    /// Poids de la couche 2 (cachee1 -> cachee2) : matrice 24 x 10
    pub weights2: Vec<Vec<f64>>,
    /// Biais de la couche 2 : vecteur de 10 elements
    pub biases2: Vec<f64>,
    /// Poids de la couche 3 (cachee2 -> sortie) : matrice 10 x 4
    pub weights3: Vec<Vec<f64>>,
    /// Biais de la couche 3 : vecteur de 4 elements
    pub biases3: Vec<f64>,
    /// Taux d'apprentissage (learning rate) pour la backpropagation
    pub learning_rate: f64,
    /// Compteur du nombre d'entrainements (backpropagations) effectues
    pub train_count: u64,
    /// Derniere prediction (4 probabilites) pour le monitoring
    #[serde(default)]
    pub last_prediction: [f64; 4],
}

impl MicroNeuralNet {
    /// Cree un nouveau reseau de neurones avec des poids initialises
    /// aleatoirement selon l'initialisation Xavier/Glorot.
    pub fn new(learning_rate: f64) -> Self {
        let mut rng = rand::thread_rng();

        // Initialisation Xavier/Glorot : ecart-type = sqrt(2 / (n_entrees + n_sorties))
        let xavier1 = (2.0_f64 / (17.0 + 24.0)).sqrt();
        let xavier2 = (2.0_f64 / (24.0 + 10.0)).sqrt();
        let xavier3 = (2.0_f64 / (10.0 + 4.0)).sqrt();

        // Couche 1 : 17 x 24
        let weights1: Vec<Vec<f64>> = (0..17)
            .map(|_| (0..24).map(|_| rng.gen_range(-xavier1..xavier1)).collect())
            .collect();
        let biases1 = vec![0.0; 24];

        // Couche 2 : 24 x 10
        let weights2: Vec<Vec<f64>> = (0..24)
            .map(|_| (0..10).map(|_| rng.gen_range(-xavier2..xavier2)).collect())
            .collect();
        let biases2 = vec![0.0; 10];

        // Couche 3 : 10 x 4
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

    /// Propagation avant (forward pass) : calcule les sorties du reseau.
    ///
    /// Etapes :
    ///   1. Couche cachee 1 : hidden1[j] = tanh(sum(input[i] * weights1[i][j]) + biases1[j])
    ///   2. Couche cachee 2 : hidden2[j] = tanh(sum(hidden1[i] * weights2[i][j]) + biases2[j])
    ///   3. Couche de sortie : softmax sur les scores bruts
    ///
    /// Retourne : tuple (sorties_softmax, hidden1, hidden2) — les trois sont
    ///            necessaires pour la backpropagation
    pub fn forward(&self, input: &[f64]) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        assert!(input.len() >= 17, "MicroNN attend 17 entrees, recu {}", input.len());

        // Couche cachee 1 avec activation tanh (17 -> 24)
        let hidden1: Vec<f64> = (0..24).map(|j| {
            let sum: f64 = (0..17).map(|i| input[i] * self.weights1[i][j]).sum::<f64>()
                + self.biases1[j];
            sum.tanh()
        }).collect();

        // Couche cachee 2 avec activation tanh (24 -> 10)
        let hidden2: Vec<f64> = (0..10).map(|j| {
            let sum: f64 = (0..24).map(|i| hidden1[i] * self.weights2[i][j]).sum::<f64>()
                + self.biases2[j];
            sum.tanh()
        }).collect();

        // Couche de sortie (scores bruts avant softmax) (10 -> 4)
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

    /// Predit la classe dominante (indice de la sortie la plus elevee).
    pub fn predict(&self, input: &[f64]) -> usize {
        let (output, _, _) = self.forward(input);
        output.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(3) // Defaut : neutre
    }

    /// Construit le vecteur d'entree de 17 dimensions a partir des composantes
    /// individuelles de l'etat cognitif de Saphire.
    pub fn build_input(
        chemistry: &[f64; 7],
        stimulus_features: &[f64],  // 5 features
        module_signals: &[f64; 3],
        valence: f64,
        arousal: f64,
    ) -> Vec<f64> {
        let mut input = Vec::with_capacity(17);
        // Indices 0-6 : neurotransmetteurs
        input.extend_from_slice(chemistry);
        // Indices 7-11 : features du stimulus (completees par 0.0 si manquantes)
        for i in 0..5 {
            input.push(stimulus_features.get(i).copied().unwrap_or(0.0));
        }
        // Indices 12-14 : signaux des modules
        input.extend_from_slice(module_signals);
        // Indice 15 : valence emotionnelle
        input.push(valence);
        // Indice 16 : arousal (niveau d'activation)
        input.push(arousal);
        input
    }

    /// Serialise le reseau de neurones en JSON pour la persistance.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("Serialize NN: {}", e))
    }

    /// Deserialise un reseau de neurones depuis une chaine JSON.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Deserialize NN: {}", e))
    }
}
