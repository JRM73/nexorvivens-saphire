// =============================================================================
// logging/mod.rs — Saphire's centralized logging system
// =============================================================================
//
// Purpose: Provides a structured logger that buffers log entries in memory,
//          flushes them in batches to the logs database, and broadcasts each
//          entry in real time via a broadcast channel to the WebSocket dashboard.
//
// Dependencies:
//   - chrono: timestamps for each log entry
//   - serde: serialization/deserialization of log entries
//   - tokio::sync::broadcast: real-time log broadcasting to the dashboard
//   - std::sync::Arc: shared ownership of the database and broadcast handles
//
// Architectural placement:
//   Entry point of the logging module. SaphireLogger is used throughout the
//   system to emit structured, categorized logs. Sub-modules `db` and `trace`
//   handle PostgreSQL persistence and per-cycle cognitive trace recording.
// =============================================================================

pub mod db;
pub mod trace;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Log severity level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

impl LogLevel {
    /// Returns the severity level as a static string label.
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
/// Each variant corresponds to a subsystem or module in Saphire's architecture.
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
    /// Returns the category as a lowercase string label for database storage and display.
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

/// A single log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// UTC timestamp of when the log entry was created
    pub timestamp: DateTime<Utc>,
    /// Severity level (Debug, Info, Warn, Error, Critical)
    pub level: LogLevel,
    /// Functional category identifying the source subsystem
    pub category: LogCategory,
    /// Human-readable log message
    pub message: String,
    /// Additional structured data as a JSON value (empty object if none)
    pub details: serde_json::Value,
    /// Cognitive cycle number during which this log was emitted
    pub cycle: u64,
    /// Session identifier for grouping logs by runtime session
    pub session_id: i64,
}

/// Saphire's centralized logger.
///
/// Buffers log entries (batches of 20) then flushes them to the LogsDb.
/// Also broadcasts each log entry to the broadcast channel for the
/// real-time WebSocket dashboard.
pub struct SaphireLogger {
    /// In-memory buffer of pending log entries awaiting flush
    buffer: Vec<LogEntry>,
    /// Maximum buffer size before triggering an automatic flush (default: 20)
    buffer_size: usize,
    /// Optional handle to the PostgreSQL logs database for batch persistence
    logs_db: Option<Arc<db::LogsDb>>,
    /// Optional broadcast sender for real-time dashboard streaming
    dashboard_tx: Option<Arc<broadcast::Sender<String>>>,
    /// Current session identifier applied to all subsequent log entries
    session_id: i64,
}

impl SaphireLogger {
    /// Creates a new logger instance.
    ///
    /// Parameter `logs_db`: optional handle to the PostgreSQL logs database
    /// Parameter `dashboard_tx`: optional broadcast sender for real-time dashboard streaming
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

    /// Sets the session identifier for all subsequent log entries.
    pub fn set_session_id(&mut self, id: i64) {
        self.session_id = id;
    }

    /// Records a log entry. Buffers it for batch persistence and broadcasts it to the dashboard.
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

        // Broadcast to the dashboard in real time
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

        // Fallback to tracing crate for structured console output
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

    /// Shorthand helper: logs at Info level with no extra details.
    pub fn info(&mut self, cat: LogCategory, msg: impl Into<String>, cycle: u64) {
        self.log(LogLevel::Info, cat, msg, serde_json::json!({}), cycle);
    }

    /// Shorthand helper: logs at Warn level with no extra details.
    pub fn warn(&mut self, cat: LogCategory, msg: impl Into<String>, cycle: u64) {
        self.log(LogLevel::Warn, cat, msg, serde_json::json!({}), cycle);
    }

    /// Shorthand helper: logs at Error level with no extra details.
    pub fn error(&mut self, cat: LogCategory, msg: impl Into<String>, cycle: u64) {
        self.log(LogLevel::Error, cat, msg, serde_json::json!({}), cycle);
    }

    /// Shorthand helper: logs at Debug level with no extra details.
    pub fn debug(&mut self, cat: LogCategory, msg: impl Into<String>, cycle: u64) {
        self.log(LogLevel::Debug, cat, msg, serde_json::json!({}), cycle);
    }

    /// Shorthand helper: logs at Critical level with no extra details.
    pub fn critical(&mut self, cat: LogCategory, msg: impl Into<String>, cycle: u64) {
        self.log(LogLevel::Critical, cat, msg, serde_json::json!({}), cycle);
    }

    /// Schedules an asynchronous flush of the buffer to the database.
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
                    tracing::error!("Flush logs to DB failed: {}", e);
                }
            });
        }
    }

    /// Forces an immediate flush (called during shutdown).
    pub fn flush(&mut self) {
        self.schedule_flush();
    }
}
