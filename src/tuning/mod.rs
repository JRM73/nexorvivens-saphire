// =============================================================================
// tuning/mod.rs — Auto-tuning des coefficients du cerveau
//
// Role : Ce fichier contient le systeme d'auto-ajustement (auto-tuning) des
// coefficients du cerveau de Saphire. Il observe la satisfaction, la coherence
// et le niveau de conscience sur un tampon d'observations, puis ajuste
// automatiquement les poids des modules, les seuils de decision et les taux
// de retroaction pour maximiser la satisfaction moyenne de l'agent.
//
// Dependances :
//   - std::collections::VecDeque : tampon circulaire pour les observations
//   - serde_json : serialisation des parametres pour la persistance
//   - self::params::TunableParams : structure des parametres ajustables
//
// Place dans l'architecture :
//   Le CoefficientTuner est possede par l'agent (SaphireAgent). A chaque cycle,
//   une observation est enregistree. Periodiquement (tous les N cycles), le tuner
//   analyse le tampon et ajuste les parametres. Les meilleurs parametres sont
//   sauvegardes en base de donnees pour etre restaures au prochain demarrage.
// =============================================================================

// Sous-module definissant la structure des parametres ajustables
pub mod params;

use std::collections::{HashMap, VecDeque};
use self::params::TunableParams;

/// Observation enregistree a chaque cycle pour alimenter le tuning.
/// Le tuner accumule ces observations dans un tampon circulaire.
#[derive(Debug, Clone)]
pub struct TuningObservation {
    /// Decision prise : -1 (Non), 0 (Peut-etre), 1 (Oui)
    pub decision: i8,
    /// Niveau de satisfaction ressentie apres la decision [0.0 - 1.0]
    pub satisfaction: f64,
    /// Coherence du consensus (accord entre les modules) [0.0 - 1.0]
    pub coherence: f64,
    /// Niveau de conscience au moment de la decision [0.0 - 1.0]
    pub consciousness_level: f64,
    /// Nom de l'emotion dominante au moment de l'observation
    pub emotion_name: String,
    /// Niveau de cortisol au moment de l'observation [0.0 - 1.0]
    pub cortisol: f64,
}

/// Auto-tuner des coefficients du cerveau.
/// Utilise une approche de recherche locale : observe les resultats, calcule
/// un score composite, et ajuste incrementalement les parametres pour ameliorer
/// la satisfaction globale.
pub struct CoefficientTuner {
    /// Tampon circulaire des observations recentes (FIFO = First In, First Out)
    observation_buffer: VecDeque<TuningObservation>,
    /// Taille maximale du tampon (les observations les plus anciennes sont supprimees)
    buffer_size: usize,
    /// Parametres actuels utilises par le cerveau
    pub current_params: TunableParams,
    /// Meilleurs parametres trouves jusqu'ici (ceux avec le meilleur score)
    best_params: TunableParams,
    /// Meilleur score de satisfaction atteint
    best_avg_satisfaction: f64,
    /// Nombre de cycles entre chaque tentative de tuning
    tuning_interval: u64,
    /// Compteur de cycles depuis le dernier tuning
    cycles_since_tuning: u64,
    /// Taux d'apprentissage : amplitude des ajustements a chaque tuning
    tuning_rate: f64,
    /// Nombre total de cycles de tuning effectues depuis la creation
    pub tuning_count: u64,
}

impl CoefficientTuner {
    /// Cree un nouveau tuner avec les parametres par defaut.
    ///
    /// # Parametres
    /// - `buffer_size` : taille du tampon d'observations
    /// - `tuning_interval` : nombre de cycles entre chaque tuning
    /// - `tuning_rate` : amplitude des ajustements
    pub fn new(buffer_size: usize, tuning_interval: u64, tuning_rate: f64) -> Self {
        let params = TunableParams::default();
        Self {
            observation_buffer: VecDeque::with_capacity(buffer_size),
            buffer_size,
            current_params: params.clone(),
            best_params: params,
            best_avg_satisfaction: 0.0,
            tuning_interval,
            cycles_since_tuning: 0,
            tuning_rate,
            tuning_count: 0,
        }
    }

    /// Enregistre une observation dans le tampon circulaire.
    /// Si le tampon est plein, l'observation la plus ancienne est supprimee.
    ///
    /// # Parametres
    /// - `obs` : l'observation du cycle actuel
    pub fn observe(&mut self, obs: TuningObservation) {
        self.observation_buffer.push_back(obs);
        if self.observation_buffer.len() > self.buffer_size {
            self.observation_buffer.pop_front();
        }
        self.cycles_since_tuning += 1;
    }

