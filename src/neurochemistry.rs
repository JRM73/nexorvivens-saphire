// =============================================================================
// neurochemistry.rs — Saphire's 9 neurotransmitter simulation
// =============================================================================
//
// Purpose: This module models Saphire's internal neurochemical state as a
// vector of 9 normalized molecules, each bounded within [0.0, 1.0]. Every
// molecule influences the agent's decision-making, emotional processing,
// and conscious experience.
//
// The neurotransmitter model is inspired by real mammalian neurochemistry:
//   - Dopamine, serotonin, oxytocin, endorphin: reward and well-being circuits
//   - Cortisol, adrenaline (epinephrine): stress and fight-or-flight responses
//   - Noradrenaline (norepinephrine): attentional vigilance (locus coeruleus)
//   - GABA: primary inhibitory neurotransmitter (tonic inhibition)
//   - Glutamate: primary excitatory neurotransmitter (synaptic plasticity)
//
// Dependencies:
//   - serde: serialization / deserialization (state persistence)
//   - crate::world::ChemistryAdjustment: external adjustments (weather, etc.)
//
// Architectural role:
//   This module is the foundational biochemical layer. It is read by:
//     - emotions.rs (dominant emotion computation via cosine similarity)
//     - consensus.rs (dynamic weighting of the 3 brain modules)
//     - consciousness.rs (consciousness level evaluation)
//     - the 3 brain modules (reptilian, limbic, neocortex)
// =============================================================================

use serde::{Deserialize, Serialize};

/// Neurochemical state of Saphire — 9 molecules normalized to [0.0, 1.0].
///
/// Each field represents the normalized concentration of a simulated
/// neurotransmitter. Together they form a 9-dimensional vector that
/// determines Saphire's emotional and decision-making state.
///
/// The first 7 molecules (dopamine through noradrenaline) form the original
/// core chemistry vector used for emotion cosine similarity matching.
/// GABA and glutamate were added to model the excitatory/inhibitory (E/I)
/// balance fundamental to neural computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroChemicalState {
    /// Dopamine: motivation, pleasure, and the mesolimbic reward circuit.
    /// High = strong drive to act (reward-seeking); low = apathy (anhedonia).
    /// References: Schultz (1997) reward prediction error; Berridge (2007) wanting vs. liking.
    pub dopamine: f64,
    /// Cortisol: primary stress hormone (HPA axis activation).
    /// High = stress state (fight-or-flight preparation); low = calm baseline.
    /// References: Sapolsky (2004) "Why Zebras Don't Get Ulcers".
    pub cortisol: f64,
    /// Serotonin: well-being, mood stability, and emotional regulation.
    /// High = serenity and emotional equilibrium; low = mood instability.
    /// References: Cools et al. (2008) serotonergic modulation of cognition.
    pub serotonin: f64,
    /// Adrenaline (epinephrine): urgency and the fight-or-flight response.
    /// High = survival mode (sympathetic activation); low = no acute pressure.
    /// References: Cannon (1929) fight-or-flight theory.
    pub adrenaline: f64,
    /// Oxytocin: attachment, empathy, and social bonding.
    /// High = desire for social connection; low = emotional detachment.
    /// References: Kosfeld et al. (2005) oxytocin and trust.
    pub oxytocin: f64,
    /// Endorphin: resilience, pain modulation, and stress buffering.
    /// High = capacity to endure stress (endogenous analgesia); low = vulnerability.
    /// References: Sprouse-Blum et al. (2010) endorphin role in pain management.
    pub endorphin: f64,
    /// Noradrenaline (norepinephrine): attention, focus, and vigilance.
    /// High = heightened concentration (locus coeruleus activation); low = distraction.
    /// References: Aston-Jones & Cohen (2005) adaptive gain theory.
    pub noradrenaline: f64,
    /// GABA (gamma-aminobutyric acid): primary inhibitory neurotransmitter.
    /// High = calm, anxiolysis (GABAergic tone); low = hyperexcitability, anxiety.
    /// Modulates all other systems via tonic inhibition of neural circuits.
    /// References: Petroff (2002) GABA and glutamate in the human brain.
    #[serde(default = "default_gaba")]
    pub gaba: f64,
    /// Glutamate: primary excitatory neurotransmitter.
    /// High = arousal, synaptic plasticity (LTP); excess = excitotoxicity risk.
    /// Maintains a fundamental balance with GABA (the E/I ratio).
    /// References: Rothman et al. (2003) glutamate-GABA cycle; Bhatt et al. (2009) excitotoxicity.
    #[serde(default = "default_glutamate")]
    pub glutamate: f64,
}

