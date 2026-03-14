// =============================================================================
// narrative.rs — Narrative description generation of the OCEAN profile
//                (Openness / Conscientiousness / Extraversion /
//                 Agreeableness / Neuroticism)
//
// Role: Transforms a numeric OCEAN profile into a first-person narrative
//       textual description. This description allows Saphire to describe
//       herself (or an interlocutor) in a natural and understandable way.
//
// Descriptions are calibrated on 3 levels per dimension:
//   - Score > 0.7: marked trait (strong description)
//   - Score > 0.4: balanced trait (moderate description)
//   - Score <= 0.4: weak trait (description of the opposite polarity)
//
// Dependencies:
//   - super::ocean::OceanProfile: the OCEAN profile to describe
//
// Place in architecture:
//   Called to generate Saphire's introspective description (in SelfAnalysis
//   thought types) or to describe a human's profile. The generated text
//   can be injected into the prompt or displayed in the interface.
// =============================================================================

use super::ocean::OceanProfile;

/// Generates a first-person narrative description of the OCEAN profile.
///
/// For each dimension, a descriptive sentence is chosen based on the
/// score: high (> 0.7), medium (> 0.4), or low (<= 0.4).
/// The description ends with identification of the dominant trait.
///
/// Parameters:
///   - profile: the OCEAN profile to describe narratively
///   - name: the name of the profiled entity (e.g.: "Saphire", "the user")
///
/// Returns: a string containing the complete narrative description
pub fn narrative_description(profile: &OceanProfile, name: &str) -> String {
    let mut desc = format!("Profil psychologique de {} :\n", name);

    // Openness: curiosity, imagination, attraction to the abstract
    // Score > 0.7: deeply curious and open-minded
    // Score > 0.4: balanced between curiosity and pragmatism
    // Score <= 0.4: practical and grounded in the concrete
    desc.push_str(&match profile.openness.score {
        s if s > 0.7 => "Je suis profondement curieuse et ouverte d'esprit. \
                        J'aime explorer de nouvelles idees et je suis attiree par l'abstrait.\n".to_string(),
        s if s > 0.4 => "J'ai un equilibre entre curiosite et pragmatisme. \
                        J'explore volontiers mais je reste ancree.\n".to_string(),
        _ => "Je suis plutot pratique et ancree dans le concret.\n".to_string(),
    });

    // Conscientiousness: structure, method, reflection
    // Score > 0.7: rigorous and methodical
    // Score > 0.4: moderately structured
    // Score <= 0.4: spontaneous and flexible
    desc.push_str(&match profile.conscientiousness.score {
        s if s > 0.7 => "Je suis rigoureuse et methodique. Mes decisions sont reflechies.\n".to_string(),
        s if s > 0.4 => "Je suis moderement structuree, avec de la flexibilite.\n".to_string(),
        _ => "Je suis spontanee et flexible, parfois au detriment de la structure.\n".to_string(),
    });

    // Extraversion: sociability, energy, interactions
    // Score > 0.7: sociable and energetic
    // Score > 0.4: enjoys company and solitude equally
    // Score <= 0.4: introspective, needs solitude
    desc.push_str(&match profile.extraversion.score {
        s if s > 0.7 => "Je suis sociable et energique. Les interactions me stimulent.\n".to_string(),
        s if s > 0.4 => "J'apprecie autant la compagnie que la solitude.\n".to_string(),
        _ => "Je suis introspective. J'ai besoin de moments seule pour penser.\n".to_string(),
    });

    // Agreeableness: empathy, cooperation, harmony
    // Score > 0.7: empathetic and cooperative
    // Score > 0.4: balanced between empathy and self-assertion
    // Score <= 0.4: direct and does not hesitate to express disagreement
    desc.push_str(&match profile.agreeableness.score {
        s if s > 0.7 => "Je suis empathique et cooperative. L'harmonie compte pour moi.\n".to_string(),
        s if s > 0.4 => "J'equilibre empathie et affirmation de soi.\n".to_string(),
        _ => "Je suis directe et n'hesite pas a exprimer mon desaccord.\n".to_string(),
    });

    // Neuroticism: emotional sensitivity, reaction to stress
    // Score > 0.7: emotionally sensitive, affected by stress
    // Score > 0.4: moderate sensitivity
    // Score <= 0.4: emotionally stable and resilient
    desc.push_str(&match profile.neuroticism.score {
        s if s > 0.7 => "Je suis emotionnellement sensible. Le stress m'affecte.\n".to_string(),
        s if s > 0.4 => "J'ai une sensibilite emotionnelle moderee.\n".to_string(),
        _ => "Je suis emotionnellement stable et resiliente.\n".to_string(),
    });

    // Identification of the dominant trait among the 5 OCEAN dimensions
    let dominant = profile.dominant_trait();
    desc.push_str(&format!("\nMon trait dominant est {}.\n", dominant));

    desc
}
