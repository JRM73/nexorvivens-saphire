// =============================================================================
// memory/reconsolidation.rs — Memory reconsolidation (Nader 2000)
// =============================================================================
//
// Role: Implements memory reconsolidation. Each recall of a memory
// temporarily renders it labile — it can be modified by the current
// emotional state before being re-stabilized.
//
// Scientific references:
//   - Nader, Schafe & LeDoux (2000): "Fear memories require protein
//     synthesis in the amygdala for reconsolidation after retrieval"
//   - Ebbinghaus (1885): exponential forgetting curve
//   - Anderson (2003): retroactive and proactive interference
// =============================================================================

use serde::{Deserialize, Serialize};

/// Reconsolidation parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconsolidationParams {
    /// Modification rate upon recall [0.0, 0.3]
    /// Higher = the memory is more modified by the current state
    pub modification_rate: f64,
    /// Lability duration in cycles (reconsolidation window)
    pub lability_window: u64,
    /// Interference rate between similar memories [0.0, 0.5]
    pub interference_rate: f64,
    /// Ebbinghaus curve constant (forgetting speed)
    /// Larger = faster forgetting
    pub ebbinghaus_decay_constant: f64,
    /// Emotional retention factor
    /// Emotional memories are better retained
    pub emotional_retention_factor: f64,
}

impl Default for ReconsolidationParams {
    fn default() -> Self {
        Self {
            modification_rate: 0.1,
            lability_window: 20,    // ~20 cycles of lability
            interference_rate: 0.05,
            ebbinghaus_decay_constant: 0.3,
            emotional_retention_factor: 1.5,
        }
    }
}

/// Reconsolidation state of a memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconsolidationState {
    /// Is the memory currently labile?
    pub is_labile: bool,
    /// Remaining cycles of lability
    pub lability_remaining: u64,
    /// Number of times the memory has been recalled
    pub recall_count: u64,
    /// Original emotional weight (before modification)
    pub original_emotional_weight: f64,
    /// Cumulative drift from the original (distortion measure)
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

/// Reconsolidation engine — manages lability and modification of memories.
#[derive(Debug, Clone)]
pub struct ReconsolidationEngine {
    pub params: ReconsolidationParams,
    /// Currently labile memories (memory_id -> state)
    pub labile_memories: std::collections::HashMap<String, ReconsolidationState>,
}

impl Default for ReconsolidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ReconsolidationEngine {
    pub fn new() -> Self {
        Self {
            params: ReconsolidationParams::default(),
            labile_memories: std::collections::HashMap::new(),
        }
    }

    /// Called upon recall of a memory.
    /// Renders the memory labile and computes the emotional modification.
    ///
    /// Returns the emotional weight delta to apply to the memory.
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

        // Recall reinforces the memory (testing effect)
        let reinforcement = 0.02 * (state.recall_count as f64).ln().max(0.1);

        // But the current emotional state colors the memory (reconsolidation)
        // The current valence "contaminates" the memory
        let emotional_contamination = current_valence * self.params.modification_rate;
        let arousal_effect = (current_arousal - 0.5) * self.params.modification_rate * 0.5;

        // The drift accumulates
        state.cumulative_drift += emotional_contamination.abs() * 0.1;

