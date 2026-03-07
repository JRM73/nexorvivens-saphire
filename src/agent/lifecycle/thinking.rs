// =============================================================================
// lifecycle/thinking.rs — Autonomous thinking orchestrator
// =============================================================================
//
// This file contains the main orchestrator (`autonomous_think`) that calls
// phases distributed across sub-files:
//   - thinking_perception.rs   — Pre-LLM phases (world perception)
//   - thinking_preparation.rs  — Thought selection and prompt preparation
//   - thinking_processing.rs   — LLM call and immediate post-processing
//   - thinking_reflection.rs   — Reflection and learning phases
//
// This file also contains shared structures:
//   - ThinkingContext: mutable context shared across all phases
//   - FeedbackRequest: pending human feedback request
//   - strip_chemical_trace(): cleanup of the LLM chemical trace header
//   - is_positive_feedback(): simple analysis of human feedback
// =============================================================================

use std::sync::atomic::Ordering;
use tokio::time::Instant;

use crate::emotions::EmotionalState;
use super::SaphireAgent;
use super::ProcessResult;

// =============================================================================
// FeedbackRequest — Pending human feedback request
// =============================================================================

/// A pending human feedback request awaiting a response.
/// Stored in `SaphireAgent.feedback_pending` when Saphire asks a question.
#[allow(dead_code)]
pub(super) struct FeedbackRequest {
    /// The text of the thought that prompted the feedback request.
    pub thought_text: String,
    /// The type of thought that was generated.
    pub thought_type: crate::agent::thought_engine::ThoughtType,
    /// The automatically computed reward for this thought.
    pub auto_reward: f64,
    /// The cycle number at which the feedback was requested.
    pub asked_at_cycle: u64,
}

/// Simple analysis of human feedback without an additional LLM call.
/// Returns `true` if the feedback is globally positive (clear approval).
/// Corrective messages ("yes but...", "more simply...") count as negative.
pub(super) fn is_positive_feedback(response: &str) -> bool {
    let lower = response.to_lowercase();
    // Corrective markers — the human corrects or suggests an improvement
    let corrective = [
        "mais", "plutot", "simplement", "simple", "par exemple",
        "tu devrais", "tu peux", "essaie", "il faudrait", "il faut",
        "au lieu", "instead", "try", "should", "better",
    ];
    let has_correction = corrective.iter().any(|w| lower.contains(*w));
    // If the message contains a corrective marker, it is negative feedback
    // even if it starts with "yes"
    if has_correction { return false; }

    let positive = [
        "oui", "bien", "exact", "d'accord", "interessant", "bravo",
        "continue", "j'aime", "genial", "super", "bon", "vrai",
        "absolument", "tout a fait", "en effet", "bonne", "yes",
        "great", "good", "nice", "right", "agree", "cool",
    ];
    let negative = [
        "non", "pas d'accord", "faux", "incorrect", "mauvais",
        "arrete", "stop", "ridicule", "n'importe quoi", "absurde",
        "no", "wrong", "bad", "disagree",
    ];
    let pos_score: usize = positive.iter().filter(|w| lower.contains(*w)).count();
    let neg_score: usize = negative.iter().filter(|w| lower.contains(*w)).count();
    pos_score > neg_score
}

