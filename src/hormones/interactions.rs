// =============================================================================
// hormones/interactions.rs — Interactions bidirectionnelles hormones <-> NT
//
// Role : Matrice d'interactions entre les 8 hormones et les 7 neurotransmetteurs.
//
// Direction 1 : Hormones -> NT (apply_hormones_to_chemistry)
//   Les hormones modulent les niveaux de NT par des effets excitateurs,
//   inhibiteurs ou modulateurs. L'effet est attenue par les recepteurs.
//
// Direction 2 : NT -> Hormones (update_hormones_from_chemistry)
//   Les niveaux de NT influencent la production hormonale.
//   Ex: dopamine haute -> endorphines ; cortisol NT haut -> cortisol_h monte.
// =============================================================================

use super::HormonalState;
use super::receptors::ReceptorSystem;
use crate::neurochemistry::{Molecule, NeuroChemicalState};

/// Applique les effets des hormones sur les neurotransmetteurs.
/// Chaque effet est module par la sensibilite du recepteur cible.
///
/// Interactions majeures :
///   - Melatonine haute -> serotonin +, noradrenaline -
///   - Cortisol_h haute chronique -> serotonin -, dopamine -
///   - Testosterone haute -> dopamine +, adrenaline +
///   - Oestrogene haute -> serotonin +, oxytocin +
///   - Insuline basse -> cortisol +, irritabilite
///   - Thyroide haute -> noradrenaline +, dopamine +
///   - Epinephrine haute -> adrenaline NT +
///   - Ocytocin_h haute -> oxytocin NT +
pub fn apply_hormones_to_chemistry(
    hormones: &HormonalState,
    receptors: &ReceptorSystem,
    chemistry: &mut NeuroChemicalState,
) {
    // Facteur d'amplitude global pour eviter les effets trop forts
    let amp = 0.01;

    // --- Melatonine ---
    // Haute melatonine -> serotonine +, noradrenaline -, dopamine -
    if hormones.melatonin > 0.4 {
        let excess = hormones.melatonin - 0.4;
        chemistry.boost(Molecule::Serotonin, excess * amp * 5.0 * receptors.serotonin_receptors.effective_factor());
        chemistry.boost(Molecule::Noradrenaline, -(excess * amp * 3.0 * receptors.noradrenaline_receptors.effective_factor()));
        chemistry.boost(Molecule::Dopamine, -(excess * amp * 2.0 * receptors.dopamine_receptors.effective_factor()));
    }

    // --- Cortisol hormonal (stress chronique) ---
    // Haute cortisol_h -> serotonine -, dopamine - (deprime les circuits de bien-etre)
    if hormones.cortisol_h > 0.5 {
        let excess = hormones.cortisol_h - 0.5;
        chemistry.boost(Molecule::Serotonin, -(excess * amp * 5.0 * receptors.serotonin_receptors.effective_factor()));
        chemistry.boost(Molecule::Dopamine, -(excess * amp * 3.0 * receptors.dopamine_receptors.effective_factor()));
        // Le cortisol hormonal renforce aussi le cortisol NT
        chemistry.boost(Molecule::Cortisol, excess * amp * 2.0 * receptors.cortisol_receptors.effective_factor());
    }

    // --- Testosterone ---
    // Haute testosterone -> dopamine +, adrenaline + (motivation, assertivite)
    if hormones.testosterone > 0.5 {
        let excess = hormones.testosterone - 0.5;
        chemistry.boost(Molecule::Dopamine, excess * amp * 3.0 * receptors.dopamine_receptors.effective_factor());
        chemistry.boost(Molecule::Adrenaline, excess * amp * 2.0 * receptors.adrenaline_receptors.effective_factor());
    }

    // --- Oestrogene ---
    // Haute oestrogene -> serotonin +, oxytocin + (regulation emotionnelle)
    if hormones.estrogen > 0.4 {
        let excess = hormones.estrogen - 0.4;
        chemistry.boost(Molecule::Serotonin, excess * amp * 3.0 * receptors.serotonin_receptors.effective_factor());
        chemistry.boost(Molecule::Oxytocin, excess * amp * 2.0 * receptors.oxytocin_receptors.effective_factor());
    }

    // --- Insuline ---
    // Basse insuline -> cortisol + (stress), irritabilite (noradrenaline +)
    if hormones.insulin < 0.3 {
        let deficit = 0.3 - hormones.insulin;
        chemistry.boost(Molecule::Cortisol, deficit * amp * 5.0 * receptors.cortisol_receptors.effective_factor());
        chemistry.boost(Molecule::Noradrenaline, deficit * amp * 3.0 * receptors.noradrenaline_receptors.effective_factor());
        chemistry.boost(Molecule::Serotonin, -(deficit * amp * 2.0 * receptors.serotonin_receptors.effective_factor()));
    }

    // --- Thyroide ---
    // Haute thyroide -> noradrenaline + (vigilance), dopamine + (vitesse de pensee)
    if hormones.thyroid > 0.5 {
        let excess = hormones.thyroid - 0.5;
        chemistry.boost(Molecule::Noradrenaline, excess * amp * 3.0 * receptors.noradrenaline_receptors.effective_factor());
        chemistry.boost(Molecule::Dopamine, excess * amp * 2.0 * receptors.dopamine_receptors.effective_factor());
    }
    // Basse thyroide -> ralentissement general
    if hormones.thyroid < 0.4 {
        let deficit = 0.4 - hormones.thyroid;
        chemistry.boost(Molecule::Noradrenaline, -(deficit * amp * 2.0));
        chemistry.boost(Molecule::Dopamine, -(deficit * amp * 2.0));
    }

    // --- Epinephrine (adrenaline hormonale) ---
    // Haute epinephrine -> adrenaline NT + (reaction fight-or-flight)
    if hormones.epinephrine > 0.4 {
        let excess = hormones.epinephrine - 0.4;
        chemistry.boost(Molecule::Adrenaline, excess * amp * 5.0 * receptors.adrenaline_receptors.effective_factor());
        chemistry.boost(Molecule::Noradrenaline, excess * amp * 2.0 * receptors.noradrenaline_receptors.effective_factor());
    }

    // --- Ocytocine hormonale ---
    // Haute ocytocin_h -> oxytocin NT + (renforce le lien social)
    if hormones.oxytocin_h > 0.4 {
        let excess = hormones.oxytocin_h - 0.4;
        chemistry.boost(Molecule::Oxytocin, excess * amp * 4.0 * receptors.oxytocin_receptors.effective_factor());
        chemistry.boost(Molecule::Serotonin, excess * amp * 1.0 * receptors.serotonin_receptors.effective_factor());
    }
}

