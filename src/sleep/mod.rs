// =============================================================================
// sleep/mod.rs — Saphire's Sleep System
//
// Role: Manages the wake/sleep cycle with sleep pressure, phases
// (Hypnagogic -> LightSleep -> DeepSleep -> REM -> Hypnopompic),
// memory consolidation, restoration, and history.
//
// Reuses SleepPhase from the existing dream orchestrator.
// =============================================================================

pub mod phases;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::SleepConfig;
use crate::orchestrators::dreams::SleepPhase;

// ─── Sleep quality factors ──────────────────────────────────────────────────

/// Factors collected during sleep to compute dynamic quality.
pub struct SleepQualityFactors {
    /// Sum of cortisol at each tick (for average computation)
    pub cortisol_sum: f64,
    /// Number of ticks (for averaging)
    pub tick_count: u64,
    /// Ticks spent in DeepSleep
    pub deep_sleep_ticks: u64,
    /// Number of nightmares
    pub nightmare_count: u32,
    /// Total number of dreams
    pub dreams_total: u32,
    /// Whether sleep was interrupted
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

    /// Computes sleep quality (0.1 - 1.0) from the collected factors.
    /// `memories_consolidated`: number of consolidated memories (bonus).
    /// `expected_deep_ticks`: expected number of ticks in deep sleep.
    pub fn compute(&self, memories_consolidated: u64, expected_deep_ticks: u64) -> f64 {
        let base = 0.7;

        // Completion bonus: proportion of time spent in deep sleep
        let deep_expected = expected_deep_ticks.max(1) as f64;
        let completion_bonus = (self.deep_sleep_ticks as f64 / deep_expected).min(1.0) * 0.15;

        // Memory consolidation bonus
        let consolidation_bonus = (memories_consolidated as f64 / 10.0).min(0.1);

        // Dream bonus (sign of good REM)
        let dream_bonus = (self.dreams_total as f64 * 0.03).min(0.05);

        // Average cortisol penalty
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

        // Nightmare penalty
        let nightmare_penalty = (self.nightmare_count as f64 * 0.15).min(0.3);

        // Interruption penalty
        let interruption_penalty = if self.interrupted { 0.2 } else { 0.0 };

        let quality = base + completion_bonus + consolidation_bonus + dream_bonus
            - cortisol_penalty - nightmare_penalty - interruption_penalty;

        quality.clamp(0.1, 1.0)
    }
}

// ─── Sleep pressure ─────────────────────────────────────────────────────────

/// Homeostatic sleep pressure model.
/// Pressure rises with time awake, fatigue, and high cortisol.
/// Adrenaline and conversation provide temporary resistance.
pub struct SleepDrive {
    /// Current sleep pressure (0-1)
    pub sleep_pressure: f64,
    /// Total awake cycles since boot
    pub awake_cycles: u64,
    /// Cycles since last complete sleep
    pub cycles_since_last_sleep: u64,
    /// Pressure threshold to trigger sleep
    pub sleep_threshold: f64,
    /// Pressure threshold for forced (irresistible) sleep
    pub forced_threshold: f64,
    /// True if pressure exceeded the forced threshold
    pub sleep_forced: bool,
    /// Accumulated sleep debt (0.0-1.0, rises slowly while awake)
    pub sleep_debt: f64,
    // Weights from config
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

    /// Updates sleep pressure based on the current state.
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
        // Sleep debt rises slowly while awake
        self.sleep_debt = (self.sleep_debt + 0.001).min(1.0);

        // Time awake factor (slow accumulation)
        let divisor = self.time_factor_divisor.max(1) as f64;
        let time_factor = (self.cycles_since_last_sleep as f64 / divisor).min(0.4);

        // Energy factor (low energy = more pressure)
        let energy_factor = (1.0 - energy) * self.energy_factor_weight;

        // Fatigue factors
        let attn_factor = attn_fatigue * self.attention_fatigue_weight;
        let dec_factor = dec_fatigue * self.decision_fatigue_weight;

        // High cortisol: disruption that increases the need for rest
        let cortisol_factor = cortisol * self.cortisol_weight;

        // Resistance: adrenaline and conversation keep awake
        let adr_resist = adrenaline * self.adrenaline_resistance;
        let conv_resist = if in_conv { 0.1 } else { 0.0 };

        // Sleep debt increases pressure
        let debt_factor = self.sleep_debt * 0.1;

        // Final computation
        let raw = time_factor + energy_factor + attn_factor + dec_factor + cortisol_factor
            + debt_factor - adr_resist - conv_resist;
        self.sleep_pressure = raw.clamp(0.0, 1.0);
        self.sleep_forced = self.sleep_pressure > self.forced_threshold;
    }

    /// Returns true if pressure exceeds the sleep onset threshold.
    pub fn should_sleep(&self) -> bool {
        self.sleep_pressure > self.sleep_threshold || self.sleep_forced
    }

    /// Residual pressure after sleep (not a full reset).
    pub fn residual_pressure(&self) -> f64 {
        (self.sleep_pressure * 0.3).min(0.4)
    }
}

