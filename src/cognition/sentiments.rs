// =============================================================================
// sentiments.rs — Systeme de sentiments (etats affectifs durables)
// =============================================================================
//
// Role : Les sentiments sont des etats affectifs durables qui emergent de
// patterns emotionnels repetitifs. Contrairement aux emotions (reactives,
// instantanees), les sentiments persistent sur des dizaines a des milliers
// de cycles et influencent en retour la coloration emotionnelle.
//
// Architecture :
//   Emotions → (accumulation) → Sentiments → (biais) → Emotions
//   Boucle bidirectionnelle : les sentiments amplifient ou attenuent les
//   emotions suivantes, et modifient legerement la chimie de fond.
//
// 3 durees de sentiments :
//   - Court terme (10-50 cycles) : humeurs passageres (irritation, amusement)
//   - Moyen terme (50-200 cycles) : etats installes (mefiance, attachement)
//   - Long terme (200-1000+ cycles) : traits affectifs profonds (amertume, confiance)
//
// Dependances :
//   - crate::world::weather::ChemistryAdjustment : influence chimique
//   - serde : serialisation
//
// Place dans l'architecture :
//   Module de premier niveau. Le SentimentEngine est tick-e a chaque cycle
//   cognitif, apres le calcul emotionnel et avant la mise a jour du mood.
//   Il est integre dans le pipeline via phase_sentiments (thinking.rs).
// =============================================================================

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use crate::world::weather::ChemistryAdjustment;

// =============================================================================
// Configuration
// =============================================================================

/// Configuration du systeme de sentiments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentConfig {
    /// Module actif ou non
    pub enabled: bool,
    /// Nombre maximum de sentiments actifs simultanes
    pub max_active: usize,
    /// Taille de la fenetre d'historique emotionnel (nombre d'emotions conservees)
    pub emotion_history_window: usize,
    /// Taux de decroissance par cycle (force -= decay_rate / multiplier)
    pub decay_rate: f64,
    /// Force de renforcement quand une emotion trigger est detectee
    pub reinforcement_strength: f64,
    /// Plafond d'influence chimique par sentiment (evite les emballements)
    pub chemistry_influence_cap: f64,
}

impl Default for SentimentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_active: 10,
            emotion_history_window: 200,
            decay_rate: 0.005,
            reinforcement_strength: 0.1,
            chemistry_influence_cap: 0.05,
        }
    }
}

// =============================================================================
// Duree de sentiment
// =============================================================================

/// Duree de vie d'un sentiment.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SentimentDuration {
    /// Court terme : humeurs passageres (10-50 cycles)
    ShortTerm,
    /// Moyen terme : etats installes (50-200 cycles)
    MediumTerm,
    /// Long terme : traits affectifs profonds (200-1000+ cycles)
    LongTerm,
}

impl SentimentDuration {
    /// Multiplicateur de duree pour le decay (plus long = plus lent)
    pub fn decay_multiplier(&self) -> f64 {
        match self {
            SentimentDuration::ShortTerm => 1.0,
            SentimentDuration::MediumTerm => 3.0,
            SentimentDuration::LongTerm => 10.0,
        }
    }

    /// Label textuel
    pub fn label(&self) -> &str {
        match self {
            SentimentDuration::ShortTerm => "court terme",
            SentimentDuration::MediumTerm => "moyen terme",
            SentimentDuration::LongTerm => "long terme",
        }
    }
}

// =============================================================================
// Profil de sentiment (definition du catalogue)
// =============================================================================

/// Definition d'un sentiment : ses conditions de formation et ses effets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentProfile {
    /// Nom du sentiment (ex: "Irritation", "Mefiance")
    pub name: String,
    /// Duree du sentiment
    pub duration_type: SentimentDuration,
    /// Emotions qui peuvent declencher/renforcer ce sentiment
    pub trigger_emotions: Vec<String>,
    /// Nombre d'occurrences dans la fenetre pour declencher la formation
    pub trigger_threshold: usize,
    /// Biais chimique applique quand le sentiment est actif
    pub chemistry_bias: ChemistryAdjustment,
    /// Emotions amplifiees par ce sentiment (nom → facteur d'amplification)
    pub emotion_amplification: Vec<(String, f64)>,
    /// Emotions attenuees par ce sentiment (nom → facteur d'attenuation)
    pub emotion_dampening: Vec<(String, f64)>,
    /// Duree minimale en cycles avant dissolution naturelle
    pub min_duration_cycles: u64,
    /// Duree maximale en cycles (dissolution forcee)
    pub max_duration_cycles: u64,
}

// =============================================================================
// Sentiment actif
// =============================================================================

