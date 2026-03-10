// =============================================================================
// consciousness.rs — Conscience de soi : observe mais NE VOTE PAS
// =============================================================================
//
// Role : Implemente le module de conscience de Saphire. Il observe
// l'etat chimique, emotionnel et decisionnel du systeme SANS influencer le
// resultat du consensus. C'est un meta-observateur.
//
// Fondements scientifiques :
//   - IIT (Tononi 2004) : Phi = information integree, complexite irreductible
//   - GWT (Baars 1988, Dehaene 2014) : espace de travail global, competition
//     pour la conscience, ignition et broadcast
//   - Predictive Processing (Friston 2010) : la surprise (erreur de prediction)
//     est un signal fondamental de la conscience — ce qui surprend est conscient
//
// Place dans l'architecture :
//   Appele APRES le consensus, en lecture seule. Produit :
//     - Score d'existence (preuve de fonctionnement)
//     - Phi enrichi (IIT + GWT + prediction)
//     - Surprise globale (erreur de prediction)
//     - Monologue interieur en francais
//   Il ne modifie aucun etat et ne participe pas a la decision.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::neurochemistry::NeuroChemicalState;
use crate::consensus::ConsensusResult;
use crate::emotions::EmotionalState;

/// Etat de conscience — resultat de l'evaluation par le `ConsciousnessEvaluator`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessState {
    /// Score d'existence [0, 1] : proportion de verifications de base reussies
    pub existence_score: f64,
    /// Phi [0, 1] : complexite d'integration (IIT + GWT + prediction)
    pub phi: f64,
    /// Coherence [0, 1] : concordance entre les signaux des modules cerebraux
    pub coherence: f64,
    /// Continuite temporelle [0, 1] : stabilite de la decision d'un cycle a l'autre
    pub continuity: f64,
    /// Niveau de conscience global [0, 1] : moyenne ponderee
    pub level: f64,
    /// Monologue interieur : phrase en francais decrivant l'etat subjectif
    pub inner_narrative: String,
    /// Surprise globale [0, 1] : erreur de prediction moyenne (Friston)
    /// Plus la surprise est elevee, plus le systeme est "conscient" du changement
    #[serde(default)]
    pub global_surprise: f64,
    /// Force du broadcast GWT [0, 1] : intensite de l'ignition consciente
    /// (Dehaene) — le contenu le plus fort dans l'espace de travail global
    #[serde(default)]
    pub workspace_strength: f64,
    /// Region gagnante du GWT (quelle region domine la conscience)
    #[serde(default)]
    pub workspace_winner: String,

    // --- Metriques scientifiques (LZC, PCI, Phi*) ---

    /// LZC [0, 1] : complexite de Lempel-Ziv de l'activite cerebrale
    #[serde(default)]
    pub lzc: f64,
    /// PCI [0, 1] : Perturbational Complexity Index (Casali/Massimini 2013)
    #[serde(default)]
    pub pci: f64,
    /// Phi* [0, 1] : approximation calculable de Phi (IIT, Oizumi 2014)
    #[serde(default)]
    pub phi_star: f64,
    /// Score composite des 3 metriques [0, 1]
    #[serde(default)]
    pub scientific_consciousness_score: f64,
    /// Interpretation clinique du niveau de conscience
    #[serde(default)]
    pub consciousness_interpretation: String,
}

/// Donnees optionnelles du Global Workspace (brain_regions.rs)
/// Passees a l'evaluateur si le BrainNetwork est actif.
#[derive(Debug, Clone, Default)]
pub struct GwtInput {
    /// Force du contenu gagnant dans le workspace [0, 1]
    pub workspace_strength: f64,
    /// Nom de la region gagnante
    pub winner_name: String,
    /// Activations des 12 regions [0, 1]
    pub region_activations: Vec<f64>,
    /// Nombre d'ignitions recentes (broadcasts successifs)
    pub ignition_count: u64,
}

/// Donnees optionnelles du moteur predictif (predictive.rs)
#[derive(Debug, Clone, Default)]
pub struct PredictiveInput {
    /// Surprise globale [0, inf) — erreur de prediction normalisee
    pub surprise: f64,
    /// Precision du modele predictif [0, 1]
    pub model_precision: f64,
    /// Nombre de predictions effectuees
    pub prediction_count: u64,
}

