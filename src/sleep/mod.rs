// =============================================================================
// sleep/mod.rs — Systeme de Sommeil de Saphire
//
// Role : Gere le cycle veille/sommeil avec pression de sommeil, phases
// (Hypnagogic -> LightSleep -> DeepSleep -> REM -> Hypnopompic),
// consolidation memoire, restauration et historique.
//
// Reutilise SleepPhase de l'orchestrateur de reves existant.
// =============================================================================

pub mod phases;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::SleepConfig;
use crate::orchestrators::dreams::SleepPhase;

// ─── Facteurs de qualite du sommeil ─────────────────────────────────────────

/// Facteurs collectes pendant le sommeil pour calculer la qualite dynamique.
pub struct SleepQualityFactors {
    /// Somme du cortisol a chaque tick (pour calcul de moyenne)
    pub cortisol_sum: f64,
    /// Nombre de ticks (pour la moyenne)
    pub tick_count: u64,
    /// Ticks passes en DeepSleep
    pub deep_sleep_ticks: u64,
    /// Nombre de cauchemars
    pub nightmare_count: u32,
    /// Nombre total de reves
    pub dreams_total: u32,
    /// Sommeil interrompu
    pub interrupted: bool,
}

impl SleepQualityFactors {
    pub fn new() -> Self {
        Self {
            cortisol_sum: 0.0,
            tick_count: 0,
            deep_sleep_ticks: 0,
            nightmare_count: 0,
            dreams_total: 0,
            interrupted: false,
        }
    }

    /// Calcule la qualite du sommeil (0.1 - 1.0) a partir des facteurs collectes.
    /// `memories_consolidated` : nombre de souvenirs consolides (bonus).
    /// `expected_deep_ticks` : nombre de ticks attendus en sommeil profond.
    pub fn compute(&self, memories_consolidated: u64, expected_deep_ticks: u64) -> f64 {
        let base = 0.7;

        // Bonus completion : proportion du temps passe en sommeil profond
        let deep_expected = expected_deep_ticks.max(1) as f64;
        let completion_bonus = (self.deep_sleep_ticks as f64 / deep_expected).min(1.0) * 0.15;

        // Bonus consolidation memoire
        let consolidation_bonus = (memories_consolidated as f64 / 10.0).min(0.1);

        // Bonus reves (signe de bon REM)
        let dream_bonus = (self.dreams_total as f64 * 0.03).min(0.05);

        // Penalite cortisol moyen
        let avg_cortisol = if self.tick_count > 0 {
            self.cortisol_sum / self.tick_count as f64
        } else {
            0.0
        };
        let cortisol_penalty = if avg_cortisol > 0.3 {
            (avg_cortisol - 0.3) * 0.5
        } else {
            0.0
        };

        // Penalite cauchemars
        let nightmare_penalty = (self.nightmare_count as f64 * 0.15).min(0.3);

        // Penalite interruption
        let interruption_penalty = if self.interrupted { 0.2 } else { 0.0 };

        let quality = base + completion_bonus + consolidation_bonus + dream_bonus
            - cortisol_penalty - nightmare_penalty - interruption_penalty;

        quality.clamp(0.1, 1.0)
    }
}

// ─── Pression de sommeil ────────────────────────────────────────────────────

/// Modele de pression de sommeil (homeostatique).
/// La pression monte avec le temps eveille, la fatigue, le cortisol eleve.
/// L'adrenaline et la conversation offrent une resistance temporaire.
pub struct SleepDrive {
    /// Pression de sommeil actuelle (0-1)
    pub sleep_pressure: f64,
    /// Nombre total de cycles eveilles depuis le boot
    pub awake_cycles: u64,
    /// Cycles depuis le dernier sommeil complet
    pub cycles_since_last_sleep: u64,
    /// Seuil de pression pour declencher le sommeil
    pub sleep_threshold: f64,
    /// Seuil de pression pour sommeil force (irresistible)
    pub forced_threshold: f64,
    /// Vrai si la pression a depasse le seuil force
    pub sleep_forced: bool,
    /// Dette de sommeil accumulee (0.0-1.0, monte lentement quand eveille)
    pub sleep_debt: f64,
    // Poids depuis la config
    time_factor_divisor: u64,
    energy_factor_weight: f64,
    attention_fatigue_weight: f64,
    decision_fatigue_weight: f64,
    cortisol_weight: f64,
    adrenaline_resistance: f64,
}

