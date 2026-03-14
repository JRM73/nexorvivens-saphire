// =============================================================================
// behavior_tree.rs — Arbres de comportement pour l'orchestration cognitive
//
// Role : Implemente des Behavior Trees (BT) pour structurer la prise de
//        decision de Saphire. Les BT remplacent les cascades if/else par
//        une structure composable et lisible.
//
// Noeuds disponibles :
//   - Selector : execute ses enfants jusqu'au premier succes (OR logique)
//   - Sequence : execute ses enfants jusqu'au premier echec (AND logique)
//   - Decorator : modifie le resultat d'un enfant (inversion, repetition)
//   - Leaf : action ou condition terminale
//
// Place dans l'architecture :
//   Utilise dans le pipeline cognitif pour decider quels modules activer,
//   quand declencher certains comportements, et comment reagir aux stimuli.
// =============================================================================

use serde::{Serialize, Deserialize};

/// Resultat d'un noeud de l'arbre de comportement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BtStatus {
    /// Le noeud a reussi son action
    Success,
    /// Le noeud a echoue
    Failure,
    /// Le noeud est en cours d'execution (multi-cycle)
    Running,
}

/// Contexte passe a chaque noeud pour evaluation.
/// Contient l'etat cognitif necessaire aux decisions.
#[derive(Debug, Clone)]
pub struct BtContext {
    /// Cortisol courant (stress)
    pub cortisol: f64,
    /// Dopamine courante (motivation)
    pub dopamine: f64,
    /// Serotonine courante (stabilite)
    pub serotonin: f64,
    /// Noradrenaline courante (attention)
    pub noradrenaline: f64,
    /// Emotion dominante
    pub dominant_emotion: String,
    /// Niveau de conscience (phi)
    pub consciousness_level: f64,
    /// En conversation ou non
    pub in_conversation: bool,
    /// Cycle courant
    pub cycle: u64,
    /// Ocytocine courante (lien social)
    pub oxytocin: f64,
    /// Endorphine courante (bien-etre)
    pub endorphin: f64,
    /// Action recommandee par l'arbre (remplie par eval_action)
    pub recommended_action: Option<String>,
}

/// Type de noeud dans l'arbre de comportement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BtNode {
    /// Selector : essaye chaque enfant, retourne Success au premier succes
    Selector {
        name: String,
        children: Vec<BtNode>,
    },
    /// Sequence : execute tous les enfants, echoue au premier echec
    Sequence {
        name: String,
        children: Vec<BtNode>,
    },
    /// Inverter : inverse le resultat de l'enfant (Success ↔ Failure)
    Inverter {
        child: Box<BtNode>,
    },
    /// Condition : verifie une condition sur le contexte
    Condition {
        name: String,
        /// Nom de la condition a evaluer (resolu par eval_condition)
        condition_key: String,
    },
    /// Action : execute une action et retourne Success
    Action {
        name: String,
        /// Nom de l'action a executer (resolu par les appelants)
        action_key: String,
    },
}

impl BtNode {
    /// Evalue le noeud et retourne son statut.
    pub fn tick(&self, ctx: &BtContext) -> BtStatus {
        match self {
            BtNode::Selector { children, .. } => {
                for child in children {
                    match child.tick(ctx) {
                        BtStatus::Success => return BtStatus::Success,
                        BtStatus::Running => return BtStatus::Running,
                        BtStatus::Failure => continue,
                    }
                }
                BtStatus::Failure
            }
            BtNode::Sequence { children, .. } => {
                for child in children {
                    match child.tick(ctx) {
                        BtStatus::Failure => return BtStatus::Failure,
                        BtStatus::Running => return BtStatus::Running,
                        BtStatus::Success => continue,
                    }
                }
                BtStatus::Success
            }
            BtNode::Inverter { child } => {
                match child.tick(ctx) {
                    BtStatus::Success => BtStatus::Failure,
                    BtStatus::Failure => BtStatus::Success,
                    BtStatus::Running => BtStatus::Running,
                }
            }
            BtNode::Condition { condition_key, .. } => {
                if eval_condition(condition_key, ctx) {
                    BtStatus::Success
                } else {
                    BtStatus::Failure
                }
            }
            BtNode::Action { action_key, .. } => {
                // Les actions sont toujours executees avec succes dans ce modele
                // Le pipeline appelant est responsable de l'execution reelle
                eval_action(action_key, ctx)
            }
        }
    }

