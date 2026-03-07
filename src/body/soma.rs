// =============================================================================
// soma.rs — System Health (soma = body in Greek)
// =============================================================================
//
// Purpose: Represents Saphire's global somatic state: energy, tension, warmth,
//          comfort, pain, and vitality. These signals are high-level abstractions
//          of what a biological body would feel, derived from the underlying
//          neurochemistry and physiological state.
//
// Architectural placement:
//   `SystemHealth` is read by the interoception module to produce body-awareness
//   signals that feed into the consciousness evaluation pipeline:
//   soma -> interoception::read_signals() -> consciousness.evaluate()
// =============================================================================

use crate::neurochemistry::NeuroChemicalState;
use super::physiology::PhysiologicalState;

/// Global somatic health — subjective body signals derived from neurochemistry and physiology.
///
/// Each field represents a subjective sensation in [0.0, 1.0] that would correspond
/// to what a biological entity "feels" internally. These are not direct physiological
/// measurements but rather experiential abstractions used for interoception and
/// consciousness evaluation.
pub struct SystemHealth {
    /// Subjective energy level in [0.0, 1.0].
    /// High when dopamine (motivation) and endorphin (resilience) are elevated;
    /// depleted by cortisol (stress-induced exhaustion). Also influenced by
    /// physiological glycemia and energy reserves.
    pub energy: f64,
    /// Muscular and psychological tension in [0.0, 1.0].
    /// High when cortisol (stress) and adrenaline (alertness) are elevated;
    /// reduced by endorphin (relaxation) and serotonin (calming).
    pub tension: f64,
    /// Internal warmth sensation in [0.0, 1.0].
    /// Driven by oxytocin (social bonding warmth) and serotonin (well-being);
    /// also modulated by physiological core body temperature.
    pub warmth: f64,
    /// Subjective comfort level in [0.0, 1.0].
    /// Inversely related to tension; boosted by serotonin and endorphin,
    /// diminished by cortisol and adrenaline.
    pub comfort: f64,
    /// Subjective pain intensity in [0.0, 1.0].
    /// Emerges when cortisol exceeds 0.6 AND endorphin is below 0.4,
    /// representing the nociceptive signal that breaks through when the
    /// endogenous analgesic system (endorphins) is insufficient to mask
    /// stress-induced tissue strain.
    pub pain: f64,
    /// Overall vitality score in [0.0, 1.0].
    /// A weighted composite of energy, comfort, warmth, absence of tension,
    /// absence of pain, and physiological overall health. Represents the
    /// general sense of "feeling alive and well."
    pub vitality: f64,
    /// Respiratory rate in breaths per minute (typical range: 8-25 RPM).
    /// Accelerates under stress (adrenaline, cortisol) and decelerates
    /// during calm states (serotonin, endorphin). Normal resting: ~12 RPM.
    pub breath_rate: f64,
}

impl Default for SystemHealth {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemHealth {
    /// Creates a default somatic state representing a balanced, pain-free baseline.
    ///
    /// Initial values: moderate energy (0.6), low tension (0.2), neutral warmth (0.5),
    /// good comfort (0.7), no pain (0.0), good vitality (0.7), normal breathing (12 RPM).
    pub fn new() -> Self {
        Self {
            energy: 0.6,
            tension: 0.2,
            warmth: 0.5,
            comfort: 0.7,
            pain: 0.0,
            vitality: 0.7,
            breath_rate: 12.0,
        }
    }

