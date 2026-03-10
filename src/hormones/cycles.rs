// =============================================================================
// hormones/cycles.rs — Cycles hormonaux (circadien, ultradian)
//
// Role : Simule les rythmes biologiques qui pilotent les niveaux hormonaux.
//   - Circadien (~24h simule) : melatonine (pic nuit), cortisol (pic matin)
//   - Ultradian (~90 cycles) : fluctuations de testosterone
//
// La vitesse du cycle circadien est configurable (circadian_cycle_real_seconds).
// Par defaut, 1h reelle = 1 "jour" simule complet.
// =============================================================================

use crate::config::HormonesConfig;
use super::HormonalState;

/// Avance la phase circadienne et ajuste les niveaux hormonaux.
///
/// Phase : 0.0 = minuit, 0.25 = 6h, 0.5 = midi, 0.75 = 18h, 1.0 = minuit.
///
/// - Melatonine : pic vers 0.0 (minuit), creux vers 0.5 (midi)
///   Courbe : 0.7 * (1 + cos(2*PI*phase)) / 2 + 0.05
/// - Cortisol_h : pic vers 0.25 (6h matin), creux vers 0.75 (18h soir)
///   Courbe : 0.5 * (1 + cos(2*PI*(phase - 0.25))) / 2 + 0.15
pub fn tick_circadian(state: &mut HormonalState, phase: &mut f64, config: &HormonesConfig) {
    // Avancer la phase en fonction de thought_interval implicite (~15s par cycle)
    // circadian_cycle_real_seconds = combien de secondes reelles pour 1 jour simule
    let phase_increment = 15.0 / config.circadian_cycle_real_seconds as f64;
    *phase = (*phase + phase_increment) % 1.0;

    let two_pi = std::f64::consts::TAU;

    // Melatonine : pic a minuit (phase=0.0), creux a midi (phase=0.5)
    let melatonin_target = 0.7 * (1.0 + (two_pi * *phase).cos()) / 2.0 + 0.05;
    // Convergence douce vers la cible
    state.melatonin += (melatonin_target - state.melatonin) * 0.05;

    // Cortisol hormonal : pic a 6h (phase=0.25), creux a 18h (phase=0.75)
    let cortisol_target = 0.5 * (1.0 + (two_pi * (*phase - 0.25)).cos()) / 2.0 + 0.15;
    state.cortisol_h += (cortisol_target - state.cortisol_h) * 0.05;

    // Insuline : legere montee post-prandiale (3 repas simules)
    // Pics vers 0.33 (8h), 0.54 (13h), 0.79 (19h)
    let insulin_base = 0.50;
    let meal_effect = 0.15 * (
        gauss(*phase, 0.33, 0.03) +
        gauss(*phase, 0.54, 0.03) +
        gauss(*phase, 0.79, 0.03)
    );
    let insulin_target = (insulin_base + meal_effect).min(1.0);
    state.insulin += (insulin_target - state.insulin) * 0.03;
}

/// Cycles ultradiens : fluctuations de testosterone (~90 cycles).
///
/// La testosterone oscille autour de son niveau courant avec une amplitude
/// de ~0.08 et une periode de 90 cycles. Convergence douce (3% par cycle).
pub fn tick_ultradian(state: &mut HormonalState, cycle: u64) {
    let two_pi = std::f64::consts::TAU;
    // Phase dans le cycle de 90 steps
    let phase_90 = (cycle as f64 % 90.0) / 90.0;
    // Cible oscillante autour de 0.5 (baseline)
    let target = 0.50 + 0.08 * (two_pi * phase_90).sin();
    let target_clamped = target.clamp(0.2, 0.8);
    // Convergence douce (3% par cycle)
    state.testosterone += (target_clamped - state.testosterone) * 0.03;
}

/// Fonction gaussienne pour simuler les pics post-prandiaux.
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
        let mut phase = 0.0; // minuit
        // Simuler quelques cycles a minuit
        for _ in 0..50 {
            tick_circadian(&mut state, &mut phase, &config);
        }
        // La melatonine devrait etre elevee pres de minuit
        // (phase aura avance un peu mais reste proche de 0)
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
        // Simuler 90 cycles
        for c in 0..90 {
            tick_ultradian(&mut state, c);
        }
        // La testosterone devrait avoir change
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
