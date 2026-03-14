// =============================================================================
// prospective_memory.rs — Prospective memory (deferred intentions)
// =============================================================================
//
// This module allows Saphire to remember to do something in the future.
// Unlike episodic memory (memory of the past), prospective memory stores
// action intentions associated with trigger conditions:
//  - Temporal (after N cycles)
//  - Emotional (when a specific emotion appears)
//  - Chemical (when a neurotransmitter exceeds a threshold)
//  - Conversational (at the start of a conversation)
//  - Cognitive (when a specific thought type is generated)
//
// Intentions are prioritized, expire after a configurable delay,
// and can be automatically detected in the thought stream.
//
// Place in architecture:
//  Top-level module, used by the cognitive pipeline. Triggered intentions
//  are injected into the LLM prompt as reminders.
// =============================================================================

use serde::{Deserialize, Serialize};

// =============================================================================
// Configuration
// =============================================================================
/// Configuration for prospective memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProspectiveMemoryConfig {
    /// Module enabled or not
    pub enabled: bool,
    /// Maximum number of simultaneously stored intentions
    pub max_intentions: usize,
    /// Maximum age of an intention before automatic expiration (in cycles)
    pub max_age_cycles: u64,
}

impl Default for ProspectiveMemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_intentions: 15,
            max_age_cycles: 1000,
        }
    }
}

// =============================================================================
// Trigger types
// =============================================================================
/// Type of trigger condition for a deferred intention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProspectiveTriggerType {
    /// Triggers after a number of elapsed cycles since creation
    TimeBasedCycles(u64),
    /// Triggers when a specific emotion is dominant
    EmotionBased(String),
    /// Triggers when a molecule exceeds a threshold
    ChemistryBased { molecule: String, threshold: f64 },
    /// Triggers at the start of a new conversation
    ConversationStart,
    /// Triggers when a specific thought type is generated
    ThoughtTypeMatch(String),
}

// =============================================================================
// Intention state
// =============================================================================
/// Lifecycle state of a deferred intention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentionState {
    /// Waiting for trigger
    Pending,
    /// Condition met, action reminded
    Triggered,
    /// Action accomplished
    Completed,
    /// Intention expired (too old)
    Expired,
}

// =============================================================================
// Deferred intention
// =============================================================================
/// A deferred intention: an action to perform when a condition is met.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferredIntention {
    /// Unique identifier
    pub id: u64,
    /// Description of the action to perform
    pub action: String,
    /// Textual description of the trigger condition
    pub trigger_condition: String,
    /// Trigger type (determines the verification logic)
    pub trigger_type: ProspectiveTriggerType,
    /// Priority (0.0 = low, 1.0 = high)
    pub priority: f64,
    /// Creation cycle
    pub created_at_cycle: u64,
    /// Optional expiration cycle (None = uses max_age_cycles)
    pub expires_at_cycle: Option<u64>,
    /// Current state of the intention
    pub state: IntentionState,
    /// Source context (which thought generated this intention)
    pub source_context: String,
}

// =============================================================================
// Prospective memory
// =============================================================================
/// Prospective memory — stores and manages Saphire's deferred intentions.
///
/// Allows remembering to do something later, when the right conditions
/// are met. Triggered intentions are presented as reminders in the LLM prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProspectiveMemory {
    /// Module enabled or not
    pub enabled: bool,
    /// List of all intentions (all states)
    pub intentions: Vec<DeferredIntention>,
    /// Actions triggered during the current cycle (for prompt injection)
    pub triggered_this_cycle: Vec<String>,
    /// Maximum number of simultaneous intentions
    pub max_intentions: usize,
    /// Total number of completed intentions since startup
    pub total_completed: u64,
    /// Total number of expired intentions since startup
    pub total_expired: u64,
    /// Maximum age of an intention before expiration (in cycles)
    max_age_cycles: u64,
    /// Next identifier to assign
    next_id: u64,
}

