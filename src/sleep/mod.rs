// =============================================================================
// sleep/ — Minimal stub for the lite edition
// =============================================================================
//
// Purpose: Only the SleepRecord structure is defined here, used by
//          db/identity.rs for persisting sleep history.
//          The full sleep system (SleepSystem, SleepDrive, phases)
//          is not ported in the lite edition.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Record of a completed sleep period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepRecord {
    /// UTC timestamp when sleep started
    pub started_at: DateTime<Utc>,
    /// UTC timestamp when sleep ended
    pub ended_at: DateTime<Utc>,
    /// Total duration in cognitive cycles
    pub duration_cycles: u64,
    /// Number of completed sleep cycles (NREM + REM)
    pub sleep_cycles_completed: u8,
    /// Overall sleep quality score [0.0, 1.0]
    pub quality: f64,
    /// Number of memories consolidated during sleep
    pub memories_consolidated: u64,
    /// Number of new knowledge connections created
    pub connections_created: u64,
    /// Number of dreams experienced
    pub dreams_count: u64,
    /// Whether sleep was interrupted prematurely
    pub interrupted: bool,
    /// Reason for interruption, if any
    pub interruption_reason: Option<String>,
}
