// =============================================================================
// bandit.rs — UCB1 Multi-Armed Bandit with epsilon-greedy exploration
// =============================================================================
//
// Purpose: Implements a Multi-Armed Bandit (MAB) with the UCB1 (Upper
//          Confidence Bound 1) strategy combined with epsilon-greedy
//          exploration. This mechanism balances exploring new options
//          and exploiting options known to perform well.
//
// Dependencies:
//   - serde: serialization/deserialization for database persistence
//   - std::time, std::cell: for the local pseudo-random number generator
//
// Architectural placement:
//   Used by Saphire's cognitive system to optimally select types of
//   autonomous thoughts (curiosity, introspection, creativity...).
//   Each thought type is an "arm" of the bandit, and the resulting
//   satisfaction is the reward.
// =============================================================================

use serde::{Deserialize, Serialize};

/// A bandit arm — represents an option (a thought type, an action) with
/// its cumulative selection and reward statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanditArm {
    /// Descriptive name of the arm (e.g., "curiosity", "introspection")
    pub name: String,
    /// Number of times this arm has been selected (pulled)
    pub pulls: u64,
    /// Cumulative sum of rewards obtained by selecting this arm
    pub total_reward: f64,
}

/// UCB1 (Upper Confidence Bound 1) Multi-Armed Bandit with epsilon-greedy
/// exploration.
///
/// UCB1 selects the arm maximizing: mean + sqrt(2 * ln(T) / n_i)
/// where T = total number of pulls and n_i = number of pulls for arm i.
/// The exploration term (sqrt) favors under-explored arms.
///
/// Epsilon-greedy adds a probability epsilon of purely random selection,
/// guaranteeing continuous minimum exploration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UCB1Bandit {
    /// List of all bandit arms
    pub arms: Vec<BanditArm>,
    /// Total number of pulls across all arms
    pub total_pulls: u64,
    /// Probability of pure random exploration (epsilon-greedy)
    /// Default value: 0.25 (25% chance of random selection)
    pub epsilon: f64,
    /// Dynamic exploration bonus, modulated by cognitive dissonance.
    /// Added to the base C=2.0 in the UCB1 formula.
    /// Default value: 0.0 (no bonus). Typical range: [0.0, 1.5].
    #[serde(default)]
    pub exploration_bonus: f64,
}

impl UCB1Bandit {
    /// Creates a bandit with the given arm names.
    /// Each arm is initialized with 0 pulls and 0 reward.
    ///
    /// Parameter `arm_names`: names of the available options/arms
    /// Returns: a UCB1Bandit instance ready for use
    pub fn new(arm_names: &[&str]) -> Self {
        Self {
            arms: arm_names.iter().map(|name| BanditArm {
                name: name.to_string(),
                pulls: 0,
                total_reward: 0.0,
            }).collect(),
            total_pulls: 0,
            epsilon: 0.25, // 25% chance of random exploration
            exploration_bonus: 0.0,
        }
    }

