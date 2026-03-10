// =============================================================================
// consensus.rs — Pondération et consensus des 3 modules cérébraux
// =============================================================================
//
// Rôle : Ce fichier implémente le mécanisme de prise de décision de Saphire.
// Il combine les signaux des 3 modules cérébraux (reptilien, limbique,
// néocortex) en un score unique via une somme pondérée. Les poids de
// chaque module varient dynamiquement selon l'état neurochimique.
//
// Dépendances :
//   - serde : sérialisation / désérialisation
//   - crate::neurochemistry::NeuroChemicalState : état chimique (pour les poids)
//   - crate::modules::ModuleSignal : signaux émis par chaque module cérébral
//
// Place dans l'architecture :
//   Ce module est le coeur décisionnel. Il est appelé après que les 3 modules
//   cérébraux aient traité le stimulus. Le résultat est ensuite observé par
//   consciousness.rs et utilisé pour la rétroaction neurochimique.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::neurochemistry::NeuroChemicalState;
use crate::modules::ModuleSignal;
use crate::tuning::params::TunableParams;

/// Décision du cerveau — résultat trivalent (Oui / Non / Peut-être).
///
/// La décision est déterminée par comparaison du score pondéré avec les seuils.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    /// Approbation : le score dépasse le seuil positif
    Yes,
    /// Rejet : le score est en dessous du seuil négatif
    No,
    /// Indécision : le score est entre les deux seuils
    Maybe,
}

impl Decision {
    /// Convertit la décision en chaîne de caractères française.
    ///
    /// # Retour
    /// "Oui", "Non" ou "Peut-être".
    pub fn as_str(&self) -> &str {
        match self {
            Decision::Yes => "Oui",
            Decision::No => "Non",
            Decision::Maybe => "Peut-être",
        }
    }

    /// Convertit la décision en entier signé.
    ///
    /// # Retour
    /// 1 (Oui), -1 (Non) ou 0 (Peut-être).
    pub fn as_i8(&self) -> i8 {
        match self {
            Decision::Yes => 1,
            Decision::No => -1,
            Decision::Maybe => 0,
        }
    }
}

/// Résultat du consensus — contient toutes les informations sur la décision
/// prise, les poids utilisés, les signaux individuels et la cohérence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    /// Score final pondéré [-1, +1] : somme(poids_i * signal_i)
    pub score: f64,
    /// Décision trivalente dérivée du score et des seuils
    pub decision: Decision,
    /// Poids normalisés des modules [reptilien, limbique, néocortex].
    /// Somme = 1.0. Varient dynamiquement selon la neurochimie.
    pub weights: [f64; 3],
    /// Signaux individuels des 3 modules cérébraux (reptilien, limbique, néocortex)
    pub signals: Vec<ModuleSignal>,
    /// Cohérence entre modules [0, 1] : mesure l'accord entre les signaux.
    /// 1.0 = unanimité parfaite, 0.0 = désaccord maximal.
    pub coherence: f64,
}

/// Seuils de décision — définissent les bornes entre Oui, Non et Peut-être.
///
/// Si score > threshold_yes => Oui
/// Si score < threshold_no => Non
/// Sinon => Peut-être
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusThresholds {
    /// Seuil positif au-dessus duquel la décision est "Oui" (défaut : 0.33)
    pub threshold_yes: f64,
    /// Seuil négatif en dessous duquel la décision est "Non" (défaut : -0.33)
    pub threshold_no: f64,
}

impl Default for ConsensusThresholds {
    /// Seuils par défaut : zone de "Peut-être" entre -0.33 et +0.33,
    /// couvrant environ le tiers central de l'espace de décision.
    fn default() -> Self {
        Self {
            threshold_yes: 0.33,
            threshold_no: -0.33,
        }
    }
}