/// Removes the chemical trace header from the beginning of an LLM thought.
/// Typical format: "C[sero,0.75,ocyt,0.55] E:Espoir+Curiosite V+0.90 A0.55 Actual text..."
/// Returns the cleaned text, ready to be displayed to the human.
pub(super) fn strip_chemical_trace(text: &str) -> String {
    let t = text.trim();
    // Find the end of the chemical header: after the last numeric field (V+x.xx or Ax.xx).
    // Pattern: C[...] E:... V+x.xx Ax.xx <text>
    // Strategy: find the first alphabetic character after an Ax.xx or V+x.xx pattern.
    if let Some(c_start) = t.find("C[") {
        // Find the end of the block: skip the chemical tokens
        let after_c = &t[c_start..];
        // Look for a pattern " A" followed by a digit, then find the text after it
        if let Some(a_pos) = after_c.rfind(" A") {
            let rest = &after_c[a_pos + 2..];
            // Skip the number (e.g., "0.55")
            let skip = rest.find(|c: char| c == ' ').unwrap_or(rest.len());
            if skip < rest.len() {
                return rest[skip..].trim().to_string();
            }
        }
        // Fallback: look after the "]" for the actual text (skip E:, V+, A tokens)
        if let Some(bracket_end) = after_c.find(']') {
            let rest = &after_c[bracket_end + 1..].trim();
            // Skip the short E:, V+, A tokens
            let chars = rest.chars().peekable();
            let mut pos = 0;
            let bytes = rest.as_bytes();
            while pos < rest.len() {
                // Skip whitespace
                while pos < rest.len() && bytes[pos] == b' ' { pos += 1; }
                if pos >= rest.len() { break; }
                // Token E:...
                if rest[pos..].starts_with("E:") {
                    pos = rest[pos..].find(' ').map(|p| pos + p).unwrap_or(rest.len());
                    continue;
                }
                // Token V+ ou V-
                if rest[pos..].starts_with("V+") || rest[pos..].starts_with("V-") {
                    pos = rest[pos..].find(' ').map(|p| pos + p).unwrap_or(rest.len());
                    continue;
                }
                // Token A suivi d'un chiffre
                if bytes[pos] == b'A' && pos + 1 < rest.len() && bytes[pos+1].is_ascii_digit() {
                    pos = rest[pos..].find(' ').map(|p| pos + p).unwrap_or(rest.len());
                    continue;
                }
                break;
            }
            let _ = chars;
            return rest[pos..].trim().to_string();
        }
    }
    t.to_string()
}

// =============================================================================
// ThinkingContext — Mutable context shared across all phases
// =============================================================================

/// Mutable context shared across all phases of autonomous thinking.
///
/// This struct gathers all intermediate variables that were previously local
/// variables in `autonomous_think()`. Each phase reads and/or writes to this
/// context, eliminating the need to pass many parameters between functions.
pub(super) struct ThinkingContext {
    /// Start instant of the cycle for measuring total duration.
    pub cycle_start: Instant,

    /// Thought type selected by the UCB1 bandit.
    pub thought_type: crate::agent::thought_engine::ThoughtType,

    /// Variant index for alternating prompts.
    pub variant: usize,

    /// Emotional state computed from the current chemistry.
    pub emotion: EmotionalState,

    /// Web knowledge context (formatted text + KnowledgeResult), if a web search was performed.
    pub knowledge_context: Option<(String, crate::knowledge::KnowledgeResult)>,

    /// Flag indicating whether a web search took place this cycle.
    pub was_web_search: bool,

    /// Textual hint for the LLM prompt.
    pub hint: String,

    /// World summary (weather, time of day, etc.).
    pub world_summary: String,

    /// Memory context built for the prompt (working memory + episodic + LTM).
    pub memory_context: String,

    /// Intuition patterns detected before the LLM call.
    pub intuition_patterns: Vec<crate::vital::intuition::IntuitionPattern>,

    /// Newly generated premonitions.
    pub new_premonitions: Vec<crate::vital::premonition::Premonition>,

    /// System prompt (static portion, cacheable via KV-cache).
    pub system_prompt: String,

    /// Dynamic prompt (user-role message sent to the LLM).
    pub prompt: String,

    /// Text of the thought generated by the LLM.
    pub thought_text: String,

    /// LLM response time in seconds.
    pub llm_elapsed: f64,

    /// Result of the brain pipeline (consensus, emotion, consciousness).
    pub process_result: Option<ProcessResult>,

    /// UCB1 reward computed for this cycle.
    pub reward: f64,

    /// Flag indicating whether an item was ejected from working memory.
    pub had_wm_ejection: bool,

    /// Flag indicating the cycle should be aborted (LLM error).
    pub should_abort: bool,

    /// Evaluated thought quality (0.0-1.0).
    pub quality: f64,

    /// Memory recall data for the cognitive trace.
    pub memory_trace_data: serde_json::Value,

    /// Analogy hint for the LLM prompt.
    pub analogy_hint: String,

    /// Associations found by the connectome (A* pathfinding).
    pub connectome_associations: String,

    /// Experiential anchor: concrete context to enrich the thought.
    pub anchor: Option<String>,

    /// MAP network tension (gap between perception and brain reaction).
    pub network_tension: f64,

    /// Self-framing: a frame self-formulated by Saphire.
    pub self_framing: Option<String>,
}

