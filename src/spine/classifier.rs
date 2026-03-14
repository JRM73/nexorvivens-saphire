// =============================================================================
// spine/classifier.rs — Classification des signaux par urgence
//
// Role : Trie chaque signal entrant par niveau d'urgence pour determiner
// le type de traitement qu'il recevra (reflexe seul, pipeline rapide, complet).
//
// 4 niveaux de priorite :
//   Reflex     — Reaction chimique seule, pas de LLM
//   Urgent     — Pipeline accelere (messages humains, alarmes)
//   Normal     — Pipeline complet (conversation, pensee autonome)
//   Background — File d'attente basse priorite (consolidation, reves)
// =============================================================================

use serde::{Deserialize, Serialize};

use super::reflex::{ReflexResult, ReflexType};

/// Niveau de priorite d'un signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SignalPriority {
    /// Basse priorite : consolidation memoire, taches de fond
    Background,
    /// Priorite normale : conversation, pensee autonome
    Normal,
    /// Priorite elevee : message humain, alarme corporelle
    Urgent,
    /// Priorite maximale : reflexe pur, pas besoin du pipeline
    Reflex,
}

/// Signal classifie avec sa priorite et ses metadonnees.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedSignal {
    /// Priorite assignee
    pub priority: SignalPriority,
    /// Source du signal
    pub source: String,
    /// Reflexes associes
    pub reflex_count: usize,
    /// Le signal contient-il un danger ?
    pub has_danger: bool,
    /// Le signal contient-il de l'affection ?
    pub has_affection: bool,
}

/// Classificateur de signaux.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalClassifier {
    /// Seuil de reflexes pour promouvoir en Urgent
    pub urgent_reflex_threshold: usize,
}

impl SignalClassifier {
    pub fn new() -> Self {
        Self {
            urgent_reflex_threshold: 2,
        }
    }

    /// Classifie un signal en fonction du texte, des reflexes declenches et de la source.
    ///
    /// Regles de classification :
    /// 1. Danger ou menace → Reflex (si seul reflexe) ou Urgent (si besoin du pipeline)
    /// 2. Message humain → au minimum Urgent
    /// 3. Message Sensoria → Normal (sauf si reflexes)
    /// 4. Pensee autonome → Normal
    /// 5. Systeme → Background
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

        // Danger ou menace intense : priorite maximale
        if has_danger && has_high_intensity {
            return SignalPriority::Reflex;
        }

        // Message humain : toujours au minimum Urgent
        if source == "human" {
            if has_danger {
                return SignalPriority::Reflex;
            }
            return SignalPriority::Urgent;
        }

        // Sensoria avec reflexes : Urgent
        if source == "sensoria" && !reflexes.is_empty() {
            return SignalPriority::Urgent;
        }

        // Beaucoup de reflexes : Urgent
        if reflexes.len() >= self.urgent_reflex_threshold {
            return SignalPriority::Urgent;
        }

        // Systeme : Background
        if source == "system" {
            return SignalPriority::Background;
        }

        // Defaut : Normal
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