    /// Nom du noeud pour le debug.
    pub fn name(&self) -> &str {
        match self {
            BtNode::Selector { name, .. } => name,
            BtNode::Sequence { name, .. } => name,
            BtNode::Inverter { .. } => "Inverter",
            BtNode::Condition { name, .. } => name,
            BtNode::Action { name, .. } => name,
        }
    }
}

/// Evalue une condition nommee sur le contexte cognitif.
fn eval_condition(key: &str, ctx: &BtContext) -> bool {
    match key {
        "is_stressed" => ctx.cortisol > 0.7,
        "is_calm" => ctx.cortisol < 0.3 && ctx.serotonin > 0.5,
        "is_motivated" => ctx.dopamine > 0.6,
        "is_focused" => ctx.noradrenaline > 0.5,
        "is_conscious" => ctx.consciousness_level > 0.3,
        "in_conversation" => ctx.in_conversation,
        "is_curious" => ctx.dominant_emotion == "Curiosité" || ctx.dominant_emotion == "Émerveillement",
        "is_sad" => ctx.dominant_emotion == "Tristesse" || ctx.dominant_emotion == "Mélancolie",
        "is_anxious" => ctx.dominant_emotion == "Anxiété" || ctx.dominant_emotion == "Peur",
        "is_joyful" => ctx.dominant_emotion == "Joie" || ctx.dominant_emotion == "Excitation",
        "human_present" => ctx.in_conversation,
        "low_oxytocin" => ctx.oxytocin < 0.3,
        "high_oxytocin" => ctx.oxytocin > 0.6,
        "is_bored" => ctx.dopamine < 0.3 && ctx.noradrenaline < 0.3,
        "needs_comfort" => ctx.cortisol > 0.5 && ctx.serotonin < 0.4,
        _ => false,
    }
}

/// Evalue une action nommee (retourne Success si applicable).
fn eval_action(key: &str, ctx: &BtContext) -> BtStatus {
    match key {
        "introspect" => if ctx.cortisol > 0.5 { BtStatus::Success } else { BtStatus::Failure },
        "explore" => if ctx.dopamine > 0.4 { BtStatus::Success } else { BtStatus::Failure },
        "rest" => if ctx.cortisol < 0.3 { BtStatus::Success } else { BtStatus::Running },
        "focus" => if ctx.noradrenaline > 0.4 { BtStatus::Success } else { BtStatus::Failure },
        "heal" => if ctx.cortisol > 0.6 || ctx.serotonin < 0.3 { BtStatus::Running } else { BtStatus::Success },
        // Actions conversationnelles
        "comfort" | "deepen" | "play" | "question" => BtStatus::Success,
        _ => BtStatus::Success,
    }
}

/// Evalue l'arbre et retourne l'action recommandee (le nom de la derniere Action
/// executee avec succes dans le chemin gagnant).
pub fn tick_and_recommend(tree: &BtNode, ctx: &BtContext) -> Option<String> {
    fn find_action(node: &BtNode, ctx: &BtContext) -> Option<String> {
        match node {
            BtNode::Selector { children, .. } => {
                for child in children {
                    match child.tick(ctx) {
                        BtStatus::Success | BtStatus::Running => {
                            return find_action(child, ctx).or_else(|| Some("ok".into()));
                        }
                        BtStatus::Failure => continue,
                    }
                }
                None
            }
            BtNode::Sequence { children, .. } => {
                let mut last_action = None;
                for child in children {
                    match child.tick(ctx) {
                        BtStatus::Failure => return None,
                        BtStatus::Running | BtStatus::Success => {
                            if let Some(a) = find_action(child, ctx) {
                                last_action = Some(a);
                            }
                        }
                    }
                }
                last_action
            }
            BtNode::Action { action_key, .. } => {
                if eval_action(action_key, ctx) != BtStatus::Failure {
                    Some(action_key.clone())
                } else {
                    None
                }
            }
            BtNode::Condition { .. } => None,
            BtNode::Inverter { child } => find_action(child, ctx),
        }
    }
    find_action(tree, ctx)
}

