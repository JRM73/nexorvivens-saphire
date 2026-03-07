// =============================================================================
// logging/trace.rs — CognitiveTrace: complete trace of one cognitive cycle
// =============================================================================
//
// Purpose: Provides a structure that incrementally accumulates data from each
//          stage of the cognitive pipeline (NLP, brain, consensus, chemistry,
//          emotion, consciousness, regulation, LLM, memory, and many more)
//          for a given cycle. Each field is a JSONB blob set via a dedicated
//          setter method, allowing pipeline stages to record their output
//          independently.
//
// Dependencies:
//   - chrono: timestamp for the trace
//   - serde: serialization/deserialization for database persistence
//
// Architectural placement:
//   Used by the cognitive pipeline to collect per-cycle diagnostics. The
//   completed trace is persisted to the `cognitive_traces` table in the
//   logs database via LogsDb::save_trace().
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Complete cognitive trace of a single cycle.
/// Accumulates data from each stage of the cognitive pipeline.
/// Each field holds a JSON blob populated by the corresponding pipeline stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveTrace {
    /// Cognitive cycle number
    pub cycle: u64,
    /// UTC timestamp when the trace was created
    pub timestamp: DateTime<Utc>,
    /// Source type (e.g., "conversation", "autonomous", "heartbeat")
    pub source_type: String,
    /// Raw input text that triggered this cycle
    pub input_text: String,
    /// NLP analysis results (intent, entities, sentiment)
    pub nlp_data: serde_json::Value,
    /// Brain region signals and activations
    pub brain_data: serde_json::Value,
    /// Consensus results from brain region voting
    pub consensus_data: serde_json::Value,
    /// Neurochemical state before the cycle
    pub chemistry_before: serde_json::Value,
    /// Neurochemical state after the cycle
    pub chemistry_after: serde_json::Value,
    /// Emotion detection and classification data
    pub emotion_data: serde_json::Value,
    /// Consciousness metrics (LZC, PCI, Phi*)
    pub consciousness_data: serde_json::Value,
    /// Moral regulation evaluation results
    pub regulation_data: serde_json::Value,
    /// LLM request/response data (model, tokens, latency)
    pub llm_data: serde_json::Value,
    /// Memory retrieval and storage operations
    pub memory_data: serde_json::Value,
    /// Heart module data (emotional heartbeat)
    pub heart_data: serde_json::Value,
    /// Body/physiology data (vital signs simulation)
    pub body_data: serde_json::Value,
    /// Ethical evaluation data
    pub ethics_data: serde_json::Value,
    /// Vital spark data (life force, genesis)
    pub vital_data: serde_json::Value,
    /// Intuition engine data (hunches, pattern matches)
    pub intuition_data: serde_json::Value,
    /// Premonition engine data (predictions, accuracy)
    pub premonition_data: serde_json::Value,
    /// Sensory data (Sensorium module)
    pub senses_data: serde_json::Value,
    /// Attention data (focus, fatigue, concentration)
    pub attention_data: serde_json::Value,
    /// Algorithm selection data (bandit choices)
    pub algorithm_data: serde_json::Value,
    /// Desire system data (active desires, top priority, needs)
    pub desire_data: serde_json::Value,
    /// Learning system data (lessons learned this cycle)
    pub learning_data: serde_json::Value,
    /// Healing system data (wounds, resilience)
    pub healing_data: serde_json::Value,
    /// Psychology data (6 psychological frameworks)
    pub psychology_data: serde_json::Value,
    /// Will/deliberation data
    pub will_data: serde_json::Value,
    /// Neural network learning data (vector learning)
    pub nn_learning_data: serde_json::Value,
    /// Subconscious processing data
    pub subconscious_data: serde_json::Value,
    /// Sleep system data
    pub sleep_data: serde_json::Value,
    // --- Advanced cognitive modules ---
    /// Theory of Mind data
    pub tom_data: serde_json::Value,
    /// Inner monologue data
    pub monologue_data: serde_json::Value,
    /// Cognitive dissonance data
    pub dissonance_data: serde_json::Value,
    /// Prospective memory data (future intentions)
    pub prospective_data: serde_json::Value,
    /// Narrative identity data
    pub narrative_data: serde_json::Value,
    /// Analogical reasoning data
    pub analogical_data: serde_json::Value,
    /// Cognitive load data
    pub cognitive_load_data: serde_json::Value,
    /// Mental imagery data
    pub imagery_data: serde_json::Value,
    /// Sentiment system data
    pub sentiments_data: serde_json::Value,
    /// Total duration of this cycle in milliseconds
    pub duration_ms: f32,
    /// Session identifier for grouping traces
    pub session_id: i64,
}

