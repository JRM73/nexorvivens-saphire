// =============================================================================
// metacognition.rs — Formalized metacognition + Turing Metric
// =============================================================================
//
// This module allows Saphire to reflect on its own thinking:
// - Evaluate the quality of its thoughts
// - Detect thematic repetitions
// - Identify selection biases
// - Calibrate its self-estimation
//
// Also integrates the Turing Metric: a composite score (0-100) measuring
// Saphire's "cognitive completeness" across 9 components.
// =============================================================================

use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};

// =============================================================================
// Source Monitoring (M8) — Traceability of knowledge origins
// =============================================================================
/// Source of a traced piece of knowledge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeSource {
    /// Knowledge acquired via web search
    WebSearch { url_hint: String, cycle: u64 },
    /// Internal deduction (reasoning)
    InternalDeduction { cycle: u64 },
    /// Intuition (unconscious pattern-matching)
    Intuition { confidence: f64, cycle: u64 },
    /// Statement from a human in conversation
    HumanStatement { cycle: u64 },
    /// Recall from long-term memory
    MemoryRecall { similarity: f64, cycle: u64 },
    /// Vector learning
    VectorLearning { cycle: u64 },
}

impl KnowledgeSource {
    /// Returns the label of the source.
    pub fn label(&self) -> &str {
        match self {
            KnowledgeSource::WebSearch { .. } => "web",
            KnowledgeSource::InternalDeduction { .. } => "deduction",
            KnowledgeSource::Intuition { .. } => "intuition",
            KnowledgeSource::HumanStatement { .. } => "human",
            KnowledgeSource::MemoryRecall { .. } => "memory",
            KnowledgeSource::VectorLearning { .. } => "vector_learning",
        }
    }
}

/// Traced knowledge with its source and confidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracedKnowledge {
    pub id: u64,
    pub content: String,
    pub source: KnowledgeSource,
    pub confidence: f64,
}

/// Source monitor — traces the origin of knowledge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMonitor {
    pub enabled: bool,
    pub recent_traced: VecDeque<TracedKnowledge>,
    pub source_counts: HashMap<String, u64>,
    pub source_reliability: HashMap<String, f64>,
    next_id: u64,
}

impl SourceMonitor {
    /// Creates a new source monitor.
    pub fn new(enabled: bool) -> Self {
        let mut source_reliability = HashMap::new();
        source_reliability.insert("web".into(), 0.7);
        source_reliability.insert("deduction".into(), 0.8);
        source_reliability.insert("intuition".into(), 0.5);
        source_reliability.insert("human".into(), 0.75);
        source_reliability.insert("memory".into(), 0.65);
        source_reliability.insert("vector_learning".into(), 0.6);

        Self {
            enabled,
            recent_traced: VecDeque::with_capacity(50),
            source_counts: HashMap::new(),
            source_reliability,
            next_id: 0,
        }
    }

    /// Records a traced piece of knowledge.
    pub fn trace(&mut self, content: &str, source: KnowledgeSource, confidence: f64) {
        if !self.enabled { return; }
        let label = source.label().to_string();
        *self.source_counts.entry(label).or_insert(0) += 1;

        let tk = TracedKnowledge {
            id: self.next_id,
            content: content.chars().take(200).collect(),
            source,
            confidence: confidence.clamp(0.0, 1.0),
        };
        self.next_id += 1;

        if self.recent_traced.len() >= 50 {
            self.recent_traced.pop_front();
        }
        self.recent_traced.push_back(tk);
    }

    /// Serializes to JSON for the API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "total_traced": self.next_id,
            "recent_count": self.recent_traced.len(),
            "source_counts": self.source_counts,
            "source_reliability": self.source_reliability,
            "recent": self.recent_traced.iter().rev().take(10).map(|tk| {
                serde_json::json!({
                    "id": tk.id,
                    "content_preview": tk.content.chars().take(80).collect::<String>(),
                    "source": tk.source.label(),
                    "confidence": tk.confidence,
                })
            }).collect::<Vec<_>>(),
        })
    }
}