    /// Tente un cycle de tuning si l'intervalle est atteint.
    /// Retourne les nouveaux parametres si un ajustement a ete effectue,
    /// None sinon (pas assez de cycles ou pas assez d'observations).
    ///
    /// # Retour
    /// - `Some(TunableParams)` : les parametres ajustes
    /// - `None` : pas de tuning effectue
    pub fn try_tune(&mut self) -> Option<TunableParams> {
        if self.cycles_since_tuning < self.tuning_interval {
            return None;
        }
        self.cycles_since_tuning = 0;
        self.tune()
    }

    /// Effectue le tuning des parametres.
    /// Analyse le tampon d'observations et ajuste les parametres en consequence.
    ///
    /// L'algorithme :
    ///   1. Calcule le score composite (60% satisfaction + 30% coherence + 10% conscience)
    ///   2. Si le score est le meilleur, sauvegarde les parametres actuels
    ///   3. Ajuste les seuils si trop d'indecision (Peut-etre) avec faible satisfaction
    ///   4. Ajuste les poids des modules selon la correlation satisfaction/coherence
    ///
    /// # Retour
    /// - `Some(TunableParams)` si le tuning a produit de nouveaux parametres
    /// - `None` si le tampon est trop petit (< 50 observations)
    fn tune(&mut self) -> Option<TunableParams> {
        // Besoin d'au moins 50 observations pour des statistiques fiables
        if self.observation_buffer.len() < 50 {
            return None;
        }

        let n = self.observation_buffer.len() as f64;

        // Calculer les moyennes des metriques
        let avg_satisfaction = self.observation_buffer.iter()
            .map(|o| o.satisfaction).sum::<f64>() / n;
        let avg_coherence = self.observation_buffer.iter()
            .map(|o| o.coherence).sum::<f64>() / n;
        let avg_consciousness = self.observation_buffer.iter()
            .map(|o| o.consciousness_level).sum::<f64>() / n;

        // ─── Diversite emotionnelle (entropie de Shannon) ────────
        let mut emotion_counts: HashMap<&str, usize> = HashMap::new();
        for obs in self.observation_buffer.iter() {
            *emotion_counts.entry(obs.emotion_name.as_str()).or_insert(0) += 1;
        }
        let distinct_emotions = emotion_counts.len();

        // Entropie de Shannon : H = -sum(p * ln(p))
        let shannon_entropy = {
            let total = self.observation_buffer.len() as f64;
            emotion_counts.values()
                .map(|&count| {
                    let p = count as f64 / total;
                    if p > 0.0 { -p * p.ln() } else { 0.0 }
                })
                .sum::<f64>()
        };

        // Moyenne cortisol sur le tampon
        let avg_cortisol = self.observation_buffer.iter()
            .map(|o| o.cortisol).sum::<f64>() / n;

        // Penalite diversite : si < 5 emotions distinctes, retrancher du score
        let diversity_penalty = if distinct_emotions < 5 {
            0.05 * (5 - distinct_emotions) as f64
        } else {
            0.0
        };

        // Score composite avec penalite de monotonie
        let score = (avg_satisfaction - diversity_penalty) * 0.6
            + avg_coherence * 0.3
            + avg_consciousness * 0.1;

        tracing::info!(
            "[Tuner] diversite: {} emotions distinctes, Shannon={:.2}, cortisol_moy={:.3}, penalite={:.2}, score={:.3}",
            distinct_emotions, shannon_entropy, avg_cortisol, diversity_penalty, score
        );

        // Sauvegarder les meilleurs parametres si le score depasse le record
        if score > self.best_avg_satisfaction {
            self.best_avg_satisfaction = score;
            self.best_params = self.current_params.clone();
        }

        let mut new_params = self.current_params.clone();

        // ─── Ajustement des seuils de decision ───────────────────
        // Si plus de 40% des decisions sont "Peut-etre" avec une satisfaction < 0.4,
        // c'est signe que l'agent est trop indecis. On resserre les seuils pour
        // favoriser des decisions plus tranchees (Oui ou Non).
        let maybe_count = self.observation_buffer.iter()
            .filter(|o| o.decision == 0).count();
        let maybe_ratio = maybe_count as f64 / n;
        let maybe_sat = self.avg_satisfaction_for_decision(0);

        if maybe_ratio > 0.4 && maybe_sat < 0.4 {
            // Remonter le seuil "Non" (le rendre moins negatif)
            new_params.threshold_no += self.tuning_rate;
            // Abaisser le seuil "Oui" (le rendre moins positif)
            new_params.threshold_yes -= self.tuning_rate;
        }

        // ─── Ajustement des poids des modules ────────────────────
        // Comparer la coherence des decisions a haute satisfaction vs basse satisfaction.
        // Si les bonnes decisions sont plus coherentes, renforcer le poids du neocortex
        // (le module le plus rationnel).
        let high_sat: Vec<&TuningObservation> = self.observation_buffer.iter()
            .filter(|o| o.satisfaction > 0.7).collect();
        let low_sat: Vec<&TuningObservation> = self.observation_buffer.iter()
            .filter(|o| o.satisfaction < 0.3).collect();

        if high_sat.len() > 5 && low_sat.len() > 5 {
            let good_coherence = high_sat.iter().map(|o| o.coherence).sum::<f64>() / high_sat.len() as f64;
            let bad_coherence = low_sat.iter().map(|o| o.coherence).sum::<f64>() / low_sat.len() as f64;
            // Si la coherence des bonnes decisions depasse celle des mauvaises de 0.1+,
            // augmenter le poids du neocortex pour favoriser les decisions coherentes.
            if good_coherence > bad_coherence + 0.1 {
                new_params.weight_base_neocortex += self.tuning_rate;
            }
        }

        // ─── Auto-correction si chimie plate ────────────────────
        // Si trop peu d'emotions distinctes ET cortisol anormalement bas,
        // injecter plus de stress d'indecision pour stimuler la variabilite.
        if distinct_emotions < 5 && avg_cortisol < 0.15 {
            new_params.feedback_indecision_stress += self.tuning_rate;
            new_params.feedback_cortisol_relief -= self.tuning_rate / 2.0;
            tracing::info!(
                "[Tuner] auto-correction chimie plate: indecision_stress +{:.3}, cortisol_relief -{:.3}",
                self.tuning_rate, self.tuning_rate / 2.0
            );
        }

        // S'assurer que tous les parametres restent dans des bornes de securite
        new_params.clamp_all();
        self.current_params = new_params.clone();
        self.tuning_count += 1;
        // Vider le tampon pour repartir sur de nouvelles observations
        self.observation_buffer.clear();

        Some(new_params)
    }

