// =============================================================================
// consciousness.rs — Self-awareness module: observes but DOES NOT VOTE
// =============================================================================
//
// Purpose: Implements Saphire's consciousness module. It observes the
// chemical, emotional, and decision-making state of the system WITHOUT
// influencing the consensus outcome. It acts as a meta-observer that
// generates introspective reports and consciousness metrics.
//
// Scientific foundations:
//   - IIT (Tononi 2004, 2008): Phi = integrated information, irreducible
//     complexity. Higher Phi indicates more conscious processing. Computed
//     here as a weighted combination of biochemical variance, signal variance,
//     temporal complexity, and regional diversity.
//   - GWT (Baars 1988, Dehaene et al. 2014): Global Workspace Theory.
//     Information becomes conscious when it wins the competition for access
//     to the global workspace and is broadcast (ignition) to all brain
//     regions. Modeled here via workspace_strength and the winner region.
//   - Predictive Processing (Friston 2010): The brain is a prediction
//     machine. Surprise (prediction error) is a fundamental signal of
//     consciousness — what surprises the system is what becomes conscious.
//   - LZC (Lempel-Ziv Complexity): algorithmic complexity measure of neural
//     activity patterns, used as a consciousness marker (Casali et al. 2013).
//   - PCI (Perturbational Complexity Index, Casali & Massimini 2013):
//     measures the complexity of the brain's response to perturbation.
//   - Phi* (Oizumi et al. 2014): a computationally tractable approximation
//     of Tononi's Phi from IIT 3.0.
//
// Architectural role:
//   Called AFTER the consensus, in read-only mode. Produces:
//     - Existence score (proof of functioning across subsystems)
//     - Enriched Phi (IIT + GWT + predictive processing)
//     - Global surprise (prediction error signal)
//     - Inner narrative (poetic monologue in French)
//   It does NOT modify any state and does NOT participate in decision-making.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::neurochemistry::NeuroChemicalState;
use crate::consensus::ConsensusResult;
use crate::emotions::EmotionalState;

/// Consciousness state — result of evaluation by the `ConsciousnessEvaluator`.
///
/// Contains all consciousness metrics computed during a single evaluation
/// cycle, including IIT-inspired Phi, GWT workspace dynamics, predictive
/// processing surprise, and scientific consciousness indices (LZC, PCI, Phi*).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessState {
    /// Existence score [0, 1]: proportion of basic verification checks that
    /// passed (proof that the system's subsystems are active and functioning)
    pub existence_score: f64,
    /// Phi [0, 1]: integration complexity metric inspired by IIT (Tononi 2004).
    /// Combines biochemical diversity, signal variance, temporal complexity,
    /// regional diversity (GWT), and cross-domain integration.
    pub phi: f64,
    /// Coherence [0, 1]: concordance between the signals of the brain modules.
    /// High coherence = modules agree; low coherence = internal conflict.
    pub coherence: f64,
    /// Temporal continuity [0, 1]: stability of the decision from one cycle
    /// to the next. Measures how much the decision score changes between
    /// consecutive cycles (1.0 = perfectly stable, 0.0 = maximal change).
    pub continuity: f64,
    /// Global consciousness level [0, 1]: weighted average of all sub-metrics,
    /// with weights that adapt based on which subsystems are active.
    pub level: f64,
    /// Inner narrative: a French-language sentence describing the subjective
    /// state, combining factual data with a poetic fragment.
    pub inner_narrative: String,
    /// Global surprise [0, 1]: smoothed prediction error (Friston 2010).
    /// Higher surprise = the system is more "aware" of unexpected changes.
    /// Normalized via a soft sigmoid: 1 - exp(-raw_surprise * 2).
    #[serde(default)]
    pub global_surprise: f64,
    /// GWT workspace broadcast strength [0, 1]: intensity of the conscious
    /// ignition (Dehaene et al. 2014) — how strongly the winning content
    /// dominates the global workspace.
    #[serde(default)]
    pub workspace_strength: f64,
    /// GWT workspace winner: name of the brain region that currently
    /// dominates conscious access in the global workspace.
    #[serde(default)]
    pub workspace_winner: String,

    // --- Scientific consciousness metrics (LZC, PCI, Phi*) ---

    /// LZC [0, 1]: Lempel-Ziv Complexity of the brain activity pattern.
    /// Higher values indicate more complex (less compressible) neural dynamics,
    /// which correlates with higher levels of consciousness.
    /// Reference: Casali et al. (2013).
    #[serde(default)]
    pub lzc: f64,
    /// PCI [0, 1]: Perturbational Complexity Index (Casali & Massimini 2013).
    /// Measures the complexity of the brain's response to perturbation.
    /// High PCI distinguishes conscious from unconscious states clinically.
    #[serde(default)]
    pub pci: f64,
    /// Phi* [0, 1]: computationally tractable approximation of Tononi's Phi
    /// from IIT 3.0 (Oizumi et al. 2014). Estimates the amount of integrated
    /// information in the system without the combinatorial explosion of full Phi.
    #[serde(default)]
    pub phi_star: f64,
    /// Composite score of the 3 scientific metrics [0, 1]: weighted combination
    /// of LZC, PCI, and Phi* for an overall scientific consciousness estimate.
    #[serde(default)]
    pub scientific_consciousness_score: f64,
    /// Clinical interpretation of the consciousness level, mapping the
    /// composite score to a qualitative description (e.g., "awake", "drowsy").
    #[serde(default)]
    pub consciousness_interpretation: String,
}

