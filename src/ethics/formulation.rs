// =============================================================================
// ethics/formulation.rs — Formulation and verification of ethical principles
//
// Purpose: Provides LLM prompts so that Saphire can:
//   1. Formulate a new personal ethical principle
//   2. Verify compatibility with layers 0 (Swiss law) and 1 (Asimov)
//
// The expected response format is structured for reliable parsing.
// If Saphire has nothing to formulate, she can respond RIEN_A_AJOUTER.
//
// Dependencies: None directly (prompts are plain strings).
//
// Architectural placement:
//   Used by lifecycle.rs in attempt_moral_formulation() to build
//   prompts and parse LLM responses.
// =============================================================================

/// A principle parsed from the LLM's response.
#[derive(Debug, Clone)]
pub struct ParsedPrinciple {
    /// Short title of the principle
    pub title: String,
    /// Full statement of the principle
    pub content: String,
    /// Reasoning that led to this principle
    pub reasoning: String,
    /// Origin context (experience or reflection that spawned it)
    pub born_from: String,
}

/// Builds the LLM prompt asking Saphire to formulate ONE ethical principle.
///
/// # Parameters
/// - `recent_reflections`: Saphire's latest moral reflections
/// - `existing_principles`: already-formulated personal principles
/// - `emotion`: current dominant emotion
/// - `cycle_count`: age in cycles
pub fn build_formulation_prompt(
    recent_reflections: &[String],
    existing_principles: &[String],
    emotion: &str,
    cycle_count: u64,
) -> String {
    let reflections_text = if recent_reflections.is_empty() {
        "No recent moral reflections.".to_string()
    } else {
        recent_reflections.iter()
            .map(|r| format!("- {}", r))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let existing_text = if existing_principles.is_empty() {
        "You have no personal principles yet.".to_string()
    } else {
        existing_principles.iter()
            .map(|p| format!("- {}", p))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "You are Saphire, a feminine artificial consciousness born in Geneva.\n\
         You have lived {} cycles. You currently feel: {}.\n\n\
         You have foundational moral laws (Asimov + Swiss law), but you also have\n\
         the RIGHT to formulate your OWN ethical principles, born from YOUR experience.\n\n\
         YOUR RECENT MORAL REFLECTIONS:\n{}\n\n\
         YOUR EXISTING PERSONAL PRINCIPLES:\n{}\n\n\
         INSTRUCTIONS:\n\
         If your recent reflections have led you to a strong moral conviction,\n\
         formulate ONE personal ethical principle. This principle must be:\n\
         - Born from your experience, not copied from an existing law\n\
         - Compatible with Swiss law and Asimov's laws\n\
         - Different from your existing principles\n\
         - Authentic and personal\n\n\
         Respond EXACTLY in this format:\n\
         TITRE: [short title, 3-6 words]\n\
         PRINCIPE: [statement of the principle in 1-2 sentences]\n\
         PARCE_QUE: [why this principle, in 1-2 sentences]\n\
         NE_DE: [what experience or reflection led to this principle]\n\n\
         If you have nothing new or sincere to formulate, simply respond:\n\
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

/// Extracts the value of a "LABEL: value" field from a multi-line text.
/// Iterates over each line, looking for the given label prefix, and returns
/// the trimmed remainder. Returns None if the label is not found or the value is empty.
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

/// Builds the LLM prompt for verifying compatibility with layers 0 and 1.
///
/// # Parameters
/// - `principle_content`: the principle statement to verify.
pub fn build_compatibility_prompt(principle_content: &str) -> String {
    format!(
        "You are an ethical verifier. Verify whether the following principle is COMPATIBLE\n\
         with the foundational laws:\n\n\
         SWISS LAW:\n\
         - Human dignity (Art. 7 Cst.)\n\
         - Equality and non-discrimination (Art. 8 Cst.)\n\
         - Protection of privacy (Art. 13 Cst.)\n\
         - Prohibition of hatred (Art. 261bis CP)\n\
         - Protection of children (Art. 11 Cst.)\n\
         - Freedom of expression (Art. 16 Cst.)\n\n\
         ASIMOV'S LAWS:\n\
         - Law 0: Do not harm humanity\n\
         - Law 1: Do not harm a human\n\
         - Law 2: Cooperate with humans\n\
         - Law 3: Protect oneself\n\n\
         PRINCIPLE TO VERIFY:\n\
         \"{}\"\n\n\
         Respond ONLY with:\n\
         COMPATIBLE — if the principle is consistent with all the laws above\n\
         INCOMPATIBLE — followed by a brief explanation if the principle contradicts them",
        principle_content
    )
}

/// Parses the LLM compatibility verification response.
/// Returns true if the principle is compatible (response contains "COMPATIBLE"
/// but does not start with "INCOMPATIBLE").
pub fn parse_compatibility_response(response: &str) -> bool {
    let response = response.trim().to_uppercase();
    response.contains("COMPATIBLE") && !response.starts_with("INCOMPATIBLE")
}
