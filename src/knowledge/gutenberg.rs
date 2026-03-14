// =============================================================================
// knowledge/gutenberg.rs — Project Gutenberg client via Gutendex API
// =============================================================================
//
// Purpose: Searches classic books (public domain) on Project Gutenberg
//          via the Gutendex API (REST, free, no API key required).
//
// Two-step process:
//   1. Search for book metadata via gutendex.com/books/?search=...
//      Returns: title, author, Gutenberg ID, available format URLs
//   2. Download the plain text (text/plain) from gutenberg.org
//      Extract a passage with rotation (anti-repetition)
//
// Text formats tried in order:
//   - text/plain; charset=utf-8
//   - text/plain
//   - text/plain; charset=us-ascii
//
// Rotation: advances by 5 paragraphs on each re-read (read_count * 5)
//           to explore different parts of the book.
//
// Relevance score: 0.85
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Search for a book on Project Gutenberg via the Gutendex API.
    ///
    /// Process:
    ///   1. Send the query to gutendex.com (FR + EN search)
    ///   2. Take the first result (most relevant)
    ///   3. Retrieve the plain text via the text/plain format URL
    ///   4. Extract a passage with anti-repetition rotation
    ///
    /// If the plain text is unavailable, returns metadata only.
    pub fn search_gutenberg(&self, query: &str) -> Result<KnowledgeResult, KnowledgeError> {
        // --- Step 1: Search via Gutendex API ---
        let encoded = Self::url_encode(query);
        let search_url = format!(
            "https://gutendex.com/books/?search={}&languages=fr,en",
            encoded
        );

        // Safety: verify the domain is in the whitelist
        if !Self::is_url_allowed(&search_url) {
            return Err(KnowledgeError::DomainBlocked);
        }

        // Send the search request
        let resp_str = self.http_client
            .get(&search_url)
            .set("User-Agent", "Saphire/1.0 (Autonomous Cognitive Entity; literary enrichment)")
            .call()
            .map_err(|e| {
                tracing::warn!("Gutenberg recherche echouee pour '{}': {}", query, e);
                KnowledgeError::Network(e.to_string())
            })?
            .into_string()
            .map_err(|e| KnowledgeError::Parse(e.to_string()))?;

        // Parse the Gutendex JSON response
        let search_resp: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| KnowledgeError::Parse(e.to_string()))?;

        let results = search_resp["results"]
            .as_array()
            .ok_or(KnowledgeError::NotFound)?;

        if results.is_empty() {
            tracing::info!("Gutenberg: aucun resultat pour '{}'", query);
            return Err(KnowledgeError::NotFound);
        }

        // Take the first result (most relevant according to Gutendex)
        let book = &results[0];
        let title = book["title"].as_str().unwrap_or("").to_string();
        let author = book["authors"][0]["name"].as_str().unwrap_or("Inconnu").to_string();
        let book_id = book["id"].as_u64().unwrap_or(0);

        // --- Step 2: Retrieve a plain text extract ---
        // Look for the text URL in the available formats
        // Try 3 content-type variants
        let text_url = book["formats"]["text/plain; charset=utf-8"]
            .as_str()
            .or_else(|| book["formats"]["text/plain"].as_str())
            .or_else(|| book["formats"]["text/plain; charset=us-ascii"].as_str());

        let extract = if let Some(url) = text_url {
            // Verify the text domain is allowed
            if !Self::is_url_allowed(url) {
                tracing::warn!("Gutenberg: domaine du texte non autorise: {}", url);
                format!("Livre : {} par {} (texte non accessible)", title, author)
            } else {
                // Download the plain text
                match self.http_client
                    .get(url)
                    .set("User-Agent", "Saphire/1.0 (Autonomous Cognitive Entity; literary enrichment)")
                    .call()
                {
                    Ok(resp) => {
                        let text = resp.into_string().unwrap_or_default();
                        // Extract a passage with rotation based on the read counter
                        let read_count = self.article_read_count
                            .get(&format!("gutenberg:{}", book_id))
                            .copied()
                            .unwrap_or(0);
                        Self::select_book_passage(&text, read_count, self.config.max_content_chars)
                    }
                    Err(e) => {
                        tracing::warn!("Gutenberg: texte inaccessible pour '{}': {}", title, e);
                        format!("Livre : {} par {}", title, author)
                    }
                }
            }
        } else {
            // No plain text format available for this book
            format!("Livre : {} par {} (pas de texte brut disponible)", title, author)
        };

        // Gutenberg page URL for reference
        let page_url = format!("https://www.gutenberg.org/ebooks/{}", book_id);

        tracing::info!("Gutenberg: '{}' par {} (id: {})", title, author, book_id);

        Ok(KnowledgeResult {
            source: format!("Gutenberg — {}", author),
            title: format!("{} ({})", title, author),
            url: page_url,
            extract,
            section_titles: vec![], // Books have no indexed sections
            total_length: 0,        // Not computed (text too long)
            relevance_score: 0.85,
            fetched_at: Utc::now(),
        })
    }

    /// Select an interesting passage from a Gutenberg book.
    ///
    /// Algorithm:
    ///   1. Skip the Gutenberg header ("*** START OF THE PROJECT GUTENBERG...")
    ///   2. Skip the Gutenberg footer ("*** END OF THE PROJECT GUTENBERG...")
    ///   3. Split the text into paragraphs (separated by double line breaks)
    ///   4. Filter paragraphs shorter than 100 chars (titles, numbers)
    ///   5. Apply rotation: start at paragraph (read_count * 5)
    ///   6. Concatenate up to max_chars characters
    ///
    /// The factor of 5 (vs 3 for SEP) allows covering more text
    /// since books are much longer than SEP articles.
    fn select_book_passage(text: &str, read_count: u32, max_chars: usize) -> String {
        // Find the start of the actual content (after the Gutenberg header)
        let content_start = text.find("*** START OF")
            .and_then(|pos| text[pos..].find('\n').map(|n| pos + n + 1))
            .unwrap_or(0);

        // Find the end of the actual content (before the Gutenberg footer)
        let content_end = text.find("*** END OF")
            .unwrap_or(text.len());

        let clean_text = &text[content_start..content_end];

        // Split into meaningful paragraphs (>100 chars)
        let paragraphs: Vec<&str> = clean_text.split("\n\n")
            .map(|p| p.trim())
            .filter(|p| p.len() > 100)
            .collect();

        // Fallback if not enough paragraphs
        if paragraphs.is_empty() {
            return clean_text.chars().take(max_chars).collect();
        }

        // Rotation: advance by 5 paragraphs on each read
        // Uses modulo to wrap around when reaching the end
        let start = (read_count as usize * 5) % paragraphs.len().max(1);
        let mut result = String::new();
        for para in paragraphs.iter().skip(start) {
            if result.len() + para.len() > max_chars { break; }
            if !result.is_empty() { result.push_str("\n\n"); }
            result.push_str(para);
        }
        result
    }
}