/// Returns the default GABA concentration (0.5 — balanced inhibitory tone).
fn default_gaba() -> f64 { 0.5 }

/// Returns the default glutamate concentration (0.45 — slightly below GABA
/// to maintain a safe E/I ratio and prevent excitotoxicity).
fn default_glutamate() -> f64 { 0.45 }

/// Configurable baselines for the homeostatic mechanism.
///
/// Each field defines the equilibrium value toward which the corresponding
/// molecule naturally tends over time. The homeostasis algorithm drives
/// the chemical state back toward these reference values using linear
/// interpolation, simulating the biological process by which the body
/// maintains neurochemical equilibrium (allostasis).
///
/// References: McEwen (1998) allostasis and allostatic load.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroBaselines {
    /// Dopamine baseline (default: 0.5 — neutral motivational state)
    pub dopamine: f64,
    /// Cortisol baseline (default: 0.3 — mild background stress, allows
    /// the system to express sadness/anxiety without requiring large deltas)
    pub cortisol: f64,
    /// Serotonin baseline (default: 0.6 — baseline well-being)
    pub serotonin: f64,
    /// Adrenaline baseline (default: 0.2 — low resting sympathetic tone)
    pub adrenaline: f64,
    /// Oxytocin baseline (default: 0.4 — moderate social readiness)
    pub oxytocin: f64,
    /// Endorphin baseline (default: 0.4 — moderate resilience reserve)
    pub endorphin: f64,
    /// Noradrenaline baseline (default: 0.5 — balanced attentional tone)
    pub noradrenaline: f64,
    /// GABA baseline (default: 0.5 — balanced inhibitory equilibrium)
    #[serde(default = "default_gaba_baseline")]
    pub gaba: f64,
    /// Glutamate baseline (default: 0.45 — balanced excitatory equilibrium,
    /// intentionally below GABA to maintain a safe E/I ratio)
    #[serde(default = "default_glutamate_baseline")]
    pub glutamate: f64,
}

/// Returns the default GABA baseline (0.5).
fn default_gaba_baseline() -> f64 { 0.5 }

/// Returns the default glutamate baseline (0.45).
fn default_glutamate_baseline() -> f64 { 0.45 }

impl Default for NeuroBaselines {
    /// Default baseline values — calibrated for a neutral, balanced state.
    /// Cortisol at 0.30 allows the system to express sadness/anxiety without
    /// extreme deltas. Serotonin at 0.50 provides a neutral mood foundation.
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
    /// Computes the "umami" signal — a composite multi-molecular reward score.
    ///
    /// Combines satisfaction (dopamine), well-being (serotonin), social bonding
    /// (oxytocin), and resilience (endorphin), while penalizing stress (cortisol).
    /// Returns a score in [0.0, 1.0] suitable for use as a reward signal in the
    /// UCB1 multi-armed bandit algorithm for strategy selection.
    ///
    /// Weight rationale:
    ///   - Dopamine (0.30): strongest contributor as the primary reward molecule
    ///   - Serotonin (0.25): sustained well-being is highly valued
    ///   - Oxytocin (0.20): social connection contributes to overall satisfaction
    ///   - Endorphin (0.15): resilience provides a secondary positive signal
    ///   - Cortisol (-0.10): stress acts as a penalty, reducing the reward score
    pub fn compute_umami(&self) -> f64 {
        let raw = self.dopamine * 0.30
            + self.serotonin * 0.25
            + self.oxytocin * 0.20
            + self.endorphin * 0.15
            - self.cortisol * 0.10;
        raw.clamp(0.0, 1.0)
    }

