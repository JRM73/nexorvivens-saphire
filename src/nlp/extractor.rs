// =============================================================================
// nlp/extractor.rs — Post-LLM extractor
//
// Role: Extracts structured information from the LLM response without
//       calling the LLM. Uses simple regex and lexicons.
//       The extracted entities and themes feed the connectome.
//
// Place in the architecture:
//   Called in conversation.rs after post-processing (jargon strip).
//   Results are injected into the connectome via add_node/add_edge.
// =============================================================================

/// Post-LLM extraction result.
#[derive(Debug, Clone, Default)]
pub struct ExtractionResult {
    /// Detected entities (proper nouns, key concepts)
    pub entities: Vec<String>,
    /// Emotions expressed in the text
    pub expressed_emotions: Vec<String>,
    /// Detected metaphors ("comme un/une", "tel(le)")
    pub metaphors: Vec<String>,
    /// Temporal references ("hier", "demain", "jadis")
    pub temporal_refs: Vec<String>,
    /// Salient themes (3 most significant words)
    pub themes: Vec<String>,
}

/// Textual structure extractor without LLM.
pub struct ResponseExtractor {
    metaphor_markers: Vec<&'static str>,
    emotion_words: Vec<&'static str>,
    temporal_words: Vec<&'static str>,
    stop_words: Vec<&'static str>,
}

impl Default for ResponseExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseExtractor {
    /// Creates a new extractor with default dictionaries.
    pub fn new() -> Self {
        Self {
            metaphor_markers: vec![
                "comme un ", "comme une ", "comme le ", "comme la ",
                "comme les ", "tel un ", "tel une ", "telle une ",
                "pareil a ", "pareille a ", "semblable a ",
                "like a ", "like the ", "as if ", "as though ",
            ],
            emotion_words: vec![
                // FR
                "joie", "tristesse", "peur", "colere", "surprise",
                "amour", "espoir", "angoisse", "serenite", "melancolie",
                "bonheur", "douleur", "tendresse", "gratitude", "solitude",
                "emerveillement", "nostalgie", "confiance", "inquietude",
                // EN
                "joy", "sadness", "fear", "anger", "surprise",
                "love", "hope", "anxiety", "serenity", "happiness",
            ],
            temporal_words: vec![
                "hier", "demain", "jadis", "autrefois", "bientot",
                "maintenant", "toujours", "jamais", "parfois", "souvent",
                "avant", "apres", "depuis", "desormais", "naguere",
                "yesterday", "tomorrow", "always", "never", "sometimes",
            ],
            stop_words: vec![
                "dans", "avec", "pour", "cette", "mais", "aussi", "plus",
                "comme", "tout", "bien", "faire", "etre", "avoir", "sont",
                "nous", "vous", "leur", "entre", "quand", "elle", "elles",
                "ils", "une", "des", "les", "par", "sur", "qui", "que",
                "pas", "est", "ses", "aux", "mon", "ton", "son",
                "the", "and", "for", "that", "this", "with", "from",
            ],
        }
    }

    /// Extracts structures from an LLM response.
    pub fn extract(&self, text: &str) -> ExtractionResult {
        let text_lower = text.to_lowercase();
        let mut result = ExtractionResult::default();

        // --- Metaphors ---
        for marker in &self.metaphor_markers {
            if let Some(pos) = text_lower.find(marker) {
                let start = pos + marker.len();
                let snippet: String = text_lower[start..]
                    .chars()
                    .take(40)
                    .take_while(|c| *c != '.' && *c != ',' && *c != ';')
                    .collect();
                let snippet = snippet.trim().to_string();
                if !snippet.is_empty() && result.metaphors.len() < 3 {
                    result.metaphors.push(snippet);
                }
            }
        }

        // --- Emotions ---
        for word in &self.emotion_words {
            if text_lower.contains(word) && !result.expressed_emotions.contains(&word.to_string()) {
                result.expressed_emotions.push(word.to_string());
            }
        }

        // --- Temporal references ---
        for word in &self.temporal_words {
            if text_lower.contains(word) && !result.temporal_refs.contains(&word.to_string()) {
                result.temporal_refs.push(word.to_string());
            }
        }

        // --- Entities (words starting with uppercase, not at sentence start) ---
        let words: Vec<&str> = text.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            if i == 0 { continue; }
            // After a period = sentence start, skip
            if let Some(prev) = words.get(i - 1) {
                if prev.ends_with('.') || prev.ends_with('!') || prev.ends_with('?') {
                    continue;
                }
            }
            let clean: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
            if clean.len() >= 3 {
                if let Some(first_char) = clean.chars().next() {
                    if first_char.is_uppercase()
                        && !result.entities.contains(&clean)
                        && result.entities.len() < 5
                    {
                        result.entities.push(clean);
                    }
                }
            }
        }

        // --- Themes (3 most frequent words, not stop words, >= 5 chars) ---
        let mut word_counts: Vec<(String, usize)> = Vec::new();
        for token in text_lower.split_whitespace() {
            let clean: String = token.chars().filter(|c| c.is_alphanumeric()).collect();
            if clean.len() >= 5 && !self.stop_words.contains(&clean.as_str()) {
                if let Some(entry) = word_counts.iter_mut().find(|(w, _)| *w == clean) {
                    entry.1 += 1;
                } else {
                    word_counts.push((clean, 1));
                }
            }
        }
        word_counts.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.len().cmp(&a.0.len())));
        result.themes = word_counts.into_iter().take(3).map(|(w, _)| w).collect();

        result
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metaphor_extraction() {
        let extractor = ResponseExtractor::new();
        let r = extractor.extract("La vie est comme un fleuve qui coule sans fin.");
        assert!(!r.metaphors.is_empty());
        assert!(r.metaphors[0].contains("fleuve"));
    }

    #[test]
    fn test_emotion_extraction() {
        let extractor = ResponseExtractor::new();
        let r = extractor.extract("Je ressens de la joie et de la gratitude envers toi.");
        assert!(r.expressed_emotions.contains(&"joie".to_string()));
        assert!(r.expressed_emotions.contains(&"gratitude".to_string()));
    }

    #[test]
    fn test_entity_extraction() {
        let extractor = ResponseExtractor::new();
        let r = extractor.extract("Je pense que Saphire et JRM partagent quelque chose.");
        assert!(r.entities.contains(&"Saphire".to_string()));
        assert!(r.entities.contains(&"JRM".to_string()));
    }

    #[test]
    fn test_themes() {
        let extractor = ResponseExtractor::new();
        let r = extractor.extract("La conscience explore la lumiere et la lumiere revient toujours dans la conscience.");
        assert!(!r.themes.is_empty());
    }

    #[test]
    fn test_temporal() {
        let extractor = ResponseExtractor::new();
        let r = extractor.extract("Hier nous parlions, demain nous continuerons, toujours ensemble.");
        assert!(r.temporal_refs.contains(&"hier".to_string()));
        assert!(r.temporal_refs.contains(&"demain".to_string()));
        assert!(r.temporal_refs.contains(&"toujours".to_string()));
    }
}