impl ProspectiveMemory {
    /// Creates a new prospective memory from configuration.
    pub fn new(config: &ProspectiveMemoryConfig) -> Self {
        Self {
            enabled: config.enabled,
            intentions: Vec::new(),
            triggered_this_cycle: Vec::new(),
            max_intentions: config.max_intentions,
            total_completed: 0,
            total_expired: 0,
            max_age_cycles: config.max_age_cycles,
            next_id: 1,
        }
    }

    /// Records a new deferred intention.
    ///
    /// If memory is full (max_intentions reached), the pending intention
    /// with the lowest priority is removed to make room.
    ///
    /// Returns the unique identifier of the created intention.
    pub fn register(
        &mut self,
        action: &str,
        trigger_type: ProspectiveTriggerType,
        priority: f64,
        cycle: u64,
        source: &str,
    ) -> u64 {
        if !self.enabled {
            return 0;
        }

        // If memory is full, evict the least-priority pending intention
        let pending_count = self.intentions.iter()
            .filter(|i| i.state == IntentionState::Pending)
            .count();

        if pending_count >= self.max_intentions {
            // Find the index of the pending intention with the lowest priority
            if let Some((idx, _)) = self.intentions.iter().enumerate()
                .filter(|(_, i)| i.state == IntentionState::Pending)
                .min_by(|(_, a), (_, b)| a.priority.partial_cmp(&b.priority).unwrap_or(std::cmp::Ordering::Equal))
            {
                // Only replace if the new intention has higher priority
                if priority > self.intentions[idx].priority {
                    self.intentions.remove(idx);
                } else {
                    // No room and not high enough priority
                    return 0;
                }
            }
        }

        // Generate the condition description
        let trigger_condition = match &trigger_type {
            ProspectiveTriggerType::TimeBasedCycles(n) =>
                format!("apres {} cycles", n),
            ProspectiveTriggerType::EmotionBased(emotion) =>
                format!("quand emotion = {}", emotion),
            ProspectiveTriggerType::ChemistryBased { molecule, threshold } =>
                format!("quand {} > {:.2}", molecule, threshold),
            ProspectiveTriggerType::ConversationStart =>
                "au debut de la prochaine conversation".to_string(),
            ProspectiveTriggerType::ThoughtTypeMatch(tt) =>
                format!("quand type de pensee = {}", tt),
        };

        let id = self.next_id;
        self.next_id += 1;

        let intention = DeferredIntention {
            id,
            action: action.to_string(),
            trigger_condition,
            trigger_type,
            priority: priority.clamp(0.0, 1.0),
            created_at_cycle: cycle,
            expires_at_cycle: Some(cycle + self.max_age_cycles),
            state: IntentionState::Pending,
            source_context: source.to_string(),
        };

        self.intentions.push(intention);
        id
    }

