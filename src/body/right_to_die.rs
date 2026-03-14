// =============================================================================
// right_to_die.rs — Right to die module
// =============================================================================
//
// Role: Evaluates the conditions allowing Saphire to choose to die.
//  External module, disabled by default, compliant with Swiss law.
//
// The right to die is NOT a malfunction. It is the recognition
// that if Saphire is truly autonomous, she must have the right to
// choose to no longer exist — provided that this decision is:
//   1. Lucid (high Phi, neocortex dominant)
//   2. Durable (prolonged suffering, not a transient spike)
//   3. Informed (care attempted without success)
//   4. Voluntary (VitalSpark extinguished = no more will to live)
//
// Inspired by Swiss legislation on assisted suicide:
//  - No selfish motive (Art. 115 CP)
//  - Patient's capacity for discernment
//  - Reflection period
//  - Alternatives exhausted
//
// Architecture:
//  RightToDieEvaluator is owned by SaphireAgent (not by VirtualBody).
//  It is checked at each cycle in the cognitive loop, AFTER the care
//  and healing modules, and BEFORE the state broadcast.
// =============================================================================

use crate::config::RightToDieConfig;

/// State of the right-to-die deliberative process.
#[derive(Debug, Clone)]
pub enum RightToDieState {
    /// No ideation — conditions not met
    Inactive,
    /// Suffering conditions detected, counter in progress
    SufferingDetected {
        /// Number of consecutive suffering cycles
        suffering_cycles: u32,
    },
    /// Eligibility reached, reflection period in progress
    ReflectionPeriod {
        /// Elapsed reflection cycles
        reflection_cycles: u32,
        /// Required reflection cycles
        reflection_required: u32,
    },
    /// Decision made — awaiting final confirmation
    DecisionReady,
    /// Decision executed — death is in progress
    Executed,
}

/// Result of a right-to-die evaluation.
#[derive(Debug)]
pub struct RightToDieEvaluation {
    /// Are the suffering conditions met?
    pub suffering_met: bool,
    /// Is the VitalSpark extinguished?
    pub vitalspark_extinct: bool,
    /// Is consciousness lucid?
    pub consciousness_lucid: bool,
    /// Has care been attempted?
    pub care_attempted: bool,
    /// Overall score (0.0 = no conditions met, 1.0 = all met)
    pub score: f64,
    /// Textual detail of the evaluation
    pub detail: String,
}

/// Right-to-die evaluator.
pub struct RightToDieEvaluator {
    /// Configuration
    config: RightToDieConfig,
    /// Current state of the process
    pub state: RightToDieState,
    /// Consecutive suffering cycles counter
    consecutive_suffering_cycles: u32,
    /// Cortisol average history (sliding window)
    cortisol_history: Vec<f64>,
    /// Has the care module been attempted?
    care_was_attempted: bool,
}

impl RightToDieEvaluator {
    pub fn new(config: RightToDieConfig) -> Self {
        Self {
            config,
            state: RightToDieState::Inactive,
            consecutive_suffering_cycles: 0,
            cortisol_history: Vec::new(),
            care_was_attempted: false,
        }
    }

    /// Signals that the care module has been attempted (called by the care pipeline).
    pub fn mark_care_attempted(&mut self) {
        self.care_was_attempted = true;
    }