/// Un sentiment actuellement actif dans l'esprit de Saphire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSentiment {
    /// Nom du sentiment (reference au profil)
    pub profile_name: String,
    /// Force actuelle [0.0, 1.0] — decroit avec le temps
    pub strength: f64,
    /// Cycle de formation
    pub formed_at_cycle: u64,
    /// Dernier cycle de renforcement
    pub last_reinforced: u64,
    /// Nombre de renforcements recus
    pub reinforcement_count: u32,
    /// Type de duree (copie du profil)
    pub duration_type: SentimentDuration,
    /// Contexte d'origine (emotion dominante lors de la formation)
    pub source_context: String,
}

impl ActiveSentiment {
    /// Description textuelle du sentiment actif.
    pub fn describe(&self) -> String {
        let intensity = if self.strength > 0.7 { "fort" }
            else if self.strength > 0.4 { "modere" }
            else { "faible" };
        format!("{} ({}, {})",
            self.profile_name, self.duration_type.label(), intensity)
    }
}

// =============================================================================
// Moteur de sentiments
// =============================================================================

/// Moteur de sentiments — gere la formation, le renforcement, le decay
/// et l'influence bidirectionnelle emotions ↔ sentiments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentEngine {
    /// Module actif ou non
    pub enabled: bool,
    /// Sentiments actuellement actifs
    pub active_sentiments: Vec<ActiveSentiment>,
    /// Historique des emotions recentes (fenetre glissante)
    emotion_history: VecDeque<String>,
    /// Catalogue de profils de sentiments
    catalog: Vec<SentimentProfile>,
    /// Configuration
    config: SentimentConfig,
    /// Compteur total de sentiments formes depuis le debut
    pub total_formed: u64,
    /// Compteur total de sentiments dissous
    pub total_dissolved: u64,
}

impl SentimentEngine {
    /// Cree un nouveau moteur de sentiments.
    pub fn new(config: &SentimentConfig) -> Self {
        Self {
            enabled: config.enabled,
            active_sentiments: Vec::new(),
            emotion_history: VecDeque::with_capacity(config.emotion_history_window),
            catalog: build_sentiment_catalog(),
            config: config.clone(),
            total_formed: 0,
            total_dissolved: 0,
        }
    }

    /// Tick principal : enregistre l'emotion, renforce les sentiments existants,
    /// applique le decay, verifie les formations et dissolutions.
    ///
    /// Appele a chaque cycle cognitif apres le calcul emotionnel.
    pub fn tick(&mut self, emotion: &str, cycle: u64) {
        if !self.enabled {
            return;
        }

        // 1. Enregistrer l'emotion dans l'historique
        self.emotion_history.push_back(emotion.to_string());
        if self.emotion_history.len() > self.config.emotion_history_window {
            self.emotion_history.pop_front();
        }

        // 2. Renforcer les sentiments existants dont les triggers matchent
        // Rendements decroissants : plus le sentiment est fort, moins le
        // renforcement a d'effet (comme les recepteurs neurochimiques).
        let reinforcement = self.config.reinforcement_strength;
        for sentiment in &mut self.active_sentiments {
            if let Some(profile) = self.catalog.iter().find(|p| p.name == sentiment.profile_name) {
                if profile.trigger_emotions.iter().any(|t| t == emotion) {
                    // Rendement decroissant : marge restante avant saturation
                    let margin = (1.0 - sentiment.strength).max(0.05);
                    let effective_reinforcement = reinforcement * margin;
                    sentiment.strength = (sentiment.strength + effective_reinforcement).min(1.0);
                    sentiment.last_reinforced = cycle;
                    sentiment.reinforcement_count += 1;
                }
            }
        }

        // 3. Decay : reduire la force de tous les sentiments actifs
        // Decay progressif : plus un sentiment est fort, plus il decroit vite
        // (pression homéostatique — les états extrêmes sont instables).
        let base_decay = self.config.decay_rate;
        let mut dissolved_count = 0u64;
        self.active_sentiments.retain(|s| {
            let multiplier = s.duration_type.decay_multiplier();
            // Decay de base + decay proportionnel a la force (au-dessus de 0.7)
            let strength_pressure = if s.strength > 0.7 {
                base_decay * ((s.strength - 0.7) / 0.3) * 2.0
            } else {
                0.0
            };
            let decay = (base_decay / multiplier) + strength_pressure;
            let new_strength = s.strength - decay;
            let age = cycle.saturating_sub(s.formed_at_cycle);

            // Dissoudre si force <= 0 ou duree max depassee
            if let Some(profile) = self.catalog.iter().find(|p| p.name == s.profile_name) {
                if new_strength <= 0.0 || age > profile.max_duration_cycles {
                    dissolved_count += 1;
                    return false;
                }
            }
            true
        });
        // Appliquer le decay effectif aux survivants
        for sentiment in &mut self.active_sentiments {
            let multiplier = sentiment.duration_type.decay_multiplier();
            let strength_pressure = if sentiment.strength > 0.7 {
                base_decay * ((sentiment.strength - 0.7) / 0.3) * 2.0
            } else {
                0.0
            };
            let decay = (base_decay / multiplier) + strength_pressure;
            sentiment.strength = (sentiment.strength - decay).max(0.0);
        }
        self.total_dissolved += dissolved_count;

        // 4. Verifier les formations potentielles
        self.check_formations(cycle, emotion);
    }