    /// Performs addition with diminishing returns (receptor saturation model).
    ///
    /// As the molecule concentration approaches 1.0, positive boosts have
    /// progressively less effect, simulating biological receptor saturation
    /// (downregulation). Negative deltas are applied linearly without
    /// saturation (receptor upregulation is faster than downregulation).
    ///
    /// The remaining headroom before saturation determines the effective boost:
    ///   effective_delta = delta * max(1.0 - current, 0.05)
    ///
    /// The 0.05 minimum headroom prevents complete blockage, ensuring that
    /// even a near-saturated molecule can still receive a tiny boost.
    ///
    /// # Parameters
    /// - `current`: current concentration of the molecule [0.0, 1.0].
    /// - `delta`: desired change (positive = increase, negative = decrease).
    ///
    /// # Returns
    /// The new concentration, clamped to [0.0, 1.0].
    pub fn diminished_add(current: f64, delta: f64) -> f64 {
        if delta <= 0.0 {
            return (current + delta).clamp(0.0, 1.0);
        }
        // Remaining headroom before saturation (minimum 0.05 to avoid total blockage)
        let saturation = (1.0 - current).max(0.05);
        (current + delta * saturation).clamp(0.0, 1.0)
    }

    /// Applies a boost with diminishing returns to a specific molecule.
    ///
    /// Uses `diminished_add` internally: at concentration 0.5 the boost is
    /// applied at 50% strength, at 0.8 it is reduced to 20%, and at 0.95
    /// it is nearly zero — modeling receptor desensitization.
    ///
    /// # Parameters
    /// - `molecule`: identifier of the target neurotransmitter.
    /// - `delta`: magnitude of the boost (positive = increase, negative = decrease).
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

    /// Creates an initial neurochemical state from the given baselines.
    ///
    /// # Parameters
    /// - `baselines`: equilibrium reference values for each molecule.
    ///
    /// # Returns
    /// A `NeuroChemicalState` initialized to the baseline concentrations.
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
    /// interpolation, simulating the natural return to biochemical equilibrium.
    ///
    /// Formula per molecule: value += (baseline - value) * rate
    ///
    /// This is a first-order exponential decay toward the target, analogous
    /// to biological negative feedback loops (e.g., HPA axis regulation for
    /// cortisol, serotonin reuptake mechanisms).
    ///
    /// # Parameters
    /// - `baselines`: target equilibrium values.
    /// - `rate`: convergence speed [0.0, 1.0]. 0.0 = no effect (no homeostasis),
    ///   1.0 = immediate return to baseline (instantaneous equilibrium).
    pub fn homeostasis(&mut self, baselines: &NeuroBaselines, rate: f64) {
        let rate = rate.clamp(0.0, 1.0);
        // Linear interpolation: value += (target - value) * rate
        self.dopamine += (baselines.dopamine - self.dopamine) * rate;
        self.cortisol += (baselines.cortisol - self.cortisol) * rate;
        self.serotonin += (baselines.serotonin - self.serotonin) * rate;
        self.adrenaline += (baselines.adrenaline - self.adrenaline) * rate;
        self.oxytocin += (baselines.oxytocin - self.oxytocin) * rate;
        self.endorphin += (baselines.endorphin - self.endorphin) * rate;
        self.noradrenaline += (baselines.noradrenaline - self.noradrenaline) * rate;
        self.gaba += (baselines.gaba - self.gaba) * rate;
        self.glutamate += (baselines.glutamate - self.glutamate) * rate;
        self.clamp_all();
    }

    /// Applies a raw delta (variation) to a specific molecule.
    ///
    /// Unlike `boost()`, this method applies the delta linearly without
    /// diminishing returns. Use this for direct, unattenuated adjustments.
    ///
    /// # Parameters
    /// - `molecule`: identifier of the molecule to modify.
    /// - `delta`: variation to apply (positive = increase, negative = decrease).
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
    ///
    /// Reinforces dopamine (satisfaction via the mesolimbic pathway),
    /// reduces cortisol (stress relief), and increases endorphins and
    /// serotonin (well-being consolidation).
    ///
    /// The `dopamine_boost` parameter comes from `TunableParams.feedback_dopamine_boost`.
    pub fn feedback_positive(&mut self, dopamine_boost: f64) {
        self.boost(Molecule::Dopamine, dopamine_boost);
        self.cortisol = (self.cortisol - dopamine_boost * 0.33).max(0.0);
        self.boost(Molecule::Endorphin, dopamine_boost * 0.53);
        self.boost(Molecule::Serotonin, dopamine_boost * 0.33);
    }

