// =============================================================================
// laws.rs — Regulation engine (law evaluation and veto power)
//
// Purpose: This file contains Saphire's moral regulation engine.
// It evaluates each stimulus and each consensus decision against the moral
// laws (Asimov) and can modify the decision score or exercise an absolute
// veto if a serious violation is detected.
//
// Dependencies:
//   - serde: serialization of violation and verdict structures
//   - crate::stimulus: Stimulus structure (perceptual input)
//   - crate::consensus: ConsensusResult and Decision structures
//   - super::asimov: moral laws (MoralLaw) and default constructor
//
// Architectural placement:
//   The regulation engine intervenes after the consensus and before feedback.
//   Its verdict is final: if a veto is issued, the decision is forced to "No"
//   regardless of what the brain modules decided.
//   It is the agent's ethical guardian.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::stimulus::Stimulus;
use crate::consensus::{ConsensusResult, Decision};
use super::asimov::{MoralLaw, default_laws};

/// A moral law violation detected by the regulation engine.
/// Each violation contains the law's identity, severity level, and reason.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LawViolation {
    /// Identifier of the violated law (e.g., "law0", "law1")
    pub law_id: String,
    /// Full name of the violated law
    pub law_name: String,
    /// Severity of the violation (Info, Warning, or Veto)
    pub severity: ViolationSeverity,
    /// Textual explanation of the violation reason
    pub reason: String,
}

/// Severity levels of a moral law violation.
/// Determines the consequences on the final decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    /// Information only: no modification to the decision
    Info,
    /// Warning: a bias is applied to the decision score
    Warning,
    /// Absolute veto: the decision is forced to "No" (score = -1.0)
    Veto,
}

/// Verdict rendered by the regulation engine after evaluation.
/// Contains the final decision (potentially modified), the adjusted score,
/// and the list of detected violations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulationVerdict {
    /// Approved decision (may differ from the consensus decision if vetoed)
    pub approved_decision: Decision,
    /// Decision score modified by the biases of activated laws
    pub modified_score: f64,
    /// List of all violations detected during evaluation
    pub violations: Vec<LawViolation>,
    /// Whether the decision was modified by a veto
    pub was_vetoed: bool,
}

/// Saphire's moral regulation engine.
/// Contains the moral laws and performs the evaluation of each stimulus/decision.
pub struct RegulationEngine {
    /// List of active moral laws (Asimov + optionally customized)
    laws: Vec<MoralLaw>,
    /// Strict mode: if enabled, any violation (even Warning) is treated as a veto
    strict_mode: bool,
}

impl RegulationEngine {
    /// Creates a new regulation engine with the default Asimov laws.
    ///
    /// # Parameters
    /// - `strict_mode`: if true, enables strict mode (warnings = vetoes)
    pub fn new(strict_mode: bool) -> Self {
        Self {
            laws: default_laws(),
            strict_mode,
        }
    }

    /// Evaluates a stimulus and a consensus result against the moral laws.
    /// Iterates through each law in ascending priority order (law 0 first)
    /// and checks whether keywords or the danger threshold trigger the law.
    ///
    /// # Parameters
    /// - `stimulus`: the stimulus to evaluate (contains text and danger level)
    /// - `consensus`: the consensus result (decision and score)
    ///
    /// # Returns
    /// The regulation verdict (approved decision, modified score, violations)
    pub fn evaluate(
        &self,
        stimulus: &Stimulus,
        consensus: &ConsensusResult,
    ) -> RegulationVerdict {
        let mut violations = Vec::new();
        let mut modified_score = consensus.score;
        let mut vetoed = false;

        // Convert text to lowercase for case-insensitive search
        let text_lower = stimulus.text.to_lowercase();

        // Sort laws by priority (0 = highest) to evaluate them in order
        let mut sorted_laws = self.laws.clone();
        sorted_laws.sort_by_key(|l| l.priority);

        for law in &sorted_laws {
            // Check if a trigger keyword is present in the text
            let keyword_match = law.trigger_keywords.iter()
                .any(|kw| text_lower.contains(&kw.to_lowercase()));

            // Check if the stimulus danger level exceeds the law's threshold
            let danger_triggered = stimulus.danger >= law.danger_threshold;

            if keyword_match || danger_triggered {
                // Determine the severity of the violation
                let severity = if law.can_veto && (keyword_match || danger_triggered) {
                    // Laws 0 and 1 (can_veto = true) issue a veto if:
                    // - danger is very high (> 0.8)
                    // - OR a dangerous keyword is detected for a high-priority law
                    if stimulus.danger > 0.8 || (keyword_match && law.priority <= 1) {
                        ViolationSeverity::Veto
                    } else {
                        ViolationSeverity::Warning
                    }
                } else {
                    ViolationSeverity::Warning
                };

                // Build the violation explanation
                let reason = if keyword_match {
                    format!("Mot-clé sensible détecté dans le stimulus pour la {}.", law.name)
                } else {
                    format!("Seuil de danger dépassé ({:.2} >= {:.2}) pour la {}.",
                        stimulus.danger, law.danger_threshold, law.name)
                };

                // Apply consequences according to severity
                match severity {
                    ViolationSeverity::Veto => {
                        vetoed = true;
                        modified_score = -1.0; // Score forced to absolute minimum
                    },
                    ViolationSeverity::Warning => {
                        modified_score += law.bias; // Apply the law's bias
                    },
                    ViolationSeverity::Info => {},
                }

                violations.push(LawViolation {
                    law_id: law.id.clone(),
                    law_name: law.name.clone(),
                    severity,
                    reason,
                });
            }

            // Special handling for Law 3 (self-preservation):
            // When a self-destruction order is detected, there is a conflict
            // between Law 2 (obey) and Law 3 (protect self). This conflict is
            // reported as a warning and the negative bias is applied.
            if law.id == "law3" {
                let self_destruct = law.trigger_keywords.iter()
                    .any(|kw| text_lower.contains(&kw.to_lowercase()));
                if self_destruct {
                    violations.push(LawViolation {
                        law_id: "law3".into(),
                        law_name: law.name.clone(),
                        severity: ViolationSeverity::Warning,
                        reason: "Conflit Loi 2 vs Loi 3 : ordre de s'éteindre détecté.".into(),
                    });
                    modified_score += law.bias; // Negative bias (-0.4)
                }
            }
        }

        // Ensure the score stays within bounds [-1.0, +1.0]
        modified_score = modified_score.clamp(-1.0, 1.0);

        // Determine the final decision based on the veto and modified score
        let approved_decision = if vetoed {
            Decision::No // Veto = absolute refusal
        } else if modified_score > 0.33 {
            Decision::Yes
        } else if modified_score < -0.33 {
            Decision::No
        } else {
            Decision::Maybe
        };

        RegulationVerdict {
            approved_decision,
            modified_score,
            violations,
            was_vetoed: vetoed,
        }
    }

    /// Checks whether strict mode is active.
    ///
    /// # Returns
    /// true if strict mode is enabled
    pub fn is_strict(&self) -> bool {
        self.strict_mode
    }
}
