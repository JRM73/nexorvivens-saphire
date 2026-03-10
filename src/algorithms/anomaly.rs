// =============================================================================
// anomaly.rs — Détection d'anomalies par Z-Score
// =============================================================================
//
// Rôle : Implémente un détecteur d'anomalies basé sur le Z-Score (score
//        standard). Une valeur est considérée comme anomalie si son écart
//        par rapport à la moyenne dépasse un seuil en nombre d'écarts-types.
//
// Dépendances : aucune (calcul statistique pur)
//
// Place dans l'architecture :
//   Utilisé par Saphire pour détecter des changements anormaux dans ses
//   niveaux de neurotransmetteurs, son humeur, ou toute métrique interne.
//   Permet une forme de « conscience des anomalies » — un signal d'alerte
//   quand quelque chose sort de l'ordinaire. Fait partie du sous-module
//   algorithms/.
// =============================================================================

/// Détecteur d'anomalies par Z-Score — maintient un historique glissant
/// de valeurs et calcule le Z-Score de chaque nouvelle observation.
///
/// Le Z-Score mesure combien d'écarts-types une valeur est éloignée de la
/// moyenne : z = (valeur - moyenne) / écart_type
pub struct ZScoreDetector {
    /// Historique des valeurs observées (fenêtre glissante)
    history: Vec<f64>,
    /// Taille maximale de l'historique (les plus anciennes valeurs sont supprimées)
    max_size: usize,
    /// Seuil Z-Score au-delà duquel une valeur est considérée comme anomalie
    /// (typiquement 2.0 ou 3.0 — respectivement ~5% ou ~0.3% des données normales)
    threshold: f64,
}

impl ZScoreDetector {
    /// Crée un nouveau détecteur d'anomalies.
    ///
    /// Paramètre `max_size` : taille de la fenêtre d'historique glissante
    /// Paramètre `threshold` : seuil Z-Score pour la détection d'anomalie
    /// Retourne : une instance de ZScoreDetector vide
    pub fn new(max_size: usize, threshold: f64) -> Self {
        Self {
            history: Vec::new(),
            max_size,
            threshold,
        }
    }

    /// Observe une nouvelle valeur et détermine si c'est une anomalie.
    ///
    /// Ajoute la valeur à l'historique, maintient la fenêtre glissante,
    /// puis calcule la moyenne, l'écart-type et le Z-Score de la nouvelle
    /// valeur.
    ///
    /// Pourquoi 5 minimum : avec moins de 5 observations, les statistiques
    /// sont trop instables pour détecter des anomalies de manière fiable.
    ///
    /// Paramètre `value` : nouvelle valeur observée
    /// Retourne : un AnomalyResult contenant le diagnostic et les statistiques
    pub fn observe(&mut self, value: f64) -> AnomalyResult {
        // Ajouter la valeur à l'historique
        self.history.push(value);
        // Maintenir la fenêtre glissante en supprimant la valeur la plus ancienne
        if self.history.len() > self.max_size {
            self.history.remove(0);
        }

        // Pas assez de données pour des statistiques fiables
        if self.history.len() < 5 {
            return AnomalyResult {
                is_anomaly: false,
                z_score: 0.0,
                mean: value,
                std_dev: 0.0,
            };
        }

        // Calculer la moyenne de l'historique
        let n = self.history.len() as f64;
        let mean = self.history.iter().sum::<f64>() / n;
        // Calculer la variance (moyenne des carrés des écarts)
        let variance = self.history.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        // Écart-type = racine carrée de la variance
        let std_dev = variance.sqrt();

        // Calculer le Z-Score : nombre d'écarts-types entre la valeur et la moyenne
        // Protection contre la division par zéro si l'écart-type est quasi-nul
        let z_score = if std_dev > 1e-10 {
            (value - mean) / std_dev
        } else {
            0.0
        };

        AnomalyResult {
            // Une anomalie est détectée si le Z-Score en valeur absolue dépasse le seuil
            is_anomaly: z_score.abs() > self.threshold,
            z_score,
            mean,
            std_dev,
        }
    }

    /// Retourne la moyenne actuelle de l'historique.
    ///
    /// Retourne : la moyenne des valeurs dans l'historique, ou 0.0 si vide
    pub fn mean(&self) -> f64 {
        if self.history.is_empty() { return 0.0; }
        self.history.iter().sum::<f64>() / self.history.len() as f64
    }
}

/// Résultat de la détection d'anomalie — contient le diagnostic et les
/// statistiques utilisées pour le calcul.
pub struct AnomalyResult {
    /// Vrai si la valeur est considérée comme une anomalie (|z_score| > seuil)
    pub is_anomaly: bool,
    /// Z-Score calculé : (valeur - moyenne) / écart_type
    /// Positif si la valeur est au-dessus de la moyenne, négatif sinon
    pub z_score: f64,
    /// Moyenne de l'historique au moment de l'observation
    pub mean: f64,
    /// Écart-type de l'historique au moment de l'observation
    pub std_dev: f64,
}