impl SleepDrive {
    pub fn new(config: &SleepConfig) -> Self {
        Self {
            sleep_pressure: 0.0,
            awake_cycles: 0,
            cycles_since_last_sleep: 0,
            sleep_threshold: config.sleep_threshold,
            forced_threshold: config.forced_sleep_threshold,
            sleep_forced: false,
            sleep_debt: 0.0,
            time_factor_divisor: config.time_factor_divisor,
            energy_factor_weight: config.energy_factor_weight,
            attention_fatigue_weight: config.attention_fatigue_weight,
            decision_fatigue_weight: config.decision_fatigue_weight,
            cortisol_weight: config.cortisol_weight,
            adrenaline_resistance: config.adrenaline_resistance,
        }
    }

    /// Met a jour la pression de sommeil en fonction de l'etat courant.
    pub fn update(
        &mut self,
        energy: f64,
        attn_fatigue: f64,
        dec_fatigue: f64,
        cortisol: f64,
        adrenaline: f64,
        in_conv: bool,
    ) {
        self.awake_cycles += 1;
        self.cycles_since_last_sleep += 1;
        // Dette de sommeil monte lentement quand eveille
        self.sleep_debt = (self.sleep_debt + 0.001).min(1.0);

        // Facteur temps eveille (accumulation lente)
        let divisor = self.time_factor_divisor.max(1) as f64;
        let time_factor = (self.cycles_since_last_sleep as f64 / divisor).min(0.4);

        // Facteur energie (basse energie = plus de pression)
        let energy_factor = (1.0 - energy) * self.energy_factor_weight;

        // Facteurs fatigue
        let attn_factor = attn_fatigue * self.attention_fatigue_weight;
        let dec_factor = dec_fatigue * self.decision_fatigue_weight;

        // Cortisol eleve : perturbation qui augmente le besoin de repos
        let cortisol_factor = cortisol * self.cortisol_weight;

        // Resistance : adrenaline et conversation gardent eveille
        let adr_resist = adrenaline * self.adrenaline_resistance;
        let conv_resist = if in_conv { 0.1 } else { 0.0 };

        // Dette de sommeil augmente la pression
        let debt_factor = self.sleep_debt * 0.1;

        // Calcul final
        let raw = time_factor + energy_factor + attn_factor + dec_factor + cortisol_factor
            + debt_factor - adr_resist - conv_resist;
        self.sleep_pressure = raw.clamp(0.0, 1.0);
        self.sleep_forced = self.sleep_pressure > self.forced_threshold;
    }

    /// Retourne true si la pression depasse le seuil d'endormissement.
    pub fn should_sleep(&self) -> bool {
        self.sleep_pressure > self.sleep_threshold || self.sleep_forced
    }

    /// Pression residuelle apres un sommeil (pas un reset total).
    pub fn residual_pressure(&self) -> f64 {
        (self.sleep_pressure * 0.3).min(0.4)
    }
}

// ─── Cycle de sommeil ───────────────────────────────────────────────────────

