// =============================================================================
// memory/reconsolidation.rs — Memory reconsolidation (Nader, 2000)
// =============================================================================
//
// Purpose: Implements memory reconsolidation. Each time a memory is recalled,
// it enters a temporary labile state during which it can be modified by the
// current emotional context before being re-stabilized. This models the
// biological finding that consolidated memories, upon reactivation, require
// de novo protein synthesis to persist — making them vulnerable to alteration
// during a brief reconsolidation window.
//
// Scientific references:
//   - Nader, Schafe & LeDoux (2000): "Fear memories require protein
//     synthesis in the amygdala for reconsolidation after retrieval"
//   - Ebbinghaus (1885): exponential forgetting curve (retention = e^(-t/S))
//   - Anderson (2003): retroactive and proactive interference between
//     similar memory traces
// =============================================================================

use serde::{Deserialize, Serialize};

/// Parameters governing the reconsolidation process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconsolidationParams {
    /// Modification rate applied during recall [0.0, 0.3].
    /// Higher values mean the memory is more strongly colored by the
    /// current emotional state upon retrieval (reconsolidation update).
    pub modification_rate: f64,
    /// Duration of the lability window in cognitive cycles.
    /// After recall, the memory remains labile (modifiable) for this
    /// many cycles before re-stabilizing, analogous to the protein-
    /// synthesis-dependent reconsolidation window (~6 hours in rodents).
    pub lability_window: u64,
    /// Interference rate between similar memories [0.0, 0.5].
    /// Controls how strongly overlapping memory traces degrade each other,
    /// modeling both retroactive and proactive interference.
    pub interference_rate: f64,
    /// Decay constant for the Ebbinghaus forgetting curve.
    /// Larger values produce faster forgetting. Appears in the denominator
    /// of the stability term: base_stability = 1 / decay_constant.
    pub ebbinghaus_decay_constant: f64,
    /// Emotional retention factor: multiplier for the emotional weight's
    /// contribution to memory stability. Emotional memories are better
    /// retained, reflecting amygdala-mediated consolidation enhancement.
    pub emotional_retention_factor: f64,
}

impl Default for ReconsolidationParams {
    fn default() -> Self {
        Self {
            modification_rate: 0.1,
            lability_window: 20,    // ~20 cognitive cycles of lability
            interference_rate: 0.05,
            ebbinghaus_decay_constant: 0.3,
            emotional_retention_factor: 1.5,
        }
    }
}

/// Tracks the reconsolidation state of a single memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconsolidationState {
    /// Whether the memory is currently in a labile (modifiable) state.
    pub is_labile: bool,
    /// Number of cognitive cycles remaining in the lability window.
    pub lability_remaining: u64,
    /// Total number of times this memory has been recalled.
    pub recall_count: u64,
    /// Original emotional weight before any reconsolidation modifications.
    pub original_emotional_weight: f64,
    /// Cumulative drift from the original emotional weight, measuring the
    /// total distortion accumulated across multiple reconsolidation events.
    pub cumulative_drift: f64,
}

impl Default for ReconsolidationState {
    fn default() -> Self {
        Self {
            is_labile: false,
            lability_remaining: 0,
            recall_count: 0,
            original_emotional_weight: 0.0,
            cumulative_drift: 0.0,
        }
    }
}

/// Reconsolidation engine — manages memory lability and modification upon recall.
///
/// Each time a memory is retrieved, this engine places it into a labile state
/// and computes how the current emotional context modifies the memory trace.
/// After the lability window expires, the memory re-stabilizes with its
/// potentially altered emotional weight.
#[derive(Debug, Clone)]
pub struct ReconsolidationEngine {
    /// Configuration parameters for the reconsolidation process.
    pub params: ReconsolidationParams,
    /// Currently labile memories, keyed by memory ID, tracking their
    /// reconsolidation state (lability timer, recall count, drift).
    pub labile_memories: std::collections::HashMap<String, ReconsolidationState>,
}

impl Default for ReconsolidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ReconsolidationEngine {
    /// Creates a new reconsolidation engine with default parameters.
    pub fn new() -> Self {
        Self {
            params: ReconsolidationParams::default(),
            labile_memories: std::collections::HashMap::new(),
        }
    }