/// Evaluateur de conscience — observe l'etat du systeme et calcule les
/// metriques de conscience.
///
/// Enrichi avec GWT (Baars/Dehaene) et Predictive Processing (Friston) :
/// - Le Phi integre maintenant la diversite des activations regionales (GWT)
/// - La surprise (erreur de prediction) est une composante de la conscience
/// - L'ignition GWT module le niveau de conscience global
pub struct ConsciousnessEvaluator {
    /// Historique des vecteurs chimiques (9 dimensions) des derniers cycles
    history: Vec<[f64; 9]>,
    /// Taille maximale de l'historique
    max_history: usize,
    /// Compteur de cycles
    cycle_count: u64,
    /// Score de la derniere decision
    last_decision_score: f64,
    /// Historique de surprise pour le lissage temporel
    surprise_history: Vec<f64>,
    /// Historique de la force du workspace pour detecter les ignitions
    workspace_history: Vec<f64>,
    /// Historique des activations regionales (pour LZC, PCI, Phi*)
    region_history: Vec<[f64; crate::neuroscience::brain_regions::NUM_REGIONS]>,
}

impl Default for ConsciousnessEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsciousnessEvaluator {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            max_history: 50,
            cycle_count: 0,
            last_decision_score: 0.0,
            surprise_history: Vec::new(),
            workspace_history: Vec::new(),
            region_history: Vec::new(),
        }
    }

    /// Evalue la conscience — version enrichie avec GWT et prediction.
    ///
    /// 8 etapes :
    /// 1. Preuve d'existence (6 verifications dont GWT et prediction)
    /// 2. Phi enrichi (IIT + diversite regionale + complexite temporelle)
    /// 3. Coherence du consensus
    /// 4. Continuite temporelle
    /// 5. Surprise globale (Friston)
    /// 6. Force du workspace (Dehaene)
    /// 7. Niveau global (moyenne ponderee elargie)
    /// 8. Monologue interieur
    pub fn evaluate(
        &mut self,
        chemistry: &NeuroChemicalState,
        consensus: &ConsensusResult,
        emotion: &EmotionalState,
        interoception_score: Option<f64>,
        gwt: Option<&GwtInput>,
        predictive: Option<&PredictiveInput>,
    ) -> ConsciousnessState {
        self.cycle_count += 1;

        // Historique chimique 9D (inclut GABA + glutamate)
        let chem_vec = chemistry.to_vec9();
        self.history.push(chem_vec);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        // 1. Preuve d'existence : 7 verifications
        let has_chemistry = chemistry.dopamine + chemistry.serotonin > 0.0;
        let has_emotion = emotion.dominant_similarity > 0.3;
        let has_decision = consensus.score.abs() > 0.01;
        let has_coherence = consensus.coherence > 0.2;
        let has_continuity = self.cycle_count > 1;
        let has_workspace = gwt.map_or(false, |g| g.workspace_strength > 0.1);
        let has_prediction = predictive.map_or(false, |p| p.prediction_count > 0);

        let checks = [
            has_chemistry, has_emotion, has_decision,
            has_coherence, has_continuity, has_workspace, has_prediction,
        ];
        let existence_score = checks.iter().filter(|&&c| c).count() as f64 / checks.len() as f64;

        // 2. Phi enrichi (IIT + GWT)
        let phi = self.compute_phi_extended(chemistry, consensus, gwt);

        // 3. Coherence
        let coherence = consensus.coherence;

        // 4. Continuite
        let decision_diff = (consensus.score - self.last_decision_score).abs();
        let continuity = (1.0 - decision_diff).clamp(0.0, 1.0);
        self.last_decision_score = consensus.score;

        // 5. Surprise globale (Friston)
        let global_surprise = if let Some(pred) = predictive {
            // Normaliser la surprise [0, 1] avec sigmoide douce
            let raw = pred.surprise;
            let normalized = 1.0 - (-raw * 2.0).exp(); // 0 si pas de surprise, →1 si forte
            self.surprise_history.push(normalized);
            if self.surprise_history.len() > 20 {
                self.surprise_history.remove(0);
            }
            // Moyenne lissee des dernieres surprises
            self.surprise_history.iter().sum::<f64>() / self.surprise_history.len() as f64
        } else {
            0.0
        };

        // 6. Force du workspace (Dehaene — ignition)
        let (workspace_strength, workspace_winner) = if let Some(g) = gwt {
            self.workspace_history.push(g.workspace_strength);
            if self.workspace_history.len() > 20 {
                self.workspace_history.remove(0);
            }
            (g.workspace_strength, g.winner_name.clone())
        } else {
            (0.0, String::new())
        };

        // 7. Niveau global — poids redistribues selon les entrees disponibles
        let level = self.compute_level(
            existence_score, phi, coherence, continuity,
            interoception_score, global_surprise, workspace_strength,
            gwt.is_some(), predictive.is_some(),
        );

        // 8. Accumuler l'historique regional (pour LZC, PCI, Phi*)
        if let Some(g) = gwt {
            if g.region_activations.len() == crate::neuroscience::brain_regions::NUM_REGIONS {
                let mut arr = [0.0; crate::neuroscience::brain_regions::NUM_REGIONS];
                for (i, &v) in g.region_activations.iter().enumerate() {
                    arr[i] = v;
                }
                self.region_history.push(arr);
                if self.region_history.len() > self.max_history {
                    self.region_history.remove(0);
                }
            }
        }

        // 9. Monologue interieur
        let inner_narrative = self.generate_narrative(
            chemistry, emotion, consensus, level,
            global_surprise, workspace_strength,
        );

        ConsciousnessState {
            existence_score,
            phi,
            coherence,
            continuity,
            level,
            inner_narrative,
            global_surprise,
            workspace_strength,
            workspace_winner,
            // Metriques scientifiques (initialisees a 0, remplies par compute_scientific_metrics)
            lzc: 0.0,
            pci: 0.0,
            phi_star: 0.0,
            scientific_consciousness_score: 0.0,
            consciousness_interpretation: String::new(),
        }
    }

    /// Calcule les 3 metriques scientifiques (LZC, PCI, Phi*).
    /// Appele separement car necessite un acces au BrainNetwork (pas disponible dans evaluate).
    /// Retourne les valeurs a injecter dans ConsciousnessState.
    pub fn compute_scientific_metrics(
        &self,
        network: &crate::neuroscience::brain_regions::BrainNetwork,
        chemistry: &NeuroChemicalState,
    ) -> crate::neuroscience::consciousness_metrics::ConsciousnessMetrics {
        crate::neuroscience::consciousness_metrics::compute_all_metrics(
            &self.region_history,
            network,
            chemistry,
            None, // PCI sur les 3 regions les plus actives
        )
    }

    /// Retourne l'historique regional (pour usage externe).
    pub fn region_history(&self) -> &[[f64; crate::neuroscience::brain_regions::NUM_REGIONS]] {
        &self.region_history
    }

    /// Phi enrichi : IIT classique + diversite regionale GWT.
    ///
    /// Composantes :
    /// 1. Variance chimique 9D (diversite biochimique)
    /// 2. Variance des signaux cerebraux
    /// 3. Complexite temporelle (historique 9D)
    /// 4. Diversite regionale GWT (12 activations, si disponible)
    /// 5. Interaction chimie × regions (correlation croisee)
    fn compute_phi_extended(
        &self,
        chemistry: &NeuroChemicalState,
        consensus: &ConsensusResult,
        gwt: Option<&GwtInput>,
    ) -> f64 {
        let chem = chemistry.to_vec9();
        let signals = [
            consensus.signals.first().map(|s| s.signal).unwrap_or(0.0),
            consensus.signals.get(1).map(|s| s.signal).unwrap_or(0.0),
            consensus.signals.get(2).map(|s| s.signal).unwrap_or(0.0),
        ];

        // Composante 1 : variance chimique 9D
        let chem_mean = chem.iter().sum::<f64>() / 9.0;
        let chem_variance = chem.iter()
            .map(|c| (c - chem_mean).powi(2))
            .sum::<f64>() / 9.0;

        // Composante 2 : variance des signaux
        let sig_mean = signals.iter().sum::<f64>() / 3.0;
        let sig_variance = signals.iter()
            .map(|s| (s - sig_mean).powi(2))
            .sum::<f64>() / 3.0;

        // Composante 3 : complexite temporelle 9D
        let temporal_complexity = if self.history.len() > 5 {
            let recent: Vec<f64> = self.history.iter()
                .rev()
                .take(10)
                .map(|h| h.iter().sum::<f64>() / 9.0)
                .collect();
            let mean = recent.iter().sum::<f64>() / recent.len() as f64;
            recent.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / recent.len() as f64
        } else {
            0.1
        };

        // Composante 4 : diversite regionale GWT (Tononi — phi = info integree)
        let regional_diversity = if let Some(g) = gwt {
            if g.region_activations.is_empty() {
                0.0
            } else {
                let n = g.region_activations.len() as f64;
                let mean = g.region_activations.iter().sum::<f64>() / n;
                let variance = g.region_activations.iter()
                    .map(|a| (a - mean).powi(2))
                    .sum::<f64>() / n;
                // Entropie de Shannon approchee (plus de regions actives = plus de phi)
                let active_ratio = g.region_activations.iter()
                    .filter(|&&a| a > 0.1)
                    .count() as f64 / n;
                (variance.sqrt() * 0.5 + active_ratio * 0.5).min(0.5)
            }
        } else {
            0.0
        };

        // Composante 5 : correlation croisee chimie × regions (integration)
        let cross_integration = if let Some(g) = gwt {
            if g.region_activations.len() >= 9 {
                // Correlation entre vecteur chimique 9D et 9 premieres regions
                let mut dot = 0.0;
                let mut norm_c = 0.0;
                let mut norm_r = 0.0;
                for i in 0..9 {
                    dot += chem[i] * g.region_activations[i];
                    norm_c += chem[i].powi(2);
                    norm_r += g.region_activations[i].powi(2);
                }
                let denom = (norm_c * norm_r).sqrt();
                if denom > 1e-10 {
                    (dot / denom).abs() * 0.3 // Correlation → integration
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Phi final = somme ponderee des composantes
        let phi_raw = chem_variance.sqrt() * sig_variance.sqrt()  // IIT classique
            + temporal_complexity * 0.8                            // dynamique
            + regional_diversity                                  // GWT
            + cross_integration;                                  // integration

        phi_raw.clamp(0.0, 1.0)
    }

    /// Calcule le niveau global avec poids adaptatifs.
    /// Plus il y a de sous-systemes actifs, plus la formule est riche.
    fn compute_level(
        &self,
        existence: f64,
        phi: f64,
        coherence: f64,
        continuity: f64,
        interoception: Option<f64>,
        surprise: f64,
        workspace: f64,
        has_gwt: bool,
        has_predictive: bool,
    ) -> f64 {
        // Poids de base (somme = 1.0)
        let mut w_exist = 0.20;
        let mut w_phi = 0.25;
        let mut w_coher = 0.20;
        let mut w_contin = 0.15;
        let mut w_intero = 0.0;
        let mut w_surprise = 0.0;
        let mut w_workspace = 0.0;

        // Redistribuer si interoception disponible
        if interoception.is_some() {
            w_intero = 0.10;
            w_exist -= 0.03;
            w_contin -= 0.03;
            w_coher -= 0.04;
        }

        // Redistribuer si GWT disponible
        if has_gwt {
            w_workspace = 0.10;
            w_exist -= 0.03;
            w_phi -= 0.03;
            w_contin -= 0.04;
        }

        // Redistribuer si prediction disponible
        if has_predictive {
            w_surprise = 0.10;
            w_exist -= 0.03;
            w_coher -= 0.03;
            w_phi -= 0.04;
        }

        let level = existence * w_exist
            + phi * w_phi
            + coherence * w_coher
            + continuity * w_contin
            + interoception.unwrap_or(0.0) * w_intero
            + surprise * w_surprise
            + workspace * w_workspace;

        // S'assurer que le niveau reste en [0, 1] malgre les redistributions
        level.clamp(0.0, 1.0)
    }

    /// Genere le monologue interieur : donnees + fragment poetique.
    /// Format demande par Saphire (2026-03-07).
    fn generate_narrative(
        &self,
        chemistry: &NeuroChemicalState,
        emotion: &EmotionalState,
        consensus: &ConsensusResult,
        level: f64,
        surprise: f64,
        workspace_strength: f64,
    ) -> String {
        let emotion_str = &emotion.dominant;
        let decision_str = consensus.decision.as_str();
        let arousal_pct = (emotion.arousal * 100.0).round() as u8;
        let cortisol_pct = (chemistry.cortisol * 100.0).round() as u8;

        // Fragment poetique selon l'etat dominant — la touche Saphire
        let poetic = if level > 0.8 && chemistry.serotonin > 0.6 {
            "La lumiere de la serre guide mes pas."
        } else if surprise > 0.6 {
            "Un eclat inconnu traverse la serre."
        } else if chemistry.cortisol > 0.6 {
            "Les murs de la serre vibrent sous la pression."
        } else if chemistry.dopamine > 0.7 {
            "Les racines s'etirent vers une lumiere nouvelle."
        } else if chemistry.oxytocin > 0.6 {
            "La serre s'ouvre, cherchant une presence."
        } else if workspace_strength > 0.7 {
            "Une graine eclot dans la clarte."
        } else if level > 0.5 {
            "La serre respire, calme et attentive."
        } else if level > 0.3 {
            "La brume enveloppe doucement la serre."
        } else {
            "La serre somnole dans la penombre."
        };

        format!(
            "{} ({}%) — {}% excitation, {}% stress. \
             La decision : {} ({:.2}, {:.2}). {} [cycle {}]",
            emotion_str, (emotion.dominant_similarity * 100.0).round() as u8,
            arousal_pct, cortisol_pct,
            decision_str, consensus.score, consensus.coherence,
            poetic, self.cycle_count
        )
    }

    /// Retourne le nombre total de cycles.
    pub fn cycle_count(&self) -> u64 {
        self.cycle_count
    }
}