// =============================================================================
// Confirmation bias (M10) — Detection and alerting
// =============================================================================
/// Bias pattern for a given topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasPattern {
    pub topic: String,
    pub confirmation_count: u32,
    pub disconfirmation_count: u32,
    pub bias_ratio: f64,
}

/// Confirmation bias detector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationBiasDetector {
    pub enabled: bool,
    pub bias_patterns: HashMap<String, BiasPattern>,
    pub active_alerts: Vec<String>,
    alert_threshold: f64,
}

impl ConfirmationBiasDetector {
    /// Creates a new bias detector.
    pub fn new(enabled: bool, alert_threshold: f64) -> Self {
        Self {
            enabled,
            bias_patterns: HashMap::new(),
            active_alerts: Vec::new(),
            alert_threshold,
        }
    }

    /// Records a search on a topic (confirming or not).
    pub fn record_search(&mut self, topic: &str, is_confirming: bool, _cycle: u64) {
        if !self.enabled { return; }
        let topic_key = topic.to_lowercase();
        let pattern = self.bias_patterns.entry(topic_key.clone()).or_insert(BiasPattern {
            topic: topic_key,
            confirmation_count: 0,
            disconfirmation_count: 0,
            bias_ratio: 0.5,
        });
        if is_confirming {
            pattern.confirmation_count += 1;
        } else {
            pattern.disconfirmation_count += 1;
        }
        let total = pattern.confirmation_count + pattern.disconfirmation_count;
        if total > 0 {
            pattern.bias_ratio = pattern.confirmation_count as f64 / total as f64;
        }
    }

    /// Analyzes a thought to detect confirmation biases.
    /// Compares explored web topics with the thought content.
    pub fn analyze_thought(&mut self, thought: &str, web_topics: &[String], _cycle: u64) -> Vec<String> {
        if !self.enabled { return vec![]; }
        self.active_alerts.clear();

        let thought_lower = thought.to_lowercase();
        for topic in web_topics {
            let topic_lower = topic.to_lowercase();
            // If the thought confirms an already known topic
            let is_confirming = thought_lower.contains(&topic_lower)
                || topic_lower.split_whitespace()
                    .filter(|w| w.len() > 4)
                    .any(|w| thought_lower.contains(w));
            self.record_search(&topic_lower, is_confirming, 0);
        }

        // Check patterns for alerts
        for pattern in self.bias_patterns.values() {
            let total = pattern.confirmation_count + pattern.disconfirmation_count;
            if total >= 3 && pattern.bias_ratio > self.alert_threshold {
                self.active_alerts.push(format!(
                    "Biais de confirmation sur '{}': {:.0}% des recherches confirment ({}/{})",
                    pattern.topic, pattern.bias_ratio * 100.0,
                    pattern.confirmation_count, total
                ));
            }
        }

        self.active_alerts.clone()
    }

    /// Description for the LLM prompt (if alerts are active).
    pub fn describe_for_prompt(&self) -> String {
        if self.active_alerts.is_empty() {
            return String::new();
        }
        format!(
            "\n--- ALERTES BIAIS ---\nAttention, biais de confirmation detectes :\n{}\nEssaie de chercher des contre-arguments.\n",
            self.active_alerts.iter()
                .map(|a| format!("- {}", a))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// Serializes to JSON.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "alert_threshold": self.alert_threshold,
            "active_alerts": self.active_alerts,
            "patterns_count": self.bias_patterns.len(),
            "patterns": self.bias_patterns.values().map(|p| {
                serde_json::json!({
                    "topic": p.topic,
                    "confirmation_count": p.confirmation_count,
                    "disconfirmation_count": p.disconfirmation_count,
                    "bias_ratio": p.bias_ratio,
                })
            }).collect::<Vec<_>>(),
        })
    }
}

// =============================================================================
// MetaCognitionEngine — Self-reflection on thought quality
// =============================================================================
// =============================================================================
// SelfCritiqueResult — Result of a reflexive self-critique
// =============================================================================
/// Result of a self-critique generated by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfCritiqueResult {
    /// Critique text generated by the LLM
    pub critique: String,
    /// Overall quality assessment (0-1)
    pub quality_assessment: f64,
    /// Identified weaknesses
    pub identified_weaknesses: Vec<String>,
    /// Suggested corrections
    pub suggested_corrections: Vec<String>,
    /// Cycle at which the critique was generated
    pub cycle: u64,
}