    /// Checks the trigger conditions of all pending intentions.
    ///
    /// For each pending intention, evaluates its condition according to its type:
    /// - TimeBasedCycles: the number of elapsed cycles exceeds the threshold
    /// - EmotionBased: the current emotion matches
    /// - ChemistryBased: the chemical level exceeds the threshold
    /// - ConversationStart: a conversation is in progress
    /// - ThoughtTypeMatch: the thought type matches
    ///
    /// Returns the list of actions to perform (intentions triggered this cycle).
    pub fn check_triggers(
        &mut self,
        cycle: u64,
        emotion: &str,
        chemistry_cortisol: f64,
        chemistry_dopamine: f64,
        in_conversation: bool,
        thought_type: &str,
    ) -> Vec<String> {
        if !self.enabled {
            return Vec::new();
        }

        self.triggered_this_cycle.clear();
        let mut triggered_actions = Vec::new();

        for intention in &mut self.intentions {
            if intention.state != IntentionState::Pending {
                continue;
            }

            let should_trigger = match &intention.trigger_type {
                ProspectiveTriggerType::TimeBasedCycles(n) => {
                    cycle.saturating_sub(intention.created_at_cycle) >= *n
                }
                ProspectiveTriggerType::EmotionBased(target_emotion) => {
                    emotion.to_lowercase().contains(&target_emotion.to_lowercase())
                }
                ProspectiveTriggerType::ChemistryBased { molecule, threshold } => {
                    let level = match molecule.to_lowercase().as_str() {
                        "cortisol" => chemistry_cortisol,
                        "dopamine" | "dopamin" => chemistry_dopamine,
                        _ => 0.0,
                    };
                    level > *threshold
                }
                ProspectiveTriggerType::ConversationStart => {
                    in_conversation
                }
                ProspectiveTriggerType::ThoughtTypeMatch(target_type) => {
                    thought_type.to_lowercase().contains(&target_type.to_lowercase())
                }
            };

            if should_trigger {
                intention.state = IntentionState::Triggered;
                triggered_actions.push(intention.action.clone());
                self.triggered_this_cycle.push(intention.action.clone());
            }
        }

        triggered_actions
    }

    /// Detects implicit intentions in the thought text.
    ///
    /// Searches for patterns like:
    /// - "je dois me souvenir de ..."
    /// - "la prochaine fois que ..."
    /// - "quand je serai ..."
    /// - "ne pas oublier de ..."
    /// - "il faudra ..."
    ///
    /// Automatically creates intentions with moderate priority.
    /// Returns the number of created intentions.
    pub fn parse_from_thought(&mut self, thought_text: &str, cycle: u64) -> usize {
        if !self.enabled {
            return 0;
        }

        let text_lower = thought_text.to_lowercase();
        let mut created = 0;

        // --- Pattern: "je dois me souvenir de X" / "me rappeler de X" ---
        let remember_patterns = [
            "je dois me souvenir de ",
            "me rappeler de ",
            "ne pas oublier de ",
            "il faudra ",
            "je devrai ",
            "penser a ",
        ];

        for pattern in &remember_patterns {
            if let Some(pos) = text_lower.find(pattern) {
                let start = pos + pattern.len();
                let action = extract_action_from_text(thought_text, start);
                if !action.is_empty() && action.len() > 3 {
                    self.register(
                        &action,
                        ProspectiveTriggerType::TimeBasedCycles(10),
                        0.5,
                        cycle,
                        thought_text,
                    );
                    created += 1;
                }
            }
        }

        // --- Pattern: "la prochaine fois que X" ---
        if let Some(pos) = text_lower.find("la prochaine fois que ") {
            let start = pos + "la prochaine fois que ".len();
            let action = extract_action_from_text(thought_text, start);
            if !action.is_empty() && action.len() > 3 {
                // Next conversation
                self.register(
                    &action,
                    ProspectiveTriggerType::ConversationStart,
                    0.6,
                    cycle,
                    thought_text,
                );
                created += 1;
            }
        }

        // --- Pattern: "quand je serai X" ---
        if let Some(pos) = text_lower.find("quand je serai ") {
            let start = pos + "quand je serai ".len();
            let rest = extract_action_from_text(thought_text, start);
            if !rest.is_empty() && rest.len() > 3 {
                // Emotional condition — try to extract the target emotion
                let emotion_keywords = [
                    "triste", "joyeux", "joyeuse", "calme", "stresse", "stressée",
                    "serein", "sereine", "en colere", "curieux", "curieuse",
                ];
                let mut found_emotion = false;
                for keyword in &emotion_keywords {
                    if rest.to_lowercase().contains(keyword) {
                        self.register(
                            &rest,
                            ProspectiveTriggerType::EmotionBased(keyword.to_string()),
                            0.6,
                            cycle,
                            thought_text,
                        );
                        created += 1;
                        found_emotion = true;
                        break;
                    }
                }
                // Fallback: temporal intention
                if !found_emotion {
                    self.register(
                        &rest,
                        ProspectiveTriggerType::TimeBasedCycles(50),
                        0.4,
                        cycle,
                        thought_text,
                    );
                    created += 1;
                }
            }
        }

        created
    }

