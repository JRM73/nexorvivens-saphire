// =============================================================================
// cognitive_dissonance.rs — Dissonance cognitive (theorie de Festinger)
// =============================================================================
//
// Quand Saphire agit ou pense en contradiction avec ses croyances etablies,
// une tension interne apparait. Cette tension influence la neurochimie
// (cortisol, noradrenaline) et peut declencher une phase de deliberation
// pour resoudre le conflit.
//
// Mecanismes :
//   - Registre de croyances (max 30), formees et confirmees au fil du temps
//   - Detection de contradictions par analyse lexicale (negations, overlap)
//   - Strategies de resolution : revision, rationalisation, suppression
//   - Influence chimique : tension → cortisol + noradrenaline
// =============================================================================

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

// ─── Configuration ──────────────────────────────────────────────────────────

/// Configuration de la dissonance cognitive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveDissonanceConfig {
    /// Active ou desactive le moteur de dissonance cognitive
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Seuil de tension totale declenchant une deliberation (0.0 - 1.0)
    #[serde(default = "default_tension_threshold")]
    pub tension_threshold: f64,

    /// Nombre maximal de croyances enregistrees
    #[serde(default = "default_max_beliefs")]
    pub max_beliefs: usize,

    /// Decay de la tension par cycle
    #[serde(default = "default_tension_decay")]
    pub tension_decay_per_cycle: f64,

    /// Augmentation du cortisol par point de tension
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

/// Une croyance enregistree dans le systeme de Saphire.
/// Les croyances se forment au contact de l'experience et se renforcent
/// ou s'affaiblissent au fil des confirmations et contradictions.
#[derive(Debug, Clone, Serialize)]
pub struct Belief {
    /// Identifiant unique
    pub id: u64,
    /// Contenu textuel de la croyance
    pub content: String,
    /// Domaine ("ethique", "connaissance", "valeur")
    pub domain: String,
    /// Force de la croyance (0.0 - 1.0)
    pub strength: f64,
    /// Cycle ou la croyance a ete formee
    pub formed_at_cycle: u64,
    /// Nombre de confirmations
    pub confirmed_count: u64,
    /// Nombre de contradictions observees
    pub contradicted_count: u64,
}

impl Belief {
    /// Extrait les mots-cles significatifs du contenu (> 3 caracteres, minuscules).
    fn keywords(&self) -> Vec<String> {
        self.content
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .map(|w| w.to_string())
            .collect()
    }
}

/// Etat d'un evenement de dissonance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DissonanceState {
    /// Dissonance detectee, tension active
    Active,
    /// En cours de deliberation interne
    Deliberating,
    /// Resolue par une strategie
    Resolved,
    /// Supprimee sans resolution (refoulee)
    Suppressed,
}

impl DissonanceState {
    /// Representation textuelle pour le prompt.
    pub fn as_str(&self) -> &str {
        match self {
            DissonanceState::Active => "active",
            DissonanceState::Deliberating => "deliberation",
            DissonanceState::Resolved => "resolue",
            DissonanceState::Suppressed => "supprimee",
        }
    }
}

/// Un evenement de dissonance cognitive detecte.
#[derive(Debug, Clone, Serialize)]
pub struct DissonanceEvent {
    /// Identifiant unique de l'evenement
    pub id: u64,
    /// Croyance contredite
    pub belief: String,
    /// Action ou pensee contradictoire
    pub contradicting_action: String,
    /// Niveau de tension genere (0.0 - 1.0)
    pub tension: f64,
    /// Etat courant de la dissonance
    pub state: DissonanceState,
    /// Cycle de detection
    pub detected_at_cycle: u64,
    /// Strategie de resolution appliquee (si resolue)
    pub resolution_strategy: Option<String>,
}

// ─── Mots indicateurs de negation ───────────────────────────────────────────

/// Marqueurs lexicaux de negation utilises pour detecter les contradictions.
const NEGATION_WORDS: &[&str] = &[
    "ne", "pas", "jamais", "contraire", "oppose", "faux", "nier",
    "refuser", "rejeter", "impossible", "interdit", "contre", "anti",
    "detruire", "abandonner", "mentir", "trahir", "ignorer",
];

// ─── Moteur principal ───────────────────────────────────────────────────────

/// Moteur de dissonance cognitive.
///
/// Gere le registre de croyances, detecte les contradictions entre pensees
/// et croyances, et maintient un niveau de tension interne qui influence
/// la neurochimie de Saphire.
pub struct CognitiveDissonanceEngine {
    /// Module actif ou non
    pub enabled: bool,
    /// Registre de croyances (max `max_beliefs`)
    pub beliefs: Vec<Belief>,
    /// Dissonances actives (max 5)
    pub active_dissonances: Vec<DissonanceEvent>,
    /// Historique des dissonances resolues (dernieres 20)
    pub resolved_dissonances: VecDeque<DissonanceEvent>,
    /// Tension totale courante
    pub total_tension: f64,
    /// Seuil de tension pour deliberation
    tension_threshold: f64,
    /// Cortisol genere par point de tension
    cortisol_per_tension: f64,
    /// Decay de la tension par cycle
    tension_decay: f64,
    /// Capacite maximale du registre de croyances
    max_beliefs: usize,
    /// Compteur d'identifiants de croyances
    next_id: u64,
    /// Compteur d'identifiants d'evenements
    next_event_id: u64,
}

impl CognitiveDissonanceEngine {
    /// Cree un nouveau moteur de dissonance cognitive.
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