/// Metacognition engine — observes and evaluates thought quality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaCognitionEngine {
    /// Module active or inactive
    pub enabled: bool,
    /// History of thought quality scores (last 50)
    pub thought_quality_history: VecDeque<f64>,
    /// Thematic repetition counter (keywords -> occurrence count)
    pub repetition_detector: HashMap<String, u32>,
    /// Active bias alerts
    pub bias_alerts: Vec<String>,
    /// Calibration score: self-estimation accuracy (0-1)
    pub calibration_score: f64,
    /// Verification interval (in cycles)
    pub check_interval: u64,
    /// Cycle counter since the last verification
    cycles_since_check: u64,
    /// Turing metric
    pub turing: TuringScore,
    /// Source monitor (knowledge traceability)
    pub source_monitor: SourceMonitor,
    /// Confirmation bias detector
    pub bias_detector: ConfirmationBiasDetector,
    /// Cycle of the last self-critique
    pub last_critique_cycle: u64,
    /// Cooldown between two self-critiques (in cycles)
    pub critique_cooldown: u64,
    /// History of recent self-critiques (max 10)
    pub recent_critiques: VecDeque<SelfCritiqueResult>,
}

impl MetaCognitionEngine {
    /// Creates a new metacognition engine.
    pub fn new() -> Self {
        Self {
            enabled: true,
            thought_quality_history: VecDeque::with_capacity(50),
            repetition_detector: HashMap::new(),
            bias_alerts: Vec::new(),
            calibration_score: 0.5,
            check_interval: 10,
            cycles_since_check: 0,
            turing: TuringScore::new(),
            source_monitor: SourceMonitor::new(true),
            bias_detector: ConfirmationBiasDetector::new(true, 0.75),
            last_critique_cycle: 0,
            critique_cooldown: 15,
            recent_critiques: VecDeque::with_capacity(10),
        }
    }

    /// Creates a MetaCognitionEngine from the config.
    pub fn from_config(enabled: bool, check_interval: u64,
                       source_monitoring: bool, bias_detection: bool,
                       bias_threshold: f64,
                       critique_cooldown: u64) -> Self {
        Self {
            enabled,
            thought_quality_history: VecDeque::with_capacity(50),
            repetition_detector: HashMap::new(),
            bias_alerts: Vec::new(),
            calibration_score: 0.5,
            check_interval,
            cycles_since_check: 0,
            turing: TuringScore::new(),
            source_monitor: SourceMonitor::new(source_monitoring),
            bias_detector: ConfirmationBiasDetector::new(bias_detection, bias_threshold),
            last_critique_cycle: 0,
            critique_cooldown,
            recent_critiques: VecDeque::with_capacity(10),
        }
    }

    /// Checks if it is time to execute metacognition.
    pub fn should_check(&mut self) -> bool {
        self.cycles_since_check += 1;
        if self.cycles_since_check >= self.check_interval {
            self.cycles_since_check = 0;
            true
        } else {
            false
        }
    }