    /// Feedback after a danger-avoidance refusal (No + high danger level).
    ///
    /// Reduces cortisol and adrenaline (relief from having avoided the threat),
    /// and releases endorphins (self-preservation reward signal). This models
    /// the post-threat parasympathetic rebound observed in mammals.
    ///
    /// The `cortisol_relief` parameter comes from `TunableParams.feedback_cortisol_relief`.
    pub fn feedback_danger_avoided(&mut self, cortisol_relief: f64) {
        self.cortisol = (self.cortisol - cortisol_relief * 2.0).max(0.0);
        self.adrenaline = (self.adrenaline - cortisol_relief * 3.0).max(0.0);
        self.boost(Molecule::Endorphin, cortisol_relief);
    }

    /// Feedback after indecision (Maybe) — moderate effects with compensation.
    ///
    /// Indecision generates mild cognitive stress (cortisol increase) but also
    /// activates vigilance (noradrenaline) and resilience (endorphin). This
    /// models the adaptive uncertainty response: uncertainty is mildly aversive
    /// but also promotes heightened attention and coping resources.
    ///
    /// The `indecision_stress` parameter comes from `TunableParams.feedback_indecision_stress`.
    pub fn feedback_indecision(&mut self, indecision_stress: f64) {
        self.apply_cortisol_penalty(indecision_stress * 0.375); // Proportional stress penalty
        self.boost(Molecule::Endorphin, indecision_stress * 0.25); // Resilience compensation
        self.boost(Molecule::Noradrenaline, indecision_stress * 0.25); // Uncertainty stimulates attention
    }

    /// Applies a cortisol penalty with anti-spiral mechanism and endorphin
    /// dampening.
    ///
    /// This system prevents uncontrolled stress feedback loops through two
    /// biologically inspired mechanisms:
    ///
    /// 1. **Endorphin dampening**: higher endorphin levels attenuate the stress
    ///    impact (the dampening factor ranges from 0.7 to 1.0). This models
    ///    the analgesic and anxiolytic effects of endogenous opioids.
    ///
    /// 2. **Receptor saturation above 0.80**: beyond this cortisol threshold,
    ///    a saturation factor progressively reduces further increases, modeling
    ///    glucocorticoid receptor downregulation under chronic stress.
    ///
    /// Additionally, when cortisol exceeds 0.80, endorphin rises slightly
    /// (+0.02 per cycle), modeling the biological defense where the body
    /// releases endogenous opioids to counteract prolonged stress.
    ///
    /// # Parameters
    /// - `base_penalty`: raw cortisol penalty before dampening is applied.
    pub fn apply_cortisol_penalty(&mut self, base_penalty: f64) {
        // Endorphin dampening: higher endorphin attenuates stress impact
        // (factor ranges between 0.7 and 1.0 — reduced dampening to allow cortisol to rise)
        let endorphin_dampening = 1.0 - (self.endorphin * 0.3);

        // Receptor saturation: above 0.80 cortisol, further increases are progressively
        // reduced — modeling glucocorticoid receptor downregulation (threshold raised to
        // allow the system to experience higher stress levels)
        let saturation_factor = if self.cortisol > 0.80 {
            1.0 - ((self.cortisol - 0.80) / 0.2) * 0.6
        } else {
            1.0
        };

        // The effective penalty is the product of all three factors
        let effective_penalty = base_penalty * endorphin_dampening * saturation_factor;
        self.cortisol = (self.cortisol + effective_penalty).min(1.0);

        // Endorphin rises naturally when stress is elevated — biological defense:
        // the body releases endogenous opioids to counteract prolonged stress
        // (threshold set at 0.80)
        if self.cortisol > 0.80 {
            self.endorphin = (self.endorphin + 0.02).min(1.0);
        }
    }

