// =============================================================================
// profiling/mod.rs — Dynamic psychological profiling based on the Big Five
//                    OCEAN model (Openness / Conscientiousness / Extraversion /
//                    Agreeableness / Neuroticism)
//
// Role: Entry point for the profiling module. Exposes sub-modules and
//       configuration structures for Saphire's bidirectional psychological
//       profiling system:
//         - Self-profiling (self_profiler): Saphire observes her own cognitive
//           cycles to build her OCEAN profile
//         - Human profiling (human_profiler): Saphire analyzes the interlocutor's
//           messages to estimate their OCEAN profile
//         - Adaptation (adaptation): generates style instructions based on
//           the human's profile to adapt responses
//         - Narrative (narrative): generates a textual description of the OCEAN profile
//
// Dependencies:
//   - serde: serialization/deserialization of configuration
//   - Sub-modules: ocean, self_profiler, human_profiler, adaptation, narrative
//
// Place in architecture:
//   Profiling is a cross-cutting component of Saphire's cognitive system.
//   It is fed by NLP results (for human profiling) and by cognitive cycle
//   observations (for self-profiling). The resulting profiles influence
//   response generation via the adaptation module.
// =============================================================================

pub mod ocean;
pub mod self_profiler;
pub mod human_profiler;
pub mod adaptation;
pub mod narrative;

use serde::{Deserialize, Serialize};

pub use ocean::OceanProfile;
pub use self_profiler::{SelfProfiler, BehaviorObservation};
pub use human_profiler::{HumanProfiler, HumanProfile, CommunicationStyle};

/// Configuration of the psychological profiling system.
///
/// Controls the operating parameters of profiling for Saphire's self-profile
/// and the profiles of human interlocutors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingConfig {
    /// Enables or disables the entire profiling system
    pub enabled: bool,
    /// Enables self-profiling (Saphire observes her own behaviors)
    pub self_profiling: bool,
    /// Enables profiling of human interlocutors
    pub human_profiling: bool,
    /// Number of cognitive cycles between each OCEAN profile recalculation.
    /// The lower the value, the more reactive the profile is to changes.
    pub recompute_interval_cycles: u64,
    /// Maximum size of the behavioral observation buffer.
    /// When the buffer is full, a recalculation is automatically triggered.
    pub observation_buffer_size: usize,
    /// Blend rate between old and new profile during recalculation.
    /// 0.3 means 30% new + 70% old, which smooths fluctuations.
    pub profile_blend_rate: f64,
    /// Maximum number of historical profile snapshots to keep.
    /// Allows tracking profile evolution over time.
    pub history_snapshots: usize,
}

impl Default for ProfilingConfig {
    /// Default profiling configuration.
    ///
    /// Returns: a configuration with profiling active, recalculation every
    ///          50 cycles, a 100-observation buffer, 30% blend rate,
    ///          and 30 historical snapshots.
    fn default() -> Self {
        Self {
            enabled: true,
            self_profiling: true,
            human_profiling: true,
            recompute_interval_cycles: 50,
            observation_buffer_size: 100,
            profile_blend_rate: 0.3,
            history_snapshots: 30,
        }
    }
}
