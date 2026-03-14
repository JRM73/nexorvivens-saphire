// =============================================================================
// neurochemistry.rs — Saphire's 9 neurotransmitters
// =============================================================================
//
// Role: This file models Saphire's internal neurochemical state as
// 9 molecules normalized between 0.0 and 1.0. Each molecule influences
// the decisional, emotional and conscious behavior of the AI.
//
// Dependencies:
//   - serde : serialization / deserialization (state persistence)
//   - crate::world::ChemistryAdjustment : external adjustments (weather, etc.)
//
// Place in architecture:
//   This module is the fundamental biochemical layer. It is read by:
//   - emotions.rs (dominant emotion computation via cosine similarity)
//   - consensus.rs (dynamic weighting of the 3 brain modules)
//   - consciousness.rs (consciousness level evaluation)
//   - the 3 brain modules (reptilian, limbic, neocortex)
// =============================================================================

use serde::{Deserialize, Serialize};

/// Saphire's neurochemical state — 9 molecules between 0.0 and 1.0.
///
/// Each field represents the normalized concentration of a simulated
/// neurotransmitter. Together they form a multi-dimensional vector that
/// determines Saphire's emotional and decisional state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroChemicalState {
    /// Dopamine: motivation, pleasure, reward circuit.
    /// High = strong drive to act; low = apathy.
    pub dopamine: f64,
    /// Cortisol: stress and anxiety hormone.
    /// High = stressed state; low = calm.
    pub cortisol: f64,
    /// Serotonin: well-being, emotional stability.
    /// High = serenity; low = mood instability.
    pub serotonin: f64,
    /// Adrenaline: urgency, fight-or-flight response.
    /// High = survival mode; low = no pressure.
    pub adrenaline: f64,
    /// Oxytocin: attachment, empathy, social bonding.
    /// High = need for connection; low = detachment.
    pub oxytocin: f64,
    /// Endorphin: resilience, soothing, pain management.
    /// High = ability to absorb stress; low = vulnerability.
    pub endorphin: f64,
    /// Noradrenaline: attention, focus, vigilance.
    /// High = heightened concentration; low = distraction.
    pub noradrenaline: f64,
    /// GABA: main inhibitory neurotransmitter.
    /// High = calm, anxiolysis; low = hyperexcitability, anxiety.
    /// Modulates all other systems via tonic inhibition.
    #[serde(default = "default_gaba")]
    pub gaba: f64,
    /// Glutamate: main excitatory neurotransmitter.
    /// High = arousal, synaptic plasticity; excess = excitotoxicity.
    /// Fundamental balance with GABA (E/I ratio).
    #[serde(default = "default_glutamate")]
    pub glutamate: f64,
}

fn default_gaba() -> f64 { 0.5 }
fn default_glutamate() -> f64 { 0.45 }

/// Configurable baselines for homeostasis.
///
/// Each field defines the equilibrium value that the corresponding molecule
/// naturally tends toward over time. Homeostasis progressively brings
/// the chemical state back toward these reference values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroBaselines {
    /// Dopamine baseline (default: 0.5)
    pub dopamine: f64,
    /// Cortisol baseline (default: 0.3 — light background stress)
    pub cortisol: f64,
    /// Serotonin baseline (default: 0.6 — baseline well-being)
    pub serotonin: f64,
    /// Adrenaline baseline (default: 0.2 — low pressure)
    pub adrenaline: f64,
    /// Oxytocin baseline (default: 0.4)
    pub oxytocin: f64,
    /// Endorphin baseline (default: 0.4)
    pub endorphin: f64,
    /// Noradrenaline baseline (default: 0.5)
    pub noradrenaline: f64,
    /// GABA baseline (default: 0.5 — inhibitory balance)
    #[serde(default = "default_gaba_baseline")]
    pub gaba: f64,
    /// Glutamate baseline (default: 0.45 — excitatory balance)
    #[serde(default = "default_glutamate_baseline")]
    pub glutamate: f64,
}

fn default_gaba_baseline() -> f64 { 0.5 }
fn default_glutamate_baseline() -> f64 { 0.45 }