impl CognitiveTrace {
    /// Creates a new trace for a given cycle. All JSONB fields are initialized to empty objects.
    pub fn new(cycle: u64, source_type: &str, session_id: i64) -> Self {
        Self {
            cycle,
            timestamp: Utc::now(),
            source_type: source_type.to_string(),
            input_text: String::new(),
            nlp_data: serde_json::json!({}),
            brain_data: serde_json::json!({}),
            consensus_data: serde_json::json!({}),
            chemistry_before: serde_json::json!({}),
            chemistry_after: serde_json::json!({}),
            emotion_data: serde_json::json!({}),
            consciousness_data: serde_json::json!({}),
            regulation_data: serde_json::json!({}),
            llm_data: serde_json::json!({}),
            memory_data: serde_json::json!({}),
            heart_data: serde_json::json!({}),
            body_data: serde_json::json!({}),
            ethics_data: serde_json::json!({}),
            vital_data: serde_json::json!({}),
            intuition_data: serde_json::json!({}),
            premonition_data: serde_json::json!({}),
            senses_data: serde_json::json!({}),
            attention_data: serde_json::json!({}),
            algorithm_data: serde_json::json!({}),
            desire_data: serde_json::json!({}),
            learning_data: serde_json::json!({}),
            healing_data: serde_json::json!({}),
            psychology_data: serde_json::json!({}),
            will_data: serde_json::json!({}),
            nn_learning_data: serde_json::json!({}),
            subconscious_data: serde_json::json!({}),
            sleep_data: serde_json::json!({}),
            tom_data: serde_json::json!({}),
            monologue_data: serde_json::json!({}),
            dissonance_data: serde_json::json!({}),
            prospective_data: serde_json::json!({}),
            narrative_data: serde_json::json!({}),
            analogical_data: serde_json::json!({}),
            cognitive_load_data: serde_json::json!({}),
            imagery_data: serde_json::json!({}),
            sentiments_data: serde_json::json!({}),
            duration_ms: 0.0,
            session_id,
        }
    }

    /// Records the input text.
    pub fn set_input(&mut self, text: &str) {
        self.input_text = text.to_string();
    }

    /// Records the NLP analysis data.
    pub fn set_nlp(&mut self, data: serde_json::Value) {
        self.nlp_data = data;
    }

    /// Records the brain module signals.
    pub fn set_brain(&mut self, data: serde_json::Value) {
        self.brain_data = data;
    }

    /// Records the consensus result.
    pub fn set_consensus(&mut self, data: serde_json::Value) {
        self.consensus_data = data;
    }

    /// Records the neurochemistry state before the cycle.
    pub fn set_chemistry_before(&mut self, data: serde_json::Value) {
        self.chemistry_before = data;
    }

    /// Records the neurochemistry state after the cycle.
    pub fn set_chemistry_after(&mut self, data: serde_json::Value) {
        self.chemistry_after = data;
    }

    /// Records the emotion data.
    pub fn set_emotion(&mut self, data: serde_json::Value) {
        self.emotion_data = data;
    }

    /// Records the consciousness metrics data.
    pub fn set_consciousness(&mut self, data: serde_json::Value) {
        self.consciousness_data = data;
    }

    /// Records the moral regulation data.
    pub fn set_regulation(&mut self, data: serde_json::Value) {
        self.regulation_data = data;
    }

    /// Records the LLM request/response data.
    pub fn set_llm(&mut self, data: serde_json::Value) {
        self.llm_data = data;
    }