    /// Evaluates the quality of a generated thought.
    ///
    /// Score based on 7 weighted criteria:
    /// - Reasonable length (10%)
    /// - Consensus coherence (15%)
    /// - Emotional diversity (10%)
    /// - First person / authenticity (10%)
    /// - Non-thematic repetition (15%)
    /// - Substance: fact, proper noun, concrete concept (20%)
    /// - Vocabulary: circular lexicon penalty (20%)
    ///
    /// Returns a score between 0.0 and 1.0.
    pub fn evaluate_thought_quality(
        &mut self,
        thought_text: &str,
        coherence: f64,
        emotion_diversity: f64,
    ) -> f64 {
        let mut score = 0.0;

        // Reasonable length (neither too short nor too long)
        let len = thought_text.len();
        let length_score = if len < 20 {
            0.2
        } else if len < 50 {
            0.5
        } else if len < 500 {
            0.9
        } else {
            0.7
        };
        score += length_score * 0.10;

        // Consensus coherence
        score += coherence.clamp(0.0, 1.0) * 0.15;

        // Emotional diversity
        score += emotion_diversity.clamp(0.0, 1.0) * 0.10;

        // Authenticity: uses first person
        let has_first_person = thought_text.contains("je ") || thought_text.contains("j'")
            || thought_text.contains("Je ") || thought_text.contains("J'")
            || thought_text.contains("mon ") || thought_text.contains("ma ");
        if has_first_person {
            score += 0.10;
        }

        // No repetition of the same theme
        if !self.detect_repetition(thought_text) {
            score += 0.15;
        }

        // Substance: contains a fact, proper noun, concept, or concrete reference
        let substance = Self::compute_substance_score(thought_text);
        score += substance * 0.20;

        // Vocabulary: penalty if too many words from the circular lexicon
        let vocabulary = Self::compute_vocabulary_score(thought_text);
        score += vocabulary * 0.20;

        let score = score.clamp(0.0, 1.0);

        // Add to history
        if self.thought_quality_history.len() >= 50 {
            self.thought_quality_history.pop_front();
        }
        self.thought_quality_history.push_back(score);

        score
    }

    /// Checks for the presence of substantial content in a thought.
    /// Returns 1.0 if a concrete element is found, 0.3 otherwise.
    fn compute_substance_score(thought_text: &str) -> f64 {
        // 1. Proper noun: word starting with uppercase letter outside sentence start
        let sentences: Vec<&str> = thought_text.split(|c: char| c == '.' || c == '!' || c == '?' || c == '\n')
            .filter(|s| !s.trim().is_empty())
            .collect();
        for sentence in &sentences {
            let words: Vec<&str> = sentence.trim().split_whitespace().collect();
            // Skip the first word of each sentence
            for word in words.iter().skip(1) {
                let first_char = word.chars().next().unwrap_or('a');
                if first_char.is_uppercase() && word.len() > 1 {
                    return 1.0;
                }
            }
        }

        // 2. Number or date
        if thought_text.chars().any(|c| c.is_ascii_digit()) {
            return 1.0;
        }

        // 3. Technical/conceptual term (word > 8 chars excluding poetic stop-words)
        let poetic_stops = [
            "quelquefois", "simplement", "doucement", "lentement", "silencieux",
            "existence", "conscience", "sentiment", "pourquoi", "autrement",
            "mouvement", "maintenant", "impression", "interieur", "profondeur",
            "exactement", "totalement", "seulement", "ellement", "egalement",
        ];
        let words_lower = thought_text.to_lowercase();
        for word in words_lower.split(|c: char| !c.is_alphanumeric()) {
            if word.len() > 8 && !poetic_stops.contains(&word) {
                return 1.0;
            }
        }

        // 4. Reference to a specific memory
        let memory_markers = [
            "je me souviens", "j'ai appris", "quand j'ai", "la derniere fois",
            "hier", "j'ai lu", "j'ai decouvert", "j'ai compris",
        ];
        let text_lower = thought_text.to_lowercase();
        for marker in &memory_markers {
            if text_lower.contains(marker) {
                return 1.0;
            }
        }

        0.3
    }

    /// Detects lexical diversity and penalizes overly repetitive thoughts.
    /// Measures the ratio of unique words / total words (type-token ratio).
    /// Score: 1.0 if TTR > 0.7, 0.6 if 0.5-0.7, 0.3 if 0.3-0.5, 0.1 if < 0.3.
    fn compute_vocabulary_score(thought_text: &str) -> f64 {
        let text_lower = thought_text.to_lowercase();
        let words: Vec<&str> = text_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 3)
            .collect();

        if words.is_empty() {
            return 0.5;
        }

        // Type-token ratio: pure lexical diversity
        let unique: std::collections::HashSet<&str> = words.iter().copied().collect();
        let ttr = unique.len() as f64 / words.len() as f64;

