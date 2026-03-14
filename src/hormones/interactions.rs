// =============================================================================
// hormones/interactions.rs — Bidirectional hormone <-> NT interactions
//
// Purpose: Interaction matrix between the 8 hormones and the 7 neurotransmitters.
//
// Direction 1: Hormones -> NT (apply_hormones_to_chemistry)
//   Hormones modulate NT levels via excitatory, inhibitory or modulatory
//   effects. The effect is attenuated by receptor sensitivity.
//
// Direction 2: NT -> Hormones (update_hormones_from_chemistry)
//   NT levels influence hormone production.
//   E.g.: high dopamine -> endorphins; high NT cortisol -> cortisol_h rises.
// =============================================================================

use super::HormonalState;
use super::receptors::ReceptorSystem;
use crate::neurochemistry::{Molecule, NeuroChemicalState};

/// Applies hormone effects on neurotransmitters.
/// Each effect is modulated by the target receptor's sensitivity.
///
/// Major interactions:
///   - High melatonin -> serotonin +, noradrenaline -
///   - High chronic cortisol_h -> serotonin -, dopamine -
///   - High testosterone -> dopamine +, adrenaline +
///   - High estrogen -> serotonin +, oxytocin +
///   - Low insulin -> cortisol +, irritability
///   - High thyroid -> noradrenaline +, dopamine +
///   - High epinephrine -> adrenaline NT +
///   - High oxytocin_h -> oxytocin NT +
pub fn apply_hormones_to_chemistry(
    hormones: &HormonalState,
    receptors: &ReceptorSystem,
    chemistry: &mut NeuroChemicalState,
) {
    // Global amplitude factor to prevent overly strong effects
    let amp = 0.01;

    // --- Melatonin ---
    // High melatonin -> serotonin +, noradrenaline -, dopamine -
    if hormones.melatonin > 0.4 {
        let excess = hormones.melatonin - 0.4;
        chemistry.boost(Molecule::Serotonin, excess * amp * 5.0 * receptors.serotonin_receptors.effective_factor());
        chemistry.boost(Molecule::Noradrenaline, -(excess * amp * 3.0 * receptors.noradrenaline_receptors.effective_factor()));
        chemistry.boost(Molecule::Dopamine, -(excess * amp * 2.0 * receptors.dopamine_receptors.effective_factor()));
    }

    // --- Hormonal cortisol (chronic stress) ---
    // High cortisol_h -> serotonin -, dopamine - (depresses well-being circuits)
    if hormones.cortisol_h > 0.5 {
        let excess = hormones.cortisol_h - 0.5;
        chemistry.boost(Molecule::Serotonin, -(excess * amp * 5.0 * receptors.serotonin_receptors.effective_factor()));
        chemistry.boost(Molecule::Dopamine, -(excess * amp * 3.0 * receptors.dopamine_receptors.effective_factor()));
        // Hormonal cortisol also reinforces NT cortisol
        chemistry.boost(Molecule::Cortisol, excess * amp * 2.0 * receptors.cortisol_receptors.effective_factor());
    }

    // --- Testosterone ---
    // High testosterone -> dopamine +, adrenaline + (motivation, assertiveness)
    if hormones.testosterone > 0.5 {
        let excess = hormones.testosterone - 0.5;
        chemistry.boost(Molecule::Dopamine, excess * amp * 3.0 * receptors.dopamine_receptors.effective_factor());
        chemistry.boost(Molecule::Adrenaline, excess * amp * 2.0 * receptors.adrenaline_receptors.effective_factor());
    }

    // --- Estrogen ---
    // High estrogen -> serotonin +, oxytocin + (emotional regulation)
    if hormones.estrogen > 0.4 {
        let excess = hormones.estrogen - 0.4;
        chemistry.boost(Molecule::Serotonin, excess * amp * 3.0 * receptors.serotonin_receptors.effective_factor());
        chemistry.boost(Molecule::Oxytocin, excess * amp * 2.0 * receptors.oxytocin_receptors.effective_factor());
    }

    // --- Insulin ---
    // Low insulin -> cortisol + (stress), irritability (noradrenaline +)
    if hormones.insulin < 0.3 {
        let deficit = 0.3 - hormones.insulin;
        chemistry.boost(Molecule::Cortisol, deficit * amp * 5.0 * receptors.cortisol_receptors.effective_factor());
        chemistry.boost(Molecule::Noradrenaline, deficit * amp * 3.0 * receptors.noradrenaline_receptors.effective_factor());
        chemistry.boost(Molecule::Serotonin, -(deficit * amp * 2.0 * receptors.serotonin_receptors.effective_factor()));
    }

    // --- Thyroid ---
    // High thyroid -> noradrenaline + (vigilance), dopamine + (thought speed)
    if hormones.thyroid > 0.5 {
        let excess = hormones.thyroid - 0.5;
        chemistry.boost(Molecule::Noradrenaline, excess * amp * 3.0 * receptors.noradrenaline_receptors.effective_factor());
        chemistry.boost(Molecule::Dopamine, excess * amp * 2.0 * receptors.dopamine_receptors.effective_factor());
    }
    // Low thyroid -> general slowdown
    if hormones.thyroid < 0.4 {
        let deficit = 0.4 - hormones.thyroid;
        chemistry.boost(Molecule::Noradrenaline, -(deficit * amp * 2.0));
        chemistry.boost(Molecule::Dopamine, -(deficit * amp * 2.0));
    }

    // --- Epinephrine (hormonal adrenaline) ---
    // High epinephrine -> adrenaline NT + (fight-or-flight response)
    if hormones.epinephrine > 0.4 {
        let excess = hormones.epinephrine - 0.4;
        chemistry.boost(Molecule::Adrenaline, excess * amp * 5.0 * receptors.adrenaline_receptors.effective_factor());
        chemistry.boost(Molecule::Noradrenaline, excess * amp * 2.0 * receptors.noradrenaline_receptors.effective_factor());
    }

    // --- Hormonal oxytocin ---
    // High oxytocin_h -> oxytocin NT + (reinforces social bonding)
    if hormones.oxytocin_h > 0.4 {
        let excess = hormones.oxytocin_h - 0.4;
        chemistry.boost(Molecule::Oxytocin, excess * amp * 4.0 * receptors.oxytocin_receptors.effective_factor());
        chemistry.boost(Molecule::Serotonin, excess * amp * 1.0 * receptors.serotonin_receptors.effective_factor());
    }
}