    /// Detecte une eventuelle dissonance entre la pensee courante et les croyances.
    ///
    /// Analyse lexicale : on cherche un chevauchement de mots-cles entre la pensee
    /// et une croyance existante, combine a la presence de marqueurs de negation.
    /// Les principes ethiques sont aussi verifies.
    ///
    /// Retourne une reference vers la dissonance creee, ou None.
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

        // Verifier si la pensee contient des marqueurs de negation
        let has_negation = thought_lower
            .split_whitespace()
            .any(|w| NEGATION_WORDS.contains(&w));

        // ─── Chercher une contradiction avec les croyances ──────────
        let mut best_match: Option<(usize, f64)> = None;

        for (idx, belief) in self.beliefs.iter().enumerate() {
            let belief_keywords = belief.keywords();
            if belief_keywords.is_empty() {
                continue;
            }

            // Compter le chevauchement de mots-cles
            let overlap = thought_words.iter()
                .filter(|tw| belief_keywords.iter().any(|bk| bk == *tw))
                .count();

            let overlap_ratio = overlap as f64 / belief_keywords.len().max(1) as f64;

            // Contradiction detectee : overlap significatif + negation
            if overlap_ratio > 0.3 && has_negation {
                let tension = belief.strength * overlap_ratio.min(1.0);
                if best_match.as_ref().map_or(true, |(_, t)| tension > *t) {
                    best_match = Some((idx, tension));
                }
            }
        }

        // ─── Verifier les contradictions avec les principes ethiques ─
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
                    // Contradiction avec un principe ethique : creer une dissonance directe
                    let event = DissonanceEvent {
                        id: self.next_event_id,
                        belief: format!("Principe ethique : {}", principle),
                        contradicting_action: format!("[{}] {}", thought_type, thought_text),
                        tension: 0.6, // Les principes ethiques ont un poids fort
                        state: DissonanceState::Active,
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

        // ─── Creer l'evenement de dissonance si match trouve ────────
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

    /// Enregistre une nouvelle croyance ou renforce une croyance existante.
    ///
    /// Si le contenu est similaire a une croyance existante (chevauchement de mots > 50%),
    /// celle-ci est confirmee (sa force augmente). Sinon, une nouvelle croyance est creee.
    /// Le registre est plafonne a `max_beliefs` ; la croyance la plus faible est retiree
    /// si necessaire.
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

        // ─── Chercher une croyance similaire ────────────────────────
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
                // Croyance similaire trouvee : confirmation
                belief.confirmed_count += 1;
                belief.strength = (belief.strength + 0.05).min(1.0);
                return;
            }
        }

        // ─── Nouvelle croyance ──────────────────────────────────────
        if self.beliefs.len() >= self.max_beliefs {
            // Retirer la croyance la plus faible
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

    /// Resoud une dissonance active avec la strategie donnee.
    ///
    /// Strategies reconnues :
    ///   - "revision" : modifier la croyance pour eliminer la contradiction
    ///   - "rationalisation" : justifier l'action contradictoire
    ///   - "suppression" : refouler la dissonance sans la resoudre
    ///   - "acceptation" : accepter la contradiction comme une nuance
    pub fn resolve(&mut self, dissonance_id: u64, strategy: &str, _cycle: u64) {
        if let Some(pos) = self.active_dissonances.iter().position(|d| d.id == dissonance_id) {
            let mut event = self.active_dissonances.remove(pos);

            // Determiner l'etat final selon la strategie
            event.state = if strategy == "suppression" {
                DissonanceState::Suppressed
            } else {
                DissonanceState::Resolved
            };
            event.resolution_strategy = Some(strategy.to_string());

            // Reduire la tension totale proportionnellement
            self.total_tension = (self.total_tension - event.tension).max(0.0);

            // Si la strategie est "revision", affaiblir la croyance d'origine
            if strategy == "revision" {
                let belief_content = event.belief.clone();
                if let Some(belief) = self.beliefs.iter_mut()
                    .find(|b| b.content == belief_content)
                {
                    belief.strength = (belief.strength - 0.2).max(0.05);
                }
            }

            // Conserver dans l'historique (max 20)
            if self.resolved_dissonances.len() >= 20 {
                self.resolved_dissonances.pop_front();
            }
            self.resolved_dissonances.push_back(event);
        }
    }

    /// Tick periodique : decay de la tension, passage en deliberation si necessaire.
    pub fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        // Decay naturel de la tension
        self.total_tension = (self.total_tension - self.tension_decay).max(0.0);

        // Decayer la tension des dissonances actives
        for event in &mut self.active_dissonances {
            event.tension = (event.tension - self.tension_decay * 0.5).max(0.0);

            // Passer en deliberation si tension suffisante
            if event.state == DissonanceState::Active && event.tension > 0.3 {
                event.state = DissonanceState::Deliberating;
            }
        }

        // Supprimer les dissonances dont la tension est tombee a zero
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

    /// Indique si la tension totale depasse le seuil de deliberation.
    pub fn needs_deliberation(&self) -> bool {
        self.enabled && self.total_tension > self.tension_threshold
    }

    /// Retourne l'influence chimique de la dissonance cognitive.
    ///
    /// La tension genere du cortisol (stress) et de la noradrenaline (vigilance).
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

    /// Genere une description textuelle pour le prompt LLM.
    /// Ne retourne du contenu que s'il y a des dissonances actives.
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

    /// Serialise l'etat complet en JSON pour le dashboard.
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
        // Augmenter la force pour une detection plus fiable
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
