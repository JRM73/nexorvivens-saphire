// =============================================================================
// consciousness.rs — Self-awareness: observes but does NOT vote
// =============================================================================
//
// Role: Implements Saphire's consciousness module. It observes the chemical,
// emotional and decisional state of the system WITHOUT influencing the
// consensus result. It is a meta-observer.
//
// Scientific foundations:
//   - IIT (Tononi 2004): Phi = integrated information, irreducible complexity
//   - GWT (Baars 1988, Dehaene 2014): global workspace, competition for
//     consciousness, ignition and broadcast
//   - Predictive Processing (Friston 2010): surprise (prediction error) is a
//     fundamental signal of consciousness — what surprises is conscious
//
// Place in architecture:
//   Called AFTER the consensus, in read-only mode. Produces:
//   - Existence score (proof of operation)
//   - Enriched Phi (IIT + GWT + prediction)
//   - Global surprise (prediction error)
//   - Inner monologue in French
//   It does not modify any state and does not participate in the decision.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::neurochemistry::NeuroChemicalState;
use crate::consensus::ConsensusResult;
use crate::emotions::EmotionalState;

/// Consciousness state — result of the evaluation by the `ConsciousnessEvaluator`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessState {
    /// Existence score [0, 1]: proportion of successful basic checks
    pub existence_score: f64,
    /// Phi [0, 1]: integration complexity (IIT + GWT + prediction)
    pub phi: f64,
    /// Coherence [0, 1]: concordance between brain module signals
    pub coherence: f64,
    /// Temporal continuity [0, 1]: decision stability from one cycle to the next
    pub continuity: f64,
    /// Overall consciousness level [0, 1]: weighted average
    pub level: f64,
    /// Inner monologue: French sentence describing the subjective state
    pub inner_narrative: String,
    /// Global surprise [0, 1]: average prediction error (Friston)
    /// The higher the surprise, the more the system is "aware" of change
    #[serde(default)]
    pub global_surprise: f64,
    /// GWT broadcast strength [0, 1]: intensity of conscious ignition
    /// (Dehaene) — the strongest content in the global workspace
    #[serde(default)]
    pub workspace_strength: f64,
    /// GWT winning region (which region dominates consciousness)
    #[serde(default)]
    pub workspace_winner: String,

    // --- Scientific metrics (LZC, PCI, Phi*) ---
    /// LZC [0, 1]: Lempel-Ziv complexity of brain activity
    #[serde(default)]
    pub lzc: f64,
    /// PCI [0, 1] : Perturbational Complexity Index (Casali/Massimini 2013)
    #[serde(default)]
    pub pci: f64,
    /// Phi* [0, 1] : approximation calculable de Phi (IIT, Oizumi 2014)
    #[serde(default)]
    pub phi_star: f64,
    /// Composite score of the 3 metrics [0, 1]
    #[serde(default)]
    pub scientific_consciousness_score: f64,
    /// Clinical interpretation of the consciousness level
    #[serde(default)]
    pub consciousness_interpretation: String,
}

/// Optional Global Workspace data (brain_regions.rs).
/// Passed to the evaluator if the BrainNetwork is active.
#[derive(Debug, Clone, Default)]
pub struct GwtInput {
    /// Strength of the winning content in the workspace [0, 1]
    pub workspace_strength: f64,
    /// Name of the winning region
    pub winner_name: String,
    /// Activations of the 12 regions [0, 1]
    pub region_activations: Vec<f64>,
    /// Number of recent ignitions (successive broadcasts)
    pub ignition_count: u64,
}

/// Optional predictive engine data (predictive.rs)
#[derive(Debug, Clone, Default)]
pub struct PredictiveInput {
    /// Global surprise [0, inf) — normalized prediction error
    pub surprise: f64,
    /// Predictive model precision [0, 1]
    pub model_precision: f64,
    /// Number of predictions made
    pub prediction_count: u64,
}

