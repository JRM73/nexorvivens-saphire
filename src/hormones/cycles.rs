// =============================================================================
// hormones/cycles.rs — Hormonal cycles (circadian, ultradian)
//
// Purpose: Simulates biological rhythms that drive hormonal levels.
//   - Circadian (~simulated 24h): melatonin (night peak), cortisol (morning peak)
//   - Ultradian (~90 cycles): testosterone fluctuations
//
// The circadian cycle speed is configurable (circadian_cycle_real_seconds).
// By default, 1 real hour = 1 complete simulated "day".
// =============================================================================

use crate::config::HormonesConfig;
use super::HormonalState;

/// Advances the circadian phase and adjusts hormonal levels.
///
/// Phase: 0.0 = midnight, 0.25 = 6am, 0.5 = noon, 0.75 = 6pm, 1.0 = midnight.
///
/// - Melatonin: peak around 0.0 (midnight), trough around 0.5 (noon)
///   Curve: 0.7 * (1 + cos(2*PI*phase)) / 2 + 0.05
/// - Cortisol_h: peak around 0.25 (6am), trough around 0.75 (6pm)
///   Curve: 0.5 * (1 + cos(2*PI*(phase - 0.25))) / 2 + 0.15
pub fn tick_circadian(state: &mut HormonalState, phase: &mut f64, config: &HormonesConfig) {
    // Advance the phase based on implicit thought_interval (~15s per cycle)
    // circadian_cycle_real_seconds = how many real seconds for 1 simulated day
    let phase_increment = 15.0 / config.circadian_cycle_real_seconds as f64;
    *phase = (*phase + phase_increment) % 1.0;

    let two_pi = std::f64::consts::TAU;

    // Melatonin: peak at midnight (phase=0.0), trough at noon (phase=0.5)
    let melatonin_target = 0.7 * (1.0 + (two_pi * *phase).cos()) / 2.0 + 0.05;
    // Smooth convergence toward the target
    state.melatonin += (melatonin_target - state.melatonin) * 0.05;

    // Hormonal cortisol: peak at 6am (phase=0.25), trough at 6pm (phase=0.75)
    let cortisol_target = 0.5 * (1.0 + (two_pi * (*phase - 0.25)).cos()) / 2.0 + 0.15;
    state.cortisol_h += (cortisol_target - state.cortisol_h) * 0.05;

    // Insulin: slight postprandial rise (3 simulated meals)
    // Peaks around 0.33 (8am), 0.54 (1pm), 0.79 (7pm)
    let insulin_base = 0.50;
    let meal_effect = 0.15 * (
        gauss(*phase, 0.33, 0.03) +
        gauss(*phase, 0.54, 0.03) +
        gauss(*phase, 0.79, 0.03)
    );
    let insulin_target = (insulin_base + meal_effect).min(1.0);
    state.insulin += (insulin_target - state.insulin) * 0.03;
}

/// Ultradian cycles: testosterone fluctuations (~90 cycles).
///
/// Testosterone oscillates around its current level with an amplitude
/// of ~0.08 and a period of 90 cycles. Smooth convergence (3% per cycle).
pub fn tick_ultradian(state: &mut HormonalState, cycle: u64) {
    let two_pi = std::f64::consts::TAU;
    // Phase within the 90-step cycle
    let phase_90 = (cycle as f64 % 90.0) / 90.0;
    // Oscillating target around 0.5 (baseline)
    let target = 0.50 + 0.08 * (two_pi * phase_90).sin();
    let target_clamped = target.clamp(0.2, 0.8);
    // Smooth convergence (3% per cycle)
    state.testosterone += (target_clamped - state.testosterone) * 0.03;
}

/// Gaussian function to simulate postprandial peaks.
fn gauss(x: f64, mean: f64, sigma: f64) -> f64 {
    let diff = x - mean;
    (-diff * diff / (2.0 * sigma * sigma)).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circadian_melatonin_high_at_midnight() {
        let config = HormonesConfig::default();
        let mut state = HormonalState::default();
        let mut phase = 0.0; // midnight
        // Simulate a few cycles at midnight
        for _ in 0..50 {
            tick_circadian(&mut state, &mut phase, &config);
        }
        // Melatonin should be elevated near midnight
        // (phase will have advanced slightly but remains close to 0)
        assert!(state.melatonin > 0.3, "Melatonin should be elevated near midnight: {}", state.melatonin);
    }

    #[test]
    fn test_circadian_phase_wraps() {
        let config = HormonesConfig { circadian_cycle_real_seconds: 150, ..HormonesConfig::default() };
        let mut state = HormonalState::default();
        let mut phase = 0.99;
        tick_circadian(&mut state, &mut phase, &config);
        assert!(phase < 1.0, "Phase should wrap around: {}", phase);
    }

    #[test]
    fn test_ultradian_oscillation() {
        let mut state = HormonalState::default();
        let initial = state.testosterone;
        // Simulate 90 cycles
        for c in 0..90 {
            tick_ultradian(&mut state, c);
        }
        // Testosterone should have changed
        let diff = (state.testosterone - initial).abs();
        assert!(diff > 0.0, "Testosterone should oscillate, diff={}", diff);
        assert!(state.testosterone >= 0.2 && state.testosterone <= 0.8);
    }

    #[test]
    fn test_gauss_peak() {
        let peak = super::gauss(0.33, 0.33, 0.03);
        assert!((peak - 1.0).abs() < 0.01, "Gauss peak should be ~1.0: {}", peak);
        let off = super::gauss(0.0, 0.33, 0.03);
        assert!(off < 0.01, "Gauss off-peak should be ~0: {}", off);
    }
}