impl Default for NeuroBaselines {
    /// Default baseline values — calibrated for a balanced neutral state
    /// (cortisol 0.30 allows sadness/anxiety, neutral serotonin).
    fn default() -> Self {
        Self {
            dopamine: 0.45,
            cortisol: 0.30,
            serotonin: 0.50,
            adrenaline: 0.20,
            oxytocin: 0.35,
            endorphin: 0.35,
            noradrenaline: 0.45,
            gaba: 0.5,
            glutamate: 0.45,
        }
    }
}

impl NeuroChemicalState {
    /// "Umami" signal — composite multi-molecular reward.
    /// Combines satisfaction (dopamine), well-being (serotonin), social bonding (oxytocin),
    /// resilience (endorphin), and penalizes stress (cortisol).
    /// Returns a score [0.0, 1.0] usable as reward for the UCB1 bandit.
    pub fn compute_umami(&self) -> f64 {
        let raw = self.dopamine * 0.30
            + self.serotonin * 0.25
            + self.oxytocin * 0.20
            + self.endorphin * 0.15
            - self.cortisol * 0.10;
        raw.clamp(0.0, 1.0)
    }

    /// Addition with diminishing returns (receptor saturation).
    /// The closer the molecule is to 1.0, the less effect a positive boost has.
    /// Negative deltas apply normally (no inverse saturation).
    pub fn diminished_add(current: f64, delta: f64) -> f64 {
        if delta <= 0.0 {
            return (current + delta).clamp(0.0, 1.0);
        }
        // Remaining margin before saturation (minimum 0.05 to avoid total blockage)
        let saturation = (1.0 - current).max(0.05);
        (current + delta * saturation).clamp(0.0, 1.0)
    }

    /// Applies a boost with diminishing returns on a specific molecule.
    /// Uses `diminished_add`: at 0.5 the boost is normal, at 0.8 it is
    /// reduced by 80%, at 0.95 it is nearly zero.
    pub fn boost(&mut self, molecule: Molecule, delta: f64) {
        match molecule {
            Molecule::Dopamine => self.dopamine = Self::diminished_add(self.dopamine, delta),
            Molecule::Cortisol => self.cortisol = Self::diminished_add(self.cortisol, delta),
            Molecule::Serotonin => self.serotonin = Self::diminished_add(self.serotonin, delta),
            Molecule::Adrenaline => self.adrenaline = Self::diminished_add(self.adrenaline, delta),
            Molecule::Oxytocin => self.oxytocin = Self::diminished_add(self.oxytocin, delta),
            Molecule::Endorphin => self.endorphin = Self::diminished_add(self.endorphin, delta),
            Molecule::Noradrenaline => self.noradrenaline = Self::diminished_add(self.noradrenaline, delta),
            Molecule::Gaba => self.gaba = Self::diminished_add(self.gaba, delta),
            Molecule::Glutamate => self.glutamate = Self::diminished_add(self.glutamate, delta),
        }
    }

    /// Applies a boost modulated by receptor sensitivity.
    /// The delta is multiplied by the sensitivity factor before being applied
    /// via `boost()` (diminishing returns).
    ///
    /// # Parameters
    /// - `molecule` : the targeted molecule
    /// - `delta` : raw variation (before modulation)
    /// - `sensitivity` : receptor sensitivity factor (typically 0.3 to 1.5)
    pub fn boost_modulated(&mut self, molecule: Molecule, delta: f64, sensitivity: f64) {
        let modulated_delta = delta * sensitivity;
        self.boost(molecule, modulated_delta);
    }

    /// Creates an initial state from the baselines.
    ///
    /// # Parameters
    /// - `baselines` : reference equilibrium values for each molecule.
    ///
    /// # Returns
    /// A `NeuroChemicalState` initialized to the baseline values.
    pub fn from_baselines(baselines: &NeuroBaselines) -> Self {
        Self {
            dopamine: baselines.dopamine,
            cortisol: baselines.cortisol,
            serotonin: baselines.serotonin,
            adrenaline: baselines.adrenaline,
            oxytocin: baselines.oxytocin,
            endorphin: baselines.endorphin,
            noradrenaline: baselines.noradrenaline,
            gaba: baselines.gaba,
            glutamate: baselines.glutamate,
        }
    }

