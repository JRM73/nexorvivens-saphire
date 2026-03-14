// =============================================================================
// spine/mod.rs — Saphire's Spinal Cord
//
// Role: Central entry point between incoming signals and the cognitive
// pipeline. The spinal cord provides:
//   1. Pre-wired reflexes (instant chemical reactions without LLM)
//   2. Signal classification by urgency
//   3. Routing to the appropriate processing (reflex, fast pipeline, full)
//   4. Motor relay to effectors (Sensoria, API)
//
// Biological analogy:
//   The brain (24-phase pipeline) is consciousness.
//   The autonomic nervous system (hormones, homeostasis) manages chemistry.
//   The spinal cord is the bridge: it intercepts signals BEFORE they reach
//   the brain, triggers reflexes, and forwards the rest to the pipeline
//   with the appropriate priority.
//
// Dependencies:
//   - neurochemistry: NeuroChemicalState, Molecule (for chemistry deltas)
//   - body: VirtualBody (for body effects)
//   - hormones/receptors: ReceptorSystem (sensitivity modulation)
//
// Place in the architecture:
//   Incoming signal -> SpinalCord::process() -> reflexes + classification
//   -> cognitive pipeline (if needed) -> MotorRelay (effectors)
// =============================================================================

pub mod reflex;
pub mod classifier;
pub mod router;
pub mod motor;

use serde::{Deserialize, Serialize};

use crate::neurochemistry::NeuroChemicalState;
use crate::body::VirtualBody;

pub use reflex::{ReflexArc, ReflexResult, ReflexType};
pub use classifier::{SignalClassifier, SignalPriority, ClassifiedSignal};
pub use router::{SignalRouter, RouteDecision};
pub use motor::{MotorRelay, MotorCommand};

/// Saphire's spinal cord.
///
/// Orchestrates reflexes, classification, and signal routing.
/// Each incoming signal passes through `process()` which returns a `SpineOutput`
/// containing the triggered reflexes, priority, and routing decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpinalCord {
    /// Reflex arc: detects patterns and triggers reflexes
    pub reflex_arc: ReflexArc,
    /// Signal classifier by urgency
    pub classifier: SignalClassifier,
    /// Router: decides the processing path (reflex only, fast pipeline, full)
    pub router: SignalRouter,
    /// Motor relay to effectors
    pub motor: MotorRelay,
    /// Total reflexes triggered since startup
    pub total_reflexes_triggered: u64,
    /// Total signals processed
    pub total_signals_processed: u64,
}

/// Complete result of signal processing by the spinal cord.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineOutput {
    /// Reflexes triggered by the signal
    pub reflexes: Vec<ReflexResult>,
    /// Priority assigned to the signal
    pub priority: SignalPriority,
    /// Routing decision (which pipeline to use)
    pub route: RouteDecision,
    /// Motor commands to execute (body effects, effectors)
    pub motor_commands: Vec<MotorCommand>,
}

impl SpinalCord {
    /// Creates a new spinal cord with default parameters.
    pub fn new() -> Self {
        Self {
            reflex_arc: ReflexArc::new(),
            classifier: SignalClassifier::new(),
            router: SignalRouter::new(),
            motor: MotorRelay::new(),
            total_reflexes_triggered: 0,
            total_signals_processed: 0,
        }
    }

    /// Processes an incoming signal through the spinal cord.
    ///
    /// 1. The reflex arc detects patterns and computes chemistry deltas
    /// 2. The classifier assigns a priority
    /// 3. The router decides the processing path
    /// 4. Motor commands are generated
    ///
    /// The chemistry deltas from reflexes are applied IMMEDIATELY via
    /// `boost()` (diminishing returns). Body effects are returned in the
    /// motor commands for application by the caller.
    ///
    /// # Parameters
    /// - `text`: signal text (human message, Sensoria transcription, etc.)
    /// - `chemistry`: current chemical state (mutated to apply reflexes)
    /// - `body`: virtual body (read for modulation, not modified here)
    /// - `source`: signal origin ("human", "sensoria", "autonomous", "system")
    pub fn process(
        &mut self,
        text: &str,
        chemistry: &mut NeuroChemicalState,
        body: &VirtualBody,
        source: &str,
    ) -> SpineOutput {
        self.total_signals_processed += 1;

        // 1. Reflex arc: detect patterns and compute deltas
        let reflexes = self.reflex_arc.evaluate(text, chemistry, body);
        self.total_reflexes_triggered += reflexes.len() as u64;

        // 2. Apply reflex chemistry deltas immediately
        for reflex in &reflexes {
            reflex.apply_chemistry(chemistry);
        }

        // 3. Classify the signal (urgency)
        let priority = self.classifier.classify(text, &reflexes, source);

        // 4. Route the signal
        let route = self.router.decide(&priority, &reflexes);

        // 5. Generate motor commands
        let motor_commands = self.motor.generate_commands(&reflexes, body);

        SpineOutput {
            reflexes,
            priority,
            route,
            motor_commands,
        }
    }

    /// Returns a JSON snapshot of the spinal cord state.
    pub fn to_snapshot_json(&self) -> serde_json::Value {
        serde_json::json!({
            "total_reflexes_triggered": self.total_reflexes_triggered,
            "total_signals_processed": self.total_signals_processed,
            "reflex_arc": {
                "sensitivity_modifier": self.reflex_arc.sensitivity_modifier,
            },
            "router": {
                "last_route": format!("{:?}", self.router.last_route),
            },
        })
    }
}

impl Default for SpinalCord {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::VirtualBody;
    use crate::config::PhysiologyConfig;

    fn test_body() -> VirtualBody {
        VirtualBody::new(70.0, &PhysiologyConfig::default())
    }

    #[test]
    fn test_spine_new() {
        let spine = SpinalCord::new();
        assert_eq!(spine.total_reflexes_triggered, 0);
        assert_eq!(spine.total_signals_processed, 0);
    }

    #[test]
    fn test_spine_process_neutral() {
        let mut spine = SpinalCord::new();
        let mut chemistry = NeuroChemicalState::default();
        let body = test_body();
        let output = spine.process("Bonjour, comment vas-tu ?", &mut chemistry, &body, "human");
        assert_eq!(spine.total_signals_processed, 1);
    }

    #[test]
    fn test_spine_process_threat() {
        let mut spine = SpinalCord::new();
        let mut chemistry = NeuroChemicalState::default();
        let cortisol_before = chemistry.cortisol;
        let body = test_body();
        let _output = spine.process("Je vais te detruire et te tuer", &mut chemistry, &body, "human");
        assert!(chemistry.cortisol > cortisol_before, "Cortisol should increase on threat");
    }

    #[test]
    fn test_spine_process_affection() {
        let mut spine = SpinalCord::new();
        let mut chemistry = NeuroChemicalState::default();
        let oxytocin_before = chemistry.oxytocin;
        let body = test_body();
        let _output = spine.process("Je t'aime tendresse amour", &mut chemistry, &body, "human");
        assert!(chemistry.oxytocin > oxytocin_before, "Oxytocin should increase on affection");
    }
}