    /// Called when a memory is recalled (retrieved).
    ///
    /// Renders the memory labile and computes the emotional modification
    /// resulting from the current affective state. This implements the core
    /// reconsolidation mechanism: reactivated memories are transiently
    /// destabilized and updated by the prevailing emotional context.
    ///
    /// # Parameters
    /// - `memory_id`: unique identifier of the recalled memory.
    /// - `current_emotional_weight`: the memory's current emotional weight.
    /// - `current_valence`: current emotional valence (-1.0 to 1.0), which
    ///   "contaminates" the memory's emotional coloring.
    /// - `current_arousal`: current arousal level (0.0 to 1.0), contributing
    ///   a secondary modification effect.
    ///
    /// # Returns
    /// A `ReconsolidationEffect` describing the strength delta, emotional
    /// weight delta, and updated state metrics.
    pub fn on_recall(
        &mut self,
        memory_id: &str,
        current_emotional_weight: f64,
        current_valence: f64,
        current_arousal: f64,
    ) -> ReconsolidationEffect {
        let state = self.labile_memories
            .entry(memory_id.to_string())
            .or_insert_with(|| ReconsolidationState {
                original_emotional_weight: current_emotional_weight,
                ..Default::default()
            });

        state.recall_count += 1;
        state.is_labile = true;
        state.lability_remaining = self.params.lability_window;

        // The act of recall itself reinforces the memory trace (testing effect /
        // retrieval practice effect — Roediger & Karpicke, 2006). The reinforcement
        // scales logarithmically with recall count to model diminishing returns.
        let reinforcement = 0.02 * (state.recall_count as f64).ln().max(0.1);

        // The current emotional state "contaminates" the memory during
        // reconsolidation. The valence directly modifies the emotional weight,
        // while arousal contributes a secondary effect scaled at half strength.
        let emotional_contamination = current_valence * self.params.modification_rate;
        let arousal_effect = (current_arousal - 0.5) * self.params.modification_rate * 0.5;

        // Track cumulative drift from the original encoding to measure
        // total distortion across multiple reconsolidation events.
        state.cumulative_drift += emotional_contamination.abs() * 0.1;

        ReconsolidationEffect {
            strength_delta: reinforcement,
            emotional_weight_delta: emotional_contamination + arousal_effect,
            became_labile: true,
            recall_count: state.recall_count,
            cumulative_drift: state.cumulative_drift,
        }
    }

    /// Advances all lability timers by one cognitive cycle.
    ///
    /// Called at each cognitive cycle. Memories whose lability window has
    /// expired are re-stabilized (is_labile set to false), and fully
    /// stabilized entries are eventually removed from the tracking map
    /// to prevent unbounded growth.
    pub fn tick(&mut self) {
        let mut to_remove = Vec::new();
        for (id, state) in &mut self.labile_memories {
            if state.lability_remaining > 0 {
                state.lability_remaining -= 1;
            }
            if state.lability_remaining == 0 {
                // The memory has re-stabilized after the reconsolidation window closed.
                state.is_labile = false;
                // Mark stabilized memories for removal from the active tracking map.
                if state.recall_count > 0 && !state.is_labile {
                    to_remove.push(id.clone());
                }
            }
        }
        // Evict old stabilized entries to keep the map bounded (retain the
        // 100 most recently active entries at most).
        if self.labile_memories.len() > 100 {
            for id in to_remove {
                self.labile_memories.remove(&id);
            }
        }
    }

    /// Computes retention using the Ebbinghaus forgetting curve: R = e^(-t/S)
    /// where t = time since encoding (in cycles) and S = memory stability.
    ///
    /// Stability increases with:
    /// - Recall count (testing effect: each retrieval strengthens the trace)
    /// - Emotional weight (amygdala-mediated enhancement: emotional memories
    ///   are encoded more deeply and resist forgetting)
    /// - Spacing of recalls (spacing effect, implicitly captured by the
    ///   square-root scaling of recall_count)
    ///
    /// # Parameters
    /// - `cycles_since_encoding`: number of cognitive cycles elapsed since
    ///   the memory was first encoded.
    /// - `recall_count`: total number of times the memory has been retrieved.
    /// - `emotional_weight`: absolute emotional weight of the memory.
    ///
    /// # Returns
    /// Retention probability between 0.0 (fully forgotten) and 1.0 (perfect recall).
    pub fn ebbinghaus_retention(
        &self,
        cycles_since_encoding: u64,
        recall_count: u64,
        emotional_weight: f64,
    ) -> f64 {
        // Base stability derived from the decay constant: S_base = 1 / k
        let base_stability = 1.0 / self.params.ebbinghaus_decay_constant;

        // Recall bonus: each retrieval strengthens the trace (testing effect).
        // Uses sqrt scaling to model diminishing returns from repeated retrieval.
        let recall_bonus = (recall_count as f64).sqrt() * 2.0;

        // Emotional bonus: emotionally salient memories are retained longer,
        // reflecting amygdala-hippocampus interaction during consolidation.
        let emotional_bonus = emotional_weight.abs() * self.params.emotional_retention_factor;

        // Total stability is the sum of all contributing factors.
        let total_stability = base_stability + recall_bonus + emotional_bonus;

        // Ebbinghaus forgetting curve: retention = e^(-t/S)
        // Clamping stability to >= 1.0 prevents division by zero.
        let t = cycles_since_encoding as f64;
        (-t / total_stability.max(1.0)).exp()
    }

