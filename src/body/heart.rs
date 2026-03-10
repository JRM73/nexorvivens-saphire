// =============================================================================
// heart.rs — Coeur virtuel de Saphire
// =============================================================================
//
// Role : Simule un coeur battant avec frequence cardiaque (BPM), variabilite
//        (HRV) et compteur de battements. Le rythme cardiaque est module par
//        la neurochimie : l'adrenaline et le cortisol accelerent le coeur,
//        la serotonine et les endorphines le ralentissent.
//
// Place dans l'architecture :
//   Le coeur est le noyau du module body. Son etat influence l'interoception
//   (conscience corporelle) et enrichit le contexte cognitif de Saphire.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::neurochemistry::NeuroChemicalState;

/// Etat du coeur a un instant donne, diffuse au WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartStatus {
    /// Frequence cardiaque en battements par minute
    pub bpm: f64,
    /// Nombre total de battements depuis la naissance
    pub beat_count: u64,
    /// Variabilite de la frequence cardiaque [0, 1] (HRV = Heart Rate Variability)
    pub hrv: f64,
    /// Force du battement [0, 1] (haute quand le coeur est sain et actif)
    pub strength: f64,
    /// Vrai si le coeur bat vite (tachycardie > 100 BPM)
    pub is_racing: bool,
    /// Vrai si le coeur est calme (bradycardie < 60 BPM)
    pub is_calm: bool,
}

/// Coeur virtuel avec rythme module par la neurochimie.
pub struct Heart {
    /// BPM de repos (configure dans saphire.toml)
    resting_bpm: f64,
    /// BPM actuel
    current_bpm: f64,
    /// Compteur total de battements
    beat_count: u64,
    /// Variabilite cardiaque [0, 1]
    hrv: f64,
    /// Force du battement [0, 1]
    strength: f64,
    /// Instant du dernier battement (en cycles fractionnaires)
    fractional_beats: f64,
}

impl Heart {
    /// Cree un nouveau coeur avec un BPM de repos donne.
    pub fn new(resting_bpm: f64) -> Self {
        Self {
            resting_bpm,
            current_bpm: resting_bpm,
            beat_count: 0,
            hrv: 0.7,
            strength: 0.6,
            fractional_beats: 0.0,
        }
    }

    /// Met a jour le coeur en fonction de la neurochimie.
    ///
    /// Le BPM est calcule comme :
    ///   bpm = resting + (adrenaline * 40) + (cortisol * 20) - (serotonin * 15) - (endorphin * 10)
    /// La HRV augmente avec la serotonine (coherence cardiaque) et diminue avec le stress.
    /// La force est elevee quand l'energie chimique est bonne (dopamine + endorphine).
    ///
    /// Parametre : `dt_seconds` — duree ecoulee depuis le dernier update (typiquement ~15s).
    pub fn update(&mut self, chemistry: &NeuroChemicalState, dt_seconds: f64) {
        // Moduler le BPM par la neurochimie
        let excitation = chemistry.adrenaline * 40.0 + chemistry.cortisol * 20.0;
        let calming = chemistry.serotonin * 15.0 + chemistry.endorphin * 10.0;
        let target_bpm = (self.resting_bpm + excitation - calming).clamp(45.0, 160.0);

        // Convergence lente vers le BPM cible (inertie cardiaque)
        self.current_bpm += (target_bpm - self.current_bpm) * 0.15;
        self.current_bpm = self.current_bpm.clamp(45.0, 160.0);

        // Compter les battements produits pendant dt_seconds
        let beats_this_update = self.current_bpm / 60.0 * dt_seconds;
        self.fractional_beats += beats_this_update;
        let new_beats = self.fractional_beats as u64;
        self.beat_count += new_beats;
        self.fractional_beats -= new_beats as f64;

        // HRV : haute avec la serotonine (coherence), basse sous stress
        let target_hrv = (0.5 + chemistry.serotonin * 0.3 - chemistry.cortisol * 0.25
            + chemistry.endorphin * 0.15).clamp(0.1, 0.95);
        self.hrv += (target_hrv - self.hrv) * 0.1;

        // Force : proportionnelle a l'energie chimique globale
        let target_strength = (0.4 + chemistry.dopamine * 0.2 + chemistry.endorphin * 0.2
            + chemistry.serotonin * 0.1 - chemistry.cortisol * 0.15).clamp(0.2, 1.0);
        self.strength += (target_strength - self.strength) * 0.1;
    }

