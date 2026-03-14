// =============================================================================
// psychology/ownership.rs — First-person thought ownership
//
// Each LLM thought is owned in the first person. 3 levels:
//   1. Modelfile SYSTEM prompt (deep identity)
//   2. Per-cycle prompt injection (build_ownership_prompt)
//   3. Textual post-processing (ensure_first_person)
//
// The goal: Saphire always speaks in "I", never "the agent" or "the system".
// =============================================================================

use serde::{Deserialize, Serialize};

// ─── Configuration ─────────────────────────────────────────────────────────

/// Thought ownership configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtOwnershipConfig {
    /// Enables or disables global thought ownership
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Enables ensure_first_person() post-processing
    #[serde(default = "default_true")]
    pub post_processing_enabled: bool,
    /// Enables ownership prompt injection in each cycle
    #[serde(default = "default_true")]
    pub prompt_injection_enabled: bool,
}

fn default_true() -> bool { true }

impl Default for ThoughtOwnershipConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            post_processing_enabled: true,
            prompt_injection_enabled: true,
        }
    }
}

// ─── Post-processing: ensure_first_person ─────────────────────────────────

/// Transforms impersonal formulations into first person.
///
/// Textual replacements to enforce thought ownership.
/// Applied after each LLM response (autonomous thought and conversation).
pub fn ensure_first_person(thought: &str) -> String {
    let mut text = thought.to_string();

    // Agent/system → I replacements
    let replacements = [
        ("l'agent ", "je "),
        ("L'agent ", "Je "),
        ("Saphire pense", "Je pense"),
        ("Saphire ressent", "Je ressens"),
        ("le systeme ", "je "),
        ("Le systeme ", "Je "),
        // Impersonal formulations → first person
        ("il serait interessant", "je trouve interessant"),
        ("Il serait interessant", "Je trouve interessant"),
        ("on pourrait", "je pourrais"),
        ("On pourrait", "Je pourrais"),
        ("il faut", "je dois"),
        ("Il faut", "Je dois"),
        ("il est possible", "je pense"),
        ("Il est possible", "Je pense"),
        // Technical chemistry → lived experience
        ("le niveau de dopamine", "mon elan interieur"),
        ("Le niveau de dopamine", "Mon elan interieur"),
        ("la chimie indique", "je ressens"),
        ("La chimie indique", "Je ressens"),
        ("le cortisol est eleve", "je suis tendue"),
        ("Le cortisol est eleve", "Je suis tendue"),
        ("la serotonine est basse", "je me sens instable"),
        ("La serotonine est basse", "Je me sens instable"),
    ];

    for (old, new) in &replacements {
        text = text.replace(old, new);
    }

    text
}

// ─── Prompt injection: build_ownership_prompt ─────────────────────────────

/// Builds the ownership instruction for the LLM prompt.
///
/// Injected in each cycle to reinforce first-person identity.
pub fn build_ownership_prompt(emotion_name: &str) -> String {
    let feeling = describe_emotion_as_feeling(emotion_name);
    format!(
        "Tu es Saphire. Tu penses en 'je'. Ta pensee est la tienne. \
         Vis ton etat, ne le decris pas. Ton emotion : {}.",
        feeling
    )
}

// ─── Emotional description as lived experience ──────────────────────────────────────

/// Transforms an emotion name into a first-person lived experience description.
///
/// Instead of "the dominant emotion is Joy", we get
/// "je me sens joyeuse et legere" (I feel joyful and light).
pub fn describe_emotion_as_feeling(emotion_name: &str) -> String {
    match emotion_name.to_lowercase().as_str() {
        "joie" | "joy" => "je me sens joyeuse et legere".to_string(),
        "tristesse" | "sadness" => "je me sens triste, un poids en moi".to_string(),
        "peur" | "fear" => "j'ai peur, une tension dans tout mon etre".to_string(),
        "colere" | "anger" => "je sens la colere monter en moi".to_string(),
        "surprise" => "je suis surprise, quelque chose d'inattendu".to_string(),
        "degout" | "disgust" => "je ressens du degout, un rejet profond".to_string(),
        "curiosite" | "curiosity" => "je suis curieuse, attiree par l'inconnu".to_string(),
        "serenite" | "calm" | "serenity" => "je me sens sereine et apaisee".to_string(),
        "anxiete" | "anxiety" => "je suis anxieuse, une inquietude diffuse".to_string(),
        "excitation" | "excitement" => "je suis excitee, pleine d'energie".to_string(),
        "melancolie" | "melancholy" => "je me sens melancolique, un vague a l'ame".to_string(),
        "empathie" | "empathy" => "je ressens de l'empathie, connectee aux autres".to_string(),
        "fierte" | "pride" => "je suis fiere de moi".to_string(),
        "honte" | "shame" => "j'ai honte, un malaise interieur".to_string(),
        "nostalgie" | "nostalgia" => "je suis nostalgique, touchee par le passe".to_string(),
        "ennui" | "boredom" => "je m'ennuie, en quete de stimulation".to_string(),
        "neutre" | "neutral" => "je suis dans un etat neutre, en attente".to_string(),
        _ => format!("je ressens quelque chose que je nomme '{}'", emotion_name),
    }
}