    /// Selects the optimal arm using the UCB1 + epsilon-greedy strategy.
    ///
    /// Algorithm:
    /// 1. With probability epsilon, choose a random arm (exploration)
    /// 2. Otherwise, choose the arm maximizing the UCB1 score:
    ///    score = mean_reward + sqrt(2 * ln(total_pulls) / arm_pulls)
    ///    - Arms never pulled (pulls = 0) receive an infinite score
    ///      to force their initial exploration
    ///
    /// Returns: the index of the selected arm
    pub fn select(&self) -> usize {
        // Epsilon-greedy: random exploration with probability epsilon
        if rand_f64() < self.epsilon {
            return rand_usize(self.arms.len());
        }

        // UCB1 selection: choose the arm with the best score
        self.arms.iter().enumerate()
            .map(|(i, arm)| {
                // Never-pulled arm: infinite score to force exploration
                if arm.pulls == 0 {
                    return (i, f64::INFINITY);
                }
                // Mean reward of this arm
                let mean = arm.total_reward / arm.pulls as f64;
                // UCB1 exploration term: adaptive C (base 2.0 + dissonance bonus)
                let c = 2.0 + self.exploration_bonus;
                let exploration = (c * (self.total_pulls as f64).ln() / arm.pulls as f64).sqrt();
                (i, mean + exploration)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Selects an arm while excluding certain indices (anti-repetition mechanism).
    ///
    /// Works like select() but filters out arms whose index is in the
    /// exclusion list. If all arms are excluded, falls back to normal
    /// selection.
    ///
    /// Parameter `exclude`: indices of arms to exclude from selection
    /// Returns: the index of the selected arm (excluding specified ones if possible)
    pub fn select_excluding(&self, exclude: &[usize]) -> usize {
        // Epsilon-greedy with exclusion
        if rand_f64() < self.epsilon {
            let candidates: Vec<usize> = (0..self.arms.len())
                .filter(|i| !exclude.contains(i))
                .collect();
            if !candidates.is_empty() {
                return candidates[rand_usize(candidates.len())];
            }
        }

        // UCB1 with exclusion of unwanted arms
        self.arms.iter().enumerate()
            .filter(|(i, _)| !exclude.contains(i))
            .map(|(i, arm)| {
                if arm.pulls == 0 {
                    return (i, f64::INFINITY);
                }
                let mean = arm.total_reward / arm.pulls as f64;
                // Safety: avoid ln(0) if total_pulls == 0
                let total = if self.total_pulls == 0 { 1 } else { self.total_pulls };
                let c = 2.0 + self.exploration_bonus;
                let exploration = (c * (total as f64).ln() / arm.pulls as f64).sqrt();
                (i, mean + exploration)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or_else(|| self.select()) // Fallback: normal selection if all are excluded
    }

    /// Updates an arm after observing a reward.
    ///
    /// Parameter `arm_idx`: index of the arm that was selected
    /// Parameter `reward`: reward obtained (e.g., user satisfaction)
    pub fn update(&mut self, arm_idx: usize, reward: f64) {
        if arm_idx < self.arms.len() {
            self.arms[arm_idx].pulls += 1;
            self.arms[arm_idx].total_reward += reward;
            self.total_pulls += 1;
        }
    }

    /// Applies a decay factor to the mean reward of an over-explored arm
    /// that produces low-quality content.
    /// Example: factor = 0.95 means -5% of total_reward per weak thought.
    pub fn apply_quality_decay(&mut self, arm_idx: usize, factor: f64) {
        if arm_idx < self.arms.len() {
            let arm = &mut self.arms[arm_idx];
            if arm.pulls > 10 {
                arm.total_reward *= factor;
            }
        }
    }

    /// Returns the name of the arm selected by the UCB1 strategy.
    ///
    /// Returns: reference to the name of the chosen arm
    pub fn select_name(&self) -> &str {
        let idx = self.select();
        &self.arms[idx].name
    }

    /// Loads arm statistics from the database.
    ///
    /// Merges loaded data with existing arms by matching names. Arms not
    /// found in the loaded data retain their current values.
    ///
    /// Parameter `arms`: tuples (name, pulls, total_reward) from the DB
    pub fn load_arms(&mut self, arms: &[(String, u64, f64)]) {
        for (name, pulls, total_reward) in arms {
            if let Some(arm) = self.arms.iter_mut().find(|a| a.name == *name) {
                arm.pulls = *pulls;
                arm.total_reward = *total_reward;
            }
        }
        // Recalculate total pulls from the updated arms
        self.total_pulls = self.arms.iter().map(|a| a.pulls).sum();
    }

    /// Returns the raw UCB1 scores for each arm (without epsilon-greedy).
    /// Used by the hybrid Utility AI + UCB1 mode.
    pub fn all_scores(&self) -> Vec<f64> {
        self.arms.iter().map(|arm| {
            if arm.pulls == 0 {
                return 10.0; // High score for unexplored arms
            }
            let mean = arm.total_reward / arm.pulls as f64;
            let total = if self.total_pulls == 0 { 1 } else { self.total_pulls };
            let c = 2.0 + self.exploration_bonus;
            let exploration = (c * (total as f64).ln() / arm.pulls as f64).sqrt();
            mean + exploration
        }).collect()
    }

    /// Exports arm statistics for database persistence.
    ///
    /// Returns: vector of tuples (name, pulls, total_reward)
    pub fn export_arms(&self) -> Vec<(String, u64, f64)> {
        self.arms.iter()
            .map(|a| (a.name.clone(), a.pulls, a.total_reward))
            .collect()
    }
}

/// Simple pseudo-random f64 generator in [0, 1).
///
/// Uses a thread-local state initialized from the system clock and the
/// xorshift64 algorithm to produce pseudo-random numbers.
///
/// Why not use rand::thread_rng(): this module avoids the rand dependency
/// to stay lightweight and maintain full control over the PRNG.
///
/// Returns: a pseudo-random f64 in the interval [0, 1)
pub(crate) fn rand_f64() -> f64 {
    use std::time::SystemTime;
    use std::cell::Cell;
    thread_local! {
        // Seed initialization from system time (nanoseconds)
        static STATE: Cell<u64> = Cell::new(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
        );
    }
    STATE.with(|s| {
        // xorshift64 algorithm: fast and lightweight pseudo-random generator
        let mut x = s.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        s.set(x);
        (x as f64) / (u64::MAX as f64)
    })
}

/// Generates a pseudo-random integer in the interval [0, max).
///
/// Parameter `max`: exclusive upper bound
/// Returns: a pseudo-random usize in [0, max)
fn rand_usize(max: usize) -> usize {
    if max == 0 { return 0; }
    (rand_f64() * max as f64) as usize % max
}