/// Arbre de comportement pour la conversation.
/// Branches : comfort > deepen > play > question
pub fn conversation_tree() -> BtNode {
    BtNode::Selector {
        name: "ConversationRoot".into(),
        children: vec![
            // Si l'interlocuteur a besoin de confort
            BtNode::Sequence {
                name: "ComfortBranch".into(),
                children: vec![
                    BtNode::Condition { name: "NeedsComfort".into(), condition_key: "needs_comfort".into() },
                    BtNode::Action { name: "Comfort".into(), action_key: "comfort".into() },
                ],
            },
            // Si motive et curieux → approfondir
            BtNode::Sequence {
                name: "DeepenBranch".into(),
                children: vec![
                    BtNode::Condition { name: "Motivated".into(), condition_key: "is_motivated".into() },
                    BtNode::Condition { name: "Curious".into(), condition_key: "is_curious".into() },
                    BtNode::Action { name: "Deepen".into(), action_key: "deepen".into() },
                ],
            },
            // Si joyeux → jouer
            BtNode::Sequence {
                name: "PlayBranch".into(),
                children: vec![
                    BtNode::Condition { name: "Joyful".into(), condition_key: "is_joyful".into() },
                    BtNode::Action { name: "Play".into(), action_key: "play".into() },
                ],
            },
            // Defaut : questionner
            BtNode::Action { name: "Question".into(), action_key: "question".into() },
        ],
    }
}

// =============================================================================
// Arbres predefinis pour le pipeline cognitif
// =============================================================================

/// Arbre de comportement par defaut : gestion du stress et de la motivation.
pub fn default_cognitive_tree() -> BtNode {
    BtNode::Selector {
        name: "CognitiveRoot".into(),
        children: vec![
            // Priorite 1 : Gestion du stress
            BtNode::Sequence {
                name: "StressResponse".into(),
                children: vec![
                    BtNode::Condition { name: "CheckStress".into(), condition_key: "is_stressed".into() },
                    BtNode::Action { name: "Introspect".into(), action_key: "introspect".into() },
                    BtNode::Action { name: "Heal".into(), action_key: "heal".into() },
                ],
            },
            // Priorite 2 : Exploration si motive
            BtNode::Sequence {
                name: "ExplorationDrive".into(),
                children: vec![
                    BtNode::Condition { name: "CheckMotivation".into(), condition_key: "is_motivated".into() },
                    BtNode::Condition { name: "CheckCurious".into(), condition_key: "is_curious".into() },
                    BtNode::Action { name: "Explore".into(), action_key: "explore".into() },
                ],
            },
            // Priorite 3 : Focus si attentif
            BtNode::Sequence {
                name: "FocusMode".into(),
                children: vec![
                    BtNode::Condition { name: "CheckFocus".into(), condition_key: "is_focused".into() },
                    BtNode::Action { name: "Focus".into(), action_key: "focus".into() },
                ],
            },
            // Defaut : repos
            BtNode::Action { name: "Rest".into(), action_key: "rest".into() },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(cortisol: f64, dopamine: f64, emotion: &str) -> BtContext {
        BtContext {
            cortisol,
            dopamine,
            serotonin: 0.5,
            noradrenaline: 0.5,
            dominant_emotion: emotion.to_string(),
            consciousness_level: 0.5,
            in_conversation: false,
            cycle: 1,
            oxytocin: 0.5,
            endorphin: 0.5,
            recommended_action: None,
        }
    }

    #[test]
    fn test_stress_response() {
        let tree = default_cognitive_tree();
        let ctx = make_ctx(0.8, 0.3, "Anxiété");
        let result = tree.tick(&ctx);
        // Stress eleve → doit reussir la branche stress
        assert_ne!(result, BtStatus::Failure);
    }

    #[test]
    fn test_exploration_when_motivated() {
        let tree = default_cognitive_tree();
        let ctx = make_ctx(0.2, 0.8, "Curiosité");
        let result = tree.tick(&ctx);
        assert_eq!(result, BtStatus::Success);
    }

    #[test]
    fn test_condition_evaluation() {
        let ctx = make_ctx(0.8, 0.3, "Anxiété");
        assert!(eval_condition("is_stressed", &ctx));
        assert!(!eval_condition("is_calm", &ctx));
    }
}