    /// Records the memory operations data.
    pub fn set_memory(&mut self, data: serde_json::Value) {
        self.memory_data = data;
    }

    /// Records the heart module data.
    pub fn set_heart(&mut self, data: serde_json::Value) {
        self.heart_data = data;
    }

    /// Records the body/physiology data.
    pub fn set_body(&mut self, data: serde_json::Value) {
        self.body_data = data;
    }

    /// Records the ethical evaluation data.
    pub fn set_ethics(&mut self, data: serde_json::Value) {
        self.ethics_data = data;
    }

    /// Records the vital spark (life force) data.
    pub fn set_vital(&mut self, data: serde_json::Value) {
        self.vital_data = data;
    }

    /// Records the intuition engine data.
    pub fn set_intuition(&mut self, data: serde_json::Value) {
        self.intuition_data = data;
    }

    /// Records the premonition engine data.
    pub fn set_premonition(&mut self, data: serde_json::Value) {
        self.premonition_data = data;
    }

    /// Records the sensory data (Sensorium).
    pub fn set_senses(&mut self, data: serde_json::Value) {
        self.senses_data = data;
    }

    /// Records the attention data (focus, fatigue, concentration).
    pub fn set_attention(&mut self, data: serde_json::Value) {
        self.attention_data = data;
    }

    /// Records the desire system data (active desires, top priority, needs).
    pub fn set_desires(&mut self, data: serde_json::Value) {
        self.desire_data = data;
    }

    /// Records the learning data (lessons learned this cycle).
    pub fn set_learning(&mut self, data: serde_json::Value) {
        self.learning_data = data;
    }

    /// Records the healing data (wounds, resilience).
    pub fn set_healing(&mut self, data: serde_json::Value) {
        self.healing_data = data;
    }

    /// Records the psychology data (6 psychological frameworks).
    pub fn set_psychology(&mut self, data: serde_json::Value) {
        self.psychology_data = data;
    }

    /// Records the will/deliberation data.
    pub fn set_will(&mut self, data: serde_json::Value) {
        self.will_data = data;
    }

    /// Records the neural network / vector learning data.
    pub fn set_nn_learning(&mut self, data: serde_json::Value) {
        self.nn_learning_data = data;
    }

    /// Records the subconscious processing data.
    pub fn set_subconscious(&mut self, data: serde_json::Value) {
        self.subconscious_data = data;
    }

    /// Records the sleep system data.
    pub fn set_sleep(&mut self, data: serde_json::Value) {
        self.sleep_data = data;
    }

    /// Records the Theory of Mind data.
    pub fn set_tom(&mut self, data: serde_json::Value) {
        self.tom_data = data;
    }

    /// Records the inner monologue data.
    pub fn set_monologue(&mut self, data: serde_json::Value) {
        self.monologue_data = data;
    }

    /// Records the cognitive dissonance data.
    pub fn set_dissonance(&mut self, data: serde_json::Value) {
        self.dissonance_data = data;
    }

    /// Records the prospective memory data.
    pub fn set_prospective(&mut self, data: serde_json::Value) {
        self.prospective_data = data;
    }

    /// Records the narrative identity data.
    pub fn set_narrative(&mut self, data: serde_json::Value) {
        self.narrative_data = data;
    }

    /// Records the analogical reasoning data.
    pub fn set_analogical(&mut self, data: serde_json::Value) {
        self.analogical_data = data;
    }

    /// Records the cognitive load data.
    pub fn set_cognitive_load(&mut self, data: serde_json::Value) {
        self.cognitive_load_data = data;
    }

    /// Records the mental imagery data.
    pub fn set_imagery(&mut self, data: serde_json::Value) {
        self.imagery_data = data;
    }

    /// Records the sentiment system data.
    pub fn set_sentiments(&mut self, data: serde_json::Value) {
        self.sentiments_data = data;
    }

    /// Records the total cycle duration in milliseconds.
    pub fn set_duration(&mut self, ms: f32) {
        self.duration_ms = ms;
    }
}
