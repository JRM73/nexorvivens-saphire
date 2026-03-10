// plugins/ — Stub for the lite edition

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BrainEvent {
    ThoughtComplete { cycle: u64 },
    ThoughtEmitted { content: String, thought_type: String },
    EmotionShift { from: String, to: String },
    MemoryConsolidated { count: usize },
    SleepStateChange { sleeping: bool },
    ChemistryUpdate,
    StimulusAnalyzed { text: String, danger: f64, reward: f64, emotion: String },
    DecisionMade { decision: String, score: f64, satisfaction: f64 },
    BootCompleted { is_genesis: bool },
    ShutdownStarted,
}

pub struct PluginManager;

impl PluginManager {
    pub fn new() -> Self { Self }
    pub fn notify(&self, _event: &BrainEvent) {}
    pub fn notify_async(&self, _event: BrainEvent) {}
    pub fn broadcast(&self, _event: &BrainEvent) {}
}