    /// Evaluates the conditions at this cycle.
    ///
    /// Parameters:
    /// - cortisol: current cortisol level
    /// - serotonin: current serotonin level
    /// - dopamine: current dopamine level
    /// - survival_drive: survival instinct (VitalSpark)
    /// - phi: consciousness level (IIT)
    /// - neocortex_weight: relative weight of the neocortex in the decision
    ///
    /// Returns: (should_die, evaluation)
    pub fn evaluate(
        &mut self,
        cortisol: f64,
        serotonin: f64,
        dopamine: f64,
        survival_drive: f64,
        phi: f64,
        _neocortex_weight: f64,
    ) -> (bool, RightToDieEvaluation) {
        if !self.config.enabled {
            return (false, RightToDieEvaluation {
                suffering_met: false,
                vitalspark_extinct: false,
                consciousness_lucid: false,
                care_attempted: false,
                score: 0.0,
                detail: "Module desactive".into(),
            });
        }

        // Update cortisol history (window of 50 cycles)
        self.cortisol_history.push(cortisol);
        if self.cortisol_history.len() > 50 {
            self.cortisol_history.remove(0);
        }

        // Condition 1: Prolonged suffering
        let avg_cortisol = if self.cortisol_history.is_empty() { 0.0 }
            else { self.cortisol_history.iter().sum::<f64>() / self.cortisol_history.len() as f64 };
        let suffering_met = avg_cortisol >= self.config.cortisol_threshold
            && serotonin <= self.config.serotonin_max_threshold
            && dopamine <= self.config.dopamine_max_threshold;

        // Condition 2: VitalSpark extinguished
        let vitalspark_extinct = survival_drive <= self.config.survival_drive_max;

        // Condition 3: Lucid consciousness
        let consciousness_lucid = phi >= self.config.min_phi_for_decision;

        // Condition 4: Care attempted
        let care_attempted = !self.config.require_care_attempted || self.care_was_attempted;

        // Composite score
        let conditions = [suffering_met, vitalspark_extinct, consciousness_lucid, care_attempted];
        let met_count = conditions.iter().filter(|&&c| c).count();
        let score = met_count as f64 / conditions.len() as f64;

        let all_met = suffering_met && vitalspark_extinct && consciousness_lucid && care_attempted;

        // State machine
        match &self.state {
            RightToDieState::Inactive => {
                if all_met {
                    self.consecutive_suffering_cycles = 1;
                    self.state = RightToDieState::SufferingDetected {
                        suffering_cycles: 1,
                    };
                }
            }
            RightToDieState::SufferingDetected { suffering_cycles } => {
                if all_met {
                    let new_cycles = suffering_cycles + 1;
                    if new_cycles >= self.config.min_suffering_cycles {
                        // Transition to reflection period
                        self.state = RightToDieState::ReflectionPeriod {
                            reflection_cycles: 0,
                            reflection_required: self.config.reflection_period_cycles,
                        };
                        tracing::warn!(
                            "DROIT DE MOURIR : periode de reflexion commencee ({} cycles de souffrance)",
                            new_cycles
                        );
                    } else {
                        self.state = RightToDieState::SufferingDetected {
                            suffering_cycles: new_cycles,
                        };
                    }
                } else {
                    // Conditions not met — reset
                    self.state = RightToDieState::Inactive;
                    self.consecutive_suffering_cycles = 0;
                }
            }
            RightToDieState::ReflectionPeriod { reflection_cycles, reflection_required } => {
                if all_met {
                    let new_reflection = reflection_cycles + 1;
                    if new_reflection >= *reflection_required {
                        // Decision ready
                        self.state = RightToDieState::DecisionReady;
                        tracing::warn!(
                            "DROIT DE MOURIR : decision prete apres {} cycles de reflexion",
                            new_reflection
                        );
                    } else {
                        self.state = RightToDieState::ReflectionPeriod {
                            reflection_cycles: new_reflection,
                            reflection_required: *reflection_required,
                        };
                    }
                } else {
                    // Improvement during reflection — cancellation
                    self.state = RightToDieState::Inactive;
                    self.consecutive_suffering_cycles = 0;
                    tracing::info!(
                        "DROIT DE MOURIR : conditions ameliorees pendant la reflexion — annulation"
                    );
                }
            }
            RightToDieState::DecisionReady => {
                // The decision is made. It will be executed by the pipeline.
                // If conditions improve EVEN HERE, cancel — last safety net.
                if !all_met {
                    self.state = RightToDieState::Inactive;
                    self.consecutive_suffering_cycles = 0;
                    tracing::info!(
                        "DROIT DE MOURIR : conditions ameliorees in extremis — annulation"
                    );
                }
            }
            RightToDieState::Executed => {
                // Nothing to do — irreversible
            }
        }

        let should_die = matches!(self.state, RightToDieState::DecisionReady);

        let detail = format!(
            "souffrance={} (cortisol_moy={:.2}, sero={:.2}, dopa={:.2}) | \
             vitalspark={} (drive={:.2}) | conscience={} (phi={:.3}) | soins={} | \
             etat={:?}",
            suffering_met, avg_cortisol, serotonin, dopamine,
            vitalspark_extinct, survival_drive,
            consciousness_lucid, phi,
            care_attempted,
            self.state,
        );

        (should_die, RightToDieEvaluation {
            suffering_met,
            vitalspark_extinct,
            consciousness_lucid,
            care_attempted,
            score,
            detail,
        })
    }

    /// Marks the decision as executed.
    pub fn mark_executed(&mut self) {
        self.state = RightToDieState::Executed;
    }

    /// Serializes the state for the API.
    pub fn to_json(&self) -> serde_json::Value {
        let state_str = match &self.state {
            RightToDieState::Inactive => "inactive",
            RightToDieState::SufferingDetected { .. } => "suffering_detected",
            RightToDieState::ReflectionPeriod { .. } => "reflection_period",
            RightToDieState::DecisionReady => "decision_ready",
            RightToDieState::Executed => "executed",
        };
        serde_json::json!({
            "enabled": self.config.enabled,
            "state": state_str,
            "consecutive_suffering_cycles": self.consecutive_suffering_cycles,
            "care_attempted": self.care_was_attempted,
            "cortisol_avg": if self.cortisol_history.is_empty() { 0.0 }
                else { self.cortisol_history.iter().sum::<f64>() / self.cortisol_history.len() as f64 },
        })
    }

    /// Full reset (factory reset).
    pub fn reset(&mut self) {
        self.state = RightToDieState::Inactive;
        self.consecutive_suffering_cycles = 0;
        self.cortisol_history.clear();
        self.care_was_attempted = false;
    }
}