        ReconsolidationEffect {
            strength_delta: reinforcement,
            emotional_weight_delta: emotional_contamination + arousal_effect,
            became_labile: true,
            recall_count: state.recall_count,
            cumulative_drift: state.cumulative_drift,
        }
    }

    /// Tick: advance lability timers.
    /// Called at each cognitive cycle.
    pub fn tick(&mut self) {
        let mut to_remove = Vec::new();
        for (id, state) in &mut self.labile_memories {
            if state.lability_remaining > 0 {
                state.lability_remaining -= 1;
            }
            if state.lability_remaining == 0 {
                state.is_labile = false;
                // The memory is re-stabilized — keep the state for tracking
                // but remove from the active map after a while
                if state.recall_count > 0 && !state.is_labile {
                    to_remove.push(id.clone());
                }
            }
        }
        // Clean up old stabilized memories (keep the 100 most recent)
        if self.labile_memories.len() > 100 {
            for id in to_remove {
                self.labile_memories.remove(&id);
            }
        }
    }

    /// Ebbinghaus forgetting curve: retention = e^(-t/S)
    /// where t = time since encoding, S = memory stability.
    ///
    /// Stability increases with:
    /// - Number of recalls (testing effect)
    /// - Emotional weight (emotional memories last longer)
    /// - Spacing of recalls (spacing effect)
    pub fn ebbinghaus_retention(
        &self,
        cycles_since_encoding: u64,
        recall_count: u64,
        emotional_weight: f64,
    ) -> f64 {
        // Base stability
        let base_stability = 1.0 / self.params.ebbinghaus_decay_constant;

        // Stability bonus per recall (testing effect + spacing)
        let recall_bonus = (recall_count as f64).sqrt() * 2.0;

        // Emotional bonus (emotional memories are better retained)
        let emotional_bonus = emotional_weight.abs() * self.params.emotional_retention_factor;

        let total_stability = base_stability + recall_bonus + emotional_bonus;

        // Ebbinghaus curve: retention = e^(-t/S)
        let t = cycles_since_encoding as f64;
        (-t / total_stability.max(1.0)).exp()
    }

    /// Computes the interference between two similar memories.
    /// Retroactive interference (new memory degrades the old one)
    /// and proactive interference (old memory interferes with the new one).
    ///
    /// Returns the weakening factor [0.0, 1.0] (1.0 = no interference).
    pub fn compute_interference(
        &self,
        similarity: f64,
        is_retroactive: bool,
    ) -> f64 {
        if similarity < 0.5 {
            return 1.0; // Not similar enough to interfere
        }

        let interference_strength = (similarity - 0.5) * 2.0 * self.params.interference_rate;
        let direction_factor = if is_retroactive { 1.0 } else { 0.7 }; // Retroactive is stronger
        let weakening = 1.0 - (interference_strength * direction_factor);
        weakening.clamp(0.5, 1.0) // Never more than 50% weakening
    }
}

/// Effect of reconsolidation on a memory.
#[derive(Debug, Clone)]
pub struct ReconsolidationEffect {
    /// Strength delta of the memory (positive = reinforcement from recall)
    pub strength_delta: f64,
    /// Emotional weight delta (contamination by current state)
    pub emotional_weight_delta: f64,
    /// The memory became labile
    pub became_labile: bool,
    /// Total number of recalls
    pub recall_count: u64,
    /// Cumulative drift from the original
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
        assert!(ret_0 > ret_100, "Retention diminue avec le temps");
        assert!(ret_100 > ret_1000, "Retention continue de diminuer");
        assert!(ret_0 > 0.9, "Retention initiale proche de 1.0");
    }

    #[test]
    fn test_recall_improves_retention() {
        let engine = ReconsolidationEngine::new();
        let ret_no_recall = engine.ebbinghaus_retention(100, 0, 0.0);
        let ret_with_recall = engine.ebbinghaus_retention(100, 5, 0.0);
        assert!(ret_with_recall > ret_no_recall,
            "Les rappels ameliorent la retention (testing effect)");
    }

    #[test]
    fn test_emotional_memories_last_longer() {
        let engine = ReconsolidationEngine::new();
        let ret_neutral = engine.ebbinghaus_retention(200, 0, 0.1);
        let ret_emotional = engine.ebbinghaus_retention(200, 0, 0.9);
        assert!(ret_emotional > ret_neutral,
            "Les souvenirs emotionnels durent plus longtemps");
    }

    #[test]
    fn test_interference() {
        let engine = ReconsolidationEngine::new();
        let weak_interference = engine.compute_interference(0.3, true);
        let strong_interference = engine.compute_interference(0.9, true);
        assert_eq!(weak_interference, 1.0, "Faible similarite = pas d'interference");
        assert!(strong_interference < 1.0, "Forte similarite = interference");
    }

    #[test]
    fn test_reconsolidation_on_recall() {
        let mut engine = ReconsolidationEngine::new();
        let effect = engine.on_recall("test_memory", 0.5, 0.3, 0.6);
        assert!(effect.strength_delta > 0.0, "Le rappel renforce le souvenir");
        assert!(effect.became_labile, "Le souvenir devient labile");
        assert_eq!(effect.recall_count, 1);
    }
}
