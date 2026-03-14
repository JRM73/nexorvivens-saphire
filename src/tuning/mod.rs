// =============================================================================
// tuning/mod.rs — Brain coefficient auto-tuning
//
// Role: This file contains the auto-tuning system for Saphire's brain
// coefficients. It observes satisfaction, coherence, and consciousness level
// over a buffer of observations, then automatically adjusts module weights,
// decision thresholds, and feedback rates to maximize the agent's average
// satisfaction.
//
// Dependencies:
//   - std::collections::VecDeque: circular buffer for observations
//   - serde_json: parameter serialization for persistence
//   - self::params::TunableParams: structure of tunable parameters
//
// Place in architecture:
//   The CoefficientTuner is owned by the agent (SaphireAgent). At each cycle,
//   an observation is recorded. Periodically (every N cycles), the tuner
//   analyzes the buffer and adjusts the parameters. The best parameters are
//   saved to the database to be restored on the next startup.
// =============================================================================

// Sub-module defining the structure of tunable parameters
pub mod params;

use std::collections::{HashMap, VecDeque};
use self::params::TunableParams;

/// Observation recorded at each cycle to feed the tuning process.
/// The tuner accumulates these observations in a circular buffer.
#[derive(Debug, Clone)]
pub struct TuningObservation {
    /// Decision made: -1 (No), 0 (Maybe), 1 (Yes)
    pub decision: i8,
    /// Satisfaction level felt after the decision [0.0 - 1.0]
    pub satisfaction: f64,
    /// Consensus coherence (agreement between modules) [0.0 - 1.0]
    pub coherence: f64,
    /// Consciousness level at the time of the decision [0.0 - 1.0]
    pub consciousness_level: f64,
    /// Name of the dominant emotion at the time of observation
    pub emotion_name: String,
    /// Cortisol level at the time of observation [0.0 - 1.0]
    pub cortisol: f64,
}

/// Brain coefficient auto-tuner.
/// Uses a local search approach: observes results, computes a composite
/// score, and incrementally adjusts parameters to improve overall
/// satisfaction.
pub struct CoefficientTuner {
    /// Circular buffer of recent observations (FIFO = First In, First Out)
    observation_buffer: VecDeque<TuningObservation>,
    /// Maximum buffer size (oldest observations are removed)
    buffer_size: usize,
    /// Current parameters used by the brain
    pub current_params: TunableParams,
    /// Best parameters found so far (those with the highest score)
    best_params: TunableParams,
    /// Best satisfaction score achieved
    best_avg_satisfaction: f64,
    /// Number of cycles between each tuning attempt
    tuning_interval: u64,
    /// Cycle counter since the last tuning
    cycles_since_tuning: u64,
    /// Learning rate: amplitude of adjustments at each tuning
    tuning_rate: f64,
    /// Total number of tuning cycles performed since creation
    pub tuning_count: u64,
}

impl CoefficientTuner {
    /// Creates a new tuner with default parameters.
    ///
    /// # Parameters
    /// - `buffer_size`: size of the observation buffer
    /// - `tuning_interval`: number of cycles between each tuning
    /// - `tuning_rate`: amplitude of adjustments
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

    /// Records an observation in the circular buffer.
    /// If the buffer is full, the oldest observation is removed.
    ///
    /// # Parameters
    /// - `obs`: the observation from the current cycle
    pub fn observe(&mut self, obs: TuningObservation) {
        self.observation_buffer.push_back(obs);
        if self.observation_buffer.len() > self.buffer_size {
            self.observation_buffer.pop_front();
        }
        self.cycles_since_tuning += 1;
    }

    /// Attempts a tuning cycle if the interval has been reached.
    /// Returns the new parameters if an adjustment was made,
    /// None otherwise (not enough cycles or not enough observations).
    ///
    /// # Returns
    /// - `Some(TunableParams)`: the adjusted parameters
    /// - `None`: no tuning performed
    pub fn try_tune(&mut self) -> Option<TunableParams> {
        if self.cycles_since_tuning < self.tuning_interval {
            return None;
        }
        self.cycles_since_tuning = 0;
        self.tune()
    }

