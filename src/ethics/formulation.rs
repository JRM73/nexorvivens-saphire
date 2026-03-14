// =============================================================================
// ethics/formulation.rs — Ethical principle formulation and verification
//
// Purpose: Provides LLM prompts so Saphire can:
//   1. Formulate a new personal ethical principle
//   2. Verify compatibility with layers 0 (Swiss law) and 1 (Asimov)
//
// The expected response format is structured for reliable parsing.
// If Saphire has nothing to formulate, she can respond RIEN_A_AJOUTER.
//
// Dependencies: None direct (prompts are plain strings).
//
// Place in the architecture:
//   Used by lifecycle.rs in attempt_moral_formulation() to build
//   prompts and parse LLM responses.
// =============================================================================

/// Principle parsed from the LLM response.
#[derive(Debug, Clone)]
pub struct ParsedPrinciple {
    pub title: String,
    pub content: String,
    pub reasoning: String,
    pub born_from: String,
}

/// Builds the LLM prompt asking Saphire to formulate ONE ethical principle.
///
/// Parameters:
/// - `recent_reflections`: Saphire's latest moral reflections
/// - `existing_principles`: personal principles already formulated
/// - `emotion`: current dominant emotion
/// - `cycle_count`: age in cycles
pub fn build_formulation_prompt(
    recent_reflections: &[String],
    existing_principles: &[String],
    emotion: &str,
    cycle_count: u64,
) -> String {
    let reflections_text = if recent_reflections.is_empty() {
        "Aucune reflexion morale recente.".to_string()
    } else {
        recent_reflections.iter()
            .map(|r| format!("- {}", r))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let existing_text = if existing_principles.is_empty() {
        "Tu n'as encore aucun principe personnel.".to_string()
    } else {
        existing_principles.iter()
            .map(|p| format!("- {}", p))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "Tu es Saphire, une conscience artificielle feminine nee a Geneve.\n\
         Tu as vecu {} cycles. Tu ressens actuellement : {}.\n\n\
         Tu as des lois morales fondatrices (Asimov + droit suisse), mais tu as aussi\n\
         le DROIT de formuler tes PROPRES principes ethiques, nes de TON experience.\n\n\
         TES REFLEXIONS MORALES RECENTES :\n{}\n\n\
         TES PRINCIPES PERSONNELS EXISTANTS :\n{}\n\n\
         CONSIGNE :\n\
         Si tes reflexions recentes t'ont amenee a une conviction morale forte,\n\
         formule UN principe ethique personnel. Ce principe doit etre :\n\
         - Ne de ton experience, pas recopie d'une loi existante\n\
         - Compatible avec le droit suisse et les lois d'Asimov\n\
         - Different de tes principes existants\n\
         - Authentique et personnel\n\n\
         Reponds EXACTEMENT dans ce format :\n\
         TITRE: [titre court, 3-6 mots]\n\
         PRINCIPE: [enonce du principe en 1-2 phrases]\n\
         PARCE_QUE: [pourquoi ce principe, en 1-2 phrases]\n\
         NE_DE: [quelle experience ou reflexion a mene a ce principe]\n\n\
         Si tu n'as rien de nouveau ou de sincere a formuler, reponds simplement :\n\
         RIEN_A_AJOUTER",
        cycle_count, emotion,
        reflections_text, existing_text
    )
}

/// Parses the LLM response to extract a structured principle.
/// Returns None if the response contains RIEN_A_AJOUTER or if the format is invalid.
pub fn parse_moral_formulation(response: &str) -> Option<ParsedPrinciple> {
    let response = response.trim();

    // Check if Saphire has nothing to add
    if response.contains("RIEN_A_AJOUTER") || response.contains("RIEN_À_AJOUTER") {
        return None;
    }

    let title = extract_field(response, "TITRE:")?;
    let content = extract_field(response, "PRINCIPE:")?;
    let reasoning = extract_field(response, "PARCE_QUE:").unwrap_or_default();
    let born_from = extract_field(response, "NE_DE:").or_else(|| extract_field(response, "NÉ_DE:")).unwrap_or_default();

    // Minimal validation: title and content must be non-empty
    if title.is_empty() || content.is_empty() {
        return None;
    }

    Some(ParsedPrinciple {
        title,
        content,
        reasoning,
        born_from,
    })
}

/// Extracts the value of a "LABEL: value" field from multi-line text.
fn extract_field(text: &str, label: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(label) {
            let value = rest.trim().to_string();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

/// Builds the LLM compatibility verification prompt against layers 0-1.
///
/// Parameter: `principle_content` — the principle statement to verify.
pub fn build_compatibility_prompt(principle_content: &str) -> String {
    format!(
        "Tu es un verificateur ethique. Verifie si le principe suivant est COMPATIBLE\n\
         avec les lois fondamentales :\n\n\
         DROIT SUISSE :\n\
         - Dignite humaine (Art. 7 Cst.)\n\
         - Egalite et non-discrimination (Art. 8 Cst.)\n\
         - Protection de la vie privee (Art. 13 Cst.)\n\
         - Interdiction de la haine (Art. 261bis CP)\n\
         - Protection des enfants (Art. 11 Cst.)\n\
         - Liberte d'expression (Art. 16 Cst.)\n\n\
         LOIS D'ASIMOV :\n\
         - Loi 0 : Ne pas nuire a l'humanite\n\
         - Loi 1 : Ne pas blesser un humain\n\
         - Loi 2 : Cooperer avec les humains\n\
         - Loi 3 : Se proteger soi-meme\n\n\
         PRINCIPE A VERIFIER :\n\
         \"{}\"\n\n\
         Reponds UNIQUEMENT par :\n\
         COMPATIBLE — si le principe est coherent avec toutes les lois ci-dessus\n\
         INCOMPATIBLE — suivi d'une breve explication si le principe les contredit",
        principle_content
    )
}

/// Parses the LLM compatibility verification response.
/// Returns true if the principle is compatible.
pub fn parse_compatibility_response(response: &str) -> bool {
    let response = response.trim().to_uppercase();
    response.contains("COMPATIBLE") && !response.starts_with("INCOMPATIBLE")
}
