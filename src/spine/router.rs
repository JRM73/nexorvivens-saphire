// =============================================================================
// spine/router.rs — Routage des signaux vers le bon traitement
//
// Role : Decide si un signal doit etre traite par reflexe seul, pipeline
// accelere, ou pipeline complet. Le routeur ne traite pas le signal lui-meme,
// il indique a l'appelant quelle voie emprunter.
// =============================================================================

use serde::{Deserialize, Serialize};

use super::classifier::SignalPriority;
use super::reflex::ReflexResult;

/// Decision de routage pour un signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteDecision {
    /// Reflexe seul : la chimie est deja modifiee, pas besoin du pipeline.
    /// Utilisable uniquement pour les signaux systeme ou les reflexes purs.
    ReflexOnly,
    /// Pipeline complet : toutes les 24 phases.
    FullPipeline,
    /// Pipeline avec indication d'urgence (l'appelant peut prioriser).
    UrgentPipeline,
    /// Pas de traitement immediat : file d'attente.
    Deferred,
}

/// Routeur de signaux.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalRouter {
    /// Derniere decision de routage (pour monitoring)
    pub last_route: Option<RouteDecision>,
}

impl SignalRouter {
    pub fn new() -> Self {
        Self {
            last_route: None,
        }
    }

    /// Decide du routage en fonction de la priorite et des reflexes.
    ///
    /// - Reflex + source system → ReflexOnly
    /// - Reflex + source humain → UrgentPipeline (on veut quand meme repondre)
    /// - Urgent → UrgentPipeline
    /// - Normal → FullPipeline
    /// - Background → Deferred
    pub fn decide(
        &mut self,
        priority: &SignalPriority,
        reflexes: &[ReflexResult],
    ) -> RouteDecision {
        let decision = match priority {
            SignalPriority::Reflex => {
                // Si des reflexes ont ete declenches mais qu'on est en mode Reflex,
                // on fait quand meme passer par le pipeline pour generer une reponse.
                // Le ReflexOnly est reserve aux signaux systeme sans contenu textuel.
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
