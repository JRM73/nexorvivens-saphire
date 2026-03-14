// =============================================================================
// spine/router.rs — Signal routing to the appropriate processing path
//
// Role: Decides whether a signal should be handled by reflex only,
// accelerated pipeline, or full pipeline. The router does not process the
// signal itself — it tells the caller which path to take.
// =============================================================================

use serde::{Deserialize, Serialize};

use super::classifier::SignalPriority;
use super::reflex::ReflexResult;

/// Routing decision for a signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteDecision {
    /// Reflex only: chemistry already modified, pipeline not needed.
    /// Only usable for system signals or pure reflexes.
    ReflexOnly,
    /// Full pipeline: all 24 phases.
    FullPipeline,
    /// Pipeline with urgency indication (caller may prioritize).
    UrgentPipeline,
    /// No immediate processing: queued.
    Deferred,
}

/// Signal router.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalRouter {
    /// Last routing decision (for monitoring)
    pub last_route: Option<RouteDecision>,
}

impl SignalRouter {
    pub fn new() -> Self {
        Self {
            last_route: None,
        }
    }

    /// Decides routing based on priority and reflexes.
    ///
    /// - Reflex + system source -> ReflexOnly
    /// - Reflex + human source -> UrgentPipeline (still want to respond)
    /// - Urgent -> UrgentPipeline
    /// - Normal -> FullPipeline
    /// - Background -> Deferred
    pub fn decide(
        &mut self,
        priority: &SignalPriority,
        reflexes: &[ReflexResult],
    ) -> RouteDecision {
        let decision = match priority {
            SignalPriority::Reflex => {
                // If reflexes were triggered but we are in Reflex mode,
                // still route through the pipeline to generate a response.
                // ReflexOnly is reserved for system signals with no text content.
                if reflexes.is_empty() {
                    RouteDecision::ReflexOnly
                } else {
                    RouteDecision::UrgentPipeline
                }
            }
            SignalPriority::Urgent => RouteDecision::UrgentPipeline,
            SignalPriority::Normal => RouteDecision::FullPipeline,
            SignalPriority::Background => RouteDecision::Deferred,
        };

        self.last_route = Some(decision);
        decision
    }
}

impl Default for SignalRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neurochemistry::Molecule;
    use super::super::reflex::{ReflexResult, ReflexType};

    fn make_reflex(rtype: ReflexType) -> ReflexResult {
        ReflexResult {
            reflex_type: rtype,
            intensity: 0.5,
            chemistry_deltas: vec![(Molecule::Cortisol, 0.1)],
            body_effects: vec![],
        }
    }

    #[test]
    fn test_route_urgent() {
        let mut router = SignalRouter::new();
        let decision = router.decide(&SignalPriority::Urgent, &[]);
        assert_eq!(decision, RouteDecision::UrgentPipeline);
    }

    #[test]
    fn test_route_normal() {
        let mut router = SignalRouter::new();
        let decision = router.decide(&SignalPriority::Normal, &[]);
        assert_eq!(decision, RouteDecision::FullPipeline);
    }

    #[test]
    fn test_route_background() {
        let mut router = SignalRouter::new();
        let decision = router.decide(&SignalPriority::Background, &[]);
        assert_eq!(decision, RouteDecision::Deferred);
    }

    #[test]
    fn test_route_reflex_with_results() {
        let mut router = SignalRouter::new();
        let reflexes = vec![make_reflex(ReflexType::DangerAlert)];
        let decision = router.decide(&SignalPriority::Reflex, &reflexes);
        assert_eq!(decision, RouteDecision::UrgentPipeline);
    }

    #[test]
    fn test_route_reflex_no_results() {
        let mut router = SignalRouter::new();
        let decision = router.decide(&SignalPriority::Reflex, &[]);
        assert_eq!(decision, RouteDecision::ReflexOnly);
    }
}