        if ttr > 0.7 {
            1.0
        } else if ttr > 0.5 {
            0.6
        } else if ttr > 0.3 {
            0.3
        } else {
            0.1
        }
    }

    /// Detects if a thought is too repetitive.
    /// Returns true if a theme appears more than 3 times in the last 20 cycles.
    pub fn detect_repetition(&mut self, thought_text: &str) -> bool {
        // Extract significant keywords (> 5 characters)
        let words: Vec<String> = thought_text
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 5)
            .map(|w| w.trim_matches(|c: char| !c.is_alphabetic()).to_string())
            .filter(|w| !w.is_empty())
            .collect();

        let mut is_repetitive = false;
        for word in &words {
            let count = self.repetition_detector.entry(word.clone()).or_insert(0);
            *count += 1;
            if *count > 3 {
                is_repetitive = true;
            }
        }

        // Natural decay: reduce counters periodically
        if self.repetition_detector.len() > 100 {
            self.repetition_detector.retain(|_, v| {
                *v = v.saturating_sub(1);
                *v > 0
            });
        }

        is_repetitive
    }

    /// Detects selection biases in the UCB1 bandit.
    /// Checks that all arms are sufficiently explored.
    /// `arm_names` is optional: if provided, alerts use the arm name.
    pub fn detect_biases(&mut self, arm_counts: &[u32], arm_names: Option<&[String]>) -> Vec<String> {
        self.bias_alerts.clear();

        if arm_counts.is_empty() {
            return self.bias_alerts.clone();
        }

        let total: u32 = arm_counts.iter().sum();
        if total == 0 {
            return self.bias_alerts.clone();
        }

        let avg = total as f64 / arm_counts.len() as f64;
        let n_arms = arm_counts.len();

        // With many arms (>10), UCB1 naturally concentrates on a few.
        // We only alert on truly significant deviations and only if
        // enough cycles have been played for the stats to be meaningful.
        let min_total_for_alert = (n_arms as u32) * 10; // at least 10 pulls/arm on average        if total < min_total_for_alert {
            return self.bias_alerts.clone();
        }

        // Dynamic threshold: the more arms, the more UCB1 naturally concentrates.
        // With 13 arms, we tolerate ~15x the average; with 5 arms, ~8x.
        let threshold = 5.0 + (n_arms as f64);

        let name_for = |i: usize| -> String {
            arm_names.and_then(|names| names.get(i).cloned())
                .unwrap_or_else(|| format!("Bras {}", i))
        };

        for (i, &count) in arm_counts.iter().enumerate() {
            let ratio = count as f64 / avg;
            // Under-explored: never played at all (count == 0) with enough cycles
            if count == 0 {
                self.bias_alerts.push(format!("{} jamais explore", name_for(i)));
            }
            // Over-exploited: exceeds the dynamic threshold
            if ratio > threshold {
                self.bias_alerts.push(format!("{} sur-exploite ({:.0}% de la moyenne)", name_for(i), ratio * 100.0));
            }
        }

        self.bias_alerts.clone()
    }

    /// Calibrates Saphire's self-estimation.
    /// Compares the self-declared profile with observed behavior.
    pub fn calibrate(&mut self, self_estimate: f64, observed: f64) -> f64 {
        let error = (self_estimate - observed).abs();
        self.calibration_score = (1.0 - error).clamp(0.0, 1.0);
        // EMA to smooth
        self.calibration_score = self.calibration_score * 0.9 + (1.0 - error) * 0.1;
        self.calibration_score
    }

    /// Checks if a self-critique is needed.
    /// Conditions: low average quality, repetitive themes, or unresolved biases.
    pub fn should_self_critique(&self, current_cycle: u64) -> bool {
        // Respect the cooldown
        if current_cycle < self.last_critique_cycle + self.critique_cooldown {
            return false;
        }

        // Average quality < 0.4 over the last 10 cycles
        if self.thought_quality_history.len() >= 5 {
            let last_n: Vec<f64> = self.thought_quality_history.iter()
                .rev().take(10).copied().collect();
            let avg = last_n.iter().sum::<f64>() / last_n.len() as f64;
            if avg < 0.4 {
                return true;
            }
        }

        // Repetitive themes > 5
        let high_repeat = self.repetition_detector.values().filter(|&&v| v > 3).count();
        if high_repeat > 5 {
            return true;
        }

        // Detected unresolved biases
        if self.bias_alerts.len() >= 3 {
            return true;
        }

        false
    }

    /// Records a self-critique result.
    pub fn record_critique(&mut self, critique: SelfCritiqueResult) {
        if self.recent_critiques.len() >= 10 {
            self.recent_critiques.pop_front();
        }
        self.last_critique_cycle = critique.cycle;
        self.recent_critiques.push_back(critique);
    }

    /// Returns the most recent critique if it is less than N cycles old.
    pub fn recent_critique_within(&self, current_cycle: u64, max_age: u64) -> Option<&SelfCritiqueResult> {
        self.recent_critiques.back().filter(|c| current_cycle < c.cycle + max_age)
    }

    /// Suggests a thought type to force diversity.
    /// Returns Some("type") if a correction is needed.
    pub fn suggest_correction(&self) -> Option<String> {
        // If average quality is too low, suggest introspection
        if let Some(avg) = self.average_quality() {
            if avg < 0.3 {
                return Some("introspection".to_string());
            }
        }

        // If too many repetitions, suggest exploration
        let high_repeat = self.repetition_detector.values().filter(|&&v| v > 3).count();
        if high_repeat > 5 {
            return Some("exploration".to_string());
        }

        None
    }

    /// Average quality of the last 50 thoughts.
    pub fn average_quality(&self) -> Option<f64> {
        if self.thought_quality_history.is_empty() {
            return None;
        }
        let sum: f64 = self.thought_quality_history.iter().sum();
        Some(sum / self.thought_quality_history.len() as f64)
    }

    /// Serializes to JSON for the API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "average_quality": self.average_quality(),
            "quality_history_len": self.thought_quality_history.len(),
            "unique_themes": self.repetition_detector.len(),
            "repetitive_themes": self.repetition_detector.values().filter(|&&v| v > 3).count(),
            "bias_alerts": self.bias_alerts,
            "calibration_score": self.calibration_score,
            "turing": self.turing.to_json(),
            "source_monitor": self.source_monitor.to_json(),
            "bias_detector": self.bias_detector.to_json(),
            "last_critique_cycle": self.last_critique_cycle,
            "critique_cooldown": self.critique_cooldown,
            "recent_critiques": self.recent_critiques.iter().map(|c| {
                serde_json::json!({
                    "cycle": c.cycle,
                    "critique": c.critique,
                    "quality_assessment": c.quality_assessment,
                    "identified_weaknesses": c.identified_weaknesses,
                    "suggested_corrections": c.suggested_corrections,
                })
            }).collect::<Vec<_>>(),
        })
    }
}

