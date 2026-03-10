// =============================================================================
// predictive.rs — Moteur de prediction (Predictive Processing / Free Energy)
// =============================================================================
//
// Role : Implemente le cadre theorique du Predictive Processing (Karl Friston).
// Le cerveau est une machine a predictions : il genere en permanence des
// predictions sur l'etat futur, puis compare avec la realite.
// L'ecart = "erreur de prediction" = surprise = signal d'apprentissage.
//
// References scientifiques :
//   - Free Energy Principle (Friston 2010)
//   - Predictive Coding (Rao & Ballard 1999)
//   - Active Inference (Friston 2009)
// =============================================================================

use serde::{Deserialize, Serialize};

/// Prediction d'un etat futur — genere a chaque cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// Chimie predite [9 molecules]
    pub predicted_chemistry: [f64; 9],
    /// Emotion dominante predite
    pub predicted_emotion: String,
    /// Confiance dans la prediction [0.0, 1.0]
    pub confidence: f64,
    /// Cycle auquel la prediction a ete faite
    pub cycle: u64,
}

/// Resultat de la comparaison prediction vs realite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionError {
    /// Erreur chimique par molecule [9 valeurs]
    pub chemistry_errors: [f64; 9],
    /// Erreur chimique totale (norme L2)
    pub total_chemistry_error: f64,
    /// L'emotion predite etait-elle correcte ?
    pub emotion_correct: bool,
    /// Surprise globale [0.0, 1.0] — combinaison des erreurs
    pub surprise: f64,
}

/// Moteur de prediction — maintient un modele generatif interne.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveEngine {
    /// Derniere prediction faite
    pub last_prediction: Option<Prediction>,
    /// Historique des erreurs de prediction (50 derniers cycles)
    pub error_history: Vec<f64>,
    /// Precision du modele predictif [0.0, 1.0] — s'ameliore avec l'experience
    pub model_precision: f64,
    /// Taux d'apprentissage du modele (vitesse d'adaptation)
    pub learning_rate: f64,
    /// Poids du modele generatif — matrice 9x9 (chimie → chimie)
    /// Predit l'etat chimique suivant a partir de l'etat courant
    pub generative_weights: [[f64; 9]; 9],
    /// Biais du modele generatif
    pub generative_bias: [f64; 9],
    /// Compteur de cycles
    pub cycle_count: u64,
}

impl Default for PredictiveEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PredictiveEngine {
    /// Cree un moteur de prediction avec un modele generatif initial.
    /// Le modele initial est une matrice identite (predit la stabilite).
    pub fn new() -> Self {
        // Matrice identite : predit que chaque molecule reste stable
        let mut weights = [[0.0f64; 9]; 9];
        for i in 0..9 {
            weights[i][i] = 0.95; // Prediction : 95% de l'etat courant persiste
        }
        // Quelques interactions connues
        weights[1][0] = -0.02; // Cortisol supprime dopamine
        weights[0][6] = 0.01;  // Dopamine booste noradrenaline
        weights[7][3] = -0.02; // GABA reduit adrenaline
        weights[8][6] = 0.01;  // Glutamate booste noradrenaline

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

    /// Genere une prediction de l'etat chimique au prochain cycle.
    /// Utilise le modele generatif interne (multiplication matrice x vecteur + biais).
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
            predicted_emotion: current_emotion.to_string(), // Prediction naive : meme emotion
            confidence: self.model_precision,
            cycle: self.cycle_count,
        };

        self.last_prediction = Some(prediction.clone());
        prediction
    }

    /// Compare la prediction precedente avec l'etat reel.
    /// Retourne l'erreur de prediction et met a jour le modele generatif.
    pub fn compute_error(
        &mut self,
        actual_chemistry: &[f64; 9],
        actual_emotion: &str,
    ) -> Option<PredictionError> {
        let prediction = self.last_prediction.as_ref()?;

        // Erreur chimique par molecule
        let mut chemistry_errors = [0.0f64; 9];
        let mut total_sq_error = 0.0;
        for i in 0..9 {
            chemistry_errors[i] = actual_chemistry[i] - prediction.predicted_chemistry[i];
            total_sq_error += chemistry_errors[i].powi(2);
        }
        let total_chemistry_error = (total_sq_error / 9.0).sqrt(); // RMSE

        // Emotion correcte ?
        let emotion_correct = prediction.predicted_emotion == actual_emotion;

        // Surprise globale [0, 1]
        let chem_surprise = (total_chemistry_error * 3.0).min(1.0); // Echelonne sur [0,1]
        let emotion_surprise = if emotion_correct { 0.0 } else { 0.5 };
        let surprise = (chem_surprise * 0.7 + emotion_surprise * 0.3).clamp(0.0, 1.0);

        // Historique
        self.error_history.push(surprise);
        if self.error_history.len() > 50 {
            self.error_history.remove(0);
        }

        // Mise a jour du modele generatif (apprentissage par erreur de prediction)
        self.update_model(actual_chemistry, &chemistry_errors);

        // Mettre a jour la precision du modele
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

    /// Met a jour les poids du modele generatif via descente de gradient.
    /// Regle delta : w += learning_rate * erreur * input
    fn update_model(&mut self, actual: &[f64; 9], errors: &[f64; 9]) {
        if let Some(ref pred) = self.last_prediction {
            for i in 0..9 {
                // Mise a jour des poids : w[j][i] += lr * error[i] * input[j]
                for j in 0..9 {
                    let input = actual[j]; // utiliser l'etat reel comme input
                    self.generative_weights[j][i] += self.learning_rate * errors[i] * input;
                    // Clamp pour eviter l'explosion
                    self.generative_weights[j][i] = self.generative_weights[j][i].clamp(-2.0, 2.0);
                }
                // Mise a jour du biais
                self.generative_bias[i] += self.learning_rate * errors[i] * 0.1;
                self.generative_bias[i] = self.generative_bias[i].clamp(-0.5, 0.5);
            }
            let _ = pred; // eviter le warning unused
        }
    }

    /// Surprise moyenne sur les N derniers cycles.
    pub fn average_surprise(&self, n: usize) -> f64 {
        if self.error_history.is_empty() { return 0.5; }
        let recent: Vec<&f64> = self.error_history.iter().rev().take(n).collect();
        recent.iter().copied().sum::<f64>() / recent.len() as f64
    }

    /// Resume pour le dashboard.
    pub fn summary(&self) -> PredictiveSummary {
        PredictiveSummary {
            model_precision: self.model_precision,
            average_surprise_10: self.average_surprise(10),
            average_surprise_50: self.average_surprise(50),
            cycle_count: self.cycle_count,
            last_prediction_confidence: self.last_prediction.as_ref().map(|p| p.confidence).unwrap_or(0.0),
        }
    }

    /// Serialise pour persistance.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "model_precision": self.model_precision,
            "generative_weights": self.generative_weights,
            "generative_bias": self.generative_bias,
            "cycle_count": self.cycle_count,
        })
    }

    /// Restaure depuis JSON persiste.
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

/// Resume du moteur predictif pour le dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveSummary {
    pub model_precision: f64,
    pub average_surprise_10: f64,
    pub average_surprise_50: f64,
    pub cycle_count: u64,
    pub last_prediction_confidence: f64,
}
