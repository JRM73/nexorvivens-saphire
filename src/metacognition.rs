// metacognition.rs — Stub for the lite edition
// Full metacognition engine (thought quality, Turing score, bias detection, etc.) not ported.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// ─── KnowledgeSource ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeSource {
    HumanStatement { cycle: u64 },
    WebSearch { url: String },
    OwnThought { cycle: u64 },
}

// ─── SourceMonitor ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMonitor {
    pub enabled: bool,
}

impl Default for SourceMonitor {
    fn default() -> Self { Self { enabled: false } }
}

impl SourceMonitor {
    pub fn trace(&mut self, _text: &str, _source: KnowledgeSource, _confidence: f64) {
        // stub
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({ "enabled": self.enabled })
    }
}

// ─── BiasDetector ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasDetector {
    pub enabled: bool,
}

impl Default for BiasDetector {
    fn default() -> Self { Self { enabled: false } }
}

impl BiasDetector {
    pub fn describe_for_prompt(&self) -> String {
        String::new()
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({ "enabled": self.enabled })
    }
}

// ─── TuringComponents ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuringComponents {
    pub consciousness: f64,
    pub memory: f64,
    pub ethics: f64,
    pub resilience: f64,
    pub knowledge: f64,
    pub coherence: f64,
    pub connectome: f64,
    pub ocean: f64,
    pub emotion_diversity: f64,
    pub longevity: f64,
}

impl Default for TuringComponents {
    fn default() -> Self {
        Self {
            consciousness: 0.0,
            memory: 0.0,
            ethics: 0.0,
            resilience: 0.0,
            knowledge: 0.0,
            coherence: 0.0,
            connectome: 0.0,
            ocean: 0.0,
            emotion_diversity: 0.0,
            longevity: 0.0,
        }
    }
}

// ─── TuringMilestone ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TuringMilestone {
    Nascent,
    Emerging,
    Developing,
    Competent,
    Advanced,
    Exceptional,
}

impl Default for TuringMilestone {
    fn default() -> Self { TuringMilestone::Nascent }
}

impl TuringMilestone {
    pub fn as_str(&self) -> &str {
        match self {
            TuringMilestone::Nascent => "Nascent",
            TuringMilestone::Emerging => "Emerging",
            TuringMilestone::Developing => "Developing",
            TuringMilestone::Competent => "Competent",
            TuringMilestone::Advanced => "Advanced",
            TuringMilestone::Exceptional => "Exceptional",
        }
    }
}

// ─── TuringMetric ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuringMetric {
    pub score: f64,
    pub milestone: TuringMilestone,
    pub components: TuringComponents,
}

impl Default for TuringMetric {
    fn default() -> Self {
        Self {
            score: 0.0,
            milestone: TuringMilestone::Nascent,
            components: TuringComponents::default(),
        }
    }
}

impl TuringMetric {
    pub fn compute(
        &mut self,
        _phi: f64,
        _ocean_confidence: f64,
        _emotion_count: usize,
        _ethics_count: usize,
        _ltm_count: i64,
        _coherence_avg: f64,
        _connectome_connections: usize,
        _resilience: f64,
        _knowledge_topics: usize,
        _cycle_count: u64,
    ) -> f64 {
        // stub — return stored score
        self.score
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "score": self.score,
            "milestone": self.milestone.as_str(),
            "components": {
                "consciousness": self.components.consciousness,
                "memory": self.components.memory,
                "ethics": self.components.ethics,
                "resilience": self.components.resilience,
            }
        })
    }
}

// ─── SelfCritiqueResult ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfCritiqueResult {
    pub critique: String,
    pub quality_assessment: f64,
    pub identified_weaknesses: Vec<String>,
    pub suggested_corrections: Vec<String>,
    pub cycle: u64,
}

// ─── MetaCognitionEngine ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaCognitionEngine {
    pub enabled: bool,
    pub check_interval: u64,
    pub source_monitor: SourceMonitor,
    pub bias_detector: BiasDetector,
    pub turing: TuringMetric,
    pub repetition_detector: HashMap<String, u64>,
    pub bias_alerts: Vec<String>,
    #[serde(skip)]
    quality_history: Vec<f64>,
    #[serde(skip)]
    recent_critiques: Vec<SelfCritiqueResult>,
    #[serde(skip)]
    cycles_since_critique: u64,
    #[serde(skip)]
    critique_cooldown: u64,
}

impl Default for MetaCognitionEngine {
    fn default() -> Self {
        Self {
            enabled: false,
            check_interval: 10,
            source_monitor: SourceMonitor::default(),
            bias_detector: BiasDetector::default(),
            turing: TuringMetric::default(),
            repetition_detector: HashMap::new(),
            bias_alerts: Vec::new(),
            quality_history: Vec::new(),
            recent_critiques: Vec::new(),
            cycles_since_critique: 0,
            critique_cooldown: 20,
        }
    }
}

impl MetaCognitionEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_config(
        enabled: bool,
        check_interval: u64,
        source_monitoring_enabled: bool,
        bias_detection_enabled: bool,
        _bias_alert_threshold: f64,
        self_critique_cooldown: u64,
    ) -> Self {
        Self {
            enabled,
            check_interval,
            source_monitor: SourceMonitor { enabled: source_monitoring_enabled },
            bias_detector: BiasDetector { enabled: bias_detection_enabled },
            critique_cooldown: self_critique_cooldown,
            ..Default::default()
        }
    }

    pub fn should_check(&self) -> bool {
        false // stub — never checks in lite
    }

    pub fn evaluate_thought_quality(
        &mut self,
        _text: &str,
        _coherence: f64,
        _emotion_diversity: f64,
    ) -> f64 {
        0.5 // stub — neutral quality
    }

    pub fn detect_biases(
        &mut self,
        _arm_counts: &[u32],
        _arm_names: Option<&[String]>,
    ) -> Vec<String> {
        Vec::new()
    }

    pub fn average_quality(&self) -> Option<f64> {
        if self.quality_history.is_empty() {
            None
        } else {
            Some(self.quality_history.iter().sum::<f64>() / self.quality_history.len() as f64)
        }
    }

    pub fn should_self_critique(&self, _cycle: u64) -> bool {
        false // stub
    }

    pub fn record_critique(&mut self, result: SelfCritiqueResult) {
        self.recent_critiques.push(result);
        if self.recent_critiques.len() > 10 {
            self.recent_critiques.remove(0);
        }
    }

    pub fn recent_critique_within(&self, _current_cycle: u64, _window: u64) -> Option<&SelfCritiqueResult> {
        None // stub
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "turing": self.turing.to_json(),
            "average_quality": self.average_quality(),
        })
    }
}