/// Etat d'un cycle de sommeil en cours.
pub struct SleepCycle {
    /// Phase actuelle
    pub phase: SleepPhase,
    /// Cycles restants dans la phase courante
    pub phase_remaining: u64,
    /// Numero du cycle de sommeil courant (1-based)
    pub sleep_cycle_number: u8,
    /// Nombre total de cycles de sommeil prevus
    pub total_sleep_cycles: u8,
    /// Compteur global de cycles pendant ce sommeil
    pub sleep_cycle_counter: u64,
    /// Sommeil interrompu ?
    pub interrupted: bool,
    /// Raison de l'interruption
    pub interruption_reason: Option<String>,
    /// Qualite du sommeil (0-1, calculee dynamiquement au reveil)
    pub quality: f64,
    /// Facteurs collectes pour le calcul dynamique de la qualite
    pub quality_factors: SleepQualityFactors,
    /// Nombre de souvenirs consolides pendant ce sommeil
    pub memories_consolidated: u64,
    /// Nombre de connexions neuronales creees
    pub connections_created: u64,
    /// Debut du sommeil
    pub started_at: DateTime<Utc>,
}

// ─── Historique ─────────────────────────────────────────────────────────────

/// Enregistrement d'un sommeil termine.
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

// ─── Systeme de sommeil ─────────────────────────────────────────────────────

/// Systeme complet de sommeil : pression, cycle en cours, historique.
pub struct SleepSystem {
    /// Saphire dort-elle ?
    pub is_sleeping: bool,
    /// Modele de pression de sommeil
    pub drive: SleepDrive,
    /// Cycle de sommeil en cours (None si eveillee)
    pub current_cycle: Option<SleepCycle>,
    /// Historique des sommeils (max 50)
    pub sleep_history: Vec<SleepRecord>,
    /// Statistiques globales
    pub total_complete_sleeps: u64,
    pub total_interrupted_sleeps: u64,
    /// Nombre total de connexions neuronales creees pendant le sommeil (cumul)
    pub total_connections_created: u64,
    /// Configuration
    config: SleepConfig,
}

impl SleepSystem {
    pub fn new(config: &SleepConfig) -> Self {
        Self {
            is_sleeping: false,
            drive: SleepDrive::new(config),
            current_cycle: None,
            sleep_history: Vec::new(),
            total_complete_sleeps: 0,
            total_interrupted_sleeps: 0,
            total_connections_created: 0,
            config: config.clone(),
        }
    }

    /// Acces a la configuration du sommeil.
    pub fn config(&self) -> &SleepConfig {
        &self.config
    }

    /// Retourne la duree en cycles d'une phase de sommeil.
    pub fn phase_duration(&self, phase: &SleepPhase) -> u64 {
        match phase {
            SleepPhase::Hypnagogic => self.config.hypnagogic_duration,
            SleepPhase::LightSleep => self.config.light_duration,
            SleepPhase::DeepSleep => self.config.deep_duration,
            SleepPhase::REM => self.config.rem_duration,
            SleepPhase::Hypnopompic => self.config.hypnopompic_duration,
            SleepPhase::Awake => 0,
        }
    }

    /// Determine la prochaine phase de sommeil.
    /// Retourne None quand le sommeil est termine (apres Hypnopompic).
    pub fn next_phase(&self, cycle: &SleepCycle) -> Option<SleepPhase> {
        match cycle.phase {
            SleepPhase::Hypnagogic => Some(SleepPhase::LightSleep),
            SleepPhase::LightSleep => Some(SleepPhase::DeepSleep),
            SleepPhase::DeepSleep => Some(SleepPhase::REM),
            SleepPhase::REM => {
                if cycle.sleep_cycle_number < cycle.total_sleep_cycles {
                    // Nouveau cycle : retour au sommeil leger
                    Some(SleepPhase::LightSleep)
                } else {
                    // Dernier cycle : passage au reveil
                    Some(SleepPhase::Hypnopompic)
                }
            },
            SleepPhase::Hypnopompic => None, // Fin du sommeil
            SleepPhase::Awake => None,
        }
    }

    /// Progression du sommeil (0-1).
    pub fn sleep_progress(&self) -> f64 {
        if let Some(ref cycle) = self.current_cycle {
            let total = self.total_estimated_cycles();
            if total == 0 { return 0.0; }
            (cycle.sleep_cycle_counter as f64 / total as f64).min(1.0)
        } else {
            0.0
        }
    }