    /// Updates all somatic signals based on current neurochemistry and physiological state.
    ///
    /// Each signal converges toward a target value using exponential smoothing (10-15%
    /// convergence rate per call), preventing abrupt jumps and modeling the body's
    /// inertial response to changing conditions.
    ///
    /// # Parameters
    /// - `chemistry`: current neurochemical concentrations (dopamine, serotonin, cortisol, etc.).
    /// - `physio`: current physiological state (glycemia, temperature, energy reserves, overall health).
    pub fn update(&mut self, chemistry: &NeuroChemicalState, physio: &PhysiologicalState) {
        // Energy: dopamine (motivation) + endorphin (resilience) + noradrenaline (arousal)
        // minus cortisol (stress-induced exhaustion).
        // Physiological contribution: energy reserves (glycogen stores) and glycemia
        // (available blood glucose for immediate cellular use).
        let chem_energy = 0.3 + chemistry.dopamine * 0.35 + chemistry.endorphin * 0.2
            + chemistry.noradrenaline * 0.15 - chemistry.cortisol * 0.25;
        let physio_energy = physio.energy_reserves * 0.5 + (physio.glycemia / 5.0 - 0.5) * 0.2;
        let target_energy = (chem_energy * 0.7 + physio_energy * 0.3).clamp(0.0, 1.0);
        self.energy += (target_energy - self.energy) * 0.12;

        // Tension: cortisol (chronic stress) + adrenaline (acute alertness/fight-or-flight)
        // minus endorphin (muscular relaxation) and serotonin (calming).
        let target_tension = (chemistry.cortisol * 0.45 + chemistry.adrenaline * 0.35
            - chemistry.endorphin * 0.2 - chemistry.serotonin * 0.1).clamp(0.0, 1.0);
        self.tension += (target_tension - self.tension) * 0.12;

        // Warmth: oxytocin (social bonding warmth, associated with interpersonal closeness)
        // + serotonin (general well-being warmth) + endorphin (mild contribution).
        // Physiological contribution: actual core body temperature normalized to [0, 1].
        let chem_warmth = 0.2 + chemistry.oxytocin * 0.4 + chemistry.serotonin * 0.3
            + chemistry.endorphin * 0.1;
        let physio_warmth = ((physio.temperature - 36.0) / 3.0).clamp(0.0, 1.0);
        let target_warmth = (chem_warmth * 0.7 + physio_warmth * 0.3).clamp(0.0, 1.0);
        self.warmth += (target_warmth - self.warmth) * 0.10;

        // Comfort: inverse of tension, boosted by serotonin (contentment) and endorphin (ease).
        let target_comfort = (0.3 + chemistry.serotonin * 0.3 + chemistry.endorphin * 0.2
            - chemistry.cortisol * 0.3 - chemistry.adrenaline * 0.15).clamp(0.0, 1.0);
        self.comfort += (target_comfort - self.comfort) * 0.10;

        // Pain: emerges when cortisol is high (> 0.6) and endorphins are low (< 0.4).
        // This models the gate-control theory of pain: nociceptive signals (stress-induced
        // tissue strain from cortisol) break through when the endogenous opioid system
        // (endorphins) is insufficient to inhibit them. Pain scales with the excess cortisol
        // and the deficit in endorphin-mediated analgesia.
        let target_pain = if chemistry.cortisol > 0.6 && chemistry.endorphin < 0.4 {
            ((chemistry.cortisol - 0.6) * 2.0 * (1.0 - chemistry.endorphin)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.pain += (target_pain - self.pain) * 0.15;

        // Vitality: global well-being measure combining somatic signals and physiological health.
        // Weighted: energy (25%), comfort (20%), warmth (10%), absence of tension (10%),
        // absence of pain (10%), physiological overall health (25%).
        let base_vitality = self.energy * 0.25 + self.comfort * 0.20 + self.warmth * 0.10
            + (1.0 - self.tension) * 0.10 + (1.0 - self.pain) * 0.10;
        let physio_vitality = physio.overall_health() * 0.25;
        self.vitality = (base_vitality + physio_vitality).clamp(0.0, 1.0);

        // Respiratory rate: accelerates under sympathetic activation (adrenaline drives
        // tachypnea for oxygen mobilization; cortisol sustains elevated ventilation).
        // Decelerates with parasympathetic dominance (serotonin and endorphin promote
        // slow, deep breathing). Normal resting: ~12 RPM. Range: [8, 25] RPM.
        let target_breath = (12.0 + chemistry.adrenaline * 8.0 + chemistry.cortisol * 5.0
            - chemistry.serotonin * 3.0 - chemistry.endorphin * 2.0).clamp(8.0, 25.0);
        self.breath_rate += (target_breath - self.breath_rate) * 0.1;
    }
}