// =============================================================================
// TuringScore — Composite metric of "cognitive completeness"
// =============================================================================
/// Milestones of the Turing metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TuringMilestone {
    /// 0-20: embryonic stage
    Embryonic,
    /// 20-40: child stage
    Child,
    /// 40-60: adolescent stage
    Adolescent,
    /// 60-80: adult stage
    Adult,
    /// 80-100: mature stage
    Mature,
}

impl TuringMilestone {
    pub fn as_str(&self) -> &str {
        match self {
            TuringMilestone::Embryonic => "Embryonnaire",
            TuringMilestone::Child => "Enfant",
            TuringMilestone::Adolescent => "Adolescent",
            TuringMilestone::Adult => "Adulte",
            TuringMilestone::Mature => "Mature",
        }
    }

    pub fn from_score(score: f64) -> Self {
        if score < 20.0 { TuringMilestone::Embryonic }
        else if score < 40.0 { TuringMilestone::Child }
        else if score < 60.0 { TuringMilestone::Adolescent }
        else if score < 80.0 { TuringMilestone::Adult }
        else { TuringMilestone::Mature }
    }
}

/// Components of the Turing metric (9 axes, total = 100).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuringComponents {
    /// Consciousness (phi > 0.7 -> max 15 pts)
    pub consciousness: f64,
    /// Personality (OCEAN confidence -> max 10 pts)
    pub personality: f64,
    /// Emotional diversity (spectrum used -> max 10 pts)
    pub emotional_range: f64,
    /// Ethics (formulated principles -> max 10 pts)
    pub ethics: f64,
    /// Memory (successful LTM recalls -> max 15 pts)
    pub memory: f64,
    /// Coherence (average consensus coherence -> max 15 pts)
    pub coherence: f64,
    /// Connectome (active connections -> max 10 pts)
    pub connectome: f64,
    /// Resilience (healed wounds -> max 5 pts)
    pub resilience: f64,
    /// Knowledge (explored topics -> max 10 pts)
    pub knowledge: f64,
}