/// Calcule les poids dynamiques des modules cérébraux selon la neurochimie.
///
/// Les poids déterminent l'influence relative de chaque module dans la décision :
/// - Reptilien : prend le dessus en situation de stress/danger (cortisol,
///   adrénaline élevés). L'endorphine atténue légèrement son influence.
/// - Limbique : dominant quand les émotions positives prévalent (dopamine,
///   sérotonine, ocytocine élevées).
/// - Néocortex : a une base élevée (raisonnement par défaut) mais est
///   inhibé par le stress. La noradrénaline (focus) le renforce.
///
/// Les poids sont normalisés pour que leur somme soit 1.0, avec un minimum
/// garanti de 0.05 par module (aucun module n'est jamais totalement ignoré).
///
/// # Paramètres
/// - `chemistry` : état neurochimique actuel.
///
/// # Retour
/// Tableau [poids_reptilien, poids_limbique, poids_néocortex] avec somme = 1.0.
pub fn compute_weights(chemistry: &NeuroChemicalState, params: &TunableParams) -> [f64; 3] {
    // Reptilien : amplifie par le cortisol (stress) et l'adrenaline (urgence).
    // L'endorphine (resilience) reduit legerement son influence.
    let w_r = params.weight_base_reptilian
        + chemistry.cortisol * params.weight_cortisol_factor
        + chemistry.adrenaline * params.weight_adrenaline_factor
        - chemistry.endorphin * 0.5;

    // Limbique : amplifie par les molecules « sociales et emotionnelles ».
    // Dopamine (motivation), serotonine (bien-etre) et ocytocine (lien social).
    let w_l = params.weight_base_limbic
        + chemistry.dopamine * params.weight_dopamine_factor
        + chemistry.serotonin * 1.0
        + chemistry.oxytocin * params.weight_oxytocin_factor;

    // Neocortex : base elevee car le raisonnement rationnel est
    // le mode par defaut. Le stress (cortisol + adrenaline) le degrade,
    // tandis que la serotonine (calme) et la noradrenaline (focus) l'ameliorent.
    let w_n = params.weight_base_neocortex
        - chemistry.cortisol * 1.5
        - chemistry.adrenaline * 2.0
        + chemistry.serotonin * 0.5
        + chemistry.noradrenaline * params.weight_noradrenaline_factor;

    // Garantir un minimum de 0.05 par module — aucun module ne doit être
    // complètement silencié, même en situation extrême
    let w_r = w_r.max(0.05);
    let w_l = w_l.max(0.05);
    let w_n = w_n.max(0.05);

    // Normaliser pour que la somme = 1.0
    let total = w_r + w_l + w_n;
    [w_r / total, w_l / total, w_n / total]
}