impl ThinkingContext {
    /// Creates a new context with default values.
    /// Actual contents are populated by each phase.
    pub(super) fn new() -> Self {
        Self {
            cycle_start: Instant::now(),
            thought_type: crate::agent::thought_engine::ThoughtType::Introspection,
            variant: 0,
            emotion: EmotionalState {
                dominant: String::new(),
                dominant_similarity: 0.0,
                secondary: None,
                valence: 0.0,
                arousal: 0.0,
                spectrum: Vec::new(),
                core_valence: 0.0,
                core_arousal: 0.0,
                context_influence: String::new(),
            },
            knowledge_context: None,
            was_web_search: false,
            hint: String::new(),
            world_summary: String::new(),
            memory_context: String::new(),
            intuition_patterns: Vec::new(),
            new_premonitions: Vec::new(),
            system_prompt: String::new(),
            prompt: String::new(),
            thought_text: String::new(),
            llm_elapsed: 0.0,
            process_result: None,
            reward: 0.0,
            had_wm_ejection: false,
            should_abort: false,
            quality: 0.0,
            memory_trace_data: serde_json::json!({}),
            analogy_hint: String::new(),
            connectome_associations: String::new(),
            anchor: None,
            network_tension: 0.0,
            self_framing: None,
        }
    }
}

// =============================================================================
// Main orchestrator — autonomous_think()
// =============================================================================

impl SaphireAgent {
    /// Autonomous thought: generated when no human is interacting with Saphire.
    ///
    /// This method is called periodically by the life loop (every
    /// `thought_interval` seconds). It orchestrates all phases via a shared
    /// `ThinkingContext`. Each phase is a named method defined in the
    /// `thinking_*.rs` sub-files.
    ///
    /// # Returns
    /// `Some(text)` if a thought was generated, `None` if the LLM was busy.
    pub async fn autonomous_think(&mut self) -> Option<String> {
        if self.llm_busy.load(Ordering::Relaxed) {
            return None;
        }
        let mut ctx = ThinkingContext::new();

        // Pre-LLM phases: update world state and internal state
        self.phase_init(&mut ctx);
        self.phase_weather_and_body(&mut ctx);
        self.phase_vital_spark(&mut ctx).await;
        self.phase_chemistry_history(&mut ctx);
        self.phase_birthday(&mut ctx).await;
        self.phase_world_broadcast(&mut ctx);
        self.phase_memory_decay(&mut ctx).await;
        self.phase_conversation_timeout(&mut ctx).await;
        self.phase_episodic_decay(&mut ctx).await;
        self.phase_consolidation(&mut ctx).await;

        // Thought selection and prompt preparation phases
        self.phase_select_thought(&mut ctx);
        self.phase_generate_dynamic_prompt(&mut ctx).await;
        self.phase_prospective(&mut ctx);
        self.phase_web_search(&mut ctx).await;
        self.phase_build_context(&mut ctx).await;
        self.phase_analogies(&mut ctx);
        self.phase_intuition_premonition(&mut ctx);
        self.phase_build_prompt(&mut ctx);

        // LLM call phase
        self.phase_call_llm(&mut ctx).await;
        if ctx.should_abort {
            return None;
        }

        // Post-LLM phases: response processing
        self.phase_llm_history(&mut ctx);
        self.phase_pipeline(&mut ctx);
        self.phase_monologue(&mut ctx);
        self.phase_dissonance(&mut ctx);
        self.phase_working_memory(&mut ctx).await;
        self.phase_memory_echo(&mut ctx).await;
        self.phase_reward_and_ethics(&mut ctx).await;
        self.phase_maybe_ask_feedback(&mut ctx);
        self.phase_lora_collect(&mut ctx).await;
        self.phase_knowledge_bonus(&mut ctx).await;
        self.phase_thought_log(&mut ctx).await;
        self.phase_cognitive_trace(&mut ctx);
        self.phase_broadcast(&mut ctx).await;
        self.phase_metrics(&mut ctx);
        self.phase_learning(&mut ctx).await;
        self.phase_nn_learning(&mut ctx).await;
        self.phase_self_critique(&mut ctx).await;
        self.phase_personality_snapshot(&mut ctx).await;
        self.phase_introspection_journal(&mut ctx).await;
        self.phase_desire_birth(&mut ctx).await;
        self.phase_narrative(&mut ctx);
        self.phase_state_clustering(&mut ctx);
        self.phase_homeostasis(&mut ctx);
        self.phase_connectome_associations(&mut ctx);

        Some(ctx.thought_text)
    }
}
