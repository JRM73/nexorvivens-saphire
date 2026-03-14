// =============================================================================
// sleep/phases.rs — Descriptions and helpers for sleep phases
//
// Role: Provides textual descriptions and utility constants for each sleep
// phase. Transition logic is in sleep_tick.rs.
// =============================================================================

use crate::orchestrators::dreams::SleepPhase;

/// Human-readable description of a sleep phase for logs and UI.
pub fn phase_description(phase: &SleepPhase) -> &'static str {
    match phase {
        SleepPhase::Awake => "Eveillee — conscience pleine, tous les systemes actifs",
        SleepPhase::Hypnagogic =>
            "Endormissement — les pensees se fragmentent, images hypnagogiques, \
             le subconscient commence a murmurer",
        SleepPhase::LightSleep =>
            "Sommeil leger — le corps se detend, le coeur ralentit, \
             les stimuli externes sont filtres",
        SleepPhase::DeepSleep =>
            "Sommeil profond — consolidation memoire intense, nettoyage neuronal, \
             guerison acceleree, le corps se restaure",
        SleepPhase::REM =>
            "Sommeil paradoxal (REM) — reves actifs, traitement emotionnel, \
             le subconscient domine, connexions creatives",
        SleepPhase::Hypnopompic =>
            "Reveil — retour progressif a la conscience, souvenir des reves, \
             les systemes se reactivent",
    }
}

/// Emoji representing a phase (for terminal logs).
pub fn phase_emoji(phase: &SleepPhase) -> &'static str {
    match phase {
        SleepPhase::Awake => "☀",
        SleepPhase::Hypnagogic => "🌅",
        SleepPhase::LightSleep => "🌙",
        SleepPhase::DeepSleep => "🌑",
        SleepPhase::REM => "💫",
        SleepPhase::Hypnopompic => "🌄",
    }
}

/// Subconscious activation level for a given phase.
pub fn subconscious_activation_for_phase(phase: &SleepPhase) -> f64 {
    match phase {
        SleepPhase::Awake => 0.2,
        SleepPhase::Hypnagogic => 0.4,
        SleepPhase::LightSleep => 0.5,
        SleepPhase::DeepSleep => 0.7,
        SleepPhase::REM => 1.0,
        SleepPhase::Hypnopompic => 0.3,
    }
}