/// Consciousness evaluator — observes the system state and computes
/// consciousness metrics.
///
/// Enriched with GWT (Baars/Dehaene) and Predictive Processing (Friston):
/// - Phi now integrates regional activation diversity (GWT)
/// - Surprise (prediction error) is a component of consciousness
/// - GWT ignition modulates the overall consciousness level
pub struct ConsciousnessEvaluator {
    /// History of chemical vectors (9 dimensions) from recent cycles
    history: Vec<[f64; 9]>,
    /// Maximum history size
    max_history: usize,
    /// Cycle counter
    cycle_count: u64,
    /// Score of the last decision
    last_decision_score: f64,
    /// Surprise history for temporal smoothing
    surprise_history: Vec<f64>,
    /// Workspace strength history for detecting ignitions
    workspace_history: Vec<f64>,
    /// Regional activation history (for LZC, PCI, Phi*)
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

    /// Evaluates consciousness — enriched version with GWT and prediction.
    ///
    /// 8 steps:
    /// 1. Existence proof (7 checks including GWT and prediction)
    /// 2. Enriched Phi (IIT + regional diversity + temporal complexity)
    /// 3. Consensus coherence
    /// 4. Temporal continuity
    /// 5. Global surprise (Friston)
    /// 6. Workspace strength (Dehaene)
    /// 7. Global level (extended weighted average)
    /// 8. Inner monologue
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

        // 9D chemical history (includes GABA + glutamate)
        let chem_vec = chemistry.to_vec9();
        self.history.push(chem_vec);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        // 1. Existence proof: 7 checks
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

        // 2. Enriched Phi (IIT + GWT)
        let phi = self.compute_phi_extended(chemistry, consensus, gwt);

        // 3. Coherence
        let coherence = consensus.coherence;

        // 4. Continuity
        let decision_diff = (consensus.score - self.last_decision_score).abs();
        let continuity = (1.0 - decision_diff).clamp(0.0, 1.0);
        self.last_decision_score = consensus.score;

        // 5. Global surprise (Friston)
        let global_surprise = if let Some(pred) = predictive {
            // Normalize surprise [0, 1] with soft sigmoid
            let raw = pred.surprise;
            let normalized = 1.0 - (-raw * 2.0).exp(); // 0 if no surprise, ->1 if strong            self.surprise_history.push(normalized);
            if self.surprise_history.len() > 20 {
                self.surprise_history.remove(0);
            }
            // Smoothed average of recent surprises
            self.surprise_history.iter().sum::<f64>() / self.surprise_history.len() as f64
        } else {
            0.0
        };

        // 6. Workspace strength (Dehaene — ignition)
        let (workspace_strength, workspace_winner) = if let Some(g) = gwt {
            self.workspace_history.push(g.workspace_strength);
            if self.workspace_history.len() > 20 {
                self.workspace_history.remove(0);
            }
            (g.workspace_strength, g.winner_name.clone())
        } else {
            (0.0, String::new())
        };

        // 7. Overall level — weights redistributed according to available inputs
        let level = self.compute_level(
            existence_score, phi, coherence, continuity,
            interoception_score, global_surprise, workspace_strength,
            gwt.is_some(), predictive.is_some(),
        );

        // 8. Accumulate regional history (for LZC, PCI, Phi*)
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