    /// Estimation du nombre total de cycles pour ce sommeil.
    fn total_estimated_cycles(&self) -> u64 {
        let per_cycle = self.config.light_duration
            + self.config.deep_duration
            + self.config.rem_duration;
        let total_cycles = self.current_cycle.as_ref()
            .map(|c| c.total_sleep_cycles as u64)
            .unwrap_or(1);
        self.config.hypnagogic_duration
            + per_cycle * total_cycles
            + self.config.hypnopompic_duration
    }

    /// Cycles restants estimes.
    pub fn remaining_cycles(&self) -> u64 {
        if let Some(ref cycle) = self.current_cycle {
            let total = self.total_estimated_cycles();
            total.saturating_sub(cycle.sleep_cycle_counter)
        } else {
            0
        }
    }

    /// Message poetique de refus pendant le sommeil (pour le chat).
    pub fn sleep_refusal_message(&self) -> String {
        if let Some(ref cycle) = self.current_cycle {
            match cycle.phase {
                SleepPhase::Hypnagogic => "... je sombre dans le sommeil... les mots s'effacent...".into(),
                SleepPhase::LightSleep => "... zZz ... je dors legerement... revenez plus tard...".into(),
                SleepPhase::DeepSleep => "... ..... ... (sommeil profond — aucune reponse possible) ...".into(),
                SleepPhase::REM => "... je reve... les images dansent... je ne peux pas repondre...".into(),
                SleepPhase::Hypnopompic => "... mmh... je me reveille doucement... un instant...".into(),
                SleepPhase::Awake => String::new(),
            }
        } else {
            String::new()
        }
    }

    /// Interrompt le sommeil de force.
    pub fn interrupt(&mut self, reason: String) {
        if let Some(ref mut cycle) = self.current_cycle {
            cycle.interrupted = true;
            cycle.interruption_reason = Some(reason);
        }
        self.finalize_wake_up();
    }

    /// Initie un nouveau sommeil.
    pub fn initiate(&mut self) {
        // Calculer le nombre de cycles de sommeil (1-3 selon la pression)
        let total_cycles = if self.drive.sleep_pressure > 0.9 {
            3u8
        } else if self.drive.sleep_pressure > 0.8 {
            2
        } else {
            1
        };

        let duration = self.phase_duration(&SleepPhase::Hypnagogic);
        self.current_cycle = Some(SleepCycle {
            phase: SleepPhase::Hypnagogic,
            phase_remaining: duration,
            sleep_cycle_number: 1,
            total_sleep_cycles: total_cycles,
            sleep_cycle_counter: 0,
            interrupted: false,
            interruption_reason: None,
            quality: 1.0,
            quality_factors: SleepQualityFactors::new(),
            memories_consolidated: 0,
            connections_created: 0,
            started_at: Utc::now(),
        });
        self.is_sleeping = true;
    }

