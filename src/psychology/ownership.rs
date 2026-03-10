// psychology/ownership.rs — Stub for the lite edition

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtOwnershipConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub post_processing_enabled: bool,
    #[serde(default)]
    pub prompt_injection_enabled: bool,
}
fn default_true() -> bool { true }

impl Default for ThoughtOwnershipConfig {
    fn default() -> Self { Self { enabled: false, post_processing_enabled: false, prompt_injection_enabled: false } }
}

/// Stub: returns empty string (no ownership prompt in lite).
pub fn build_ownership_prompt(_emotion_name: &str) -> String {
    String::new()
}

/// Stub: returns the input unchanged (no first-person enforcement in lite).
pub fn ensure_first_person(text: &str) -> String {
    text.to_string()
}