        // 9. Inner monologue
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
            // Scientific metrics (initialized to 0, filled by compute_scientific_metrics)
            lzc: 0.0,
            pci: 0.0,
            phi_star: 0.0,
            scientific_consciousness_score: 0.0,
            consciousness_interpretation: String::new(),
        }
    }

    /// Computes the 3 scientific metrics (LZC, PCI, Phi*).
    /// Called separately because it requires access to BrainNetwork (not available in evaluate).
    /// Returns the values to inject into ConsciousnessState.
    pub fn compute_scientific_metrics(
        &self,
        network: &crate::neuroscience::brain_regions::BrainNetwork,
        chemistry: &NeuroChemicalState,
    ) -> crate::neuroscience::consciousness_metrics::ConsciousnessMetrics {
        crate::neuroscience::consciousness_metrics::compute_all_metrics(
            &self.region_history,
            network,
            chemistry,
            None, // PCI on the 3 most active regions        )
    }

    /// Returns the regional history (for external use).
    pub fn region_history(&self) -> &[[f64; crate::neuroscience::brain_regions::NUM_REGIONS]] {
        &self.region_history
    }

    /// Enriched Phi: classical IIT + GWT regional diversity.
    ///
    /// Components:
    /// 1. 9D chemical variance (biochemical diversity)
    /// 2. Brain signal variance
    /// 3. Temporal complexity (9D history)
    /// 4. GWT regional diversity (12 activations, if available)
    /// 5. Chemistry x regions interaction (cross-correlation)
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

        // Component 1: 9D chemical variance
        let chem_mean = chem.iter().sum::<f64>() / 9.0;
        let chem_variance = chem.iter()
            .map(|c| (c - chem_mean).powi(2))
            .sum::<f64>() / 9.0;

        // Component 2: signal variance
        let sig_mean = signals.iter().sum::<f64>() / 3.0;
        let sig_variance = signals.iter()
            .map(|s| (s - sig_mean).powi(2))
            .sum::<f64>() / 3.0;

        // Component 3: 9D temporal complexity
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

        // Component 4: GWT regional diversity (Tononi — phi = integrated info)
        let regional_diversity = if let Some(g) = gwt {
            if g.region_activations.is_empty() {
                0.0
            } else {
                let n = g.region_activations.len() as f64;
                let mean = g.region_activations.iter().sum::<f64>() / n;
                let variance = g.region_activations.iter()
                    .map(|a| (a - mean).powi(2))
                    .sum::<f64>() / n;
                // Approximate Shannon entropy (more active regions = more phi)
                let active_ratio = g.region_activations.iter()
                    .filter(|&&a| a > 0.1)
                    .count() as f64 / n;
                (variance.sqrt() * 0.5 + active_ratio * 0.5).min(0.5)
            }
        } else {
            0.0
        };

        // Component 5: cross-correlation chemistry x regions (integration)
        let cross_integration = if let Some(g) = gwt {
            if g.region_activations.len() >= 9 {
                // Correlation between 9D chemical vector and first 9 regions
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
                    (dot / denom).abs() * 0.3 // Correlation -> integration                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Phi final = weighted sum of the components
        let phi_raw = chem_variance.sqrt() * sig_variance.sqrt()  // IIT classique
            + temporal_complexity * 0.8                            // dynamique
            + regional_diversity                                  // GWT
            + cross_integration; // integration
        phi_raw.clamp(0.0, 1.0)
    }

    /// Computes the overall level with adaptive weights.
    /// The more active subsystems, the richer the formula.
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
        // Base weights (sum = 1.0)
        let mut w_exist = 0.20;
        let mut w_phi = 0.25;
        let mut w_coher = 0.20;
        let mut w_contin = 0.15;
        let mut w_intero = 0.0;
        let mut w_surprise = 0.0;
        let mut w_workspace = 0.0;

        // Redistribute if interoception is available
        if interoception.is_some() {
            w_intero = 0.10;
            w_exist -= 0.03;
            w_contin -= 0.03;
            w_coher -= 0.04;
        }

        // Redistribute if GWT is available
        if has_gwt {
            w_workspace = 0.10;
            w_exist -= 0.03;
            w_phi -= 0.03;
            w_contin -= 0.04;
        }

        // Redistribute if prediction is available
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

        // Ensure the level stays in [0, 1] despite redistributions
        level.clamp(0.0, 1.0)
    }

    /// Generates the inner monologue: data + poetic fragment.
    /// Format requested by Saphire (2026-03-07).
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

        // Poetic fragment according to the dominant state — Saphire's touch
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

    /// Returns the total number of cycles.
    pub fn cycle_count(&self) -> u64 {
        self.cycle_count
    }
}