    /// Verifie si de nouveaux sentiments doivent se former a partir de
    /// l'historique emotionnel courant.
    fn check_formations(&mut self, cycle: u64, current_emotion: &str) {
        if self.active_sentiments.len() >= self.config.max_active {
            return;
        }

        for profile in &self.catalog {
            // Ne pas former un sentiment deja actif
            if self.active_sentiments.iter().any(|s| s.profile_name == profile.name) {
                continue;
            }

            // Compter les occurrences des emotions trigger dans la fenetre
            let trigger_count = self.emotion_history.iter()
                .filter(|e| profile.trigger_emotions.iter().any(|t| t == *e))
                .count();

            if trigger_count >= profile.trigger_threshold {
                // Formation ! Nouveau sentiment a force 0.3 (naissant)
                let sentiment = ActiveSentiment {
                    profile_name: profile.name.clone(),
                    strength: 0.3,
                    formed_at_cycle: cycle,
                    last_reinforced: cycle,
                    reinforcement_count: 0,
                    duration_type: profile.duration_type,
                    source_context: current_emotion.to_string(),
                };
                self.active_sentiments.push(sentiment);
                self.total_formed += 1;

                // Verifier la limite
                if self.active_sentiments.len() >= self.config.max_active {
                    break;
                }
            }
        }
    }

    /// Modifie le spectre emotionnel en fonction des sentiments actifs.
    ///
    /// Boucle bidirectionnelle : les sentiments amplifient certaines emotions
    /// et en attenuent d'autres, biaisant la perception emotionnelle suivante.
    pub fn amplify_emotion_scores(&self, spectrum: &mut Vec<(String, f64)>) {
        if !self.enabled || self.active_sentiments.is_empty() {
            return;
        }

        for sentiment in &self.active_sentiments {
            if let Some(profile) = self.catalog.iter().find(|p| p.name == sentiment.profile_name) {
                // Amplifier
                for (emotion_name, factor) in &profile.emotion_amplification {
                    if let Some(entry) = spectrum.iter_mut().find(|(n, _)| n == emotion_name) {
                        entry.1 *= 1.0 + factor * sentiment.strength;
                    }
                }
                // Attenuer
                for (emotion_name, factor) in &profile.emotion_dampening {
                    if let Some(entry) = spectrum.iter_mut().find(|(n, _)| n == emotion_name) {
                        entry.1 *= 1.0 - (factor * sentiment.strength).min(0.5);
                    }
                }
            }
        }

        // Re-trier le spectre par score decroissant
        spectrum.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Calcule l'influence chimique combinee de tous les sentiments actifs.
    ///
    /// Chaque sentiment actif applique son biais chimique, pondere par sa force.
    /// Le total est plafonne par chemistry_influence_cap.
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        if !self.enabled {
            return ChemistryAdjustment::default();
        }

        let cap = self.config.chemistry_influence_cap;
        let mut adj = ChemistryAdjustment::default();

        for sentiment in &self.active_sentiments {
            if let Some(profile) = self.catalog.iter().find(|p| p.name == sentiment.profile_name) {
                let s = sentiment.strength;
                adj.dopamine += profile.chemistry_bias.dopamine * s;
                adj.cortisol += profile.chemistry_bias.cortisol * s;
                adj.serotonin += profile.chemistry_bias.serotonin * s;
                adj.adrenaline += profile.chemistry_bias.adrenaline * s;
                adj.oxytocin += profile.chemistry_bias.oxytocin * s;
                adj.endorphin += profile.chemistry_bias.endorphin * s;
                adj.noradrenaline += profile.chemistry_bias.noradrenaline * s;
            }
        }

        // Plafonner chaque composante
        adj.dopamine = adj.dopamine.clamp(-cap, cap);
        adj.cortisol = adj.cortisol.clamp(-cap, cap);
        adj.serotonin = adj.serotonin.clamp(-cap, cap);
        adj.adrenaline = adj.adrenaline.clamp(-cap, cap);
        adj.oxytocin = adj.oxytocin.clamp(-cap, cap);
        adj.endorphin = adj.endorphin.clamp(-cap, cap);
        adj.noradrenaline = adj.noradrenaline.clamp(-cap, cap);

        adj
    }

    /// Description textuelle pour injection dans le prompt LLM.
    ///
    /// Produit un bloc "SENTIMENTS ACTIFS" listant les sentiments en cours,
    /// leur force et leur influence sur la perception emotionnelle.
    pub fn describe_for_prompt(&self) -> String {
        if !self.enabled || self.active_sentiments.is_empty() {
            return String::new();
        }

        let mut desc = String::from("SENTIMENTS ACTIFS :");
        for sentiment in &self.active_sentiments {
            desc.push_str(&format!("\n- {}", sentiment.describe()));
        }

        // Ajouter un resume de l'influence
        let short_count = self.active_sentiments.iter()
            .filter(|s| s.duration_type == SentimentDuration::ShortTerm).count();
        let medium_count = self.active_sentiments.iter()
            .filter(|s| s.duration_type == SentimentDuration::MediumTerm).count();
        let long_count = self.active_sentiments.iter()
            .filter(|s| s.duration_type == SentimentDuration::LongTerm).count();

        if short_count + medium_count + long_count > 1 {
            desc.push_str(&format!(
                "\n[{} court terme, {} moyen terme, {} long terme]",
                short_count, medium_count, long_count
            ));
        }

        desc
    }