// ─── Sleep cycle ────────────────────────────────────────────────────────────

/// State of an ongoing sleep cycle.
pub struct SleepCycle {
    /// Current phase
    pub phase: SleepPhase,
    /// Remaining cycles in the current phase
    pub phase_remaining: u64,
    /// Current sleep cycle number (1-based)
    pub sleep_cycle_number: u8,
    /// Total number of planned sleep cycles
    pub total_sleep_cycles: u8,
    /// Global cycle counter during this sleep
    pub sleep_cycle_counter: u64,
    /// Was sleep interrupted?
    pub interrupted: bool,
    /// Interruption reason
    pub interruption_reason: Option<String>,
    /// Sleep quality (0-1, dynamically computed at wake-up)
    pub quality: f64,
    /// Factors collected for dynamic quality computation
    pub quality_factors: SleepQualityFactors,
    /// Number of memories consolidated during this sleep
    pub memories_consolidated: u64,
    /// Number of neural connections created
    pub connections_created: u64,
    /// Sleep start time
    pub started_at: DateTime<Utc>,
}

// ─── History ────────────────────────────────────────────────────────────────

/// Record of a completed sleep.
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

// ─── Sleep system ───────────────────────────────────────────────────────────

/// Complete sleep system: pressure, current cycle, history.
pub struct SleepSystem {
    /// Is Saphire sleeping?
    pub is_sleeping: bool,
    /// Sleep pressure model
    pub drive: SleepDrive,
    /// Current sleep cycle (None if awake)
    pub current_cycle: Option<SleepCycle>,
    /// Sleep history (max 50)
    pub sleep_history: Vec<SleepRecord>,
    /// Global statistics
    pub total_complete_sleeps: u64,
    pub total_interrupted_sleeps: u64,
    /// Total neural connections created during sleep (cumulative)
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

    /// Access to sleep configuration.
    pub fn config(&self) -> &SleepConfig {
        &self.config
    }

    /// Returns the duration in cycles of a sleep phase.
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

    /// Determines the next sleep phase.
    /// Returns None when sleep is complete (after Hypnopompic).
    pub fn next_phase(&self, cycle: &SleepCycle) -> Option<SleepPhase> {
        match cycle.phase {
            SleepPhase::Hypnagogic => Some(SleepPhase::LightSleep),
            SleepPhase::LightSleep => Some(SleepPhase::DeepSleep),
            SleepPhase::DeepSleep => Some(SleepPhase::REM),
            SleepPhase::REM => {
                if cycle.sleep_cycle_number < cycle.total_sleep_cycles {
                    // New cycle: return to light sleep
                    Some(SleepPhase::LightSleep)
                } else {
                    // Last cycle: transition to waking
                    Some(SleepPhase::Hypnopompic)
                }
            },
            SleepPhase::Hypnopompic => None, // End of sleep
            SleepPhase::Awake => None,
        }
    }

    /// Sleep progression (0-1).
    pub fn sleep_progress(&self) -> f64 {
        if let Some(ref cycle) = self.current_cycle {
            let total = self.total_estimated_cycles();
            if total == 0 { return 0.0; }
            (cycle.sleep_cycle_counter as f64 / total as f64).min(1.0)
        } else {
            0.0
        }
    }

    /// Estimates the total number of cycles for this sleep.
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

    /// Estimated remaining cycles.
    pub fn remaining_cycles(&self) -> u64 {
        if let Some(ref cycle) = self.current_cycle {
            let total = self.total_estimated_cycles();
            total.saturating_sub(cycle.sleep_cycle_counter)
        } else {
            0
        }
    }

    /// Poetic refusal message during sleep (for chat).
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

    /// Forcefully interrupts sleep.
    pub fn interrupt(&mut self, reason: String) {
        if let Some(ref mut cycle) = self.current_cycle {
            cycle.interrupted = true;
            cycle.interruption_reason = Some(reason);
        }
        self.finalize_wake_up();
    }

    /// Initiates a new sleep.
    pub fn initiate(&mut self) {
        // Compute the number of sleep cycles (1-3 based on pressure)
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

    /// Finalizes wake-up: computes dynamic quality, creates a SleepRecord, resets pressure.
    pub fn finalize_wake_up(&mut self) {
        if let Some(mut cycle) = self.current_cycle.take() {
            // Compute dynamic quality from collected factors
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
                dreams_count: 0, // updated by the caller
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

            // Reduce sleep debt proportionally to quality
            self.drive.sleep_debt = (self.drive.sleep_debt - quality * 0.3).max(0.0);

            // Residual pressure: poor night = more residual pressure
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

    /// Serializes the current state to JSON for the API/WebSocket.
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

    /// Sleep history as JSON.
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

    /// Sleep drive (pressure) JSON for the API.
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