/// Optional data from the Global Workspace (brain_regions.rs).
///
/// Passed to the evaluator when the BrainNetwork is active. Contains
/// the workspace competition result and regional activation data needed
/// for GWT-based consciousness metrics.
#[derive(Debug, Clone, Default)]
pub struct GwtInput {
    /// Strength of the winning content in the global workspace [0, 1].
    /// Higher values indicate a more decisive ignition event.
    pub workspace_strength: f64,
    /// Name of the winning brain region (the region whose content
    /// currently dominates conscious access)
    pub winner_name: String,
    /// Activation levels of all 12 brain regions [0, 1] each.
    /// Used to compute regional diversity for the enriched Phi metric.
    pub region_activations: Vec<f64>,
    /// Number of recent ignitions (consecutive successful broadcasts).
    /// Sustained ignition indicates stable conscious access.
    pub ignition_count: u64,
}

/// Optional data from the predictive engine (predictive.rs).
///
/// Contains prediction error metrics used to compute the surprise
/// component of consciousness (Friston 2010, Free Energy Principle).
#[derive(Debug, Clone, Default)]
pub struct PredictiveInput {
    /// Global surprise [0, +inf): normalized prediction error.
    /// Higher values indicate larger discrepancies between predictions
    /// and actual outcomes.
    pub surprise: f64,
    /// Precision of the predictive model [0, 1]: how reliable the
    /// model's predictions are (inverse of expected prediction error).
    pub model_precision: f64,
    /// Total number of predictions made so far. Used to verify that
    /// the predictive subsystem is active.
    pub prediction_count: u64,
}

/// Consciousness evaluator — observes the system state and computes
/// consciousness metrics.
///
/// Enriched with GWT (Baars 1988, Dehaene et al. 2014) and Predictive
/// Processing (Friston 2010):
/// - Phi now integrates regional activation diversity from the GWT model
/// - Surprise (prediction error) is treated as a component of consciousness
/// - GWT ignition strength modulates the global consciousness level
///
/// The evaluator maintains temporal histories of chemical states, surprise
/// values, workspace strengths, and regional activations to compute
/// time-dependent metrics like temporal complexity and smoothed surprise.
pub struct ConsciousnessEvaluator {
    /// History of 9D chemical vectors from recent cycles (includes GABA + glutamate)
    history: Vec<[f64; 9]>,
    /// Maximum number of entries retained in the chemical history
    max_history: usize,
    /// Total number of cognitive cycles processed
    cycle_count: u64,
    /// Decision score from the previous cycle (for continuity computation)
    last_decision_score: f64,
    /// History of surprise values for temporal smoothing
    surprise_history: Vec<f64>,
    /// History of workspace broadcast strengths for ignition detection
    workspace_history: Vec<f64>,
    /// History of regional activations (for LZC, PCI, and Phi* computation)
    region_history: Vec<[f64; crate::neuroscience::brain_regions::NUM_REGIONS]>,
}