    /// Feedback after a negative stimulus (hostile message, failure, etc.).
    ///
    /// Increases cortisol (stress response), decreases dopamine (reduced
    /// motivation), serotonin (impaired well-being), and oxytocin (social
    /// withdrawal). Noradrenaline rises slightly (heightened vigilance in
    /// response to threat).
    ///
    /// # Parameters
    /// - `severity`: intensity of the negativity [0.0, 1.0].
    pub fn feedback_negative(&mut self, severity: f64) {
        let s = severity.clamp(0.0, 1.0);
        self.apply_cortisol_penalty(s * 0.15);
        self.dopamine = (self.dopamine - s * 0.10).max(0.0);
        self.serotonin = (self.serotonin - s * 0.08).max(0.0);
        self.oxytocin = (self.oxytocin - s * 0.05).max(0.0);
        self.noradrenaline = (self.noradrenaline + s * 0.05).min(1.0);
    }

    /// Feedback when consensus coherence is low.
    ///
    /// A mild cognitive stress emerges from the inability to reach a clear
    /// decision (cognitive dissonance). When coherence drops below 0.3,
    /// cortisol rises and noradrenaline increases (heightened attentional
    /// effort to resolve the conflict).
    ///
    /// # Parameters
    /// - `coherence`: consensus coherence score [0.0, 1.0].
    pub fn feedback_low_coherence(&mut self, coherence: f64) {
        if coherence < 0.3 {
            let stress = (0.3 - coherence) * 0.10;
            self.apply_cortisol_penalty(stress);
            self.noradrenaline = (self.noradrenaline + stress * 0.5).min(1.0);
        }
    }

    /// Feedback after a satisfying social interaction.
    ///
    /// Increases oxytocin (social bonding, trust) and serotonin (well-being).
    /// Models the prosocial neurochemical cascade observed during positive
    /// interpersonal exchanges.
    pub fn feedback_social(&mut self) {
        self.boost(Molecule::Oxytocin, 0.10);
        self.boost(Molecule::Serotonin, 0.05);
    }

    /// Feedback after encountering novelty.
    ///
    /// Increases noradrenaline (attentional orienting response, locus coeruleus
    /// phasic firing) and dopamine (curiosity-driven exploration reward).
    /// Models the novelty-seeking response described by Berlyne (1960).
    pub fn feedback_novelty(&mut self) {
        self.boost(Molecule::Noradrenaline, 0.08);
        self.boost(Molecule::Dopamine, 0.05);
    }

    /// Applies a chemical adjustment from external sources (weather,
    /// world events, environmental stimuli, etc.).
    ///
    /// # Parameters
    /// - `adj`: structure containing per-molecule deltas, defined in the
    ///   `world` module.
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

    /// Applies a chemical adjustment with a per-molecule delta limit.
    ///
    /// Prevents external sources (needs, phobias, drugs, etc.) from causing
    /// overly abrupt changes in a single cycle. Each molecule's delta is
    /// individually clamped to [-max_delta, +max_delta] before application.
    ///
    /// # Parameters
    /// - `adj`: chemical adjustment to apply.
    /// - `max_delta`: maximum allowed variation per molecule per cycle (e.g., 0.05).
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

    /// Detects molecules in a runaway state above the 0.85 alert threshold.
    ///
    /// Returns a list of molecule names and their current values for all
    /// molecules exceeding the threshold. This is used as a safety mechanism
    /// to flag neurochemical imbalances that could lead to pathological states
    /// (e.g., dopamine > 0.85 could indicate manic-like behavior).
    ///
    /// # Returns
    /// A vector of (molecule_name, current_value) tuples for all molecules
    /// above the alert threshold.
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

    /// Clamps all molecule concentrations to the valid range [0.0, 1.0].
    ///
    /// Called after every modification to guarantee data integrity and
    /// prevent out-of-range values from propagating through the system.
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

    /// Converts the chemical state into a 7-dimensional vector.
    ///
    /// Order: [dopamine, cortisol, serotonin, adrenaline, oxytocin,
    /// endorphin, noradrenaline].
    ///
    /// This is the original chemistry vector used for cosine similarity
    /// matching against the 36-emotion catalog in `emotions.rs`.
    ///
    /// # Returns
    /// An array of 7 floats representing the core neurotransmitter concentrations.
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

