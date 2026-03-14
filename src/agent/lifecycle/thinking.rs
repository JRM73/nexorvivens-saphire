// =============================================================================
// lifecycle/thinking.rs — Autonomous thought orchestrator
// =============================================================================
//
// This file contains the main orchestrator (autonomous_think) that calls
// phases distributed across sub-files:
//  - thinking_perception.rs — Pre-LLM phases (world perception)
//  - thinking_preparation.rs — Thought selection and prompt preparation phases
//  - thinking_processing.rs — LLM and immediate post-processing phases
//  - thinking_reflection.rs — Reflection, learning, psychology phases
//
// This file also contains shared structures:
//  - ThinkingContext : mutable context shared between all phases
//  - FeedbackRequest : pending human feedback request
//  - strip_chemical_trace() : LLM chemical trace cleanup
//  - is_positive_feedback() : simple human feedback analysis
// =============================================================================

use std::sync::atomic::Ordering;
use tokio::time::Instant;

use crate::emotions::EmotionalState;
use super::SaphireAgent;
use super::ProcessResult;

// =============================================================================
// FeedbackRequest — Pending human feedback request
// =============================================================================
/// Pending human feedback request awaiting response.
/// Stored in `SaphireAgent.feedback_pending` when Saphire asks a question.
#[allow(dead_code)]
pub(super) struct FeedbackRequest {
    pub thought_text: String,
    pub thought_type: crate::agent::thought_engine::ThoughtType,
    pub auto_reward: f64,
    pub asked_at_cycle: u64,
}

/// Simple analysis of human feedback without additional LLM call.
/// Returns true if the feedback is overall positive (clear approval).
/// Corrective messages ("yes but...", "more simply...") are negative.
/// Analyzes the sentiment of human feedback via the LLM.
/// Falls back to simple heuristic if the LLM fails.
pub(super) async fn is_positive_feedback_llm(response: &str, llm_config: &crate::llm::LlmConfig) -> bool {
    let backend = crate::llm::create_backend(llm_config);
    let system = "Tu es un analyseur de sentiment. On te donne un message humain envoye en reponse \
                  a une question posee par une IA. Determine si le sentiment global du message est \
                  positif (encouragement, accord, interet, compliment, soutien) ou negatif \
                  (desaccord, critique, correction, rejet). \
                  Reponds UNIQUEMENT par le mot \"positif\" ou \"negatif\".".to_string();
    let user = response.to_string();

    let result = tokio::task::spawn_blocking(move || {
        backend.chat(&system, &user, 0.1, 5)
    }).await;

    match result {
        Ok(Ok(answer)) => {
            let lower = answer.trim().to_lowercase();
            if lower.contains("positif") {
                true
            } else if lower.contains("negatif") || lower.contains("négatif") {
                false
            } else {
                tracing::warn!("Feedback LLM: reponse inattendue '{}', fallback heuristique", answer.trim());
                is_positive_feedback_heuristic(response)
            }
        }
        _ => {
            tracing::warn!("Feedback LLM: echec appel, fallback heuristique");
            is_positive_feedback_heuristic(response)
        }
    }
}