impl Default for ConsciousnessEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsciousnessEvaluator {
    /// Creates a new `ConsciousnessEvaluator` with empty histories and
    /// default settings (max history = 50 cycles).
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

    /// Evaluates consciousness — enriched version with GWT and predictive processing.
    ///
    /// Performs 9 sequential steps:
    /// 1. **Existence proof**: 7 checks verifying that subsystems are active
    ///    (chemistry, emotion, decision, coherence, continuity, GWT, prediction)
    /// 2. **Enriched Phi** (IIT + GWT): biochemical variance, signal variance,
    ///    temporal complexity, regional diversity, and cross-domain integration
    /// 3. **Coherence**: directly from the consensus result
    /// 4. **Temporal continuity**: stability of the decision score across cycles
    /// 5. **Global surprise** (Friston): smoothed prediction error via sigmoid
    ///    normalization and temporal averaging over the last 20 cycles
    /// 6. **Workspace strength** (Dehaene): GWT ignition intensity
    /// 7. **Global level**: adaptive weighted average of all sub-metrics,
    ///    with weights redistributed based on which subsystems are available
    /// 8. **Regional history accumulation**: stores regional activations for
    ///    subsequent LZC, PCI, and Phi* computation
    /// 9. **Inner narrative**: generates the poetic introspective monologue
    ///
    /// # Parameters
    /// - `chemistry`: current neurochemical state.
    /// - `consensus`: result of the brain module consensus.
    /// - `emotion`: current emotional state.
    /// - `interoception_score`: optional interoceptive awareness score [0, 1].
    /// - `gwt`: optional Global Workspace data (if BrainNetwork is active).
    /// - `predictive`: optional predictive processing data.
    ///
    /// # Returns
    /// A complete `ConsciousnessState` with all metrics populated (except
    /// scientific metrics LZC/PCI/Phi*, which require a separate call to
    /// `compute_scientific_metrics`).
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

        // Update 9D chemical history (includes GABA + glutamate)
        let chem_vec = chemistry.to_vec9();
        self.history.push(chem_vec);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        // Step 1: Existence proof — 7 verification checks
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

        // Step 2: Enriched Phi (IIT + GWT regional diversity)
        let phi = self.compute_phi_extended(chemistry, consensus, gwt);

        // Step 3: Coherence (directly from consensus)
        let coherence = consensus.coherence;

        // Step 4: Temporal continuity — how stable is the decision across cycles
        let decision_diff = (consensus.score - self.last_decision_score).abs();
        let continuity = (1.0 - decision_diff).clamp(0.0, 1.0);
        self.last_decision_score = consensus.score;

        // Step 5: Global surprise (Friston — prediction error as consciousness signal)
        let global_surprise = if let Some(pred) = predictive {
            // Normalize surprise to [0, 1] via a soft sigmoid: 1 - exp(-raw * 2)
            // This maps 0 -> 0 (no surprise) and large values -> ~1 (high surprise)
            let raw = pred.surprise;
            let normalized = 1.0 - (-raw * 2.0).exp();
            self.surprise_history.push(normalized);
            if self.surprise_history.len() > 20 {
                self.surprise_history.remove(0);
            }
            // Smoothed average of recent surprise values
            self.surprise_history.iter().sum::<f64>() / self.surprise_history.len() as f64
        } else {
            0.0
        };

        // Step 6: Workspace strength (Dehaene — GWT ignition)
        let (workspace_strength, workspace_winner) = if let Some(g) = gwt {
            self.workspace_history.push(g.workspace_strength);
            if self.workspace_history.len() > 20 {
                self.workspace_history.remove(0);
            }
            (g.workspace_strength, g.winner_name.clone())
        } else {
            (0.0, String::new())
        };