    /// Calcule la satisfaction moyenne pour un type de decision donne.
    /// Utile pour analyser si un type de decision (Oui, Non, Peut-etre)
    /// est associe a une bonne ou mauvaise satisfaction.
    ///
    /// # Parametres
    /// - `decision` : le type de decision (-1, 0, 1)
    ///
    /// # Retour
    /// La satisfaction moyenne (0.5 par defaut si aucune observation ne correspond)
    fn avg_satisfaction_for_decision(&self, decision: i8) -> f64 {
        let matching: Vec<f64> = self.observation_buffer.iter()
            .filter(|o| o.decision == decision)
            .map(|o| o.satisfaction)
            .collect();
        if matching.is_empty() { 0.5 } else {
            matching.iter().sum::<f64>() / matching.len() as f64
        }
    }

    /// Charge les parametres depuis des chaines JSON (restauration depuis la base de donnees).
    ///
    /// # Parametres
    /// - `params_json` : parametres actuels en JSON
    /// - `best_json` : meilleurs parametres en JSON
    /// - `best_score` : meilleur score atteint
    /// - `count` : nombre de tunings effectues
    pub fn load_params(&mut self, params_json: &str, best_json: &str, best_score: f64, count: u64) {
        if let Ok(params) = serde_json::from_str::<TunableParams>(params_json) {
            self.current_params = params;
        }
        if let Ok(best) = serde_json::from_str::<TunableParams>(best_json) {
            self.best_params = best;
        }
        self.best_avg_satisfaction = best_score;
        self.tuning_count = count;
    }

    /// Serialise les parametres actuels en JSON.
    ///
    /// # Retour
    /// Chaine JSON representant les parametres actuels
    pub fn params_json(&self) -> String {
        serde_json::to_string(&self.current_params).unwrap_or_default()
    }

    /// Serialise les meilleurs parametres trouves en JSON.
    ///
    /// # Retour
    /// Chaine JSON representant les meilleurs parametres
    pub fn best_params_json(&self) -> String {
        serde_json::to_string(&self.best_params).unwrap_or_default()
    }

    /// Retourne le meilleur score de satisfaction atteint.
    pub fn best_score(&self) -> f64 {
        self.best_avg_satisfaction
    }
}