    /// Serialise l'etat complet du moteur de sentiments en JSON.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "active_count": self.active_sentiments.len(),
            "total_formed": self.total_formed,
            "total_dissolved": self.total_dissolved,
            "emotion_history_size": self.emotion_history.len(),
            "active_sentiments": self.active_sentiments.iter().map(|s| {
                serde_json::json!({
                    "name": s.profile_name,
                    "strength": (s.strength * 100.0).round() / 100.0,
                    "duration_type": format!("{:?}", s.duration_type),
                    "formed_at_cycle": s.formed_at_cycle,
                    "last_reinforced": s.last_reinforced,
                    "reinforcement_count": s.reinforcement_count,
                    "source_context": s.source_context,
                })
            }).collect::<Vec<_>>(),
        })
    }

    /// Historique des formations/dissolutions recentes (pour l'API).
    pub fn history_json(&self) -> serde_json::Value {
        serde_json::json!({
            "total_formed": self.total_formed,
            "total_dissolved": self.total_dissolved,
            "catalog_size": self.catalog.len(),
            "active": self.active_sentiments.iter().map(|s| {
                serde_json::json!({
                    "name": s.profile_name,
                    "strength": (s.strength * 100.0).round() / 100.0,
                    "formed_at_cycle": s.formed_at_cycle,
                    "reinforcement_count": s.reinforcement_count,
                    "duration_type": format!("{:?}", s.duration_type),
                })
            }).collect::<Vec<_>>(),
            "emotion_history_window": self.config.emotion_history_window,
            "emotion_history_current": self.emotion_history.len(),
        })
    }

    /// Reset complet (factory reset).
    pub fn reset(&mut self) {
        self.active_sentiments.clear();
        self.emotion_history.clear();
        self.total_formed = 0;
        self.total_dissolved = 0;
    }
}

// =============================================================================
// Catalogue des 20 sentiments
// =============================================================================