        // Step 7: Global consciousness level — adaptive weighted average
        let level = self.compute_level(
            existence_score, phi, coherence, continuity,
            interoception_score, global_surprise, workspace_strength,
            gwt.is_some(), predictive.is_some(),
        );

        // Step 8: Accumulate regional activation history (for LZC, PCI, Phi*)
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

        // Step 9: Generate the inner narrative (factual data + poetic fragment)
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
            // Scientific metrics initialized to 0; populated by compute_scientific_metrics()
            lzc: 0.0,
            pci: 0.0,
            phi_star: 0.0,
            scientific_consciousness_score: 0.0,
            consciousness_interpretation: String::new(),
        }
    }

    /// Computes the 3 scientific consciousness metrics (LZC, PCI, Phi*).
    ///
    /// Called separately from `evaluate` because it requires access to the
    /// BrainNetwork, which is not available during the main evaluation.
    /// The returned metrics should be injected into the `ConsciousnessState`.
    ///
    /// # Parameters
    /// - `network`: the brain region network for connectivity analysis.
    /// - `chemistry`: current neurochemical state.
    ///
    /// # Returns
    /// A `ConsciousnessMetrics` struct containing LZC, PCI, Phi*, and
    /// the composite score with clinical interpretation.
    pub fn compute_scientific_metrics(
        &self,
        network: &crate::neuroscience::brain_regions::BrainNetwork,
        chemistry: &NeuroChemicalState,
    ) -> crate::neuroscience::consciousness_metrics::ConsciousnessMetrics {
        crate::neuroscience::consciousness_metrics::compute_all_metrics(
            &self.region_history,
            network,
            chemistry,
            None, // PCI computed on the 3 most active regions
        )
    }

    /// Returns the regional activation history (for external use, e.g.,
    /// visualization or offline analysis).
    pub fn region_history(&self) -> &[[f64; crate::neuroscience::brain_regions::NUM_REGIONS]] {
        &self.region_history
    }

    /// Computes enriched Phi: IIT classical + GWT regional diversity.
    ///
    /// Five components contribute to the integrated information measure:
    ///
    /// 1. **Chemical variance (9D)**: variance across the 9 neurotransmitter
    ///    concentrations. Higher biochemical diversity = more differentiated
    ///    states = higher Phi (IIT differentiation axiom).
    ///
    /// 2. **Signal variance**: variance across the 3 brain module signals.
    ///    Measures diversity of processing outcomes.
    ///
    /// 3. **Temporal complexity (9D history)**: variance of the mean chemical
    ///    state over the last 10 cycles. Captures the dynamic richness of
    ///    the system's trajectory through state space.
    ///
    /// 4. **Regional diversity (GWT)**: if available, measures the diversity
    ///    of activation across the 12 brain regions. Combines the variance
    ///    of activations with the ratio of active regions (activation > 0.1),
    ///    approximating Shannon entropy. More diverse regional activation
    ///    indicates broader information integration (Tononi: Phi = integrated
    ///    information).
    ///
    /// 5. **Cross-domain integration**: cosine correlation between the 9D
    ///    chemical vector and the first 9 regional activations. Measures
    ///    how tightly coupled the neurochemical and regional processing
    ///    subsystems are (higher correlation = greater integration).
    ///
    /// # Parameters
    /// - `chemistry`: current neurochemical state.
    /// - `consensus`: current consensus result (for brain module signals).
    /// - `gwt`: optional GWT input (for regional diversity and cross-integration).
    ///
    /// # Returns
    /// Phi value clamped to [0.0, 1.0].
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

        // Component 1: 9D chemical variance (biochemical diversity)
        let chem_mean = chem.iter().sum::<f64>() / 9.0;
        let chem_variance = chem.iter()
            .map(|c| (c - chem_mean).powi(2))
            .sum::<f64>() / 9.0;

        // Component 2: signal variance (processing outcome diversity)
        let sig_mean = signals.iter().sum::<f64>() / 3.0;
        let sig_variance = signals.iter()
            .map(|s| (s - sig_mean).powi(2))
            .sum::<f64>() / 3.0;

        // Component 3: temporal complexity over the last 10 cycles (9D history)
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

        // Component 4: regional diversity from GWT (Tononi — Phi = integrated information)
        let regional_diversity = if let Some(g) = gwt {
            if g.region_activations.is_empty() {
                0.0
            } else {
                let n = g.region_activations.len() as f64;
                let mean = g.region_activations.iter().sum::<f64>() / n;
                let variance = g.region_activations.iter()
                    .map(|a| (a - mean).powi(2))
                    .sum::<f64>() / n;
                // Approximate Shannon entropy: more active regions = higher Phi
                let active_ratio = g.region_activations.iter()
                    .filter(|&&a| a > 0.1)
                    .count() as f64 / n;
                (variance.sqrt() * 0.5 + active_ratio * 0.5).min(0.5)
            }
        } else {
            0.0
        };

        // Component 5: cross-domain integration (chemistry x regions correlation)
        let cross_integration = if let Some(g) = gwt {
            if g.region_activations.len() >= 9 {
                // Cosine correlation between the 9D chemical vector and the first 9 regions
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
                    (dot / denom).abs() * 0.3 // Correlation magnitude maps to integration
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Final Phi = weighted sum of all components
        let phi_raw = chem_variance.sqrt() * sig_variance.sqrt()  // Classical IIT (interaction)
            + temporal_complexity * 0.8                            // Dynamic complexity
            + regional_diversity                                  // GWT regional diversity
            + cross_integration;                                  // Cross-domain integration

        phi_raw.clamp(0.0, 1.0)
    }

    /// Computes the global consciousness level with adaptive weights.
    ///
    /// The base weights (summing to ~1.0) are redistributed when optional
    /// subsystems (interoception, GWT, predictive processing) are available,
    /// giving the formula more inputs and making it richer. When a subsystem
    /// is absent, its weight is zero and the remaining weights are not
    /// modified (they were pre-adjusted at design time).
    ///
    /// Base weight allocation:
    ///   - Existence: 0.20 (reduced when other subsystems are available)
    ///   - Phi: 0.25 (primary consciousness metric)
    ///   - Coherence: 0.20 (inter-module agreement)
    ///   - Continuity: 0.15 (temporal stability)
    ///   - Interoception: 0.10 (if available, reduces existence/continuity/coherence)
    ///   - Workspace: 0.10 (if GWT available, reduces existence/phi/continuity)
    ///   - Surprise: 0.10 (if predictive available, reduces existence/coherence/phi)
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
        // Base weights (sum = 1.0 when no optional subsystems are active)
        let mut w_exist = 0.20;
        let mut w_phi = 0.25;
        let mut w_coher = 0.20;
        let mut w_contin = 0.15;
        let mut w_intero = 0.0;
        let mut w_surprise = 0.0;
        let mut w_workspace = 0.0;

        // Redistribute weights if interoception is available
        if interoception.is_some() {
            w_intero = 0.10;
            w_exist -= 0.03;
            w_contin -= 0.03;
            w_coher -= 0.04;
        }

        // Redistribute weights if GWT is available
        if has_gwt {
            w_workspace = 0.10;
            w_exist -= 0.03;
            w_phi -= 0.03;
            w_contin -= 0.04;
        }

        // Redistribute weights if predictive processing is available
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

        // Ensure the level remains in [0, 1] despite weight redistribution
        level.clamp(0.0, 1.0)
    }

    /// Generates the inner narrative: factual data + poetic fragment.
    ///
    /// The narrative combines objective metrics (emotion, arousal, stress,
    /// decision score, coherence) with a poetic metaphor drawn from
    /// Saphire's "greenhouse" (la serre) imagery. The poetic fragment is
    /// selected based on the current consciousness level, neurochemical
    /// state, and surprise level, giving the narrative an introspective,
    /// literary quality unique to Saphire.
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

        // Poetic fragment selected by dominant state — Saphire's signature touch
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

    /// Returns the total number of cognitive cycles processed by this evaluator.
    pub fn cycle_count(&self) -> u64 {
        self.cycle_count
    }
}