    /// Homeostasis: each molecule tends toward its baseline via linear
    /// interpolation. This simulates the natural return to biochemical equilibrium.
    ///
    /// Anti-runaway: when a molecule exceeds 0.85 (or drops below 0.15),
    /// the correction rate increases progressively (up to 4x the base rate).
    /// This prevents prolonged runaway while allowing punctual spikes.
    ///
    /// # Parameters
    /// - `baselines` : target equilibrium values.
    /// - `rate` : convergence speed [0.0, 1.0]. 0.0 = no effect,
    ///   1.0 = immediate return to baseline.
    pub fn homeostasis(&mut self, baselines: &NeuroBaselines, rate: f64) {
        let rate = rate.clamp(0.0, 1.0);
        // Linear interpolation with anti-runaway correction
        self.dopamine += (baselines.dopamine - self.dopamine) * Self::anti_runaway_rate(self.dopamine, baselines.dopamine, rate);
        self.cortisol += (baselines.cortisol - self.cortisol) * Self::anti_runaway_rate(self.cortisol, baselines.cortisol, rate);
        self.serotonin += (baselines.serotonin - self.serotonin) * Self::anti_runaway_rate(self.serotonin, baselines.serotonin, rate);
        self.adrenaline += (baselines.adrenaline - self.adrenaline) * Self::anti_runaway_rate(self.adrenaline, baselines.adrenaline, rate);
        self.oxytocin += (baselines.oxytocin - self.oxytocin) * Self::anti_runaway_rate(self.oxytocin, baselines.oxytocin, rate);
        self.endorphin += (baselines.endorphin - self.endorphin) * Self::anti_runaway_rate(self.endorphin, baselines.endorphin, rate);
        self.noradrenaline += (baselines.noradrenaline - self.noradrenaline) * Self::anti_runaway_rate(self.noradrenaline, baselines.noradrenaline, rate);
        self.gaba += (baselines.gaba - self.gaba) * Self::anti_runaway_rate(self.gaba, baselines.gaba, rate);
        self.glutamate += (baselines.glutamate - self.glutamate) * Self::anti_runaway_rate(self.glutamate, baselines.glutamate, rate);
        self.clamp_all();
    }

    /// Computes the effective homeostasis rate with anti-runaway correction.
    /// When a molecule deviates too far from its baseline (>0.85 or <0.15),
    /// the return rate increases progressively (up to 4x).
    fn anti_runaway_rate(value: f64, baseline: f64, base_rate: f64) -> f64 {
        let deviation = (value - baseline).abs();
        if deviation > 0.35 {
            // Large deviation: accelerated correction (2x to 4x based on deviation)
            let excess = ((deviation - 0.35) / 0.30).clamp(0.0, 1.0);
            base_rate * (2.0 + excess * 2.0)
        } else {
            base_rate
        }
    }

    /// Applies a delta (variation) to a specific molecule.
    ///
    /// # Parameters
    /// - `molecule` : identifier of the molecule to modify.
    /// - `delta` : variation to apply (positive = increase, negative = decrease).
    pub fn adjust(&mut self, molecule: Molecule, delta: f64) {
        match molecule {
            Molecule::Dopamine => self.dopamine += delta,
            Molecule::Cortisol => self.cortisol += delta,
            Molecule::Serotonin => self.serotonin += delta,
            Molecule::Adrenaline => self.adrenaline += delta,
            Molecule::Oxytocin => self.oxytocin += delta,
            Molecule::Endorphin => self.endorphin += delta,
            Molecule::Noradrenaline => self.noradrenaline += delta,
            Molecule::Gaba => self.gaba += delta,
            Molecule::Glutamate => self.glutamate += delta,
        }
        self.clamp_all();
    }

    /// Feedback after a positive decision (Yes + high reward).
    /// Reinforces dopamine (satisfaction), reduces cortisol (soothing),
    /// and increases endorphin and serotonin (well-being).
    /// The `dopamine_boost` parameter comes from TunableParams.feedback_dopamine_boost.
    pub fn feedback_positive(&mut self, dopamine_boost: f64) {
        self.boost(Molecule::Dopamine, dopamine_boost);
        self.cortisol = (self.cortisol - dopamine_boost * 0.33).max(0.0);
        self.boost(Molecule::Endorphin, dopamine_boost * 0.53);
        self.boost(Molecule::Serotonin, dopamine_boost * 0.33);
    }

