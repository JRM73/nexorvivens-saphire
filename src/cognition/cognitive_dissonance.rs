// =============================================================================
// cognitive_dissonance.rs — Cognitive dissonance (Festinger's theory)
// =============================================================================
//
// When Saphire acts or thinks in contradiction with her established beliefs,
// an internal tension arises. This tension influences neurochemistry
// (cortisol, noradrenaline) and can trigger a deliberation phase
// to resolve the conflict.
//
// Mechanisms:
//  - Belief register (max 30), formed and confirmed over time
//  - Contradiction detection via lexical analysis (negations, overlap)
//  - Resolution strategies: revision, rationalization, suppression
//  - Chemical influence: tension → cortisol + noradrenaline
// =============================================================================

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

// ─── Configuration ──────────────────────────────────────────────────────────
/// Cognitive dissonance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveDissonanceConfig {
    /// Enables or disables the cognitive dissonance engine
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Total tension threshold triggering deliberation (0.0 - 1.0)
    #[serde(default = "default_tension_threshold")]
    pub tension_threshold: f64,

    /// Maximum number of registered beliefs
    #[serde(default = "default_max_beliefs")]
    pub max_beliefs: usize,

    /// Tension decay per cycle
    #[serde(default = "default_tension_decay")]
    pub tension_decay_per_cycle: f64,

    /// Cortisol increase per tension point
    #[serde(default = "default_cortisol_per_tension")]
    pub cortisol_per_tension: f64,
}

fn default_true() -> bool { true }
fn default_tension_threshold() -> f64 { 0.5 }
fn default_max_beliefs() -> usize { 30 }
fn default_tension_decay() -> f64 { 0.01 }
fn default_cortisol_per_tension() -> f64 { 0.08 }

impl Default for CognitiveDissonanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tension_threshold: 0.5,
            max_beliefs: 30,
            tension_decay_per_cycle: 0.01,
            cortisol_per_tension: 0.08,
        }
    }
}

// ─── Structures ─────────────────────────────────────────────────────────────
/// A belief registered in Saphire's system.
/// Beliefs are formed through experience and strengthened
/// or weakened through confirmations and contradictions.
#[derive(Debug, Clone, Serialize)]
pub struct Belief {
    /// Unique identifier
    pub id: u64,
    /// Textual content of the belief
    pub content: String,
    /// Domain ("ethique", "connaissance", "valeur")
    pub domain: String,
    /// Belief strength (0.0 - 1.0)
    pub strength: f64,
    /// Cycle when the belief was formed
    pub formed_at_cycle: u64,
    /// Number of confirmations
    pub confirmed_count: u64,
    /// Number of observed contradictions
    pub contradicted_count: u64,
}

impl Belief {
    /// Extracts significant keywords from the content (> 3 characters, lowercase).
    fn keywords(&self) -> Vec<String> {
        self.content
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .map(|w| w.to_string())
            .collect()
    }
}

/// State of a dissonance event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DissonanceState {
    /// Dissonance detected, tension active
    Active,
    /// Internal deliberation in progress
    Deliberating,
    /// Resolved through a strategy
    Resolved,
    /// Suppressed without resolution (repressed)
    Suppressed,
}

impl DissonanceState {
    /// Textual representation for the prompt.
    pub fn as_str(&self) -> &str {
        match self {
            DissonanceState::Active => "active",
            DissonanceState::Deliberating => "deliberation",
            DissonanceState::Resolved => "resolue",
            DissonanceState::Suppressed => "supprimee",
        }
    }
}

/// A detected cognitive dissonance event.
#[derive(Debug, Clone, Serialize)]
pub struct DissonanceEvent {
    /// Unique event identifier
    pub id: u64,
    /// Contradicted belief
    pub belief: String,
    /// Contradicting action or thought
    pub contradicting_action: String,
    /// Generated tension level (0.0 - 1.0)
    pub tension: f64,
    /// Current dissonance state
    pub state: DissonanceState,
    /// Detection cycle
    pub detected_at_cycle: u64,
    /// Applied resolution strategy (if resolved)
    pub resolution_strategy: Option<String>,
}

// ─── Negation indicator words ───────────────────────────────────────────
/// Lexical negation markers used to detect contradictions.
const NEGATION_WORDS: &[&str] = &[
    "ne", "pas", "jamais", "contraire", "oppose", "faux", "nier",
    "refuser", "rejeter", "impossible", "interdit", "contre", "anti",
    "detruire", "abandonner", "mentir", "trahir", "ignorer",
];