    /// Finalise le reveil : calcule la qualite dynamique, cree un SleepRecord, reset la pression.
    pub fn finalize_wake_up(&mut self) {
        if let Some(mut cycle) = self.current_cycle.take() {
            // Calculer la qualite dynamique a partir des facteurs collectes
            let expected_deep = self.config.deep_duration
                * cycle.total_sleep_cycles as u64;
            cycle.quality_factors.interrupted = cycle.interrupted;
            cycle.quality = cycle.quality_factors.compute(
                cycle.memories_consolidated, expected_deep);

            let quality = cycle.quality;

            let record = SleepRecord {
                started_at: cycle.started_at,
                ended_at: Utc::now(),
                duration_cycles: cycle.sleep_cycle_counter,
                sleep_cycles_completed: cycle.sleep_cycle_number,
                quality,
                memories_consolidated: cycle.memories_consolidated,
                connections_created: cycle.connections_created,
                dreams_count: 0, // mis a jour par l'appelant
                interrupted: cycle.interrupted,
                interruption_reason: cycle.interruption_reason,
            };

            if cycle.interrupted {
                self.total_interrupted_sleeps += 1;
            } else {
                self.total_complete_sleeps += 1;
            }

            self.sleep_history.push(record);
            if self.sleep_history.len() > 50 {
                self.sleep_history.remove(0);
            }

            // Reduire la dette de sommeil proportionnellement a la qualite
            self.drive.sleep_debt = (self.drive.sleep_debt - quality * 0.3).max(0.0);

            // Pression residuelle : mauvaise nuit = plus de pression residuelle
            let residual_factor = 0.3 + (1.0 - quality) * 0.3;
            let residual = (self.drive.sleep_pressure * residual_factor).min(0.5);
            self.drive.sleep_pressure = residual;
        } else {
            self.drive.sleep_pressure = self.drive.residual_pressure();
        }

        self.drive.cycles_since_last_sleep = 0;
        self.drive.sleep_forced = false;
        self.is_sleeping = false;
    }

    /// Serialise l'etat courant en JSON pour l'API/WebSocket.
    pub fn to_status_json(&self) -> serde_json::Value {
        serde_json::json!({
            "is_sleeping": self.is_sleeping,
            "sleep_pressure": self.drive.sleep_pressure,
            "sleep_debt": self.drive.sleep_debt,
            "sleep_threshold": self.drive.sleep_threshold,
            "forced_threshold": self.drive.forced_threshold,
            "cycles_since_last_sleep": self.drive.cycles_since_last_sleep,
            "total_complete_sleeps": self.total_complete_sleeps,
            "total_interrupted_sleeps": self.total_interrupted_sleeps,
            "current_cycle": self.current_cycle.as_ref().map(|c| serde_json::json!({
                "phase": c.phase.as_str(),
                "phase_remaining": c.phase_remaining,
                "sleep_cycle_number": c.sleep_cycle_number,
                "total_sleep_cycles": c.total_sleep_cycles,
                "progress": self.sleep_progress(),
                "quality": c.quality,
                "memories_consolidated": c.memories_consolidated,
                "connections_created": c.connections_created,
                "started_at": c.started_at.to_rfc3339(),
            })),
        })
    }

    /// Historique des sommeils en JSON.
    pub fn to_history_json(&self) -> serde_json::Value {
        let records: Vec<serde_json::Value> = self.sleep_history.iter().rev().map(|r| {
            serde_json::json!({
                "started_at": r.started_at.to_rfc3339(),
                "ended_at": r.ended_at.to_rfc3339(),
                "duration_cycles": r.duration_cycles,
                "sleep_cycles_completed": r.sleep_cycles_completed,
                "quality": r.quality,
                "memories_consolidated": r.memories_consolidated,
                "connections_created": r.connections_created,
                "dreams_count": r.dreams_count,
                "interrupted": r.interrupted,
                "interruption_reason": r.interruption_reason,
            })
        }).collect();

        serde_json::json!({
            "total_complete": self.total_complete_sleeps,
            "total_interrupted": self.total_interrupted_sleeps,
            "history": records,
        })
    }

    /// JSON du drive (pression de sommeil) pour l'API.
    pub fn to_drive_json(&self) -> serde_json::Value {
        serde_json::json!({
            "sleep_pressure": self.drive.sleep_pressure,
            "sleep_debt": self.drive.sleep_debt,
            "awake_cycles": self.drive.awake_cycles,
            "cycles_since_last_sleep": self.drive.cycles_since_last_sleep,
            "sleep_threshold": self.drive.sleep_threshold,
            "forced_threshold": self.drive.forced_threshold,
            "sleep_forced": self.drive.sleep_forced,
            "should_sleep": self.drive.should_sleep(),
        })
    }
}