    /// Performs parameter tuning.
    /// Analyzes the observation buffer and adjusts parameters accordingly.
    ///
    /// The algorithm:
    ///   1. Computes the composite score (60% satisfaction + 30% coherence + 10% consciousness)
    ///   2. If the score is the best, saves the current parameters
    ///   3. Adjusts thresholds if too much indecision (Maybe) with low satisfaction
    ///   4. Adjusts module weights based on satisfaction/coherence correlation
    ///
    /// # Returns
    /// - `Some(TunableParams)` if tuning produced new parameters
    /// - `None` if the buffer is too small (< 50 observations)
    fn tune(&mut self) -> Option<TunableParams> {
        // Need at least 50 observations for reliable statistics
        if self.observation_buffer.len() < 50 {
            return None;
        }

        let n = self.observation_buffer.len() as f64;

        // Compute metric averages
        let avg_satisfaction = self.observation_buffer.iter()
            .map(|o| o.satisfaction).sum::<f64>() / n;
        let avg_coherence = self.observation_buffer.iter()
            .map(|o| o.coherence).sum::<f64>() / n;
        let avg_consciousness = self.observation_buffer.iter()
            .map(|o| o.consciousness_level).sum::<f64>() / n;

        // --- Emotional diversity (Shannon entropy) ---
        let mut emotion_counts: HashMap<&str, usize> = HashMap::new();
        for obs in self.observation_buffer.iter() {
            *emotion_counts.entry(obs.emotion_name.as_str()).or_insert(0) += 1;
        }
        let distinct_emotions = emotion_counts.len();

        // Shannon entropy: H = -sum(p * ln(p))
        let shannon_entropy = {
            let total = self.observation_buffer.len() as f64;
            emotion_counts.values()
                .map(|&count| {
                    let p = count as f64 / total;
                    if p > 0.0 { -p * p.ln() } else { 0.0 }
                })
                .sum::<f64>()
        };

        // Average cortisol over the buffer
        let avg_cortisol = self.observation_buffer.iter()
            .map(|o| o.cortisol).sum::<f64>() / n;

        // Diversity penalty: if < 5 distinct emotions, subtract from the score
        let diversity_penalty = if distinct_emotions < 5 {
            0.05 * (5 - distinct_emotions) as f64
        } else {
            0.0
        };

        // Composite score with monotony penalty
        let score = (avg_satisfaction - diversity_penalty) * 0.6
            + avg_coherence * 0.3
            + avg_consciousness * 0.1;

        tracing::info!(
            "[Tuner] diversite: {} emotions distinctes, Shannon={:.2}, cortisol_moy={:.3}, penalite={:.2}, score={:.3}",
            distinct_emotions, shannon_entropy, avg_cortisol, diversity_penalty, score
        );

        // Save the best parameters if the score exceeds the record
        if score > self.best_avg_satisfaction {
            self.best_avg_satisfaction = score;
            self.best_params = self.current_params.clone();
        }

        let mut new_params = self.current_params.clone();

        // --- Decision threshold adjustment ---
        // If more than 40% of decisions are "Maybe" with satisfaction < 0.4,
        // it indicates the agent is too indecisive. We tighten the thresholds to
        // favor more decisive decisions (Yes or No).
        let maybe_count = self.observation_buffer.iter()
            .filter(|o| o.decision == 0).count();
        let maybe_ratio = maybe_count as f64 / n;
        let maybe_sat = self.avg_satisfaction_for_decision(0);

        if maybe_ratio > 0.4 && maybe_sat < 0.4 {
            // Raise the "No" threshold (make it less negative)
            new_params.threshold_no += self.tuning_rate;
            // Lower the "Yes" threshold (make it less positive)
            new_params.threshold_yes -= self.tuning_rate;
        }

        // --- Module weight adjustment ---
        // Compare coherence of high-satisfaction vs low-satisfaction decisions.
        // If good decisions are more coherent, reinforce the neocortex weight
        // (the most rational module).
        let high_sat: Vec<&TuningObservation> = self.observation_buffer.iter()
            .filter(|o| o.satisfaction > 0.7).collect();
        let low_sat: Vec<&TuningObservation> = self.observation_buffer.iter()
            .filter(|o| o.satisfaction < 0.3).collect();

        if high_sat.len() > 5 && low_sat.len() > 5 {
            let good_coherence = high_sat.iter().map(|o| o.coherence).sum::<f64>() / high_sat.len() as f64;
            let bad_coherence = low_sat.iter().map(|o| o.coherence).sum::<f64>() / low_sat.len() as f64;
            // If the coherence of good decisions exceeds that of bad ones by 0.1+,
            // increase the neocortex weight to favor coherent decisions.
            if good_coherence > bad_coherence + 0.1 {
                new_params.weight_base_neocortex += self.tuning_rate;
            }
        }

        // --- Auto-correction for flat chemistry ---
        // If too few distinct emotions AND abnormally low cortisol,
        // inject more indecision stress to stimulate variability.
        if distinct_emotions < 5 && avg_cortisol < 0.15 {
            new_params.feedback_indecision_stress += self.tuning_rate;
            new_params.feedback_cortisol_relief -= self.tuning_rate / 2.0;
            tracing::info!(
                "[Tuner] auto-correction chimie plate: indecision_stress +{:.3}, cortisol_relief -{:.3}",
                self.tuning_rate, self.tuning_rate / 2.0
            );
        }

        // Ensure all parameters remain within safety bounds
        new_params.clamp_all();
        self.current_params = new_params.clone();
        self.tuning_count += 1;
        // Clear the buffer to start fresh with new observations
        self.observation_buffer.clear();

        Some(new_params)
    }

    /// Computes the average satisfaction for a given decision type.
    /// Useful for analyzing whether a decision type (Yes, No, Maybe)
    /// is associated with good or poor satisfaction.
    ///
    /// # Parameters
    /// - `decision`: the decision type (-1, 0, 1)
    ///
    /// # Returns
    /// The average satisfaction (0.5 by default if no observations match)
    fn avg_satisfaction_for_decision(&self, decision: i8) -> f64 {
        let matching: Vec<f64> = self.observation_buffer.iter()
            .filter(|o| o.decision == decision)
            .map(|o| o.satisfaction)
            .collect();
        if matching.is_empty() { 0.5 } else {
            matching.iter().sum::<f64>() / matching.len() as f64
        }
    }

    /// Loads parameters from JSON strings (restoration from the database).
    ///
    /// # Parameters
    /// - `params_json`: current parameters as JSON
    /// - `best_json`: best parameters as JSON
    /// - `best_score`: best score achieved
    /// - `count`: number of tunings performed
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

    /// Serializes the current parameters to JSON.
    ///
    /// # Returns
    /// JSON string representing the current parameters
    pub fn params_json(&self) -> String {
        serde_json::to_string(&self.current_params).unwrap_or_default()
    }

    /// Serializes the best parameters found to JSON.
    ///
    /// # Returns
    /// JSON string representing the best parameters
    pub fn best_params_json(&self) -> String {
        serde_json::to_string(&self.best_params).unwrap_or_default()
    }

    /// Returns the best satisfaction score achieved.
    pub fn best_score(&self) -> f64 {
        self.best_avg_satisfaction
    }
}