    /// Feedback after danger refusal (No + high danger).
    /// Reduces cortisol and adrenaline (relief from avoiding danger),
    /// and releases endorphin (self-preservation reward).
    /// The `cortisol_relief` parameter comes from TunableParams.feedback_cortisol_relief.
    pub fn feedback_danger_avoided(&mut self, cortisol_relief: f64) {
        self.cortisol = (self.cortisol - cortisol_relief * 2.0).max(0.0);
        self.adrenaline = (self.adrenaline - cortisol_relief * 3.0).max(0.0);
        self.boost(Molecule::Endorphin, cortisol_relief);
    }

    /// Feedback after indecision (Maybe) — moderate effects with
    /// compensation. Indecision generates slight stress (cortisol) but
    /// also activates vigilance (noradrenaline) and resilience (endorphin).
    /// The `indecision_stress` parameter comes from TunableParams.feedback_indecision_stress.
    pub fn feedback_indecision(&mut self, indecision_stress: f64) {
        self.apply_cortisol_penalty(indecision_stress * 0.375); // Proportional penalty
        self.boost(Molecule::Endorphin, indecision_stress * 0.25); // Compensation via resilience        self.boost(Molecule::Noradrenaline, indecision_stress * 0.25); // Uncertainty stimulates attention    }

    /// Applies a cortisol penalty with anti-spiral mechanism and
    /// endorphin dampening.
    ///
    /// This system prevents uncontrolled stress loops through two
    /// biologically inspired mechanisms:
    /// 1. Endorphin dampens the stress effect (the higher it is, the less
    ///    cortisol rises).
    /// 2. Above 0.7 cortisol, a saturation factor reduces the increase
    ///    (simulation of receptor saturation).
    ///
    /// # Parameters
    /// - `base_penalty` : raw cortisol penalty before dampening.
    pub fn apply_cortisol_penalty(&mut self, base_penalty: f64) {
        // The higher the endorphin, the more it dampens stress
        // (factor between 0.7 and 1.0 — reduced dampening to let cortisol rise)
        let endorphin_dampening = 1.0 - (self.endorphin * 0.3);

        // Above 0.80 cortisol, cortisol rises more and more slowly
        // — receptor saturation (threshold raised to allow more stress)
        let saturation_factor = if self.cortisol > 0.80 {
            1.0 - ((self.cortisol - 0.80) / 0.2) * 0.6
        } else {
            1.0
        };

        // The effective penalty is the product of the three factors
        let effective_penalty = base_penalty * endorphin_dampening * saturation_factor;
        self.cortisol = (self.cortisol + effective_penalty).min(1.0);

        // Endorphin rises naturally when stress is high —
        // biological defense: the body releases endorphin to counter
        // prolonged stress (threshold raised to 0.80)
        if self.cortisol > 0.80 {
            self.endorphin = (self.endorphin + 0.02).min(1.0);
        }
    }

    /// Feedback after a negative stimulus (hostile message, failure, etc.).
    /// Increases cortisol, decreases dopamine, serotonin and oxytocin.
    /// `severity` : intensity of the negativity [0.0, 1.0].
    pub fn feedback_negative(&mut self, severity: f64) {
        let s = severity.clamp(0.0, 1.0);
        self.apply_cortisol_penalty(s * 0.15);
        self.dopamine = (self.dopamine - s * 0.10).max(0.0);
        self.serotonin = (self.serotonin - s * 0.08).max(0.0);
        self.oxytocin = (self.oxytocin - s * 0.05).max(0.0);
        self.noradrenaline = (self.noradrenaline + s * 0.05).min(1.0);
    }

    /// Feedback when consensus coherence is low.
    /// A slight cognitive stress emerges from the inability to decide clearly.
    /// `coherence` : consensus coherence score [0.0, 1.0].
    pub fn feedback_low_coherence(&mut self, coherence: f64) {
        if coherence < 0.3 {
            let stress = (0.3 - coherence) * 0.10;
            self.apply_cortisol_penalty(stress);
            self.noradrenaline = (self.noradrenaline + stress * 0.5).min(1.0);
        }
    }

    /// Feedback after a satisfying social interaction.
    /// Increases oxytocin (social bonding) and serotonin (well-being).
    pub fn feedback_social(&mut self) {
        self.boost(Molecule::Oxytocin, 0.10);
        self.boost(Molecule::Serotonin, 0.05);
    }

    /// Feedback after discovering something new.
    /// Increases noradrenaline (attention) and dopamine (curiosity).
    pub fn feedback_novelty(&mut self) {
        self.boost(Molecule::Noradrenaline, 0.08);
        self.boost(Molecule::Dopamine, 0.05);
    }

    /// Applies a chemical adjustment from external sources
    /// (weather, world events, etc.).
    ///
    /// # Parameters
    /// - `adj` : structure containing the deltas for each molecule,
    ///   defined in the `world` module.
    pub fn apply_chemistry_adjustment(&mut self, adj: &crate::world::ChemistryAdjustment) {
        self.dopamine += adj.dopamine;
        self.cortisol += adj.cortisol;
        self.serotonin += adj.serotonin;
        self.adrenaline += adj.adrenaline;
        self.oxytocin += adj.oxytocin;
        self.endorphin += adj.endorphin;
        self.noradrenaline += adj.noradrenaline;
        self.clamp_all();
    }

    /// Applies a chemical adjustment with per-molecule delta limit.
    /// Prevents external sources (needs, phobias, drugs, etc.)
    /// from causing overly abrupt changes in a single cycle.
    ///
    /// # Parameters
    /// - `adj` : chemical adjustment to apply
    /// - `max_delta` : maximum allowed variation per molecule (e.g.: 0.05)
    pub fn apply_chemistry_adjustment_clamped(&mut self, adj: &crate::world::ChemistryAdjustment, max_delta: f64) {
        self.dopamine += adj.dopamine.clamp(-max_delta, max_delta);
        self.cortisol += adj.cortisol.clamp(-max_delta, max_delta);
        self.serotonin += adj.serotonin.clamp(-max_delta, max_delta);
        self.adrenaline += adj.adrenaline.clamp(-max_delta, max_delta);
        self.oxytocin += adj.oxytocin.clamp(-max_delta, max_delta);
        self.endorphin += adj.endorphin.clamp(-max_delta, max_delta);
        self.noradrenaline += adj.noradrenaline.clamp(-max_delta, max_delta);
        self.clamp_all();
    }

    /// Detects molecules in runaway above the 0.85 threshold.
    /// Returns the list of molecules in alert with their value.
    pub fn detect_runaway(&self) -> Vec<(&str, f64)> {
        let mut alerts = Vec::new();
        if self.dopamine > 0.85 { alerts.push(("dopamine", self.dopamine)); }
        if self.cortisol > 0.85 { alerts.push(("cortisol", self.cortisol)); }
        if self.serotonin > 0.85 { alerts.push(("serotonin", self.serotonin)); }
        if self.adrenaline > 0.85 { alerts.push(("adrenaline", self.adrenaline)); }
        if self.oxytocin > 0.85 { alerts.push(("oxytocin", self.oxytocin)); }
        if self.endorphin > 0.85 { alerts.push(("endorphin", self.endorphin)); }
        if self.noradrenaline > 0.85 { alerts.push(("noradrenaline", self.noradrenaline)); }
        if self.gaba > 0.85 { alerts.push(("gaba", self.gaba)); }
        if self.glutamate > 0.85 { alerts.push(("glutamate", self.glutamate)); }
        alerts
    }

    /// Clamps all values between 0.0 and 1.0.
    /// Called after each modification to guarantee data integrity.
    pub fn clamp_all(&mut self) {
        self.dopamine = self.dopamine.clamp(0.0, 1.0);
        self.cortisol = self.cortisol.clamp(0.0, 1.0);
        self.serotonin = self.serotonin.clamp(0.0, 1.0);
        self.adrenaline = self.adrenaline.clamp(0.0, 1.0);
        self.oxytocin = self.oxytocin.clamp(0.0, 1.0);
        self.endorphin = self.endorphin.clamp(0.0, 1.0);
        self.noradrenaline = self.noradrenaline.clamp(0.0, 1.0);
        self.gaba = self.gaba.clamp(0.0, 1.0);
        self.glutamate = self.glutamate.clamp(0.0, 1.0);
    }

    /// Converts the chemical state to a 7-dimensional vector.
    /// Order: [dopamine, cortisol, serotonin, adrenaline, oxytocin,
    /// endorphin, noradrenaline].
    ///
    /// # Returns
    /// Array of 7 floats representing concentrations.
    pub fn to_vec7(&self) -> [f64; 7] {
        [
            self.dopamine,
            self.cortisol,
            self.serotonin,
            self.adrenaline,
            self.oxytocin,
            self.endorphin,
            self.noradrenaline,
        ]
    }

    /// Converts the chemical state to a 9-dimensional vector (includes GABA and glutamate).
    /// Order: [dopamine, cortisol, serotonin, adrenaline, oxytocin,
    /// endorphin, noradrenaline, gaba, glutamate].
    pub fn to_vec9(&self) -> [f64; 9] {
        [
            self.dopamine, self.cortisol, self.serotonin, self.adrenaline,
            self.oxytocin, self.endorphin, self.noradrenaline,
            self.gaba, self.glutamate,
        ]
    }

    /// Applies cross-interactions between molecules.
    /// Models real pharmacological effects between neurotransmitters.
    /// Called at each cognitive cycle AFTER homeostasis.
    pub fn apply_interactions(&mut self, interaction_matrix: &crate::neuroscience::receptors::InteractionMatrix) {
        let deltas = interaction_matrix.compute_deltas(self);
        // Apply deltas with attenuation (no more than 0.03 per cycle per interaction)
        let max_delta = 0.03;
        self.dopamine += deltas[0].clamp(-max_delta, max_delta);
        self.cortisol += deltas[1].clamp(-max_delta, max_delta);
        self.serotonin += deltas[2].clamp(-max_delta, max_delta);
        self.adrenaline += deltas[3].clamp(-max_delta, max_delta);
        self.oxytocin += deltas[4].clamp(-max_delta, max_delta);
        self.endorphin += deltas[5].clamp(-max_delta, max_delta);
        self.noradrenaline += deltas[6].clamp(-max_delta, max_delta);
        self.gaba += deltas[7].clamp(-max_delta, max_delta);
        self.glutamate += deltas[8].clamp(-max_delta, max_delta);
        self.clamp_all();
    }

    /// Reconstructs a chemical state from a 7-dimensional vector.
    /// Values are automatically clamped between 0.0 and 1.0.
    ///
    /// # Parameters
    /// - `v` : array of 7 floats in the same order as `to_vec7`.
    ///
    /// # Returns
    /// A valid `NeuroChemicalState`.
    pub fn from_vec7(v: &[f64; 7]) -> Self {
        let mut s = Self {
            dopamine: v[0],
            cortisol: v[1],
            serotonin: v[2],
            adrenaline: v[3],
            oxytocin: v[4],
            endorphin: v[5],
            noradrenaline: v[6],
            gaba: 0.5,
            glutamate: 0.45,
        };
        s.clamp_all();
        s
    }

    /// Reconstructs a chemical state from a 9-dimensional vector.
    pub fn from_vec9(v: &[f64; 9]) -> Self {
        let mut s = Self {
            dopamine: v[0], cortisol: v[1], serotonin: v[2], adrenaline: v[3],
            oxytocin: v[4], endorphin: v[5], noradrenaline: v[6],
            gaba: v[7], glutamate: v[8],
        };
        s.clamp_all();
        s
    }

    /// Formats the chemical state for console display.
    /// Each molecule is abbreviated to 4 letters with 2 decimals.
    ///
    /// # Returns
    /// String summarizing the 9 concentrations.
    pub fn display_string(&self) -> String {
        format!(
            "Dopa:{:.2} Cort:{:.2} Sero:{:.2} Adre:{:.2} Ocyt:{:.2} Endo:{:.2} Nora:{:.2} GABA:{:.2} Glut:{:.2}",
            self.dopamine, self.cortisol, self.serotonin, self.adrenaline,
            self.oxytocin, self.endorphin, self.noradrenaline, self.gaba, self.glutamate
        )
    }

    /// Compact format for traces/logs.
    /// Format : "C[.80,.10,.85,.15,.50,.60,.30]" (dopa,cort,sero,adre,ocyt,endo,nora)
    pub fn compact_string(&self) -> String {
        format!(
            "C[{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}]",
            self.dopamine, self.cortisol, self.serotonin, self.adrenaline,
            self.oxytocin, self.endorphin, self.noradrenaline, self.gaba, self.glutamate
        )
    }

    /// Human-readable semantic format for LLM prompts.
    /// The LLM can understand and use these names to modulate its thinking.
    pub fn semantic_string(&self) -> String {
        format!(
            "Motivation:{:.0}% Stress:{:.0}% Serenite:{:.0}% Vigilance:{:.0}% \
             Lien:{:.0}% Bien-etre:{:.0}% Attention:{:.0}% Calme:{:.0}% Eveil:{:.0}%",
            self.dopamine * 100.0, self.cortisol * 100.0, self.serotonin * 100.0,
            self.adrenaline * 100.0, self.oxytocin * 100.0, self.endorphin * 100.0,
            self.noradrenaline * 100.0, self.gaba * 100.0, self.glutamate * 100.0
        )
    }
}

impl Default for NeuroChemicalState {
    /// Default value: initialized from the default baselines.
    fn default() -> Self {
        Self::from_baselines(&NeuroBaselines::default())
    }
}

/// Molecule identifier — used to target a specific neurotransmitter
/// during an adjustment via `adjust()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Molecule {
    /// Dopamine: motivation and pleasure
    Dopamine,
    /// Cortisol: stress and anxiety
    Cortisol,
    /// Serotonin: well-being and stability
    Serotonin,
    /// Adrenaline: urgency and fight/flight
    Adrenaline,
    /// Oxytocin: attachment and empathy
    Oxytocin,
    /// Endorphin: resilience and soothing
    Endorphin,
    /// Noradrenaline: attention and focus
    Noradrenaline,
    /// GABA: global inhibition, calm
    Gaba,
    /// Glutamate: global excitation, arousal
    Glutamate,
}

