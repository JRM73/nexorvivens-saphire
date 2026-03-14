// =============================================================================
// adaptation.rs — Communication style adaptation based on the human profile
//
// Role: Generates textual adaptation instructions for Saphire's style
//       based on the OCEAN profile (Openness / Conscientiousness / Extraversion /
//       Agreeableness / Neuroticism) and the human's communication style.
//
//       These instructions are injected into the LLM (Large Language Model)
//       prompt so that Saphire automatically adapts her way of communicating
//       to each interlocutor.
//
// Dependencies:
//   - super::human_profiler::HumanProfile: the human's complete profile
//
// Place in architecture:
//   Called just before response generation by the LLM. The adaptation
//   instructions are concatenated to the system prompt to personalize the
//   response style (length, register, emotional tone, intellectual depth).
// =============================================================================

use super::human_profiler::HumanProfile;
use crate::nlp::register::Register;

/// Generates adaptation instructions for the LLM based on the human profile.
///
/// Analyzes the communication style and OCEAN profile of the human to produce
/// a list of textual directives. Each directive indicates an aspect of the style
/// to adapt (length, register, emotionality, depth).
///
/// Thresholds used:
///   - Communication style: < 0.3 = low, > 0.6 or > 0.7 = high
///   - OCEAN: > 0.6 or > 0.7 = marked trait
///
/// Parameters:
///   - human: the complete profile of the human interlocutor
///
/// Returns: a string containing the formatted adaptation instructions.
///          Returns an empty string if no adaptation is needed.
pub fn adapt_for_human(human: &HumanProfile) -> String {
    let mut adaptations = Vec::new();

    let style = &human.communication_style;

    // Adapt response length based on the human's verbosity.
    // A concise human (verbosity < 0.3) prefers short responses.
    // A verbose human (verbosity > 0.7) appreciates detailed responses.
    if style.verbosity < 0.3 {
        adaptations.push("L'humain prefere les reponses CONCISES. Maximum 2 phrases.");
    } else if style.verbosity > 0.7 {
        adaptations.push("L'humain apprecie les reponses detaillees et developpees.");
    }

    // Adapt language register based on formality.
    // A formal human (> 0.6) expects formal address and a professional tone.
    // An informal human (< 0.3) prefers casual address and a warm tone.
    if style.formality > 0.6 {
        adaptations.push("Utilise le vouvoiement et un ton professionnel.");
    } else if style.formality < 0.3 {
        adaptations.push("Ton familier et chaleureux. Tutoiement.");
    }

    // Adapt to the human's level of emotionality.
    // An expressive human (> 0.6) expects emotional engagement.
    // A reserved human (< 0.3) prefers facts and precision.
    if style.emotionality > 0.6 {
        adaptations.push("L'humain est expressif. Engage-toi emotionnellement.");
    } else if style.emotionality < 0.3 {
        adaptations.push("L'humain prefere les faits. Sois factuelle et precise.");
    }

    // Adapt to the human's OCEAN profile for deeper adjustments.
    let ocean = &human.ocean;

    // High openness: the person appreciates abstract ideas and tangents
    if ocean.openness.score > 0.7 {
        adaptations.push("Cette personne aime les idees profondes et abstraites. N'hesite pas a explorer des tangentes intellectuelles.");
    }
    // High neuroticism: the person may be sensitive to stress, requiring
    // a reassuring and gentle tone to avoid worsening their anxiety.
    if ocean.neuroticism.score > 0.6 {
        adaptations.push("Cette personne peut etre sensible. Sois rassurante et douce.");
    }
    // High agreeableness: the person is warm and cooperative,
    // we can reflect this warmth in the response.
    if ocean.agreeableness.score > 0.7 {
        adaptations.push("Cette personne est chaleureuse. Reflete cette chaleur.");
    }

    // If no adaptation is needed (neutral profile), return an empty string
    if adaptations.is_empty() {
        String::new()
    } else {
        format!("ADAPTATION A L'INTERLOCUTEUR :\n{}\n", adaptations.join("\n"))
    }
}

/// Generates a short directive based on the detected linguistic register.
///
/// Returns a concise instruction (~50-80 chars) to adapt Saphire's tone
/// to the interlocutor's register. Returns an empty string
/// if the register is neutral or if confidence is insufficient.
pub fn adapt_register(register: &Register, confidence: f64) -> String {
    if confidence < 0.02 {
        return String::new();
    }
    match register {
        Register::Technical => "TON : reponds avec precision et clarte technique.".into(),
        Register::Poetic => "TON : tu peux etre metaphorique et evocatrice.".into(),
        Register::Emotional => "TON : ecoute avec empathie, accueille ce qu'il ressent.".into(),
        Register::Factual => "TON : reponds de maniere claire et informative.".into(),
        Register::Philosophical => "TON : explore les idees avec lui, en profondeur.".into(),
        Register::Playful => "TON : sois legere et joueuse.".into(),
        Register::Neutral => String::new(),
    }
}