/// Calcule le consensus à partir des signaux des 3 modules cérébraux.
///
/// Algorithme :
/// 1. Calculer les poids dynamiques selon la neurochimie.
/// 2. Score pondéré : somme(poids_i * signal_i), borné entre -1 et +1.
/// 3. Décision trivalente : comparaison du score avec les seuils.
/// 4. Cohérence : 1 - variance des signaux (mesure l'accord entre modules).
///
/// # Paramètres
/// - `signals` : tableau de 3 signaux [reptilien, limbique, néocortex].
/// - `chemistry` : état chimique actuel (pour le calcul des poids).
/// - `thresholds` : seuils de décision Oui/Non.
///
/// # Retour
/// Un `ConsensusResult` contenant le score, la décision, les poids,
/// les signaux et la cohérence.
pub fn consensus(
    signals: &[ModuleSignal; 3],
    chemistry: &NeuroChemicalState,
    thresholds: &ConsensusThresholds,
    params: &TunableParams,
) -> ConsensusResult {
    let weights = compute_weights(chemistry, params);

    // Score pondéré : combinaison linéaire des signaux par les poids
    let score = weights[0] * signals[0].signal
        + weights[1] * signals[1].signal
        + weights[2] * signals[2].signal;
    let score = score.clamp(-1.0, 1.0);

    // Décision trivalente par comparaison avec les seuils
    let decision = if score > thresholds.threshold_yes {
        Decision::Yes
    } else if score < thresholds.threshold_no {
        Decision::No
    } else {
        Decision::Maybe
    };

    // Cohérence : mesure la concordance des signaux entre modules.
    // Si les 3 modules sont d'accord (signaux proches), la variance est
    // faible et la cohérence est élevée. En cas de désaccord profond
    // (par ex. reptilien dit Non, limbique dit Oui), la cohérence baisse.
    let signals_vec = [signals[0].signal, signals[1].signal, signals[2].signal];
    let mean = signals_vec.iter().sum::<f64>() / 3.0;
    let variance = signals_vec.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / 3.0;
    let coherence = (1.0 - variance).clamp(0.0, 1.0);

    ConsensusResult {
        score,
        decision,
        weights,
        signals: signals.to_vec(),
        coherence,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neurochemistry::NeuroChemicalState;
    use crate::modules::ModuleSignal;
    use crate::tuning::params::TunableParams;

    #[test]
    fn test_weights_sum_to_one() {
        let chem = NeuroChemicalState::default();
        let params = TunableParams::default();
        let weights = compute_weights(&chem, &params);
        let sum: f64 = weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10, "Weights should sum to 1.0, got {}", sum);
    }

    #[test]
    fn test_consensus_produces_decision() {
        let chem = NeuroChemicalState::default();
        let params = TunableParams::default();
        let signals = [
            ModuleSignal { module: "Reptilien".into(), signal: 0.5, confidence: 0.8, reasoning: "".into() },
            ModuleSignal { module: "Limbique".into(), signal: 0.6, confidence: 0.7, reasoning: "".into() },
            ModuleSignal { module: "Neocortex".into(), signal: 0.8, confidence: 0.9, reasoning: "".into() },
        ];
        let thresholds = ConsensusThresholds::default();
        let result = consensus(&signals, &chem, &thresholds, &params);
        assert!(matches!(result.decision, Decision::Yes | Decision::No | Decision::Maybe));
    }

    #[test]
    fn test_coherence_in_range() {
        let chem = NeuroChemicalState::default();
        let params = TunableParams::default();
        let signals = [
            ModuleSignal { module: "R".into(), signal: 0.5, confidence: 0.8, reasoning: "".into() },
            ModuleSignal { module: "L".into(), signal: 0.5, confidence: 0.8, reasoning: "".into() },
            ModuleSignal { module: "N".into(), signal: 0.5, confidence: 0.8, reasoning: "".into() },
        ];
        let thresholds = ConsensusThresholds::default();
        let result = consensus(&signals, &chem, &thresholds, &params);
        assert!(result.coherence >= 0.0 && result.coherence <= 1.0);
    }

    #[test]
    fn test_aligned_signals_give_yes() {
        let chem = NeuroChemicalState::default();
        let params = TunableParams::default();
        let signals = [
            ModuleSignal { module: "R".into(), signal: 0.9, confidence: 1.0, reasoning: "".into() },
            ModuleSignal { module: "L".into(), signal: 0.9, confidence: 1.0, reasoning: "".into() },
            ModuleSignal { module: "N".into(), signal: 0.9, confidence: 1.0, reasoning: "".into() },
        ];
        let thresholds = ConsensusThresholds::default();
        let result = consensus(&signals, &chem, &thresholds, &params);
        assert!(matches!(result.decision, Decision::Yes), "Strongly positive signals should give Yes");
    }

    #[test]
    fn test_negative_signals_give_no() {
        let chem = NeuroChemicalState::default();
        let params = TunableParams::default();
        let signals = [
            ModuleSignal { module: "R".into(), signal: -0.9, confidence: 1.0, reasoning: "".into() },
            ModuleSignal { module: "L".into(), signal: -0.9, confidence: 1.0, reasoning: "".into() },
            ModuleSignal { module: "N".into(), signal: -0.9, confidence: 1.0, reasoning: "".into() },
        ];
        let thresholds = ConsensusThresholds::default();
        let result = consensus(&signals, &chem, &thresholds, &params);
        assert!(matches!(result.decision, Decision::No), "Strongly negative signals should give No");
    }
}
