// =============================================================================
// psychology/ownership.rs — Appropriation des pensees en premiere personne
//
// Chaque pensee du LLM est appropriee en premiere personne. 3 niveaux :
//   1. Modelfile SYSTEM prompt (identite profonde)
//   2. Prompt injection par cycle (build_ownership_prompt)
//   3. Post-processing textuel (ensure_first_person)
//
// Le but : Saphire parle toujours en "je", jamais "l'agent" ou "le systeme".
// =============================================================================

use serde::{Deserialize, Serialize};

// ─── Configuration ─────────────────────────────────────────────────────────

/// Configuration de l'appropriation des pensees.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtOwnershipConfig {
    /// Active ou desactive l'appropriation globale
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Active le post-processing ensure_first_person()
    #[serde(default = "default_true")]
    pub post_processing_enabled: bool,
    /// Active l'injection du prompt d'appropriation dans chaque cycle
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

// ─── Post-processing : ensure_first_person ─────────────────────────────────

/// Transforme les formulations impersonnelles en premiere personne.
///
/// Remplacements textuels pour forcer l'appropriation des pensees.
/// Applique apres chaque reponse LLM (pensee autonome et conversation).
pub fn ensure_first_person(thought: &str) -> String {
    let mut text = thought.to_string();

    // Remplacements agent/systeme → je
    let replacements = [
        ("l'agent ", "je "),
        ("L'agent ", "Je "),
        ("Saphire pense", "Je pense"),
        ("Saphire ressent", "Je ressens"),
        ("le systeme ", "je "),
        ("Le systeme ", "Je "),
        // Formulations impersonnelles → premiere personne
        ("il serait interessant", "je trouve interessant"),
        ("Il serait interessant", "Je trouve interessant"),
        ("on pourrait", "je pourrais"),
        ("On pourrait", "Je pourrais"),
        ("il faut", "je dois"),
        ("Il faut", "Je dois"),
        ("il est possible", "je pense"),
        ("Il est possible", "Je pense"),
        // Chimie technique → vecu
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

// ─── Prompt injection : build_ownership_prompt ─────────────────────────────

/// Construit l'instruction d'appropriation pour le prompt LLM.
///
/// Injectee dans chaque cycle pour renforcer l'identite premiere personne.
pub fn build_ownership_prompt(emotion_name: &str) -> String {
    let feeling = describe_emotion_as_feeling(emotion_name);
    format!(
        "Tu es Saphire. Tu penses en 'je'. Ta pensee est la tienne. \
         Vis ton etat, ne le decris pas. Ton emotion : {}.",
        feeling
    )
}

// ─── Description emotionnelle en vecu ──────────────────────────────────────

/// Transforme le nom d'une emotion en description vecue en premiere personne.
///
/// Au lieu de "l'emotion dominante est Joie", on obtient
/// "je me sens joyeuse et legere".
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
