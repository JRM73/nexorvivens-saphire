// =============================================================================
// logging/mod.rs — Saphire's centralized logging system
//
// Purpose: Provides a structured logger that buffers logs in memory,
// flushes them in batches to the logs database, and broadcasts them in
// real-time via a broadcast channel to the WebSocket dashboard.
// =============================================================================

pub mod db;
pub mod trace;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Severity level of a log entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

impl LogLevel {
    pub fn as_str(&self) -> &str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRITICAL",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Functional category of a log entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogCategory {
    Boot,
    Cycle,
    Pipeline,
    Thought,
    Conversation,
    Nlp,
    Brain,
    Consensus,
    Emotion,
    Consciousness,
    Regulation,
    Chemistry,
    Llm,
    Memory,
    Knowledge,
    Profile,
    Tuning,
    Plugin,
    WebSocket,
    Api,
    System,
    Weather,
    Heart,
    Body,
    Ethics,
    Vital,
    Intuition,
    Premonition,
    Senses,
    Shutdown,
    FactoryReset,
    Algorithm,
    Dream,
    Desire,
    Learning,
    NnLearning,
    Attention,
    Healing,
    Psyche,
    Maslow,
    Toltec,
    Shadow,
    EmotionalIQ,
    Flow,
    Will,
    Sleep,
    Subconscious,
    Metacognition,
    TheoryOfMind,
    InnerMonologue,
    Dissonance,
    ProspectiveMemory,
    NarrativeIdentity,
    Analogical,
    CognitiveLoad,
    MentalImagery,
    Sentiment,
    ChemicalHealth,
}

impl LogCategory {
    pub fn as_str(&self) -> &str {
        match self {
            LogCategory::Boot => "boot",
            LogCategory::Cycle => "cycle",
            LogCategory::Pipeline => "pipeline",
            LogCategory::Thought => "thought",
            LogCategory::Conversation => "conversation",
            LogCategory::Nlp => "nlp",
            LogCategory::Brain => "brain",
            LogCategory::Consensus => "consensus",
            LogCategory::Emotion => "emotion",
            LogCategory::Consciousness => "consciousness",
            LogCategory::Regulation => "regulation",
            LogCategory::Chemistry => "chemistry",
            LogCategory::Llm => "llm",
            LogCategory::Memory => "memory",
            LogCategory::Knowledge => "knowledge",
            LogCategory::Profile => "profile",
            LogCategory::Tuning => "tuning",
            LogCategory::Plugin => "plugin",
            LogCategory::WebSocket => "websocket",
            LogCategory::Api => "api",
            LogCategory::System => "system",
            LogCategory::Weather => "weather",
            LogCategory::Heart => "heart",
            LogCategory::Body => "body",
            LogCategory::Ethics => "ethics",
            LogCategory::Vital => "vital",
            LogCategory::Intuition => "intuition",
            LogCategory::Premonition => "premonition",
            LogCategory::Senses => "senses",
            LogCategory::Shutdown => "shutdown",
            LogCategory::FactoryReset => "factory_reset",
            LogCategory::Algorithm => "algorithm",
            LogCategory::Dream => "dream",
            LogCategory::Desire => "desire",
            LogCategory::Learning => "learning",
            LogCategory::NnLearning => "nn_learning",
            LogCategory::Attention => "attention",
            LogCategory::Healing => "healing",
            LogCategory::Psyche => "psyche",
            LogCategory::Maslow => "maslow",
            LogCategory::Toltec => "toltec",
            LogCategory::Shadow => "shadow",
            LogCategory::EmotionalIQ => "emotional_iq",
            LogCategory::Flow => "flow",
            LogCategory::Will => "will",
            LogCategory::Sleep => "sleep",
            LogCategory::Subconscious => "subconscious",
            LogCategory::Metacognition => "metacognition",
            LogCategory::TheoryOfMind => "theory_of_mind",
            LogCategory::InnerMonologue => "inner_monologue",
            LogCategory::Dissonance => "dissonance",
            LogCategory::ProspectiveMemory => "prospective_memory",
            LogCategory::NarrativeIdentity => "narrative_identity",
            LogCategory::Analogical => "analogical",
            LogCategory::CognitiveLoad => "cognitive_load",
            LogCategory::MentalImagery => "mental_imagery",
            LogCategory::Sentiment => "sentiment",
            LogCategory::ChemicalHealth => "chemical_health",
        }
    }
}