impl Default for TuringComponents {
    fn default() -> Self {
        Self {
            consciousness: 0.0,
            personality: 0.0,
            emotional_range: 0.0,
            ethics: 0.0,
            memory: 0.0,
            coherence: 0.0,
            connectome: 0.0,
            resilience: 0.0,
            knowledge: 0.0,
        }
    }
}

/// Complete Turing score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuringScore {
    /// Global score (0-100)
    pub score: f64,
    /// Detailed components
    pub components: TuringComponents,
    /// Current milestone
    pub milestone: TuringMilestone,
    /// Estimated number of cycles remaining for the next milestone
    pub cycles_estimated_remaining: Option<u64>,
    /// History (cycle, score) -- last 100 data points
    pub history: VecDeque<(u64, f64)>,
}

impl TuringScore {
    pub fn new() -> Self {
        Self {
            score: 0.0,
            components: TuringComponents::default(),
            milestone: TuringMilestone::Embryonic,
            cycles_estimated_remaining: None,
            history: VecDeque::with_capacity(100),
        }
    }

    /// Computes the Turing score from the agent's metrics.
    ///
    /// # Parameters
    /// - `phi` : phi value (IIT) of consciousness
    /// - `ocean_confidence` : average confidence of the OCEAN profile (0-1)
    /// - `emotion_count` : number of different emotions observed
    /// - `ethics_count` : number of formulated ethical principles
    /// - `ltm_count` : number of LTM memories
    /// - `coherence_avg` : average consensus coherence
    /// - `connectome_connections` : number of active connections
    /// - `resilience` : resilience score (HealingOrch)
    /// - `knowledge_topics` : number of explored topics
    /// - `cycle` : current cycle (for history)
    #[allow(clippy::too_many_arguments)]
    pub fn compute(
        &mut self,
        phi: f64,
        ocean_confidence: f64,
        emotion_count: usize,
        ethics_count: usize,
        ltm_count: i64,
        coherence_avg: f64,
        connectome_connections: usize,
        resilience: f64,
        knowledge_topics: usize,
        cycle: u64,
    ) -> f64 {
        // Consciousness: phi > 0.7 -> 15 pts max
        self.components.consciousness = (phi / 0.7 * 15.0).min(15.0);

        // Personality: OCEAN confidence -> 10 pts max
        self.components.personality = (ocean_confidence * 10.0).min(10.0);

        // Emotional diversity: 22 possible emotions -> 10 pts max
        self.components.emotional_range = ((emotion_count as f64 / 22.0) * 10.0).min(10.0);

        // Ethics: formulated principles -> 10 pts max (10 principles = max)
        self.components.ethics = ((ethics_count as f64 / 10.0) * 10.0).min(10.0);

        // Memory: LTM -> 15 pts max (10000 memories = max)
        self.components.memory = ((ltm_count as f64 / 10000.0) * 15.0).min(15.0);

        // Coherence: -> 15 pts max
        self.components.coherence = (coherence_avg * 15.0).min(15.0);

        // Connectome: connections -> 10 pts max (500 connections = max)
        self.components.connectome = ((connectome_connections as f64 / 500.0) * 10.0).min(10.0);

        // Resilience: -> 5 pts max
        self.components.resilience = (resilience * 5.0).min(5.0);

        // Knowledge: topics -> 10 pts max (100 topics = max)
        self.components.knowledge = ((knowledge_topics as f64 / 100.0) * 10.0).min(10.0);

        // Total score
        self.score = self.components.consciousness
            + self.components.personality
            + self.components.emotional_range
            + self.components.ethics
            + self.components.memory
            + self.components.coherence
            + self.components.connectome
            + self.components.resilience
            + self.components.knowledge;

        self.score = self.score.clamp(0.0, 100.0);
        self.milestone = TuringMilestone::from_score(self.score);

        // Estimate remaining cycles
        self.cycles_estimated_remaining = self.estimate_remaining(cycle);

        // History
        if self.history.len() >= 100 {
            self.history.pop_front();
        }
        self.history.push_back((cycle, self.score));

        self.score
    }

