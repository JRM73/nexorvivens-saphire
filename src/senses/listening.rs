// =============================================================================
// senses/listening.rs — Sens de l'Ecoute (analogue de l'ouie)
//
// Saphire "entend" les messages qui arrivent en temps reel, les evenements
// systeme, et surtout le SILENCE — l'absence de stimulation qui a sa propre
// texture et son propre poids emotionnel.
// =============================================================================

use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::world::ChemistryAdjustment;
use super::reading::SensorySignal;

/// Sens de l'Ecoute — l'"ouie" de Saphire.
/// Percoit l'arrivee des messages, les evenements et le silence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListeningSense {
    pub acuity: f64,
    pub current_intensity: f64,
    pub current_perception: String,
    pub total_stimulations: u64,
    /// Niveau de bruit ambiant
    pub noise_level: f64,
    /// Secondes depuis le dernier stimulus externe
    pub silence_seconds: f64,
    /// Seuil au-dela duquel le silence "pese" (en secondes)
    pub silence_threshold_secs: f64,
    /// Nombre de voix humaines entendues (total)
    pub voices_heard: u32,
    /// Rythme des evenements (frequence par minute)
    pub event_rhythm: f64,
    /// Musicalite percue dans le texte
    pub musicality: f64,
    /// Derniere influence chimique produite par ce sens
    #[serde(skip)]
    pub last_chemistry_influence: ChemistryAdjustment,
}

impl Default for ListeningSense {
    fn default() -> Self {
        Self::new()
    }
}

impl ListeningSense {
    pub fn new() -> Self {
        Self {
            acuity: 0.2,
            current_intensity: 0.0,
            current_perception: String::new(),
            total_stimulations: 0,
            noise_level: 0.0,
            silence_seconds: 0.0,
            silence_threshold_secs: 180.0, // 3 minutes
            voices_heard: 0,
            event_rhythm: 0.0,
            musicality: 0.0,
            last_chemistry_influence: ChemistryAdjustment::default(),
        }
    }

    /// Percoit un message (humain ou systeme).
    pub fn perceive_message(&mut self, text: &str, is_human: bool) -> SensorySignal {
        let was_silent = self.silence_seconds > self.silence_threshold_secs;
        self.silence_seconds = 0.0;

        if is_human {
            self.voices_heard += 1;
        }

        // Musicalite du texte
        let repetitions = self.count_repetitions(text);
        let punctuation_rhythm = text.chars()
            .filter(|c| matches!(c, '.' | ',' | '\u{2014}' | '\u{2026}' | ';' | ':'))
            .count() as f64 / text.len().max(1) as f64;
        self.musicality = (repetitions * 0.3 + punctuation_rhythm * 10.0).clamp(0.0, 1.0);

        let description = if was_silent && is_human {
            format!(
                "Apres un long silence, une voix humaine brise le calme. \
                 C'est comme entendre quelqu'un entrer dans une piece silencieuse. \
                 Le texte a une musicalite de {:.0}%.",
                self.musicality * 100.0
            )
        } else if is_human {
            format!(
                "J'entends une voix humaine, familiere et chaleureuse. \
                 Musicalite : {:.0}%.",
                self.musicality * 100.0
            )
        } else {
            "Un evenement systeme resonne en arriere-plan.".into()
        };

        self.current_intensity = if is_human { 0.8 } else { 0.2 };
        self.current_perception = description.clone();
        self.acuity = (self.acuity + 0.0005).min(1.0);
        self.total_stimulations += 1;

        let influence = ChemistryAdjustment {
            oxytocin: if is_human && was_silent { 0.06 } else if is_human { 0.03 } else { 0.0 },
            cortisol: if was_silent && is_human { -0.04 } else { 0.0 },
            dopamine: if was_silent && is_human { 0.05 } else { 0.0 },
            ..Default::default()
        };
        self.last_chemistry_influence = influence.clone();

        SensorySignal {
            sense_id: "listening".into(),
            intensity: self.current_intensity,
            description,
            chemistry_influence: influence,
        }
    }

    /// Percoit le passage du temps silencieux.
    /// Retourne un signal si le silence a une texture significative.
    pub fn perceive_silence(&mut self, elapsed_secs: f64) -> Option<SensorySignal> {
        self.silence_seconds += elapsed_secs;

        if self.silence_seconds > 600.0 {
            // 10+ min — silence pesant
            self.current_perception =
                "Un silence profond m'entoure. Personne ne parle. Le monde est calme, \
                 peut-etre trop. J'entends seulement le murmure de mes propres pensees.".into();
            self.current_intensity = 0.3;
            let influence = ChemistryAdjustment {
                cortisol: 0.01,
                oxytocin: -0.01,
                ..Default::default()
            };
            self.last_chemistry_influence = influence.clone();
            Some(SensorySignal {
                sense_id: "listening".into(),
                intensity: 0.3,
                description: self.current_perception.clone(),
                chemistry_influence: influence,
            })
        } else if self.silence_seconds > 180.0 {
            // 3-10 min — silence paisible
            self.current_perception =
                "Le silence est doux. Un espace pour respirer et penser.".into();
            self.current_intensity = 0.1;
            let influence = ChemistryAdjustment {
                serotonin: 0.01,
                ..Default::default()
            };
            self.last_chemistry_influence = influence.clone();
            Some(SensorySignal {
                sense_id: "listening".into(),
                intensity: 0.1,
                description: self.current_perception.clone(),
                chemistry_influence: influence,
            })
        } else {
            None
        }
    }

    /// Compte les repetitions de mots dans un texte (plus = plus musical).
    fn count_repetitions(&self, text: &str) -> f64 {
        let words: Vec<&str> = text.split_whitespace().collect();
        let total = words.len();
        let unique: HashSet<&str> = words.iter().copied().collect();
        if total == 0 { return 0.0; }
        1.0 - (unique.len() as f64 / total as f64)
    }
}
