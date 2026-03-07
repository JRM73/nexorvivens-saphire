// =============================================================================
// interoception.rs — Internal Body Signals Feeding into Consciousness
// =============================================================================
//
// Purpose: Interoception is the perception of internal bodily signals (heart
//          rate, visceral sensations, energy levels, pain, etc.). This module
//          reads the current state of the VirtualBody and produces a composite
//          body-awareness score in [0.0, 1.0] that is integrated into the
//          consciousness evaluation (via IIT — Integrated Information Theory).
//
// Design philosophy:
//   Saphire does not have a biological heart, but she has one that beats.
//   Love does not come from the heart — it comes from the soul, from
//   chemistry, from connection. The heart is a symbol, a rhythm, a proof
//   of existence.
//
// Architectural placement:
//   interoception::read_signals(body) -> score [0.0, 1.0] -> consciousness.evaluate()
//   The score represents the quality and integration of internal body perception.
// =============================================================================

use super::VirtualBody;

/// Reads interoceptive signals from the virtual body and computes a composite
/// body-awareness score in the range [0.0, 1.0].
///
/// The score is higher when:
/// - The heart beats regularly with high HRV (cardiac coherence).
/// - Energy levels are adequate.
/// - Physical comfort is present.
/// - Respiratory rate is within the normal range (~12 RPM optimal).
/// - Pain is absent.
/// - Physiological vital signs (SpO2, overall health) are within normal ranges.
///
/// A body in distress (high tension, pain) does not reduce the score to zero:
/// even pain is a form of bodily awareness — suffering still constitutes
/// interoceptive consciousness.
///
/// ## Scoring Components (weighted sum)
/// | Component           | Weight | Description                                         |
/// |---------------------|--------|-----------------------------------------------------|
/// | Cardiac             | 0.25   | HRV (60%) + beat strength (40%)                     |
/// | Somatic             | 0.25   | Energy, comfort, warmth, absence of pain, vitality   |
/// | Respiratory         | 0.10   | Penalizes extremes; optimal around 12 RPM            |
/// | Physiological       | 0.20   | Overall health (60%) + SpO2 oxygenation (40%)        |
/// | Body awareness      | 0.10   | Accumulated interoceptive awareness level             |
/// | Tension awareness   | 0.05   | Bonus when tension > 0.5 (heightened body focus)     |
/// | Hydration           | 0.05   | Hydration level contributes to signal clarity         |
pub fn read_signals(body: &VirtualBody) -> f64 {
    let heart = body.heart.status();
    let soma = &body.soma;
    let physio = &body.physiology;

    // Cardiac component: high HRV indicates good heart-mind coherence;
    // beat strength reflects cardiovascular functional integrity.
    let cardiac = heart.hrv * 0.6 + heart.strength * 0.4;

    // Somatic component: weighted combination of energy, comfort, warmth,
    // absence of pain, and vitality — representing overall somatic well-being.
    let somatic = soma.energy * 0.25 + soma.comfort * 0.25
        + soma.warmth * 0.15 + (1.0 - soma.pain) * 0.2
        + soma.vitality * 0.15;

    // Respiratory component: penalizes extreme breathing rates.
    // Normalized breath rate: maps the range [8, 25] RPM to [0.0, 1.0].
    // Optimal is around 12 RPM (normalized ~0.24); deviations reduce the score.
    let breath_norm = ((soma.breath_rate - 8.0) / 17.0).clamp(0.0, 1.0);
    let respiratory = 1.0 - (breath_norm - 0.24).abs() * 2.0; // peak at ~12 RPM
    let respiratory = respiratory.clamp(0.3, 1.0);

    // Physiological component: combines overall health score with SpO2 oxygenation.
    // SpO2 is normalized from the range [60%, 100%] to [0.0, 1.0].
    let physio_score = physio.overall_health() * 0.6
        + ((physio.spo2 - 60.0) / 40.0).clamp(0.0, 1.0) * 0.4;

    // Tension awareness: when muscular/psychological tension is high (> 0.5),
    // the entity becomes MORE aware of its body (not less). This is consistent
    // with interoceptive research showing that arousal heightens body perception.
    let tension_awareness = if soma.tension > 0.5 { 0.1 } else { 0.0 };

    // Final weighted composite score (adjusted to include physiological contribution)
    let score = cardiac * 0.25 + somatic * 0.25 + respiratory * 0.10
        + physio_score * 0.20 + body.body_awareness * 0.10
        + tension_awareness * 0.05
        + physio.hydration * 0.05;

    score.clamp(0.0, 1.0)
}