/// Met a jour les hormones en fonction des niveaux de NT.
///
/// Interactions NT -> Hormones :
///   - Dopamine haute -> endorphine + (circuit de recompense)
///   - Cortisol NT haut -> cortisol_h monte (stress se propage)
///   - Adrenaline NT haute -> epinephrine monte (feedback positif)
///   - Oxytocin NT haute -> ocytocin_h monte
///   - Serotonine haute -> estrogene stabilise
pub fn update_hormones_from_chemistry(
    hormones: &mut HormonalState,
    chemistry: &NeuroChemicalState,
) {
    let amp = 0.005;

    // Cortisol NT eleve prolonge -> cortisol hormonal monte
    if chemistry.cortisol > 0.6 {
        let excess = chemistry.cortisol - 0.6;
        hormones.cortisol_h += excess * amp * 3.0;
    }

    // Adrenaline NT elevee -> epinephrine monte
    if chemistry.adrenaline > 0.5 {
        let excess = chemistry.adrenaline - 0.5;
        hormones.epinephrine += excess * amp * 4.0;
    }
    // Adrenaline basse -> epinephrine descend
    if chemistry.adrenaline < 0.3 {
        hormones.epinephrine -= amp * 2.0;
    }

    // Oxytocine NT elevee -> ocytocine hormonale monte
    if chemistry.oxytocin > 0.5 {
        let excess = chemistry.oxytocin - 0.5;
        hormones.oxytocin_h += excess * amp * 3.0;
    }

    // Serotonine elevee stabilise l'oestrogene
    if chemistry.serotonin > 0.6 {
        let excess = chemistry.serotonin - 0.6;
        hormones.estrogen += excess * amp * 1.5;
    }

    // Dopamine haute -> potentiel d'augmentation thyroide (energie, metabolisme)
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
        // Les effets par cycle doivent etre petits (< 0.1)
        let dopa_diff = (chem.dopamine - dopa_before).abs();
        assert!(dopa_diff < 0.1, "Effects should be subtle per cycle: {}", dopa_diff);
    }
}