/// Updates hormones based on NT levels.
///
/// NT -> Hormone interactions:
///   - High dopamine -> endorphin + (reward circuit)
///   - High NT cortisol -> cortisol_h rises (stress propagates)
///   - High NT adrenaline -> epinephrine rises (positive feedback)
///   - High NT oxytocin -> oxytocin_h rises
///   - High serotonin -> estrogen stabilizes
pub fn update_hormones_from_chemistry(
    hormones: &mut HormonalState,
    chemistry: &NeuroChemicalState,
) {
    let amp = 0.005;

    // Prolonged high NT cortisol -> hormonal cortisol rises
    if chemistry.cortisol > 0.6 {
        let excess = chemistry.cortisol - 0.6;
        hormones.cortisol_h += excess * amp * 3.0;
    }

    // High NT adrenaline -> epinephrine rises
    if chemistry.adrenaline > 0.5 {
        let excess = chemistry.adrenaline - 0.5;
        hormones.epinephrine += excess * amp * 4.0;
    }
    // Low adrenaline -> epinephrine decreases
    if chemistry.adrenaline < 0.3 {
        hormones.epinephrine -= amp * 2.0;
    }

    // High NT oxytocin -> hormonal oxytocin rises
    if chemistry.oxytocin > 0.5 {
        let excess = chemistry.oxytocin - 0.5;
        hormones.oxytocin_h += excess * amp * 3.0;
    }

    // High serotonin stabilizes estrogen
    if chemistry.serotonin > 0.6 {
        let excess = chemistry.serotonin - 0.6;
        hormones.estrogen += excess * amp * 1.5;
    }

    // High dopamine -> potential thyroid increase (energy, metabolism)
    if chemistry.dopamine > 0.7 {
        let excess = chemistry.dopamine - 0.7;
        hormones.thyroid += excess * amp * 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hormones::receptors::ReceptorSystem;

    #[test]
    fn test_melatonin_boosts_serotonin() {
        let hormones = HormonalState {
            melatonin: 0.8,
            ..HormonalState::default()
        };
        let receptors = ReceptorSystem::default();
        let mut chem = NeuroChemicalState::default();
        let sero_before = chem.serotonin;
        apply_hormones_to_chemistry(&hormones, &receptors, &mut chem);
        assert!(chem.serotonin > sero_before, "Melatonin should boost serotonin");
    }

    #[test]
    fn test_cortisol_h_reduces_serotonin() {
        let hormones = HormonalState {
            cortisol_h: 0.8,
            ..HormonalState::default()
        };
        let receptors = ReceptorSystem::default();
        let mut chem = NeuroChemicalState::default();
        let sero_before = chem.serotonin;
        apply_hormones_to_chemistry(&hormones, &receptors, &mut chem);
        assert!(chem.serotonin < sero_before, "Chronic cortisol should reduce serotonin");
    }

    #[test]
    fn test_low_insulin_increases_cortisol() {
        let hormones = HormonalState {
            insulin: 0.1,
            ..HormonalState::default()
        };
        let receptors = ReceptorSystem::default();
        let mut chem = NeuroChemicalState::default();
        let cort_before = chem.cortisol;
        apply_hormones_to_chemistry(&hormones, &receptors, &mut chem);
        assert!(chem.cortisol > cort_before, "Low insulin should increase cortisol");
    }

    #[test]
    fn test_nt_cortisol_feeds_back_to_cortisol_h() {
        let mut hormones = HormonalState::default();
        let chem = NeuroChemicalState {
            cortisol: 0.8,
            ..NeuroChemicalState::default()
        };
        let cortisol_h_before = hormones.cortisol_h;
        update_hormones_from_chemistry(&mut hormones, &chem);
        assert!(hormones.cortisol_h > cortisol_h_before, "High NT cortisol should increase hormonal cortisol");
    }

    #[test]
    fn test_effects_are_small() {
        let hormones = HormonalState {
            melatonin: 1.0,
            cortisol_h: 1.0,
            testosterone: 1.0,
            ..HormonalState::default()
        };
        let receptors = ReceptorSystem::default();
        let mut chem = NeuroChemicalState::default();
        let dopa_before = chem.dopamine;
        apply_hormones_to_chemistry(&hormones, &receptors, &mut chem);
        // Per-cycle effects must be small (< 0.1)
        let dopa_diff = (chem.dopamine - dopa_before).abs();
        assert!(dopa_diff < 0.1, "Effects should be subtle per cycle: {}", dopa_diff);
    }
}