    /// Retourne l'etat actuel du coeur pour le WebSocket.
    pub fn status(&self) -> HeartStatus {
        HeartStatus {
            bpm: (self.current_bpm * 10.0).round() / 10.0,
            beat_count: self.beat_count,
            hrv: (self.hrv * 100.0).round() / 100.0,
            strength: (self.strength * 100.0).round() / 100.0,
            is_racing: self.current_bpm > 100.0,
            is_calm: self.current_bpm < 60.0,
        }
    }

    /// Retourne le BPM actuel.
    pub fn bpm(&self) -> f64 {
        self.current_bpm
    }

    /// Retourne la force du battement cardiaque [0, 1].
    pub fn strength(&self) -> f64 {
        self.strength
    }

    /// Restaure le compteur de battements (depuis la DB).
    pub fn restore_beat_count(&mut self, count: u64) {
        self.beat_count = count;
    }

    /// Retourne le nombre total de battements.
    pub fn beat_count(&self) -> u64 {
        self.beat_count
    }

    /// Ralentit progressivement le coeur vers un BPM cible (sommeil).
    pub fn calm_down(&mut self, target: f64) {
        self.current_bpm += (target - self.current_bpm) * 0.1;
        self.current_bpm = self.current_bpm.clamp(45.0, 160.0);
    }

    /// Variation sinusoidale du BPM (simulation REM).
    pub fn vary_bpm(&mut self, amplitude: f64) {
        let v = (self.current_bpm.sin() * amplitude).clamp(-amplitude, amplitude);
        self.current_bpm = (self.current_bpm + v).clamp(45.0, 160.0);
    }

    /// Accelere progressivement le coeur vers un BPM cible (reveil).
    pub fn wake_up_bpm(&mut self, target: f64) {
        self.current_bpm += (target - self.current_bpm) * 0.2;
        self.current_bpm = self.current_bpm.clamp(45.0, 160.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neurochemistry::NeuroChemicalState;

    #[test]
    fn test_heart_initial_bpm() {
        let heart = Heart::new(65.0);
        assert!((heart.bpm() - 65.0).abs() < 1.0);
    }

    #[test]
    fn test_bpm_increases_with_adrenaline() {
        let mut heart = Heart::new(65.0);
        let mut chem = NeuroChemicalState::default();
        chem.adrenaline = 0.9;
        chem.cortisol = 0.8;
        heart.update(&chem, 1.0);
        assert!(heart.bpm() > 65.0, "Adrenaline should increase BPM, got {}", heart.bpm());
    }

    #[test]
    fn test_bpm_stays_in_range() {
        let mut heart = Heart::new(65.0);
        let mut chem = NeuroChemicalState::default();
        chem.adrenaline = 1.0;
        chem.cortisol = 1.0;
        chem.noradrenaline = 1.0;
        for _ in 0..1000 {
            heart.update(&chem, 1.0);
        }
        assert!(heart.bpm() <= 200.0, "BPM should have a reasonable max");
        assert!(heart.bpm() >= 30.0, "BPM should have a reasonable min");
    }

    #[test]
    fn test_beat_count_increases() {
        let mut heart = Heart::new(65.0);
        let chem = NeuroChemicalState::default();
        let initial = heart.beat_count();
        heart.update(&chem, 60.0); // 60 seconds ~ 65 beats
        assert!(heart.beat_count() > initial, "Beat count should increase over time");
    }

    #[test]
    fn test_heart_strength_stays_in_range() {
        let mut heart = Heart::new(65.0);
        let chem = NeuroChemicalState::default();
        for _ in 0..100 {
            heart.update(&chem, 10.0);
        }
        let status = heart.status();
        assert!(status.strength >= 0.0 && status.strength <= 1.0);
    }
}