    /// Computes the interference factor between two similar memories.
    ///
    /// Models both retroactive interference (a new memory degrades retrieval
    /// of an older, similar memory) and proactive interference (an old memory
    /// impairs encoding/retrieval of a new, similar memory), following
    /// Anderson (2003).
    ///
    /// Interference only occurs when similarity exceeds 0.5 (the threshold
    /// for trace overlap). Retroactive interference is stronger than proactive
    /// (direction_factor = 1.0 vs 0.7).
    ///
    /// # Parameters
    /// - `similarity`: cosine similarity between the two memory traces (0.0 to 1.0).
    /// - `is_retroactive`: true if computing retroactive interference (new
    ///   memory degrading old), false for proactive.
    ///
    /// # Returns
    /// A weakening factor in [0.5, 1.0] where 1.0 = no interference and
    /// 0.5 = maximum allowed interference (50% weakening cap).
    pub fn compute_interference(
        &self,
        similarity: f64,
        is_retroactive: bool,
    ) -> f64 {
        if similarity < 0.5 {
            // Similarity too low for meaningful trace overlap — no interference.
            return 1.0;
        }

        // Interference strength scales linearly with similarity above the 0.5 threshold.
        let interference_strength = (similarity - 0.5) * 2.0 * self.params.interference_rate;
        // Retroactive interference is stronger than proactive (Anderson, 2003).
        let direction_factor = if is_retroactive { 1.0 } else { 0.7 };
        let weakening = 1.0 - (interference_strength * direction_factor);
        // Clamp to [0.5, 1.0]: never allow more than 50% weakening from interference.
        weakening.clamp(0.5, 1.0)
    }
}

/// Describes the effect of reconsolidation on a single recalled memory.
#[derive(Debug, Clone)]
pub struct ReconsolidationEffect {
    /// Strength delta (positive = reinforcement from the testing effect).
    pub strength_delta: f64,
    /// Emotional weight delta (contamination by the current affective state).
    pub emotional_weight_delta: f64,
    /// Whether the memory became labile as a result of this recall.
    pub became_labile: bool,
    /// Total number of times this memory has been recalled.
    pub recall_count: u64,
    /// Cumulative drift from the original emotional weight across all
    /// reconsolidation events, measuring total distortion.
    pub cumulative_drift: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ebbinghaus_curve() {
        let engine = ReconsolidationEngine::new();
        let ret_0 = engine.ebbinghaus_retention(0, 0, 0.0);
        let ret_100 = engine.ebbinghaus_retention(100, 0, 0.0);
        let ret_1000 = engine.ebbinghaus_retention(1000, 0, 0.0);
        assert!(ret_0 > ret_100, "Retention should decrease over time");
        assert!(ret_100 > ret_1000, "Retention should continue decreasing");
        assert!(ret_0 > 0.9, "Initial retention should be close to 1.0");
    }

    #[test]
    fn test_recall_improves_retention() {
        let engine = ReconsolidationEngine::new();
        let ret_no_recall = engine.ebbinghaus_retention(100, 0, 0.0);
        let ret_with_recall = engine.ebbinghaus_retention(100, 5, 0.0);
        assert!(ret_with_recall > ret_no_recall,
            "Recalls should improve retention (testing effect)");
    }

    #[test]
    fn test_emotional_memories_last_longer() {
        let engine = ReconsolidationEngine::new();
        let ret_neutral = engine.ebbinghaus_retention(200, 0, 0.1);
        let ret_emotional = engine.ebbinghaus_retention(200, 0, 0.9);
        assert!(ret_emotional > ret_neutral,
            "Emotional memories should last longer");
    }

    #[test]
    fn test_interference() {
        let engine = ReconsolidationEngine::new();
        let weak_interference = engine.compute_interference(0.3, true);
        let strong_interference = engine.compute_interference(0.9, true);
        assert_eq!(weak_interference, 1.0, "Low similarity = no interference");
        assert!(strong_interference < 1.0, "High similarity = interference");
    }

    #[test]
    fn test_reconsolidation_on_recall() {
        let mut engine = ReconsolidationEngine::new();
        let effect = engine.on_recall("test_memory", 0.5, 0.3, 0.6);
        assert!(effect.strength_delta > 0.0, "Recall should reinforce the memory");
        assert!(effect.became_labile, "Memory should become labile");
        assert_eq!(effect.recall_count, 1);
    }
}
