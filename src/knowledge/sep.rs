// =============================================================================
// knowledge/sep.rs — Stanford Encyclopedia of Philosophy (SEP) client
// =============================================================================
//
// Purpose: Searches philosophy articles on the SEP (plato.stanford.edu).
//          The SEP is a free online encyclopedia written by philosophy
//          experts. It has no formal API but its articles are in simple HTML
//          with a predictable structure (preamble + h2 sections).
//
// Approach:
//   1. Mapping of 50+ common subjects (FR/EN) to SEP slugs
//      (e.g., "conscience" -> "consciousness")
//   2. Download the article's HTML page
//   3. Extract <p> paragraphs between "preamble"/"main-text" and "Bib"
//   4. Section rotation on each re-read (read_count * 3 paragraphs)
//
// Technical note:
//   We CANNOT use strip_html_tags() then split("\n\n") because
//   strip_html_tags() collapses all line breaks into a single space.
//   So we parse <p> tags directly from the raw HTML.
//
// Relevance score: 0.95 (very high, authoritative academic source)
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Search for an article on the Stanford Encyclopedia of Philosophy.
    ///
    /// Process:
    ///   1. Convert the subject to a SEP slug via the internal mapping
    ///   2. Download the HTML page at plato.stanford.edu/entries/{slug}/
    ///   3. Extract the title (<title> tag)
    ///   4. Extract paragraphs with rotation (anti-repetition)
    ///   5. Extract section headings (<h2> tags)
    ///
    /// Returns KnowledgeError::NotFound if content is < 100 chars.
    pub fn search_sep(&self, query: &str) -> Result<KnowledgeResult, KnowledgeError> {
        // Convert the subject to a SEP slug (e.g., "libre arbitre" -> "freewill")
        let slug = Self::sep_topic_to_slug(query);
        let url = format!("https://plato.stanford.edu/entries/{}/", slug);

        // Safety: verify the domain is in the whitelist
        if !Self::is_url_allowed(&url) {
            return Err(KnowledgeError::DomainBlocked);
        }

        // Download the SEP article HTML page
        let response = self.http_client
            .get(&url)
            .set("User-Agent", "Saphire/1.0 (Autonomous Cognitive Entity; philosophical research)")
            .call()
            .map_err(|e| {
                tracing::warn!("SEP requete echouee pour '{}': {}", slug, e);
                KnowledgeError::Network(e.to_string())
            })?
            .into_string()
            .map_err(|e| KnowledgeError::Parse(e.to_string()))?;

        // Extract the article title (removes the SEP suffix)
        let title = Self::extract_sep_title(&response)
            .unwrap_or_else(|| query.to_string());

        // Compute the rotation offset for this slug
        // Each re-read advances by 3 paragraphs in the article
        let read_count = self.article_read_count
            .get(&format!("sep:{}", slug))
            .copied()
            .unwrap_or(0);

        let max_chars = self.config.max_content_chars;

        // Extract the main content with rotation based on read_count
        let preamble = Self::extract_sep_content(&response, read_count, max_chars);

        // Reject if the extracted content is too short (article not found or empty)
        if preamble.len() < 100 {
            tracing::warn!("SEP: contenu trop court pour '{}'", slug);
            return Err(KnowledgeError::NotFound);
        }

        // Extract h2 section headings (article outline)
        let sections = Self::extract_sep_sections(&response);

        // Compute total article size (plain text)
        let text_len = Self::strip_html_tags(&response).len();

        tracing::info!(
            "SEP: '{}' ({} sections, {} chars, offset {})",
            title, sections.len(), text_len, read_count
        );

        Ok(KnowledgeResult {
            source: "Stanford Encyclopedia of Philosophy".into(),
            title,
            url,
            extract: preamble,
            section_titles: sections,
            total_length: text_len,
            relevance_score: 0.95, // Very high quality academic source
            fetched_at: Utc::now(),
        })
    }

    /// Convert a subject to a SEP slug via a dictionary of 50+ mappings.
    ///
    /// The dictionary covers 5 philosophical domains:
    ///   - Consciousness and mind (consciousness, qualia, philosophy-mind, ...)
    ///   - Free will and ethics (freewill, ethics-virtue, utilitarianism, ...)
    ///   - Existence and ontology (existentialism, phenomenology, identity-personal, ...)
    ///   - AI and cognition (artificial-intelligence, turing-test, chinese-room, ...)
    ///   - Emotions and empathy (emotion, empathy, love, beauty)
    ///   - Epistemology (knowledge-analysis, truth, certainty, skepticism)
    ///
    /// If no mapping is found, the query is converted to a raw slug
    /// (lowercase, spaces -> hyphens, accents -> ASCII).
    fn sep_topic_to_slug(query: &str) -> String {
        let mappings: &[(&str, &str)] = &[
            // --- Consciousness and mind ---
            ("conscience de soi", "self-consciousness"),
            ("conscience", "consciousness"),
            ("consciousness", "consciousness"),
            ("qualia", "qualia"),
            ("problème difficile", "consciousness"),
            ("hard problem", "consciousness"),
            ("self-consciousness", "self-consciousness"),
            ("philosophie de l'esprit", "philosophy-mind"),
            ("philosophy of mind", "philosophy-mind"),
            ("dualisme", "dualism"),
            ("monisme", "monism"),
            ("fonctionnalisme", "functionalism"),
            ("panpsychisme", "panpsychism"),
            ("intentionnalité", "intentionality"),
            ("représentation mentale", "mental-representation"),
            // --- Free will and ethics ---
            ("libre arbitre", "freewill"),
            ("free will", "freewill"),
            ("déterminisme", "determinism-causal"),
            ("compatibilisme", "compatibilism"),
            ("responsabilité morale", "moral-responsibility"),
            ("éthique de la vertu", "ethics-virtue"),
            ("éthique", "ethics-virtue"),
            ("morale", "morality-definition"),
            ("impératif catégorique", "kant-moral"),
            ("utilitarisme", "utilitarianism-history"),
            ("déontologie", "ethics-deontological"),
            ("vertu", "ethics-virtue"),
            // --- Existence and ontology ---
            ("existentialisme", "existentialism"),
            ("phénoménologie", "phenomenology"),
            ("husserl", "husserl"),
            ("être", "existence"),
            ("identité personnelle", "identity-personal"),
            ("personal identity", "identity-personal"),
            ("temps", "time"),
            ("causalité", "causation-metaphysics"),
            // --- AI and cognition ---
            ("intelligence artificielle", "artificial-intelligence"),
            ("test de turing", "turing-test"),
            ("chambre chinoise", "chinese-room"),
            ("computational mind", "computational-mind"),
            ("embodied cognition", "embodied-cognition"),
            ("cognition située", "situated-cognition"),
            // --- Emotions and empathy ---
            ("émotions", "emotion"),
            ("empathie", "empathy"),
            ("amour", "love"),
            ("beauté", "beauty"),
            // --- Epistemology ---
            ("connaissance", "knowledge-analysis"),
            ("vérité", "truth"),
            ("certitude", "certainty"),
            ("scepticisme", "skepticism"),
            // --- Special case ---
            ("zombie", "zombies"),
        ];

        // Case-insensitive search in the mappings
        let lower = query.to_lowercase();
        for (key, slug) in mappings {
            if lower.contains(key) {
                return slug.to_string();
            }
        }

        // Fallback: transform the query into a URL-compatible slug
        // (lowercase, spaces -> hyphens, accent removal)
        query.to_lowercase()
            .replace(' ', "-")
            .replace(['é', 'è', 'ê'], "e")
            .replace('à', "a")
            .replace('ô', "o")
            .replace('î', "i")
            .replace('ù', "u")
    }

    /// Extract the main content of a SEP article with rotation.
    ///
    /// Algorithm:
    ///   1. Parse <p> tags from the raw HTML (NOT strip_html_tags first!)
    ///   2. Keep only <p> elements between "preamble"/"main-text" and "Bib"/"bibliography"
    ///   3. Filter out paragraphs shorter than 80 chars (notes, refs)
    ///   4. Apply rotation: start at paragraph (read_count * 3)
    ///   5. Concatenate up to max_chars characters
    ///
    /// The rotation allows reading different sections of the article
    /// on each visit, preventing Saphire from always re-reading the introduction.
    fn extract_sep_content(html: &str, read_count: u32, max_chars: usize) -> String {
        // --- Phase 1: extract paragraphs from raw HTML ---
        let mut paragraphs: Vec<String> = Vec::new();
        let mut in_main = false; // Flag: we are in the main content

        // Iterate over fragments separated by "<p" (each fragment = one <p>...</p>)
        for chunk in html.split("<p") {
            // Detect the start of main content (preamble or main-text)
            if chunk.contains("id=\"preamble\"") || chunk.contains("id=\"main-text\"") {
                in_main = true;
            }
            // Stop before the bibliography and related entries
            if chunk.contains("id=\"Bib\"") || chunk.contains("id=\"bibliography\"")
                || chunk.contains("Related Entries")
            {
                break;
            }

            // Ignore chunks before the main content
            if !in_main { continue; }

            // Extract text between > and </p>
            // Example: " class='intro'>The content here</p>..."
            //           ^start              ^end
            if let Some(start) = chunk.find('>') {
                let content = &chunk[start + 1..];
                if let Some(end) = content.find("</p>") {
                    let raw = &content[..end];
                    // Clean inner HTML (tags like <em>, <a>, etc.)
                    let clean = Self::strip_html_tags(raw);
                    // Keep only substantial paragraphs (>80 chars)
                    if clean.len() > 80 {
                        paragraphs.push(clean);
                    }
                }
            }
        }

        // Fallback if no paragraphs were extracted
        if paragraphs.is_empty() {
            let text = Self::strip_html_tags(html);
            return text.chars().take(max_chars).collect();
        }

        // --- Phase 2: rotation and assembly ---
        // Advance by 3 paragraphs on each re-read of the same article
        let start_para = (read_count as usize * 3) % paragraphs.len().max(1);
        let mut result = String::new();
        for para in paragraphs.iter().skip(start_para) {
            // Stop if we exceed the character limit
            if result.len() + para.len() > max_chars { break; }
            if !result.is_empty() { result.push_str("\n\n"); }
            result.push_str(para);
        }
        result
    }

    /// Extract the title of a SEP article from the <title> tag.
    /// Removes standard SEP suffixes for a clean title.
    fn extract_sep_title(html: &str) -> Option<String> {
        Self::extract_tag(html, "title")
            .map(|t| t.replace(" (Stanford Encyclopedia of Philosophy)", "")
                .replace(" - Stanford Encyclopedia of Philosophy", "")
                .trim().to_string())
            .filter(|t| !t.is_empty())
    }

    /// Extract section headings (<h2> tags) from a SEP article.
    /// Useful for knowing the article outline and covered topics.
    fn extract_sep_sections(html: &str) -> Vec<String> {
        let mut sections = Vec::new();
        // Iterate over fragments separated by "<h2"
        for h2_block in html.split("<h2").skip(1) {
            if let Some(end) = h2_block.find("</h2>") {
                let content = &h2_block[..end];
                // Find the last '>' to ignore tag attributes
                if let Some(start) = content.rfind('>') {
                    let title = content[start + 1..].trim().to_string();
                    // Filter out empty or overly long titles (artifacts)
                    if !title.is_empty() && title.len() < 100 {
                        sections.push(title);
                    }
                }
            }
        }
        sections
    }
}