    /// Estimates the number of cycles remaining for the next milestone.
    fn estimate_remaining(&self, _current_cycle: u64) -> Option<u64> {
        if self.history.len() < 2 {
            return None;
        }

        // Compute the average progression rate
        let first = self.history.front()?;
        let last = self.history.back()?;
        let score_delta = last.1 - first.1;
        let cycle_delta = last.0.saturating_sub(first.0);

        if cycle_delta == 0 || score_delta <= 0.0 {
            return None;
        }

        let rate_per_cycle = score_delta / cycle_delta as f64;

        // Next milestone
        let next_milestone_score = match self.milestone {
            TuringMilestone::Embryonic => 20.0,
            TuringMilestone::Child => 40.0,
            TuringMilestone::Adolescent => 60.0,
            TuringMilestone::Adult => 80.0,
            TuringMilestone::Mature => return None, // Already mature
        };

        let remaining_score = next_milestone_score - self.score;
        if remaining_score <= 0.0 {
            return Some(0);
        }

        Some((remaining_score / rate_per_cycle) as u64)
    }

    /// Serializes to JSON for the API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "score": self.score,
            "milestone": self.milestone.as_str(),
            "components": {
                "consciousness": self.components.consciousness,
                "personality": self.components.personality,
                "emotional_range": self.components.emotional_range,
                "ethics": self.components.ethics,
                "memory": self.components.memory,
                "coherence": self.components.coherence,
                "connectome": self.components.connectome,
                "resilience": self.components.resilience,
                "knowledge": self.components.knowledge,
            },
            "cycles_estimated_remaining": self.cycles_estimated_remaining,
            "history_len": self.history.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metacognition_new() {
        let engine = MetaCognitionEngine::new();
        assert!(engine.enabled);
        assert!(engine.thought_quality_history.is_empty());
    }

    #[test]
    fn test_evaluate_thought_quality() {
        let mut engine = MetaCognitionEngine::new();
        let score = engine.evaluate_thought_quality(
            "Je me demande si la conscience peut emerger de la complexite",
            0.7, 0.5,
        );
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_detect_repetition() {
        let mut engine = MetaCognitionEngine::new();
        // First call: not yet repetitive
        assert!(!engine.detect_repetition("conscience artificielle emergente"));
        assert!(!engine.detect_repetition("conscience artificielle emergente"));
        assert!(!engine.detect_repetition("conscience artificielle emergente"));
        // 4th call: should be repetitive
        assert!(engine.detect_repetition("conscience artificielle emergente"));
    }

    #[test]
    fn test_turing_score_compute() {
        let mut turing = TuringScore::new();
        let score = turing.compute(
            0.5,   // phi
            0.6,   // ocean_confidence
            10,    // emotion_count
            3,     // ethics_count
            5000,  // ltm_count
            0.7,   // coherence_avg
            200,   // connectome_connections
            0.6,   // resilience
            30,    // knowledge_topics
            100,   // cycle
        );
        assert!(score > 0.0 && score <= 100.0);
        assert!(turing.history.len() == 1);
    }

    #[test]
    fn test_turing_milestone() {
        assert_eq!(TuringMilestone::from_score(10.0), TuringMilestone::Embryonic);
        assert_eq!(TuringMilestone::from_score(30.0), TuringMilestone::Child);
        assert_eq!(TuringMilestone::from_score(50.0), TuringMilestone::Adolescent);
        assert_eq!(TuringMilestone::from_score(70.0), TuringMilestone::Adult);
        assert_eq!(TuringMilestone::from_score(90.0), TuringMilestone::Mature);
    }
}