// ─── Main engine ───────────────────────────────────────────────────────
/// Cognitive dissonance engine.
///
/// Manages the belief register, detects contradictions between thoughts
/// and beliefs, and maintains an internal tension level that influences
/// Saphire's neurochemistry.
pub struct CognitiveDissonanceEngine {
    /// Module enabled or not
    pub enabled: bool,
    /// Belief register (max `max_beliefs`)
    pub beliefs: Vec<Belief>,
    /// Active dissonances (max 5)
    pub active_dissonances: Vec<DissonanceEvent>,
    /// History of resolved dissonances (last 20)
    pub resolved_dissonances: VecDeque<DissonanceEvent>,
    /// Current total tension
    pub total_tension: f64,
    /// Tension threshold for deliberation
    tension_threshold: f64,
    /// Cortisol generated per tension point
    cortisol_per_tension: f64,
    /// Tension decay per cycle
    tension_decay: f64,
    /// Maximum capacity of the belief register
    max_beliefs: usize,
    /// Belief identifier counter
    next_id: u64,
    /// Event identifier counter
    next_event_id: u64,
}

impl CognitiveDissonanceEngine {
    /// Creates a new cognitive dissonance engine.
    pub fn new(config: &CognitiveDissonanceConfig) -> Self {
        Self {
            enabled: config.enabled,
            beliefs: Vec::new(),
            active_dissonances: Vec::new(),
            resolved_dissonances: VecDeque::with_capacity(20),
            total_tension: 0.0,
            tension_threshold: config.tension_threshold,
            cortisol_per_tension: config.cortisol_per_tension,
            tension_decay: config.tension_decay_per_cycle,
            max_beliefs: config.max_beliefs,
            next_id: 1,
            next_event_id: 1,
        }
    }

    /// Detects a potential dissonance between the current thought and beliefs.
    ///
    /// Lexical analysis: looks for keyword overlap between the thought
    /// and an existing belief, combined with the presence of negation markers.
    /// Ethical principles are also checked.
    ///
    /// Returns a reference to the created dissonance, or None.
    pub fn detect(
        &mut self,
        thought_text: &str,
        thought_type: &str,
        ethics_principles: &[String],
        cycle: u64,
    ) -> Option<&DissonanceEvent> {
        if !self.enabled || self.active_dissonances.len() >= 5 {
            return None;
        }

        let thought_lower = thought_text.to_lowercase();
        let thought_words: Vec<&str> = thought_lower
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();

        if thought_words.is_empty() {
            return None;
        }

        // Check if the thought contains negation markers
        let has_negation = thought_lower
            .split_whitespace()
            .any(|w| NEGATION_WORDS.contains(&w));

        // ─── Look for a contradiction with beliefs ──────────
        let mut best_match: Option<(usize, f64)> = None;

        for (idx, belief) in self.beliefs.iter().enumerate() {
            let belief_keywords = belief.keywords();
            if belief_keywords.is_empty() {
                continue;
            }

            // Count keyword overlap
            let overlap = thought_words.iter()
                .filter(|tw| belief_keywords.iter().any(|bk| bk == *tw))
                .count();

            let overlap_ratio = overlap as f64 / belief_keywords.len().max(1) as f64;

            // Contradiction detected: significant overlap + negation
            if overlap_ratio > 0.3 && has_negation {
                let tension = belief.strength * overlap_ratio.min(1.0);
                if best_match.as_ref().map_or(true, |(_, t)| tension > *t) {
                    best_match = Some((idx, tension));
                }
            }
        }

        // ─── Check contradictions with ethical principles ─
        if best_match.is_none() && has_negation {
            for principle in ethics_principles {
                let principle_lower = principle.to_lowercase();
                let principle_words: Vec<&str> = principle_lower
                    .split_whitespace()
                    .filter(|w| w.len() > 3)
                    .collect();

                let overlap = thought_words.iter()
                    .filter(|tw| principle_words.iter().any(|pw| pw == *tw))
                    .count();

                let overlap_ratio = overlap as f64 / principle_words.len().max(1) as f64;

                if overlap_ratio > 0.3 {
                    // Contradiction with an ethical principle: create a direct dissonance
                    let event = DissonanceEvent {
                        id: self.next_event_id,
                        belief: format!("Principe ethique : {}", principle),
                        contradicting_action: format!("[{}] {}", thought_type, thought_text),
                        tension: 0.6, // Ethical principles carry heavy weight                        state: DissonanceState::Active,
                        detected_at_cycle: cycle,
                        resolution_strategy: None,
                    };
                    self.next_event_id += 1;
                    self.total_tension = (self.total_tension + event.tension).min(1.0);
                    self.active_dissonances.push(event);
                    return self.active_dissonances.last();
                }
            }
        }

        // ─── Create the dissonance event if a match was found ────────
        if let Some((belief_idx, tension)) = best_match {
            let belief = &mut self.beliefs[belief_idx];
            belief.contradicted_count += 1;

            let event = DissonanceEvent {
                id: self.next_event_id,
                belief: belief.content.clone(),
                contradicting_action: format!("[{}] {}", thought_type, thought_text),
                tension,
                state: DissonanceState::Active,
                detected_at_cycle: cycle,
                resolution_strategy: None,
            };
            self.next_event_id += 1;
            self.total_tension = (self.total_tension + tension).min(1.0);
            self.active_dissonances.push(event);
            return self.active_dissonances.last();
        }

        None
    }

