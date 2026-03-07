// =============================================================================
// sleep/ — Stub minimal pour la version lite
//
// Seule la structure SleepRecord est definie ici, utilisee par
// db/identity.rs pour la persistance de l'historique de sommeil.
// Le systeme de sommeil complet (SleepSystem, SleepDrive, phases)
// n'est pas porte dans la version lite.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Enregistrement d'une periode de sommeil complete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepRecord {
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_cycles: u64,
    pub sleep_cycles_completed: u8,
    pub quality: f64,
    pub memories_consolidated: u64,
    pub connections_created: u64,
    pub dreams_count: u64,
    pub interrupted: bool,
    pub interruption_reason: Option<String>,
}
