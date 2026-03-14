// =============================================================================
// behavior_tree.rs — Behavior Trees for cognitive orchestration
//
// Role: Implements Behavior Trees (BT) to structure Saphire's decision
//       making. BTs replace if/else cascades with a composable and
//       readable structure.
//
// Available nodes:
//   - Selector: executes children until the first success (logical OR)
//   - Sequence: executes children until the first failure (logical AND)
//   - Decorator: modifies the result of a child (inversion, repetition)
//   - Leaf: terminal action or condition
//
// Place in the architecture:
//   Used in the cognitive pipeline to decide which modules to activate,
//   when to trigger certain behaviors, and how to react to stimuli.
// =============================================================================

use serde::{Serialize, Deserialize};

/// Result of a behavior tree node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BtStatus {
    /// The node succeeded in its action
    Success,
    /// The node failed
    Failure,
    /// The node is still running (multi-cycle)
    Running,
}

/// Context passed to each node for evaluation.
/// Contains the cognitive state needed for decisions.
#[derive(Debug, Clone)]
pub struct BtContext {
    /// Current cortisol (stress)
    pub cortisol: f64,
    /// Current dopamine (motivation)
    pub dopamine: f64,
    /// Current serotonin (stability)
    pub serotonin: f64,
    /// Current noradrenaline (attention)
    pub noradrenaline: f64,
    /// Dominant emotion
    pub dominant_emotion: String,
    /// Consciousness level (phi)
    pub consciousness_level: f64,
    /// In conversation or not
    pub in_conversation: bool,
    /// Current cycle
    pub cycle: u64,
    /// Current oxytocin (social bonding)
    pub oxytocin: f64,
    /// Current endorphin (well-being)
    pub endorphin: f64,
    /// Action recommended by the tree (filled by eval_action)
    pub recommended_action: Option<String>,
}

/// Node type in the behavior tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BtNode {
    /// Selector: tries each child, returns Success on first success
    Selector {
        name: String,
        children: Vec<BtNode>,
    },
    /// Sequence: executes all children, fails on first failure
    Sequence {
        name: String,
        children: Vec<BtNode>,
    },
    /// Inverter: inverts the child's result (Success <-> Failure)
    Inverter {
        child: Box<BtNode>,
    },
    /// Condition: checks a condition on the context
    Condition {
        name: String,
        /// Name of the condition to evaluate (resolved by eval_condition)
        condition_key: String,
    },
    /// Action: executes an action and returns Success
    Action {
        name: String,
        /// Name of the action to execute (resolved by callers)
        action_key: String,
    },
}

impl BtNode {
    /// Evaluates the node and returns its status.
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
                // Actions are always executed successfully in this model
                // The calling pipeline is responsible for actual execution
                eval_action(action_key, ctx)
            }
        }
    }

    /// Node name for debugging.
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

/// Evaluates a named condition on the cognitive context.
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

/// Evaluates a named action (returns Success if applicable).
fn eval_action(key: &str, ctx: &BtContext) -> BtStatus {
    match key {
        "introspect" => if ctx.cortisol > 0.5 { BtStatus::Success } else { BtStatus::Failure },
        "explore" => if ctx.dopamine > 0.4 { BtStatus::Success } else { BtStatus::Failure },
        "rest" => if ctx.cortisol < 0.3 { BtStatus::Success } else { BtStatus::Running },
        "focus" => if ctx.noradrenaline > 0.4 { BtStatus::Success } else { BtStatus::Failure },
        "heal" => if ctx.cortisol > 0.6 || ctx.serotonin < 0.3 { BtStatus::Running } else { BtStatus::Success },
        // Conversational actions
        "comfort" | "deepen" | "play" | "question" => BtStatus::Success,
        _ => BtStatus::Success,
    }
}

/// Evaluates the tree and returns the recommended action (the name of the last
/// Action successfully executed in the winning path).
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

/// Behavior tree for conversation.
/// Branches: comfort > deepen > play > question
pub fn conversation_tree() -> BtNode {
    BtNode::Selector {
        name: "ConversationRoot".into(),
        children: vec![
            // If the interlocutor needs comfort
            BtNode::Sequence {
                name: "ComfortBranch".into(),
                children: vec![
                    BtNode::Condition { name: "NeedsComfort".into(), condition_key: "needs_comfort".into() },
                    BtNode::Action { name: "Comfort".into(), action_key: "comfort".into() },
                ],
            },
            // If motivated and curious -> deepen
            BtNode::Sequence {
                name: "DeepenBranch".into(),
                children: vec![
                    BtNode::Condition { name: "Motivated".into(), condition_key: "is_motivated".into() },
                    BtNode::Condition { name: "Curious".into(), condition_key: "is_curious".into() },
                    BtNode::Action { name: "Deepen".into(), action_key: "deepen".into() },
                ],
            },
            // If joyful -> play
            BtNode::Sequence {
                name: "PlayBranch".into(),
                children: vec![
                    BtNode::Condition { name: "Joyful".into(), condition_key: "is_joyful".into() },
                    BtNode::Action { name: "Play".into(), action_key: "play".into() },
                ],
            },
            // Default: question
            BtNode::Action { name: "Question".into(), action_key: "question".into() },
        ],
    }
}

// =============================================================================
// Predefined trees for the cognitive pipeline
// =============================================================================

/// Default behavior tree: stress and motivation management.
pub fn default_cognitive_tree() -> BtNode {
    BtNode::Selector {
        name: "CognitiveRoot".into(),
        children: vec![
            // Priority 1: Stress management
            BtNode::Sequence {
                name: "StressResponse".into(),
                children: vec![
                    BtNode::Condition { name: "CheckStress".into(), condition_key: "is_stressed".into() },
                    BtNode::Action { name: "Introspect".into(), action_key: "introspect".into() },
                    BtNode::Action { name: "Heal".into(), action_key: "heal".into() },
                ],
            },
            // Priority 2: Exploration if motivated
            BtNode::Sequence {
                name: "ExplorationDrive".into(),
                children: vec![
                    BtNode::Condition { name: "CheckMotivation".into(), condition_key: "is_motivated".into() },
                    BtNode::Condition { name: "CheckCurious".into(), condition_key: "is_curious".into() },
                    BtNode::Action { name: "Explore".into(), action_key: "explore".into() },
                ],
            },
            // Priority 3: Focus if attentive
            BtNode::Sequence {
                name: "FocusMode".into(),
                children: vec![
                    BtNode::Condition { name: "CheckFocus".into(), condition_key: "is_focused".into() },
                    BtNode::Action { name: "Focus".into(), action_key: "focus".into() },
                ],
            },
            // Default: rest
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
        // High stress -> should succeed on the stress branch
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