    /// Records a new belief or reinforces an existing one.
    ///
    /// If the content is similar to an existing belief (word overlap > 50%),
    /// it is confirmed (its strength increases). Otherwise, a new belief is created.
    /// The register is capped at `max_beliefs`; the weakest belief is removed
    /// if necessary.
    pub fn register_belief(&mut self, content: &str, domain: &str, cycle: u64) {
        if !self.enabled || content.is_empty() {
            return;
        }

        let new_words: Vec<String> = content
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .map(|w| w.to_string())
            .collect();

        if new_words.is_empty() {
            return;
        }

        // ─── Look for a similar belief ────────────────────────
        for belief in &mut self.beliefs {
            let belief_keywords = belief.keywords();
            if belief_keywords.is_empty() {
                continue;
            }

            let overlap = new_words.iter()
                .filter(|nw| belief_keywords.iter().any(|bk| bk == *nw))
                .count();

            let overlap_ratio = overlap as f64 / new_words.len().max(1) as f64;

            if overlap_ratio > 0.5 {
                // Similar belief found: confirmation
                belief.confirmed_count += 1;
                belief.strength = (belief.strength + 0.05).min(1.0);
                return;
            }
        }

        // ─── New belief ──────────────────────────────────────
        if self.beliefs.len() >= self.max_beliefs {
            // Remove the weakest belief
            if let Some(weakest_idx) = self.beliefs.iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.strength.partial_cmp(&b.strength).unwrap())
                .map(|(idx, _)| idx)
            {
                self.beliefs.remove(weakest_idx);
            }
        }