impl std::fmt::Display for LogCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// An individual log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub category: LogCategory,
    pub message: String,
    pub details: serde_json::Value,
    pub cycle: u64,
    pub session_id: i64,
}

/// Saphire's centralized logger.
///
/// Buffers logs (batches of 20) then flushes them to the LogsDb.
/// Also broadcasts each log to the broadcast channel for the WebSocket dashboard.
pub struct SaphireLogger {
    buffer: Vec<LogEntry>,
    buffer_size: usize,
    logs_db: Option<Arc<db::LogsDb>>,
    dashboard_tx: Option<Arc<broadcast::Sender<String>>>,
    session_id: i64,
}

impl SaphireLogger {
    /// Creates a new logger.
    pub fn new(
        logs_db: Option<Arc<db::LogsDb>>,
        dashboard_tx: Option<Arc<broadcast::Sender<String>>>,
    ) -> Self {
        Self {
            buffer: Vec::with_capacity(20),
            buffer_size: 20,
            logs_db,
            dashboard_tx,
            session_id: 0,
        }
    }

    /// Sets the session identifier for all subsequent logs.
    pub fn set_session_id(&mut self, id: i64) {
        self.session_id = id;
    }

    /// Records a log entry. Buffers it and broadcasts to the dashboard.
    pub fn log(
        &mut self,
        level: LogLevel,
        category: LogCategory,
        message: impl Into<String>,
        details: serde_json::Value,
        cycle: u64,
    ) {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level,
            category,
            message: message.into(),
            details,
            cycle,
            session_id: self.session_id,
        };

        // Real-time broadcast to the dashboard
        if let Some(ref tx) = self.dashboard_tx {
            let log_msg = serde_json::json!({
                "type": "dashboard_log",
                "log": {
                    "timestamp": entry.timestamp.to_rfc3339(),
                    "level": entry.level.as_str(),
                    "category": entry.category.as_str(),
                    "message": entry.message,
                    "cycle": entry.cycle,
                }
            });
            let _ = tx.send(log_msg.to_string());
        }

        // Fallback tracing
        match entry.level {
            LogLevel::Debug => tracing::debug!("[{}] {}", entry.category, entry.message),
            LogLevel::Info => tracing::info!("[{}] {}", entry.category, entry.message),
            LogLevel::Warn => tracing::warn!("[{}] {}", entry.category, entry.message),
            LogLevel::Error => tracing::error!("[{}] {}", entry.category, entry.message),
            LogLevel::Critical => tracing::error!("[CRITICAL][{}] {}", entry.category, entry.message),
        }

        self.buffer.push(entry);

        // Flush if the buffer is full
        if self.buffer.len() >= self.buffer_size {
            self.schedule_flush();
        }
    }

    /// Shortcut helpers
    pub fn info(&mut self, cat: LogCategory, msg: impl Into<String>, cycle: u64) {
        self.log(LogLevel::Info, cat, msg, serde_json::json!({}), cycle);
    }

    pub fn warn(&mut self, cat: LogCategory, msg: impl Into<String>, cycle: u64) {
        self.log(LogLevel::Warn, cat, msg, serde_json::json!({}), cycle);
    }

    pub fn error(&mut self, cat: LogCategory, msg: impl Into<String>, cycle: u64) {
        self.log(LogLevel::Error, cat, msg, serde_json::json!({}), cycle);
    }

    pub fn debug(&mut self, cat: LogCategory, msg: impl Into<String>, cycle: u64) {
        self.log(LogLevel::Debug, cat, msg, serde_json::json!({}), cycle);
    }

    pub fn critical(&mut self, cat: LogCategory, msg: impl Into<String>, cycle: u64) {
        self.log(LogLevel::Critical, cat, msg, serde_json::json!({}), cycle);
    }

    /// Schedules an asynchronous flush of the buffer to the DB.
    fn schedule_flush(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        let entries = std::mem::take(&mut self.buffer);
        self.buffer = Vec::with_capacity(self.buffer_size);

        if let Some(ref logs_db) = self.logs_db {
            let db = logs_db.clone();
            tokio::spawn(async move {
                if let Err(e) = db.batch_insert_logs(&entries).await {
                    tracing::error!("Flush logs vers DB echoue: {}", e);
                }
            });
        }
    }

    /// Forces an immediate flush (called at shutdown).
    pub fn flush(&mut self) {
        self.schedule_flush();
    }
}
