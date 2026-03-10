// =============================================================================
// sleep/phases.rs — Descriptions et helpers pour les phases de sommeil
//
// Role : Fournit des descriptions textuelles et des constantes utilitaires
// pour chaque phase de sommeil. La logique de transition est dans sleep_tick.rs.
// =============================================================================

use crate::orchestrators::dreams::SleepPhase;

/// Description humaine d'une phase de sommeil pour les logs et l'IU.
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

/// Emoji representant une phase (pour les logs terminal).
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

/// Niveau d'activation du subconscient selon la phase.
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