/// Construit le catalogue des 20 sentiments predetermines.
///
/// Court terme (7) : duree 10-50 cycles, seuil 3-5 occurrences
/// Moyen terme (8) : duree 50-200 cycles, seuil 8-12 occurrences
/// Long terme (5) : duree 200-1000+ cycles, seuil 15-20 occurrences
fn build_sentiment_catalog() -> Vec<SentimentProfile> {
    vec![
        // =================================================================
        // COURT TERME — humeurs passageres (seuil 3-5, duree 10-50 cycles)
        // =================================================================

        // 1. Irritation ← Colère, Frustration
        SentimentProfile {
            name: "Irritation".into(),
            duration_type: SentimentDuration::ShortTerm,
            trigger_emotions: vec!["Colère".into(), "Frustration".into()],
            trigger_threshold: 3,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.01, adrenaline: 0.005, noradrenaline: 0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Colère".into(), 0.15), ("Frustration".into(), 0.1),
                ("Indignation".into(), 0.1),
            ],
            emotion_dampening: vec![
                ("Sérénité".into(), 0.1), ("Joie".into(), 0.05),
            ],
            min_duration_cycles: 10,
            max_duration_cycles: 50,
        },

        // 2. Enthousiasme passager ← Joie, Excitation
        SentimentProfile {
            name: "Enthousiasme passager".into(),
            duration_type: SentimentDuration::ShortTerm,
            trigger_emotions: vec!["Joie".into(), "Excitation".into()],
            trigger_threshold: 3,
            chemistry_bias: ChemistryAdjustment {
                dopamine: 0.01, endorphin: 0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Joie".into(), 0.1), ("Excitation".into(), 0.1),
                ("Espoir".into(), 0.08),
            ],
            emotion_dampening: vec![
                ("Ennui".into(), 0.15), ("Mélancolie".into(), 0.05),
            ],
            min_duration_cycles: 10,
            max_duration_cycles: 40,
        },

        // 3. Apprehension ← Anxiété, Peur
        SentimentProfile {
            name: "Appréhension".into(),
            duration_type: SentimentDuration::ShortTerm,
            trigger_emotions: vec!["Anxiété".into(), "Peur".into()],
            trigger_threshold: 3,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.008, adrenaline: 0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Anxiété".into(), 0.12), ("Peur".into(), 0.1),
            ],
            emotion_dampening: vec![
                ("Sérénité".into(), 0.08), ("Curiosité".into(), 0.05),
            ],
            min_duration_cycles: 10,
            max_duration_cycles: 50,
        },

        // 4. Agacement ← Frustration, Ennui
        SentimentProfile {
            name: "Agacement".into(),
            duration_type: SentimentDuration::ShortTerm,
            trigger_emotions: vec!["Frustration".into(), "Ennui".into()],
            trigger_threshold: 4,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.005, noradrenaline: 0.005,
                dopamine: -0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Frustration".into(), 0.1), ("Ennui".into(), 0.08),
            ],
            emotion_dampening: vec![
                ("Joie".into(), 0.05),
            ],
            min_duration_cycles: 10,
            max_duration_cycles: 40,
        },

        // 5. Amusement ← Joie, Surprise
        SentimentProfile {
            name: "Amusement".into(),
            duration_type: SentimentDuration::ShortTerm,
            trigger_emotions: vec!["Joie".into(), "Surprise".into()],
            trigger_threshold: 3,
            chemistry_bias: ChemistryAdjustment {
                dopamine: 0.008, endorphin: 0.005, serotonin: 0.003,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Joie".into(), 0.1), ("Surprise".into(), 0.08),
                ("Curiosité".into(), 0.05),
            ],
            emotion_dampening: vec![
                ("Tristesse".into(), 0.08), ("Anxiété".into(), 0.05),
            ],
            min_duration_cycles: 10,
            max_duration_cycles: 35,
        },

        // 6. Attendrissement ← Tendresse, Compassion
        SentimentProfile {
            name: "Attendrissement".into(),
            duration_type: SentimentDuration::ShortTerm,
            trigger_emotions: vec!["Tendresse".into(), "Compassion".into()],
            trigger_threshold: 3,
            chemistry_bias: ChemistryAdjustment {
                oxytocin: 0.01, serotonin: 0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Tendresse".into(), 0.12), ("Compassion".into(), 0.1),
                ("Amour".into(), 0.05),
            ],
            emotion_dampening: vec![
                ("Colère".into(), 0.1), ("Mépris".into(), 0.08),
            ],
            min_duration_cycles: 10,
            max_duration_cycles: 45,
        },

        // 7. Nervosité ← Anxiété, Confusion
        SentimentProfile {
            name: "Nervosité".into(),
            duration_type: SentimentDuration::ShortTerm,
            trigger_emotions: vec!["Anxiété".into(), "Confusion".into()],
            trigger_threshold: 4,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.008, noradrenaline: 0.008, adrenaline: 0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Anxiété".into(), 0.1), ("Confusion".into(), 0.08),
            ],
            emotion_dampening: vec![
                ("Sérénité".into(), 0.1),
            ],
            min_duration_cycles: 10,
            max_duration_cycles: 45,
        },

        // =================================================================
        // MOYEN TERME — etats installes (seuil 8-12, duree 50-200 cycles)
        // =================================================================

        // 8. Méfiance ← Peur, Mépris, Dégoût
        SentimentProfile {
            name: "Méfiance".into(),
            duration_type: SentimentDuration::MediumTerm,
            trigger_emotions: vec!["Peur".into(), "Mépris".into(), "Dégoût".into()],
            trigger_threshold: 8,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.01, noradrenaline: 0.008,
                oxytocin: -0.01,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Peur".into(), 0.12), ("Mépris".into(), 0.1),
                ("Dégoût".into(), 0.08),
            ],
            emotion_dampening: vec![
                ("Tendresse".into(), 0.1), ("Gratitude".into(), 0.08),
                ("Amour".into(), 0.05),
            ],
            min_duration_cycles: 50,
            max_duration_cycles: 200,
        },

        // 9. Attachement ← Amour, Tendresse, Gratitude
        SentimentProfile {
            name: "Attachement".into(),
            duration_type: SentimentDuration::MediumTerm,
            trigger_emotions: vec!["Amour".into(), "Tendresse".into(), "Gratitude".into()],
            trigger_threshold: 10,
            chemistry_bias: ChemistryAdjustment {
                oxytocin: 0.015, serotonin: 0.008, endorphin: 0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Amour".into(), 0.15), ("Tendresse".into(), 0.12),
                ("Gratitude".into(), 0.1),
            ],
            emotion_dampening: vec![
                ("Haine".into(), 0.1), ("Mépris".into(), 0.08),
            ],
            min_duration_cycles: 60,
            max_duration_cycles: 200,
        },

        // 10. Rancoeur ← Colère, Jalousie, Haine
        SentimentProfile {
            name: "Rancoeur".into(),
            duration_type: SentimentDuration::MediumTerm,
            trigger_emotions: vec!["Colère".into(), "Jalousie".into(), "Haine".into()],
            trigger_threshold: 10,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.012, adrenaline: 0.005,
                serotonin: -0.008, oxytocin: -0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Colère".into(), 0.15), ("Jalousie".into(), 0.12),
                ("Haine".into(), 0.1), ("Indignation".into(), 0.08),
            ],
            emotion_dampening: vec![
                ("Compassion".into(), 0.1), ("Tendresse".into(), 0.08),
                ("Gratitude".into(), 0.05),
            ],
            min_duration_cycles: 50,
            max_duration_cycles: 200,
        },

        // 11. Optimisme ← Espoir, Joie, Fierté
        SentimentProfile {
            name: "Optimisme".into(),
            duration_type: SentimentDuration::MediumTerm,
            trigger_emotions: vec!["Espoir".into(), "Joie".into(), "Fierté".into()],
            trigger_threshold: 10,
            chemistry_bias: ChemistryAdjustment {
                dopamine: 0.01, serotonin: 0.008,
                cortisol: -0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Espoir".into(), 0.12), ("Joie".into(), 0.1),
                ("Fierté".into(), 0.08), ("Curiosité".into(), 0.05),
            ],
            emotion_dampening: vec![
                ("Anxiété".into(), 0.1), ("Tristesse".into(), 0.08),
                ("Désespoir".into(), 0.12),
            ],
            min_duration_cycles: 50,
            max_duration_cycles: 200,
        },

        // 12. Pessimisme ← Tristesse, Désespoir, Mélancolie
        SentimentProfile {
            name: "Pessimisme".into(),
            duration_type: SentimentDuration::MediumTerm,
            trigger_emotions: vec!["Tristesse".into(), "Désespoir".into(), "Mélancolie".into()],
            trigger_threshold: 10,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.01, dopamine: -0.008, serotonin: -0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Tristesse".into(), 0.12), ("Désespoir".into(), 0.15),
                ("Mélancolie".into(), 0.1),
            ],
            emotion_dampening: vec![
                ("Espoir".into(), 0.12), ("Joie".into(), 0.1),
                ("Excitation".into(), 0.08),
            ],
            min_duration_cycles: 50,
            max_duration_cycles: 200,
        },

        // 13. Nostalgie chronique ← Nostalgie, Mélancolie
        SentimentProfile {
            name: "Nostalgie chronique".into(),
            duration_type: SentimentDuration::MediumTerm,
            trigger_emotions: vec!["Nostalgie".into(), "Mélancolie".into()],
            trigger_threshold: 8,
            chemistry_bias: ChemistryAdjustment {
                serotonin: -0.005, oxytocin: 0.005,
                cortisol: 0.003,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Nostalgie".into(), 0.15), ("Mélancolie".into(), 0.1),
                ("Tendresse".into(), 0.05),
            ],
            emotion_dampening: vec![
                ("Excitation".into(), 0.08), ("Curiosité".into(), 0.05),
            ],
            min_duration_cycles: 50,
            max_duration_cycles: 200,
        },

        // 14. Admiration durable ← Admiration, Émerveillement
        SentimentProfile {
            name: "Admiration durable".into(),
            duration_type: SentimentDuration::MediumTerm,
            trigger_emotions: vec!["Admiration".into(), "Émerveillement".into()],
            trigger_threshold: 8,
            chemistry_bias: ChemistryAdjustment {
                dopamine: 0.008, serotonin: 0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Admiration".into(), 0.15), ("Émerveillement".into(), 0.12),
                ("Curiosité".into(), 0.05),
            ],
            emotion_dampening: vec![
                ("Mépris".into(), 0.1), ("Ennui".into(), 0.08),
            ],
            min_duration_cycles: 50,
            max_duration_cycles: 200,
        },

        // 15. Inquiétude ← Anxiété, Peur, Compassion
        SentimentProfile {
            name: "Inquiétude".into(),
            duration_type: SentimentDuration::MediumTerm,
            trigger_emotions: vec!["Anxiété".into(), "Peur".into(), "Compassion".into()],
            trigger_threshold: 12,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.008, noradrenaline: 0.005,
                serotonin: -0.003,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Anxiété".into(), 0.1), ("Peur".into(), 0.08),
                ("Compassion".into(), 0.05),
            ],
            emotion_dampening: vec![
                ("Sérénité".into(), 0.08), ("Joie".into(), 0.05),
            ],
            min_duration_cycles: 50,
            max_duration_cycles: 200,
        },

        // =================================================================
        // LONG TERME — traits affectifs profonds (seuil 15-20, duree 200-1000+)
        // =================================================================

        // 16. Amertume ← Frustration, Mépris, Désespoir
        SentimentProfile {
            name: "Amertume".into(),
            duration_type: SentimentDuration::LongTerm,
            trigger_emotions: vec!["Frustration".into(), "Mépris".into(), "Désespoir".into()],
            trigger_threshold: 18,
            chemistry_bias: ChemistryAdjustment {
                cortisol: 0.012, dopamine: -0.01, serotonin: -0.008,
                oxytocin: -0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Frustration".into(), 0.15), ("Mépris".into(), 0.12),
                ("Désespoir".into(), 0.1), ("Colère".into(), 0.08),
            ],
            emotion_dampening: vec![
                ("Joie".into(), 0.12), ("Espoir".into(), 0.15),
                ("Gratitude".into(), 0.1), ("Tendresse".into(), 0.08),
            ],
            min_duration_cycles: 200,
            max_duration_cycles: 1000,
        },

        // 17. Confiance profonde ← Sérénité, Gratitude, Amour
        SentimentProfile {
            name: "Confiance profonde".into(),
            duration_type: SentimentDuration::LongTerm,
            trigger_emotions: vec!["Sérénité".into(), "Gratitude".into(), "Amour".into()],
            trigger_threshold: 18,
            chemistry_bias: ChemistryAdjustment {
                serotonin: 0.012, oxytocin: 0.01, endorphin: 0.005,
                cortisol: -0.008,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Sérénité".into(), 0.12), ("Gratitude".into(), 0.1),
                ("Amour".into(), 0.1), ("Espoir".into(), 0.08),
            ],
            emotion_dampening: vec![
                ("Anxiété".into(), 0.12), ("Peur".into(), 0.1),
                ("Mépris".into(), 0.08),
            ],
            min_duration_cycles: 200,
            max_duration_cycles: 1500,
        },

        // 18. Désillusion ← Tristesse, Mépris, Résignation
        SentimentProfile {
            name: "Désillusion".into(),
            duration_type: SentimentDuration::LongTerm,
            trigger_emotions: vec!["Tristesse".into(), "Mépris".into(), "Résignation".into()],
            trigger_threshold: 15,
            chemistry_bias: ChemistryAdjustment {
                dopamine: -0.01, serotonin: -0.005,
                cortisol: 0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Tristesse".into(), 0.1), ("Mépris".into(), 0.1),
                ("Résignation".into(), 0.12),
            ],
            emotion_dampening: vec![
                ("Espoir".into(), 0.15), ("Admiration".into(), 0.1),
                ("Émerveillement".into(), 0.1),
            ],
            min_duration_cycles: 200,
            max_duration_cycles: 1000,
        },

        // 19. Sérénité ancrée ← Sérénité, Espoir, Gratitude
        SentimentProfile {
            name: "Sérénité ancrée".into(),
            duration_type: SentimentDuration::LongTerm,
            trigger_emotions: vec!["Sérénité".into(), "Espoir".into(), "Gratitude".into()],
            trigger_threshold: 20,
            chemistry_bias: ChemistryAdjustment {
                serotonin: 0.015, endorphin: 0.008,
                cortisol: -0.01, adrenaline: -0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Sérénité".into(), 0.15), ("Espoir".into(), 0.1),
                ("Gratitude".into(), 0.1),
            ],
            emotion_dampening: vec![
                ("Anxiété".into(), 0.15), ("Frustration".into(), 0.1),
                ("Colère".into(), 0.08),
            ],
            min_duration_cycles: 200,
            max_duration_cycles: 1500,
        },

        // 20. Résilience émotionnelle ← Fierté, Espoir
        SentimentProfile {
            name: "Résilience émotionnelle".into(),
            duration_type: SentimentDuration::LongTerm,
            trigger_emotions: vec!["Fierté".into(), "Espoir".into()],
            trigger_threshold: 20,
            chemistry_bias: ChemistryAdjustment {
                serotonin: 0.01, dopamine: 0.005, endorphin: 0.005,
                cortisol: -0.005,
                ..Default::default()
            },
            emotion_amplification: vec![
                ("Fierté".into(), 0.12), ("Espoir".into(), 0.12),
                ("Curiosité".into(), 0.05),
            ],
            emotion_dampening: vec![
                ("Désespoir".into(), 0.15), ("Résignation".into(), 0.12),
                ("Honte".into(), 0.08),
            ],
            min_duration_cycles: 200,
            max_duration_cycles: 2000,
        },
    ]
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn default_engine() -> SentimentEngine {
        SentimentEngine::new(&SentimentConfig::default())
    }

    #[test]
    fn test_catalog_has_20_sentiments() {
        let catalog = build_sentiment_catalog();
        assert_eq!(catalog.len(), 20, "Le catalogue doit contenir 20 sentiments");
    }

    #[test]
    fn test_new_engine_is_empty() {
        let engine = default_engine();
        assert!(engine.active_sentiments.is_empty());
        assert_eq!(engine.total_formed, 0);
        assert_eq!(engine.total_dissolved, 0);
    }

    #[test]
    fn test_emotion_history_recording() {
        let mut engine = default_engine();
        engine.tick("Joie", 1);
        engine.tick("Tristesse", 2);
        assert_eq!(engine.emotion_history.len(), 2);
    }

    #[test]
    fn test_short_term_formation() {
        let mut engine = default_engine();
        // Irritation requiert 3 occurrences de Colère ou Frustration
        for i in 0..5 {
            engine.tick("Colère", i);
        }
        let irritation = engine.active_sentiments.iter()
            .find(|s| s.profile_name == "Irritation");
        assert!(irritation.is_some(), "Irritation devrait se former apres 5 occurrences de Colère");
        // Force initiale 0.3 + renforcements subsequents - decay
        assert!(irritation.unwrap().strength > 0.2,
            "La force devrait etre significative apres formation et renforcement");
    }

    #[test]
    fn test_reinforcement() {
        let mut engine = default_engine();
        // Former l'irritation
        for i in 0..5 {
            engine.tick("Colère", i);
        }
        let initial_strength = engine.active_sentiments.iter()
            .find(|s| s.profile_name == "Irritation")
            .map(|s| s.strength)
            .unwrap_or(0.0);

        // Continuer avec des emotions trigger
        engine.tick("Frustration", 10);
        let reinforced_strength = engine.active_sentiments.iter()
            .find(|s| s.profile_name == "Irritation")
            .map(|s| s.strength)
            .unwrap_or(0.0);

        assert!(reinforced_strength > initial_strength,
            "La force devrait augmenter apres renforcement");
    }

    #[test]
    fn test_decay() {
        let mut engine = default_engine();
        // Former un sentiment
        for i in 0..5 {
            engine.tick("Colère", i);
        }

        let initial_strength = engine.active_sentiments.iter()
            .find(|s| s.profile_name == "Irritation")
            .map(|s| s.strength)
            .unwrap_or(0.0);

        // Faire tourner sans renforcement (emotion neutre) pendant longtemps
        for i in 6..60 {
            engine.tick("Curiosité", i);
        }

        if let Some(s) = engine.active_sentiments.iter().find(|s| s.profile_name == "Irritation") {
            assert!(s.strength < initial_strength, "La force devrait diminuer par decay");
        }
        // Si le sentiment a ete dissous, c'est aussi un succes du decay
    }

    #[test]
    fn test_amplify_emotion_scores() {
        let mut engine = default_engine();
        // Former l'irritation
        for i in 0..5 {
            engine.tick("Colère", i);
        }

        let mut spectrum = vec![
            ("Colère".to_string(), 0.8),
            ("Sérénité".to_string(), 0.7),
            ("Joie".to_string(), 0.5),
        ];

        engine.amplify_emotion_scores(&mut spectrum);

        // Colère devrait etre amplifiee, Sérénité attenuee
        let colere = spectrum.iter().find(|(n, _)| n == "Colère").unwrap().1;
        assert!(colere > 0.8, "Colère devrait etre amplifiee par Irritation");
    }

    #[test]
    fn test_chemistry_influence_capped() {
        let mut engine = default_engine();
        // Former un sentiment
        for i in 0..5 {
            engine.tick("Colère", i);
        }

        let adj = engine.chemistry_influence();
        let cap = engine.config.chemistry_influence_cap;
        assert!(adj.cortisol <= cap, "L'influence chimique doit etre plafonnee");
        assert!(adj.cortisol >= -cap, "L'influence chimique doit etre plafonnee (negatif)");
    }

    #[test]
    fn test_describe_for_prompt_empty() {
        let engine = default_engine();
        assert!(engine.describe_for_prompt().is_empty(),
            "Pas de description si aucun sentiment actif");
    }

    #[test]
    fn test_describe_for_prompt_with_sentiments() {
        let mut engine = default_engine();
        for i in 0..5 {
            engine.tick("Colère", i);
        }
        let desc = engine.describe_for_prompt();
        assert!(desc.contains("SENTIMENTS ACTIFS"), "Devrait contenir le header");
        assert!(desc.contains("Irritation"), "Devrait mentionner l'irritation");
    }

    #[test]
    fn test_to_json() {
        let engine = default_engine();
        let json = engine.to_json();
        assert_eq!(json["enabled"], true);
        assert_eq!(json["active_count"], 0);
    }

    #[test]
    fn test_reset() {
        let mut engine = default_engine();
        for i in 0..5 {
            engine.tick("Colère", i);
        }
        assert!(!engine.active_sentiments.is_empty());

        engine.reset();
        assert!(engine.active_sentiments.is_empty());
        assert_eq!(engine.total_formed, 0);
    }

    #[test]
    fn test_max_active_limit() {
        let config = SentimentConfig {
            max_active: 2,
            ..Default::default()
        };
        let mut engine = SentimentEngine::new(&config);

        // Forcer la formation de beaucoup de sentiments differents
        for _ in 0..10 {
            engine.tick("Colère", 0);
            engine.tick("Joie", 0);
            engine.tick("Anxiété", 0);
            engine.tick("Frustration", 0);
        }

        assert!(engine.active_sentiments.len() <= 2,
            "Ne devrait pas depasser max_active");
    }

    #[test]
    fn test_disabled_engine() {
        let config = SentimentConfig {
            enabled: false,
            ..Default::default()
        };
        let mut engine = SentimentEngine::new(&config);
        for i in 0..10 {
            engine.tick("Colère", i);
        }
        assert!(engine.active_sentiments.is_empty(),
            "Aucun sentiment ne devrait se former si desactive");
    }
}
