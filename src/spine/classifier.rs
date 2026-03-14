// =============================================================================
// spine/classifier.rs — Signal classification by urgency
//
// Role: Sorts each incoming signal by urgency level to determine
// the type of processing it will receive (reflex only, fast pipeline, full).
//
// 4 priority levels:
//   Reflex     — Chemical reaction only, no LLM
//   Urgent     — Accelerated pipeline (human messages, alarms)
//   Normal     — Full pipeline (conversation, autonomous thought)
//   Background — Low-priority queue (consolidation, dreams)
// =============================================================================

use serde::{Deserialize, Serialize};

use super::reflex::{ReflexResult, ReflexType};

/// Signal priority level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SignalPriority {
    /// Low priority: memory consolidation, background tasks
    Background,
    /// Normal priority: conversation, autonomous thought
    Normal,
    /// High priority: human message, body alarm
    Urgent,
    /// Maximum priority: pure reflex, pipeline not needed
    Reflex,
}

/// Classified signal with its priority and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedSignal {
    /// Assigned priority
    pub priority: SignalPriority,
    /// Signal source
    pub source: String,
    /// Associated reflexes
    pub reflex_count: usize,
    /// Does the signal contain danger?
    pub has_danger: bool,
    /// Does the signal contain affection?
    pub has_affection: bool,
}

/// Signal classifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalClassifier {
    /// Reflex threshold for promotion to Urgent
    pub urgent_reflex_threshold: usize,
}

impl SignalClassifier {
    pub fn new() -> Self {
        Self {
            urgent_reflex_threshold: 2,
        }
    }

    /// Classifies a signal based on text, triggered reflexes, and source.
    ///
    /// Classification rules:
    /// 1. Danger or threat -> Reflex (if sole reflex) or Urgent (if pipeline needed)
    /// 2. Human message -> at least Urgent
    /// 3. Sensoria message -> Normal (unless reflexes triggered)
    /// 4. Autonomous thought -> Normal
    /// 5. System -> Background
    pub fn classify(
        &self,
        _text: &str,
        reflexes: &[ReflexResult],
        source: &str,
    ) -> SignalPriority {
        let has_danger = reflexes.iter().any(|r| {
            matches!(r.reflex_type, ReflexType::DangerAlert | ReflexType::ThreatResponse)
        });

        let has_high_intensity = reflexes.iter().any(|r| r.intensity > 0.7);

        // Danger or intense threat: maximum priority
        if has_danger && has_high_intensity {
            return SignalPriority::Reflex;
        }

        // Human message: always at least Urgent
        if source == "human" {
            if has_danger {
                return SignalPriority::Reflex;
            }
            return SignalPriority::Urgent;
        }

        // Sensoria with reflexes: Urgent
        if source == "sensoria" && !reflexes.is_empty() {
            return SignalPriority::Urgent;
        }

        // Many reflexes: Urgent
        if reflexes.len() >= self.urgent_reflex_threshold {
            return SignalPriority::Urgent;
        }

        // System: Background
        if source == "system" {
            return SignalPriority::Background;
        }

        // Default: Normal
        SignalPriority::Normal
    }
}

impl Default for SignalClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neurochemistry::Molecule;
    use super::super::reflex::{ReflexResult, ReflexType, BodyEffect};

    fn make_reflex(rtype: ReflexType, intensity: f64) -> ReflexResult {
        ReflexResult {
            reflex_type: rtype,
            intensity,
            chemistry_deltas: vec![(Molecule::Cortisol, 0.1)],
            body_effects: vec![],
        }
    }

    #[test]
    fn test_classify_human_message() {
        let classifier = SignalClassifier::new();
        let priority = classifier.classify("bonjour", &[], "human");
        assert_eq!(priority, SignalPriority::Urgent);
    }

    #[test]
    fn test_classify_danger() {
        let classifier = SignalClassifier::new();
        let reflexes = vec![make_reflex(ReflexType::DangerAlert, 0.8)];
        let priority = classifier.classify("danger !", &reflexes, "human");
        assert_eq!(priority, SignalPriority::Reflex);
    }

    #[test]
    fn test_classify_autonomous() {
        let classifier = SignalClassifier::new();
        let priority = classifier.classify("pensee libre", &[], "autonomous");
        assert_eq!(priority, SignalPriority::Normal);
    }

    #[test]
    fn test_classify_system() {
        let classifier = SignalClassifier::new();
        let priority = classifier.classify("tick", &[], "system");
        assert_eq!(priority, SignalPriority::Background);
    }

    #[test]
    fn test_classify_sensoria_with_reflexes() {
        let classifier = SignalClassifier::new();
        let reflexes = vec![make_reflex(ReflexType::WarmthResponse, 0.5)];
        let priority = classifier.classify("je t'aime", &reflexes, "sensoria");
        assert_eq!(priority, SignalPriority::Urgent);
    }
}