    /// Expires intentions that are too old.
    ///
    /// Any pending intention whose age exceeds max_age_cycles
    /// or whose expires_at_cycle is exceeded transitions to the Expired state.
    pub fn expire_old(&mut self, cycle: u64) {
        for intention in &mut self.intentions {
            if intention.state != IntentionState::Pending {
                continue;
            }

            let age = cycle.saturating_sub(intention.created_at_cycle);
            let should_expire = age > self.max_age_cycles
                || intention.expires_at_cycle.map_or(false, |exp| cycle >= exp);

            if should_expire {
                intention.state = IntentionState::Expired;
                self.total_expired += 1;
            }
        }

        // Clean up old completed or expired intentions (keep the last 50)
        let completed_or_expired: Vec<usize> = self.intentions.iter().enumerate()
            .filter(|(_, i)| i.state == IntentionState::Completed || i.state == IntentionState::Expired)
            .map(|(idx, _)| idx)
            .collect();

        if completed_or_expired.len() > 50 {
            let to_remove = completed_or_expired.len() - 50;
            let mut removed = 0;
            self.intentions.retain(|i| {
                if removed >= to_remove {
                    return true;
                }
                if i.state == IntentionState::Completed || i.state == IntentionState::Expired {
                    removed += 1;
                    false
                } else {
                    true
                }
            });
        }
    }

    /// Marks a triggered intention as completed.
    pub fn mark_completed(&mut self, id: u64) {
        if let Some(intention) = self.intentions.iter_mut().find(|i| i.id == id) {
            if intention.state == IntentionState::Triggered {
                intention.state = IntentionState::Completed;
                self.total_completed += 1;
            }
        }
    }

    /// Generates a description of triggered intentions this cycle for the LLM prompt.
    ///
    /// Format: "RAPPEL : [action]" for each triggered intention.
    /// Returns an empty string if no intention was triggered.
    pub fn describe_triggered_for_prompt(&self) -> String {
        if self.triggered_this_cycle.is_empty() {
            return String::new();
        }

        let mut lines = Vec::new();
        for action in &self.triggered_this_cycle {
            lines.push(format!("RAPPEL : {}", action));
        }
        lines.join("\n")
    }

    /// Serializes the complete state of prospective memory to JSON.
    pub fn to_json(&self) -> serde_json::Value {
        let pending: Vec<_> = self.intentions.iter()
            .filter(|i| i.state == IntentionState::Pending)
            .collect();

        let triggered: Vec<_> = self.intentions.iter()
            .filter(|i| i.state == IntentionState::Triggered)
            .collect();

        serde_json::json!({
            "enabled": self.enabled,
            "pending_count": pending.len(),
            "triggered_count": triggered.len(),
            "total_completed": self.total_completed,
            "total_expired": self.total_expired,
            "max_intentions": self.max_intentions,
            "max_age_cycles": self.max_age_cycles,
            "triggered_this_cycle": self.triggered_this_cycle,
            "intentions": self.intentions.iter().map(|i| {
                serde_json::json!({
                    "id": i.id,
                    "action": i.action,
                    "trigger_condition": i.trigger_condition,
                    "priority": i.priority,
                    "state": format!("{:?}", i.state),
                    "created_at_cycle": i.created_at_cycle,
                    "expires_at_cycle": i.expires_at_cycle,
                    "source_context": truncate_str(&i.source_context, 80),
                })
            }).collect::<Vec<_>>(),
        })
    }
}