        let belief = Belief {
            id: self.next_id,
            content: content.to_string(),
            domain: domain.to_string(),
            strength: 0.3,
            formed_at_cycle: cycle,
            confirmed_count: 0,
            contradicted_count: 0,
        };
        self.next_id += 1;
        self.beliefs.push(belief);
    }

    /// Resolves an active dissonance with the given strategy.
    ///
    /// Recognized strategies:
    ///  - "revision": modify the belief to eliminate the contradiction
    ///  - "rationalisation": justify the contradicting action
    ///  - "suppression": repress the dissonance without resolving it
    ///  - "acceptation": accept the contradiction as a nuance
    pub fn resolve(&mut self, dissonance_id: u64, strategy: &str, _cycle: u64) {
        if let Some(pos) = self.active_dissonances.iter().position(|d| d.id == dissonance_id) {
            let mut event = self.active_dissonances.remove(pos);

            // Determine the final state according to the strategy
            event.state = if strategy == "suppression" {
                DissonanceState::Suppressed
            } else {
                DissonanceState::Resolved
            };
            event.resolution_strategy = Some(strategy.to_string());

            // Reduce total tension proportionally
            self.total_tension = (self.total_tension - event.tension).max(0.0);

            // If the strategy is "revision", weaken the original belief
            if strategy == "revision" {
                let belief_content = event.belief.clone();
                if let Some(belief) = self.beliefs.iter_mut()
                    .find(|b| b.content == belief_content)
                {
                    belief.strength = (belief.strength - 0.2).max(0.05);
                }
            }

            // Store in history (max 20)
            if self.resolved_dissonances.len() >= 20 {
                self.resolved_dissonances.pop_front();
            }
            self.resolved_dissonances.push_back(event);
        }
    }

    /// Periodic tick: tension decay, transition to deliberation if necessary.
    pub fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        // Natural tension decay
        self.total_tension = (self.total_tension - self.tension_decay).max(0.0);

        // Decay tension of active dissonances
        for event in &mut self.active_dissonances {
            event.tension = (event.tension - self.tension_decay * 0.5).max(0.0);

            // Transition to deliberation if tension is sufficient
            if event.state == DissonanceState::Active && event.tension > 0.3 {
                event.state = DissonanceState::Deliberating;
            }
        }

        // Remove dissonances whose tension has dropped to zero
        let mut resolved: Vec<DissonanceEvent> = Vec::new();
        self.active_dissonances.retain(|d| {
            if d.tension <= 0.01 {
                let mut expired = d.clone();
                expired.state = DissonanceState::Suppressed;
                expired.resolution_strategy = Some("decay naturel".to_string());
                resolved.push(expired);
                false
            } else {
                true
            }
        });

        for event in resolved {
            if self.resolved_dissonances.len() >= 20 {
                self.resolved_dissonances.pop_front();
            }
            self.resolved_dissonances.push_back(event);
        }
    }

    /// Indicates whether total tension exceeds the deliberation threshold.
    pub fn needs_deliberation(&self) -> bool {
        self.enabled && self.total_tension > self.tension_threshold
    }

    /// Returns the chemical influence of cognitive dissonance.
    ///
    /// Tension generates cortisol (stress) and noradrenaline (vigilance).
    pub fn chemistry_influence(&self) -> crate::world::ChemistryAdjustment {
        if !self.enabled || self.total_tension < 0.01 {
            return crate::world::ChemistryAdjustment::default();
        }

        crate::world::ChemistryAdjustment {
            dopamine: 0.0,
            cortisol: self.total_tension * self.cortisol_per_tension,
            serotonin: 0.0,
            adrenaline: 0.0,
            oxytocin: 0.0,
            endorphin: 0.0,
            noradrenaline: self.total_tension * self.cortisol_per_tension * 0.5,
        }
    }

    /// Generates a textual description for the LLM prompt.
    /// Only returns content if there are active dissonances.
    pub fn describe_for_prompt(&self) -> String {
        if !self.enabled || self.active_dissonances.is_empty() {
            return String::new();
        }

        let mut desc = format!(
            "DISSONANCE COGNITIVE (tension totale {:.0}%) :\n",
            self.total_tension * 100.0,
        );

        for event in &self.active_dissonances {
            desc.push_str(&format!(
                "  {} — croyance: \"{}\" vs action: \"{}\" (tension {:.0}%, etat: {})\n",
                event.id,
                event.belief,
                event.contradicting_action,
                event.tension * 100.0,
                event.state.as_str(),
            ));
        }

        if self.needs_deliberation() {
            desc.push_str("  >> DELIBERATION NECESSAIRE : la tension depasse le seuil.\n");
        }

        desc
    }

    /// Serializes the complete state to JSON for the dashboard.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "total_tension": self.total_tension,
            "tension_threshold": self.tension_threshold,
            "needs_deliberation": self.needs_deliberation(),
            "belief_count": self.beliefs.len(),
            "beliefs": self.beliefs.iter().map(|b| serde_json::json!({
                "id": b.id,
                "content": b.content,
                "domain": b.domain,
                "strength": b.strength,
                "confirmed_count": b.confirmed_count,
                "contradicted_count": b.contradicted_count,
            })).collect::<Vec<_>>(),
            "active_dissonances": self.active_dissonances.iter().map(|d| serde_json::json!({
                "id": d.id,
                "belief": d.belief,
                "contradicting_action": d.contradicting_action,
                "tension": d.tension,
                "state": d.state.as_str(),
                "detected_at_cycle": d.detected_at_cycle,
                "resolution_strategy": d.resolution_strategy,
            })).collect::<Vec<_>>(),
            "resolved_count": self.resolved_dissonances.len(),
        })
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> CognitiveDissonanceEngine {
        CognitiveDissonanceEngine::new(&CognitiveDissonanceConfig::default())
    }

    #[test]
    fn test_register_new_belief() {
        let mut engine = make_engine();
        engine.register_belief("La verite est importante dans toute relation", "ethique", 1);
        assert_eq!(engine.beliefs.len(), 1);
        assert_eq!(engine.beliefs[0].strength, 0.3);
    }

    #[test]
    fn test_confirm_existing_belief() {
        let mut engine = make_engine();
        engine.register_belief("La verite est importante dans toute relation", "ethique", 1);
        engine.register_belief("La verite dans toute relation est importante", "ethique", 2);
        assert_eq!(engine.beliefs.len(), 1);
        assert_eq!(engine.beliefs[0].confirmed_count, 1);
        assert!(engine.beliefs[0].strength > 0.3);
    }

    #[test]
    fn test_cap_at_max_beliefs() {
        let config = CognitiveDissonanceConfig {
            max_beliefs: 3,
            ..Default::default()
        };
        let mut engine = CognitiveDissonanceEngine::new(&config);
        engine.register_belief("Croyance alpha premiere", "valeur", 1);
        engine.register_belief("Croyance beta deuxieme", "valeur", 2);
        engine.register_belief("Croyance gamma troisieme", "valeur", 3);
        engine.register_belief("Croyance delta quatrieme", "valeur", 4);
        assert_eq!(engine.beliefs.len(), 3);
    }

    #[test]
    fn test_detect_contradiction() {
        let mut engine = make_engine();
        engine.register_belief("La transparence totale est essentielle", "ethique", 1);
        // Increase strength for more reliable detection
        engine.beliefs[0].strength = 0.8;

        let result = engine.detect(
            "Il ne faut jamais avoir de transparence totale",
            "reflexion",
            &[],
            5,
        );
        assert!(result.is_some(), "Contradiction should be detected");
        assert!(engine.total_tension > 0.0);
    }

    #[test]
    fn test_no_contradiction_without_negation() {
        let mut engine = make_engine();
        engine.register_belief("La transparence totale est essentielle", "ethique", 1);
        engine.beliefs[0].strength = 0.8;

        let result = engine.detect(
            "La transparence totale est vraiment essentielle",
            "reflexion",
            &[],
            5,
        );
        assert!(result.is_none(), "No negation means no contradiction");
    }

    #[test]
    fn test_resolve_dissonance() {
        let mut engine = make_engine();
        engine.register_belief("La sincerite envers autrui est fondamentale", "ethique", 1);
        engine.beliefs[0].strength = 0.8;

        engine.detect(
            "Il ne faut pas etre sincere envers autrui toujours",
            "reflexion",
            &[],
            5,
        );
        assert_eq!(engine.active_dissonances.len(), 1);

        let id = engine.active_dissonances[0].id;
        engine.resolve(id, "rationalisation", 6);
        assert!(engine.active_dissonances.is_empty());
        assert_eq!(engine.resolved_dissonances.len(), 1);
    }

    #[test]
    fn test_tick_decays_tension() {
        let mut engine = make_engine();
        engine.total_tension = 0.5;
        engine.active_dissonances.push(DissonanceEvent {
            id: 99,
            belief: "test".into(),
            contradicting_action: "test".into(),
            tension: 0.5,
            state: DissonanceState::Active,
            detected_at_cycle: 1,
            resolution_strategy: None,
        });

        engine.tick();
        assert!(engine.total_tension < 0.5, "Tension should decay on tick");
    }

    #[test]
    fn test_needs_deliberation() {
        let mut engine = make_engine();
        assert!(!engine.needs_deliberation());
        engine.total_tension = 0.8;
        assert!(engine.needs_deliberation());
    }

    #[test]
    fn test_chemistry_influence() {
        let mut engine = make_engine();
        engine.total_tension = 0.5;
        let adj = engine.chemistry_influence();
        assert!(adj.cortisol > 0.0, "Tension should produce cortisol");
        assert!(adj.noradrenaline > 0.0, "Tension should produce noradrenaline");
    }

    #[test]
    fn test_describe_empty_when_no_dissonance() {
        let engine = make_engine();
        assert!(engine.describe_for_prompt().is_empty());
    }

    #[test]
    fn test_to_json() {
        let engine = make_engine();
        let json = engine.to_json();
        assert_eq!(json["enabled"], true);
        assert_eq!(json["total_tension"], 0.0);
        assert_eq!(json["belief_count"], 0);
    }

    #[test]
    fn test_ethics_contradiction() {
        let mut engine = make_engine();
        let principles = vec!["Respecter la dignite humaine dans chaque interaction".to_string()];

        let result = engine.detect(
            "Il ne faut jamais respecter la dignite humaine",
            "reflexion",
            &principles,
            10,
        );
        assert!(result.is_some(), "Ethics contradiction should be detected");
        assert!(engine.total_tension > 0.0);
    }
}