/// Simple fallback heuristic (former method).
fn is_positive_feedback_heuristic(response: &str) -> bool {
    let lower = response.to_lowercase();
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

/// Removes the chemical trace from the start of an LLM thought.
/// Typical format: "C[sero,0.75,ocyt,0.55] E:Espoir+Curiosite V+0.90 A0.55 Real text..."
/// Returns the cleaned text, ready to be displayed to the human.
pub(super) fn strip_chemical_trace(text: &str) -> String {
    let t = text.trim();
    // Find the end of the chemical header: after the last numeric field (V+x.xx or Ax.xx)
    // The pattern is: C[...] E:... V+x.xx Ax.xx <text>
    // Strategy: find the first alphabetic character after an Ax.xx or V+x.xx pattern
    if let Some(c_start) = t.find("C[") {
        // Find the end of the block: skip chemical tokens
        let after_c = &t[c_start..];
        // Find a pattern " A" followed by a digit then find the text after
        if let Some(a_pos) = after_c.rfind(" A") {
            let rest = &after_c[a_pos + 2..];
            // Skip the number (e.g.: "0.55")
            let skip = rest.find(|c: char| c == ' ').unwrap_or(rest.len());
            if skip < rest.len() {
                return rest[skip..].trim().to_string();
            }
        }
        // Fallback: find the real text after "]" (skip E:, V+, A tokens)
        if let Some(bracket_end) = after_c.find(']') {
            let rest = &after_c[bracket_end + 1..].trim();
            // Skip the short E:, V+, A tokens
            let chars = rest.chars().peekable();
            let mut pos = 0;
            let bytes = rest.as_bytes();
            while pos < rest.len() {
                // Skip spaces
                while pos < rest.len() && bytes[pos] == b' ' { pos += 1; }
                if pos >= rest.len() { break; }
                // Token E:...
                if rest[pos..].starts_with("E:") {
                    pos = rest[pos..].find(' ').map(|p| pos + p).unwrap_or(rest.len());
                    continue;
                }
                // Token V+ or V-
                if rest[pos..].starts_with("V+") || rest[pos..].starts_with("V-") {
                    pos = rest[pos..].find(' ').map(|p| pos + p).unwrap_or(rest.len());
                    continue;
                }
                // Token A followed by a digit
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
// ThinkingContext — Mutable context shared between all phases
// =============================================================================
/// Mutable context shared between all autonomous thought phases.
///
/// This struct groups all intermediate variables that were formerly
/// local variables in autonomous_think(). Each phase reads and/or writes
/// to this context, eliminating the need to pass 15+ parameters
/// between functions.
pub(super) struct ThinkingContext {
    /// Cycle start instant for measuring total duration
    pub cycle_start: Instant,

    /// Thought type selected by the UCB1 bandit
    pub thought_type: crate::agent::thought_engine::ThoughtType,

    /// Variant index for alternating prompts
    pub variant: usize,

    /// Emotional state computed from current chemistry
    pub emotion: EmotionalState,

    /// Web knowledge context (text + KnowledgeResult)
    pub knowledge_context: Option<(String, crate::knowledge::KnowledgeResult)>,

    /// Flag indicating whether a web search occurred
    pub was_web_search: bool,

    /// Textual hint for the LLM prompt
    pub hint: String,

    /// World summary (weather, time, etc.)
    pub world_summary: String,

    /// Memory context built for the prompt
    pub memory_context: String,

    /// Intuitive patterns detected before the LLM
    pub intuition_patterns: Vec<crate::vital::intuition::IntuitionPattern>,

    /// Newly generated premonitions
    pub new_premonitions: Vec<crate::vital::premonition::Premonition>,

    /// System prompt (static, KV-cache cacheable)
    pub system_prompt: String,

    /// Dynamic prompt (user message)
    pub prompt: String,

    /// Text of the thought generated by the LLM
    pub thought_text: String,

    /// LLM response time in seconds
    pub llm_elapsed: f64,

    /// Result from the brain pipeline (consensus, emotion, consciousness)
    pub process_result: Option<ProcessResult>,

    /// UCB1 reward computed for this cycle
    pub reward: f64,

    /// Flag indicating whether an item was ejected from working memory
    pub had_wm_ejection: bool,

    /// Flag indicating the cycle should be aborted (LLM error)
    pub should_abort: bool,

    /// Optional voluntary deliberation for this cycle
    pub deliberation: Option<crate::psychology::will::Deliberation>,

    /// Total number of vector learnings (for metrics)
    pub nn_learnings_count: i32,

    /// Thought quality evaluated by metacognition (0.0-1.0)
    pub quality: f64,

    /// Memory recall data for the cognitive trace
    pub memory_trace_data: serde_json::Value,

    /// Analogy hint for the prompt (analogical reasoning M6)
    pub analogy_hint: String,

    /// Associations found by the connectome (A* pathfinding)
    pub connectome_associations: String,

    /// Experiential anchoring: concrete context to enrich the thought
    pub anchor: Option<String>,

    /// MAP network tension (gap between perception and brain reaction)
    pub network_tension: f64,

    /// Self-formulated frame by Saphire (self-framing): metrics, angle, depth
    pub self_framing: Option<String>,
}

impl ThinkingContext {
    /// Creates a new context with default values.
    /// Real contents are filled in by each phase.
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
            deliberation: None,
            nn_learnings_count: 0,
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
    /// `thought_interval` seconds). It orchestrates ~55 phases via a
    /// shared ThinkingContext. Each phase is a named method defined
    /// in the thinking_*.rs sub-files.
    ///
    /// Returns: `Some(text)` if a thought was generated, `None` if the LLM was busy.
    pub async fn autonomous_think(&mut self) -> Option<String> {
        if self.llm_busy.load(Ordering::Relaxed) {
            return None;
        }
        let mut ctx = ThinkingContext::new();

        // Pre-LLM phases: update world and internal state
        self.phase_init(&mut ctx);
        self.phase_weather_and_body(&mut ctx);
        self.phase_needs(&mut ctx);
        self.phase_vital_spark(&mut ctx).await;
        self.phase_senses(&mut ctx);
        self.phase_map_sync(&mut ctx);             // MAP : synchronise BrainNetwork + Connectome
        self.phase_chemistry_history(&mut ctx);
        self.phase_birthday(&mut ctx).await;
        self.phase_world_broadcast(&mut ctx);
        self.phase_memory_decay(&mut ctx).await;
        self.phase_conversation_timeout(&mut ctx).await;
        self.phase_episodic_decay(&mut ctx).await;
        self.phase_consolidation(&mut ctx).await;
        self.phase_auto_algorithms(&mut ctx).await;

        // Thought selection and prompt preparation phases
        self.phase_select_thought(&mut ctx);
        self.phase_generate_dynamic_prompt(&mut ctx).await;
        self.phase_connectome_associations(&mut ctx); // GA1 : A* pathfinding connectome
        self.phase_prospective(&mut ctx);           // M4 : memoire prospective
        self.phase_web_search(&mut ctx).await;
        self.phase_build_context(&mut ctx).await;
        self.phase_analogies(&mut ctx); // M6 : raisonnement analogique        self.phase_intuition_premonition(&mut ctx);
        self.phase_orchestrators(&mut ctx).await;
        self.phase_cognitive_load(&mut ctx);        // M7 : charge cognitive
        self.phase_build_prompt(&mut ctx); // inclut continuation monologue M2        self.phase_deliberation(&mut ctx);

        // LLM phase
        self.phase_call_llm(&mut ctx).await;
        if ctx.should_abort {
            return None;
        }

        // Post-LLM phases: response processing
        self.phase_llm_history(&mut ctx);
        self.phase_vectorial_filter(&mut ctx); // P2 : filtrage vectoriel anti-repetition        self.phase_drift_check(&mut ctx); // P0 : moniteur de derive de persona        if ctx.should_abort {
            return None;
        }
        self.phase_algorithm_request(&mut ctx).await;
        self.phase_pipeline(&mut ctx);
        self.phase_monologue(&mut ctx); // M2 : monologue interieur        self.phase_dissonance(&mut ctx);            // M3 : dissonance cognitive
        self.phase_imagery(&mut ctx).await;           // M9 : imagerie mentale
        self.phase_sentiments(&mut ctx);              // Sentiments (etats affectifs durables)
        self.phase_state_clustering(&mut ctx);         // PCA + K-Means etat cognitif
        self.phase_working_memory(&mut ctx).await;
        self.phase_memory_echo(&mut ctx).await;
        self.phase_reward_and_ethics(&mut ctx).await;
        self.phase_verify_predictions(&mut ctx);
        self.phase_maybe_ask_feedback(&mut ctx);
        self.phase_lora_collect(&mut ctx).await;
        self.phase_knowledge_bonus(&mut ctx).await;
        self.phase_thought_log(&mut ctx).await;
        self.phase_profiling(&mut ctx).await;
        self.phase_cognitive_trace(&mut ctx);
        self.phase_broadcast(&mut ctx).await;
        self.phase_metrics(&mut ctx);
        self.phase_learning(&mut ctx).await;
        self.phase_nn_learning(&mut ctx).await;
        self.phase_metacognition(&mut ctx).await;   // inclut M8+M10
        self.phase_self_critique(&mut ctx).await;  // Auto-critique reflexive (periodique)
        self.phase_personality_snapshot(&mut ctx).await;  // Portrait temporel (50 cycles)
        self.phase_introspection_journal(&mut ctx).await; // Journal introspectif (200 cycles)
        self.phase_desire_birth(&mut ctx).await;
        self.phase_self_modification(&mut ctx).await; // Auto-modification niveaux 1+2        self.phase_psychology(&mut ctx);
        self.phase_values(&mut ctx); // Character values (virtues)        self.phase_narrative(&mut ctx);             // M5: narrative identity
        self.phase_behavior_tree(&mut ctx);          // BT : instinct cognitif
        self.phase_game_algorithms(&mut ctx);       // GA2 : influence map, FSM, steering, GOAP
        self.phase_homeostasis(&mut ctx);

        // Filter internal technical terms before display
        ctx.thought_text = super::conversation::strip_internal_jargon(&ctx.thought_text);

        Some(ctx.thought_text)
    }
}