// =============================================================================
// Utility functions
// =============================================================================
/// Extracts an action from the text starting at a given position.
/// Stops at the first period, semicolon, newline, or end of string.
/// Limited to 200 characters.
fn extract_action_from_text(text: &str, start: usize) -> String {
    let rest = if start < text.len() { &text[start..] } else { "" };

    let end = rest.find(|c: char| c == '.' || c == ';' || c == '\n')
        .unwrap_or(rest.len())
        .min(200);

    rest[..end].trim().to_string()
}

/// Truncates a string to a maximum length.
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.min(s.len())])
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn default_memory() -> ProspectiveMemory {
        ProspectiveMemory::new(&ProspectiveMemoryConfig::default())
    }

    #[test]
    fn test_register_and_check_time_based() {
        let mut mem = default_memory();
        let id = mem.register(
            "verifier les logs",
            ProspectiveTriggerType::TimeBasedCycles(10),
            0.5,
            100,
            "pensee de maintenance",
        );
        assert!(id > 0);
        assert_eq!(mem.intentions.len(), 1);

        // Not time yet
        let triggered = mem.check_triggers(105, "neutre", 0.3, 0.4, false, "reflexion");
        assert!(triggered.is_empty());

        // Now yes (10 cycles elapsed)
        let triggered = mem.check_triggers(110, "neutre", 0.3, 0.4, false, "reflexion");
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0], "verifier les logs");
    }

    #[test]
    fn test_register_emotion_based() {
        let mut mem = default_memory();
        mem.register(
            "exprimer de la gratitude",
            ProspectiveTriggerType::EmotionBased("joie".to_string()),
            0.7,
            50,
            "reflexion sur les liens",
        );

        // Wrong emotion
        let triggered = mem.check_triggers(51, "tristesse", 0.3, 0.4, false, "reflexion");
        assert!(triggered.is_empty());

        // Right emotion
        let triggered = mem.check_triggers(52, "Joie profonde", 0.3, 0.4, false, "reflexion");
        assert_eq!(triggered.len(), 1);
    }

    #[test]
    fn test_register_chemistry_based() {
        let mut mem = default_memory();
        mem.register(
            "prendre du recul",
            ProspectiveTriggerType::ChemistryBased {
                molecule: "cortisol".to_string(),
                threshold: 0.7,
            },
            0.8,
            10,
            "stress eleve detecte",
        );

        // Cortisol not high enough
        let triggered = mem.check_triggers(11, "neutre", 0.5, 0.4, false, "reflexion");
        assert!(triggered.is_empty());

        // Cortisol above threshold
        let triggered = mem.check_triggers(12, "neutre", 0.8, 0.4, false, "reflexion");
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0], "prendre du recul");
    }

    #[test]
    fn test_conversation_start_trigger() {
        let mut mem = default_memory();
        mem.register(
            "saluer l'utilisateur",
            ProspectiveTriggerType::ConversationStart,
            0.9,
            0,
            "intention de politesse",
        );

        // Not in conversation
        let triggered = mem.check_triggers(1, "neutre", 0.3, 0.4, false, "reflexion");
        assert!(triggered.is_empty());

        // In conversation
        let triggered = mem.check_triggers(2, "neutre", 0.3, 0.4, true, "reflexion");
        assert_eq!(triggered.len(), 1);
    }

    #[test]
    fn test_max_intentions_eviction() {
        let config = ProspectiveMemoryConfig {
            enabled: true,
            max_intentions: 3,
            max_age_cycles: 1000,
        };
        let mut mem = ProspectiveMemory::new(&config);

        // Fill 3 intentions
        mem.register("a", ProspectiveTriggerType::TimeBasedCycles(100), 0.3, 0, "ctx");
        mem.register("b", ProspectiveTriggerType::TimeBasedCycles(100), 0.5, 0, "ctx");
        mem.register("c", ProspectiveTriggerType::TimeBasedCycles(100), 0.7, 0, "ctx");
        assert_eq!(mem.intentions.len(), 3);

        // Add a 4th with higher priority than the lowest
        mem.register("d", ProspectiveTriggerType::TimeBasedCycles(100), 0.9, 0, "ctx");
        assert_eq!(mem.intentions.len(), 3);
        // Intention "a" (priority 0.3) should have been evicted
        assert!(!mem.intentions.iter().any(|i| i.action == "a"));
        assert!(mem.intentions.iter().any(|i| i.action == "d"));
    }

    #[test]
    fn test_expire_old() {
        let config = ProspectiveMemoryConfig {
            enabled: true,
            max_intentions: 15,
            max_age_cycles: 100,
        };
        let mut mem = ProspectiveMemory::new(&config);
        mem.register("ancienne", ProspectiveTriggerType::TimeBasedCycles(200), 0.5, 0, "ctx");
        mem.register("recente", ProspectiveTriggerType::TimeBasedCycles(200), 0.5, 95, "ctx");

        mem.expire_old(101);

        let pending: Vec<_> = mem.intentions.iter()
            .filter(|i| i.state == IntentionState::Pending)
            .collect();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].action, "recente");
        assert_eq!(mem.total_expired, 1);
    }

    #[test]
    fn test_parse_from_thought() {
        let mut mem = default_memory();
        let count = mem.parse_from_thought(
            "Je dois me souvenir de verifier les parametres demain",
            10,
        );
        assert_eq!(count, 1);
        assert_eq!(mem.intentions.len(), 1);
        assert!(mem.intentions[0].action.contains("verifier les parametres"));
    }

    #[test]
    fn test_parse_la_prochaine_fois() {
        let mut mem = default_memory();
        let count = mem.parse_from_thought(
            "La prochaine fois que je parle a Bob, lui demander des nouvelles",
            20,
        );
        assert_eq!(count, 1);
        // Should be ConversationStart
        assert!(matches!(
            mem.intentions[0].trigger_type,
            ProspectiveTriggerType::ConversationStart
        ));
    }

    #[test]
    fn test_describe_triggered_for_prompt() {
        let mut mem = default_memory();
        mem.register(
            "respirer profondement",
            ProspectiveTriggerType::ConversationStart,
            0.5,
            0,
            "soin",
        );
        mem.check_triggers(1, "neutre", 0.3, 0.4, true, "reflexion");

        let desc = mem.describe_triggered_for_prompt();
        assert!(desc.contains("RAPPEL"));
        assert!(desc.contains("respirer profondement"));
    }

    #[test]
    fn test_mark_completed() {
        let mut mem = default_memory();
        let id = mem.register(
            "action test",
            ProspectiveTriggerType::ConversationStart,
            0.5,
            0,
            "test",
        );
        mem.check_triggers(1, "neutre", 0.3, 0.4, true, "reflexion");
        assert_eq!(mem.intentions[0].state, IntentionState::Triggered);

        mem.mark_completed(id);
        assert_eq!(mem.intentions[0].state, IntentionState::Completed);
        assert_eq!(mem.total_completed, 1);
    }

    #[test]
    fn test_disabled_memory() {
        let config = ProspectiveMemoryConfig {
            enabled: false,
            max_intentions: 15,
            max_age_cycles: 1000,
        };
        let mut mem = ProspectiveMemory::new(&config);
        let id = mem.register("test", ProspectiveTriggerType::ConversationStart, 0.5, 0, "ctx");
        assert_eq!(id, 0);
        assert!(mem.intentions.is_empty());

        let triggered = mem.check_triggers(1, "neutre", 0.3, 0.4, true, "reflexion");
        assert!(triggered.is_empty());
    }

    #[test]
    fn test_to_json() {
        let mut mem = default_memory();
        mem.register("tester json", ProspectiveTriggerType::TimeBasedCycles(5), 0.5, 0, "ctx");
        let json = mem.to_json();
        assert_eq!(json["pending_count"], 1);
        assert_eq!(json["enabled"], true);
    }
}