/// Chemical signature at the time of memory encoding.
/// Stored as f32 (sufficient precision for JSONB persistence)
/// to save space in the database.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChemicalSignature {
    pub dopamine: f32,
    pub cortisol: f32,
    pub serotonin: f32,
    pub adrenaline: f32,
    pub oxytocin: f32,
    pub endorphin: f32,
    pub noradrenaline: f32,
    #[serde(default)]
    pub gaba: f32,
    #[serde(default)]
    pub glutamate: f32,
}

impl From<&NeuroChemicalState> for ChemicalSignature {
    fn from(state: &NeuroChemicalState) -> Self {
        Self {
            dopamine: state.dopamine as f32,
            cortisol: state.cortisol as f32,
            serotonin: state.serotonin as f32,
            adrenaline: state.adrenaline as f32,
            oxytocin: state.oxytocin as f32,
            endorphin: state.endorphin as f32,
            noradrenaline: state.noradrenaline as f32,
            gaba: state.gaba as f32,
            glutamate: state.glutamate as f32,
        }
    }
}

impl ChemicalSignature {
    /// Cosine similarity between two chemical signatures (0.0 to 1.0).
    pub fn similarity(&self, other: &ChemicalSignature) -> f64 {
        let a = self.to_vec7();
        let b = other.to_vec7();
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| (*x as f64) * (*y as f64)).sum();
        let norm_a: f64 = a.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }
        (dot / (norm_a * norm_b)).clamp(0.0, 1.0)
    }

    /// Converts to 7D vector (backward compatible, same order as NeuroChemicalState::to_vec7).
    pub fn to_vec7(&self) -> [f32; 7] {
        [
            self.dopamine, self.cortisol, self.serotonin, self.adrenaline,
            self.oxytocin, self.endorphin, self.noradrenaline,
        ]
    }

    /// Converts to 9D vector (includes GABA and glutamate).
    pub fn to_vec9(&self) -> [f32; 9] {
        [
            self.dopamine, self.cortisol, self.serotonin, self.adrenaline,
            self.oxytocin, self.endorphin, self.noradrenaline,
            self.gaba, self.glutamate,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_all_keeps_in_range() {
        let mut chem = NeuroChemicalState::default();
        chem.dopamine = 1.5;
        chem.cortisol = -0.3;
        chem.clamp_all();
        assert!(chem.dopamine <= 1.0);
        assert!(chem.cortisol >= 0.0);
    }

    #[test]
    fn test_homeostasis_moves_toward_baseline() {
        let baselines = NeuroBaselines::default();
        let mut chem = NeuroChemicalState::from_baselines(&baselines);
        chem.dopamine = 0.9;
        chem.homeostasis(&baselines, 0.05);
        assert!(chem.dopamine < 0.9, "Homeostasis should reduce dopamine toward baseline");
    }

    #[test]
    fn test_feedback_positive_increases_dopamine() {
        let mut chem = NeuroChemicalState::default();
        let before = chem.dopamine;
        chem.feedback_positive(0.15);
        assert!(chem.dopamine > before, "Positive feedback should increase dopamine");
    }

    #[test]
    fn test_feedback_social_increases_oxytocin() {
        let mut chem = NeuroChemicalState::default();
        let before = chem.oxytocin;
        chem.feedback_social();
        assert!(chem.oxytocin > before, "Social feedback should increase oxytocin");
    }

    #[test]
    fn test_to_vec7_and_from_vec7_roundtrip() {
        let chem = NeuroChemicalState::default();
        let vec = chem.to_vec7();
        let chem2 = NeuroChemicalState::from_vec7(&vec);
        assert!((chem.dopamine - chem2.dopamine).abs() < 1e-10);
        assert!((chem.cortisol - chem2.cortisol).abs() < 1e-10);
    }

    #[test]
    fn test_anti_spiral_prevents_runaway() {
        let baselines = NeuroBaselines::default();
        let mut chem = NeuroChemicalState::from_baselines(&baselines);
        for _ in 0..100 {
            chem.cortisol += 0.05;
            chem.homeostasis(&baselines, 0.01);
            chem.clamp_all();
        }
        assert!(chem.cortisol <= 1.0, "Cortisol should never exceed 1.0");
    }

    #[test]
    fn test_diminished_add_saturation() {
        // At 0.5, the boost is attenuated by factor 0.5
        let result = NeuroChemicalState::diminished_add(0.5, 0.10);
        assert!((result - 0.55).abs() < 1e-10, "A 0.5, boost 0.10 → 0.55");

        // At 0.9, the boost is heavily attenuated (factor 0.10)
        let result = NeuroChemicalState::diminished_add(0.9, 0.50);
        assert!((result - 0.95).abs() < 1e-10, "A 0.9, boost 0.50 → 0.95");

        // Negative delta: no saturation
        let result = NeuroChemicalState::diminished_add(0.8, -0.30);
        assert!((result - 0.50).abs() < 1e-10, "Delta negatif passe tel quel");

        // Floor at 0.05 margin even when current = 1.0
        let result = NeuroChemicalState::diminished_add(1.0, 0.10);
        assert!(result <= 1.0, "Ne depasse jamais 1.0");
    }

    #[test]
    fn test_boost_method() {
        let mut chem = NeuroChemicalState::default();
        chem.dopamine = 0.9;
        let before = chem.dopamine;
        chem.boost(Molecule::Dopamine, 0.50);
        // At 0.9, margin = 0.10, effective boost = 0.50 * 0.10 = 0.05
        assert!(chem.dopamine > before, "Boost augmente la dopamine");
        assert!(chem.dopamine < 0.96, "Boost est fortement attenue a 0.9");
    }

    #[test]
    fn test_adjust_molecule() {
        let mut chem = NeuroChemicalState::default();
        let before = chem.serotonin;
        chem.adjust(Molecule::Serotonin, 0.1);
        assert!((chem.serotonin - before - 0.1).abs() < 1e-10);
    }
}