    /// Converts the chemical state into a 9-dimensional vector (includes GABA
    /// and glutamate).
    ///
    /// Order: [dopamine, cortisol, serotonin, adrenaline, oxytocin,
    /// endorphin, noradrenaline, gaba, glutamate].
    ///
    /// Used by the consciousness evaluator and interaction matrix for
    /// full-spectrum neurochemical analysis.
    ///
    /// # Returns
    /// An array of 9 floats representing all neurotransmitter concentrations.
    pub fn to_vec9(&self) -> [f64; 9] {
        [
            self.dopamine, self.cortisol, self.serotonin, self.adrenaline,
            self.oxytocin, self.endorphin, self.noradrenaline,
            self.gaba, self.glutamate,
        ]
    }

    /// Applies cross-molecule interactions using the pharmacological
    /// interaction matrix.
    ///
    /// Models real neurotransmitter interactions (e.g., serotonin inhibits
    /// dopamine release, cortisol suppresses serotonin synthesis, GABA
    /// inhibits glutamate excitation). Called every cognitive cycle AFTER
    /// homeostasis has been applied.
    ///
    /// Each interaction delta is clamped to [-0.03, +0.03] per cycle to
    /// prevent cascading instabilities from cross-molecule feedback.
    ///
    /// # Parameters
    /// - `interaction_matrix`: the pharmacological interaction matrix from
    ///   the `neuroscience::receptors` module.
    pub fn apply_interactions(&mut self, interaction_matrix: &crate::neuroscience::receptors::InteractionMatrix) {
        let deltas = interaction_matrix.compute_deltas(self);
        // Apply deltas with attenuation (maximum 0.03 per cycle per interaction
        // to prevent cascading oscillations)
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
    ///
    /// GABA and glutamate are initialized to their default values (0.5 and
    /// 0.45 respectively) since they are not present in the 7D vector.
    /// All values are automatically clamped to [0.0, 1.0].
    ///
    /// # Parameters
    /// - `v`: array of 7 floats in the same order as `to_vec7`.
    ///
    /// # Returns
    /// A valid `NeuroChemicalState` with clamped values.
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
    ///
    /// All values are automatically clamped to [0.0, 1.0].
    ///
    /// # Parameters
    /// - `v`: array of 9 floats in the same order as `to_vec9`.
    ///
    /// # Returns
    /// A valid `NeuroChemicalState` with clamped values.
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
    ///
    /// Each molecule is abbreviated to 4 letters with 2 decimal places.
    ///
    /// # Returns
    /// A string summarizing all 9 neurotransmitter concentrations.
    pub fn display_string(&self) -> String {
        format!(
            "Dopa:{:.2} Cort:{:.2} Sero:{:.2} Adre:{:.2} Ocyt:{:.2} Endo:{:.2} Nora:{:.2} GABA:{:.2} Glut:{:.2}",
            self.dopamine, self.cortisol, self.serotonin, self.adrenaline,
            self.oxytocin, self.endorphin, self.noradrenaline, self.gaba, self.glutamate
        )
    }

    /// Compact format for trace logs.
    ///
    /// Format: "C[.80,.10,.85,.15,.50,.60,.30,.50,.45]"
    /// (dopamine, cortisol, serotonin, adrenaline, oxytocin, endorphin,
    /// noradrenaline, gaba, glutamate)
    pub fn compact_string(&self) -> String {
        format!(
            "C[{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}]",
            self.dopamine, self.cortisol, self.serotonin, self.adrenaline,
            self.oxytocin, self.endorphin, self.noradrenaline, self.gaba, self.glutamate
        )
    }

    /// Semantic, human-readable format for LLM prompts.
    ///
    /// Uses descriptive labels (Motivation, Stress, Serenity, etc.) with
    /// percentage values so the LLM can understand and modulate its thinking
    /// based on the current neurochemical state.
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

/// Identifier for a specific molecule — used to target a neurotransmitter
/// in calls to `adjust()` or `boost()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Molecule {
    /// Dopamine: motivation and pleasure (mesolimbic reward pathway)
    Dopamine,
    /// Cortisol: stress and anxiety (HPA axis)
    Cortisol,
    /// Serotonin: well-being and mood stability (raphe nuclei)
    Serotonin,
    /// Adrenaline: urgency and the fight-or-flight response (adrenal medulla)
    Adrenaline,
    /// Oxytocin: attachment and empathy (hypothalamic-pituitary system)
    Oxytocin,
    /// Endorphin: resilience and pain modulation (endogenous opioid system)
    Endorphin,
    /// Noradrenaline: attention and focus (locus coeruleus)
    Noradrenaline,
    /// GABA: global inhibition, calm (GABAergic interneurons)
    Gaba,
    /// Glutamate: global excitation, arousal (glutamatergic synapses)
    Glutamate,
}

/// Chemical signature captured at the moment a memory is encoded.
///
/// Stored as f32 (sufficient precision for JSONB persistence) to conserve
/// database storage space. This snapshot allows memory retrieval to be
/// modulated by neurochemical similarity — memories encoded under similar
/// chemistry are more easily recalled (state-dependent memory).
///
/// References: Godden & Baddeley (1975) state-dependent retrieval.
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
    /// Converts a full-precision `NeuroChemicalState` (f64) into a
    /// compact `ChemicalSignature` (f32) for storage.
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
    /// Computes cosine similarity between two chemical signatures (range [0.0, 1.0]).
    ///
    /// Uses the 7D core vector (excluding GABA and glutamate) for backward
    /// compatibility with older stored signatures. The cosine similarity
    /// measures the angular distance between two neurochemical profiles,
    /// enabling state-dependent memory retrieval.
    ///
    /// Formula: cos(theta) = (A . B) / (||A|| * ||B||)
    ///
    /// # Parameters
    /// - `other`: the other chemical signature to compare against.
    ///
    /// # Returns
    /// Similarity score clamped to [0.0, 1.0]. Returns 0.0 if either
    /// vector has zero norm.
    pub fn similarity(&self, other: &ChemicalSignature) -> f64 {
        let a = self.to_vec7();
        let b = other.to_vec7();
        // Compute dot product in f64 precision to avoid f32 rounding errors
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| (*x as f64) * (*y as f64)).sum();
        let norm_a: f64 = a.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }
        (dot / (norm_a * norm_b)).clamp(0.0, 1.0)
    }

    /// Converts to a 7D vector (backward-compatible, same order as
    /// `NeuroChemicalState::to_vec7`).
    pub fn to_vec7(&self) -> [f32; 7] {
        [
            self.dopamine, self.cortisol, self.serotonin, self.adrenaline,
            self.oxytocin, self.endorphin, self.noradrenaline,
        ]
    }

    /// Converts to a 9D vector (includes GABA and glutamate).
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
        // At 0.5, the boost is attenuated by the saturation factor 0.5
        let result = NeuroChemicalState::diminished_add(0.5, 0.10);
        assert!((result - 0.55).abs() < 1e-10, "At 0.5, boost 0.10 -> 0.55");

        // At 0.9, the boost is heavily attenuated (factor 0.10)
        let result = NeuroChemicalState::diminished_add(0.9, 0.50);
        assert!((result - 0.95).abs() < 1e-10, "At 0.9, boost 0.50 -> 0.95");

        // Negative delta: no saturation applied
        let result = NeuroChemicalState::diminished_add(0.8, -0.30);
        assert!((result - 0.50).abs() < 1e-10, "Negative delta applied linearly");

        // Floor at 0.05 headroom even when current = 1.0
        let result = NeuroChemicalState::diminished_add(1.0, 0.10);
        assert!(result <= 1.0, "Must never exceed 1.0");
    }

    #[test]
    fn test_boost_method() {
        let mut chem = NeuroChemicalState::default();
        chem.dopamine = 0.9;
        let before = chem.dopamine;
        chem.boost(Molecule::Dopamine, 0.50);
        // At 0.9, headroom = 0.10, effective boost = 0.50 * 0.10 = 0.05
        assert!(chem.dopamine > before, "Boost should increase dopamine");
        assert!(chem.dopamine < 0.96, "Boost should be heavily attenuated at 0.9");
    }

    #[test]
    fn test_adjust_molecule() {
        let mut chem = NeuroChemicalState::default();
        let before = chem.serotonin;
        chem.adjust(Molecule::Serotonin, 0.1);
        assert!((chem.serotonin - before - 0.1).abs() < 1e-10);
    }
}
